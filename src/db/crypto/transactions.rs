//! Crypto transaction database operations
//!
//! CRUD operations for crypto transactions.

use crate::db::{Database, DbError};
use crate::models::CryptoTransaction;
use rusqlite::{params, Error as RusqliteError};

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
             (id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions
             WHERE wallet_id = ?1
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map(params![wallet_id], |row| {
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
                    date: row.get(10)?,
                    notes: row.get(11)?,
                    related_tx_id: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Gets wallet transactions up to a given date (inclusive), ordered ascending.
    pub fn get_wallet_transactions_up_to_date(
        &self,
        wallet_id: &str,
        date: &str,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions
             WHERE wallet_id = ?1
               AND date <= ?2
             ORDER BY date ASC, rowid ASC",
        )?;

        let transactions = stmt
            .query_map(params![wallet_id, date], |row| {
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
                    date: row.get(10)?,
                    notes: row.get(11)?,
                    related_tx_id: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Gets all crypto transactions across all wallets
    pub fn get_all_crypto_transactions(&self) -> Result<Vec<CryptoTransaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map([], |row| {
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
                    date: row.get(10)?,
                    notes: row.get(11)?,
                    related_tx_id: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Gets all crypto transactions for a specific coin across all wallets
    pub fn get_crypto_transactions_by_coin(
        &self,
        coin_id: &str,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions
             WHERE coin_id = ?1
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map(params![coin_id], |row| {
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
                    date: row.get(10)?,
                    notes: row.get(11)?,
                    related_tx_id: row.get(12)?,
                })
            })?
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
        let result = self.conn.query_row(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions WHERE id = ?1",
            params![id],
            |row| {
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
                    date: row.get(10)?,
                    notes: row.get(11)?,
                    related_tx_id: row.get(12)?,
                })
            },
        );

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
    ) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE crypto_transactions
             SET amount = ?1,
                 price_per_coin = ?2,
                 fee = ?3,
                 fee_coin_id = ?4,
                 fee_amount = ?5,
                 date = ?6,
                 notes = ?7
             WHERE id = ?8",
            params![
                amount,
                price_per_coin,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
                id
            ],
        )?;
        Ok(())
    }
}
