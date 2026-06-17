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

//! Crypto portfolio aggregation database operations
//!
//! Balance calculations, cost basis tracking, and portfolio aggregation.

use crate::db::{Database, DbError};
use crate::features::crypto::tax::types::derive_mechanical_type;
use crate::models::{AggregatedAsset, CryptoTransaction, CryptoTransactionType};
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const MIN_VISIBLE_ASSET_AMOUNT: f64 = 1e-12;

static DEFAULT_SYMBOL_BY_COIN_ID: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    crate::features::crypto::api::default_coin_catalog()
        .into_iter()
        .map(|coin| (coin.id.to_lowercase(), coin.symbol))
        .collect()
});

fn canonical_symbol_for_coin(coin_id: &str, fallback_symbol: &str) -> String {
    DEFAULT_SYMBOL_BY_COIN_ID
        .get(&coin_id.to_lowercase())
        .cloned()
        .unwrap_or_else(|| fallback_symbol.to_string())
}

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
                "SELECT coin_id, type, amount, fee_coin_id, fee_amount, subtype
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
                    row.get::<_, Option<String>>(5)?,
                ))
            })?;
            for row in rows {
                let (row_coin_id, tx_type, amount, fee_coin_id, fee_amount, subtype) = row?;
                if row_coin_id == coin_id {
                    let mech = derive_mechanical_type(&tx_type, subtype.as_deref());
                    if let Ok(kind) = mech.parse::<CryptoTransactionType>() {
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
                "SELECT coin_id, type, amount, fee_coin_id, fee_amount, subtype
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
                    row.get::<_, Option<String>>(5)?,
                ))
            })?;
            for row in rows {
                let (row_coin_id, tx_type, amount, fee_coin_id, fee_amount, subtype) = row?;
                if row_coin_id == coin_id {
                    let mech = derive_mechanical_type(&tx_type, subtype.as_deref());
                    if let Ok(kind) = mech.parse::<CryptoTransactionType>() {
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
                    let tx_mech = tx.mechanical_type();
                    let counter_mech = counter.mechanical_type();
                    let is_transfer_pair = (tx_mech == "transfer_out"
                        && counter_mech == "transfer_in")
                        || (tx_mech == "transfer_in" && counter_mech == "transfer_out");
                    let is_swap_pair = tx_mech == "swap" && counter_mech == "swap";

                    if is_transfer_pair {
                        if processed.contains(rel_id) || processed.contains(&tx.id) {
                            continue;
                        }

                        let applied = if tx_mech == "transfer_out" {
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

                        let (source, target) =
                            Self::resolve_swap_pair_direction(&assets, &tx, counter);
                        Self::apply_swap_pair(&mut assets, source, target);
                        continue;
                    }
                }
            }

            let tx_type = match tx.mechanical_type().parse::<CryptoTransactionType>() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let entry = assets.entry(tx.coin_id.clone()).or_insert_with(|| {
                AggregatedAsset::new(
                    tx.coin_id.clone(),
                    canonical_symbol_for_coin(&tx.coin_id, &tx.symbol),
                )
            });

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
        let transactions = self.get_all_crypto_transactions(0, i64::MAX)?;
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
        let source_entry = assets.entry(source.coin_id.clone()).or_insert_with(|| {
            AggregatedAsset::new(
                source.coin_id.clone(),
                canonical_symbol_for_coin(&source.coin_id, &source.symbol),
            )
        });

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
        let target_entry = assets.entry(target.coin_id.clone()).or_insert_with(|| {
            AggregatedAsset::new(
                target.coin_id.clone(),
                canonical_symbol_for_coin(&target.coin_id, &target.symbol),
            )
        });
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

    /// Resolves swap direction for portfolio aggregation.
    /// Prefers the side with available balance, then type signal scoring,
    /// and finally deterministic ID fallback.
    fn resolve_swap_pair_direction<'a>(
        assets: &HashMap<String, AggregatedAsset>,
        a: &'a CryptoTransaction,
        b: &'a CryptoTransaction,
    ) -> (&'a CryptoTransaction, &'a CryptoTransaction) {
        let eps = 1e-8_f64;
        let a_balance = assets
            .get(&a.coin_id)
            .map(|x| x.total_amount)
            .unwrap_or(0.0);
        let b_balance = assets
            .get(&b.coin_id)
            .map(|x| x.total_amount)
            .unwrap_or(0.0);
        let a_can_outflow = a.amount > 0.0 && a_balance + eps >= a.amount;
        let b_can_outflow = b.amount > 0.0 && b_balance + eps >= b.amount;

        // 1) Strict balance: only one side can cover the full outflow
        if a_can_outflow && !b_can_outflow {
            return (a, b);
        }
        if b_can_outflow && !a_can_outflow {
            return (b, a);
        }

        // 2) Field-based scoring (fees, price, overrides)
        let source_score = |tx: &CryptoTransaction| -> i32 {
            let mut score = 0;
            if tx.override_proceeds.is_some() {
                score += 8;
            }
            if tx.override_cost_basis.is_some() {
                score -= 8;
            }
            if tx.fee_coin_id.is_some() || tx.fee_amount.is_some() {
                score += 4;
            }
            if tx.fee.is_some() {
                score += 2;
            }
            if tx.price_per_coin.is_some() {
                score += 1;
            }
            score
        };

        let a_score = source_score(a);
        let b_score = source_score(b);
        if a_score > b_score {
            return (a, b);
        }
        if b_score > a_score {
            return (b, a);
        }

        // 3) Soft balance: one side has *some* balance, the other has none.
        //    The side with balance is more likely the source (you sell what
        //    you hold), even if the balance doesn't fully cover the amount.
        let a_has_balance = a_balance > eps;
        let b_has_balance = b_balance > eps;
        if a_has_balance && !b_has_balance {
            return (a, b);
        }
        if b_has_balance && !a_has_balance {
            return (b, a);
        }

        // 4) Partial-coverage ratio: when both sides have balance, the one
        //    whose balance covers a larger fraction of the swap amount is a
        //    stronger source candidate (more of it is available to spend).
        if a_has_balance && b_has_balance && a.amount > eps && b.amount > eps {
            let a_ratio = (a_balance / a.amount).min(1.0);
            let b_ratio = (b_balance / b.amount).min(1.0);
            let ratio_diff = (a_ratio - b_ratio).abs();
            if ratio_diff > 0.05 {
                return if a_ratio > b_ratio { (a, b) } else { (b, a) };
            }
        }

        // 5) Deterministic ID fallback -- harmless, just noisy during dev.
        log::debug!(
            "Swap direction fallback by id (a={}, b={}, a_score={}, b_score={}, a_balance={:.8}, b_balance={:.8})",
            a.id,
            b.id,
            a_score,
            b_score,
            a_balance,
            b_balance
        );
        if a.id < b.id { (a, b) } else { (b, a) }
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

        let fallback_symbol = fee_symbol
            .map(|s| s.to_uppercase())
            .or_else(|| {
                DEFAULT_SYMBOL_BY_COIN_ID
                    .get(&fee_coin_id.to_lowercase())
                    .cloned()
            })
            .unwrap_or_else(|| fee_coin_id.to_uppercase());

        let entry = assets
            .entry(fee_coin_id.to_string())
            .or_insert_with(|| AggregatedAsset::new(fee_coin_id.to_string(), fallback_symbol));

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

        let entry = assets.entry(source.coin_id.clone()).or_insert_with(|| {
            AggregatedAsset::new(
                source.coin_id.clone(),
                canonical_symbol_for_coin(&source.coin_id, &source.symbol),
            )
        });

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

    pub(crate) fn aggregate_crypto_transactions(
        mut transactions: Vec<CryptoTransaction>,
    ) -> Vec<AggregatedAsset> {
        if transactions.is_empty() {
            return Vec::new();
        }

        // Process transactions chronologically to keep cost basis adjustments consistent
        // When dates are equal, process inflows (buy, transfer_in) before outflows (sell, transfer_out, swap)
        // This ensures correct balance calculation when buy and sell happen on the same day
        fn tx_type_order(mech: &str) -> u8 {
            match mech {
                "buy" => 0,
                "transfer_in" => 1,
                "sell" => 2,
                "transfer_out" => 3,
                "swap" => 4,
                _ => 5,
            }
        }
        transactions.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| {
                    tx_type_order(a.mechanical_type()).cmp(&tx_type_order(b.mechanical_type()))
                })
                .then_with(|| a.id.cmp(&b.id))
        });

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
                    let tx_mech = tx.mechanical_type();
                    let counter_mech = counter.mechanical_type();
                    let is_transfer_pair = (tx_mech == "transfer_out"
                        && counter_mech == "transfer_in")
                        || (tx_mech == "transfer_in" && counter_mech == "transfer_out");
                    let is_swap_pair = tx_mech == "swap" && counter_mech == "swap";

                    if is_transfer_pair {
                        if processed.contains(rel_id) || processed.contains(&tx.id) {
                            continue;
                        }

                        let applied = if tx_mech == "transfer_out" {
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

                        let (source, target) =
                            Self::resolve_swap_pair_direction(&assets, &tx, counter);
                        Self::apply_swap_pair(&mut assets, source, target);
                        continue;
                    }
                }
            }

            let tx_type = match tx.mechanical_type().parse::<CryptoTransactionType>() {
                Ok(t) => t,
                Err(_) => continue, // Skip invalid transaction types
            };

            let entry = assets.entry(tx.coin_id.clone()).or_insert_with(|| {
                AggregatedAsset::new(
                    tx.coin_id.clone(),
                    canonical_symbol_for_coin(&tx.coin_id, &tx.symbol),
                )
            });

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
                // Keep genuine dust balances while still hiding pure
                // floating-point residue.
                if asset.total_amount > MIN_VISIBLE_ASSET_AMOUNT {
                    asset.calculate_avg_price();
                    Some(asset)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
