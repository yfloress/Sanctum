// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
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
use crate::features::finance::{
    NewAccount, NewTransaction, NewTransfer, UpdateAccount, UpdateTransaction, UpdateTransfer,
};
use secrecy::SecretString;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
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
    let service = FinanceService::new(Arc::new(RwLock::new(Some(db))));
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
    svc.create_account(NewAccount {
        name: name.to_string(),
        account_type: "bank".to_string(),
        currency: currency.to_string(),
        initial_balance_cents: initial_balance,
        color: "#8b5cf6".to_string(),
        icon: None,
    })
    .expect("create account")
}

fn create_test_transaction(
    svc: &FinanceService,
    account_id: &str,
    amount: i64,
    category: &str,
    is_expense: bool,
) -> String {
    svc.add_transaction(NewTransaction {
        account_id: account_id.to_string(),
        amount_cents: amount,
        category: category.to_string(),
        description: "test transaction".to_string(),
        date: "2024-06-15".to_string(),
        is_expense,
    })
    .expect("create transaction")
}

fn create_test_category(svc: &FinanceService, name: &str, category_type: &str) -> String {
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
    let balance = balances
        .iter()
        .find(|b| b.account_id == id)
        .expect("find balance");
    // current_balance = initial_balance + income - expense
    // With no transactions: 10000 + 0 - 0 = 10000 ✓
    assert_eq!(balance.current_balance, 10000);
    // total_income only counts income TRANSACTIONS, not initial_balance
    assert_eq!(balance.total_income, 0);
    assert_eq!(balance.total_expense, 0);
}

#[test]
fn test_update_account_changes_fields() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Old Name", "USD", 0);
    h.service
        .update_account(UpdateAccount {
            id: id.clone(),
            name: "New Name".to_string(),
            account_type: "savings".to_string(),
            currency: "EUR".to_string(),
            initial_balance_cents: 20000,
            color: "#ec4899".to_string(),
            icon: Some("icon.svg".to_string()),
        })
        .expect("update account");
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
    h.service
        .update_account_icon(id.clone(), Some("new-icon.svg".to_string()))
        .expect("update icon");
    let accounts = h.service.get_accounts().expect("get accounts");
    let account = accounts.iter().find(|a| a.id == id).expect("find account");
    assert_eq!(account.icon.as_deref(), Some("new-icon.svg"));
    assert_eq!(account.name, "Test");
}

#[test]
fn test_update_account_icon_with_none_clears_icon() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Test", "USD", 0);
    h.service
        .update_account_icon(id.clone(), Some("old.svg".to_string()))
        .expect("set icon");
    h.service
        .update_account_icon(id.clone(), None)
        .expect("clear icon");
    let accounts = h.service.get_accounts().expect("get accounts");
    let account = accounts.iter().find(|a| a.id == id).expect("find account");
    assert_eq!(account.icon, None);
}

#[test]
fn test_update_account_name_changes_only_name() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Original", "USD", 0);
    h.service
        .update_account_name(id.clone(), "Renamed".to_string())
        .expect("update name");
    let accounts = h.service.get_accounts().expect("get accounts");
    let account = accounts.iter().find(|a| a.id == id).expect("find account");
    assert_eq!(account.name, "Renamed");
    assert_eq!(account.currency, "USD");
}

#[test]
fn test_archive_and_unarchive_account() {
    let h = new_test_service();
    let id = create_test_account(&h.service, "Archivable", "USD", 0);
    h.service
        .archive_account(id.clone())
        .expect("archive account");
    let active = h.service.get_accounts().expect("get accounts");
    assert!(active.iter().all(|a| a.id != id));
    let archived = h.service.get_archived_accounts().expect("get archived");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].id, id);
    h.service
        .unarchive_account(id.clone())
        .expect("unarchive account");
    let active_again = h
        .service
        .get_accounts()
        .expect("get accounts after unarchive");
    assert!(active_again.iter().any(|a| a.id == id));
}

// ==================== Validation Error Tests ====================

#[test]
fn test_create_account_rejects_empty_name() {
    let h = new_test_service();
    let result = h.service.create_account(NewAccount {
        name: "".to_string(),
        account_type: "bank".to_string(),
        currency: "USD".to_string(),
        initial_balance_cents: 0,
        color: "#8b5cf6".to_string(),
        icon: None,
    });
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, FinanceError::Validation(_)));
    assert!(err.to_string().contains("empty") || err.to_string().contains("length"));
}

