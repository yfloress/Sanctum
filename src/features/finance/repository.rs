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

//! Finance repository
//!
//! Database operations for finance feature.
//! Delegates to the main Database struct.

use crate::db::{Database, DbError};
// Use shared models from the central domain layer
use crate::models::{Account, AccountBalance, BalanceSummary, Transaction, TransactionCategory};

/// Repository for finance-related database operations
pub struct FinanceRepository;

impl FinanceRepository {
    // Account operations
    pub fn create_account(db: &Database, account: &Account) -> Result<(), DbError> {
        db.create_account(account)
    }

    pub fn get_accounts(db: &Database) -> Result<Vec<Account>, DbError> {
        db.get_accounts()
    }

    pub fn get_account(db: &Database, id: &str) -> Result<Account, DbError> {
        db.get_account(id)
    }

    pub fn update_account(db: &Database, account: &Account) -> Result<(), DbError> {
        db.update_account(account)
    }

    pub fn archive_account(db: &Database, id: &str) -> Result<(), DbError> {
        db.archive_account(id)
    }

    pub fn get_archived_accounts(db: &Database) -> Result<Vec<Account>, DbError> {
        db.get_archived_accounts()
    }

    pub fn unarchive_account(db: &Database, id: &str) -> Result<(), DbError> {
        db.unarchive_account(id)
    }

    pub fn get_account_balance(db: &Database, account_id: &str) -> Result<AccountBalance, DbError> {
        db.get_account_balance(account_id)
    }

    pub fn get_all_account_balances(db: &Database) -> Result<Vec<AccountBalance>, DbError> {
        db.get_all_account_balances()
    }

    // Transaction operations
    pub fn create_transaction(db: &Database, transaction: &Transaction) -> Result<(), DbError> {
        db.create_transaction(transaction)
    }

    pub fn update_transaction(db: &Database, transaction: &Transaction) -> Result<(), DbError> {
        db.update_transaction(transaction)
    }

    pub fn get_transactions(db: &Database) -> Result<Vec<Transaction>, DbError> {
        db.get_transactions()
    }

    pub fn get_transactions_by_account(
        db: &Database,
        account_id: &str,
    ) -> Result<Vec<Transaction>, DbError> {
        db.get_transactions_by_account(account_id)
    }

    pub fn delete_transaction(db: &Database, id: &str) -> Result<(), DbError> {
        db.delete_transaction(id)
    }

    pub fn create_transfer(
        db: &Database,
        from_id: &str,
        to_id: &str,
        amount: i64,
        description: &str,
        date: &str,
    ) -> Result<String, DbError> {
        db.create_transfer(from_id, to_id, amount, description, date)
    }

    pub fn update_transfer(
        db: &Database,
        id: &str,
        from_id: &str,
        to_id: &str,
        amount: i64,
        description: &str,
        date: &str,
    ) -> Result<(), DbError> {
        db.update_transfer(id, from_id, to_id, amount, description, date)
    }

    // Category operations
    pub fn get_transaction_categories(
        db: &Database,
        category_type: &str,
    ) -> Result<Vec<TransactionCategory>, DbError> {
        db.get_transaction_categories(category_type)
    }

    pub fn add_transaction_category(
        db: &Database,
        name: &str,
        category_type: &str,
    ) -> Result<String, DbError> {
        db.add_transaction_category(name, category_type)
    }

    pub fn update_transaction_category(
        db: &Database,
        id: &str,
        new_name: &str,
    ) -> Result<(), DbError> {
        db.update_transaction_category(id, new_name)
    }

    pub fn delete_transaction_category(db: &Database, id: &str) -> Result<(), DbError> {
        db.delete_transaction_category(id)
    }

    // Balance operations
    pub fn get_balance_summary(db: &Database) -> Result<BalanceSummary, DbError> {
        db.get_balance_summary()
    }

    // Exchange rate operations
    pub fn save_exchange_rate(db: &Database, pair: &str, rate: f64) -> Result<(), DbError> {
        db.save_exchange_rate(pair, rate)
    }

    pub fn load_exchange_rate(db: &Database, pair: &str) -> Result<Option<(f64, String)>, DbError> {
        db.load_exchange_rate(pair)
    }
}
