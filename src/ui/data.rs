// Sanctum — a privacy-first personal finance and crypto vault.
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

use crate::features::finance::FinanceService;
use crate::ui::{
    format_category_label, format_decimal_from_cents, format_money, format_money_signed,
    normalize_bank_icon_path,
};
use std::collections::HashMap;

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
    finance: &FinanceService,
    preferred_currency: &str,
) -> Result<AccountsState, String> {
    fn load_cached_usd_rate(finance: &FinanceService, currency: &str) -> f64 {
        let code = currency.trim().to_uppercase();
        if code == "USD" {
            return 1.0;
        }
        finance
            .load_exchange_rate_allow_stale(format!("{}_USD", code))
            .ok()
            .and_then(|rate| rate.map(|(r, _)| r))
            .filter(|r| *r > 0.0)
            .unwrap_or(1.0)
    }

    let preferred_currency = preferred_currency.trim().to_uppercase();
    let accounts = finance.get_accounts().map_err(|e| e.to_string())?;
    let balances = finance.get_account_balances().map_err(|e| e.to_string())?;

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
            usd_rates.insert(currency.clone(), load_cached_usd_rate(finance, currency));
        }
    }
    let preferred_rate = if preferred_currency == "USD" {
        1.0
    } else {
        usd_rates
            .get(&preferred_currency)
            .copied()
            .unwrap_or_else(|| load_cached_usd_rate(finance, &preferred_currency))
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

            let icon_path = normalize_bank_icon_path(acc.icon.clone());

            AccountDisplayData {
                id: acc.id.clone(),
                name: acc.name.clone(),
                account_type: acc.account_type.clone(),
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
    finance: &FinanceService,
    filter: TransactionFilterParams<'_>,
) -> Result<TransactionsState, String> {
    let accounts = finance.get_accounts().map_err(|e| e.to_string())?;
    let mut account_lookup: HashMap<String, (String, String)> = HashMap::new();
    let mut account_index_map: HashMap<String, i32> = HashMap::new();
    for (idx, account) in accounts.iter().enumerate() {
        account_lookup.insert(
            account.id.clone(),
            (account.currency.clone(), account.name.clone()),
        );
        account_index_map.insert(account.id.clone(), idx as i32);
    }

    let expense_categories = finance
        .get_transaction_categories("expense".to_string())
        .map_err(|e| e.to_string())?;
    let income_categories = finance
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

    let transactions = finance.get_transactions().map_err(|e| e.to_string())?;

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
    finance: &FinanceService,
    category_type: &str,
) -> Result<Vec<CategoryDisplayData>, String> {
    let categories = finance
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

pub fn load_balance_data(finance: &FinanceService) -> Result<BalanceDisplayData, String> {
    let balance = finance.get_balance().map_err(|e| e.to_string())?;

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
    finance: &FinanceService,
    limit: usize,
) -> Result<Vec<RecentTransactionData>, String> {
    let accounts = finance.get_accounts().map_err(|e| e.to_string())?;
    let currency_map: HashMap<String, String> = accounts
        .iter()
        .map(|acc| (acc.id.clone(), acc.currency.clone()))
        .collect();

    let transactions = finance.get_transactions().map_err(|e| e.to_string())?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::vault_manager::VaultManager;
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};
    use uuid::Uuid;

    struct TestHarness {
        finance: FinanceService,
        test_dir: PathBuf,
    }

    impl Drop for TestHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.test_dir);
        }
    }

    fn new_harness() -> TestHarness {
        let base_dir =
            std::env::temp_dir().join(format!("sanctum-ui-data-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&base_dir).expect("create test dir");
        // Build the shared db handle, create the vault via the manager, then
        // operate through a FinanceService that shares the same handle.
        let db: Arc<RwLock<Option<Database>>> = Arc::new(RwLock::new(None));
        let vault = VaultManager::new(base_dir.clone(), db.clone());
        let finance = FinanceService::new(db);
        let password = "test-password-123".to_string();
        vault.create_db(password, None).expect("create vault");
        TestHarness {
            finance,
            test_dir: base_dir,
        }
    }

    fn create_account(finance: &FinanceService, name: &str, currency: &str, balance: i64) {
        finance
            .create_account(crate::features::finance::NewAccount {
                name: name.to_string(),
                account_type: "bank".to_string(),
                currency: currency.to_string(),
                initial_balance_cents: balance,
                color: "#8b5cf6".to_string(),
                icon: None,
            })
            .expect("create account");
    }

    fn add_transaction(
        finance: &FinanceService,
        account_id: &str,
        amount: i64,
        category: &str,
        is_expense: bool,
    ) {
        finance
            .add_transaction(crate::features::finance::NewTransaction {
                account_id: account_id.to_string(),
                amount_cents: amount,
                category: category.to_string(),
                description: "test".to_string(),
                date: "2024-06-15".to_string(),
                is_expense,
            })
            .expect("add transaction");
    }

    #[test]
    fn test_load_categories_for_expense() {
        let h = new_harness();
        let result = load_categories(&h.finance, "expense").expect("load categories");
        assert!(
            !result.is_empty(),
            "default expense categories should exist"
        );
        assert!(result.iter().all(|c| c.category_type == "expense"));
    }

    #[test]
    fn test_load_categories_for_income() {
        let h = new_harness();
        let result = load_categories(&h.finance, "income").expect("load categories");
        assert!(!result.is_empty(), "default income categories should exist");
        assert!(result.iter().all(|c| c.category_type == "income"));
    }

    #[test]
    fn test_load_categories_rejects_invalid_type() {
        let h = new_harness();
        let result = load_categories(&h.finance, "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_balance_data_empty() {
        let h = new_harness();
        let result = load_balance_data(&h.finance).expect("load balance");
        assert_eq!(result.current_balance, "USD 0.00");
        assert_eq!(result.total_income, "USD 0.00");
        assert_eq!(result.total_expenses, "USD 0.00");
    }

    #[test]
    fn test_load_balance_data_with_transactions() {
        let h = new_harness();
        create_account(&h.finance, "Checking", "USD", 0);
        let accounts = h.finance.get_accounts().expect("get accounts");
        let acc_id = &accounts
            .iter()
            .find(|a| a.name == "Checking")
            .expect("find account")
            .id;
        add_transaction(&h.finance, acc_id, 20000, "Salary", false);
        add_transaction(&h.finance, acc_id, 5000, "Food", true);
        let result = load_balance_data(&h.finance).expect("load balance");
        assert_eq!(result.total_income, "USD 200.00");
        assert_eq!(result.total_expenses, "USD 50.00");
    }

    #[test]
    fn test_load_accounts_state_empty() {
        let h = new_harness();
        let result = load_accounts_state(&h.finance, "USD").expect("load accounts state");
        assert!(result.accounts.is_empty(), "new vault has no user accounts");
        assert_eq!(result.total_balance, "USD 0.00");
    }

    #[test]
    fn test_load_accounts_state_with_accounts() {
        let h = new_harness();
        create_account(&h.finance, "Checking", "USD", 0);
        create_account(&h.finance, "Savings", "USD", 0);
        let accounts = h.finance.get_accounts().expect("get accounts");
        let checking = accounts
            .iter()
            .find(|a| a.name == "Checking")
            .expect("find checking");
        let savings = accounts
            .iter()
            .find(|a| a.name == "Savings")
            .expect("find savings");
        add_transaction(&h.finance, &checking.id, 50000, "Deposit", false);
        add_transaction(&h.finance, &savings.id, 100000, "Deposit", false);
        let result = load_accounts_state(&h.finance, "USD").expect("load accounts state");
        assert_eq!(result.accounts.len(), 2);
        assert_eq!(
            result.total_balance, "USD 1,500.00",
            "total should be sum of account balances"
        );
    }

    #[test]
    fn test_load_transactions_state_empty() {
        let h = new_harness();
        let filter = TransactionFilterParams {
            query: "",
            account_filter: "",
            category_filter: "",
            display_limit: 50,
        };
        let result = load_transactions_state(&h.finance, filter).expect("load transactions");
        assert!(result.transactions.is_empty());
        assert!(!result.has_more);
    }

    #[test]
    fn test_load_transactions_state_filters_by_account() {
        let h = new_harness();
        create_account(&h.finance, "A", "USD", 0);
        create_account(&h.finance, "B", "USD", 0);
        let accounts = h.finance.get_accounts().expect("get accounts");
        let acc_a = accounts.iter().find(|a| a.name == "A").expect("find A");
        let acc_b = accounts.iter().find(|a| a.name == "B").expect("find B");
        add_transaction(&h.finance, &acc_a.id, 1000, "Food", true);
        add_transaction(&h.finance, &acc_b.id, 2000, "Food", true);
        let filter = TransactionFilterParams {
            query: "",
            account_filter: &acc_a.id,
            category_filter: "",
            display_limit: 50,
        };
        let result = load_transactions_state(&h.finance, filter).expect("load transactions");
        assert_eq!(result.transactions.len(), 1);
        assert_eq!(result.transactions[0].account_id, acc_a.id);
    }

    #[test]
    fn test_load_transactions_state_filters_by_query() {
        let h = new_harness();
        create_account(&h.finance, "Test", "USD", 0);
        let accounts = h.finance.get_accounts().expect("get accounts");
        let acc_id = &accounts[0].id;
        add_transaction(&h.finance, acc_id, 1000, "Food", true);
        add_transaction(&h.finance, acc_id, 2000, "Salary", false);
        let filter = TransactionFilterParams {
            query: "salary",
            account_filter: "",
            category_filter: "",
            display_limit: 50,
        };
        let result = load_transactions_state(&h.finance, filter).expect("load transactions");
        assert_eq!(result.transactions.len(), 1);
        assert!(
            result.transactions[0]
                .category
                .to_lowercase()
                .contains("salary")
        );
    }

    #[test]
    fn test_load_transactions_state_respects_limit() {
        let h = new_harness();
        create_account(&h.finance, "Test", "USD", 0);
        let accounts = h.finance.get_accounts().expect("get accounts");
        let acc_id = &accounts[0].id;
        for i in 0..5 {
            add_transaction(&h.finance, acc_id, 1000 + i, "Food", true);
        }
        let filter = TransactionFilterParams {
            query: "",
            account_filter: "",
            category_filter: "",
            display_limit: 3,
        };
        let result = load_transactions_state(&h.finance, filter).expect("load transactions");
        assert_eq!(result.transactions.len(), 3);
        assert!(result.has_more);
    }

    #[test]
    fn test_load_recent_transactions_respects_limit() {
        let h = new_harness();
        create_account(&h.finance, "Test", "USD", 0);
        let accounts = h.finance.get_accounts().expect("get accounts");
        let acc_id = &accounts[0].id;
        for i in 0..5 {
            add_transaction(&h.finance, acc_id, 1000 + i, "Food", true);
        }
        let result = load_recent_transactions(&h.finance, 2).expect("load recent transactions");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_load_recent_transactions_empty() {
        let h = new_harness();
        let result = load_recent_transactions(&h.finance, 10).expect("load recent");
        assert!(result.is_empty());
    }
}
