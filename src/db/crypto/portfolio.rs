//! Crypto portfolio aggregation database operations
//!
//! Balance calculations, cost basis tracking, and portfolio aggregation.

use crate::db::{Database, DbError};
use crate::models::{AggregatedAsset, CryptoTransaction, CryptoTransactionType};
use rusqlite::params;
use std::collections::{HashMap, HashSet};

impl Database {
    // ==================== Balance Calculations ====================

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

    // ==================== Portfolio Aggregation ====================

    /// Calculates aggregated portfolio from all transactions across all wallets
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

    // ==================== Internal Helpers ====================

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
}
