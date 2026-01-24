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

//! Dashboard domain callbacks
//!
//! Callback setup for DashboardAdapter and AnalyticsAdapter.

use crate::controller::{AppController, SETTING_PREFERRED_CURRENCY};
use crate::ui::{color_from_hex, convert_usd_to_preferred, format_preferred};
use crate::{AnalyticsAdapter, AnalyticsData, AppWindow, BalanceData, CategoryData, DashboardAdapter};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::Arc;

/// Maximum days for crypto portfolio snapshots (2 years)
const MAX_SNAPSHOT_DAYS: i64 = 730;

/// Sets up all DashboardAdapter and AnalyticsAdapter callbacks
pub fn setup_dashboard_callbacks<F>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    reload_recent: F,
) where
    F: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
{
    // on_fetch_balance
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<DashboardAdapter>().on_fetch_balance(move || {
            // Set loading state
            if let Some(ui) = ui_weak.upgrade() {
                ui.global::<DashboardAdapter>().set_is_loading(true);
                ui.global::<DashboardAdapter>().set_has_error(false);
            }

            // Load preferred currency setting
            let preferred_currency = controller
                .get_app_setting(SETTING_PREFERRED_CURRENCY)
                .unwrap_or_else(|_| "USD".to_string());

            // 1. Load Exchange Rate (CLP -> USD)
            let (clp_rate, missing_rate) =
                match controller.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
                    Ok(Some((r, _))) if r > 0.0 => (r, false),
                    _ => (1.0, true), // Fallback to 1:1, flag as missing
                };

            // 2. Fetch Accounts & Balances (for normalized calculation)
            let accounts_res = controller.get_accounts();
            let balances_res = controller.get_account_balances();

            // 3. Fetch Crypto Portfolio
            let crypto_result = controller.get_aggregated_portfolio();
            let prices = controller.load_crypto_prices().unwrap_or_default();

            // Create price map for O(1) lookup
            let price_map: HashMap<String, f64> = prices
                .into_iter()
                .map(|p| (p.id, p.current_price))
                .collect();

            // Handle errors
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let dash = ui.global::<DashboardAdapter>();
            dash.set_is_loading(false);

            // Check for errors
            if accounts_res.is_err() || balances_res.is_err() || crypto_result.is_err() {
                dash.set_has_error(true);
                dash.set_error_message("Failed to load dashboard data".into());
                return;
            }

            let accounts = accounts_res.unwrap();
            let balances = balances_res.unwrap();
            let assets = crypto_result.unwrap();

            // Flag missing exchange rate (CLP balances may be inaccurate)
            let has_clp_accounts = accounts.iter().any(|a| a.currency.to_uppercase() == "CLP");
            dash.set_missing_exchange_rate(missing_rate && has_clp_accounts);

            // Create Currency Map (Account ID -> Currency)
            let currency_map: HashMap<String, String> = accounts
                .into_iter()
                .map(|a| (a.id, a.currency.to_uppercase()))
                .collect();

            // Calculate Normalized Fiat Total (always in USD first)
            let mut total_fiat_usd: f64 = 0.0;

            for bal in balances {
                let currency = currency_map
                    .get(&bal.account_id)
                    .map(|s| s.as_str())
                    .unwrap_or("USD");
                let rate = if currency == "CLP" { clp_rate } else { 1.0 };
                total_fiat_usd += (bal.current_balance as f64) / rate;
            }

            // Calculate Total Crypto Value (in USD)
            let crypto_total_usd: f64 = assets
                .iter()
                .map(|asset| {
                    let price = price_map.get(&asset.coin_id).cloned().unwrap_or(0.0);
                    asset.total_amount * price
                })
                .sum();

            // Net Worth in USD (Normalized Fiat + Crypto)
            let fiat_total_dollars = total_fiat_usd / 100.0;
            let net_worth_usd = fiat_total_dollars + crypto_total_usd;

            // Convert to preferred currency for display
            let net_worth = convert_usd_to_preferred(net_worth_usd, &preferred_currency, clp_rate);
            let fiat_display =
                convert_usd_to_preferred(fiat_total_dollars, &preferred_currency, clp_rate);
            let crypto_display =
                convert_usd_to_preferred(crypto_total_usd, &preferred_currency, clp_rate);

            dash.set_balance(BalanceData {
                total_balance: format_preferred(net_worth, &preferred_currency).into(),
                fiat_balance: format_preferred(fiat_display, &preferred_currency).into(),
                crypto_value: format_preferred(crypto_display, &preferred_currency).into(),
            });
        });
    }

    // on_fetch_recent
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_recent = reload_recent.clone();
        ui.global::<DashboardAdapter>().on_fetch_recent(move || {
            let _ = reload_recent(&ui_weak, &controller);
        });
    }

    // on_fetch_analytics
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<AnalyticsAdapter>()
            .on_fetch_analytics(move |range| {
                // Load preferred currency and exchange rate
                let preferred_currency = controller
                    .get_app_setting(SETTING_PREFERRED_CURRENCY)
                    .unwrap_or_else(|_| "USD".to_string());

                let clp_rate = controller
                    .load_exchange_rate_allow_stale("CLP_USD".to_string())
                    .ok()
                    .and_then(|r| r.map(|(rate, _)| rate))
                    .unwrap_or(1.0);

                // Calculate crypto total for dashboard data
                let crypto_total = calculate_crypto_total(&controller);

                // Get crypto snapshots for historical chart data
                let crypto_snapshots = controller
                    .get_crypto_portfolio_snapshots(MAX_SNAPSHOT_DAYS)
                    .unwrap_or_default();

                let range_str = range.to_string();
                match controller.get_dashboard_data(crypto_total, &crypto_snapshots, range_str) {
                    Ok(data) => {
                        let Some(ui) = ui_weak.upgrade() else {
                            return;
                        };

                        let adapter = ui.global::<AnalyticsAdapter>();

                        // Render chart image from values (following crypto/habits pattern)
                        let chart_image = controller
                            .render_net_worth_chart(&data.chart_values)
                            .unwrap_or_default();

                        let breakdown: Vec<CategoryData> = data
                            .expense_slices
                            .iter()
                            .map(|slice| {
                                // Convert expense amount to preferred currency
                                let amount_usd = slice.amount as f64 / 100.0;
                                let amount_display =
                                    convert_usd_to_preferred(amount_usd, &preferred_currency, clp_rate);
                                CategoryData {
                                    name: SharedString::from(&slice.category),
                                    amount: SharedString::from(format_preferred(
                                        amount_display,
                                        &preferred_currency,
                                    )),
                                    percentage: slice.percentage,
                                    color: color_from_hex(&slice.color),
                                }
                            })
                            .collect();

                        adapter.set_summary(AnalyticsData {
                            chart_image,
                            net_worth: SharedString::from(data.net_worth),
                            max_value: SharedString::from(data.max_value),
                            min_value: SharedString::from(data.min_value),
                            expense_breakdown: ModelRc::new(VecModel::from(breakdown)),
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to fetch analytics: {:?}", e);
                    }
                }
            });
    }
}

/// Calculates total crypto portfolio value in USD
fn calculate_crypto_total(controller: &AppController) -> f64 {
    let assets = match controller.get_aggregated_portfolio() {
        Ok(a) => a,
        Err(_) => return 0.0,
    };

    let prices = controller.load_crypto_prices().unwrap_or_default();
    let price_map: HashMap<String, f64> = prices
        .into_iter()
        .map(|p| (p.id, p.current_price))
        .collect();

    assets
        .iter()
        .map(|asset| {
            let price = price_map.get(&asset.coin_id).cloned().unwrap_or(0.0);
            asset.total_amount * price
        })
        .sum()
}
