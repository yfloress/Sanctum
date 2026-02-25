// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

//! Crypto tax UI callbacks (IPC import + summary)

use crate::controller::{AppController, SETTING_PREFERRED_CURRENCY};
use crate::features::crypto::tax::types::{TaxJurisdiction, TaxMethod};
use crate::services::i18n::{t, t_args};
use crate::ui::{convert_usd_to_preferred, format_preferred};
use crate::{
    AppWindow, CryptoAdapter, SettingsAdapter, TaxReadinessItem, TaxWalletEntry, Translations,
};
use slint::platform::Clipboard;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

/// Resolves the display currency and USD/target exchange rate from the controller.
/// Chile tax views are always displayed in CLP.
fn resolve_display_currency(
    controller: &AppController,
    jurisdiction: TaxJurisdiction,
) -> Result<(String, f64), String> {
    let preferred = if matches!(jurisdiction, TaxJurisdiction::Chile) {
        "CLP".to_string()
    } else {
        controller
            .get_app_setting(SETTING_PREFERRED_CURRENCY)
            .unwrap_or_else(|_| "USD".to_string())
            .trim()
            .to_uppercase()
    };
    let preferred = if preferred.is_empty() {
        "USD".to_string()
    } else {
        preferred
    };
    if preferred == "USD" {
        return Ok(("USD".to_string(), 1.0));
    }

    let pair = format!("{}_USD", preferred.as_str());
    let rate = controller
        .load_exchange_rate_allow_stale(pair)
        .ok()
        .and_then(|r| r.map(|(value, _)| value))
        .filter(|value| *value > 0.0);

    if matches!(jurisdiction, TaxJurisdiction::Chile) {
        if let Some(value) = rate {
            return Ok((preferred, value));
        }
        return Err(
            "CLP tax display requires a valid USD/CLP rate. Please sync prices first.".to_string(),
        );
    }

    Ok((preferred, rate.unwrap_or(1.0)))
}

/// Formats a USD-denominated float amount in the selected display currency.
fn fmt_preferred(amount_usd: f64, currency: &str, clp_rate: f64) -> String {
    let converted = convert_usd_to_preferred(amount_usd, currency, clp_rate);
    format_preferred(converted, currency)
}

/// Formats a USD-denominated float amount with a sign prefix in the selected display currency.
fn fmt_preferred_signed(amount_usd: f64, currency: &str, clp_rate: f64) -> String {
    let converted = convert_usd_to_preferred(amount_usd, currency, clp_rate);
    let abs = if converted < 0.0 {
        -converted
    } else {
        converted
    };
    let formatted = format_preferred(abs, currency);
    if converted < 0.0 {
        format!("- {}", formatted)
    } else {
        format!("+ {}", formatted)
    }
}

/// Interprets input as tax year (AT) and returns the period year used by the
/// engine. For Chile, AT maps to commercial year (AT - 1).
fn effective_period_id(raw: &str, jurisdiction: TaxJurisdiction) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(t("crypto-tax-period-required"));
    }
    if trimmed.len() != 4 {
        return Err("Tax period must be a 4-digit year".to_string());
    }
    let year: i32 = trimmed
        .parse()
        .map_err(|_| "Tax period must be a valid year".to_string())?;
    if matches!(jurisdiction, TaxJurisdiction::Chile) {
        year.checked_sub(1)
            .map(|value| value.to_string())
            .ok_or_else(|| "Tax period must be greater than 0000".to_string())
    } else {
        Ok(year.to_string())
    }
}

/// Returns the year label used in export file names.
/// For Chile, UI tax period (AT) is one year ahead of the engine period.
fn export_period_label(effective_period: &str, jurisdiction: TaxJurisdiction) -> String {
    if matches!(jurisdiction, TaxJurisdiction::Chile) {
        return effective_period
            .parse::<i32>()
            .ok()
            .and_then(|y| y.checked_add(1))
            .map(|y| y.to_string())
            .unwrap_or_else(|| effective_period.to_string());
    }
    effective_period.to_string()
}

static TAX_PRICE_SYNC_IN_FLIGHT: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static TAX_HISTORICAL_PRICE_CACHE: LazyLock<Mutex<HashMap<String, f64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Copy, PartialEq, Eq)]
enum MissingPriceField {
    PricePerCoin,
    FeeUsd,
    SwapOverrideProceeds,
}