#[test]
fn test_create_account_rejects_empty_currency() {
    let h = new_test_service();
    let result = h.service.create_account(NewAccount {
        name: "Test".to_string(),
        account_type: "bank".to_string(),
        currency: "".to_string(),
        initial_balance_cents: 0,
        color: "#8b5cf6".to_string(),
        icon: None,
    });
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, FinanceError::Validation(_)));
}

#[test]
fn test_create_account_rejects_invalid_color() {
    let h = new_test_service();
    let result = h.service.create_account(NewAccount {
        name: "Test".to_string(),
        account_type: "bank".to_string(),
        currency: "USD".to_string(),
        initial_balance_cents: 0,
        color: "not-a-color".to_string(),
        icon: None,
    });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_create_account_rejects_name_too_long() {
    let h = new_test_service();
    let long_name = "a".repeat(65);
    let result = h.service.create_account(NewAccount {
        name: long_name,
        account_type: "bank".to_string(),
        currency: "USD".to_string(),
        initial_balance_cents: 0,
        color: "#8b5cf6".to_string(),
        icon: None,
    });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_update_account_rejects_invalid_id() {
    let h = new_test_service();
    let result = h.service.update_account(UpdateAccount {
        id: "not-a-uuid".to_string(),
        name: "Name".to_string(),
        account_type: "bank".to_string(),
        currency: "USD".to_string(),
        initial_balance_cents: 0,
        color: "#8b5cf6".to_string(),
        icon: None,
    });
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
    let result = h.service.add_transaction(NewTransaction {
        account_id,
        amount_cents: 0,
        category: "Test".to_string(),
        description: "desc".to_string(),
        date: "2024-06-15".to_string(),
        is_expense: false,
    });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_add_transaction_rejects_negative_amount() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    let result = h.service.add_transaction(NewTransaction {
        account_id,
        amount_cents: -100,
        category: "Test".to_string(),
        description: "desc".to_string(),
        date: "2024-06-15".to_string(),
        is_expense: false,
    });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_add_transaction_rejects_invalid_account() {
    let h = new_test_service();
    let result = h.service.add_transaction(NewTransaction {
        account_id: "bad-uuid".to_string(),
        amount_cents: 1000,
        category: "Test".to_string(),
        description: "desc".to_string(),
        date: "2024-06-15".to_string(),
        is_expense: false,
    });
    assert!(result.is_err());
}

#[test]
fn test_update_transaction_changes_fields() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 0);
    let tx_id = create_test_transaction(&h.service, &account_id, 1000, "Old", false);
    h.service
        .update_transaction(UpdateTransaction {
            id: tx_id.clone(),
            account_id: account_id.clone(),
            amount_cents: 2000,
            category: "Updated".to_string(),
            description: "updated description".to_string(),
            date: "2024-06-20".to_string(),
            is_expense: true,
        })
        .expect("update transaction");
    let txs = h.service.get_transactions().expect("get transactions");
    let tx = txs
        .iter()
        .find(|t| t.id == tx_id)
        .expect("find transaction");
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
    h.service
        .delete_transaction(tx_id.clone())
        .expect("delete transaction");
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
    let tx_id = h
        .service
        .transfer_funds(NewTransfer {
            from_account_id: from_id.clone(),
            to_account_id: to_id.clone(),
            amount_cents: 30000,
            description: "monthly transfer".to_string(),
            date: "2024-06-15".to_string(),
        })
        .expect("transfer funds");
    assert!(!tx_id.is_empty());
    let balances = h.service.get_account_balances().expect("get balances");
    let from_bal = balances
        .iter()
        .find(|b| b.account_id == from_id)
        .expect("find source");
    let to_bal = balances
        .iter()
        .find(|b| b.account_id == to_id)
        .expect("find dest");
    assert_eq!(from_bal.current_balance, 70000, "source should decrease");
    assert_eq!(to_bal.current_balance, 30000, "dest should increase");
}

#[test]
fn test_transfer_funds_rejects_different_currencies() {
    let h = new_test_service();
    let from_id = create_test_account(&h.service, "USD Acc", "USD", 0);
    let to_id = create_test_account(&h.service, "EUR Acc", "EUR", 0);
    let result = h.service.transfer_funds(NewTransfer {
        from_account_id: from_id,
        to_account_id: to_id,
        amount_cents: 1000,
        description: "test".to_string(),
        date: "2024-06-15".to_string(),
    });
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, FinanceError::Validation(_)));
    assert!(
        err.to_string().contains("same currency"),
        "error should mention currency mismatch"
    );
}

