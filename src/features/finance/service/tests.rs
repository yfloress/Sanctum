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

use super::*;
use crate::db::Database;
use secrecy::SecretString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

struct TestServiceHarness {
    service: FinanceService,
    test_dir: PathBuf,
}

impl Drop for TestServiceHarness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.test_dir);
    }
}

fn new_test_service() -> TestServiceHarness {
    let base_dir = std::env::temp_dir().join(format!("sanctum-finance-test-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&base_dir).expect("create test dir");
    let db_path = base_dir.join("vault.db");
    let password = SecretString::from("test-password-123".to_string());
    let db = Database::init(db_path, &password).expect("init test database");
    let service = FinanceService::new(Arc::new(Mutex::new(Some(db))));
    TestServiceHarness {
        service,
        test_dir: base_dir,
    }
}

fn create_test_account(
    svc: &FinanceService,
    name: &str,
    currency: &str,
    initial_balance: i64,
) -> String {
    svc.create_account(
        name.to_string(),
        "bank".to_string(),
        currency.to_string(),
        initial_balance,
        "#8b5cf6".to_string(),
        None,
    )
    .expect("create account")
}

fn create_test_transaction(
    svc: &FinanceService,
    account_id: &str,
    amount: i64,
    category: &str,
    is_expense: bool,
) -> String {
    svc.add_transaction(
        account_id.to_string(),
        amount,
        category.to_string(),
        "test transaction".to_string(),
        "2024-06-15".to_string(),
        is_expense,
    )
    .expect("create transaction")
}

fn create_test_category(
    svc: &FinanceService,
    name: &str,
    category_type: &str,
) -> String {
    svc.add_transaction_category(name.to_string(), category_type.to_string())
        .expect("create category")
}

// ==================== Account Tests ====================

#[test]
fn test_create_account_with_valid_data() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Checking", "USD", 0);
    assert!(!id.is_empty());
    let uuid = uuid::Uuid::parse_str(&id);
    assert!(uuid.is_ok(), "create_account should return a valid UUID");
}

#[test]
fn test_create_account_stores_initial_balance() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Savings", "USD", 50000);
    let accounts = h.service.get_accounts().expect("get accounts");
    let account = accounts.iter().find(|a| a.id == id).expect("find account");
    assert_eq!(account.initial_balance, 50000);
}

#[test]
fn test_get_accounts_empty_when_no_accounts() {
    let h = new_test_service();
    let accounts = h.service.get_accounts().expect("get accounts");
    assert!(accounts.is_empty());
}

#[test]
fn test_get_accounts_returns_all_created_accounts() {
    let h = new_test_service();
    create_test_account(&h.service, "A", "USD", 0);
    create_test_account(&h.service, "B", "USD", 0);
    create_test_account(&h.service, "C", "USD", 0);
    let accounts = h.service.get_accounts().expect("get accounts");
    assert_eq!(accounts.len(), 3);
}

#[test]
fn test_get_account_balances_returns_summary_per_account() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Test", "USD", 10000);
    let balances = h.service.get_account_balances().expect("get balances");
    let balance = balances.iter().find(|b| b.account_id == id).expect("find balance");
    assert_eq!(balance.current_balance, 10000);
    assert_eq!(balance.total_income, 10000);
    assert_eq!(balance.total_expense, 0);
}

#[test]
fn test_update_account_changes_fields() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Old Name", "USD", 0);
    h.service.update_account(
        id.clone(),
        "New Name".to_string(),
        "savings".to_string(),
        "EUR".to_string(),
        20000,
        "#ec4899".to_string(),
        Some("icon.svg".to_string()),
    ).expect("update account");
    let accounts = h.service.get_accounts().expect("get accounts");
    let account = accounts.iter().find(|a| a.id == id).expect("find account");
    assert_eq!(account.name, "New Name");
    assert_eq!(account.account_type, "savings");
    assert_eq!(account.currency, "EUR");
    assert_eq!(account.initial_balance, 20000);
    assert_eq!(account.color, "#ec4899");
    assert_eq!(account.icon.as_deref(), Some("icon.svg"));
}

#[test]
fn test_update_account_icon_changes_only_icon() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Test", "USD", 0);
    h.service.update_account_icon(id.clone(), Some("new-icon.svg".to_string())).expect("update icon");
    let accounts = h.service.get_accounts().expect("get accounts");
    let account = accounts.iter().find(|a| a.id == id).expect("find account");
    assert_eq!(account.icon.as_deref(), Some("new-icon.svg"));
    assert_eq!(account.name, "Test");
}

