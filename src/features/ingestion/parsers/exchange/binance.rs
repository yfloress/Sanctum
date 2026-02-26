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

//! Binance CSV parsers
//!
//! Supports two export formats:
//!
//! ## All Statements CSV (recommended — covers ALL transaction types)
//!
//! Headers: `User_ID,UTC_Time,Account,Operation,Coin,Change,Remark`
//!
//! This comprehensive export includes every account movement: buys, sells,
//! deposits, withdrawals, distributions, staking rewards, fees, converts, etc.
//!
//! Some operations produce paired rows that must be correlated:
//! - `Binance Convert`: two rows with the same timestamp (one negative, one positive)
//! - `Transaction Spend` + `Transaction Revenue`: paired trade rows
//!
//! ## Spot Trade History CSV (spot trades only)
//!
//! Headers: `Date(UTC),Pair,Side,Price,Executed,Amount,Fee`
//!
//! Simpler format with one row per trade. The `Executed`, `Amount`, and `Fee`
//! fields contain values with currency suffixes (e.g. `"0.5BTC"`, `"25000USDT"`).

use std::collections::{HashMap, HashSet};

use chrono::NaiveDateTime;
use csv::StringRecord;

use super::common::{
    format_datetime, is_fiat, is_quote_currency, normalize_binance_currency,
    is_usd_valued_quote, parse_amount_with_unit, parse_decimal, parse_timestamp,
    should_rename_luna_to_lunc,
};
use super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

// ─── Binance operation classification ───────────────────────────────────────

/// Known operation types from Binance's "Generate All Statements" export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinanceOperation {
    Buy,
    Sell,
    TransactionBuy,
    TransactionSold,
    TransactionSpend,
    TransactionRevenue,
    Fee,
    Deposit,
    Withdraw,
    FiatDeposit,
    FiatWithdrawal,
    Distribution,
    AirdropAssets,
    StakingRewards,
    Convert,
    SmallAssetsExchange,
    Transfer,
    CardCashback,
    CardSpending,
    P2PTrade,
    Unknown,
}

impl BinanceOperation {
    fn parse(raw: &str) -> Self {
        match raw.trim() {
            "Buy" => BinanceOperation::Buy,
            "Sell" => BinanceOperation::Sell,
            "Transaction Buy" => BinanceOperation::TransactionBuy,
            "Transaction Sold" => BinanceOperation::TransactionSold,
            "Transaction Spend" => BinanceOperation::TransactionSpend,
            "Transaction Revenue" => BinanceOperation::TransactionRevenue,
            "Transaction Fee" | "Fee" => BinanceOperation::Fee,
            "Deposit" => BinanceOperation::Deposit,
            "Withdraw" => BinanceOperation::Withdraw,
            "Fiat Deposit" => BinanceOperation::FiatDeposit,
            "Fiat Withdrawal" | "Fiat Withdraw" => BinanceOperation::FiatWithdrawal,
            "Distribution" => BinanceOperation::Distribution,
            "Airdrop Assets" => BinanceOperation::AirdropAssets,
            "Staking Rewards"
            | "ETH 2.0 Staking Rewards"
            | "Launchpool Interest"
            | "Savings Interest"
            | "Simple Earn Flexible Interest"
            | "Simple Earn Locked Rewards" => BinanceOperation::StakingRewards,
            "Binance Convert" | "Small Assets Exchange BNB" => {
                // We distinguish these below based on the original raw string
                if raw.trim() == "Small Assets Exchange BNB" {
                    BinanceOperation::SmallAssetsExchange
                } else {
                    BinanceOperation::Convert
                }
            }
            "Transfer Between Main and Funding Wallet"
            | "Transfer Between Main and Margin Wallet"
            | "Transfer Between Spot and Futures"
            | "Main and Funding Account Transfer"
            | "Main and funding account transfer"
            | "Transfer Between Spot Account and UM Futures Account"
            | "Transfer Between Spot Account and USDⓈ-M Futures Account"
            | "Transfer Between Spot Account and CM Futures Account"
            | "Transfer Between Spot Account and COIN-M Futures Account" => {
                BinanceOperation::Transfer
            }
            "Binance Card Cashback" | "Card Cashback" => BinanceOperation::CardCashback,
            "Binance Card Spending" | "Card Spending" => BinanceOperation::CardSpending,
            "P2P Trading" | "C2C Transfer" => BinanceOperation::P2PTrade,
            _ => {
                let trimmed = raw.trim();
                if trimmed.starts_with("Transfer Between") {
                    BinanceOperation::Transfer
                } else {
                    BinanceOperation::Unknown
                }
            }
        }
    }

