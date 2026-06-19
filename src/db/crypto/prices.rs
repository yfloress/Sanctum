// Sanctum — a privacy-first personal finance and crypto vault.
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

//! Crypto price cache database operations
//!
//! Exchange rates, crypto prices, and portfolio snapshots.

use crate::db::{Database, DbError};
use chrono::Utc;
use rusqlite::params;

impl Database {
    // ==================== Exchange Rate Cache ====================

    /// Saves an exchange rate to cache (e.g., CLP_USD)
    pub fn save_exchange_rate(&self, pair: &str, rate: f64) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.write(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO exchange_rate_cache (currency_pair, rate, updated_at)
                 VALUES (?1, ?2, ?3)",
                params![pair, rate, now],
            )?;
            Ok(())
        })
    }

    /// Loads a cached exchange rate
    pub fn load_exchange_rate(&self, pair: &str) -> Result<Option<(f64, String)>, DbError> {
        self.read(|conn| {
            let result = conn.query_row(
                "SELECT rate, updated_at FROM exchange_rate_cache WHERE currency_pair = ?1",
                params![pair],
                |row| Ok((row.get::<_, f64>(0)?, row.get::<_, String>(1)?)),
            );

            match result {
                Ok((rate, updated_at)) => Ok(Some((rate, updated_at))),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DbError::Sqlite(e)),
            }
        })
    }

    // ==================== Crypto Price Cache ====================

    /// Saves a crypto price to cache
    pub fn save_crypto_price(
        &self,
        coin_id: &str,
        symbol: &str,
        name: &str,
        price_usd: f64,
        price_change_24h: f64,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.write(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO crypto_price_cache
                 (coin_id, symbol, name, price_usd, price_change_24h, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![coin_id, symbol, name, price_usd, price_change_24h, now],
            )?;
            Ok(())
        })
    }

    /// Loads all cached crypto prices
    pub fn load_crypto_prices(
        &self,
    ) -> Result<Vec<(String, String, String, f64, f64, String)>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT coin_id, symbol, name, price_usd, price_change_24h, updated_at
                 FROM crypto_price_cache",
            )?;

            let prices = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })?;

            prices
                .collect::<Result<Vec<_>, _>>()
                .map_err(DbError::Sqlite)
        })
    }

    /// Loads a specific cached crypto price
    pub fn load_crypto_price(&self, coin_id: &str) -> Result<Option<(f64, String)>, DbError> {
        self.read(|conn| {
            let result = conn.query_row(
                "SELECT price_usd, updated_at FROM crypto_price_cache WHERE coin_id = ?1",
                params![coin_id],
                |row| Ok((row.get::<_, f64>(0)?, row.get::<_, String>(1)?)),
            );

            match result {
                Ok((price, updated_at)) => Ok(Some((price, updated_at))),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DbError::Sqlite(e)),
            }
        })
    }

    // ==================== Portfolio Snapshots ====================

    /// Saves a daily crypto portfolio snapshot (upsert by date)
    pub fn save_crypto_portfolio_snapshot(
        &self,
        snapshot_date: &str,
        total_value_usd: f64,
        total_cost_usd: f64,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.write(|conn| {
            conn.execute(
                "INSERT INTO crypto_portfolio_snapshots
                 (snapshot_date, total_value_usd, total_cost_usd, created_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(snapshot_date) DO UPDATE SET
                    total_value_usd = ?2,
                    total_cost_usd = ?3,
                    created_at = ?4",
                params![snapshot_date, total_value_usd, total_cost_usd, now],
            )?;
            Ok(())
        })
    }

    /// Loads crypto portfolio snapshots from a starting date (inclusive)
    pub fn load_crypto_portfolio_snapshots(
        &self,
        start_date: &str,
    ) -> Result<Vec<(String, f64, f64)>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT snapshot_date, total_value_usd, total_cost_usd
                 FROM crypto_portfolio_snapshots
                 WHERE snapshot_date >= ?1
                 ORDER BY snapshot_date ASC",
            )?;

            let snapshots = stmt.query_map(params![start_date], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            })?;

            snapshots
                .collect::<Result<Vec<_>, _>>()
                .map_err(DbError::Sqlite)
        })
    }
}
