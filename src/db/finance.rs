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

//! Finance database operations
//!
//! CRUD operations for FIAT accounts, transactions, categories, and balance summaries.
//!
//! Reads run on pooled read-only connections (`self.read`); writes run on the
//! serialized writer connection (`self.write`). Composition within a single
//! connection uses `*_on(conn, …)` helpers to avoid nesting pool checkouts.

use super::{Database, DbError};
use crate::models::{
    Account, AccountBalance, BalanceSummary, BudgetStatus, CategoryBudget, RecurrenceFrequency,
    RecurringTransaction, Transaction, TransactionCategory,
};
use rusqlite::{Connection, Error as RusqliteError, Row, params};
use std::collections::HashMap;

impl Database {
    // ==================== FIAT Accounts CRUD ====================

    /// Creates a new account
    pub fn create_account(&self, account: &Account) -> Result<(), DbError> {
        if !account.validate() {
            return Err(DbError::InvalidAccountType);
        }

        self.write(|conn| {
            conn.execute(
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
        })
    }

    /// Reads all non-archived accounts from a given connection.
    fn get_accounts_on(conn: &Connection) -> Result<Vec<Account>, DbError> {
        let mut stmt = conn.prepare(
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

    /// Gets all non-archived accounts
    pub fn get_accounts(&self) -> Result<Vec<Account>, DbError> {
        self.read(Self::get_accounts_on)
    }

    /// Reads a single account by ID from a given connection.
    fn get_account_on(conn: &Connection, id: &str) -> Result<Account, DbError> {
        conn.query_row(
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

    /// Gets a single account by ID
    pub fn get_account(&self, id: &str) -> Result<Account, DbError> {
        self.read(|conn| Self::get_account_on(conn, id))
    }

    /// Updates an account
    pub fn update_account(&self, account: &Account) -> Result<(), DbError> {
        if !account.validate() {
            return Err(DbError::InvalidAccountType);
        }

        self.write(|conn| {
            let rows = conn.execute(
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
        })
    }

    /// Archives an account (soft delete) - only if it has no transactions
    pub fn archive_account(&self, id: &str) -> Result<(), DbError> {
        self.write(|conn| {
            // Check if account has transactions
            let tx_count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM transactions WHERE account_id = ?1 OR transfer_account_id = ?1",
                params![id],
                |row| row.get(0),
            )?;

            if tx_count > 0 {
                return Err(DbError::AccountNotEmpty);
            }

            let rows = conn.execute(
                "UPDATE accounts SET is_archived = 1 WHERE id = ?1",
                params![id],
            )?;

            if rows == 0 {
                return Err(DbError::AccountNotFound);
            }

            Ok(())
        })
    }

    pub fn get_archived_accounts(&self) -> Result<Vec<Account>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, type, currency, initial_balance, color, icon, is_archived, created_at
                 FROM accounts
                 WHERE is_archived = 1
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
        })
    }

    pub fn unarchive_account(&self, id: &str) -> Result<(), DbError> {
        self.write(|conn| {
            let rows = conn.execute(
                "UPDATE accounts SET is_archived = 0 WHERE id = ?1",
                params![id],
            )?;

            if rows == 0 {
                return Err(DbError::AccountNotFound);
            }

            Ok(())
        })
    }

    /// Computes the balance for an account on a given connection.
    fn account_balance_on(conn: &Connection, account_id: &str) -> Result<AccountBalance, DbError> {
        // First get the account to verify it exists and get initial balance
        let account = Self::get_account_on(conn, account_id)?;

        // Calculate income (money coming IN to this account)
        let total_income: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE account_id = ?1 AND type = 'income'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Add transfers IN (where this account is the destination)
        let transfers_in: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE transfer_account_id = ?1 AND type = 'transfer'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Calculate expenses (money going OUT of this account)
        let total_expense: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE account_id = ?1 AND type = 'expense'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Add transfers OUT (where this account is the source)
        let transfers_out: i64 = conn
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

    /// Gets the calculated balance for an account
    pub fn get_account_balance(&self, account_id: &str) -> Result<AccountBalance, DbError> {
        self.read(|conn| Self::account_balance_on(conn, account_id))
    }

    /// Gets balances for all non-archived accounts
    pub fn get_all_account_balances(&self) -> Result<Vec<AccountBalance>, DbError> {
        self.read(|conn| {
            let accounts = Self::get_accounts_on(conn)?;
            let mut balances = Vec::with_capacity(accounts.len());

            for account in accounts {
                balances.push(Self::account_balance_on(conn, &account.id)?);
            }

            Ok(balances)
        })
    }

    // ==================== Reconciliation ====================

    /// Balance of `account_id` counting only what the user has confirmed.
    ///
    /// The account's opening balance is included unconditionally: it is the
    /// figure the user declared the account started at, so it is confirmed by
    /// definition and the difference is meant to be measured from there.
    pub fn reconciled_balance(&self, account_id: &str) -> Result<i64, DbError> {
        self.read(|conn| Self::reconciled_balance_on(conn, account_id))
    }

    fn reconciled_balance_on(conn: &Connection, account_id: &str) -> Result<i64, DbError> {
        let account = Self::get_account_on(conn, account_id)?;

        let sum = |sql: &str| -> i64 {
            conn.query_row(sql, params![account_id], |row| row.get(0))
                .unwrap_or(0)
        };

        // Each half of a transfer carries its own flag, so the incoming side
        // reads `transfer_reconciled` and the outgoing side reads `reconciled`.
        let income = sum("SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE account_id = ?1 AND type = 'income' AND reconciled = 1");
        let transfers_in = sum("SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE transfer_account_id = ?1 AND type = 'transfer' AND transfer_reconciled = 1");
        let expense = sum("SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE account_id = ?1 AND type = 'expense' AND reconciled = 1");
        let transfers_out = sum("SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE account_id = ?1 AND type = 'transfer' AND reconciled = 1");

        Ok(account.initial_balance + income + transfers_in - expense - transfers_out)
    }

    /// Rows of `account_id` the user has not confirmed yet, oldest first.
    ///
    /// Oldest first because that is the order a statement lists them in, and
    /// ticking down a list in the same order as the paper is the whole point.
    pub fn unreconciled_transactions(&self, account_id: &str) -> Result<Vec<Transaction>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, account_id, amount, category, description, date, type, transfer_account_id,
                        reconciled, transfer_reconciled
                 FROM transactions
                 WHERE (account_id = ?1 AND reconciled = 0)
                    OR (transfer_account_id = ?1 AND type = 'transfer' AND transfer_reconciled = 0)
                 ORDER BY date ASC, rowid ASC",
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
                        reconciled: row.get(8)?,
                        transfer_reconciled: row.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(transactions)
        })
    }

