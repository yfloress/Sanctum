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
    CreditStatus, NewAccount, NewCharge, NewCredit, NewTransaction, NewTransfer, UpdateAccount,
    UpdateInstallment, UpdateTransaction, UpdateTransfer, amortization, credit_interest,
    credit_progress, credit_totals, french_installment,
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

// ==================== Credits ====================

fn create_test_credit(
    svc: &FinanceService,
    account_id: &str,
    name: &str,
    installment_amount: i64,
    count: i32,
    first_due: &str,
) -> String {
    svc.create_credit(NewCredit {
        account_id: account_id.to_string(),
        name: name.to_string(),
        category: "SHOPPING".to_string(),
        kind: "installments".to_string(),
        down_payment_cents: 0,
        down_payment_date: None,
        installment_amount_cents: installment_amount,
        installment_count: count,
        first_due_date: first_due.to_string(),
        cash_price_cents: None,
        principal_cents: None,
        monthly_rate_ppm: None,
    })
    .expect("create credit")
}

fn installments_of(svc: &FinanceService, credit_id: &str) -> Vec<crate::models::CreditInstallment> {
    svc.get_credit_installments()
        .expect("installments")
        .remove(credit_id)
        .unwrap_or_default()
}

#[test]
fn creating_a_credit_writes_its_whole_schedule() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 500_000);
    let credit = create_test_credit(&h.service, &account, "Fridge", 25_000, 12, "2026-03-15");

    let schedule = installments_of(&h.service, &credit);
    assert_eq!(schedule.len(), 12);
    assert_eq!(schedule[0].due_date, "2026-03-15");
    assert_eq!(schedule[11].due_date, "2027-02-15");
    assert!(
        schedule.iter().all(|i| !i.is_paid()),
        "a new plan owes everything"
    );
}

#[test]
fn a_schedule_anchored_on_the_31st_returns_to_the_31st_after_february() {
    let first = chrono::NaiveDate::from_ymd_opt(2026, 1, 31).expect("date");
    let dates: Vec<String> = crate::features::finance::credits::schedule_dates(first, 4)
        .into_iter()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .collect();

    // Offsets are taken from the first date, so February borrowing a shorter
    // month does not shift every date after it.
    assert_eq!(
        dates,
        vec!["2026-01-31", "2026-02-28", "2026-03-31", "2026-04-30"]
    );
}

#[test]
fn a_credit_needs_at_least_one_installment() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);

    let err = h
        .service
        .create_credit(NewCredit {
            account_id: account,
            name: "Nothing".to_string(),
            category: "SHOPPING".to_string(),
            kind: "installments".to_string(),
            down_payment_cents: 0,
            down_payment_date: None,
            installment_amount_cents: 1000,
            installment_count: 0,
            first_due_date: "2026-03-15".to_string(),
            cash_price_cents: None,
            principal_cents: None,
            monthly_rate_ppm: None,
        })
        .expect_err("a plan with no installments is not a plan");
    assert!(matches!(err, FinanceError::Validation(_)));
}

#[test]
fn a_credit_name_keeps_its_accents() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);
    let credit = create_test_credit(&h.service, &account, "Colchón niño", 1000, 3, "2026-03-15");

    let stored = h
        .service
        .get_credits()
        .expect("credits")
        .into_iter()
        .find(|c| c.id == credit)
        .expect("the credit");
    assert_eq!(stored.name, "Colchón niño");
}

#[test]
fn paying_an_installment_writes_the_expense_it_stands_for() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 500_000);
    let credit = create_test_credit(&h.service, &account, "Fridge", 25_000, 12, "2026-03-15");
    let first = installments_of(&h.service, &credit).remove(0);

    h.service
        .pay_installment(first.id.clone(), Some("2026-03-14".to_string()))
        .expect("pay");

    let payment = h
        .service
        .get_transactions()
        .expect("transactions")
        .into_iter()
        .find(|tx| tx.description == "Fridge 1/12")
        .expect("the payment landed in the ledger");
    assert_eq!(payment.account_id, account);
    assert_eq!(payment.amount, 25_000);
    assert_eq!(payment.transaction_type, "expense");
    assert_eq!(payment.date, "2026-03-14");

    let balance = h
        .service
        .get_account_balances()
        .expect("balances")
        .into_iter()
        .find(|b| b.account_id == account)
        .expect("the account")
        .current_balance;
    assert_eq!(balance, 475_000, "the money left the account");
}

#[test]
fn an_installment_cannot_be_paid_twice() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 500_000);
    let credit = create_test_credit(&h.service, &account, "Fridge", 25_000, 12, "2026-03-15");
    let first = installments_of(&h.service, &credit).remove(0);

    h.service
        .pay_installment(first.id.clone(), None)
        .expect("pay");
    let err = h
        .service
        .pay_installment(first.id, None)
        .expect_err("a second click must not write a second expense");
    assert!(matches!(err, FinanceError::Validation(_)));

    assert_eq!(
        h.service.get_transactions().expect("transactions").len(),
        1,
        "and the ledger still holds exactly one payment"
    );
}

