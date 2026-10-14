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
//! Portfolio history reconstructed from transactions.
//!
//! Snapshots are normally taken once a day while the app is open, so imported
//! history has none. This rebuilds the missing days from the prices recorded in
//! the transactions themselves, without any network call.

use crate::db::Database;
use crate::models::CryptoTransaction;
use std::collections::{HashMap, HashSet};

/// Returns `(date, value_usd, cost_usd)` for every past day that has
/// transactions but no snapshot in `existing`.
///
/// Holdings come from the same aggregation the live portfolio uses. Each coin is
/// valued at the last price recorded on or before that day; a coin never priced
/// counts as zero.
pub(crate) fn missing_snapshots(
    mut transactions: Vec<CryptoTransaction>,
    existing: &HashSet<String>,
    today: &str,
) -> Vec<(String, f64, f64)> {
    transactions.retain(|tx| day(tx).is_some());
    transactions.sort_by(|a, b| a.date.cmp(&b.date));

    let mut dates: Vec<&str> = transactions
        .iter()
        .filter_map(day)
        .filter(|d| *d < today && !existing.contains(*d))
        .collect();
    dates.dedup();

    let mut snapshots = Vec::with_capacity(dates.len());
    for date in dates {
        let upto: Vec<CryptoTransaction> = transactions
            .iter()
            .take_while(|tx| day(tx).is_some_and(|d| d <= date))
            .cloned()
            .collect();

        let mut prices: HashMap<&str, f64> = HashMap::new();
        for tx in transactions.iter().take(upto.len()) {
            if let Some(price) = tx.price_per_coin.filter(|p| *p > 0.0) {
                prices.insert(tx.coin_id.as_str(), price);
            }
        }

        let assets = Database::aggregate_crypto_transactions(upto);
        let value = assets
            .iter()
            .map(|a| a.total_amount * prices.get(a.coin_id.as_str()).copied().unwrap_or(0.0))
            .sum();
        let cost = assets.iter().map(|a| a.total_cost_basis).sum();
        snapshots.push((date.to_string(), value, cost));
    }
    snapshots
}

fn day(tx: &CryptoTransaction) -> Option<&str> {
    tx.date.get(..10)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(
        id: &str,
        coin: &str,
        kind: &str,
        amount: f64,
        price: f64,
        date: &str,
    ) -> CryptoTransaction {
        let (tx_type, subtype) = match kind {
            "sell" => ("trade", "sell"),
            "transfer_in" => ("transfer", "deposit"),
            "transfer_out" => ("transfer", "withdrawal"),
            _ => ("trade", "buy"),
        };
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet".to_string(),
            coin.to_string(),
            coin.to_uppercase(),
            tx_type.to_string(),
            amount,
            Some(price),
            None,
            date.to_string(),
            None,
        );
        tx.subtype = Some(subtype.to_string());
        tx
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn one_snapshot_per_past_day_valued_at_the_latest_price() {
        let txs = vec![
            tx("1", "btc", "buy", 1.0, 100.0, "2026-01-05"),
            tx("2", "btc", "buy", 1.0, 200.0, "2026-02-05"),
            tx("3", "eth", "buy", 10.0, 5.0, "2026-02-05T12:00:00Z"),
        ];
        let out = missing_snapshots(txs, &HashSet::new(), "2026-03-01");

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].0, "2026-01-05");
        assert!(approx(out[0].1, 100.0));
        assert!(approx(out[0].2, 100.0));
        // Both BTC revalued at the newer price, plus the ETH bought that day.
        assert_eq!(out[1].0, "2026-02-05");
        assert!(approx(out[1].1, 2.0 * 200.0 + 10.0 * 5.0));
        assert!(approx(out[1].2, 100.0 + 200.0 + 50.0));
    }

    #[test]
    fn keeps_existing_snapshots_and_skips_today() {
        let txs = vec![
            tx("1", "btc", "buy", 1.0, 100.0, "2026-01-05"),
            tx("2", "btc", "buy", 1.0, 110.0, "2026-01-06"),
            tx("3", "btc", "buy", 1.0, 120.0, "2026-01-07"),
        ];
        let existing = HashSet::from(["2026-01-06".to_string()]);
        let out = missing_snapshots(txs, &existing, "2026-01-07");

        let dates: Vec<&str> = out.iter().map(|s| s.0.as_str()).collect();
        assert_eq!(dates, ["2026-01-05"]);
    }

    #[test]
    fn a_sale_lowers_the_holding() {
        let txs = vec![
            tx("1", "sol", "buy", 4.0, 10.0, "2026-01-05"),
            tx("2", "sol", "sell", 1.0, 20.0, "2026-01-09"),
        ];
        let out = missing_snapshots(txs, &HashSet::new(), "2026-02-01");

        assert!(approx(out[1].1, 3.0 * 20.0));
    }

    #[test]
    fn nothing_to_do_without_transactions() {
        assert!(missing_snapshots(Vec::new(), &HashSet::new(), "2026-01-01").is_empty());
    }
}