    /// Marks `ids` confirmed for `account_id`. Returns how many rows changed.
    ///
    /// Which flag moves depends on the side `account_id` sits on, so confirming
    /// a transfer against one statement leaves the other account untouched.
    /// All or nothing: a reconciliation is one act, and a half-applied one
    /// would leave a difference the user cannot explain.
    pub fn set_reconciled(&self, account_id: &str, ids: &[String]) -> Result<usize, DbError> {
        self.with_transaction(|conn| {
            let mut outgoing = conn.prepare(
                "UPDATE transactions SET reconciled = 1
                 WHERE id = ?1 AND account_id = ?2",
            )?;
            let mut incoming = conn.prepare(
                "UPDATE transactions SET transfer_reconciled = 1
                 WHERE id = ?1 AND transfer_account_id = ?2 AND type = 'transfer'",
            )?;

            let mut changed = 0;
            for id in ids {
                changed += outgoing.execute(params![id, account_id])?;
                changed += incoming.execute(params![id, account_id])?;
            }
            Ok(changed)
        })
    }

    // ==================== Tags ====================

    /// Replaces the tags on a transaction with `tags`.
    ///
    /// Delete-then-insert rather than a diff: the caller sends the whole set it
    /// wants, and a handful of rows is not worth the bookkeeping of working out
    /// which ones changed.
    pub fn set_transaction_tags(
        &self,
        transaction_id: &str,
        tags: &[String],
    ) -> Result<(), DbError> {
        self.with_transaction(|conn| Self::set_tags_on(conn, transaction_id, tags))
    }

    fn set_tags_on(
        conn: &Connection,
        transaction_id: &str,
        tags: &[String],
    ) -> Result<(), DbError> {
        conn.execute(
            "DELETE FROM transaction_tags WHERE transaction_id = ?1",
            params![transaction_id],
        )?;
        let mut stmt = conn.prepare(
            "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag) VALUES (?1, ?2)",
        )?;
        for tag in tags {
            stmt.execute(params![transaction_id, tag])?;
        }
        Ok(())
    }