#[test]
fn undoing_a_payment_takes_its_expense_with_it() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 500_000);
    let credit = create_test_credit(&h.service, &account, "Fridge", 25_000, 12, "2026-03-15");
    let first = installments_of(&h.service, &credit).remove(0);

    h.service
        .pay_installment(first.id.clone(), None)
        .expect("pay");
    h.service.unpay_installment(first.id).expect("undo");

    assert!(
        h.service
            .get_transactions()
            .expect("transactions")
            .is_empty(),
        "an undone payment is not a payment"
    );
    assert!(
        installments_of(&h.service, &credit)
            .iter()
            .all(|i| !i.is_paid())
    );
}

#[test]
fn deleting_the_payment_from_the_ledger_puts_its_installment_back_to_pending() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 500_000);
    let credit = create_test_credit(&h.service, &account, "Fridge", 25_000, 12, "2026-03-15");
    let first = installments_of(&h.service, &credit).remove(0);

    let payment = h
        .service
        .pay_installment(first.id.clone(), None)
        .expect("pay");

    // The ledger is the place a user deletes things from, and the credit has to
    // follow: otherwise it claims money that no longer moved.
    h.service.delete_transaction(payment).expect("delete");

    let schedule = installments_of(&h.service, &credit);
    assert!(!schedule[0].is_paid());
    assert!(schedule[0].paid_date.is_none());
}

#[test]
fn deleting_a_credit_keeps_the_payments_already_made() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 500_000);
    let credit = create_test_credit(&h.service, &account, "Fridge", 25_000, 12, "2026-03-15");
    let first = installments_of(&h.service, &credit).remove(0);
    h.service.pay_installment(first.id, None).expect("pay");

    h.service.delete_credit(credit.clone()).expect("delete");

    assert!(h.service.get_credits().expect("credits").is_empty());
    assert!(
        installments_of(&h.service, &credit).is_empty(),
        "the schedule goes with the credit"
    );
    assert_eq!(
        h.service.get_transactions().expect("transactions").len(),
        1,
        "but the money did leave the account, so the expense stays"
    );
}

#[test]
fn an_unpaid_installment_past_its_date_is_overdue() {
    let schedule = vec![
        test_installment(1, "2026-01-15", false),
        test_installment(2, "2026-02-15", false),
    ];
    let progress = credit_progress(&schedule, "2026-02-20");

    assert_eq!(progress.overdue_count, 2);
    assert_eq!(progress.status, CreditStatus::Overdue);
    assert_eq!(progress.next_due_date.as_deref(), Some("2026-01-15"));
}

#[test]
fn paying_further_than_the_calendar_asks_reads_as_ahead() {
    let schedule = vec![
        test_installment(1, "2026-01-15", true),
        test_installment(2, "2026-02-15", true),
        test_installment(3, "2026-03-15", false),
    ];

    assert_eq!(
        credit_progress(&schedule, "2026-01-20").status,
        CreditStatus::Ahead
    );
    // On the day the second one falls due it is simply paid on time.
    assert_eq!(
        credit_progress(&schedule, "2026-02-15").status,
        CreditStatus::OnTrack
    );
}

#[test]
fn a_fully_paid_credit_is_done() {
    let schedule = vec![
        test_installment(1, "2026-01-15", true),
        test_installment(2, "2026-02-15", true),
    ];
    let progress = credit_progress(&schedule, "2026-06-01");

    assert_eq!(progress.status, CreditStatus::Done);
    assert_eq!(progress.paid_count, 2);
    assert!(progress.next_due_date.is_none());
}

#[test]
fn interest_is_what_the_plan_costs_over_the_cash_price() {
    let mut credit = test_credit(25_000, 12);
    let schedule: Vec<crate::models::CreditInstallment> = (1..=12)
        .map(|n| test_installment(n, "2026-01-15", false))
        .collect();

    credit.cash_price = Some(250_000);
    assert_eq!(credit_interest(&credit, &schedule), Some(50_000));

    credit.cash_price = None;
    assert_eq!(
        credit_interest(&credit, &schedule),
        None,
        "not knowing the cash price is not the same as paying no interest"
    );
}

fn test_installment(number: i32, due_date: &str, paid: bool) -> crate::models::CreditInstallment {
    test_row("installment", number, 25_000, due_date, paid)
}

