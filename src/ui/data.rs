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

//! UI data types
//!
//! Intermediate data structures for UI display.
//! Used by DTOs and Tauri commands for data transformation.

use crate::controller::AppController;
use crate::ui::{
    format_category_label, format_decimal_from_cents, format_money, format_money_signed,
    normalize_bank_icon_path,
};
use std::collections::HashMap;
use std::sync::Arc;

// ==================== Account Display Data ====================

#[derive(Clone, Debug)]
pub struct AccountDisplayData {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub account_type_key: String,
    pub icon_path: Option<String>,
    pub currency: String,
    pub balance: String,
    pub balance_negative: bool,
    pub initial_balance: String,
    pub is_archived: bool,
}

#[derive(Clone, Debug)]
pub struct AccountsState {
    pub accounts: Vec<AccountDisplayData>,
    pub total_balance: String,
    pub total_balance_negative: bool,
}

pub fn load_accounts_state(
    controller: &Arc<AppController>,
    preferred_currency: &str,
) -> Result<AccountsState, String> {
    fn load_cached_usd_rate(controller: &AppController, currency: &str) -> f64 {
        let code = currency.trim().to_uppercase();
        if code == "USD" {
            return 1.0;
        }
        controller
            .load_exchange_rate_allow_stale(format!("{}_USD", code))
            .ok()
            .and_then(|rate| rate.map(|(r, _)| r))
            .filter(|r| *r > 0.0)
            .unwrap_or(1.0)
    }

    let preferred_currency = preferred_currency.trim().to_uppercase();
    let accounts = controller.get_accounts().map_err(|e| e.to_string())?;
    let balances = controller
        .get_account_balances()
        .map_err(|e| e.to_string())?;

    let mut balance_map: HashMap<String, i64> = HashMap::new();
    for bal in &balances {
        balance_map.insert(bal.account_id.clone(), bal.current_balance);
    }

    let currency_map: HashMap<String, String> = accounts
        .iter()
        .map(|acc| (acc.id.clone(), acc.currency.to_uppercase()))
        .collect();

    let mut usd_rates: HashMap<String, f64> = HashMap::from([("USD".to_string(), 1.0)]);
    for currency in currency_map.values() {
        if !usd_rates.contains_key(currency) {
            usd_rates.insert(currency.clone(), load_cached_usd_rate(controller, currency));
        }
    }
    let preferred_rate = if preferred_currency == "USD" {
        1.0
    } else {
        usd_rates
            .get(&preferred_currency)
            .copied()
            .unwrap_or_else(|| load_cached_usd_rate(controller, &preferred_currency))
    };

    // Calculate total in preferred currency
    let mut total_preferred_cents: i64 = 0;
    for bal in &balances {
        let acc_currency = currency_map
            .get(&bal.account_id)
            .map(|s| s.as_str())
            .unwrap_or("USD");

        let usd_rate = usd_rates.get(acc_currency).copied().unwrap_or(1.0);
        let usd_cents = (bal.current_balance as f64 / usd_rate).round() as i64;

        let preferred_cents = if preferred_currency == "USD" {
            usd_cents
        } else {
            (usd_cents as f64 * preferred_rate).round() as i64
        };
        total_preferred_cents += preferred_cents;
    }

    let mapped: Vec<AccountDisplayData> = accounts
        .iter()
        .map(|acc| {
            let current_balance = balance_map
                .get(&acc.id)
                .cloned()
                .unwrap_or(acc.initial_balance);

            let account_type = match acc.account_type.as_str() {
                "bank" | "Bank" => "Bank",
                "cash" | "Cash" => "Cash",
                "savings" | "Savings" => "Savings",
                "credit_card" | "CreditCard" => "Credit Card",
                "other" | "Other" => "Other",
                _ => acc.account_type.as_str(),
            };

            let is_bank = acc.account_type.eq_ignore_ascii_case("bank");
            let icon_path = if is_bank {
                normalize_bank_icon_path(acc.icon.clone())
            } else {
                None
            };

            AccountDisplayData {
                id: acc.id.clone(),
                name: acc.name.clone(),
                account_type: account_type.to_string(),
                account_type_key: acc.account_type.clone(),
                icon_path,
                currency: acc.currency.clone(),
                balance: format_money_signed(current_balance, &acc.currency),
                balance_negative: current_balance < 0,
                initial_balance: format_decimal_from_cents(acc.initial_balance),
                is_archived: acc.is_archived,
            }
        })
        .collect();

    Ok(AccountsState {
        accounts: mapped,
        total_balance: format_money_signed(total_preferred_cents, &preferred_currency),
        total_balance_negative: total_preferred_cents < 0,
    })
}

// ==================== Transaction Display Data ====================

#[derive(Clone, Debug)]
pub struct TransactionDisplayData {
    pub id: String,
    pub account_id: String,
    pub account_index: i32,
    pub account_name: String,
    pub amount: String,
    pub is_expense: bool,
    pub category: String,
    pub category_index: i32,
    pub description: String,
    pub date: String,
    pub related_tx_id: String,
}

#[derive(Clone, Debug)]
pub struct TransactionsState {
    pub transactions: Vec<TransactionDisplayData>,
    pub has_more: bool,
}

pub struct TransactionFilterParams<'a> {
    pub query: &'a str,
    pub account_filter: &'a str,
    pub category_filter: &'a str,
    pub display_limit: usize,
}