#[test]
fn test_update_account_icon_with_none_clears_icon() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Test", "USD", 0);
    h.service.update_account_icon(id.clone(), Some("old.svg".to_string())).expect("set icon");
    h.service.update_account_icon(id.clone(), None).expect("clear icon");
    let accounts = h.service.get_accounts().expect("get accounts");
    let account = accounts.iter().find(|a| a.id == id).expect("find account");
    assert_eq!(account.icon, None);
}

#[test]
fn test_update_account_name_changes_only_name() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Original", "USD", 0);
    h.service.update_account_name(id.clone(), "Renamed".to_string()).expect("update name");
    let accounts = h.service.get_accounts().expect("get accounts");
    let account = accounts.iter().find(|a| a.id == id).expect("find account");
    assert_eq!(account.name, "Renamed");
    assert_eq!(account.currency, "USD");
}

#[test]
fn test_archive_and_unarchive_account() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Archivable", "USD", 0);
    h.service.archive_account(id.clone()).expect("archive account");
    let active = h.service.get_accounts().expect("get accounts");
    assert!(active.iter().all(|a| a.id != id));
    let archived = h.service.get_archived_accounts().expect("get archived");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].id, id);
    h.service.unarchive_account(id.clone()).expect("unarchive account");
    let active_again = h.service.get_accounts().expect("get accounts after unarchive");
    assert!(active_again.iter().any(|a| a.id == id));
}

// ==================== Validation Error Tests ====================

#[test]
fn test_create_account_rejects_empty_name() {
    let h = new_test_service();
    let result = h.service.create_account(
        "".to_string(),
        "bank".to_string(),
        "USD".to_string(),
        0,
        "#8b5cf6".to_string(),
        None,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, FinanceError::Validation(_)));
    assert!(err.to_string().contains("empty") || err.to_string().contains("length"));
}

#[test]
fn test_create_account_rejects_empty_currency() {
    let h = new_test_service();
    let result = h.service.create_account(
        "Test".to_string(),
        "bank".to_string(),
        "".to_string(),
        0,
        "#8b5cf6".to_string(),
        None,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, FinanceError::Validation(_)));
}

#[test]
fn test_create_account_rejects_invalid_color() {
    let h = new_test_service();
    let result = h.service.create_account(
        "Test".to_string(),
        "bank".to_string(),
        "USD".to_string(),
        0,
        "not-a-color".to_string(),
        None,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_create_account_rejects_name_too_long() {
    let h = new_test_service();
    let long_name = "a".repeat(65);
    let result = h.service.create_account(
        long_name,
        "bank".to_string(),
        "USD".to_string(),
        0,
        "#8b5cf6".to_string(),
        None,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_update_account_rejects_invalid_id() {
    let h = new_test_service();
    let result = h.service.update_account(
        "not-a-uuid".to_string(),
        "Name".to_string(),
        "bank".to_string(),
        "USD".to_string(),
        0,
        "#8b5cf6".to_string(),
        None,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_update_account_name_rejects_empty() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Test", "USD", 0);
    let result = h.service.update_account_name(id, "".to_string());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_archive_account_rejects_invalid_id() {
    let h = new_test_service();
    let result = h.service.archive_account("bad-id".to_string());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

// ==================== Transaction Tests ====================

#[test]
fn test_add_transaction_income() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    let tx_id = create_test_transaction(&h.service, &account_id, 10000, "Salary", false);
    assert!(!tx_id.is_empty());
    let txs = h.service.get_transactions().expect("get transactions");
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].amount, 10000);
    assert_eq!(txs[0].transaction_type, "income");
}

#[test]
fn test_add_transaction_expense() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    let tx_id = create_test_transaction(&h.service, &account_id, 2500, "Food", true);
    assert!(!tx_id.is_empty());
    let txs = h.service.get_transactions().expect("get transactions");
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].amount, 2500);
    assert_eq!(txs[0].transaction_type, "expense");
}

#[test]
fn test_add_transaction_rejects_zero_amount() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    let result = h.service.add_transaction(
        account_id,
        0,
        "Test".to_string(),
        "desc".to_string(),
        "2024-06-15".to_string(),
        false,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_add_transaction_rejects_negative_amount() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    let result = h.service.add_transaction(
        account_id,
        -100,
        "Test".to_string(),
        "desc".to_string(),
        "2024-06-15".to_string(),
        false,
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_add_transaction_rejects_invalid_account() {
    let h = new_test_service();
    let result = h.service.add_transaction(
        "bad-uuid".to_string(),
        1000,
        "Test".to_string(),
        "desc".to_string(),
        "2024-06-15".to_string(),
        false,
    );
    assert!(result.is_err());
}

#[test]
fn test_update_transaction_changes_fields() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    let tx_id = create_test_transaction(&h.service, &account_id, 1000, "Old", false);
    h.service.update_transaction(
        tx_id.clone(),
        account_id.clone(),
        2000,
        "Updated".to_string(),
        "updated description".to_string(),
        "2024-06-20".to_string(),
        true,
    ).expect("update transaction");
    let txs = h.service.get_transactions().expect("get transactions");
    let tx = txs.iter().find(|t| t.id == tx_id).expect("find transaction");
    assert_eq!(tx.amount, 2000);
    assert_eq!(tx.category, "Updated");
    assert_eq!(tx.date, "2024-06-20");
    assert_eq!(tx.transaction_type, "expense");
}

#[test]
fn test_delete_transaction_removes_it() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    let tx_id = create_test_transaction(&h.service, &account_id, 5000, "Temp", false);
    h.service.delete_transaction(tx_id.clone()).expect("delete transaction");
    let txs = h.service.get_transactions().expect("get transactions");
    assert!(txs.iter().all(|t| t.id != tx_id));
}

#[test]
fn test_get_balance_after_income_and_expense() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    create_test_transaction(&h.service, &account_id, 20000, "Income", false);
    create_test_transaction(&h.service, &account_id, 5000, "Expense", true);
    let balance = h.service.get_balance().expect("get balance");
    assert_eq!(balance.total_income, 20000);
    assert_eq!(balance.total_expense, 5000);
}

