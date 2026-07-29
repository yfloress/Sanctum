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

//! Crypto wallet database operations
//!
//! CRUD operations for crypto wallets.

use crate::db::{Database, DbError};
use crate::models::CryptoWallet;
use rusqlite::{Connection, Error as RusqliteError, params};

impl Database {
    /// Creates a new crypto wallet
    pub fn create_wallet(&self, wallet: &CryptoWallet) -> Result<(), DbError> {
        if !wallet.validate() {
            return Err(DbError::InvalidWalletCategory);
        }

        self.write(|conn| {
            conn.execute(
                "INSERT INTO crypto_wallets (id, name, category, icon) VALUES (?1, ?2, ?3, ?4)",
                params![&wallet.id, &wallet.name, &wallet.category, &wallet.icon],
            )?;
            Ok(())
        })
    }

    /// Gets all wallets
    pub fn get_wallets(&self) -> Result<Vec<CryptoWallet>, DbError> {
        self.read(|conn| {
            let mut stmt = conn
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
        })
    }

    /// Gets a single wallet by ID
    pub fn get_wallet(&self, id: &str) -> Result<Option<CryptoWallet>, DbError> {
        self.read(|conn| {
            let result = conn.query_row(
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
        })
    }

    /// Updates a wallet
    pub fn update_wallet(&self, wallet: &CryptoWallet) -> Result<(), DbError> {
        if !wallet.validate() {
            return Err(DbError::InvalidWalletCategory);
        }

        self.write(|conn| {
            let rows = conn.execute(
                "UPDATE crypto_wallets SET name = ?2, category = ?3, icon = ?4 WHERE id = ?1",
                params![&wallet.id, &wallet.name, &wallet.category, &wallet.icon],
            )?;

            if rows == 0 {
                return Err(DbError::WalletNotFound);
            }

            Ok(())
        })
    }

    /// Counts transactions for a wallet on a given connection.
    fn wallet_transaction_count_on(conn: &Connection, wallet_id: &str) -> Result<i64, DbError> {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM crypto_transactions WHERE wallet_id = ?1",
                params![wallet_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count)
    }

    /// Returns the number of transactions associated with a wallet
    pub fn get_wallet_transaction_count(&self, wallet_id: &str) -> Result<i64, DbError> {
        self.read(|conn| Self::wallet_transaction_count_on(conn, wallet_id))
    }

    /// Deletes a wallet.
    /// When `force` is false, blocks deletion if the wallet has transactions.
    /// When `force` is true, deletes regardless (CASCADE removes transactions).
    pub fn delete_wallet(&self, id: &str, force: bool) -> Result<(), DbError> {
        self.write(|conn| {
            if !force {
                let tx_count = Self::wallet_transaction_count_on(conn, id)?;
                if tx_count > 0 {
                    return Err(DbError::WalletNotEmpty);
                }
            }

            let rows = conn.execute("DELETE FROM crypto_wallets WHERE id = ?1", params![id])?;

            if rows == 0 {
                return Err(DbError::WalletNotFound);
            }

            Ok(())
        })
    }
}