#[derive(Clone)]
struct MissingPriceRequest {
    tx_id: String,
    coin_id: String,
    date: String,
    field: MissingPriceField,
    amount: Option<f64>,
    fee_amount: Option<f64>,
}

fn try_start_tax_price_sync() -> bool {
    let Ok(mut in_flight) = TAX_PRICE_SYNC_IN_FLIGHT.lock() else {
        return false;
    };
    if *in_flight {
        return false;
    }
    *in_flight = true;
    true
}

fn finish_tax_price_sync(ui_weak: &Weak<AppWindow>) {
    if let Ok(mut in_flight) = TAX_PRICE_SYNC_IN_FLIGHT.lock() {
        *in_flight = false;
    }

    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
        ui.global::<CryptoAdapter>().set_tax_price_syncing(false);
    });
}

fn get_cached_tax_historical_price(cache_key: &str) -> Option<f64> {
    let Ok(cache) = TAX_HISTORICAL_PRICE_CACHE.lock() else {
        return None;
    };
    cache.get(cache_key).copied()
}

fn cache_tax_historical_price(cache_key: &str, price: f64) {
    if !price.is_finite() || price <= 0.0 {
        return;
    }
    if let Ok(mut cache) = TAX_HISTORICAL_PRICE_CACHE.lock() {
        cache.insert(cache_key.to_string(), price);
    }
}

fn is_usd_stablecoin_coin_id(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "usd"
            | "usdt"
            | "usdc"
            | "busd"
            | "dai"
            | "tusd"
            | "fdusd"
            | "usdd"
            | "usdp"
            | "pyusd"
            | "ust"
            | "frax"
            | "tether"
            | "usd-coin"
            | "binance-usd"
            | "true-usd"
            | "first-digital-usd"
            | "pax-dollar"
            | "paypal-usd"
            | "terrausd"
    )
}

fn normalize_history_date(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let date_part = if trimmed.len() >= 10 {
        &trimmed[..10]
    } else {
        trimmed
    };

    chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
        .or_else(|_| chrono::NaiveDate::parse_from_str(trimmed, "%d-%m-%Y"))
        .ok()
        .map(|date| date.format("%Y-%m-%d").to_string())
}

fn collect_missing_price_requests(
    controller: &AppController,
    warnings: &[crate::features::crypto::TaxWarning],
) -> Vec<MissingPriceRequest> {
    let mut requests = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for warning in warnings {
        let code = warning.code.as_str();
        if !matches!(
            code,
            "missing_price" | "swap_missing_price" | "income_missing_price" | "fee_missing_price"
        ) {
            continue;
        }

        let Some(tx_id) = warning.tx_id.as_deref() else {
            continue;
        };

        let Ok(Some(tx)) = controller.get_crypto_transaction(tx_id.to_string()) else {
            continue;
        };
        let Some(date) = normalize_history_date(&tx.date) else {
            continue;
        };

        match code {
            "fee_missing_price" => {
                let (Some(fee_coin_id), Some(fee_amount)) = (tx.fee_coin_id.clone(), tx.fee_amount)
                else {
                    continue;
                };
                if fee_amount <= 0.0 || tx.fee.unwrap_or(0.0) > 0.0 {
                    continue;
                }
                let request_key = format!("fee:{}:{}:{}", tx.id, fee_coin_id, date);
                if seen.insert(request_key) {
                    requests.push(MissingPriceRequest {
                        tx_id: tx.id,
                        coin_id: fee_coin_id,
                        date,
                        field: MissingPriceField::FeeUsd,
                        amount: None,
                        fee_amount: Some(fee_amount),
                    });
                }
            }
            "swap_missing_price" => {
                if tx.override_proceeds.unwrap_or(0.0) > 0.0 {
                    continue;
                }
                let request_key = format!("swap:{}:{}:{}", tx.id, tx.coin_id, date);
                if seen.insert(request_key) {
                    requests.push(MissingPriceRequest {
                        tx_id: tx.id,
                        coin_id: tx.coin_id,
                        date,
                        field: MissingPriceField::SwapOverrideProceeds,
                        amount: Some(tx.amount),
                        fee_amount: None,
                    });
                }
            }
            _ => {
                if tx.price_per_coin.unwrap_or(0.0) > 0.0 {
                    continue;
                }
                let request_key = format!("price:{}:{}:{}", tx.id, tx.coin_id, date);
                if seen.insert(request_key) {
                    requests.push(MissingPriceRequest {
                        tx_id: tx.id,
                        coin_id: tx.coin_id,
                        date,
                        field: MissingPriceField::PricePerCoin,
                        amount: Some(tx.amount),
                        fee_amount: None,
                    });
                }
            }
        }
    }

    requests
}