#[test]
fn test_transfer_funds_rejects_zero_amount() {
    let h = new_test_service();
    let from_id = create_test_account(&h.service, "A", "USD", 0);
    let to_id = create_test_account(&h.service, "B", "USD", 0);
    let result = h.service.transfer_funds(NewTransfer {
        from_account_id: from_id,
        to_account_id: to_id,
        amount_cents: 0,
        description: "test".to_string(),
        date: "2024-06-15".to_string(),
    });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FinanceError::Validation(_)));
}

#[test]
fn test_update_transfer_preserves_account_balances() {
    let h = new_test_service();
    let from_id = create_test_account(&h.service, "A", "USD", 100000);
    let to_id = create_test_account(&h.service, "B", "USD", 0);
    let tx_id = h
        .service
        .transfer_funds(NewTransfer {
            from_account_id: from_id.clone(),
            to_account_id: to_id.clone(),
            amount_cents: 30000,
            description: "first".to_string(),
            date: "2024-06-15".to_string(),
        })
        .expect("transfer");
    h.service
        .update_transfer(UpdateTransfer {
            id: tx_id,
            from_account_id: from_id.clone(),
            to_account_id: to_id.clone(),
            amount_cents: 50000,
            description: "updated".to_string(),
            date: "2024-06-20".to_string(),
        })
        .expect("update transfer");
    let balances = h.service.get_account_balances().expect("get balances");
    let from_bal = balances
        .iter()
        .find(|b| b.account_id == from_id)
        .expect("find source");
    let to_bal = balances
        .iter()
        .find(|b| b.account_id == to_id)
        .expect("find dest");
    assert_eq!(from_bal.current_balance, 50000, "source after update");
    assert_eq!(to_bal.current_balance, 50000, "dest after update");
}

// ==================== Category Tests ====================

#[test]
fn test_create_and_retrieve_expense_category() {
    let h = new_test_service();
    let id = create_test_category(&h.service, "Groceries", "expense");
    assert!(!id.is_empty());
    let cats = h
        .service
        .get_transaction_categories("expense".to_string())
        .expect("get categories");
    assert!(cats.iter().any(|c| c.name == "Groceries"));
}

#[test]
fn test_create_and_retrieve_income_category() {
    let h = new_test_service();
    let id = create_test_category(&h.service, "Freelance", "income");
    assert!(!id.is_empty());
    let cats = h
        .service
        .get_transaction_categories("income".to_string())
        .expect("get categories");
    assert!(cats.iter().any(|c| c.name == "Freelance"));
}