pub fn load_transactions_state(
    controller: &Arc<AppController>,
    filter: TransactionFilterParams<'_>,
) -> Result<TransactionsState, String> {
    let accounts = controller.get_accounts().map_err(|e| e.to_string())?;
    let mut account_lookup: HashMap<String, (String, String)> = HashMap::new();
    let mut account_index_map: HashMap<String, i32> = HashMap::new();
    for (idx, account) in accounts.iter().enumerate() {
        account_lookup.insert(
            account.id.clone(),
            (account.currency.clone(), account.name.clone()),
        );
        account_index_map.insert(account.id.clone(), idx as i32);
    }

    let expense_categories = controller
        .get_transaction_categories("expense".to_string())
        .map_err(|e| e.to_string())?;
    let income_categories = controller
        .get_transaction_categories("income".to_string())
        .map_err(|e| e.to_string())?;

    let expense_index_map: HashMap<String, i32> = expense_categories
        .iter()
        .enumerate()
        .map(|(idx, cat)| (cat.name.to_uppercase(), idx as i32))
        .collect();
    let income_index_map: HashMap<String, i32> = income_categories
        .iter()
        .enumerate()
        .map(|(idx, cat)| (cat.name.to_uppercase(), idx as i32))
        .collect();

    let query = filter.query.trim().to_lowercase();
    let account_filter = filter.account_filter.trim();
    let category_filter = filter.category_filter.trim();
    let category_filter_upper = category_filter.to_uppercase();
    let mut matched_count: usize = 0;

    let transactions = controller.get_transactions().map_err(|e| e.to_string())?;

    let mapped: Vec<TransactionDisplayData> = transactions
        .into_iter()
        .filter(|tx| {
            // Filter by account
            if !account_filter.is_empty() && tx.account_id != account_filter {
                return false;
            }
            // Filter by category
            if !category_filter.is_empty() && tx.category.to_uppercase() != category_filter_upper {
                return false;
            }
            // Filter by search query
            if !query.is_empty() {
                let desc_match = tx.description.to_lowercase().contains(&query);
                let cat_match = tx.category.to_lowercase().contains(&query);
                let date_match = tx.date.contains(&query);
                if !desc_match && !cat_match && !date_match {
                    return false;
                }
            }
            true
        })
        .take_while(|_| {
            matched_count += 1;
            matched_count <= filter.display_limit
        })
        .map(|tx| {
            let (currency, account_name) = account_lookup
                .get(&tx.account_id)
                .cloned()
                .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));

            let is_expense = tx.transaction_type == "expense";
            let category_index = if is_expense {
                expense_index_map
                    .get(&tx.category.to_uppercase())
                    .cloned()
                    .unwrap_or(-1)
            } else {
                income_index_map
                    .get(&tx.category.to_uppercase())
                    .cloned()
                    .unwrap_or(-1)
            };

            TransactionDisplayData {
                id: tx.id.clone(),
                account_id: tx.account_id.clone(),
                account_index: account_index_map.get(&tx.account_id).cloned().unwrap_or(-1),
                account_name,
                amount: format_money(tx.amount.abs(), &currency),
                is_expense,
                category: format_category_label(&tx.category),
                category_index,
                description: tx.description.clone(),
                date: tx.date.clone(),
                related_tx_id: tx.transfer_account_id.clone().unwrap_or_default(),
            }
        })
        .collect();

    let has_more = matched_count > filter.display_limit;

    Ok(TransactionsState {
        transactions: mapped,
        has_more,
    })
}

// ==================== Category Display Data ====================

#[derive(Clone, Debug)]
pub struct CategoryDisplayData {
    pub id: String,
    pub name: String,
    pub category_type: String,
    pub is_default: bool,
}

pub fn load_categories(
    controller: &Arc<AppController>,
    category_type: &str,
) -> Result<Vec<CategoryDisplayData>, String> {
    let categories = controller
        .get_transaction_categories(category_type.to_string())
        .map_err(|e| e.to_string())?;

    Ok(categories
        .into_iter()
        .map(|cat| CategoryDisplayData {
            id: cat.id,
            name: format_category_label(&cat.name),
            category_type: cat.category_type,
            is_default: cat.is_default,
        })
        .collect())
}

// ==================== Dashboard/Balance Data ====================

#[derive(Clone, Debug)]
pub struct BalanceDisplayData {
    pub current_balance: String,
    pub total_income: String,
    pub total_expenses: String,
}

pub fn load_balance_data(controller: &Arc<AppController>) -> Result<BalanceDisplayData, String> {
    let balance = controller.get_balance().map_err(|e| e.to_string())?;

    Ok(BalanceDisplayData {
        current_balance: format_money(balance.total_balance, "USD"),
        total_income: format_money(balance.total_income, "USD"),
        total_expenses: format_money(balance.total_expense, "USD"),
    })
}

// ==================== Recent Transactions ====================

#[derive(Clone, Debug)]
pub struct RecentTransactionData {
    pub id: String,
    pub description: String,
    pub amount: String,
    pub is_expense: bool,
    pub date: String,
    pub category: String,
}

pub fn load_recent_transactions(
    controller: &Arc<AppController>,
    limit: usize,
) -> Result<Vec<RecentTransactionData>, String> {
    let accounts = controller.get_accounts().map_err(|e| e.to_string())?;
    let currency_map: HashMap<String, String> = accounts
        .iter()
        .map(|acc| (acc.id.clone(), acc.currency.clone()))
        .collect();

    let transactions = controller.get_transactions().map_err(|e| e.to_string())?;

    Ok(transactions
        .into_iter()
        .take(limit)
        .map(|tx| {
            let currency = currency_map
                .get(&tx.account_id)
                .cloned()
                .unwrap_or_else(|| "USD".to_string());

            RecentTransactionData {
                id: tx.id,
                description: tx.description,
                amount: format_money(tx.amount.abs(), &currency),
                is_expense: tx.transaction_type == "expense",
                date: tx.date,
                category: format_category_label(&tx.category),
            }
        })
        .collect())
}
