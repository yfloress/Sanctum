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

//! Crypto transaction database operations
//!
//! CRUD operations for crypto transactions.

use crate::db::{Database, DbError};
use crate::models::{CryptoTransaction, CryptoTransactionUpdate, CryptoTxFilter};
use rusqlite::{Connection, OptionalExtension, Row, params};

/// Escapes LIKE wildcards so searching for `50%` does not match every row.
fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Column list used in all SELECT queries for crypto transactions.
/// Kept in a single place so that any schema change only requires one update.
const CRYPTO_TX_COLUMNS: &str = "\
    id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, \
    fee_coin_id, fee_amount, subtype, override_proceeds, \
    override_cost_basis, date, notes, related_tx_id";

/// Maps a database row to a `CryptoTransaction`.
///
/// The column order MUST match [`CRYPTO_TX_COLUMNS`].
fn row_to_crypto_transaction(row: &Row<'_>) -> rusqlite::Result<CryptoTransaction> {
    Ok(CryptoTransaction {
        id: row.get(0)?,
        wallet_id: row.get(1)?,
        coin_id: row.get(2)?,
        symbol: row.get(3)?,
        transaction_type: row.get(4)?,
        amount: row.get(5)?,
        price_per_coin: row.get(6)?,
        fee: row.get(7)?,
        fee_coin_id: row.get(8)?,
        fee_amount: row.get(9)?,
        subtype: row.get(10)?,
        override_proceeds: row.get(11)?,
        override_cost_basis: row.get(12)?,
        date: row.get(13)?,
        notes: row.get(14)?,
        related_tx_id: row.get(15)?,
    })
}