#[test]
fn test_get_transaction_categories_filters_by_type() {
    let h = new_test_service();
    create_test_category(&h.service, "Food", "expense");
    create_test_category(&h.service, "Salary", "income");
    let expenses = h
        .service
        .get_transaction_categories("expense".to_string())
        .expect("get expenses");
    let incomes = h
        .service
        .get_transaction_categories("income".to_string())
        .expect("get incomes");
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
    let result = h
        .service
        .add_transaction_category("Food".to_string(), "expense".to_string());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_update_transaction_category_renames() {
    let h = new_test_service();
    let id = create_test_category(&h.service, "Old Name", "expense");
    h.service
        .update_transaction_category(id.clone(), "New Name".to_string())
        .expect("rename");
    let cats = h
        .service
        .get_transaction_categories("expense".to_string())
        .expect("get categories");
    assert!(cats.iter().any(|c| c.name == "New Name"));
    assert!(cats.iter().all(|c| c.name != "Old Name"));
}

#[test]
fn test_delete_transaction_category_removes_it() {
    let h = new_test_service();
    let id = create_test_category(&h.service, "Temp", "expense");
    h.service
        .delete_transaction_category(id.clone())
        .expect("delete");
    let cats = h
        .service
        .get_transaction_categories("expense".to_string())
        .expect("get categories");
    assert!(cats.iter().all(|c| c.id != id));
}

// ==================== Exchange Rate Tests ====================

#[test]
fn test_save_and_load_exchange_rate() {
    let h = new_test_service();
    h.service
        .save_exchange_rate("EUR_USD".to_string(), 1.05)
        .expect("save rate");
    let loaded = h
        .service
        .load_exchange_rate_allow_stale("EUR_USD".to_string())
        .expect("load rate");
    assert!(loaded.is_some());
    let (rate, _) = loaded.unwrap();
    assert!((rate - 1.05).abs() < f64::EPSILON);
}

#[test]
fn test_load_exchange_rate_returns_none_for_missing_pair() {
    let h = new_test_service();
    let loaded = h
        .service
        .load_exchange_rate_allow_stale("XXX_YYY".to_string())
        .expect("load rate");
    assert!(loaded.is_none());
}

#[test]
fn test_load_exchange_rate_returns_none_when_stale() {
    let h = new_test_service();
    h.service
        .save_exchange_rate("GBP_USD".to_string(), 1.25)
        .expect("save rate");
    let loaded = h
        .service
        .load_exchange_rate("GBP_USD".to_string())
        .expect("load rate");
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
    assert!(
        !result.iter().any(|(cat, _)| cat == "Salary"),
        "income should not appear"
    );
    assert!(result.iter().any(|(cat, _)| cat == "Coffee"));
}

#[test]
fn test_get_dashboard_data_returns_structure() {
    let h = new_test_service();
    let account_id = create_test_account(&h.service, "Test", "USD", 10000);
    create_test_transaction(&h.service, &account_id, 2000, "Food", true);
    let data = h
        .service
        .get_dashboard_data(0.0, &[], "1M".to_string(), "USD".to_string())
        .expect("get dashboard data");
    assert!(data.net_worth.contains("USD"));
}

// ==================== Category integrity ====================

/// Finds a seeded category by its stored code.
fn category_id(service: &FinanceService, kind: &str, name: &str) -> String {
    service
        .get_transaction_categories(kind.to_string())
        .expect("categories")
        .into_iter()
        .find(|c| c.name == name)
        .map(|c| c.id)
        .unwrap_or_else(|| panic!("seeded category {name} not found"))
}

fn account_with_expense(harness: &TestServiceHarness, category: &str) -> String {
    let account_id = harness
        .service
        .create_account(NewAccount {
            name: "Checking".to_string(),
            account_type: "bank".to_string(),
            currency: "USD".to_string(),
            initial_balance_cents: 100_000,
            color: "#8b5cf6".to_string(),
            icon: None,
        })
        .expect("create account");

    harness
        .service
        .add_transaction(NewTransaction {
            account_id: account_id.clone(),
            amount_cents: 1_500,
            category: category.to_string(),
            description: "Lunch".to_string(),
            date: "2026-07-29".to_string(),
            is_expense: true,
        })
        .expect("add transaction");

    account_id
}

#[test]
fn renaming_a_category_moves_its_transactions_too() {
    let harness = new_test_service();
    account_with_expense(&harness, "FOOD");
    let id = category_id(&harness.service, "expense", "FOOD");

    harness
        .service
        .update_transaction_category(id, "Comida".to_string())
        .expect("rename category");

    // The transaction follows the rename instead of becoming a separate,
    // unreachable category in filters and charts.
    let transactions = harness.service.get_transactions().expect("transactions");
    assert!(
        transactions.iter().all(|t| t.category == "Comida"),
        "transactions kept the old category name: {:?}",
        transactions.iter().map(|t| &t.category).collect::<Vec<_>>()
    );
}

#[test]
fn deleting_a_category_in_use_is_refused() {
    let harness = new_test_service();
    account_with_expense(&harness, "FOOD");
    let id = category_id(&harness.service, "expense", "FOOD");

    let result = harness.service.delete_transaction_category(id.clone());

    assert!(result.is_err(), "a category in use must not be deleted");
    assert!(
        harness
            .service
            .get_transaction_categories("expense".to_string())
            .expect("categories")
            .iter()
            .any(|c| c.id == id),
        "the category must still be there"
    );
}

#[test]
fn deleting_an_unused_category_still_works() {
    let harness = new_test_service();
    let id = category_id(&harness.service, "expense", "SHOPPING");

    harness
        .service
        .delete_transaction_category(id.clone())
        .expect("delete unused category");

    assert!(
        !harness
            .service
            .get_transaction_categories("expense".to_string())
            .expect("categories")
            .iter()
            .any(|c| c.id == id)
    );
}

// ==================== Tags ====================

#[test]
fn tags_are_lowercased_and_deduplicated() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);

    h.service
        .set_transaction_tags(
            tx.clone(),
            vec!["Snack".into(), "SNACK".into(), "work".into()],
        )
        .expect("set tags");

    let tags = h.service.get_all_transaction_tags().expect("read tags");
    assert_eq!(tags.get(&tx).map(Vec::len), Some(2));
    assert!(tags[&tx].contains(&"snack".to_string()));
    assert!(tags[&tx].contains(&"work".to_string()));
}

