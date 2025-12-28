//! Crypto wallet database operations
//!
//! CRUD operations for crypto wallets.

use crate::db::{Database, DbError};
use crate::models::CryptoWallet;
use rusqlite::{params, Error as RusqliteError};

impl Database {
    /// Creates a new crypto wallet
    pub fn create_wallet(&self, wallet: &CryptoWallet) -> Result<(), DbError> {
        if !wallet.validate() {
            return Err(DbError::InvalidWalletCategory);
        }

        self.conn.execute(
            "INSERT INTO crypto_wallets (id, name, category, icon) VALUES (?1, ?2, ?3, ?4)",
            params![&wallet.id, &wallet.name, &wallet.category, &wallet.icon],
        )?;

        Ok(())
    }

    /// Gets all wallets
    pub fn get_wallets(&self) -> Result<Vec<CryptoWallet>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, category, icon FROM crypto_wallets ORDER BY name ASC")?;

        let wallets = stmt
            .query_map([], |row| {
                Ok(CryptoWallet {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    icon: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(wallets)
    }

    /// Gets a single wallet by ID
    pub fn get_wallet(&self, id: &str) -> Result<Option<CryptoWallet>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, name, category, icon FROM crypto_wallets WHERE id = ?1",
            params![id],
            |row| {
                Ok(CryptoWallet {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    icon: row.get(3)?,
                })
            },
        );

        match result {
            Ok(wallet) => Ok(Some(wallet)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Updates a wallet
    pub fn update_wallet(&self, wallet: &CryptoWallet) -> Result<(), DbError> {
        if !wallet.validate() {
            return Err(DbError::InvalidWalletCategory);
        }

        let rows = self.conn.execute(
            "UPDATE crypto_wallets SET name = ?2, category = ?3, icon = ?4 WHERE id = ?1",
            params![&wallet.id, &wallet.name, &wallet.category, &wallet.icon],
        )?;

        if rows == 0 {
            return Err(DbError::WalletNotFound);
        }

        Ok(())
    }

    /// Deletes a wallet (blocks if wallet has transactions)
    pub fn delete_wallet(&self, id: &str) -> Result<(), DbError> {
        // Block deletion if wallet has existing transactions
        let tx_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM crypto_transactions WHERE wallet_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if tx_count > 0 {
            return Err(DbError::WalletNotEmpty);
        }

        let rows = self
            .conn
            .execute("DELETE FROM crypto_wallets WHERE id = ?1", params![id])?;

        if rows == 0 {
            return Err(DbError::WalletNotFound);
        }

        Ok(())
    }
}