fn infer_swap_unit_price_from_pair(
    controller: &AppController,
    tx_id: &str,
    target_coin_id: &str,
) -> Option<f64> {
    let tx = controller
        .get_crypto_transaction(tx_id.to_string())
        .ok()
        .flatten()?;
    if tx.subtype.as_deref() != Some("swap") {
        return None;
    }

    if tx.coin_id == target_coin_id
        && let Some(price) = tx.price_per_coin
        && price.is_finite()
        && price > 0.0
    {
        return Some(price);
    }
    if tx.coin_id == target_coin_id
        && let Some(proceeds) = tx.override_proceeds
        && tx.amount > 0.0
    {
        let price = proceeds / tx.amount;
        if price.is_finite() && price > 0.0 {
            return Some(price);
        }
    }

    let related_id = tx.related_tx_id.as_deref()?;
    let related = controller
        .get_crypto_transaction(related_id.to_string())
        .ok()
        .flatten()?;

    if related.coin_id == target_coin_id
        && let Some(price) = related.price_per_coin
        && price.is_finite()
        && price > 0.0
    {
        return Some(price);
    }
    if related.coin_id == target_coin_id
        && let Some(proceeds) = related.override_proceeds
        && related.amount > 0.0
    {
        let price = proceeds / related.amount;
        if price.is_finite() && price > 0.0 {
            return Some(price);
        }
    }

    let tx_stable = is_usd_stablecoin_coin_id(&tx.coin_id);
    let related_stable = is_usd_stablecoin_coin_id(&related.coin_id);

    if tx_stable
        && !related_stable
        && related.coin_id == target_coin_id
        && related.amount > 0.0
        && tx.amount > 0.0
    {
        let price = tx.amount / related.amount;
        if price.is_finite() && price > 0.0 {
            return Some(price);
        }
    }

    if related_stable
        && !tx_stable
        && tx.coin_id == target_coin_id
        && tx.amount > 0.0
        && related.amount > 0.0
    {
        let price = related.amount / tx.amount;
        if price.is_finite() && price > 0.0 {
            return Some(price);
        }
    }

    None
}

async fn resolve_historical_unit_price(
    controller: &Arc<AppController>,
    request: &MissingPriceRequest,
) -> Result<f64, String> {
    if is_usd_stablecoin_coin_id(&request.coin_id) {
        return Ok(1.0);
    }

    let cache_key = format!("{}|{}", request.coin_id, request.date);
    if let Some(price) = get_cached_tax_historical_price(&cache_key) {
        return Ok(price);
    }

    let price = controller
        .get_crypto_historical_price_usd(request.coin_id.clone(), request.date.clone())
        .await
        .map_err(|err| err.to_string())?;

    if !price.is_finite() || price <= 0.0 {
        return Err("Historical price is invalid".to_string());
    }

    cache_tax_historical_price(&cache_key, price);
    Ok(price)
}

