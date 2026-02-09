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
use crate::{AppWindow, CryptoAdapter, TaxReadinessItem, TaxWalletEntry, Translations};
use slint::platform::Clipboard;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::sync::Arc;

/// Resolves the display currency and CLP exchange rate from the controller.
fn resolve_display_currency(controller: &AppController) -> (String, f64) {
    let preferred = controller
        .get_app_setting(SETTING_PREFERRED_CURRENCY)
        .unwrap_or_else(|_| "USD".to_string());
    let clp_rate = controller
        .load_exchange_rate_allow_stale("CLP_USD".to_string())
        .ok()
        .and_then(|r| r.map(|(rate, _)| rate))
        .unwrap_or(0.0);
    (preferred, clp_rate)
}

/// Formats a USD-denominated float amount in the user's preferred currency.
fn fmt_preferred(amount_usd: f64, currency: &str, clp_rate: f64) -> String {
    let converted = convert_usd_to_preferred(amount_usd, currency, clp_rate);
    format_preferred(converted, currency)
}

/// Formats a USD-denominated float amount with a sign prefix in the user's preferred currency.
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
                let period = period_raw.trim().to_string();
                if period.is_empty() {
                    notify(t("crypto-tax-period-required"), true);
                    return;
                }

                let (currency, clp_rate) = resolve_display_currency(&controller);

                match controller.generate_tax_summary(period) {
                    Ok(summary) => {
                        update_report_state(&adapter, &summary.report, &currency, clp_rate);
                        update_summary_state(&adapter, &summary, &currency, clp_rate);
                        notify(t("crypto-tax-report-generated"), false);
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
            let period = period_raw.trim().to_string();
            if period.is_empty() {
                notify(t("crypto-tax-period-required"), true);
                return;
            }

            let file_name = format!("sanctum-crypto-capital-gains-{}.csv", period);
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
            let period = period_raw.trim().to_string();
            if period.is_empty() {
                notify(t("crypto-tax-period-required"), true);
                return;
            }

            let file_name = format!("sanctum-crypto-transaction-history-{}.csv", period);
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
                let period = period_raw.trim().to_string();
                if period.is_empty() {
                    return;
                }

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
                    if let Some(mut entry) = model.row_data(i) {
                        if entry.id == wallet_id {
                            entry.excluded = !entry.excluded;
                            model.set_row_data(i, entry);
                            break;
                        }
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
            let period_raw = adapter.get_tax_period();
            let period = period_raw.trim().to_string();
            if period.is_empty() {
                notify(t("crypto-tax-period-required"), true);
                return;
            }

            // Preserve current excluded_wallet_ids from the already-saved settings.
            let existing_excluded = controller
                .load_tax_settings(period.clone())
                .map(|s| s.excluded_wallet_ids)
                .unwrap_or_default();

            let settings = crate::features::crypto::TaxPeriodSettings {
                period_id: period,
                jurisdiction: TaxJurisdiction::parse_or_default(&adapter.get_tax_jurisdiction()),
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
    let period_raw = adapter.get_tax_period();
    let period = period_raw.trim().to_string();

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
    adapter.set_tax_readiness_issues(issue_count as i32);

    adapter.set_tax_readiness(ModelRc::new(VecModel::from(readiness_items)));
}

fn load_tax_settings(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>) {
    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<CryptoAdapter>();
        let period_raw = adapter.get_tax_period();
        let period_clean = period_raw.trim();
        let period_id = if period_clean.is_empty() {
            let year = chrono::Local::now().format("%Y").to_string();
            adapter.set_tax_period(SharedString::from(&year));
            year
        } else {
            period_clean.to_string()
        };

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
