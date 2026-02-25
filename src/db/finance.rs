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

//! Finance database operations
//!
//! CRUD operations for FIAT accounts, transactions, categories, and balance summaries.

use super::{Database, DbError};
use crate::models::{Account, AccountBalance, BalanceSummary, Transaction, TransactionCategory};
use rusqlite::{Error as RusqliteError, params};

impl Database {
    // ==================== FIAT Accounts CRUD ====================

    /// Creates a new account
    pub fn create_account(&self, account: &Account) -> Result<(), DbError> {
        if !account.validate() {
            return Err(DbError::InvalidAccountType);
        }

        self.conn.execute(
            "INSERT INTO accounts (id, name, type, currency, initial_balance, color, icon, is_archived, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &account.id,
                &account.name,
                &account.account_type,
                &account.currency,
                &account.initial_balance,
                &account.color,
                &account.icon,
                account.is_archived,
                &account.created_at,
            ],
        )?;

        Ok(())
    }

    /// Gets all non-archived accounts
    pub fn get_accounts(&self) -> Result<Vec<Account>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, currency, initial_balance, color, icon, is_archived, created_at
             FROM accounts
             WHERE is_archived = 0
             ORDER BY created_at ASC",
        )?;

        let accounts = stmt
            .query_map([], |row| {
                Ok(Account {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    account_type: row.get(2)?,
                    currency: row.get(3)?,
                    initial_balance: row.get(4)?,
                    color: row.get(5)?,
                    icon: row.get(6)?,
                    is_archived: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accounts)
    }

    /// Gets a single account by ID
    pub fn get_account(&self, id: &str) -> Result<Account, DbError> {
        self.conn
            .query_row(
                "SELECT id, name, type, currency, initial_balance, color, icon, is_archived, created_at
                 FROM accounts WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Account {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        account_type: row.get(2)?,
                        currency: row.get(3)?,
                        initial_balance: row.get(4)?,
                        color: row.get(5)?,
                        icon: row.get(6)?,
                        is_archived: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                },
            )
            .map_err(|e| match e {
                RusqliteError::QueryReturnedNoRows => DbError::AccountNotFound,
                _ => DbError::Sqlite(e),
            })
    }

    /// Updates an account
    pub fn update_account(&self, account: &Account) -> Result<(), DbError> {
        if !account.validate() {
            return Err(DbError::InvalidAccountType);
        }

        let rows = self.conn.execute(
            "UPDATE accounts SET name = ?1, type = ?2, currency = ?3, initial_balance = ?4, color = ?5, icon = ?6
             WHERE id = ?7 AND is_archived = 0",
            params![
                &account.name,
                &account.account_type,
                &account.currency,
                &account.initial_balance,
                &account.color,
                &account.icon,
                &account.id,
            ],
        )?;

        if rows == 0 {
            return Err(DbError::AccountNotFound);
        }

        Ok(())
    }

    /// Archives an account (soft delete) - only if it has no transactions
    pub fn archive_account(&self, id: &str) -> Result<(), DbError> {
        // Check if account has transactions
        let tx_count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE account_id = ?1 OR transfer_account_id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        if tx_count > 0 {
            return Err(DbError::AccountNotEmpty);
        }

        let rows = self.conn.execute(
            "UPDATE accounts SET is_archived = 1 WHERE id = ?1",
            params![id],
        )?;

        if rows == 0 {
            return Err(DbError::AccountNotFound);
        }

        Ok(())
    }

    /// Gets the calculated balance for an account
    pub fn get_account_balance(&self, account_id: &str) -> Result<AccountBalance, DbError> {
        // First get the account to verify it exists and get initial balance
        let account = self.get_account(account_id)?;

        // Calculate income (money coming IN to this account)
        let total_income: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE account_id = ?1 AND type = 'income'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Add transfers IN (where this account is the destination)
        let transfers_in: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE transfer_account_id = ?1 AND type = 'transfer'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Calculate expenses (money going OUT of this account)
        let total_expense: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE account_id = ?1 AND type = 'expense'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Add transfers OUT (where this account is the source)
        let transfers_out: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE account_id = ?1 AND type = 'transfer'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let current_balance =
            account.initial_balance + total_income + transfers_in - total_expense - transfers_out;

        Ok(AccountBalance {
            account_id: account_id.to_string(),
            account_name: account.name,
            current_balance,
            total_income: total_income + transfers_in,
            total_expense: total_expense + transfers_out,
        })
    }

    /// Gets balances for all non-archived accounts
    pub fn get_all_account_balances(&self) -> Result<Vec<AccountBalance>, DbError> {
        let accounts = self.get_accounts()?;
        let mut balances = Vec::with_capacity(accounts.len());

        for account in accounts {
            let balance = self.get_account_balance(&account.id)?;
            balances.push(balance);
        }

        Ok(balances)
    }

    // ==================== Financial Transactions CRUD ====================

    /// Creates a new transaction in the database
    pub fn create_transaction(&self, transaction: &Transaction) -> Result<(), DbError> {
        // Validate transaction
        if !transaction.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        // Verify account exists
        self.get_account(&transaction.account_id)?;

        // For transfers, verify destination account exists and is different
        if let Some(ref transfer_id) = transaction.transfer_account_id {
            if transfer_id == &transaction.account_id {
                return Err(DbError::SameAccountTransfer);
            }
            self.get_account(transfer_id)?;
        }

        self.conn.execute(
            "INSERT INTO transactions (id, account_id, amount, category, description, date, type, transfer_account_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &transaction.id,
                &transaction.account_id,
                &transaction.amount,
                &transaction.category,
                &transaction.description,
                &transaction.date,
                &transaction.transaction_type,
                &transaction.transfer_account_id,
            ],
        )?;

        Ok(())
    }

    /// Updates an existing transaction
    pub fn update_transaction(&self, transaction: &Transaction) -> Result<(), DbError> {
        if !transaction.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        self.get_account(&transaction.account_id)?;

        if let Some(ref transfer_id) = transaction.transfer_account_id {
            if transfer_id == &transaction.account_id {
                return Err(DbError::SameAccountTransfer);
            }
            self.get_account(transfer_id)?;
        }

        self.conn.execute(
            "UPDATE transactions
             SET account_id = ?2, amount = ?3, category = ?4, description = ?5, date = ?6, type = ?7, transfer_account_id = ?8
             WHERE id = ?1",
            params![
                &transaction.id,
                &transaction.account_id,
                &transaction.amount,
                &transaction.category,
                &transaction.description,
                &transaction.date,
                &transaction.transaction_type,
                &transaction.transfer_account_id,
            ],
        )?;

        Ok(())
    }

    /// Creates a transfer between two accounts (atomic operation)
    pub fn create_transfer(
        &self,
        from_account_id: &str,
        to_account_id: &str,
        amount: i64,
        description: &str,
        date: &str,
    ) -> Result<String, DbError> {
        if from_account_id == to_account_id {
            return Err(DbError::SameAccountTransfer);
        }

        // Verify both accounts exist
        self.get_account(from_account_id)?;
        self.get_account(to_account_id)?;

        let tx_id = uuid::Uuid::new_v4().to_string();

        // Create a single transfer transaction (from source to destination)
        self.conn.execute(
            "INSERT INTO transactions (id, account_id, amount, category, description, date, type, transfer_account_id)
             VALUES (?1, ?2, ?3, 'Transfer', ?4, ?5, 'transfer', ?6)",
            params![
                &tx_id,
                from_account_id,
                amount,
                description,
                date,
                to_account_id,
            ],
        )?;

        Ok(tx_id)
    }

    /// Updates an existing transfer transaction
    pub fn update_transfer(
        &self,
        id: &str,
        from_account_id: &str,
        to_account_id: &str,
        amount: i64,
        description: &str,
        date: &str,
    ) -> Result<(), DbError> {
        if from_account_id == to_account_id {
            return Err(DbError::SameAccountTransfer);
        }

        self.get_account(from_account_id)?;
        self.get_account(to_account_id)?;

        let changed = self.conn.execute(
            "UPDATE transactions
             SET account_id = ?2, amount = ?3, description = ?4, date = ?5, transfer_account_id = ?6
             WHERE id = ?1 AND type = 'transfer'",
            params![
                id,
                from_account_id,
                amount,
                description,
                date,
                to_account_id
            ],
        )?;

        if changed == 0 {
            return Err(DbError::TransactionNotFound);
        }

        Ok(())
    }

    /// Gets all transactions ordered by descending date
    pub fn get_transactions(&self) -> Result<Vec<Transaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, amount, category, description, date, type, transfer_account_id
             FROM transactions
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map([], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    amount: row.get(2)?,
                    category: row.get(3)?,
                    description: row.get(4)?,
                    date: row.get(5)?,
                    transaction_type: row.get(6)?,
                    transfer_account_id: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Gets transactions for a specific account
    pub fn get_transactions_by_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<Transaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, amount, category, description, date, type, transfer_account_id
             FROM transactions
             WHERE account_id = ?1 OR transfer_account_id = ?1
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map(params![account_id], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    amount: row.get(2)?,
                    category: row.get(3)?,
                    description: row.get(4)?,
                    date: row.get(5)?,
                    transaction_type: row.get(6)?,
                    transfer_account_id: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Deletes a transaction by its ID
    pub fn delete_transaction(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM transactions WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== Transaction Categories CRUD ====================

    /// Gets all categories of a specific type (expense or income)
    pub fn get_transaction_categories(
        &self,
        category_type: &str,
    ) -> Result<Vec<TransactionCategory>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, category_type, sort_order, is_default, created_at
             FROM transaction_categories
             WHERE category_type = ?1
             ORDER BY sort_order, name",
        )?;

        let categories = stmt
            .query_map(params![category_type], |row| {
                Ok(TransactionCategory {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category_type: row.get(2)?,
                    sort_order: row.get(3)?,
                    is_default: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(categories)
    }

    /// Adds a new transaction category
    pub fn add_transaction_category(
        &self,
        name: &str,
        category_type: &str,
    ) -> Result<String, DbError> {
        // Validate category type
        if category_type != "expense" && category_type != "income" {
            return Err(DbError::InvalidTransactionType);
        }

        // Check for duplicate names within the same type
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM transaction_categories WHERE name = ?1 AND category_type = ?2)",
            params![name, category_type],
            |row| row.get(0),
        )?;

        if exists {
            return Err(DbError::Sqlite(RusqliteError::ExecuteReturnedResults));
        }

        // Get max sort order for this type
        let max_sort: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM transaction_categories WHERE category_type = ?1",
            params![category_type],
            |row| row.get(0),
        )?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO transaction_categories (id, name, category_type, sort_order, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, category_type, max_sort + 1, 0, now],
        )?;

        Ok(id)
    }

    /// Updates a category name
    pub fn update_transaction_category(&self, id: &str, new_name: &str) -> Result<(), DbError> {
        // Check if category exists and get its type
        let category_type: String = self.conn.query_row(
            "SELECT category_type FROM transaction_categories WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        // Check for duplicate names within the same type (excluding current category)
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM transaction_categories WHERE name = ?1 AND category_type = ?2 AND id != ?3)",
            params![new_name, category_type, id],
            |row| row.get(0),
        )?;

        if exists {
            return Err(DbError::Sqlite(RusqliteError::ExecuteReturnedResults));
        }

        self.conn.execute(
            "UPDATE transaction_categories SET name = ?1 WHERE id = ?2",
            params![new_name, id],
        )?;

        Ok(())
    }

    /// Deletes a category
    pub fn delete_transaction_category(&self, id: &str) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM transaction_categories WHERE id = ?1",
            params![id],
        )?;

        Ok(())
    }

    // ==================== Balance Summary ====================

    /// Gets the balance summary (income, expenses and total) including account initial balances
    pub fn get_balance_summary(&self) -> Result<BalanceSummary, DbError> {
        // Get sum of all initial balances from non-archived accounts
        let initial_balances: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(initial_balance), 0) FROM accounts WHERE is_archived = 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Get income and expenses from transactions
        let (total_income, total_expense): (i64, i64) = self
            .conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN type = 'income' THEN amount ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN type = 'expense' THEN amount ELSE 0 END), 0)
                 FROM transactions",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(DbError::Sqlite)?;

        // Total balance = initial balances + income - expenses
        let total_balance = initial_balances + total_income - total_expense;

        Ok(BalanceSummary {
            total_balance,
            total_income,
            total_expense,
        })
    }
}