fn start_tax_missing_price_sync<N>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
    user_initiated: bool,
) where
    N: Fn(String, bool) + Clone + Send + 'static,
{
    let Some(ui) = ui_weak.upgrade() else {
        return;
    };

    let adapter = ui.global::<CryptoAdapter>();
    let period_raw = adapter.get_tax_period();
    let jurisdiction = TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction());
    let period = match effective_period_id(&period_raw, jurisdiction) {
        Ok(value) => value,
        Err(err) => {
            if user_initiated {
                notify(err, true);
            }
            return;
        }
    };

    if !try_start_tax_price_sync() {
        if user_initiated {
            notify(t("crypto-tax-price-sync-running"), true);
        }
        return;
    }

    adapter.set_tax_price_syncing(true);
    let controller_async = controller.clone();
    let ui_weak_async = ui_weak.clone();
    let notify_async = notify.clone();

    tokio::spawn(async move {
        let summary = match controller_async.generate_tax_summary(period.clone()) {
            Ok(value) => value,
            Err(err) => {
                if user_initiated {
                    let notify_for_ui = notify_async.clone();
                    let message = format!("Failed to load tax warnings: {}", err);
                    let _ = ui_weak_async.upgrade_in_event_loop(move |_| {
                        notify_for_ui(message, true);
                    });
                }
                finish_tax_price_sync(&ui_weak_async);
                return;
            }
        };

        let requests = collect_missing_price_requests(&controller_async, &summary.report.warnings);
        if requests.is_empty() {
            if user_initiated {
                let notify_for_ui = notify_async.clone();
                let _ = ui_weak_async.upgrade_in_event_loop(move |_| {
                    notify_for_ui(t("crypto-tax-price-sync-no-missing"), false);
                });
            }
            finish_tax_price_sync(&ui_weak_async);
            return;
        }

        let mut updated = 0usize;
        let mut unresolved = 0usize;

        for request in requests {
            let inferred_price = if matches!(
                request.field,
                MissingPriceField::FeeUsd | MissingPriceField::SwapOverrideProceeds
            ) {
                infer_swap_unit_price_from_pair(&controller_async, &request.tx_id, &request.coin_id)
            } else {
                None
            };

            let unit_price = match inferred_price {
                Some(value) => value,
                None => match resolve_historical_unit_price(&controller_async, &request).await {
                    Ok(value) => value,
                    Err(_) => {
                        unresolved += 1;
                        continue;
                    }
                },
            };

            let (price_per_coin, fee_usd, override_proceeds) = match request.field {
                MissingPriceField::PricePerCoin => (Some(unit_price), None, None),
                MissingPriceField::FeeUsd => {
                    let fee_amount = request.fee_amount.unwrap_or(0.0);
                    if fee_amount <= 0.0 {
                        unresolved += 1;
                        continue;
                    }
                    let fee_value = fee_amount * unit_price;
                    if !fee_value.is_finite() || fee_value < 0.0 {
                        unresolved += 1;
                        continue;
                    }
                    (None, Some(fee_value), None)
                }
                MissingPriceField::SwapOverrideProceeds => {
                    let amount = request.amount.unwrap_or(0.0);
                    if amount <= 0.0 {
                        unresolved += 1;
                        continue;
                    }
                    let proceeds = amount * unit_price;
                    if !proceeds.is_finite() || proceeds <= 0.0 {
                        unresolved += 1;
                        continue;
                    }
                    (None, None, Some(proceeds))
                }
            };

            match controller_async.fill_missing_tax_price_fields(
                request.tx_id,
                price_per_coin,
                fee_usd,
                override_proceeds,
            ) {
                Ok(true) => updated += 1,
                Ok(false) => {}
                Err(_) => unresolved += 1,
            }
        }

        if updated > 0
            && let Ok(refreshed_summary) = controller_async.generate_tax_summary(period.clone())
        {
            let report_jurisdiction =
                TaxJurisdiction::parse_or_default(&refreshed_summary.report.jurisdiction);
            if let Ok((currency, clp_rate)) =
                resolve_display_currency(&controller_async, report_jurisdiction)
            {
                let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                    let adapter = ui.global::<CryptoAdapter>();
                    update_report_state(&adapter, &refreshed_summary.report, &currency, clp_rate);
                    update_summary_state(&adapter, &refreshed_summary, &currency, clp_rate);
                });
            }
        }

        if user_initiated {
            if updated > 0 {
                let notify_for_ui = notify_async.clone();
                let count = updated.to_string();
                let message = t_args(
                    "crypto-tax-price-sync-finished",
                    &[("count", count.as_str())],
                );
                let _ = ui_weak_async.upgrade_in_event_loop(move |_| {
                    notify_for_ui(message, false);
                });
            }
            if unresolved > 0 {
                let notify_for_ui = notify_async.clone();
                let count = unresolved.to_string();
                let message = t_args(
                    "crypto-tax-price-sync-unresolved",
                    &[("count", count.as_str())],
                );
                let _ = ui_weak_async.upgrade_in_event_loop(move |_| {
                    notify_for_ui(message, true);
                });
            }
        }

        finish_tax_price_sync(&ui_weak_async);
    });
}

