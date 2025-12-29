//! Dashboard domain callbacks
//!
//! Callback setup for DashboardAdapter and AnalyticsAdapter.

use crate::controller::AppController;
use crate::ui::{color_from_hex, format_money, format_usd};
use crate::{AnalyticsAdapter, AnalyticsData, AppWindow, BalanceData, CategoryData, DashboardAdapter};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::Arc;

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
            // 1. Load Exchange Rate (CLP -> USD)
            let clp_rate = match controller.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
                Ok(Some((r, _))) => r,
                _ => 0.0,
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

            if let Ok(accounts) = accounts_res
                && let Ok(balances) = balances_res
                && let Ok(assets) = crypto_result
                && let Some(ui) = ui_weak.upgrade()
            {
                // Create Currency Map (Account ID -> Currency)
                let currency_map: HashMap<String, String> = accounts
                    .into_iter()
                    .map(|a| (a.id, a.currency.to_uppercase()))
                    .collect();

                // Calculate Normalized Fiat Totals
                let mut total_fiat_usd: f64 = 0.0;
                let mut total_income_usd: f64 = 0.0;
                let mut total_expense_usd: f64 = 0.0;

                for bal in balances {
                    let currency = currency_map
                        .get(&bal.account_id)
                        .map(|s| s.as_str())
                        .unwrap_or("USD");
                    let rate = if currency == "CLP" { clp_rate } else { 1.0 };

                    if rate > 0.0 {
                        total_fiat_usd += (bal.current_balance as f64) / rate;
                        total_income_usd += (bal.total_income as f64) / rate;
                        total_expense_usd += (bal.total_expense as f64) / rate;
                    }
                }

                // Calculate Total Crypto Value (in USD)
                let crypto_total: f64 = assets
                    .iter()
                    .map(|asset| {
                        let price = price_map.get(&asset.coin_id).cloned().unwrap_or(0.0);
                        asset.total_amount * price
                    })
                    .sum();

                // Net Worth (Normalized Fiat + Crypto)
                let fiat_total_dollars = total_fiat_usd / 100.0;
                let net_worth = fiat_total_dollars + crypto_total;

                let dash = ui.global::<DashboardAdapter>();
                dash.set_balance(BalanceData {
                    total_balance: format_usd(net_worth).into(),
                    total_income: format_money(total_income_usd as i64, "USD").into(),
                    total_expense: format_money(total_expense_usd as i64, "USD").into(),
                });
            }
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
                if let Ok(summary) = controller.get_analytics_summary(range.to_string())
                    && let Some(ui) = ui_weak.upgrade()
                {
                    let adapter = ui.global::<AnalyticsAdapter>();

                    let breakdown: Vec<CategoryData> = summary
                        .expense_slices
                        .iter()
                        .map(|slice| CategoryData {
                            name: SharedString::from(&slice.category),
                            amount: SharedString::from(format_money(slice.amount, "USD")),
                            percentage: slice.percentage,
                            color: color_from_hex(&slice.color),
                        })
                        .collect();

                    adapter.set_summary(AnalyticsData {
                        chart_path: SharedString::from(summary.chart_path),
                        net_worth: SharedString::from(summary.net_worth),
                        max_value: SharedString::from(summary.max_value),
                        min_value: SharedString::from(summary.min_value),
                        expense_breakdown: ModelRc::new(VecModel::from(breakdown)),
                    });
                }
            });
    }
}