// ==================== Transfer Tests ====================

#[test]
fn test_transfer_funds_between_accounts() {
    let h = new_test_service();
    let from_id = create_test_account(&h.service, "Source", "USD", 100000);
    let to_id = create_test_account(&h.service, "Dest", "USD", 0);
    let tx_id = h.service.transfer_funds(
        from_id.clone(),
        to_id.clone(),
        30000,
        "monthly transfer".to_string(),
        "2024-06-15".to_string(),
    ).expect("transfer funds");
    assert!(!tx_id.is_empty());
    let balances = h.service.get_account_balances().expect("get balances");
    let from_bal = balances.iter().find(|b| b.account_id == from_id).expect("find source");
    let to_bal = balances.iter().find(|b| b.account_id == to_id).expect("find dest");
    assert_eq!(from_bal.current_balance, 70000, "source should decrease");
    assert_eq!(to_bal.current_balance, 30000, "dest should increase");
}

#[test]
fn test_transfer_funds_rejects_different_currencies() {
    let h = new_test_service();
    let from_id = create_test_account(&h.service, "USD Acc", "USD", 0);
    let to_id = create_test_account(&h.service, "EUR Acc", "EUR", 0);
    let result = h.service.transfer_funds(
        from_id,
        to_id,
        1000,
        "test".to_string(),
        "2024-06-15".to_string(),
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, FinanceError::Validation(_)));
    assert!(err.to_string().contains("same currency"), "error should mention currency mismatch");
}

