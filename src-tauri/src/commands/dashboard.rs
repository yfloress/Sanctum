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

//! Dashboard domain Tauri commands.
//!
//! Covers: net worth balance, recent transactions, analytics with chart data.

use sanctum::controller::{AppController, SETTING_PREFERRED_CURRENCY};
use sanctum::ui::dto::dashboard::{
    AnalyticsData, BalanceOverview, ExpenseBreakdownItem, MonthlyCashFlowItem, NetWorthChartData,
    RecentTransaction,
};
use sanctum::ui::{format_category_label, format_money, format_preferred};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

/// Maximum days for crypto portfolio snapshots (2 years).
const MAX_SNAPSHOT_DAYS: i64 = 730;

fn normalize_currency_code(code: &str) -> String {
    code.trim().to_uppercase()
}

fn load_cached_usd_rate(controller: &AppController, currency: &str) -> f64 {
    let target = normalize_currency_code(currency);
    if target == "USD" {
        return 1.0;
    }
    let pair = format!("{}_USD", target);
    controller
        .load_exchange_rate_allow_stale(pair)
        .ok()
        .and_then(|r| r.map(|(rate, _)| rate))
        .filter(|rate| *rate > 0.0)
        .unwrap_or(1.0)
}

fn convert_usd_to_preferred(amount_usd: f64, currency: &str, rate: f64) -> f64 {
    let target = normalize_currency_code(currency);
    if target == "USD" {
        amount_usd
    } else {
        amount_usd * rate
    }
}

/// Fetch net worth balance overview (fiat + crypto totals).
#[tauri::command]
pub fn fetch_balance(
    controller: State<'_, Arc<AppController>>,
) -> Result<BalanceOverview, String> {
    let preferred_currency = normalize_currency_code(
        &controller
            .get_app_setting(SETTING_PREFERRED_CURRENCY)
            .unwrap_or_else(|_| "USD".to_string()),
    );
    let preferred_rate = load_cached_usd_rate(&controller, &preferred_currency);

    let accounts = controller.get_accounts().map_err(|e| e.to_string())?;
    let balances = controller.get_account_balances().map_err(|e| e.to_string())?;
    let assets = controller.get_aggregated_portfolio().map_err(|e| e.to_string())?;
    let prices = controller.load_crypto_prices().unwrap_or_default();

    let currency_map: HashMap<String, String> = accounts
        .iter()
        .map(|a| (a.id.clone(), a.currency.to_uppercase()))
        .collect();

    let price_map: HashMap<String, f64> = prices
        .into_iter()
        .map(|p| (p.id, p.current_price))
        .collect();

    // Load USD rates for account currencies
    let mut usd_rates: HashMap<String, f64> = HashMap::from([("USD".to_string(), 1.0)]);
    for currency in currency_map.values() {
        if !usd_rates.contains_key(currency) {
            usd_rates.insert(
                currency.clone(),
                load_cached_usd_rate(&controller, currency),
            );
        }
    }

    // Calculate fiat total in USD
    let mut total_fiat_usd: f64 = 0.0;
    for bal in &balances {
        let currency = currency_map
            .get(&bal.account_id)
            .map(|s| s.as_str())
            .unwrap_or("USD");
        let rate = usd_rates.get(currency).copied().unwrap_or(1.0);
        total_fiat_usd += (bal.current_balance as f64) / rate;
    }

    // Calculate crypto total in USD
    let crypto_total_usd: f64 = assets
        .iter()
        .map(|asset| {
            let price = price_map.get(&asset.coin_id).copied().unwrap_or(0.0);
            asset.total_amount * price
        })
        .sum();

    let fiat_dollars = total_fiat_usd / 100.0;
    let net_worth_usd = fiat_dollars + crypto_total_usd;

    let net_worth = convert_usd_to_preferred(net_worth_usd, &preferred_currency, preferred_rate);
    let fiat_display =
        convert_usd_to_preferred(fiat_dollars, &preferred_currency, preferred_rate);
    let crypto_display =
        convert_usd_to_preferred(crypto_total_usd, &preferred_currency, preferred_rate);

    Ok(BalanceOverview {
        total: format_preferred(net_worth, &preferred_currency),
        total_negative: net_worth < 0.0,
        fiat_total: format_preferred(fiat_display, &preferred_currency),
        fiat_negative: fiat_display < 0.0,
        crypto_total: format_preferred(crypto_display, &preferred_currency),
        crypto_negative: crypto_display < 0.0,
        currency: preferred_currency,
    })
}

