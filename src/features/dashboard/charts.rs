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

//! Dashboard charts data calculation
//!
//! Net worth calculations and expense breakdowns.
//! Aggregates financial and crypto data for dashboard visualization.
//! Note: Actual chart rendering is done by services/charts.rs

use crate::ui::currency::format_money;
use crate::models::{Account, AccountBalance, Transaction};
use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashMap;

/// Date format used for parsing transaction dates
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Colors for expense breakdown categories (from theme)
const CATEGORY_COLORS: [&str; 8] = [
    "#8b5cf6", // violet
    "#ec4899", // pink
    "#3b82f6", // blue
    "#10b981", // emerald
    "#f59e0b", // amber
    "#ef4444", // red
    "#6366f1", // indigo
    "#14b8a6", // teal
];

#[derive(Debug, Clone)]
pub struct ExpenseSlice {
    pub category: String,
    pub amount: i64,
    pub percentage: f32,
    pub color: String,
}

/// Dashboard data for UI (chart values, not rendered image)
#[derive(Debug, Clone)]
pub struct DashboardData {
    /// Raw balance values for chart rendering (call controller.render_net_worth_chart)
    pub chart_values: Vec<i64>,
    /// ISO date strings corresponding to each chart_values entry.
    pub chart_dates: Vec<String>,
    pub net_worth: String,
    pub max_value: String,
    pub min_value: String,
    pub expense_slices: Vec<ExpenseSlice>,
    /// Total income for the selected range, in USD cents.
    pub total_income_cents: i64,
    /// Total expenses for the selected range, in USD cents.
    pub total_expense_cents: i64,
    /// Last 6 months income in USD cents (oldest → newest).
    pub monthly_income: Vec<i64>,
    /// Last 6 months expenses in USD cents (oldest → newest).
    pub monthly_expense: Vec<i64>,
    /// Short month labels for monthly_income/monthly_expense (e.g. "Jan").
    pub monthly_labels: Vec<String>,
}

/// Chart data calculation for dashboard
pub struct DashboardCharts;

impl DashboardCharts {
    /// Calculates dashboard data (FIAT + Crypto combined)
    /// Returns raw values - use controller.render_net_worth_chart() for the image
    ///
    /// # Arguments
    /// * `crypto_total_usd` - Current crypto portfolio value in USD
    /// * `crypto_snapshots` - Historical snapshots: Vec<(date_str, value_usd, cost_usd)>
    /// * `usd_rates` - Currency rates in `CURRENCY/USD` format (e.g. CLP per 1 USD)
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_dashboard_data(
        balances: &[AccountBalance],
        accounts: &[Account],
        transactions: &[Transaction],
        crypto_total_usd: f64,
        crypto_snapshots: &[(String, f64, f64)],
        usd_rates: &HashMap<String, f64>,
        range: &str,
        preferred_currency: &str,
    ) -> DashboardData {
        let currency_map: HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            let rate = usd_rates
                .get(currency)
                .copied()
                .filter(|r| *r > 0.0)
                .unwrap_or(1.0);
            ((amount as f64) / rate).round() as i64
        };

        // Calculate FIAT balance in cents
        let fiat_balance: i64 = balances
            .iter()
            .map(|b| normalize(b.current_balance, &b.account_id))
            .sum();

        // Convert crypto to cents
        let crypto_cents = (crypto_total_usd * 100.0) as i64;
        let total_balance = fiat_balance + crypto_cents;

        // Build crypto snapshot lookup (date -> value in cents)
        let crypto_by_date: HashMap<String, i64> = crypto_snapshots
            .iter()
            .map(|(date, value, _)| (date.clone(), (*value * 100.0) as i64))
            .collect();

        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();
        let start_date = match range {
            "1M" => today - chrono::Duration::days(30),
            "3M" => today - chrono::Duration::days(90),
            "6M" => today - chrono::Duration::days(180),
            "1Y" => today - chrono::Duration::days(365),
            _ => today - chrono::Duration::days(365 * 5), // ALL
        };