#[test]
fn tags_keep_accents_and_enye() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);

    h.service
        .set_transaction_tags(tx.clone(), vec!["Niños".into(), "Educación".into()])
        .expect("set tags");

    let tags = h.service.get_all_transaction_tags().expect("read tags");
    assert!(tags[&tx].contains(&"niños".to_string()));
    assert!(tags[&tx].contains(&"educación".to_string()));
}

#[test]
fn setting_tags_replaces_the_previous_set() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);

    h.service
        .set_transaction_tags(tx.clone(), vec!["one".into()])
        .expect("first set");
    h.service
        .set_transaction_tags(tx.clone(), vec!["two".into()])
        .expect("second set");

    let tags = h.service.get_all_transaction_tags().expect("read tags");
    assert_eq!(tags[&tx], vec!["two".to_string()]);
}

#[test]
fn deleting_a_transaction_takes_its_tags_with_it() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);
    h.service
        .set_transaction_tags(tx.clone(), vec!["orphan".into()])
        .expect("set tags");

    h.service
        .delete_transaction(tx)
        .expect("delete transaction");

    assert!(
        h.service.get_tag_catalog().expect("catalog").is_empty(),
        "the cascade should have removed the tag rows"
    );
}

#[test]
fn the_tag_catalog_puts_the_most_used_first() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    for _ in 0..3 {
        let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);
        h.service
            .set_transaction_tags(tx, vec!["common".into()])
            .expect("set tags");
    }
    let rare = create_test_transaction(&h.service, &account, 1000, "FOOD", true);
    h.service
        .set_transaction_tags(rare, vec!["rare".into()])
        .expect("set tags");

    let catalog = h.service.get_tag_catalog().expect("catalog");
    assert_eq!(catalog, vec!["common".to_string(), "rare".to_string()]);
}

#[test]
fn a_tag_beyond_the_length_limit_is_rejected() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);

    let result = h.service.set_transaction_tags(tx, vec!["x".repeat(33)]);
    assert!(result.is_err());
}

#[test]
fn tagging_in_bulk_skips_rows_that_already_carry_it() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    let first = create_test_transaction(&h.service, &account, 1000, "FOOD", true);
    let second = create_test_transaction(&h.service, &account, 2000, "FOOD", true);
    h.service
        .set_transaction_tags(first.clone(), vec!["shared".into()])
        .expect("set tags");

    let added = h
        .service
        .tag_transactions(vec![first, second], "shared".to_string())
        .expect("bulk tag");
    assert_eq!(added, 1, "only the untagged row should count");
}

// ==================== Reconciliation ====================

#[test]
fn a_fresh_account_reconciles_to_its_opening_balance() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 50_000);
    create_test_transaction(&h.service, &account, 1000, "FOOD", true);

    let confirmed = h
        .service
        .reconciled_balance(account.clone())
        .expect("reconciled balance");
    assert_eq!(
        confirmed, 50_000,
        "the opening balance counts, the unconfirmed expense does not"
    );
}

#[test]
fn confirming_a_row_moves_the_reconciled_balance() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 50_000);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);

    h.service
        .confirm_reconciliation(account.clone(), vec![tx])
        .expect("confirm");

    let confirmed = h
        .service
        .reconciled_balance(account)
        .expect("reconciled balance");
    assert_eq!(confirmed, 49_000);
}

#[test]
fn confirming_a_transfer_on_one_side_leaves_the_other_pending() {
    let h = new_test_service();
    let from = create_test_account(&h.service, "Checking", "USD", 100_000);
    let to = create_test_account(&h.service, "Savings", "USD", 0);
    let tx = h
        .service
        .transfer_funds(NewTransfer {
            from_account_id: from.clone(),
            to_account_id: to.clone(),
            amount_cents: 30_000,
            description: "move".to_string(),
            date: "2024-06-15".to_string(),
        })
        .expect("transfer");

    h.service
        .confirm_reconciliation(from.clone(), vec![tx])
        .expect("confirm the outgoing side");

    assert_eq!(
        h.service.reconciled_balance(from).expect("from balance"),
        70_000,
        "the source has seen the money leave"
    );
    assert_eq!(
        h.service
            .reconciled_balance(to.clone())
            .expect("to balance"),
        0,
        "the destination has not confirmed it yet"
    );
    assert_eq!(
        h.service
            .unreconciled_transactions(to)
            .expect("pending")
            .len(),
        1,
        "the same row is still waiting on the destination"
    );
}

