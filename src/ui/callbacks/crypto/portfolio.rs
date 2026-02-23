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

//! Portfolio and price-related crypto callbacks

use super::helpers::{
    SETTING_CRYPTO_LAST_UPDATED, badge_currency_for_preferred, format_compact_price_preferred,
    format_compact_asset_amount, load_preferred_usd_rate, reload_portfolio,
    resolve_preferred_currency,
    usd_pair_for_target_currency,
};
use crate::controller::AppController;
use crate::models::CryptoTransaction;
use crate::services::i18n::{t, t_args};
use crate::ui::{
    convert_usd_to_preferred, crypto_icon_for_symbol, format_crypto_tx_display, format_fee_display,
    format_fx_rate,
    format_preferred,
};
use crate::{AssetTransaction, AssetWalletBreakdown, AppWindow, CryptoAdapter, CryptoAssetData};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const MANUAL_REFRESH_COOLDOWN_SECS: u64 = 8;

static LAST_MANUAL_REFRESH_MS: AtomicU64 = AtomicU64::new(0);
static HISTORICAL_REQUESTS_IN_FLIGHT: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static HISTORICAL_AUTO_REQUESTED_KEYS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static HISTORICAL_PRICE_CACHE: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn try_acquire_manual_refresh_slot() -> Option<u64> {
    let cooldown_ms = MANUAL_REFRESH_COOLDOWN_SECS.saturating_mul(1000);
    loop {
        let now = now_millis();
        let last = LAST_MANUAL_REFRESH_MS.load(Ordering::SeqCst);

        if last == 0 || now >= last.saturating_add(cooldown_ms) {
            if LAST_MANUAL_REFRESH_MS
                .compare_exchange(last, now, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return None;
            }
            continue;
        }

        let remaining_ms = last.saturating_add(cooldown_ms).saturating_sub(now);
        return Some(remaining_ms.div_ceil(1000));
    }
}

fn is_suspicious_fx_jump(
    new_rate: f64,
    cached: Option<&(f64, String)>,
) -> Option<f64> {
    let (old_rate, updated_at) = cached?;
    if *old_rate <= 0.0 || !new_rate.is_finite() || new_rate <= 0.0 {
        return None;
    }
    let Ok(previous_dt) = chrono::DateTime::parse_from_rfc3339(updated_at) else {
        return None;
    };
    let age = chrono::Utc::now().signed_duration_since(previous_dt.with_timezone(&chrono::Utc));
    if age.num_seconds() < 0 {
        return None;
    }

    let age_hours = (age.num_seconds() as f64 / 3600.0).max(1.0);
    // Adaptive limit based on elapsed time since last trusted rate:
    // after 24h allows ~100% move; shorter windows allow proportionally less.
    let allowed_jump = (age_hours / 24.0).sqrt();
    let jump = (new_rate - *old_rate).abs() / old_rate.abs();
    if jump > allowed_jump {
        Some(jump)
    } else {
        None
    }
}

fn try_start_historical_request(request_key: &str) -> bool {
    let Ok(mut in_flight) = HISTORICAL_REQUESTS_IN_FLIGHT.lock() else {
        return false;
    };
    if in_flight.contains(request_key) {
        return false;
    }
    in_flight.insert(request_key.to_string());
    true
}

fn has_auto_historical_request(request_key: &str) -> bool {
    let Ok(keys) = HISTORICAL_AUTO_REQUESTED_KEYS.lock() else {
        return false;
    };
    keys.contains(request_key)
}

fn mark_auto_historical_request(request_key: &str) {
    if let Ok(mut keys) = HISTORICAL_AUTO_REQUESTED_KEYS.lock() {
        keys.insert(request_key.to_string());
    }
}

fn finish_historical_request(request_key: &str) {
    if let Ok(mut in_flight) = HISTORICAL_REQUESTS_IN_FLIGHT.lock() {
        in_flight.remove(request_key);
    }
}

fn get_cached_historical_price(request_key: &str) -> Option<String> {
    let Ok(cache) = HISTORICAL_PRICE_CACHE.lock() else {
        return None;
    };
    cache.get(request_key).cloned()
}

fn cache_historical_price(request_key: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    if let Ok(mut cache) = HISTORICAL_PRICE_CACHE.lock() {
        cache.insert(request_key.to_string(), value.to_string());
    }
}

