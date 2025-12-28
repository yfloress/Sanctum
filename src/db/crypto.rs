//! Crypto database operations
//!
//! CRUD operations for crypto wallets, transactions, prices, and portfolio aggregation.

use super::{Database, DbError};
use crate::models::{AggregatedAsset, CryptoTransaction, CryptoTransactionType, CryptoWallet};
use chrono::Utc;
use rusqlite::{params, Error as RusqliteError};
use std::collections::{HashMap, HashSet};

impl Database {
    // ==================== Price Cache Functions ====================

    /// Saves an exchange rate to cache (e.g., CLP_USD)
    pub fn save_exchange_rate(&self, pair: &str, rate: f64) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO exchange_rate_cache (currency_pair, rate, updated_at)
             VALUES (?1, ?2, ?3)",
            params![pair, rate, now],
        )?;
        Ok(())
    }

    /// Loads a cached exchange rate
    pub fn load_exchange_rate(&self, pair: &str) -> Result<Option<(f64, String)>, DbError> {
        let result = self.conn.query_row(
            "SELECT rate, updated_at FROM exchange_rate_cache WHERE currency_pair = ?1",
            params![pair],
            |row| Ok((row.get::<_, f64>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Ok((rate, updated_at)) => Ok(Some((rate, updated_at))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

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
        self.conn.execute(
            "INSERT OR REPLACE INTO crypto_price_cache
             (coin_id, symbol, name, price_usd, price_change_24h, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![coin_id, symbol, name, price_usd, price_change_24h, now],
        )?;
        Ok(())
    }

    /// Loads all cached crypto prices
    pub fn load_crypto_prices(
        &self,
    ) -> Result<Vec<(String, String, String, f64, f64, String)>, DbError> {
        let mut stmt = self.conn.prepare(
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
    }

    /// Loads a specific cached crypto price
    pub fn load_crypto_price(&self, coin_id: &str) -> Result<Option<(f64, String)>, DbError> {
        let result = self.conn.query_row(
            "SELECT price_usd, updated_at FROM crypto_price_cache WHERE coin_id = ?1",
            params![coin_id],
            |row| Ok((row.get::<_, f64>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Ok((price, updated_at)) => Ok(Some((price, updated_at))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Saves a daily crypto portfolio snapshot (upsert by date)
    pub fn save_crypto_portfolio_snapshot(
        &self,
        snapshot_date: &str,
        total_value_usd: f64,
        total_cost_usd: f64,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
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
    }

    /// Loads crypto portfolio snapshots from a starting date (inclusive)
    pub fn load_crypto_portfolio_snapshots(
        &self,
        start_date: &str,
    ) -> Result<Vec<(String, f64, f64)>, DbError> {
        let mut stmt = self.conn.prepare(
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
    }

    // ==================== Crypto Wallets CRUD ====================

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

    /// Deletes a wallet and all its transactions
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

    // ==================== Crypto Transactions CRUD ====================

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

    /// Gets the wallet balance for a coin at (or before) a given date.
    pub fn get_wallet_coin_balance_at(
        &self,
        wallet_id: &str,
        coin_id: &str,
        date: &str,
        exclude_tx_id: Option<&str>,
    ) -> Result<f64, DbError> {
        let mut balance = 0.0;
        if let Some(exclude) = exclude_tx_id {
            let mut stmt = self.conn.prepare(
                "SELECT coin_id, type, amount, fee_coin_id, fee_amount
                 FROM crypto_transactions
                 WHERE wallet_id = ?1
                   AND date <= ?2
                   AND id != ?3
                   AND (coin_id = ?4 OR fee_coin_id = ?4)",
            )?;
            let rows = stmt.query_map(params![wallet_id, date, exclude, coin_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<f64>>(4)?,
                ))
            })?;
            for row in rows {
                let (row_coin_id, tx_type, amount, fee_coin_id, fee_amount) = row?;
                if row_coin_id == coin_id {
                    if let Ok(kind) = tx_type.parse::<CryptoTransactionType>() {
                        match kind {
                            CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                                balance += amount
                            }
                            CryptoTransactionType::Sell
                            | CryptoTransactionType::TransferOut
                            | CryptoTransactionType::Swap => balance -= amount,
                        }
                    }
                }

                if let Some(fee_coin_id) = fee_coin_id {
                    if fee_coin_id == coin_id {
                        if let Some(fee_amount) = fee_amount {
                            balance -= fee_amount;
                        }
                    }
                }
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT coin_id, type, amount, fee_coin_id, fee_amount
                 FROM crypto_transactions
                 WHERE wallet_id = ?1
                   AND date <= ?2
                   AND (coin_id = ?3 OR fee_coin_id = ?3)",
            )?;
            let rows = stmt.query_map(params![wallet_id, date, coin_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<f64>>(4)?,
                ))
            })?;
            for row in rows {
                let (row_coin_id, tx_type, amount, fee_coin_id, fee_amount) = row?;
                if row_coin_id == coin_id {
                    if let Ok(kind) = tx_type.parse::<CryptoTransactionType>() {
                        match kind {
                            CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                                balance += amount
                            }
                            CryptoTransactionType::Sell
                            | CryptoTransactionType::TransferOut
                            | CryptoTransactionType::Swap => balance -= amount,
                        }
                    }
                }

                if let Some(fee_coin_id) = fee_coin_id {
                    if fee_coin_id == coin_id {
                        if let Some(fee_amount) = fee_amount {
                            balance -= fee_amount;
                        }
                    }
                }
            }
        }

        Ok(balance)
    }

    /// Gets the wallet balance and cost basis for a coin at (or before) a given date.
    pub fn get_wallet_coin_state_at(
        &self,
        wallet_id: &str,
        coin_id: &str,
        date: &str,
    ) -> Result<(f64, f64), DbError> {
        let mut transactions = self.get_wallet_transactions_up_to_date(wallet_id, date)?;

        transactions.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));

        let tx_map: HashMap<String, CryptoTransaction> = transactions
            .iter()
            .cloned()
            .map(|tx| (tx.id.clone(), tx))
            .collect();
        let mut processed: HashSet<String> = HashSet::new();

        let mut assets: HashMap<String, AggregatedAsset> = HashMap::new();

        for tx in transactions {
            if processed.contains(&tx.id) {
                continue;
            }

            if let Some(rel_id) = &tx.related_tx_id {
                if let Some(counter) = tx_map.get(rel_id) {
                    let is_transfer_pair = (tx.transaction_type == "transfer_out"
                        && counter.transaction_type == "transfer_in")
                        || (tx.transaction_type == "transfer_in"
                            && counter.transaction_type == "transfer_out");
                    let is_swap_pair = (tx.transaction_type == "swap"
                        && counter.transaction_type == "transfer_in")
                        || (tx.transaction_type == "transfer_in"
                            && counter.transaction_type == "swap");

                    if is_transfer_pair {
                        if processed.contains(rel_id) || processed.contains(&tx.id) {
                            continue;
                        }

                        let applied = if tx.transaction_type == "transfer_out" {
                            Self::apply_transfer_pair(&mut assets, &tx, counter)
                        } else {
                            Self::apply_transfer_pair(&mut assets, counter, &tx)
                        };

                        if applied {
                            processed.insert(tx.id.clone());
                            processed.insert(rel_id.clone());
                            continue;
                        }
                    }

                    if is_swap_pair {
                        if processed.contains(rel_id) || processed.contains(&tx.id) {
                            continue;
                        }

                        processed.insert(tx.id.clone());
                        processed.insert(rel_id.clone());

                        if tx.transaction_type == "swap" {
                            Self::apply_swap_pair(&mut assets, &tx, counter);
                        } else {
                            Self::apply_swap_pair(&mut assets, counter, &tx);
                        }
                        continue;
                    }
                }
            }

            let tx_type = match tx.transaction_type.parse::<CryptoTransactionType>() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let entry = assets
                .entry(tx.coin_id.clone())
                .or_insert_with(|| AggregatedAsset::new(tx.coin_id.clone(), tx.symbol.clone()));

            if matches!(tx_type, CryptoTransactionType::Buy) {
                entry.total_amount += tx.amount;
                let cost = tx.amount * tx.price_per_coin.unwrap_or(0.0);
                entry.total_cost_basis += cost + tx.fee.unwrap_or(0.0);
            } else if matches!(tx_type, CryptoTransactionType::TransferIn) {
                entry.total_amount += tx.amount;
                if let Some(price) = tx.price_per_coin {
                    let fee = tx.fee.unwrap_or(0.0);
                    entry.total_cost_basis += (tx.amount * price) + fee;
                }
            } else if tx_type.is_outflow() || matches!(tx_type, CryptoTransactionType::Swap) {
                let prev_amount = entry.total_amount;
                entry.total_amount -= tx.amount;
                if entry.total_amount < 0.0 {
                    entry.total_amount = 0.0;
                }
                if prev_amount > 0.0 {
                    let ratio = (tx.amount / prev_amount).min(1.0);
                    entry.total_cost_basis *= 1.0 - ratio;
                    entry.total_cost_basis = entry.total_cost_basis.max(0.0);
                }
            }

            if let (Some(fee_coin_id), Some(fee_amount)) =
                (tx.fee_coin_id.as_deref(), tx.fee_amount)
            {
                let fee_symbol = if fee_coin_id == tx.coin_id {
                    Some(tx.symbol.as_str())
                } else {
                    None
                };
                Self::apply_fee_coin_outflow(&mut assets, fee_coin_id, fee_amount, fee_symbol);
            }
        }

        if let Some(asset) = assets.get(coin_id) {
            return Ok((asset.total_amount, asset.total_cost_basis));
        }

        Ok((0.0, 0.0))
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

    // ==================== Portfolio Aggregation ====================

    /// Applies a swap pair atomically to source and target assets to preserve cost basis
    fn apply_swap_pair(
        assets: &mut HashMap<String, AggregatedAsset>,
        source: &CryptoTransaction,
        target: &CryptoTransaction,
    ) {
        let source_entry = assets
            .entry(source.coin_id.clone())
            .or_insert_with(|| AggregatedAsset::new(source.coin_id.clone(), source.symbol.clone()));

        // Capture state before the swap to compute proportional cost
        let prev_amount = source_entry.total_amount;
        let prev_cost = source_entry.total_cost_basis;

        // Compute cost transferred using proportional cost basis of the source asset
        let proportion = if prev_amount > 0.0 {
            (source.amount / prev_amount).min(1.0)
        } else {
            0.0
        };
        let cost_transferred = prev_cost * proportion + source.fee.unwrap_or(0.0);

        // Apply outflow on source asset
        source_entry.total_amount -= source.amount;
        if source_entry.total_amount < 0.0 {
            source_entry.total_amount = 0.0;
        }
        source_entry.total_cost_basis = (source_entry.total_cost_basis - cost_transferred).max(0.0);

        // Apply inflow on target asset
        let target_entry = assets
            .entry(target.coin_id.clone())
            .or_insert_with(|| AggregatedAsset::new(target.coin_id.clone(), target.symbol.clone()));
        target_entry.total_amount += target.amount;
        target_entry.total_cost_basis += cost_transferred.max(0.0);

        if let (Some(fee_coin_id), Some(fee_amount)) =
            (source.fee_coin_id.as_deref(), source.fee_amount)
        {
            let fee_symbol = if fee_coin_id == source.coin_id {
                Some(source.symbol.as_str())
            } else {
                None
            };
            Self::apply_fee_coin_outflow(assets, fee_coin_id, fee_amount, fee_symbol);
        }

        if let (Some(fee_coin_id), Some(fee_amount)) =
            (target.fee_coin_id.as_deref(), target.fee_amount)
        {
            let fee_symbol = if fee_coin_id == target.coin_id {
                Some(target.symbol.as_str())
            } else {
                None
            };
            Self::apply_fee_coin_outflow(assets, fee_coin_id, fee_amount, fee_symbol);
        }
    }

    fn apply_fee_coin_outflow(
        assets: &mut HashMap<String, AggregatedAsset>,
        fee_coin_id: &str,
        fee_amount: f64,
        fee_symbol: Option<&str>,
    ) {
        if fee_amount <= 0.0 {
            return;
        }

        let entry = assets.entry(fee_coin_id.to_string()).or_insert_with(|| {
            AggregatedAsset::new(
                fee_coin_id.to_string(),
                fee_symbol.unwrap_or(fee_coin_id).to_uppercase(),
            )
        });

        let prev_amount = entry.total_amount;
        entry.total_amount -= fee_amount;
        if entry.total_amount < 0.0 {
            entry.total_amount = 0.0;
        }

        if prev_amount > 0.0 {
            let ratio = (fee_amount / prev_amount).min(1.0);
            entry.total_cost_basis *= 1.0 - ratio;
            entry.total_cost_basis = entry.total_cost_basis.max(0.0);
        }
    }

    /// Applies a transfer pair for the same asset, reducing cost basis only for fee losses
    fn apply_transfer_pair(
        assets: &mut HashMap<String, AggregatedAsset>,
        source: &CryptoTransaction,
        target: &CryptoTransaction,
    ) -> bool {
        if source.coin_id != target.coin_id {
            return false;
        }

        let entry = assets
            .entry(source.coin_id.clone())
            .or_insert_with(|| AggregatedAsset::new(source.coin_id.clone(), source.symbol.clone()));

        let prev_amount = entry.total_amount;
        let prev_cost = entry.total_cost_basis;
        if prev_amount <= 0.0 {
            entry.total_amount = (entry.total_amount - source.amount).max(0.0) + target.amount;
            entry.total_cost_basis += target.fee.unwrap_or(0.0);
            return true;
        }

        let unit_cost = prev_cost / prev_amount;
        let cost_out = unit_cost * source.amount;

        entry.total_amount = (entry.total_amount - source.amount).max(0.0);
        entry.total_cost_basis = (entry.total_cost_basis - cost_out).max(0.0);

        entry.total_amount += target.amount;
        entry.total_cost_basis += unit_cost * target.amount;
        entry.total_cost_basis += target.fee.unwrap_or(0.0);

        if let (Some(fee_coin_id), Some(fee_amount)) =
            (source.fee_coin_id.as_deref(), source.fee_amount)
        {
            let fee_symbol = if fee_coin_id == source.coin_id {
                Some(source.symbol.as_str())
            } else {
                None
            };
            Self::apply_fee_coin_outflow(assets, fee_coin_id, fee_amount, fee_symbol);
        }

        if let (Some(fee_coin_id), Some(fee_amount)) =
            (target.fee_coin_id.as_deref(), target.fee_amount)
        {
            let fee_symbol = if fee_coin_id == target.coin_id {
                Some(target.symbol.as_str())
            } else {
                None
            };
            Self::apply_fee_coin_outflow(assets, fee_coin_id, fee_amount, fee_symbol);
        }
        true
    }

    fn aggregate_crypto_transactions(
        mut transactions: Vec<CryptoTransaction>,
    ) -> Vec<AggregatedAsset> {
        if transactions.is_empty() {
            return Vec::new();
        }

        // Process transactions chronologically to keep cost basis adjustments consistent
        transactions.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));

        let tx_map: HashMap<String, CryptoTransaction> = transactions
            .iter()
            .cloned()
            .map(|tx| (tx.id.clone(), tx))
            .collect();

        let mut processed: HashSet<String> = HashSet::new();
        let mut assets: HashMap<String, AggregatedAsset> = HashMap::new();

        for tx in transactions {
            if processed.contains(&tx.id) {
                continue;
            }

            // Handle swap/transfer pairs to carry over cost basis
            if let Some(rel_id) = &tx.related_tx_id {
                if let Some(counter) = tx_map.get(rel_id) {
                    let is_transfer_pair = (tx.transaction_type == "transfer_out"
                        && counter.transaction_type == "transfer_in")
                        || (tx.transaction_type == "transfer_in"
                            && counter.transaction_type == "transfer_out");
                    let is_swap_pair = (tx.transaction_type == "swap"
                        && counter.transaction_type == "transfer_in")
                        || (tx.transaction_type == "transfer_in"
                            && counter.transaction_type == "swap");

                    if is_transfer_pair {
                        if processed.contains(rel_id) || processed.contains(&tx.id) {
                            continue;
                        }

                        let applied = if tx.transaction_type == "transfer_out" {
                            Self::apply_transfer_pair(&mut assets, &tx, counter)
                        } else {
                            Self::apply_transfer_pair(&mut assets, counter, &tx)
                        };

                        if applied {
                            processed.insert(tx.id.clone());
                            processed.insert(rel_id.clone());
                            continue;
                        }
                    }

                    if is_swap_pair {
                        if processed.contains(rel_id) || processed.contains(&tx.id) {
                            continue;
                        }

                        processed.insert(tx.id.clone());
                        processed.insert(rel_id.clone());

                        // Determine which side is the source (swap out) and target (swap in)
                        if tx.transaction_type == "swap" {
                            Self::apply_swap_pair(&mut assets, &tx, counter);
                        } else {
                            Self::apply_swap_pair(&mut assets, counter, &tx);
                        }
                        continue;
                    }
                }
            }

            let tx_type = match tx.transaction_type.parse::<CryptoTransactionType>() {
                Ok(t) => t,
                Err(_) => continue, // Skip invalid transaction types
            };

            let entry = assets
                .entry(tx.coin_id.clone())
                .or_insert_with(|| AggregatedAsset::new(tx.coin_id.clone(), tx.symbol.clone()));

            match tx_type {
                CryptoTransactionType::Buy => {
                    entry.total_amount += tx.amount;
                    let cost = tx.amount * tx.price_per_coin.unwrap_or(0.0);
                    let fee = tx.fee.unwrap_or(0.0);
                    entry.total_cost_basis += cost + fee;
                }
                CryptoTransactionType::TransferIn => {
                    entry.total_amount += tx.amount;
                    if let Some(price) = tx.price_per_coin {
                        let fee = tx.fee.unwrap_or(0.0);
                        entry.total_cost_basis += (tx.amount * price) + fee;
                    }
                }
                CryptoTransactionType::Sell
                | CryptoTransactionType::TransferOut
                | CryptoTransactionType::Swap => {
                    let prev_amount = entry.total_amount;
                    entry.total_amount -= tx.amount;
                    if entry.total_amount < 0.0 {
                        entry.total_amount = 0.0;
                    }
                    if prev_amount > 0.0 {
                        let ratio = (tx.amount / prev_amount).min(1.0);
                        entry.total_cost_basis *= 1.0 - ratio;
                        entry.total_cost_basis = entry.total_cost_basis.max(0.0);
                    }
                }
            }

            if let (Some(fee_coin_id), Some(fee_amount)) =
                (tx.fee_coin_id.as_deref(), tx.fee_amount)
            {
                let fee_symbol = if fee_coin_id == tx.coin_id {
                    Some(tx.symbol.as_str())
                } else {
                    None
                };
                Self::apply_fee_coin_outflow(&mut assets, fee_coin_id, fee_amount, fee_symbol);
            }
        }

        assets
            .into_values()
            .filter_map(|mut asset| {
                if asset.total_amount > 0.0001 {
                    asset.calculate_avg_price();
                    Some(asset)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Calculates aggregated portfolio from all transactions across all wallets
    /// This is the CRITICAL function that computes total holdings per coin
    pub fn get_aggregated_portfolio(&self) -> Result<Vec<AggregatedAsset>, DbError> {
        let transactions = self.get_all_crypto_transactions()?;
        Ok(Self::aggregate_crypto_transactions(transactions))
    }

    /// Gets aggregated holdings for a specific wallet
    pub fn get_wallet_aggregated_holdings(
        &self,
        wallet_id: &str,
    ) -> Result<Vec<AggregatedAsset>, DbError> {
        let transactions = self.get_wallet_transactions(wallet_id)?;
        Ok(Self::aggregate_crypto_transactions(transactions))
    }
}