/// Fetch recent transactions for the dashboard feed.
#[tauri::command]
pub fn fetch_recent(
    controller: State<'_, Arc<AppController>>,
) -> Result<Vec<RecentTransaction>, String> {
    let accounts = controller.get_accounts().map_err(|e| e.to_string())?;
    let account_lookup: HashMap<String, (String, String)> = accounts
        .iter()
        .map(|a| (a.id.clone(), (a.currency.clone(), a.name.clone())))
        .collect();

    let mut transactions = controller.get_transactions().map_err(|e| e.to_string())?;
    transactions.sort_by(|a, b| b.date.cmp(&a.date));

    let recent: Vec<RecentTransaction> = transactions
        .into_iter()
        .take(5)
        .map(|tx| {
            let (currency, from_name) = account_lookup
                .get(&tx.account_id)
                .cloned()
                .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));

            let is_transfer = tx.transaction_type == "transfer";
            let is_expense = tx.transaction_type == "expense";

            let transfer_label = tx
                .transfer_account_id
                .as_ref()
                .and_then(|id| account_lookup.get(id))
                .map(|(_, name)| name.as_str())
                .unwrap_or("Account");

            let description = if is_transfer {
                if tx.description.is_empty() {
                    format!("{from_name} -> {transfer_label}")
                } else {
                    format!("{} ({from_name} -> {transfer_label})", tx.description)
                }
            } else {
                tx.description.clone()
            };

            let category = if is_transfer {
                "Transfer".to_string()
            } else {
                format_category_label(&tx.category.to_uppercase())
            };

            RecentTransaction {
                id: tx.id,
                date: tx.date,
                description,
                category,
                amount: format_money(tx.amount.abs(), &currency),
                is_expense,
                is_transfer,
                account_name: from_name,
            }
        })
        .collect();

    Ok(recent)
}

/// Fetch analytics data with chart series for a given time range.
#[tauri::command]
pub fn fetch_analytics(
    controller: State<'_, Arc<AppController>>,
    range: String,
) -> Result<AnalyticsData, String> {
    let preferred_currency = normalize_currency_code(
        &controller
            .get_app_setting(SETTING_PREFERRED_CURRENCY)
            .unwrap_or_else(|_| "USD".to_string()),
    );
    let preferred_rate = load_cached_usd_rate(&controller, &preferred_currency);

    let crypto_total = calculate_crypto_total(&controller);
    let crypto_snapshots = controller
        .get_crypto_portfolio_snapshots(MAX_SNAPSHOT_DAYS)
        .unwrap_or_default();

    let data = controller
        .get_dashboard_data(crypto_total, &crypto_snapshots, range)
        .map_err(|e| e.to_string())?;

    let breakdown: Vec<ExpenseBreakdownItem> = data
        .expense_slices
        .iter()
        .map(|slice| {
            let amount_usd = slice.amount as f64 / 100.0;
            let amount_display =
                convert_usd_to_preferred(amount_usd, &preferred_currency, preferred_rate);
            ExpenseBreakdownItem {
                category: slice.category.clone(),
                amount: format_preferred(amount_display, &preferred_currency),
                percentage: slice.percentage as f64,
                color: slice.color.clone(),
            }
        })
        .collect();

    let chart = NetWorthChartData {
        dates: data.chart_dates,
        values: data.chart_values.iter().map(|v| *v as f64).collect(),
    };

    let income_pref = convert_usd_to_preferred(
        data.total_income_cents as f64 / 100.0,
        &preferred_currency,
        preferred_rate,
    );
    let expense_pref = convert_usd_to_preferred(
        data.total_expense_cents as f64 / 100.0,
        &preferred_currency,
        preferred_rate,
    );
    let net_pref = income_pref - expense_pref;

    let monthly_cash_flow: Vec<MonthlyCashFlowItem> = data
        .monthly_labels
        .iter()
        .zip(data.monthly_income.iter().zip(data.monthly_expense.iter()))
        .map(|(label, (inc, exp))| {
            let inc_pref = convert_usd_to_preferred(
                *inc as f64 / 100.0,
                &preferred_currency,
                preferred_rate,
            );
            let exp_pref = convert_usd_to_preferred(
                *exp as f64 / 100.0,
                &preferred_currency,
                preferred_rate,
            );
            MonthlyCashFlowItem {
                month: label.clone(),
                income: inc_pref,
                expenses: exp_pref,
            }
        })
        .collect();

    Ok(AnalyticsData {
        net_worth: data.net_worth,
        net_worth_min: data.min_value,
        net_worth_max: data.max_value,
        total_income: format_preferred(income_pref, &preferred_currency),
        total_expenses: format_preferred(expense_pref, &preferred_currency),
        total_net: format_preferred(net_pref.abs(), &preferred_currency),
        total_net_negative: net_pref < 0.0,
        expense_breakdown: breakdown,
        chart,
        monthly_cash_flow,
    })
}

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
            let price = price_map.get(&asset.coin_id).copied().unwrap_or(0.0);
            asset.total_amount * price
        })
        .sum()
}