fn should_suppress_historical_price_error(error_text: &str) -> bool {
    error_text.contains("Date cannot be empty")
        || error_text.contains("Invalid date format")
        || error_text.contains("Coin ID cannot be empty")
        || error_text.contains("Coin ID contains invalid characters")
}

fn map_historical_price_error_for_ui(error_text: &str) -> String {
    if error_text.contains("Historical price data not found")
        || error_text.contains("Historical USD price not available")
        || error_text.contains("Date cannot be in the future")
    {
        return "No historical price available for that coin/date.".to_string();
    }
    if error_text.contains("Rate limit exceeded") {
        return "Historical API rate limit reached. Please wait and try again.".to_string();
    }
    if let Some(stripped) = error_text.strip_prefix("API error: ") {
        return stripped.to_string();
    }
    error_text.to_string()
}

/// Sets up portfolio-related callbacks
pub fn setup_portfolio_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + Send + 'static,
{
    // on_fetch_portfolio
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>().on_fetch_portfolio(move || {
            reload_portfolio::<fn(String, bool)>(&ui_weak, &controller, None);
        });
    }

    // Shared refresh implementation (show_loading = show the loading overlay)
    fn do_refresh_prices<N>(
        controller: Arc<AppController>,
        ui_weak: Weak<AppWindow>,
        notify: N,
        show_loading: bool,
    ) where
        N: Fn(String, bool) + Clone + Send + 'static,
    {
        let controller_async = controller.clone();
        let ui_weak_async = ui_weak.clone();

        // Only show loading overlay for manual refresh
        if show_loading {
            if let Some(ui) = ui_weak.upgrade() {
                ui.global::<CryptoAdapter>().set_is_refreshing(true);
            }
            notify("Fetching prices...".into(), false);
        }

        let notify_for_async = notify.clone();

        tokio::spawn(async move {
            let coins = controller_async
                .get_monitored_coin_ids()
                .unwrap_or_default();

            let limit_reached = coins.len() > 50;
            let limit_excluded = if limit_reached {
                let extra_count = coins.len().saturating_sub(50);
                let preview: Vec<String> = coins.iter().skip(50).take(3).cloned().collect();
                if preview.is_empty() {
                    String::new()
                } else if extra_count > preview.len() {
                    format!(
                        "{} +{} more",
                        preview.join(", "),
                        extra_count - preview.len()
                    )
                } else {
                    preview.join(", ")
                }
            } else {
                String::new()
            };
            let has_coins = !coins.is_empty();

            let mut prices_updated = false;
            if !coins.is_empty() {
                match controller_async.get_crypto_prices(coins).await {
                    Ok(prices) => {
                        let _ = controller_async.save_crypto_prices(prices);
                        prices_updated = true;
                    }
                    Err(e) => {
                        let notify_fail = notify_for_async.clone();
                        let _ = ui_weak_async.upgrade_in_event_loop(move |_| {
                            notify_fail(format!("Price update failed: {}", e), true);
                        });
                    }
                }
            }

            let preferred_currency = resolve_preferred_currency(&controller_async);
            let badge_currency = badge_currency_for_preferred(&preferred_currency);
            let badge_pair = usd_pair_for_target_currency(&badge_currency);
            let badge_label = format!("USD/{}", badge_currency);
            let cached_fx = controller_async
                .load_exchange_rate_allow_stale(badge_pair.clone())
                .ok()
                .flatten();

            let mut fx_warning: Option<String> = None;
            let (fx_display, fx_updated) = match controller_async
                .get_usd_fx_rate(badge_currency.clone())
                .await
            {
                Ok(rate) => {
                    if let Some(jump) = is_suspicious_fx_jump(rate, cached_fx.as_ref()) {
                        if let Some((cached_rate, _)) = cached_fx.as_ref() {
                            fx_warning = Some(format!(
                                "Ignored suspicious USD/{} jump ({:.1}%). Using cached rate.",
                                badge_currency,
                                jump * 100.0
                            ));
                            (format_fx_rate(*cached_rate, &badge_currency), true)
                        } else {
                            let _ = controller_async.save_exchange_rate(badge_pair.clone(), rate);
                            (format_fx_rate(rate, &badge_currency), true)
                        }
                    } else {
                        let _ = controller_async.save_exchange_rate(badge_pair.clone(), rate);
                        (format_fx_rate(rate, &badge_currency), true)
                    }
                }
                Err(_) => {
                    if let Some((rate, _)) = cached_fx.as_ref() {
                        (format_fx_rate(*rate, &badge_currency), true)
                    } else {
                        ("N/A".to_string(), false)
                    }
                }
            };

            let now = chrono::Local::now();
            let timestamp_to_save = if prices_updated {
                Some(now.to_rfc3339())
            } else {
                None
            };
            let last_updated_label = if prices_updated {
                let time = now.format("%H:%M").to_string();
                Some(t_args("crypto-last-updated-today-at", &[("time", time.as_str())]))
            } else {
                None
            };

            if let Some(ts) = timestamp_to_save.as_ref() {
                let _ = controller_async.set_app_setting(SETTING_CRYPTO_LAST_UPDATED, ts);
            }

            let notify_success = notify_for_async.clone();
            let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                ui.global::<CryptoAdapter>().set_is_refreshing(false);
                ui.global::<CryptoAdapter>().invoke_fetch_portfolio();
                ui.global::<CryptoAdapter>()
                    .set_fx_rate_label(SharedString::from(badge_label));
                ui.global::<CryptoAdapter>()
                    .set_clp_rate(SharedString::from(fx_display));
                if let Some(label) = last_updated_label {
                    ui.global::<CryptoAdapter>().set_last_updated(label.into());
                }
                ui.global::<CryptoAdapter>()
                    .set_limit_reached(limit_reached);
                ui.global::<CryptoAdapter>()
                    .set_limit_excluded(limit_excluded.into());
                if prices_updated {
                    notify_success("Prices updated".into(), false);
                } else if !has_coins && fx_updated {
                    notify_success("Rates updated".into(), false);
                }
                if let Some(warn) = fx_warning {
                    notify_success(warn, true);
                }
            });
        });
    }

    // on_refresh_prices (manual - with loading overlay)
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_refresh_prices(move || {
            if let Some(wait_secs) = try_acquire_manual_refresh_slot() {
                notify(
                    format!("Please wait {}s before syncing again.", wait_secs),
                    true,
                );
                return;
            }
            do_refresh_prices(controller.clone(), ui_weak.clone(), notify.clone(), true);
        });
    }

    // on_refresh_prices_silent (auto-fetch - no loading overlay, but still shows toast)
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>()
            .on_refresh_prices_silent(move || {
                do_refresh_prices(controller.clone(), ui_weak.clone(), notify.clone(), false);
            });
    }

    // on_get_last_price
    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>()
            .on_get_last_price(move |coin_id| {
                let prices = controller.load_crypto_prices().unwrap_or_default();
                if let Some(data) = prices.iter().find(|p| p.id == coin_id.as_str()) {
                    format!("{:.4}", data.current_price).into()
                } else {
                    "".into()
                }
            });
    }

    // on_request_historical_price
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<CryptoAdapter>()
            .on_request_historical_price(move |coin_id, date, user_initiated| {
                let coin_id_str = coin_id.to_string();
                let date_str = date.to_string();
                let request_key = format!("{}|{}", coin_id_str, date_str);

                if !user_initiated && has_auto_historical_request(&request_key) {
                    if let Some(cached_value) = get_cached_historical_price(&request_key) {
                        let request_key_cached = request_key.clone();
                        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                            let adapter = ui.global::<CryptoAdapter>();
                            adapter.set_historical_price_key(SharedString::from(request_key_cached));
                            adapter.set_historical_price_value(SharedString::from(cached_value));
                        });
                    }
                    return;
                }

                if let Some(cached_value) = get_cached_historical_price(&request_key) {
                    let request_key_cached = request_key.clone();
                    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                        let adapter = ui.global::<CryptoAdapter>();
                        adapter.set_historical_price_key(SharedString::from(request_key_cached));
                        adapter.set_historical_price_value(SharedString::from(cached_value));
                    });
                    return;
                }

                if !try_start_historical_request(&request_key) {
                    return;
                }
                if !user_initiated {
                    mark_auto_historical_request(&request_key);
                }
                let controller_async = controller.clone();
                let ui_weak_async = ui_weak.clone();
                let notify_async = notify.clone();
                let request_key_async = request_key.clone();

                tokio::spawn(async move {
                    let result = controller_async
                        .get_crypto_historical_price_usd(coin_id_str, date_str)
                        .await;

                    let (value, notify_msg) = match result {
                        Ok(price) => (format!("{:.4}", price), None),
                        Err(err) => {
                            let err_text = err.to_string();
                            let msg = if should_suppress_historical_price_error(&err_text)
                                || !user_initiated
                            {
                                None
                            } else {
                                Some(map_historical_price_error_for_ui(&err_text))
                            };
                            (String::new(), msg)
                        }
                    };

                    if let Some(message) = notify_msg {
                        let notify_for_ui = notify_async.clone();
                        let _ = ui_weak_async.upgrade_in_event_loop(move |_| {
                            notify_for_ui(message, true);
                        });
                    }

                    cache_historical_price(&request_key_async, &value);
                    finish_historical_request(&request_key_async);
                    let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                        let adapter = ui.global::<CryptoAdapter>();
                        adapter.set_historical_price_key(SharedString::from(request_key));
                        adapter.set_historical_price_value(SharedString::from(value));
                    });
                });
            });
    }

    // on_show_last_updated_info
    {
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<CryptoAdapter>().on_show_last_updated_info(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let adapter = ui.global::<CryptoAdapter>();
            let raw = adapter.get_last_updated().trim().to_string();
            let value = if raw.is_empty() {
                t("crypto-last-updated-never")
            } else {
                raw
            };
            let msg = t_args("crypto-last-updated-info", &[("value", value.as_str())]);
            notify(msg, false);
        });
    }

    // on_get_swap_quote
    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>().on_get_swap_quote(
            move |from_coin_id, to_coin_id, amount_str| {
                let amount_clean = amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let amount: f64 = match amount_clean.parse() {
                    Ok(value) if value > 0.0 => value,
                    _ => return SharedString::from(""),
                };

                let prices = controller.load_crypto_prices().unwrap_or_default();
                let from_price = prices
                    .iter()
                    .find(|p| p.id == from_coin_id.as_str())
                    .map(|p| p.current_price)
                    .unwrap_or(0.0);
                let to_price = prices
                    .iter()
                    .find(|p| p.id == to_coin_id.as_str())
                    .map(|p| p.current_price)
                    .unwrap_or(0.0);

                if from_price <= 0.0 || to_price <= 0.0 {
                    return SharedString::from("");
                }

                let to_amount = amount * (from_price / to_price);
                let mut formatted = format!("{:.8}", to_amount);
                while formatted.contains('.') && formatted.ends_with('0') {
                    formatted.pop();
                }
                if formatted.ends_with('.') {
                    formatted.pop();
                }

                SharedString::from(formatted)
            },
        );
    }

    // on_get_available_balance
    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>()
            .on_get_available_balance(move |wallet_id, coin_id, date| {
                match controller.get_available_balance(
                    wallet_id.to_string(),
                    coin_id.to_string(),
                    date.to_string(),
                ) {
                    Ok(balance) => {
                        let mut formatted = format!("{:.8}", balance);
                        while formatted.contains('.') && formatted.ends_with('0') {
                            formatted.pop();
                        }
                        if formatted.ends_with('.') {
                            formatted.pop();
                        }
                        SharedString::from(formatted)
                    }
                    Err(e) => {
                        log::error!("Error getting available balance: {:?}", e);
                        SharedString::from("")
                    }
                }
            });
    }

    // on_fetch_asset_details
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>()
            .on_fetch_asset_details(move |coin_id| {
                let coin_id_str = coin_id.to_string();

                // Load preferred currency and exchange rate
                let preferred_currency = resolve_preferred_currency(&controller);
                let usd_rate = load_preferred_usd_rate(&controller, &preferred_currency);

                if let Ok(assets) = controller.get_aggregated_portfolio()
                    && let Some(asset) = assets.iter().find(|a| a.coin_id == coin_id_str)
                {
                    let prices = controller.load_crypto_prices().unwrap_or_default();
                    let price_data = prices.iter().find(|p| p.id == coin_id_str);
                    let catalog_map: HashMap<String, String> = controller
                        .get_coin_catalog_or_default()
                        .into_iter()
                        .map(|coin| (coin.id, coin.name))
                        .collect();
                    let current_price = price_data.map(|p| p.current_price).unwrap_or(0.0);
                    let price_change = price_data
                        .map(|p| p.price_change_percentage_24h)
                        .unwrap_or(0.0);
                    let asset_name = price_data
                        .map(|p| p.name.clone())
                        .or_else(|| catalog_map.get(&coin_id_str).cloned())
                        .unwrap_or_else(|| asset.symbol.clone());

                    let mut updated_asset = asset.clone();
                    if current_price > 0.0 {
                        updated_asset.update_with_price(current_price);
                    }

                    let missing_price = price_data.is_none();
                    let change_str = if missing_price {
                        "N/A".to_string()
                    } else if price_change >= 0.0 {
                        format!("+ {:.2}%", price_change)
                    } else {
                        format!("{:.2}%", price_change)
                    };

                    // Convert price and value to preferred currency
                    let price_preferred =
                        convert_usd_to_preferred(updated_asset.current_price, &preferred_currency, usd_rate);
                    let value_preferred =
                        convert_usd_to_preferred(updated_asset.current_value, &preferred_currency, usd_rate);

                    let price_fmt = if missing_price {
                        "N/A".to_string()
                    } else {
                        format_compact_price_preferred(price_preferred, &preferred_currency)
                    };

                    let value_fmt = if missing_price {
                        "N/A".to_string()
                    } else {
                        format_preferred(value_preferred, &preferred_currency)
                    };

                    let selected = CryptoAssetData {
                        id: SharedString::from(&updated_asset.coin_id),
                        symbol: SharedString::from(&updated_asset.symbol),
                        icon: crypto_icon_for_symbol(&updated_asset.symbol),
                        name: SharedString::from(asset_name),
                        price: SharedString::from(price_fmt),
                        amount: SharedString::from(format!(
                            "{} {}",
                            format_compact_asset_amount(updated_asset.total_amount),
                            updated_asset.symbol
                        )),
                        value: SharedString::from(value_fmt),
                        change_24h: SharedString::from(change_str),
                        is_positive: price_change >= 0.0,
                        allocation: 0.0,
                    };

                    let wallets = controller.get_wallets().unwrap_or_default();
                    let mut wallet_breakdown: Vec<AssetWalletBreakdown> = Vec::new();

                    for w in wallets {
                        let holdings = controller
                            .get_wallet_holdings(w.id.clone())
                            .unwrap_or_default();
                        if let Some(h) = holdings.iter().find(|h| h.coin_id == coin_id_str)
                            && h.total_amount > 0.0
                        {
                            let val_usd = h.total_amount * current_price;
                            let val_preferred =
                                convert_usd_to_preferred(val_usd, &preferred_currency, usd_rate);
                            wallet_breakdown.push(AssetWalletBreakdown {
                                wallet_name: SharedString::from(w.name),
                                amount: SharedString::from(format_compact_asset_amount(
                                    h.total_amount,
                                )),
                                value: SharedString::from(format_preferred(
                                    val_preferred,
                                    &preferred_currency,
                                )),
                            });
                        }
                    }

                    let history = controller
                        .get_crypto_transactions_by_coin(coin_id_str)
                        .unwrap_or_default();
                    let symbol_map: HashMap<String, String> = controller
                        .get_coin_catalog_or_default()
                        .into_iter()
                        .map(|coin| (coin.id, coin.symbol))
                        .collect();
                    let history_map: HashMap<String, CryptoTransaction> = history
                        .iter()
                        .cloned()
                        .map(|tx| (tx.id.clone(), tx))
                        .collect();
                    let history_mapped: Vec<AssetTransaction> = history
                        .iter()
                        .map(|tx| {
                            let related =
                                tx.related_tx_id.as_ref().and_then(|id| history_map.get(id));
                            let (label, amount_display, price_display, is_swap) =
                                format_crypto_tx_display(tx, related);
                            let fee_fmt = format_fee_display(tx, &symbol_map);
                            let notes = tx.notes.clone().unwrap_or_default();

                            AssetTransaction {
                                id: SharedString::from(&tx.id),
                                date: SharedString::from(&tx.date),
                                r#type: SharedString::from(label),
                                amount: SharedString::from(amount_display),
                                price: SharedString::from(price_display),
                                fee: SharedString::from(fee_fmt),
                                notes: SharedString::from(notes),
                                is_swap,
                            }
                        })
                        .collect();

                    if let Some(ui) = ui_weak.upgrade() {
                        let adapter = ui.global::<CryptoAdapter>();
                        adapter.set_selected_asset(selected);
                        adapter.set_asset_wallets(ModelRc::new(VecModel::from(wallet_breakdown)));
                        adapter.set_asset_history(ModelRc::new(VecModel::from(history_mapped)));
                    }
                }
            });
    }
}

#[cfg(test)]
mod tests;