pub fn setup_tax_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + Send + 'static,
{
    // Load tax settings (IPC summary)
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>().on_load_tax_settings(move || {
            load_tax_settings(&ui_weak, &controller);
            update_tax_wallet_list(&ui_weak, &controller);
            update_ipc_summary(&ui_weak, &controller, None::<&fn(String, bool)>);
        });
    }

    // Import IPC CSV
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_import_ipc_csv(move || {
            let file_path = rfd::FileDialog::new()
                .add_filter("CSV", &["csv"])
                .pick_file();

            let Some(path) = file_path else {
                return;
            };

            let content = match std::fs::read_to_string(&path) {
                Ok(data) => data,
                Err(err) => {
                    notify(format!("Failed to read IPC file: {}", err), true);
                    return;
                }
            };

            match controller.import_ipc_csv(&content) {
                Ok(summary) => {
                    update_ipc_summary(&ui_weak, &controller, Some(&notify));
                    let msg = t_args(
                        "crypto-tax-ipc-import-success",
                        &[
                            ("count", summary.inserted.to_string().as_str()),
                            ("first", summary.first_period.as_deref().unwrap_or("")),
                            ("last", summary.last_period.as_deref().unwrap_or("")),
                        ],
                    );
                    notify(msg, false);
                }
                Err(err) => {
                    notify(format!("IPC import failed: {}", err), true);
                }
            }
        });
    }

    // Copy IPC source URL to clipboard
    {
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>().on_copy_ipc_url(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let url = ui.global::<Translations>().get_crypto_tax_ipc_source_url();
            let _ = i_slint_backend_selector::with_platform(|platform| {
                platform.set_clipboard_text(&url, Clipboard::DefaultClipboard);
                Ok(())
            });
        });
    }

    // Resolve missing tax prices (manual/auto)
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>()
            .on_sync_tax_missing_prices(move |user_initiated| {
                start_tax_missing_price_sync(&ui_weak, &controller, notify.clone(), user_initiated);
            });
    }

    // Generate tax report
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>()
            .on_generate_tax_report(move || {
                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };

                let adapter = ui.global::<CryptoAdapter>();
                let period_raw = adapter.get_tax_period();
                let jurisdiction =
                    TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction());
                let period = match effective_period_id(&period_raw, jurisdiction) {
                    Ok(value) => value,
                    Err(err) => {
                        notify(err, true);
                        return;
                    }
                };

                match controller.generate_tax_summary(period) {
                    Ok(summary) => {
                        let report_jurisdiction =
                            TaxJurisdiction::parse_or_default(&summary.report.jurisdiction);
                        let (currency, clp_rate) =
                            match resolve_display_currency(&controller, report_jurisdiction) {
                                Ok(value) => value,
                                Err(err) => {
                                    notify(err, true);
                                    return;
                                }
                            };
                        update_report_state(&adapter, &summary.report, &currency, clp_rate);
                        update_summary_state(&adapter, &summary, &currency, clp_rate);
                        notify(t("crypto-tax-report-generated"), false);

                        if ui.global::<SettingsAdapter>().get_auto_fetch_enabled()
                            && adapter.get_tax_can_resolve_prices()
                            && !adapter.get_tax_price_syncing()
                        {
                            adapter.invoke_sync_tax_missing_prices(false);
                        }
                    }
                    Err(err) => {
                        notify(format!("Failed to generate report: {}", err), true);
                    }
                }
            });
    }

    // Export tax report CSV
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_export_tax_report(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let adapter = ui.global::<CryptoAdapter>();
            let period_raw = adapter.get_tax_period();
            let jurisdiction = TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction());
            let period = match effective_period_id(&period_raw, jurisdiction) {
                Ok(value) => value,
                Err(err) => {
                    notify(err, true);
                    return;
                }
            };
            let period_label = export_period_label(&period, jurisdiction);

            let file_name = format!("sanctum-crypto-capital-gains-{}.csv", period_label);
            let file_path = rfd::FileDialog::new()
                .set_file_name(&file_name)
                .add_filter("CSV", &["csv"])
                .save_file();

            let Some(path) = file_path else {
                return;
            };

            let path_str = path.to_string_lossy().to_string();
            match controller.export_tax_report_csv(period, path_str) {
                Ok(_) => notify(t("crypto-tax-report-exported"), false),
                Err(err) => notify(format!("Failed to export report: {}", err), true),
            }
        });
    }

    // Export tax transaction history CSV
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_export_tax_history(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let adapter = ui.global::<CryptoAdapter>();
            let period_raw = adapter.get_tax_period();
            let jurisdiction = TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction());
            let period = match effective_period_id(&period_raw, jurisdiction) {
                Ok(value) => value,
                Err(err) => {
                    notify(err, true);
                    return;
                }
            };
            let period_label = export_period_label(&period, jurisdiction);

            let file_name = format!("sanctum-crypto-transaction-history-{}.csv", period_label);
            let file_path = rfd::FileDialog::new()
                .set_file_name(&file_name)
                .add_filter("CSV", &["csv"])
                .save_file();

            let Some(path) = file_path else {
                return;
            };

            let path_str = path.to_string_lossy().to_string();
            match controller.export_tax_history_csv(period, path_str) {
                Ok(_) => notify(t("crypto-tax-report-exported"), false),
                Err(err) => notify(format!("Failed to export history: {}", err), true),
            }
        });
    }

    // Toggle wallet exclusion
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>()
            .on_toggle_tax_wallet_exclusion(move |wallet_id| {
                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };

                let adapter = ui.global::<CryptoAdapter>();
                let period_raw = adapter.get_tax_period();
                let jurisdiction =
                    TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction());
                let period = match effective_period_id(&period_raw, jurisdiction) {
                    Ok(value) => value,
                    Err(_) => {
                        return;
                    }
                };

                let mut settings =
                    controller
                        .load_tax_settings(period.clone())
                        .unwrap_or_else(|_| {
                            crate::features::crypto::TaxPeriodSettings::defaults_for(&period)
                        });

                let wid = wallet_id.to_string();
                if let Some(pos) = settings
                    .excluded_wallet_ids
                    .iter()
                    .position(|id| id == &wid)
                {
                    settings.excluded_wallet_ids.remove(pos);
                } else {
                    settings.excluded_wallet_ids.push(wid);
                }

                if let Err(err) = controller.save_tax_settings(settings) {
                    notify(format!("Failed to save wallet exclusion: {}", err), true);
                }

                // Update the specific item in-place so Slint sees a property
                // transition and fires the toggle animation (rebuilding the
                // whole model would recreate the for-loop items, skipping it).
                let model = adapter.get_tax_wallet_list();
                for i in 0..model.row_count() {
                    if let Some(mut entry) = model.row_data(i)
                        && entry.id == wallet_id
                    {
                        entry.excluded = !entry.excluded;
                        model.set_row_data(i, entry);
                        break;
                    }
                }
            });
    }

    // Save tax settings
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_save_tax_settings(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let adapter = ui.global::<CryptoAdapter>();
            let jurisdiction = TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction());
            let period_raw = adapter.get_tax_period();
            let period = match effective_period_id(&period_raw, jurisdiction) {
                Ok(value) => value,
                Err(err) => {
                    notify(err, true);
                    return;
                }
            };

            // Preserve current excluded_wallet_ids from the already-saved settings.
            let existing_excluded = controller
                .load_tax_settings(period.clone())
                .map(|s| s.excluded_wallet_ids)
                .unwrap_or_default();

            let settings = crate::features::crypto::TaxPeriodSettings {
                period_id: period,
                jurisdiction,
                method: TaxMethod::parse_or_default(&adapter.get_tax_method()),
                include_swaps: adapter.get_tax_include_swaps(),
                include_fee_crypto: adapter.get_tax_include_fee_crypto(),
                excluded_wallet_ids: existing_excluded,
            };

            match controller.save_tax_settings(settings) {
                Ok(_) => notify(t("crypto-tax-settings-saved"), false),
                Err(err) => notify(format!("Failed to save tax settings: {}", err), true),
            }
        });
    }
}