impl Database {
    /// Creates a new crypto transaction
    pub fn create_crypto_transaction(&self, tx: &CryptoTransaction) -> Result<(), DbError> {
        if !tx.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        self.write(|conn| {
            // Verify wallet exists
            let wallet_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM crypto_wallets WHERE id = ?1",
                    params![&tx.wallet_id],
                    |row| row.get(0),
                )
                .unwrap_or(0)
                > 0;

            if !wallet_exists {
                return Err(DbError::WalletNotFound);
            }

            conn.execute(
                "INSERT INTO crypto_transactions
                 (id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, subtype, override_proceeds, override_cost_basis, date, notes, related_tx_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    &tx.id,
                    &tx.wallet_id,
                    &tx.coin_id,
                    &tx.symbol,
                    &tx.transaction_type,
                    &tx.amount,
                    &tx.price_per_coin,
                    &tx.fee,
                    &tx.fee_coin_id,
                    &tx.fee_amount,
                    &tx.subtype,
                    &tx.override_proceeds,
                    &tx.override_cost_basis,
                    &tx.date,
                    &tx.notes,
                    &tx.related_tx_id,
                ],
            )?;

            Ok(())
        })
    }

    /// Gets all transactions for a specific wallet
    pub fn get_wallet_transactions(
        &self,
        wallet_id: &str,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let sql = format!(
            "SELECT {} FROM crypto_transactions WHERE wallet_id = ?1 ORDER BY date DESC, rowid DESC",
            CRYPTO_TX_COLUMNS
        );
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;

            let transactions = stmt
                .query_map(params![wallet_id], row_to_crypto_transaction)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(transactions)
        })
    }

    /// Gets wallet transactions up to a given date (inclusive), ordered ascending.
    pub fn get_wallet_transactions_up_to_date(
        &self,
        wallet_id: &str,
        date: &str,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let sql = format!(
            "SELECT {} FROM crypto_transactions WHERE wallet_id = ?1 AND date <= ?2 ORDER BY date ASC, rowid ASC",
            CRYPTO_TX_COLUMNS
        );
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;

            let transactions = stmt
                .query_map(params![wallet_id, date], row_to_crypto_transaction)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(transactions)
        })
    }

    /// Gets all crypto transactions across all wallets, paginated by offset/limit.
    /// Returns limit+1 rows so the caller can detect has_more.
    pub fn get_all_crypto_transactions(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        self.get_filtered_crypto_transactions(&CryptoTxFilter::default(), offset, limit)
    }

    /// Lists transactions matching `filter`, newest first.
    ///
    /// Filtering happens in SQL rather than over an already-paginated page, so
    /// a match on row 5000 is found even when only the first page is on screen.
    /// Returns one row more than `limit` so the caller can detect further pages.
    pub fn get_filtered_crypto_transactions(
        &self,
        filter: &CryptoTxFilter,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let mut conditions: Vec<&str> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(wallet_id) = &filter.wallet_id {
            conditions.push("wallet_id = ?");
            params.push(Box::new(wallet_id.clone()));
        }
        if let Some(tx_type) = &filter.transaction_type {
            conditions.push("type = ?");
            params.push(Box::new(tx_type.clone()));
        }
        // ISO dates compare correctly as text.
        if let Some(date_from) = &filter.date_from {
            conditions.push("date >= ?");
            params.push(Box::new(date_from.clone()));
        }
        if let Some(date_to) = &filter.date_to {
            conditions.push("date <= ?");
            params.push(Box::new(date_to.clone()));
        }
        if let Some(query) = &filter.query {
            conditions.push("(symbol LIKE ? ESCAPE '\\' OR notes LIKE ? ESCAPE '\\')");
            let pattern = format!("%{}%", escape_like(query));
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        // One extra row signals that another page exists.
        params.push(Box::new(limit.saturating_add(1)));
        params.push(Box::new(offset));

        let sql = format!(
            "SELECT {CRYPTO_TX_COLUMNS} FROM crypto_transactions{where_clause} \
             ORDER BY date DESC, rowid DESC LIMIT ? OFFSET ?"
        );

        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let transactions = stmt
                .query_map(refs.as_slice(), row_to_crypto_transaction)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(transactions)
        })
    }

    /// Gets all crypto transactions for a specific coin across all wallets
    pub fn get_crypto_transactions_by_coin(
        &self,
        coin_id: &str,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let sql = format!(
            "SELECT {} FROM crypto_transactions WHERE coin_id = ?1 ORDER BY date DESC, rowid DESC",
            CRYPTO_TX_COLUMNS
        );
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;

            let transactions = stmt
                .query_map(params![coin_id], row_to_crypto_transaction)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(transactions)
        })
    }

    pub(crate) fn transfer_movements(&self) -> Result<Vec<CryptoTransaction>, DbError> {
        let sql = format!(
            "SELECT {CRYPTO_TX_COLUMNS} FROM crypto_transactions WHERE type = 'transfer' ORDER BY date DESC, id"
        );
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            Ok(stmt
                .query_map([], row_to_crypto_transaction)?
                .collect::<Result<Vec<_>, _>>()?)
        })
    }

    pub(crate) fn set_transfer_link(
        &self,
        outgoing: &str,
        incoming: &str,
        linked: bool,
        eligible: impl FnOnce(&CryptoTransaction, &CryptoTransaction) -> bool,
    ) -> Result<bool, DbError> {
        self.with_transaction(|conn| {
            let out = Self::transaction_on(conn, outgoing)?.ok_or(DbError::TransactionNotFound)?;
            let inc = Self::transaction_on(conn, incoming)?.ok_or(DbError::TransactionNotFound)?;
            let paired = out.related_tx_id.as_deref() == Some(incoming)
                && inc.related_tx_id.as_deref() == Some(outgoing);
            let free = out.related_tx_id.is_none() && inc.related_tx_id.is_none();
            if out.mechanical_type() != "transfer_out"
                || inc.mechanical_type() != "transfer_in"
                || (!paired && !free)
                || (linked && !paired && !eligible(&out, &inc))
            {
                return Ok(false);
            }
            if linked != paired {
                let update = "UPDATE crypto_transactions SET related_tx_id = ?2 WHERE id = ?1";
                conn.execute(update, params![outgoing, linked.then_some(incoming)])?;
                conn.execute(update, params![incoming, linked.then_some(outgoing)])?;
            }
            Ok(true)
        })
    }

    /// Only swaps are deleted together. A confirmed transfer has two independently
    /// sourced movements; unlink it before removing either movement.
    pub fn delete_crypto_transaction(&self, id: &str) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            let Some(tx) = Self::transaction_on(conn, id)? else {
                return Ok(());
            };
            if let Some(other) = tx.related_tx_id.as_deref()
                && let Some(counter) = Self::transaction_on(conn, other)?
                && counter.related_tx_id.as_deref() == Some(id)
            {
                if tx.mechanical_type() != "swap" || counter.mechanical_type() != "swap" {
                    return Err(DbError::LinkedCryptoTransaction);
                }
                conn.execute("DELETE FROM crypto_transactions WHERE id = ?1", [other])?;
            }
            conn.execute("DELETE FROM crypto_transactions WHERE id = ?1", [id])?;
            Ok(())
        })
    }

    fn transaction_on(conn: &Connection, id: &str) -> Result<Option<CryptoTransaction>, DbError> {
        let sql = format!("SELECT {CRYPTO_TX_COLUMNS} FROM crypto_transactions WHERE id = ?1");
        Ok(conn
            .query_row(&sql, [id], row_to_crypto_transaction)
            .optional()?)
    }

    pub fn get_crypto_transaction(&self, id: &str) -> Result<Option<CryptoTransaction>, DbError> {
        self.read(|conn| Self::transaction_on(conn, id))
    }

    /// Updates editable fields of a crypto transaction
    pub fn update_crypto_transaction_fields(
        &self,
        update: CryptoTransactionUpdate<'_>,
    ) -> Result<(), DbError> {
        let CryptoTransactionUpdate {
            id,
            amount,
            price_per_coin,
            fee,
            fee_coin_id,
            fee_amount,
            date,
            notes,
            subtype,
            override_proceeds,
            override_cost_basis,
        } = update;

        self.write(|conn| {
            let changed = conn.execute(
                "UPDATE crypto_transactions
                 SET amount = ?1,
                     price_per_coin = ?2,
                     fee = ?3,
                     fee_coin_id = ?4,
                     fee_amount = ?5,
                     subtype = ?6,
                     override_proceeds = ?7,
                     override_cost_basis = ?8,
                     date = ?9,
                     notes = ?10
                 WHERE id = ?11 AND related_tx_id IS NULL",
                params![
                    amount,
                    price_per_coin,
                    fee,
                    fee_coin_id,
                    fee_amount,
                    subtype,
                    override_proceeds,
                    override_cost_basis,
                    date,
                    notes,
                    id
                ],
            )?;
            if changed == 0 {
                return Err(DbError::LinkedCryptoTransaction);
            }
            Ok(())
        })
    }
}
