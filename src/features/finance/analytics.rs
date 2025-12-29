//! Finance analytics and reporting
//!
//! Net worth calculations, expense breakdowns, and chart data generation.

use crate::models::{Account, AccountBalance, Transaction};
use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashMap;

use super::validation::format_money_display;

#[derive(Debug, Clone)]
pub struct ExpenseSlice {
    pub category: String,
    pub amount: i64,
    pub percentage: f32,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct AnalyticsSummary {
    pub chart_path: String,
    pub net_worth: String,
    pub max_value: String,
    pub min_value: String,
    pub expense_slices: Vec<ExpenseSlice>,
}

/// Analytics operations for finance
pub struct FinanceAnalytics;

impl FinanceAnalytics {
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

    pub fn get_analytics_summary(
        balances: &[AccountBalance],
        accounts: &[Account],
        transactions: &[Transaction],
        clp_rate: f64,
        range: &str,
    ) -> AnalyticsSummary {
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

        let current_balance: i64 = balances
            .iter()
            .map(|b| normalize(b.current_balance, &b.account_id))
            .sum();

        let today = Local::now().date_naive();
        let start_date = match range {
            "1M" => today
                .checked_sub_signed(chrono::Duration::days(30))
                .unwrap_or(today),
            "3M" => today
                .checked_sub_signed(chrono::Duration::days(90))
                .unwrap_or(today),
            "6M" => today
                .checked_sub_signed(chrono::Duration::days(180))
                .unwrap_or(today),
            "1Y" => today
                .checked_sub_signed(chrono::Duration::days(365))
                .unwrap_or(today),
            _ => today,
        };

        let mut delta_by_day: HashMap<NaiveDate, i64> = HashMap::new();
        let mut earliest_tx: Option<NaiveDate> = None;
        for tx in transactions {
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d") {
                let raw_delta = match tx.transaction_type.as_str() {
                    "income" => tx.amount,
                    "expense" => -tx.amount,
                    _ => 0,
                };
                let delta = normalize(raw_delta, &tx.account_id);
                *delta_by_day.entry(date).or_insert(0) += delta;
                earliest_tx = Some(earliest_tx.map_or(date, |d| d.min(date)));
            }
        }

        let effective_start = if range == "ALL" {
            earliest_tx.unwrap_or(today)
        } else {
            start_date.min(today)
        };

        let mut cursor = today;
        let mut points_rev: Vec<(NaiveDate, i64)> = Vec::new();
        let mut balance = current_balance;

        loop {
            points_rev.push((cursor, balance));
            let delta = *delta_by_day.get(&cursor).unwrap_or(&0);
            balance -= delta;

            if cursor <= effective_start {
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
            points_rev.push((today, current_balance));
        }

        let values: Vec<i64> = points_rev.iter().map(|(_, v)| *v).collect();
        let min_val = *values.iter().min().unwrap_or(&0);
        let max_val = *values.iter().max().unwrap_or(&0);
        let safe_range = ((max_val - min_val) as f64).max(1.0);

        let points: Vec<(f64, f64)> = values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = (i as f64 / (values.len().max(2) - 1) as f64) * 100.0;
                let y_ratio = (v - min_val) as f64 / safe_range;
                let y = 100.0 - (5.0 + (y_ratio * 90.0));
                (x, y)
            })
            .collect();

        let path_cmd = Self::generate_smooth_path(&points);

        let expense_slices =
            Self::calculate_expense_slices(transactions, &currency_map, rate, today);