    /// Every tag on every transaction, keyed by transaction id.
    ///
    /// Fetched in one pass because the activity list needs the tags of a whole
    /// page at once, and asking per row would be a query per transaction.
    pub fn get_all_transaction_tags(&self) -> Result<HashMap<String, Vec<String>>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT transaction_id, tag FROM transaction_tags ORDER BY transaction_id, tag",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            let mut map: HashMap<String, Vec<String>> = HashMap::new();
            for row in rows {
                let (id, tag) = row?;
                map.entry(id).or_default().push(tag);
            }
            Ok(map)
        })
    }

    /// Distinct tags in use, most used first, for offering as suggestions.
    pub fn get_tag_catalog(&self) -> Result<Vec<String>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT tag FROM transaction_tags
                 GROUP BY tag
                 ORDER BY COUNT(*) DESC, tag ASC",
            )?;
            let tags = stmt
                .query_map([], |row| row.get(0))?
                .collect::<Result<Vec<String>, _>>()?;
            Ok(tags)
        })
    }

    /// Adds `tag` to every transaction in `ids`. Returns how many rows gained it.
    pub fn tag_transactions(&self, ids: &[String], tag: &str) -> Result<usize, DbError> {
        self.with_transaction(|conn| {
            let mut stmt = conn.prepare(
                "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag) VALUES (?1, ?2)",
            )?;
            let mut added = 0;
            for id in ids {
                added += stmt.execute(params![id, tag])?;
            }
            Ok(added)
        })
    }

    /// Whether an edit leaves everything a bank statement could show untouched.
    ///
    /// A confirmation says "this row matched my statement". Only the figures
    /// the bank itself reports can invalidate that — the amount, the date and
    /// which accounts moved. The category and the description are the user's
    /// own labels; renaming one must not force a whole account to be
    /// reconciled again.
    ///
    /// A row that has vanished counts as changed, so a lost id can never be
    /// mistaken for "nothing moved".
    fn bank_visible_unchanged(
        conn: &Connection,
        id: &str,
        amount: i64,
        date: &str,
        account_id: &str,
        transfer_account_id: Option<&str>,
    ) -> Result<bool, DbError> {
        let stored: Option<(i64, String, String, Option<String>)> = conn
            .query_row(
                "SELECT amount, date, account_id, transfer_account_id
                 FROM transactions WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .ok();

        let Some((old_amount, old_date, old_account, old_transfer)) = stored else {
            return Ok(false);
        };

        Ok(old_amount == amount
            && old_date == date
            && old_account == account_id
            && old_transfer.as_deref() == transfer_account_id)
    }

    // ==================== Financial Transactions CRUD ====================

    /// Creates a new transaction in the database
    pub fn create_transaction(&self, transaction: &Transaction) -> Result<(), DbError> {
        // Validate transaction
        if !transaction.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        self.write(|conn| {
            // Verify account exists
            Self::get_account_on(conn, &transaction.account_id)?;

            // For transfers, verify destination account exists and is different
            if let Some(ref transfer_id) = transaction.transfer_account_id {
                if transfer_id == &transaction.account_id {
                    return Err(DbError::SameAccountTransfer);
                }
                Self::get_account_on(conn, transfer_id)?;
            }

            conn.execute(
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
        })
    }

    /// Updates an existing transaction
    pub fn update_transaction(&self, transaction: &Transaction) -> Result<(), DbError> {
        if !transaction.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        self.write(|conn| {
            Self::get_account_on(conn, &transaction.account_id)?;

            if let Some(ref transfer_id) = transaction.transfer_account_id {
                if transfer_id == &transaction.account_id {
                    return Err(DbError::SameAccountTransfer);
                }
                Self::get_account_on(conn, transfer_id)?;
            }

            let keeps_confirmation = Self::bank_visible_unchanged(
                conn,
                &transaction.id,
                transaction.amount,
                &transaction.date,
                &transaction.account_id,
                transaction.transfer_account_id.as_deref(),
            )?;

            conn.execute(
                "UPDATE transactions
                 SET account_id = ?2, amount = ?3, category = ?4, description = ?5, date = ?6, type = ?7, transfer_account_id = ?8,
                     reconciled = reconciled AND ?9,
                     transfer_reconciled = transfer_reconciled AND ?9
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
                    keeps_confirmation,
                ],
            )?;

            Ok(())
        })
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

        self.write(|conn| {
            // Verify both accounts exist
            Self::get_account_on(conn, from_account_id)?;
            Self::get_account_on(conn, to_account_id)?;

            let tx_id = uuid::Uuid::new_v4().to_string();

            // Create a single transfer transaction (from source to destination)
            conn.execute(
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
        })
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

        self.write(|conn| {
            Self::get_account_on(conn, from_account_id)?;
            Self::get_account_on(conn, to_account_id)?;

            let keeps_confirmation = Self::bank_visible_unchanged(
                conn,
                id,
                amount,
                date,
                from_account_id,
                Some(to_account_id),
            )?;

            let changed = conn.execute(
                "UPDATE transactions
                 SET account_id = ?2, amount = ?3, description = ?4, date = ?5, transfer_account_id = ?6,
                     reconciled = reconciled AND ?7,
                     transfer_reconciled = transfer_reconciled AND ?7
                 WHERE id = ?1 AND type = 'transfer'",
                params![
                    id,
                    from_account_id,
                    amount,
                    description,
                    date,
                    to_account_id,
                    keeps_confirmation
                ],
            )?;

            if changed == 0 {
                return Err(DbError::TransactionNotFound);
            }

            Ok(())
        })
    }

    /// Gets all transactions ordered by descending date
    pub fn get_transactions(&self) -> Result<Vec<Transaction>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, account_id, amount, category, description, date, type, transfer_account_id,
                        reconciled, transfer_reconciled
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
                        reconciled: row.get(8)?,
                        transfer_reconciled: row.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(transactions)
        })
    }

    /// Gets transactions for a specific account
    pub fn get_transactions_by_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<Transaction>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, account_id, amount, category, description, date, type, transfer_account_id,
                        reconciled, transfer_reconciled
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
                        reconciled: row.get(8)?,
                        transfer_reconciled: row.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(transactions)
        })
    }

    /// Deletes a transaction by its ID
    pub fn delete_transaction(&self, id: &str) -> Result<(), DbError> {
        self.write(|conn| {
            conn.execute("DELETE FROM transactions WHERE id = ?1", params![id])?;
            Ok(())
        })
    }

    /// Deletes several transactions at once. Returns how many rows went.
    ///
    /// All or nothing: the user confirms a bulk action as a single decision, so
    /// a failure halfway through must not leave the ledger partly changed with
    /// no record of where it stopped. Ids that match nothing are simply counted
    /// as zero rather than failing the batch.
    pub fn delete_transactions(&self, ids: &[String]) -> Result<usize, DbError> {
        self.with_transaction(|conn| {
            let mut stmt = conn.prepare("DELETE FROM transactions WHERE id = ?1")?;
            let mut deleted = 0;
            for id in ids {
                deleted += stmt.execute(params![id])?;
            }
            Ok(deleted)
        })
    }

    /// Moves several transactions to `category`. Returns how many rows changed.
    ///
    /// Transfers are skipped in SQL rather than rejected: their category is
    /// structural, and a selection that happens to include one should still
    /// recategorise everything else instead of failing whole.
    pub fn recategorize_transactions(
        &self,
        ids: &[String],
        category: &str,
    ) -> Result<usize, DbError> {
        self.with_transaction(|conn| {
            let mut stmt = conn.prepare(
                "UPDATE transactions SET category = ?2 WHERE id = ?1 AND type != 'transfer'",
            )?;
            let mut updated = 0;
            for id in ids {
                updated += stmt.execute(params![id, category])?;
            }
            Ok(updated)
        })
    }

    // ==================== Transaction Categories CRUD ====================

    /// Gets all categories of a specific type (expense or income)
    pub fn get_transaction_categories(
        &self,
        category_type: &str,
    ) -> Result<Vec<TransactionCategory>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
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
        })
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

        self.write(|conn| {
            // Check for duplicate names within the same type
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM transaction_categories WHERE name = ?1 AND category_type = ?2)",
                params![name, category_type],
                |row| row.get(0),
            )?;

            if exists {
                return Err(DbError::Sqlite(RusqliteError::ExecuteReturnedResults));
            }

            // Get max sort order for this type
            let max_sort: i32 = conn.query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM transaction_categories WHERE category_type = ?1",
                params![category_type],
                |row| row.get(0),
            )?;

            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO transaction_categories (id, name, category_type, sort_order, is_default, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, name, category_type, max_sort + 1, 0, now],
            )?;

            Ok(id)
        })
    }

    /// Renames a category and every transaction filed under the old name.
    ///
    /// Transactions store the category name rather than its id, so both have to
    /// move together or the existing rows silently become a separate category in
    /// filters and charts. Atomic for that reason.
    pub fn update_transaction_category(&self, id: &str, new_name: &str) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            // Check if category exists and get its type
            let (category_type, old_name): (String, String) = conn.query_row(
                "SELECT category_type, name FROM transaction_categories WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;

            // Check for duplicate names within the same type (excluding current category)
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM transaction_categories WHERE name = ?1 AND category_type = ?2 AND id != ?3)",
                params![new_name, category_type, id],
                |row| row.get(0),
            )?;

            if exists {
                return Err(DbError::Sqlite(RusqliteError::ExecuteReturnedResults));
            }

            conn.execute(
                "UPDATE transaction_categories SET name = ?1 WHERE id = ?2",
                params![new_name, id],
            )?;

            conn.execute(
                "UPDATE transactions SET category = ?1 WHERE category = ?2 COLLATE NOCASE",
                params![new_name, old_name],
            )?;

            Ok(())
        })
    }

    /// Deletes a category, refusing while transactions still reference it.
    ///
    /// Without the guard those transactions would point at a name that no longer
    /// exists in any list, which is invisible until a filter or a chart drops
    /// them.
    pub fn delete_transaction_category(&self, id: &str) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            let name: String = conn.query_row(
                "SELECT name FROM transaction_categories WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )?;

            let in_use: i64 = conn.query_row(
                "SELECT COUNT(*) FROM transactions WHERE category = ?1 COLLATE NOCASE",
                params![name],
                |row| row.get(0),
            )?;

            if in_use > 0 {
                return Err(DbError::CategoryInUse);
            }

            conn.execute(
                "DELETE FROM transaction_categories WHERE id = ?1",
                params![id],
            )?;

            Ok(())
        })
    }

    // ==================== Balance Summary ====================

    /// Gets the balance summary (income, expenses and total) including account initial balances
    pub fn get_balance_summary(&self) -> Result<BalanceSummary, DbError> {
        self.read(|conn| {
            // Get sum of all initial balances from non-archived accounts
            let initial_balances: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(initial_balance), 0) FROM accounts WHERE is_archived = 0",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            // Get income and expenses from transactions
            let (total_income, total_expense): (i64, i64) = conn
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
        })
    }
    // ==================== Recurring Transactions ====================

    /// Inserts a recurring rule.
    pub fn create_recurring_transaction(&self, rule: &RecurringTransaction) -> Result<(), DbError> {
        self.write(|conn| {
            conn.execute(
                "INSERT INTO recurring_transactions
                    (id, account_id, amount, category, description, type, frequency,
                     next_date, last_created_date, is_active, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    rule.id,
                    rule.account_id,
                    rule.amount,
                    rule.category,
                    rule.description,
                    rule.transaction_type,
                    rule.frequency,
                    rule.next_date,
                    rule.last_created_date,
                    rule.is_active as i32,
                    rule.created_at,
                ],
            )?;
            Ok(())
        })
    }

    /// Lists every rule, soonest first.
    pub fn get_recurring_transactions(&self) -> Result<Vec<RecurringTransaction>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, account_id, amount, category, description, type, frequency,
                        next_date, last_created_date, is_active, created_at
                 FROM recurring_transactions
                 ORDER BY is_active DESC, next_date ASC",
            )?;
            let rules = stmt
                .query_map([], row_to_recurring)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rules)
        })
    }

    /// Enables or disables a rule without deleting its history.
    pub fn set_recurring_active(&self, id: &str, active: bool) -> Result<(), DbError> {
        self.write(|conn| {
            let changed = conn.execute(
                "UPDATE recurring_transactions SET is_active = ?1 WHERE id = ?2",
                params![active as i32, id],
            )?;
            if changed == 0 {
                return Err(DbError::RecurringNotFound);
            }
            Ok(())
        })
    }

    /// Deletes a rule. Transactions it already created are left alone.
    pub fn delete_recurring_transaction(&self, id: &str) -> Result<(), DbError> {
        self.write(|conn| {
            let changed = conn.execute(
                "DELETE FROM recurring_transactions WHERE id = ?1",
                params![id],
            )?;
            if changed == 0 {
                return Err(DbError::RecurringNotFound);
            }
            Ok(())
        })
    }

    /// Creates every occurrence owed up to `today` and advances each rule.
    ///
    /// Runs in one transaction: a rule whose `next_date` moved must have its
    /// transaction, and vice versa. A rule several periods behind catches up in
    /// this single pass, which is what makes it safe to have skipped days.
    /// Returns how many transactions were created.
    pub fn apply_due_recurring(&self, today: &str) -> Result<usize, DbError> {
        self.with_transaction(|conn| {
            let due: Vec<RecurringTransaction> = {
                let mut stmt = conn.prepare(
                    "SELECT id, account_id, amount, category, description, type, frequency,
                            next_date, last_created_date, is_active, created_at
                     FROM recurring_transactions
                     WHERE is_active = 1 AND next_date <= ?1
                     ORDER BY next_date ASC",
                )?;
                stmt.query_map(params![today], row_to_recurring)?
                    .collect::<Result<Vec<_>, _>>()?
            };

            let today_date = parse_iso_date(today)?;
            let mut created = 0usize;

            for rule in due {
                let frequency = RecurrenceFrequency::parse(&rule.frequency)
                    .ok_or(DbError::InvalidTransactionType)?;
                let mut occurrence = parse_iso_date(&rule.next_date)?;
                let mut last_created = rule.last_created_date.clone();

                // Catch up period by period; a rule months behind lands every
                // occurrence it owes rather than only the latest one.
                while occurrence <= today_date {
                    let occurrence_str = occurrence.format("%Y-%m-%d").to_string();
                    conn.execute(
                        "INSERT INTO transactions
                            (id, account_id, amount, category, description, date, type, transfer_account_id)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
                        params![
                            uuid::Uuid::new_v4().to_string(),
                            rule.account_id,
                            rule.amount,
                            rule.category,
                            rule.description,
                            occurrence_str,
                            rule.transaction_type,
                        ],
                    )?;
                    created += 1;
                    last_created = Some(occurrence_str);

                    match frequency.next_after(occurrence) {
                        Some(next) => occurrence = next,
                        // Out of representable dates: stop rather than loop.
                        None => break,
                    }
                }

                conn.execute(
                    "UPDATE recurring_transactions
                     SET next_date = ?1, last_created_date = ?2
                     WHERE id = ?3",
                    params![
                        occurrence.format("%Y-%m-%d").to_string(),
                        last_created,
                        rule.id
                    ],
                )?;
            }

            Ok(created)
        })
    }
    // ==================== Category Budgets ====================

    /// Creates or replaces the limit for a category.
    pub fn upsert_category_budget(&self, budget: &CategoryBudget) -> Result<(), DbError> {
        self.write(|conn| {
            conn.execute(
                "INSERT INTO category_budgets (id, category, amount, created_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(category) DO UPDATE SET amount = excluded.amount",
                params![budget.id, budget.category, budget.amount, budget.created_at],
            )?;
            Ok(())
        })
    }

    pub fn delete_category_budget(&self, category: &str) -> Result<(), DbError> {
        self.write(|conn| {
            let changed = conn.execute(
                "DELETE FROM category_budgets WHERE category = ?1 COLLATE NOCASE",
                params![category],
            )?;
            if changed == 0 {
                return Err(DbError::CategoryBudgetNotFound);
            }
            Ok(())
        })
    }

    /// Every budget with what has been spent against it inside `month`.
    ///
    /// `month` is a `YYYY-MM` prefix; dates are ISO text, so a prefix match is
    /// both correct and index-friendly. Expenses only — income never counts
    /// against a spending limit.
    pub fn get_budget_status(&self, month: &str) -> Result<Vec<BudgetStatus>, DbError> {
        let like = format!("{month}%");
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT b.category, b.amount,
                        COALESCE((
                            SELECT SUM(t.amount) FROM transactions t
                            WHERE t.category = b.category COLLATE NOCASE
                              AND t.type = 'expense'
                              AND t.date LIKE ?1
                        ), 0)
                 FROM category_budgets b
                 ORDER BY b.category ASC",
            )?;
            let rows = stmt
                .query_map(params![like], |row| {
                    Ok(BudgetStatus {
                        category: row.get(0)?,
                        limit: row.get(1)?,
                        spent: row.get(2)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }
}

fn row_to_recurring(row: &Row<'_>) -> rusqlite::Result<RecurringTransaction> {
    Ok(RecurringTransaction {
        id: row.get(0)?,
        account_id: row.get(1)?,
        amount: row.get(2)?,
        category: row.get(3)?,
        description: row.get(4)?,
        transaction_type: row.get(5)?,
        frequency: row.get(6)?,
        next_date: row.get(7)?,
        last_created_date: row.get(8)?,
        is_active: row.get::<_, i32>(9)? != 0,
        created_at: row.get(10)?,
    })
}

/// Parses an ISO date, mapping a malformed one to a database error.
fn parse_iso_date(value: &str) -> Result<chrono::NaiveDate, DbError> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| DbError::InvalidDate)
}