fn update_tax_wallet_list(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>) {
    let wallets = controller.get_wallets().unwrap_or_default();

    let Some(ui) = ui_weak.upgrade() else {
        return;
    };
    let adapter = ui.global::<CryptoAdapter>();
    let jurisdiction = TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction());
    let period_raw = adapter.get_tax_period();
    let period = effective_period_id(&period_raw, jurisdiction).unwrap_or_default();

    let excluded = if !period.is_empty() {
        controller
            .load_tax_settings(period)
            .map(|s| s.excluded_wallet_ids)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let entries: Vec<TaxWalletEntry> = wallets
        .iter()
        .map(|w| TaxWalletEntry {
            id: SharedString::from(w.id.as_str()),
            name: SharedString::from(w.name.as_str()),
            category: SharedString::from(w.category.as_str()),
            excluded: excluded.contains(&w.id),
        })
        .collect();

    adapter.set_tax_wallet_list(ModelRc::new(VecModel::from(entries)));
}

fn update_ipc_summary<N>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: Option<&N>,
) where
    N: Fn(String, bool),
{
    let summary = match controller.get_ipc_summary() {
        Ok(value) => value,
        Err(err) => {
            if let Some(n) = notify {
                n(format!("Failed to load IPC data: {}", err), true);
            }
            None
        }
    };

    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<CryptoAdapter>();
        let (text, loaded) = if let Some(summary) = summary {
            let t = t_args(
                "crypto-tax-ipc-summary",
                &[
                    ("count", summary.count.to_string().as_str()),
                    ("first", summary.first_period.as_str()),
                    ("last", summary.last_period.as_str()),
                ],
            );
            (t, summary.count > 0)
        } else {
            (t("crypto-tax-ipc-empty"), false)
        };

        adapter.set_tax_ipc_summary(SharedString::from(text));
        adapter.set_tax_ipc_loaded(loaded);
    }
}

