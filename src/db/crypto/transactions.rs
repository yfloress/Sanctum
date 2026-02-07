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

//! Crypto transaction database operations
//!
//! CRUD operations for crypto transactions.

use crate::db::{Database, DbError};
use crate::models::CryptoTransaction;
use rusqlite::{Error as RusqliteError, Row, params};

/// Column list used in all SELECT queries for crypto transactions.
/// Kept in a single place so that any schema change only requires one update.
const CRYPTO_TX_COLUMNS: &str = "\
    id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, \
    fee_coin_id, fee_amount, tax_type, tax_subtype, override_proceeds, \
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
        tax_type: row.get(10)?,
        tax_subtype: row.get(11)?,
        override_proceeds: row.get(12)?,
        override_cost_basis: row.get(13)?,
        date: row.get(14)?,
        notes: row.get(15)?,
        related_tx_id: row.get(16)?,
    })
}

impl Database {
    /// Creates a new crypto transaction
    pub fn create_crypto_transaction(&self, tx: &CryptoTransaction) -> Result<(), DbError> {
        if !tx.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        // Verify wallet exists
        let wallet_exists: bool = self
            .conn
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

        self.conn.execute(
            "INSERT INTO crypto_transactions
             (id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, tax_type, tax_subtype, override_proceeds, override_cost_basis, date, notes, related_tx_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
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
                &tx.tax_type,
                &tx.tax_subtype,
                &tx.override_proceeds,
                &tx.override_cost_basis,
                &tx.date,
                &tx.notes,
                &tx.related_tx_id,
            ],
        )?;

        Ok(())
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
        let mut stmt = self.conn.prepare(&sql)?;

        let transactions = stmt
            .query_map(params![wallet_id], row_to_crypto_transaction)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
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
        let mut stmt = self.conn.prepare(&sql)?;

        let transactions = stmt
            .query_map(params![wallet_id, date], row_to_crypto_transaction)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Gets all crypto transactions across all wallets
    pub fn get_all_crypto_transactions(&self) -> Result<Vec<CryptoTransaction>, DbError> {
        let sql = format!(
            "SELECT {} FROM crypto_transactions ORDER BY date DESC, rowid DESC",
            CRYPTO_TX_COLUMNS
        );
        let mut stmt = self.conn.prepare(&sql)?;

        let transactions = stmt
            .query_map([], row_to_crypto_transaction)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
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
        let mut stmt = self.conn.prepare(&sql)?;

        let transactions = stmt
            .query_map(params![coin_id], row_to_crypto_transaction)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Deletes a crypto transaction by ID
    pub fn delete_crypto_transaction(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM crypto_transactions WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Gets a crypto transaction by ID
    pub fn get_crypto_transaction(&self, id: &str) -> Result<Option<CryptoTransaction>, DbError> {
        let sql = format!(
            "SELECT {} FROM crypto_transactions WHERE id = ?1",
            CRYPTO_TX_COLUMNS
        );

        let result = self
            .conn
            .query_row(&sql, params![id], row_to_crypto_transaction);

        match result {
            Ok(tx) => Ok(Some(tx)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Updates editable fields of a crypto transaction
    #[allow(clippy::too_many_arguments)]
    pub fn update_crypto_transaction_fields(
        &self,
        id: &str,
        amount: f64,
        price_per_coin: Option<f64>,
        fee: Option<f64>,
        fee_coin_id: Option<&str>,
        fee_amount: Option<f64>,
        date: &str,
        notes: Option<&str>,
        tax_type: Option<&str>,
        tax_subtype: Option<&str>,
        override_proceeds: Option<f64>,
        override_cost_basis: Option<f64>,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE crypto_transactions
             SET amount = ?1,
                 price_per_coin = ?2,
                 fee = ?3,
                 fee_coin_id = ?4,
                 fee_amount = ?5,
                 tax_type = ?6,
                 tax_subtype = ?7,
                 override_proceeds = ?8,
                 override_cost_basis = ?9,
                 date = ?10,
                 notes = ?11
             WHERE id = ?12",
            params![
                amount,
                price_per_coin,
                fee,
                fee_coin_id,
                fee_amount,
                tax_type,
                tax_subtype,
                override_proceeds,
                override_cost_basis,
                date,
                notes,
                id
            ],
        )?;
        Ok(())
    }
}