#[test]
fn editing_a_transaction_withdraws_its_confirmation() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 50_000);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);
    h.service
        .confirm_reconciliation(account.clone(), vec![tx.clone()])
        .expect("confirm");

    h.service
        .update_transaction(UpdateTransaction {
            id: tx,
            account_id: account.clone(),
            amount_cents: 2000,
            category: "FOOD".to_string(),
            description: "changed".to_string(),
            date: "2024-06-15".to_string(),
            is_expense: true,
        })
        .expect("update");

    assert_eq!(
        h.service
            .reconciled_balance(account.clone())
            .expect("balance"),
        50_000,
        "a changed row is no longer proven by the old statement"
    );
    assert_eq!(
        h.service
            .unreconciled_transactions(account)
            .expect("pending")
            .len(),
        1
    );
}

#[test]
fn a_confirmed_row_drops_out_of_the_pending_list() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);
    assert_eq!(
        h.service
            .unreconciled_transactions(account.clone())
            .expect("pending")
            .len(),
        1
    );

    h.service
        .confirm_reconciliation(account.clone(), vec![tx])
        .expect("confirm");

    assert!(
        h.service
            .unreconciled_transactions(account)
            .expect("pending")
            .is_empty()
    );
}

#[test]
fn renaming_a_transaction_keeps_its_confirmation() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 50_000);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);
    h.service
        .confirm_reconciliation(account.clone(), vec![tx.clone()])
        .expect("confirm");

    // Same money, same day, same account: the statement still matches.
    h.service
        .update_transaction(UpdateTransaction {
            id: tx,
            account_id: account.clone(),
            amount_cents: 1000,
            category: "SHOPPING".to_string(),
            description: "renamed".to_string(),
            date: "2024-06-15".to_string(),
            is_expense: true,
        })
        .expect("update");

    assert_eq!(
        h.service.reconciled_balance(account).expect("balance"),
        49_000,
        "a relabelled row is still the one the bank showed"
    );
}

#[test]
fn editing_a_transfer_amount_withdraws_both_confirmations() {
    let h = new_test_service();
    let from = create_test_account(&h.service, "Checking", "USD", 100_000);
    let to = create_test_account(&h.service, "Savings", "USD", 0);
    let tx = h
        .service
        .transfer_funds(NewTransfer {
            from_account_id: from.clone(),
            to_account_id: to.clone(),
            amount_cents: 30_000,
            description: "move".to_string(),
            date: "2024-06-15".to_string(),
        })
        .expect("transfer");
    h.service
        .confirm_reconciliation(from.clone(), vec![tx.clone()])
        .expect("confirm source");
    h.service
        .confirm_reconciliation(to.clone(), vec![tx.clone()])
        .expect("confirm destination");

    h.service
        .update_transfer(UpdateTransfer {
            id: tx,
            from_account_id: from.clone(),
            to_account_id: to.clone(),
            amount_cents: 45_000,
            description: "move".to_string(),
            date: "2024-06-15".to_string(),
        })
        .expect("update transfer");

    assert_eq!(
        h.service.reconciled_balance(from).expect("from balance"),
        100_000,
        "the source confirmation no longer holds"
    );
    assert_eq!(
        h.service.reconciled_balance(to).expect("to balance"),
        0,
        "nor does the destination one"
    );
}

#[test]
fn moving_a_transaction_to_another_day_withdraws_its_confirmation() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 50_000);
    let tx = create_test_transaction(&h.service, &account, 1000, "FOOD", true);
    h.service
        .confirm_reconciliation(account.clone(), vec![tx.clone()])
        .expect("confirm");

    h.service
        .update_transaction(UpdateTransaction {
            id: tx,
            account_id: account.clone(),
            amount_cents: 1000,
            category: "FOOD".to_string(),
            description: "test transaction".to_string(),
            date: "2024-07-01".to_string(),
            is_expense: true,
        })
        .expect("update");

    assert_eq!(
        h.service.reconciled_balance(account).expect("balance"),
        50_000
    );
}