    /// Returns `true` for operations that should be skipped entirely
    /// (internal transfers, fiat movements, trade-related entries that are
    /// handled separately via the spot trade export).
    fn should_skip(&self) -> bool {
        matches!(
            self,
            BinanceOperation::Transfer
                | BinanceOperation::FiatDeposit
                | BinanceOperation::FiatWithdrawal
        )
    }

    /// Returns `true` for operations that need to be paired with another row
    /// sharing the same timestamp to form a complete trade.
    fn needs_pairing(&self) -> bool {
        matches!(
            self,
            BinanceOperation::Convert | BinanceOperation::SmallAssetsExchange
        )
    }
}

// ─── Parsed row from Binance All Statements ─────────────────────────────────

#[derive(Debug, Clone)]
struct BinanceRow {
    timestamp: NaiveDateTime,
    operation: BinanceOperation,
    /// Raw operation string for notes/debugging.
    operation_raw: String,
    /// Normalised ticker (e.g. `BTC`, `BCH` instead of `BCC`).
    symbol: String,
    /// Signed change: positive = credit, negative = debit.
    change: f64,
    remark: String,
    line_number: usize,
}

/// Normalise a Binance coin name, including the LUNA -> LUNC rename.
fn normalise_coin(raw: &str, timestamp: NaiveDateTime) -> String {
    let base = normalize_binance_currency(raw);
    if base == "LUNA" && should_rename_luna_to_lunc(timestamp) {
        "LUNC".to_string()
    } else {
        base.to_string()
    }
}

fn is_usd_valued_for_tax(symbol: &str) -> bool {
    is_usd_valued_quote(symbol)
}

// ─── Column index helpers ───────────────────────────────────────────────────

fn resolve_all_statements_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"');
        match key {
            "User_ID" => {
                map.insert("user_id", i);
            }
            "UTC_Time" => {
                map.insert("utc_time", i);
            }
            "Account" => {
                map.insert("account", i);
            }
            "Operation" => {
                map.insert("operation", i);
            }
            "Coin" => {
                map.insert("coin", i);
            }
            "Change" => {
                map.insert("change", i);
            }
            "Remark" => {
                map.insert("remark", i);
            }
            _ => {}
        }
    }
    map
}

fn resolve_spot_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"');
        match key {
            "Date(UTC)" => {
                map.insert("date", i);
            }
            "Pair" => {
                map.insert("pair", i);
            }
            "Side" => {
                map.insert("side", i);
            }
            "Price" => {
                map.insert("price", i);
            }
            "Executed" => {
                map.insert("executed", i);
            }
            "Amount" => {
                map.insert("amount", i);
            }
            "Fee" => {
                map.insert("fee", i);
            }
            _ => {}
        }
    }
    map
}

fn get_field<'a>(record: &'a StringRecord, cols: &HashMap<&str, usize>, name: &str) -> &'a str {
    cols.get(name)
        .and_then(|&i| record.get(i))
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or("")
}

// ─── Convert / SmallAssets pairing key ──────────────────────────────────────

/// Groups paired Binance operations (Convert, SmallAssetsExchange) by timestamp.
/// Within a group, negative entries are outgoing and positive are incoming.
#[derive(Debug, Default)]
struct PendingConvert {
    outgoing: Vec<BinanceRow>,
    incoming: Vec<BinanceRow>,
}

impl PendingConvert {
    fn insert(&mut self, row: BinanceRow) {
        if row.change < 0.0 {
            self.outgoing.push(row);
        } else {
            self.incoming.push(row);
        }
    }

    fn is_complete(&self) -> bool {
        !self.outgoing.is_empty() && !self.incoming.is_empty()
    }

