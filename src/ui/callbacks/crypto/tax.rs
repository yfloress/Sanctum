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

use crate::controller::AppController;
use crate::services::i18n::{t, t_args};
use crate::ui::{format_money, format_money_signed};
use crate::{AppWindow, CryptoAdapter};
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;

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
            update_ipc_summary(&ui_weak, &controller, None::<&fn(String, bool)>);
            reset_report_state(&ui_weak);
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
                            (
                                "first",
                                summary.first_period.as_deref().unwrap_or(""),
                            ),
                            (
                                "last",
                                summary.last_period.as_deref().unwrap_or(""),
                            ),
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

    // Generate tax report
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_generate_tax_report(move || {
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

            match controller.generate_tax_report(period) {
                Ok(report) => {
                    update_report_state(&adapter, &report);
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

            let file_name = format!("sanctum-crypto-tax-{}.csv", period);
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

            let settings = crate::features::crypto::TaxPeriodSettings {
                period_id: period,
                jurisdiction: adapter.get_tax_jurisdiction().to_string(),
                method: adapter.get_tax_method().to_string(),
                include_swaps: adapter.get_tax_include_swaps(),
                include_fee_crypto: adapter.get_tax_include_fee_crypto(),
            };

            match controller.save_tax_settings(settings) {
                Ok(_) => notify(t("crypto-tax-settings-saved"), false),
                Err(err) => notify(format!("Failed to save tax settings: {}", err), true),
            }
        });
    }
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
        let text = if let Some(summary) = summary {
            t_args(
                "crypto-tax-ipc-summary",
                &[
                    ("count", summary.count.to_string().as_str()),
                    ("first", summary.first_period.as_str()),
                    ("last", summary.last_period.as_str()),
                ],
            )
        } else {
            t("crypto-tax-ipc-empty")
        };

        adapter.set_tax_ipc_summary(SharedString::from(text));
    }
}

fn reset_report_state(ui_weak: &Weak<AppWindow>) {
    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<CryptoAdapter>();
        adapter.set_tax_report_summary(SharedString::from(t("crypto-tax-report-summary-empty")));
        adapter.set_tax_report_warnings(SharedString::from(t("crypto-tax-report-warnings-empty")));
    }
}

fn update_report_state(adapter: &CryptoAdapter, report: &crate::features::crypto::TaxReport) {
    let proceeds = format_money((report.summary.total_proceeds * 100.0) as i64, "USD");
    let cost = format_money((report.summary.total_cost * 100.0) as i64, "USD");
    let gain = format_money_signed((report.summary.total_gain * 100.0) as i64, "USD");
    let disposals = report.summary.disposals.to_string();

    let summary = if let (Some(short), Some(long)) = (
        report.summary.short_term_gain,
        report.summary.long_term_gain,
    ) {
        let short_fmt = format_money_signed((short * 100.0) as i64, "USD");
        let long_fmt = format_money_signed((long * 100.0) as i64, "USD");
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
        t_args("crypto-tax-report-warnings-count", &[("count", count.as_str())])
    };

    adapter.set_tax_report_summary(SharedString::from(summary));
    adapter.set_tax_report_warnings(SharedString::from(warning_text));
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
            .unwrap_or_else(|_| crate::features::crypto::TaxPeriodSettings::defaults_for(&period_id));
        adapter.set_tax_jurisdiction(SharedString::from(settings.jurisdiction));
        adapter.set_tax_method(SharedString::from(settings.method));
        adapter.set_tax_include_swaps(settings.include_swaps);
        adapter.set_tax_include_fee_crypto(settings.include_fee_crypto);
    }
}