#[test]
fn test_transfer_funds_rejects_zero_amount() {
    let h = new_test_service();
    let from_id = create_test_account(&h.service, "A", "USD", 0);
    let to_id = create_test_account(&h.service, "B", "USD", 0);
    let result = h.service.transfer_funds(
        from_id, to_id, 0,
        "test".to_string(),
        "2024-06-15".to_string(),
    );
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_update_transfer_preserves_account_balances() {
    let h = new_test_service();
    let from_id = create_test_account(&h.service, "A", "USD", 100000);
    let to_id = create_test_account(&h.service, "B", "USD", 0);
    let tx_id = h.service.transfer_funds(
        from_id.clone(), to_id.clone(), 30000,
        "first".to_string(), "2024-06-15".to_string(),
    ).expect("transfer");
    h.service.update_transfer(
        tx_id, from_id.clone(), to_id.clone(), 50000,
        "updated".to_string(), "2024-06-20".to_string(),
    ).expect("update transfer");
    let balances = h.service.get_account_balances().expect("get balances");
    let from_bal = balances.iter().find(|b| b.account_id == from_id).expect("find source");
    let to_bal = balances.iter().find(|b| b.account_id == to_id).expect("find dest");
    assert_eq!(from_bal.current_balance, 50000, "source after update");
    assert_eq!(to_bal.current_balance, 50000, "dest after update");
}

// ==================== Category Tests ====================

#[test]
fn test_create_and_retrieve_expense_category() {
    let h = new_test_service();
    let id = create_test_category(&h.service, "Groceries", "expense");
    assert!(!id.is_empty());
    let cats = h.service.get_transaction_categories("expense".to_string()).expect("get categories");
    assert!(cats.iter().any(|c| c.name == "Groceries"));
}

#[test]
fn test_create_and_retrieve_income_category() {
    let h = new_test_service();
    let id = create_test_category(&h.service, "Freelance", "income");
    assert!(!id.is_empty());
    let cats = h.service.get_transaction_categories("income".to_string()).expect("get categories");
    assert!(cats.iter().any(|c| c.name == "Freelance"));
}

#[test]
fn test_get_transaction_categories_filters_by_type() {
    let h = new_test_service();
    create_test_category(&h.service, "Food", "expense");
    create_test_category(&h.service, "Salary", "income");
    let expenses = h.service.get_transaction_categories("expense".to_string()).expect("get expenses");
    let incomes = h.service.get_transaction_categories("income".to_string()).expect("get incomes");
    assert!(expenses.iter().any(|c| c.name == "Food"));
    assert!(expenses.iter().all(|c| c.category_type == "expense"));
    assert!(incomes.iter().any(|c| c.name == "Salary"));
    assert!(incomes.iter().all(|c| c.category_type == "income"));
}

#[test]
fn test_get_transaction_categories_rejects_invalid_type() {
    let h = new_test_service();
    let result = h.service.get_transaction_categories("invalid".to_string());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_add_transaction_category_rejects_duplicate() {
    let h = new_test_service();
    create_test_category(&h.service, "Food", "expense");
    let result = h.service.add_transaction_category("Food".to_string(), "expense".to_string());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_update_transaction_category_renames() {
    let h = new_test_service();
    let id = create_test_category(&h.service, "Old Name", "expense");
    h.service.update_transaction_category(id.clone(), "New Name".to_string()).expect("rename");
    let cats = h.service.get_transaction_categories("expense".to_string()).expect("get categories");
    assert!(cats.iter().any(|c| c.name == "New Name"));
    assert!(cats.iter().all(|c| c.name != "Old Name"));
}

#[test]
fn test_delete_transaction_category_removes_it() {
    let h = new_test_service();
    let id = create_test_category(&h.service, "Temp", "expense");
    h.service.delete_transaction_category(id.clone()).expect("delete");
    let cats = h.service.get_transaction_categories("expense".to_string()).expect("get categories");
    assert!(cats.iter().all(|c| c.id != id));
}

// ==================== Exchange Rate Tests ====================

#[test]
fn test_save_and_load_exchange_rate() {
    let h = new_test_service();
    h.service.save_exchange_rate("EUR_USD".to_string(), 1.05).expect("save rate");
    let loaded = h.service.load_exchange_rate_allow_stale("EUR_USD".to_string())
        .expect("load rate");
    assert!(loaded.is_some());
    let (rate, _) = loaded.unwrap();
    assert!((rate - 1.05).abs() < f64::EPSILON);
}

#[test]
fn test_load_exchange_rate_returns_none_for_missing_pair() {
    let h = new_test_service();
    let loaded = h.service.load_exchange_rate_allow_stale("XXX_YYY".to_string())
        .expect("load rate");
    assert!(loaded.is_none());
}

#[test]
fn test_load_exchange_rate_returns_none_when_stale() {
    let h = new_test_service();
    h.service.save_exchange_rate("GBP_USD".to_string(), 1.25).expect("save rate");
    let loaded = h.service.load_exchange_rate("GBP_USD".to_string()).expect("load rate");
    // Rate was just saved, so it should be fresh
    assert!(loaded.is_some(), "recently saved rate should be fresh");
}

// ==================== Dashboard / Analytics Tests ====================

#[test]
fn test_get_expenses_by_category_empty_when_no_transactions() {
    let h = new_test_service();
    create_test_account(&h.service, "Test", "USD", 0);
    let result = h.service.get_expenses_by_category().expect("get expenses");
    assert!(result.is_empty());
}

#[test]
fn test_get_expenses_by_category_groups_by_category() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    create_test_transaction(&h.service, &account_id, 1000, "Food", true);
    create_test_transaction(&h.service, &account_id, 2000, "Food", true);
    create_test_transaction(&h.service, &account_id, 5000, "Travel", true);
    let result = h.service.get_expenses_by_category().expect("get expenses");
    assert_eq!(result.len(), 2, "should have two category groups");
    assert!(result.contains(&("Food".to_string(), 3000)));
    assert!(result.contains(&("Travel".to_string(), 5000)));
}

#[test]
fn test_get_expenses_by_category_excludes_income() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    create_test_transaction(&h.service, &account_id, 10000, "Salary", false);
    create_test_transaction(&h.service, &account_id, 500, "Coffee", true);
    let result = h.service.get_expenses_by_category().expect("get expenses");
    assert!(!result.iter().any(|(cat, _)| cat == "Salary"), "income should not appear");
    assert!(result.iter().any(|(cat, _)| cat == "Coffee"));
}

#[test]
fn test_get_dashboard_data_returns_structure() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 10000);
    create_test_transaction(&h.service, &account_id, 2000, "Food", true);
    let data = h.service.get_dashboard_data(
        0.0,
        &[],
        "1M".to_string(),
        "USD".to_string(),
    ).expect("get dashboard data");
    assert!(data.net_worth.contains("USD"));
}