        AnalyticsSummary {
            chart_path: path_cmd,
            net_worth: format_money_display(current_balance),
            max_value: format_money_display(max_val),
            min_value: format_money_display(min_val),
            expense_slices,
        }
    }

    fn generate_smooth_path(points: &[(f64, f64)]) -> String {
        let mut path_cmd = String::new();
        if !points.is_empty() {
            path_cmd.push_str(&format!("M {:.2} {:.2}", points[0].0, points[0].1));

            for i in 0..points.len() - 1 {
                let p0 = if i == 0 { points[0] } else { points[i - 1] };
                let p1 = points[i];
                let p2 = points[i + 1];
                let p3 = if i + 2 < points.len() {
                    points[i + 2]
                } else {
                    p2
                };

                let cp1x = p1.0 + (p2.0 - p0.0) / 6.0;
                let cp1y = p1.1 + (p2.1 - p0.1) / 6.0;
                let cp2x = p2.0 - (p3.0 - p1.0) / 6.0;
                let cp2y = p2.1 - (p3.1 - p1.1) / 6.0;

                path_cmd.push_str(&format!(
                    " C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                    cp1x, cp1y, cp2x, cp2y, p2.0, p2.1
                ));
            }
        } else {
            path_cmd = "M 0 50 L 100 50".to_string();
        }
        path_cmd
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

    pub fn get_net_worth_history(
        accounts: &[Account],
        transactions: &[Transaction],
        clp_rate: f64,
        range: &str,
    ) -> (String, String, String, String) {
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

        struct FinancialEvent {
            date: NaiveDate,
            amount_delta: i64,
        }

        let mut events: Vec<FinancialEvent> = Vec::new();

        for acc in accounts {
            if acc.initial_balance != 0 {
                let date = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&acc.created_at) {
                    dt.date_naive()
                } else if let Ok(d) = NaiveDate::parse_from_str(&acc.created_at, "%Y-%m-%d") {
                    d
                } else {
                    Local::now().date_naive()
                };

                let amount_delta = normalize(acc.initial_balance, &acc.id);
                events.push(FinancialEvent { date, amount_delta });
            }
        }

        for tx in transactions {
            let raw_delta = match tx.transaction_type.as_str() {
                "income" => tx.amount,
                "expense" => -tx.amount,
                _ => 0,
            };

            if raw_delta == 0 {
                continue;
            }

            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d") {
                let amount_delta = normalize(raw_delta, &tx.account_id);
                events.push(FinancialEvent { date, amount_delta });
            }
        }

        events.sort_by(|a, b| a.date.cmp(&b.date));

        let mut full_history: Vec<(NaiveDate, i64)> = Vec::new();
        let mut current_balance = 0;

        if let Some(first) = events.first() {
            full_history.push((first.date.pred_opt().unwrap_or(first.date), 0));
        } else {
            full_history.push((Local::now().date_naive(), 0));
        }

        for event in events {
            current_balance += event.amount_delta;
            full_history.push((event.date, current_balance));
        }

        let today = Local::now().date_naive();
        if full_history.last().is_some_and(|last| last.0 < today) {
            full_history.push((today, current_balance));
        }

        let net_worth_formatted = format_money_display(current_balance);

        let start_date = match range {
            "1M" => Some(today - chrono::Duration::days(30)),
            "3M" => Some(today - chrono::Duration::days(90)),
            "6M" => Some(today - chrono::Duration::days(180)),
            "1Y" => Some(today - chrono::Duration::days(365)),
            _ => None,
        };

        let filtered_history: Vec<(NaiveDate, i64)> = if let Some(start) = start_date {
            let start_balance = full_history
                .iter()
                .rfind(|(d, _)| *d <= start)
                .map(|(_, b)| *b)
                .unwrap_or(0);

            let mut range_points: Vec<(NaiveDate, i64)> = Vec::new();
            range_points.push((start, start_balance));
            range_points.extend(full_history.into_iter().filter(|(d, _)| *d >= start));
            range_points
        } else {
            full_history
        };

        if filtered_history.is_empty() {
            return (
                "M 0 50 L 100 50".to_string(),
                net_worth_formatted,
                "$ 0.00".to_string(),
                "$ 0.00".to_string(),
            );
        }

        let balances: Vec<i64> = filtered_history.iter().map(|(_, b)| *b).collect();
        let min_val = *balances.iter().min().unwrap_or(&0);
        let max_val = *balances.iter().max().unwrap_or(&0);

        let min_formatted = format_money_display(min_val);
        let max_formatted = format_money_display(max_val);

        let len = balances.len() as f32;
        let mut path_cmd = String::new();

        let range_val = (max_val - min_val) as f32;
        let safe_range = if range_val == 0.0 { 1.0 } else { range_val };

        for (idx, val) in balances.iter().enumerate() {
            let x = if len > 1.0 {
                (idx as f32) * (100.0 / (len - 1.0))
            } else {
                0.0
            };

            let y_norm = if max_val == min_val {
                50.0
            } else {
                let ratio = (*val - min_val) as f32 / safe_range;
                100.0 - (5.0 + (ratio * 90.0))
            };

            if idx == 0 {
                path_cmd.push_str(&format!("M {:.2} {:.2}", x, y_norm));
            } else {
                path_cmd.push_str(&format!(" L {:.2} {:.2}", x, y_norm));
            }
        }

        if path_cmd.is_empty() {
            path_cmd = "M 0 50 L 100 50".to_string();
        }

        (path_cmd, net_worth_formatted, min_formatted, max_formatted)
    }
}