fn test_row(
    kind: &str,
    number: i32,
    amount: i64,
    due_date: &str,
    paid: bool,
) -> crate::models::CreditInstallment {
    crate::models::CreditInstallment {
        id: format!("{kind}-{number}"),
        credit_id: "credit".to_string(),
        kind: kind.to_string(),
        number,
        amount,
        due_date: due_date.to_string(),
        note: None,
        transaction_id: paid.then(|| format!("tx-{kind}-{number}")),
        paid_date: paid.then(|| due_date.to_string()),
    }
}

fn test_credit(installment_amount: i64, installment_count: i32) -> crate::models::Credit {
    crate::models::Credit {
        id: "credit".to_string(),
        account_id: "account".to_string(),
        name: "Fridge".to_string(),
        category: "SHOPPING".to_string(),
        kind: "installments".to_string(),
        down_payment: 0,
        installment_amount,
        installment_count,
        first_due_date: "2026-01-15".to_string(),
        cash_price: None,
        principal: None,
        monthly_rate_ppm: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

// ==================== Credits: down payment, loans, corrections ====================

#[test]
fn a_down_payment_becomes_the_first_row_of_the_schedule() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 1_000_000);

    let credit = h
        .service
        .create_credit(NewCredit {
            account_id: account.clone(),
            name: "Car".to_string(),
            category: "TRANSPORT".to_string(),
            kind: "installments".to_string(),
            down_payment_cents: 200_000,
            down_payment_date: Some("2026-02-01".to_string()),
            installment_amount_cents: 50_000,
            installment_count: 6,
            first_due_date: "2026-03-01".to_string(),
            cash_price_cents: None,
            principal_cents: None,
            monthly_rate_ppm: None,
        })
        .expect("create credit");

    let schedule = installments_of(&h.service, &credit);
    assert_eq!(schedule.len(), 7, "the down payment is a row of its own");
    assert_eq!(schedule[0].kind, "down_payment");
    assert_eq!(schedule[0].amount, 200_000);
    assert_eq!(schedule[0].due_date, "2026-02-01");

    // Paying it writes a real expense, like any other row.
    h.service
        .pay_installment(schedule[0].id.clone(), Some("2026-02-01".to_string()))
        .expect("pay the down payment");
    let payment = h
        .service
        .get_transactions()
        .expect("transactions")
        .into_iter()
        .find(|tx| tx.amount == 200_000)
        .expect("the down payment landed in the ledger");
    assert_eq!(payment.description, "Car 0/6");
}

#[test]
fn a_down_payment_counts_towards_the_total_but_not_towards_the_installments() {
    let schedule = vec![
        test_row("down_payment", 1, 200_000, "2026-02-01", true),
        test_row("installment", 1, 50_000, "2026-03-01", true),
        test_row("installment", 2, 50_000, "2026-04-01", false),
    ];
    let totals = credit_totals(&schedule);
    assert_eq!(totals.plan, 300_000);
    assert_eq!(totals.paid, 250_000);

    // The bar follows the money, so a large down payment shows as the large
    // share of the debt it actually is.
    assert!((totals.percentage() - 83.333).abs() < 0.01);

    let progress = credit_progress(&schedule, "2026-03-02");
    assert_eq!(
        progress.paid_count, 1,
        "one installment of the plan, not two"
    );
}

#[test]
fn a_loan_payment_follows_the_constant_payment_formula() {
    // 1.000.000 over 12 months at 1,5% a month is the textbook case.
    assert_eq!(french_installment(100_000_000, 15_000, 12), 9_167_999);

    // Without interest the principal is simply split evenly.
    assert_eq!(french_installment(120_000, 0, 12), 10_000);
    // And a division that does not come out even rounds up, so the schedule
    // covers the debt rather than falling a cent short.
    assert_eq!(french_installment(100_001, 0, 10), 10_001);
}

#[test]
fn an_amortization_table_pays_interest_first_and_principal_later() {
    let schedule: Vec<crate::models::CreditInstallment> = (1..=12)
        .map(|n| test_row("installment", n, 9_167_999, "2026-03-01", false))
        .collect();
    let table = amortization(100_000_000, 15_000, &schedule);

    assert_eq!(table.len(), 12);
    assert_eq!(table[0].interest, 1_500_000);
    assert!(
        table[0].principal < table[11].principal,
        "the first payment buys less debt than the last"
    );
    assert!(
        table[11].balance.abs() < 100,
        "and the schedule lands on zero, give or take the rounding"
    );
}

#[test]
fn a_loan_reports_the_interest_it_charges_over_what_was_lent() {
    let mut credit = test_credit(9_167_999, 12);
    credit.kind = "loan".to_string();
    credit.principal = Some(100_000_000);
    credit.monthly_rate_ppm = Some(150);

    let schedule: Vec<crate::models::CreditInstallment> = (1..=12)
        .map(|n| test_row("installment", n, 9_167_999, "2026-03-01", false))
        .collect();

    // Twelve payments against the money actually lent, with no cash price in
    // sight: a loan knows its cost from the other side.
    assert_eq!(credit_interest(&credit, &schedule), Some(10_015_988));
}