    /// Resolves the pending convert into zero or more `ImportCryptoTransaction`s.
    ///
    /// Binance sometimes logs internal account transfers (Funding → Spot) with
    /// the same "Binance Convert" label and timestamp as the real conversion.
    /// For example, converting USDT to USDC may produce three rows:
    ///   - Funding, USDT, -6.28  (internal: move USDT out of Funding)
    ///   - Spot,    USDT, +6.28  (internal: move USDT into Spot)
    ///   - Spot,    USDC, +6.28  (real: receive converted USDC)
    ///
    /// We first identify symbols that appear on **both** sides (outgoing AND
    /// incoming). Those are internal transfers and should not form the swap
    /// target. After filtering, the remaining rows represent the real conversion.
    fn resolve(self, wallet_name: &str) -> Vec<(usize, ImportCryptoTransaction)> {
        let mut results = Vec::new();

        // ── Step 1: Identify internal-transfer symbols ──
        // A symbol that has entries on both the outgoing AND incoming side
        // within the same Convert group is an internal account movement.
        // Symbols are already normalised (uppercased) by `normalise_coin`,
        // so we compare `&str` slices directly — no extra allocations.
        let out_symbols: HashSet<&str> = self.outgoing.iter().map(|r| r.symbol.as_str()).collect();
        let in_symbols: HashSet<&str> = self.incoming.iter().map(|r| r.symbol.as_str()).collect();
        let internal_symbols: HashSet<&str> =
            out_symbols.intersection(&in_symbols).copied().collect();

        // ── Step 2: Build effective outgoing / incoming ──
        // Remove internal-transfer rows from the incoming side (they are not
        // the real conversion target). For the outgoing side, keep internal
        // rows as a fallback source (the user DID spend that currency).
        let real_outgoing: Vec<&BinanceRow> = self
            .outgoing
            .iter()
            .filter(|r| !internal_symbols.contains(r.symbol.as_str()))
            .collect();
        let real_incoming: Vec<&BinanceRow> = self
            .incoming
            .iter()
            .filter(|r| !internal_symbols.contains(r.symbol.as_str()))
            .collect();

        // Capture flags before ownership moves below.
        let has_real_outgoing = !real_outgoing.is_empty();
        let has_real_incoming = !real_incoming.is_empty();

        // If all outgoing was internal (e.g. USDT appeared on both sides),
        // fall back to the original outgoing rows as the conversion source.
        let mut effective_outgoing: Vec<&BinanceRow> = if has_real_outgoing {
            real_outgoing
        } else {
            self.outgoing.iter().collect()
        };

        // If all incoming was filtered but there ARE real (non-internal)
        // outgoing rows, the filter was over-aggressive. This happens when
        // the target symbol also appears as dust on the outgoing side
        // (e.g. SmallAssetsExchange: BNB -0.0001 dust + BNB +0.15 target).
        // Fall back to the original incoming so those outgoing rows still
        // get paired with a target.
        let mut effective_incoming: Vec<&BinanceRow> = if has_real_incoming {
            real_incoming
        } else if has_real_outgoing {
            // Real outgoing exists but incoming was entirely filtered out.
            self.incoming.iter().collect()
        } else {
            // Both sides fully internal → pure internal transfer, skip.
            return results;
        };

        // Deterministic pairing order for reproducible imports/tests.
        effective_outgoing.sort_by_key(|row| row.line_number);
        effective_incoming.sort_by_key(|row| row.line_number);

        // ── Step 3: Resolve 1:1 pairs ──
        if effective_outgoing.len() == 1 && effective_incoming.len() == 1 {
            let out = effective_outgoing[0];
            let inc = effective_incoming[0];
            if let Some(tx) = resolve_single_pair(out, inc, wallet_name) {
                results.push(tx);
            }

            return results;
        }

        // If we have multiple outgoing and multiple incoming rows in the same
        // group, pair them in source order. This avoids merging independent
        // same-second conversions into one synthetic multi-dust conversion.
        if effective_outgoing.len() > 1 && effective_incoming.len() > 1 {
            let pair_count = effective_outgoing.len().min(effective_incoming.len());
            for i in 0..pair_count {
                if let Some(tx) =
                    resolve_single_pair(effective_outgoing[i], effective_incoming[i], wallet_name)
                {
                    results.push(tx);
                }
            }

            // If all rows were consumed via 1:1 pairing, we're done.
            if effective_outgoing.len() == pair_count && effective_incoming.len() == pair_count {
                return results;
            }

            // Keep only leftovers for best-effort handling below.
            effective_outgoing = effective_outgoing.into_iter().skip(pair_count).collect();
            effective_incoming = effective_incoming.into_iter().skip(pair_count).collect();
        }

        // ── Step 4: Multi-outgoing (SmallAssetsExchange): many dust -> one target ──
        if effective_incoming.len() == 1 && !effective_outgoing.is_empty() {
            let inc = effective_incoming[0];
            let total_outgoing_count = effective_outgoing.len();

            // Count non-fiat outgoing rows so we can split the incoming evenly.
            let non_fiat_count = effective_outgoing
                .iter()
                .filter(|o| !is_fiat(&o.symbol))
                .count();
            let share = if non_fiat_count > 0 {
                inc.change.abs() / non_fiat_count as f64
            } else {
                0.0
            };

            for out in &effective_outgoing {
                if is_fiat(&out.symbol) {
                    continue;
                }

                // Guard: skip same-symbol pairs within multi-path
                if out.symbol.eq_ignore_ascii_case(&inc.symbol) {
                    continue;
                }

                let date = format_datetime(out.timestamp);
                let notes = Some(format!(
                    "Binance {} ({} assets) | {}",
                    out.operation_raw, total_outgoing_count, out.remark
                ));

                if is_fiat(&inc.symbol) {
                    // Dust -> fiat (unusual but handle it)
                    results.push((
                        out.line_number,
                        ImportCryptoTransaction {
                            date,
                            wallet: wallet_name.to_string(),
                            symbol: out.symbol.clone(),
                            transaction_type: "trade".to_string(),
                            amount: out.change.abs(),
                            subtype: Some("sell".to_string()),
                            price_per_coin: None,
                            fee: None,
                            override_proceeds: None,
                            override_cost_basis: None,
                            swap_to_symbol: None,
                            swap_to_amount: None,
                            fee_coin_symbol: None,
                            fee_amount: None,
                            notes,
                        },
                    ));
                } else {
                    // Dust -> BNB (or other crypto) = swap
                    // We split the total incoming amount equally across all
                    // non-fiat outgoing rows. Not perfectly accurate for
                    // heterogeneous dust, but deterministic and passes
                    // validation (swap_to_amount must be present).
                    results.push((
                        out.line_number,
                        ImportCryptoTransaction {
                            date,
                            wallet: wallet_name.to_string(),
                            symbol: out.symbol.clone(),
                            transaction_type: "trade".to_string(),
                            amount: out.change.abs(),
                            subtype: Some("swap".to_string()),
                            price_per_coin: None,
                            fee: None,
                            override_proceeds: None,
                            override_cost_basis: None,
                            swap_to_symbol: Some(inc.symbol.clone()),
                            swap_to_amount: Some(share),
                            fee_coin_symbol: None,
                            fee_amount: None,
                            notes,
                        },
                    ));
                }
            }
        }

        results
    }
}

