//! Dashboard charts data calculation
//!
//! Net worth calculations and expense breakdowns.
//! Aggregates financial and crypto data for dashboard visualization.
//! Note: Actual chart rendering is done by services/charts.rs

use crate::core::validation::format_money_display;
use crate::models::{Account, AccountBalance, Transaction};
use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashMap;

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
    pub net_worth: String,
    pub max_value: String,
    pub min_value: String,
    pub expense_slices: Vec<ExpenseSlice>,
}

/// Chart data calculation for dashboard
pub struct DashboardCharts;

impl DashboardCharts {
    /// Calculates dashboard data (FIAT + Crypto combined)
    /// Returns raw values - use controller.render_net_worth_chart() for the image
    pub fn calculate_dashboard_data(
        balances: &[AccountBalance],
        accounts: &[Account],
        transactions: &[Transaction],
        crypto_total_usd: f64,
        clp_rate: f64,
        range: &str,
    ) -> DashboardData {
        let currency_map: HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let rate = if clp_rate > 0.0 { clp_rate } else { 1.0 };

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
        };

        // Calculate FIAT balance in cents
        let fiat_balance: i64 = balances
            .iter()
            .map(|b| normalize(b.current_balance, &b.account_id))
            .sum();

        // Convert crypto to cents and add to total
        let crypto_cents = (crypto_total_usd * 100.0) as i64;
        let total_balance = fiat_balance + crypto_cents;

        let today = Local::now().date_naive();
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
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d")
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
            let total_at_date = if cursor == today {
                balance + crypto_cents
            } else {
                balance
            };
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

        let values: Vec<i64> = points_rev.iter().map(|(_, v)| *v).collect();
        let min_val = *values.iter().min().unwrap_or(&0);
        let max_val = *values.iter().max().unwrap_or(&0);

        let expense_slices =
            Self::calculate_expense_slices(transactions, &currency_map, rate, today);

        DashboardData {
            chart_values: values,
            net_worth: format_money_display(total_balance),
            max_value: format_money_display(max_val),
            min_value: format_money_display(min_val),
            expense_slices,
        }
    }

    /// Gets expenses grouped by category
    pub fn get_expenses_by_category(
        transactions: &[Transaction],
        accounts: &[Account],
        clp_rate: f64,
    ) -> Vec<(String, i64)> {
        let currency_map: HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let rate = if clp_rate > 0.0 { clp_rate } else { 1.0 };

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
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
        rate: f64,
        today: NaiveDate,
    ) -> Vec<ExpenseSlice> {
        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
        };

        let mut expenses: HashMap<String, i64> = HashMap::new();
        let current_month = today.month();
        let current_year = today.year();

        for tx in transactions {
            if tx.transaction_type != "expense" {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d")
                && date.year() == current_year
                && date.month() == current_month
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

            let colors = [
                "#8b5cf6", "#ec4899", "#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#6366f1",
                "#14b8a6",
            ];

            for (idx, (category, amount)) in by_amount.iter().enumerate() {
                if *amount <= 0 {
                    continue;
                }
                let percentage = *amount as f32 / total_expense as f32;
                let color = colors[idx % colors.len()].to_string();

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
        let result = DashboardCharts::get_expenses_by_category(&transactions, &accounts, 1.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_expenses_by_category_single_expense() {
        let accounts = vec![make_account("acc1", "USD", 0)];
        let transactions = vec![make_transaction(
            "tx1", "acc1", 5000, "Food", "expense", "2024-12-01",
        )];

        let result = DashboardCharts::get_expenses_by_category(&transactions, &accounts, 1.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "Food");
        assert_eq!(result[0].1, 5000);
    }

    #[test]
    fn test_dashboard_data_empty() {
        let balances: Vec<AccountBalance> = vec![];
        let accounts: Vec<Account> = vec![];
        let transactions: Vec<Transaction> = vec![];

        let result = DashboardCharts::calculate_dashboard_data(
            &balances, &accounts, &transactions, 0.0, 1.0, "1M",
        );

        assert_eq!(result.net_worth, "$ 0.00");
        assert!(!result.chart_values.is_empty());
    }

    #[test]
    fn test_dashboard_data_with_crypto() {
        let balances: Vec<AccountBalance> = vec![];
        let accounts: Vec<Account> = vec![];
        let transactions: Vec<Transaction> = vec![];

        let result = DashboardCharts::calculate_dashboard_data(
            &balances, &accounts, &transactions, 500.0, 1.0, "1M",
        );

        assert_eq!(result.net_worth, "$ 500.00");
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

        // $100 FIAT + $200 crypto = $300
        let result = DashboardCharts::calculate_dashboard_data(
            &balances, &accounts, &transactions, 200.0, 1.0, "1M",
        );

        assert_eq!(result.net_worth, "$ 300.00");
    }

    #[test]
    fn test_format_money_display() {
        assert_eq!(format_money_display(0), "$ 0.00");
        assert_eq!(format_money_display(100), "$ 1.00");
        assert_eq!(format_money_display(-500), "-$ 5.00");
    }
}