#[test]
fn a_loan_needs_the_amount_financed() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 0);

    let err = h
        .service
        .create_credit(NewCredit {
            account_id: account,
            name: "Loan".to_string(),
            category: "OTHER".to_string(),
            kind: "loan".to_string(),
            down_payment_cents: 0,
            down_payment_date: None,
            installment_amount_cents: 50_000,
            installment_count: 12,
            first_due_date: "2026-03-01".to_string(),
            cash_price_cents: None,
            principal_cents: None,
            monthly_rate_ppm: Some(15_000),
        })
        .expect_err("a loan with nothing lent is not a loan");
    assert!(matches!(err, FinanceError::Validation(_)));
}

#[test]
fn correcting_an_installment_makes_an_irregular_schedule_possible() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 1_000_000);
    let credit = create_test_credit(&h.service, &account, "Car", 50_000, 4, "2026-03-01");
    let schedule = installments_of(&h.service, &credit);

    // A balloon final payment: the last row is simply worth more than the rest.
    h.service
        .update_installment(UpdateInstallment {
            installment_id: schedule[3].id.clone(),
            amount_cents: 400_000,
            due_date: "2026-06-15".to_string(),
        })
        .expect("correct the last installment");

    let updated = installments_of(&h.service, &credit);
    assert_eq!(updated[3].amount, 400_000);
    assert_eq!(updated[3].due_date, "2026-06-15");
    assert_eq!(
        credit_totals(&updated).plan,
        550_000,
        "and the total follows the rows, not the nominal installment"
    );

    // Paying it charges what the row says, not what the credit was created with.
    h.service
        .pay_installment(updated[3].id.clone(), None)
        .expect("pay");
    let payment = h
        .service
        .get_transactions()
        .expect("transactions")
        .into_iter()
        .find(|tx| tx.description == "Car 4/4")
        .expect("the payment");
    assert_eq!(payment.amount, 400_000);
}

#[test]
fn a_paid_installment_cannot_be_corrected_behind_the_ledgers_back() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 1_000_000);
    let credit = create_test_credit(&h.service, &account, "Car", 50_000, 4, "2026-03-01");
    let first = installments_of(&h.service, &credit).remove(0);
    h.service
        .pay_installment(first.id.clone(), None)
        .expect("pay");

    let err = h
        .service
        .update_installment(UpdateInstallment {
            installment_id: first.id,
            amount_cents: 999,
            due_date: "2026-03-01".to_string(),
        })
        .expect_err("the transaction is the amount once it is paid");
    assert!(matches!(err, FinanceError::Validation(_)));
}

#[test]
fn a_charge_is_payable_but_never_part_of_the_plan() {
    let h = new_test_service();
    let account = create_test_account(&h.service, "Checking", "USD", 1_000_000);
    let credit = create_test_credit(&h.service, &account, "Car", 50_000, 4, "2026-03-01");

    h.service
        .add_charge(NewCharge {
            credit_id: credit.clone(),
            amount_cents: 7_500,
            date: "2026-04-05".to_string(),
            note: "Interés por atraso".to_string(),
        })
        .expect("add charge");

    let schedule = installments_of(&h.service, &credit);
    let totals = credit_totals(&schedule);
    assert_eq!(totals.plan, 200_000, "the plan is untouched by a fee");
    assert_eq!(totals.charges, 7_500);

    let charge = schedule
        .iter()
        .find(|row| row.kind == "charge")
        .expect("the charge");
    h.service
        .pay_installment(charge.id.clone(), None)
        .expect("pay the charge");

    // The user's own wording travels into the ledger, so it needs no translating.
    let payment = h
        .service
        .get_transactions()
        .expect("transactions")
        .into_iter()
        .find(|tx| tx.amount == 7_500)
        .expect("the payment");
    assert_eq!(payment.description, "Car - Interés por atraso");
}

#[test]
fn a_charge_does_not_decide_whether_the_plan_is_on_schedule() {
    let schedule = vec![
        test_row("installment", 1, 50_000, "2026-03-01", true),
        test_row("installment", 2, 50_000, "2026-04-01", false),
        // Unpaid and long overdue, yet it is the lender's fee, not the plan.
        test_row("charge", 1, 7_500, "2026-01-05", false),
    ];

    let progress = credit_progress(&schedule, "2026-03-15");
    assert_eq!(progress.overdue_count, 0);
    assert_eq!(progress.status, CreditStatus::OnTrack);
    assert_eq!(progress.next_due_date.as_deref(), Some("2026-04-01"));
}