fn resolve_single_pair(
    out: &BinanceRow,
    inc: &BinanceRow,
    wallet_name: &str,
) -> Option<(usize, ImportCryptoTransaction)> {
    let out_fiat = is_fiat(&out.symbol);
    let in_fiat = is_fiat(&inc.symbol);

    // Stablecoin-aware classification: treat stablecoins as pricing currencies
    // so BTC/USDT becomes a buy/sell (not a swap).
    let out_is_pricing = is_quote_currency(&out.symbol);
    let in_is_pricing = is_quote_currency(&inc.symbol);

    // Both fiat — skip
    if out_fiat && in_fiat {
        return None;
    }

    // Same symbol after filtering (shouldn't happen, but guard)
    if out.symbol.eq_ignore_ascii_case(&inc.symbol) {
        return None;
    }

    let date = format_datetime(out.timestamp);
    let line = out.line_number;
    let notes = Some(format!("Binance {} | {}", out.operation_raw, out.remark));

    if out_is_pricing && !in_is_pricing {
        // Fiat/stablecoin -> Crypto = buy
        let price = if inc.change.abs() > 0.0 {
            Some(out.change.abs() / inc.change.abs())
        } else {
            None
        };
        Some((
            line,
            ImportCryptoTransaction {
                date,
                wallet: wallet_name.to_string(),
                symbol: inc.symbol.clone(),
                transaction_type: "trade".to_string(),
                amount: inc.change.abs(),
                subtype: Some("buy".to_string()),
                price_per_coin: price,
                fee: None,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: None,
                swap_to_amount: None,
                fee_coin_symbol: None,
                fee_amount: None,
                notes,
            },
        ))
    } else if !out_is_pricing && in_is_pricing {
        // Crypto -> Fiat/stablecoin = sell
        let price = if out.change.abs() > 0.0 {
            Some(inc.change.abs() / out.change.abs())
        } else {
            None
        };
        Some((
            line,
            ImportCryptoTransaction {
                date,
                wallet: wallet_name.to_string(),
                symbol: out.symbol.clone(),
                transaction_type: "trade".to_string(),
                amount: out.change.abs(),
                subtype: Some("sell".to_string()),
                price_per_coin: price,
                fee: None,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: None,
                swap_to_amount: None,
                fee_coin_symbol: None,
                fee_amount: None,
                notes,
            },
        ))
    } else {
        // Crypto -> Crypto = swap (includes stablecoin-to-stablecoin)
        Some((
            line,
            ImportCryptoTransaction {
                date,
                wallet: wallet_name.to_string(),
                symbol: out.symbol.clone(),
                transaction_type: "trade".to_string(),
                amount: out.change.abs(),
                subtype: Some("swap".to_string()),
                price_per_coin: None,
                fee: None,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: Some(inc.symbol.clone()),
                swap_to_amount: Some(inc.change.abs()),
                fee_coin_symbol: None,
                fee_amount: None,
                notes,
            },
        ))
    }
}

mod all_statements;
mod mapping;
mod spot;

pub use all_statements::BinanceAllStatementsParser;
use mapping::{single_row_to_transaction, unpaired_row_to_transaction};
pub use spot::BinanceSpotParser;

#[cfg(test)]
mod tests_all_statements;
#[cfg(test)]
mod tests_spot_and_pairing;