        // Build daily balance history from transactions
        let mut delta_by_day: HashMap<NaiveDate, i64> = HashMap::new();
        for tx in transactions {
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, DATE_FORMAT)
                && date >= start_date
            {
                let raw_delta = match tx.transaction_type.as_str() {
                    "income" => tx.amount,
                    "expense" => -tx.amount,
                    _ => 0,
                };
                let delta = normalize(raw_delta, &tx.account_id);
                *delta_by_day.entry(date).or_insert(0) += delta;
            }
        }

        // Build balance history going backwards from today
        let mut cursor = today;
        let mut points_rev: Vec<(NaiveDate, i64)> = Vec::new();
        let mut balance = fiat_balance;

        loop {
            let date_str = cursor.format("%Y-%m-%d").to_string();

            // Use historical crypto snapshot if available, else current value
            let crypto_at_date = if date_str == today_str {
                crypto_cents // Always use current value for today
            } else {
                crypto_by_date
                    .get(&date_str)
                    .copied()
                    .unwrap_or(crypto_cents)
            };

            let total_at_date = balance + crypto_at_date;
            points_rev.push((cursor, total_at_date));

            let delta = *delta_by_day.get(&cursor).unwrap_or(&0);
            balance -= delta;

            if cursor <= start_date {
                break;
            }
            if let Some(prev) = cursor.pred_opt() {
                cursor = prev;
            } else {
                break;
            }
        }

        points_rev.reverse();
        if points_rev.is_empty() {
            points_rev.push((today, total_balance));
        }

        let dates: Vec<String> = points_rev
            .iter()
            .map(|(d, _)| d.format("%Y-%m-%d").to_string())
            .collect();
        let values: Vec<i64> = points_rev.iter().map(|(_, v)| *v).collect();
        let min_val = *values.iter().min().unwrap_or(&0);
        let max_val = *values.iter().max().unwrap_or(&0);

        let expense_slices = Self::calculate_expense_slices(
            transactions,
            &currency_map,
            usd_rates,
            start_date,
            today,
        );

        // Total income for the selected range
        let mut total_income_cents: i64 = 0;
        let mut total_expense_cents: i64 = 0;
        for tx in transactions {
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, DATE_FORMAT)
                && date >= start_date
                && date <= today
            {
                let amount = normalize(tx.amount, &tx.account_id);
                match tx.transaction_type.as_str() {
                    "income" => total_income_cents += amount,
                    "expense" => total_expense_cents += amount,
                    _ => {}
                }
            }
        }

        // Last 6 months cash flow (oldest → newest)
        const MONTH_NAMES: [&str; 12] =
            ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
        let mut monthly_income: Vec<i64> = Vec::with_capacity(6);
        let mut monthly_expense: Vec<i64> = Vec::with_capacity(6);
        let mut monthly_labels: Vec<String> = Vec::with_capacity(6);
        for offset in (0..6i32).rev() {
            let mut m = today.month() as i32 - offset;
            let mut y = today.year();
            while m <= 0 {
                m += 12;
                y -= 1;
            }
            let month_key = format!("{:04}-{:02}", y, m);
            let mut inc: i64 = 0;
            let mut exp: i64 = 0;
            for tx in transactions {
                if tx.date.starts_with(&month_key) {
                    let amount = normalize(tx.amount, &tx.account_id);
                    match tx.transaction_type.as_str() {
                        "income" => inc += amount,
                        "expense" => exp += amount,
                        _ => {}
                    }
                }
            }
            monthly_income.push(inc);
            monthly_expense.push(exp);
            monthly_labels.push(MONTH_NAMES[(m as usize - 1) % 12].to_string());
        }

        DashboardData {
            chart_values: values,
            chart_dates: dates,
            net_worth: format_money(total_balance, preferred_currency),
            max_value: format_money(max_val, preferred_currency),
            min_value: format_money(min_val, preferred_currency),
            expense_slices,
            total_income_cents,
            total_expense_cents,
            monthly_income,
            monthly_expense,
            monthly_labels,
        }
    }

    /// Gets expenses grouped by category
    pub fn get_expenses_by_category(
        transactions: &[Transaction],
        accounts: &[Account],
        usd_rates: &HashMap<String, f64>,
    ) -> Vec<(String, i64)> {
        let currency_map: HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            let rate = usd_rates
                .get(currency)
                .copied()
                .filter(|r| *r > 0.0)
                .unwrap_or(1.0);
            ((amount as f64) / rate).round() as i64
        };

        let mut map: HashMap<String, i64> = HashMap::new();

        for tx in transactions {
            if tx.transaction_type == "expense" {
                let amount = normalize(tx.amount, &tx.account_id);
                *map.entry(tx.category.clone()).or_default() += amount;
            }
        }

        let mut result: Vec<(String, i64)> = map.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    fn calculate_expense_slices(
        transactions: &[Transaction],
        currency_map: &HashMap<String, String>,
        usd_rates: &HashMap<String, f64>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<ExpenseSlice> {
        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            let rate = usd_rates
                .get(currency)
                .copied()
                .filter(|r| *r > 0.0)
                .unwrap_or(1.0);
            ((amount as f64) / rate).round() as i64
        };

        let mut expenses: HashMap<String, i64> = HashMap::new();

        for tx in transactions {
            if tx.transaction_type != "expense" {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, DATE_FORMAT)
                && date >= start_date
                && date <= end_date
            {
                let amount = normalize(tx.amount, &tx.account_id);
                *expenses.entry(tx.category.to_uppercase()).or_insert(0) += amount;
            }
        }

        let total_expense: i64 = expenses.values().sum();
        let mut expense_slices: Vec<ExpenseSlice> = Vec::new();

        if total_expense > 0 {
            let mut by_amount: Vec<(String, i64)> = expenses.into_iter().collect();
            by_amount.sort_by(|a, b| b.1.cmp(&a.1));

            for (idx, (category, amount)) in by_amount.iter().enumerate() {
                if *amount <= 0 {
                    continue;
                }
                let percentage = *amount as f32 / total_expense as f32;
                let color = CATEGORY_COLORS[idx % CATEGORY_COLORS.len()].to_string();

                expense_slices.push(ExpenseSlice {
                    category: category.clone(),
                    amount: *amount,
                    percentage,
                    color,
                });
            }
        }

        expense_slices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::validation::format_money_display;
    use crate::models::{Account, AccountBalance, Transaction};
    use std::collections::HashMap;

    fn usd_rates(rates: &[(&str, f64)]) -> HashMap<String, f64> {
        let mut map = HashMap::from([("USD".to_string(), 1.0)]);
        for (currency, rate) in rates {
            map.insert((*currency).to_string(), *rate);
        }
        map
    }

    fn make_account(id: &str, currency: &str, initial_balance: i64) -> Account {
        Account {
            id: id.to_string(),
            name: format!("Account {}", id),
            account_type: "bank".to_string(),
            currency: currency.to_string(),
            initial_balance,
            color: "#8b5cf6".to_string(),
            icon: None,
            is_archived: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    fn make_transaction(
        id: &str,
        account_id: &str,
        amount: i64,
        category: &str,
        tx_type: &str,
        date: &str,
    ) -> Transaction {
        Transaction {
            id: id.to_string(),
            account_id: account_id.to_string(),
            amount,
            category: category.to_string(),
            description: "Test".to_string(),
            date: date.to_string(),
            transaction_type: tx_type.to_string(),
            transfer_account_id: None,
        }
    }

    #[test]
    fn test_get_expenses_by_category_empty() {
        let transactions: Vec<Transaction> = vec![];
        let accounts: Vec<Account> = vec![];
        let rates = usd_rates(&[]);
        let result = DashboardCharts::get_expenses_by_category(&transactions, &accounts, &rates);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_expenses_by_category_single_expense() {
        let accounts = vec![make_account("acc1", "USD", 0)];
        let transactions = vec![make_transaction(
            "tx1",
            "acc1",
            5000,
            "Food",
            "expense",
            "2024-12-01",
        )];

        let rates = usd_rates(&[]);
        let result = DashboardCharts::get_expenses_by_category(&transactions, &accounts, &rates);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "Food");
        assert_eq!(result[0].1, 5000);
    }

    #[test]
    fn test_get_expenses_by_category_converts_non_usd_with_cached_rate() {
        let accounts = vec![make_account("acc1", "EUR", 0)];
        let transactions = vec![make_transaction(
            "tx1",
            "acc1",
            9200,
            "Food",
            "expense",
            "2024-12-01",
        )];
        let rates = usd_rates(&[("EUR", 0.92)]);

        let result = DashboardCharts::get_expenses_by_category(&transactions, &accounts, &rates);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "Food");
        // 92.00 EUR -> ~100.00 USD (cents)
        assert_eq!(result[0].1, 10000);
    }

    #[test]
    fn test_dashboard_data_converts_non_usd_balances_with_cached_rate() {
        let balances = vec![AccountBalance {
            account_id: "acc1".to_string(),
            account_name: "Euro".to_string(),
            current_balance: 9200,
            total_income: 9200,
            total_expense: 0,
        }];
        let accounts = vec![make_account("acc1", "EUR", 0)];
        let transactions: Vec<Transaction> = vec![];
        let snapshots: Vec<(String, f64, f64)> = vec![];
        let rates = usd_rates(&[("EUR", 0.92)]);

        let result = DashboardCharts::calculate_dashboard_data(
            &balances,
            &accounts,
            &transactions,
            0.0,
            &snapshots,
            &rates,
            "1M",
            "USD",
        );
        assert_eq!(result.net_worth, "USD 100.00");
    }

    #[test]
    fn test_dashboard_data_empty() {
        let balances: Vec<AccountBalance> = vec![];
        let accounts: Vec<Account> = vec![];
        let transactions: Vec<Transaction> = vec![];
        let snapshots: Vec<(String, f64, f64)> = vec![];
        let rates = usd_rates(&[]);

        let result = DashboardCharts::calculate_dashboard_data(
            &balances,
            &accounts,
            &transactions,
            0.0,
            &snapshots,
            &rates,
            "1M",
            "USD",
        );

        assert_eq!(result.net_worth, "USD 0.00");
        assert!(!result.chart_values.is_empty());
    }

    #[test]
    fn test_dashboard_data_with_crypto() {
        let balances: Vec<AccountBalance> = vec![];
        let accounts: Vec<Account> = vec![];
        let transactions: Vec<Transaction> = vec![];
        let snapshots: Vec<(String, f64, f64)> = vec![];
        let rates = usd_rates(&[]);

        let result = DashboardCharts::calculate_dashboard_data(
            &balances,
            &accounts,
            &transactions,
            500.0,
            &snapshots,
            &rates,
            "1M",
            "USD",
        );

        assert_eq!(result.net_worth, "USD 500.00");
    }

    #[test]
    fn test_dashboard_data_combined() {
        let balances = vec![AccountBalance {
            account_id: "acc1".to_string(),
            account_name: "Test".to_string(),
            current_balance: 10000,
            total_income: 10000,
            total_expense: 0,
        }];
        let accounts = vec![make_account("acc1", "USD", 0)];
        let transactions: Vec<Transaction> = vec![];
        let snapshots: Vec<(String, f64, f64)> = vec![];
        let rates = usd_rates(&[]);

        // $100 FIAT + $200 crypto = $300
        let result = DashboardCharts::calculate_dashboard_data(
            &balances,
            &accounts,
            &transactions,
            200.0,
            &snapshots,
            &rates,
            "1M",
            "USD",
        );

        assert_eq!(result.net_worth, "USD 300.00");
    }

    #[test]
    fn test_format_money_display() {
        assert_eq!(format_money_display(0), "$ 0.00");
        assert_eq!(format_money_display(100), "$ 1.00");
        assert_eq!(format_money_display(-500), "-$ 5.00");
    }
}