fn update_report_state(
    adapter: &CryptoAdapter,
    report: &crate::features::crypto::TaxReport,
    currency: &str,
    clp_rate: f64,
) {
    let proceeds = fmt_preferred(report.summary.total_proceeds, currency, clp_rate);
    let cost = fmt_preferred(report.summary.total_cost, currency, clp_rate);
    let gain = fmt_preferred_signed(report.summary.total_gain, currency, clp_rate);
    let disposals = report.summary.disposals.to_string();

    // Populate gain polarity for semantic coloring
    adapter.set_tax_gain_positive(report.summary.total_gain >= 0.0);
    adapter.set_tax_summary_disposals(SharedString::from(&disposals));

    // Populate short/long term gains for USA
    let summary = if let (Some(short), Some(long)) = (
        report.summary.short_term_gain,
        report.summary.long_term_gain,
    ) {
        let short_fmt = fmt_preferred_signed(short, currency, clp_rate);
        let long_fmt = fmt_preferred_signed(long, currency, clp_rate);
        adapter.set_tax_summary_short_gain(SharedString::from(&short_fmt));
        adapter.set_tax_summary_long_gain(SharedString::from(&long_fmt));
        t_args(
            "crypto-tax-report-summary-us",
            &[
                ("disposals", disposals.as_str()),
                ("proceeds", proceeds.as_str()),
                ("cost", cost.as_str()),
                ("gain", gain.as_str()),
                ("short", short_fmt.as_str()),
                ("long", long_fmt.as_str()),
            ],
        )
    } else {
        adapter.set_tax_summary_short_gain(SharedString::from("--"));
        adapter.set_tax_summary_long_gain(SharedString::from("--"));
        t_args(
            "crypto-tax-report-summary",
            &[
                ("disposals", disposals.as_str()),
                ("proceeds", proceeds.as_str()),
                ("cost", cost.as_str()),
                ("gain", gain.as_str()),
            ],
        )
    };

    let warning_text = if report.warnings.is_empty() {
        t("crypto-tax-report-warnings-empty")
    } else {
        let count = report.warnings.len().to_string();
        t_args(
            "crypto-tax-report-warnings-count",
            &[("count", count.as_str())],
        )
    };

    adapter.set_tax_report_summary(SharedString::from(summary));
    adapter.set_tax_report_warnings(SharedString::from(warning_text));
}

fn update_summary_state(
    adapter: &CryptoAdapter,
    summary: &crate::features::crypto::TaxSummaryPayload,
    currency: &str,
    clp_rate: f64,
) {
    let proceeds = fmt_preferred(summary.report.summary.total_proceeds, currency, clp_rate);
    let cost = fmt_preferred(summary.report.summary.total_cost, currency, clp_rate);
    let gain = fmt_preferred_signed(summary.report.summary.total_gain, currency, clp_rate);
    let income = fmt_preferred(summary.taxable_income_total, currency, clp_rate);
    let balance = match summary.end_balance_value {
        Some(value) if summary.end_balance_missing == 0 => fmt_preferred(value, currency, clp_rate),
        _ => "N/A".to_string(),
    };
    let period_text = format!(
        "{}: {} -> {}",
        summary.report.period_id, summary.report.period_start, summary.report.period_end
    );
    let warnings_text = if summary.report.warnings.is_empty() {
        "".to_string()
    } else {
        let count = summary.report.warnings.len().to_string();
        t_args(
            "crypto-tax-report-warnings-count",
            &[("count", count.as_str())],
        )
    };

    adapter.set_tax_summary_ready(true);
    adapter.set_tax_summary_period(SharedString::from(period_text));
    adapter.set_tax_summary_proceeds(SharedString::from(proceeds));
    adapter.set_tax_summary_cost(SharedString::from(cost));
    adapter.set_tax_summary_gain(SharedString::from(gain));
    adapter.set_tax_summary_income(SharedString::from(income));
    adapter.set_tax_summary_balance(SharedString::from(balance));
    adapter.set_tax_summary_transactions(SharedString::from(
        summary.transactions_in_period.to_string(),
    ));
    adapter.set_tax_summary_volume(SharedString::from(fmt_preferred(
        summary.volume_processed,
        currency,
        clp_rate,
    )));
    adapter.set_tax_summary_warnings(SharedString::from(warnings_text));

    let readiness_items: Vec<TaxReadinessItem> = summary
        .readiness
        .iter()
        .map(|item| TaxReadinessItem {
            code: SharedString::from(item.code.as_str()),
            status: SharedString::from(item.status.as_str()),
            detail: SharedString::from(item.detail.as_str()),
        })
        .collect();
    // Count issues (non-ok, non-info statuses)
    let issue_count = summary
        .readiness
        .iter()
        .filter(|item| item.status != "ok" && item.status != "info")
        .count();
    let missing_price_count = summary
        .readiness
        .iter()
        .find(|item| item.code == "prices" && item.status == "warn" && item.detail != "invalid")
        .and_then(|item| item.detail.parse::<i32>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(0);
    adapter.set_tax_readiness_issues(issue_count as i32);
    adapter.set_tax_missing_price_count(missing_price_count);
    adapter.set_tax_can_resolve_prices(missing_price_count > 0);

    adapter.set_tax_readiness(ModelRc::new(VecModel::from(readiness_items)));
}

fn load_tax_settings(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>) {
    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<CryptoAdapter>();
        let jurisdiction = TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction());
        let period_raw = adapter.get_tax_period();
        let period_clean = period_raw.trim();
        let period_id = if period_clean.is_empty() {
            let year = chrono::Local::now().format("%Y").to_string();
            adapter.set_tax_period(SharedString::from(&year));
            year
        } else {
            period_clean.to_string()
        };
        let period_id = effective_period_id(&period_id, jurisdiction).unwrap_or(period_id);

        let settings = controller
            .load_tax_settings(period_id.clone())
            .unwrap_or_else(|_| {
                crate::features::crypto::TaxPeriodSettings::defaults_for(&period_id)
            });
        adapter.set_tax_jurisdiction(SharedString::from(settings.jurisdiction_str()));
        adapter.set_tax_method(SharedString::from(settings.method_str()));
        adapter.set_tax_include_swaps(settings.include_swaps);
        adapter.set_tax_include_fee_crypto(settings.include_fee_crypto);
    }
}
