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

use std::collections::HashMap;

use chrono::NaiveDateTime;
use csv::{ReaderBuilder, StringRecord, Trim};

use super::common::{
    format_datetime, is_fiat, normalize_binance_currency, parse_amount_with_unit, parse_decimal,
    parse_timestamp, should_rename_luna_to_lunc,
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
    /// A simple 1:1 convert produces a single swap.
    /// A SmallAssetsExchange can have many outgoing (small dust) and one incoming
    /// (BNB). We emit one swap per outgoing row paired with the single incoming.
    fn resolve(self, wallet_name: &str) -> Vec<(usize, ImportCryptoTransaction)> {
        let mut results = Vec::new();

        // Simple case: exactly one of each side
        if self.outgoing.len() == 1 && self.incoming.len() == 1 {
            let out = &self.outgoing[0];
            let inc = &self.incoming[0];

            let out_fiat = is_fiat(&out.symbol);
            let in_fiat = is_fiat(&inc.symbol);

            // Both fiat — skip
            if out_fiat && in_fiat {
                return results;
            }

            // Same symbol on both sides (e.g. USDT -> USDT) — no-op, skip
            if out.symbol.eq_ignore_ascii_case(&inc.symbol) {
                return results;
            }

            let date = format_datetime(out.timestamp);
            let line = out.line_number;
            let notes = Some(format!("Binance {} | {}", out.operation_raw, out.remark));

            if out_fiat && !in_fiat {
                // Fiat -> Crypto = buy
                let price = if inc.change.abs() > 0.0 {
                    Some(out.change.abs() / inc.change.abs())
                } else {
                    None
                };
                results.push((
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
                ));
            } else if !out_fiat && in_fiat {
                // Crypto -> Fiat = sell
                let price = if out.change.abs() > 0.0 {
                    Some(inc.change.abs() / out.change.abs())
                } else {
                    None
                };
                results.push((
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
                ));
            } else {
                // Crypto -> Crypto = swap
                results.push((
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
                ));
            }

            return results;
        }

        // Multi-outgoing case (SmallAssetsExchange): many dust -> one BNB
        // We emit one swap for each outgoing asset, distributing the incoming
        // amount equally among all non-fiat outgoing rows.
        if !self.incoming.is_empty() && !self.outgoing.is_empty() {
            let inc = &self.incoming[0];
            let total_outgoing_count = self.outgoing.len();

            // Count non-fiat outgoing rows so we can split the incoming evenly.
            let non_fiat_count = self.outgoing.iter().filter(|o| !is_fiat(&o.symbol)).count();
            let share = if non_fiat_count > 0 {
                inc.change.abs() / non_fiat_count as f64
            } else {
                0.0
            };

            for out in &self.outgoing {
                if is_fiat(&out.symbol) {
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

/// Converts a single (non-paired) Binance row into an `ImportCryptoTransaction`.
/// Returns `None` for fiat rows, skippable ops, or rows that need pairing.
fn single_row_to_transaction(
    row: &BinanceRow,
    wallet_name: &str,
) -> Option<ImportCryptoTransaction> {
    if is_fiat(&row.symbol) {
        return None;
    }

    if row.operation.needs_pairing() {
        return None;
    }

    if row.operation.should_skip() {
        return None;
    }

    let date = format_datetime(row.timestamp);
    let notes = Some(format!("Binance {} | {}", row.operation_raw, row.remark));

    let (tx_type, subtype) = match row.operation {
        BinanceOperation::Buy | BinanceOperation::TransactionBuy => {
            ("trade".to_string(), Some("buy".to_string()))
        }
        BinanceOperation::Sell | BinanceOperation::TransactionSold => {
            ("trade".to_string(), Some("sell".to_string()))
        }
        BinanceOperation::P2PTrade => {
            if row.change > 0.0 {
                ("trade".to_string(), Some("buy".to_string()))
            } else {
                ("trade".to_string(), Some("sell".to_string()))
            }
        }
        BinanceOperation::Deposit => ("transfer".to_string(), Some("deposit".to_string())),
        BinanceOperation::Withdraw => ("transfer".to_string(), Some("withdrawal".to_string())),
        BinanceOperation::Distribution | BinanceOperation::AirdropAssets => {
            ("income".to_string(), Some("airdrop".to_string()))
        }
        BinanceOperation::StakingRewards => ("income".to_string(), Some("staking".to_string())),
        BinanceOperation::Fee => ("expense".to_string(), Some("fee".to_string())),
        BinanceOperation::CardCashback => ("income".to_string(), Some("rebate".to_string())),
        BinanceOperation::CardSpending => {
            if row.change > 0.0 {
                // Refund
                ("income".to_string(), Some("rebate".to_string()))
            } else {
                ("expense".to_string(), Some("payment".to_string()))
            }
        }
        BinanceOperation::TransactionSpend => {
            // Standalone spend without a matching revenue — treat as expense
            ("expense".to_string(), Some("payment".to_string()))
        }
        BinanceOperation::TransactionRevenue => {
            // Standalone revenue without a matching spend — treat as income
            ("income".to_string(), Some("other".to_string()))
        }
        BinanceOperation::Unknown => {
            if row.change > 0.0 {
                ("income".to_string(), Some("other".to_string()))
            } else {
                ("expense".to_string(), Some("other".to_string()))
            }
        }
        _ => return None,
    };

    Some(ImportCryptoTransaction {
        date,
        wallet: wallet_name.to_string(),
        symbol: row.symbol.clone(),
        transaction_type: tx_type,
        amount: row.change.abs(),
        subtype,
        price_per_coin: None,
        fee: None,
        override_proceeds: None,
        override_cost_basis: None,
        swap_to_symbol: None,
        swap_to_amount: None,
        fee_coin_symbol: None,
        fee_amount: None,
        notes,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
//  Binance All Statements Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct BinanceAllStatementsParser;

impl ExchangeParser for BinanceAllStatementsParser {
    fn parse(
        &self,
        content: &str,
        wallet_name: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError> {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());

        let headers = reader
            .headers()
            .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {}", e)))?
            .clone();

        let cols = resolve_all_statements_columns(&headers);

        for required in &["utc_time", "operation", "coin", "change"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!(
                        "Missing required Binance column: '{}'",
                        match *required {
                            "utc_time" => "UTC_Time",
                            "operation" => "Operation",
                            "coin" => "Coin",
                            "change" => "Change",
                            other => other,
                        }
                    ),
                ));
            }
        }

        let mut result: ParseResult<ImportCryptoTransaction> = ParseResult::default();

        // Accumulate all rows first so we can pair Convert/SmallAssets rows.
        let mut rows: Vec<BinanceRow> = Vec::new();

        for (idx, record) in reader.records().enumerate() {
            let record = match record {
                Ok(r) => r,
                Err(err) => {
                    let line = err.position().map(|p| p.line()).unwrap_or((idx + 2) as u64);
                    result.errors.push(RowError::new(
                        line as usize,
                        None,
                        format!("Invalid CSV record: {}", err),
                    ));
                    continue;
                }
            };

            let line_number = record
                .position()
                .map(|p| p.line())
                .unwrap_or((idx + 2) as u64) as usize;

            let time_raw = get_field(&record, &cols, "utc_time");
            let operation_raw = get_field(&record, &cols, "operation").to_string();
            let coin_raw = get_field(&record, &cols, "coin");
            let change_raw = get_field(&record, &cols, "change");
            let remark = get_field(&record, &cols, "remark").to_string();

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("UTC_Time"),
                        format!("Invalid timestamp: '{}'", time_raw),
                    ));
                    continue;
                }
            };

            let change = match parse_decimal(change_raw) {
                Some(v) => v,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Change"),
                        format!("Invalid change value: '{}'", change_raw),
                    ));
                    continue;
                }
            };

            // Skip zero-change rows
            if change.abs() < f64::EPSILON {
                continue;
            }

            let symbol = normalise_coin(coin_raw, timestamp);
            let operation = BinanceOperation::parse(&operation_raw);

            rows.push(BinanceRow {
                timestamp,
                operation,
                operation_raw,
                symbol,
                change,
                remark,
                line_number,
            });
        }

        // ── Phase 1: pair Convert and SmallAssetsExchange rows ──

        // We pair by (timestamp, operation_type). For SmallAssetsExchange there
        // may be many outgoing rows with the same timestamp.
        //
        // Key: (timestamp_string, is_convert_or_small_assets)
        // We use the formatted timestamp as key to handle minor rounding.
        let mut pending_converts: HashMap<String, PendingConvert> = HashMap::new();
        let mut standalone_rows: Vec<BinanceRow> = Vec::new();

        // Also handle Transaction Spend/Revenue pairing
        let mut pending_spend_revenue: HashMap<String, PendingConvert> = HashMap::new();

        for row in rows {
            if row.operation.should_skip() {
                continue;
            }

            if row.operation.needs_pairing() {
                let key = format!(
                    "{}_{:?}",
                    row.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    row.operation
                );
                let entry = pending_converts.entry(key).or_default();
                entry.insert(row);
            } else if matches!(
                row.operation,
                BinanceOperation::TransactionSpend | BinanceOperation::TransactionRevenue
            ) {
                // Pair TransactionSpend + TransactionRevenue by timestamp
                let key = row.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                let entry = pending_spend_revenue.entry(key).or_default();
                // Spend is outgoing (negative), Revenue is incoming (positive)
                entry.insert(row);
            } else {
                standalone_rows.push(row);
            }
        }

        // Resolve Convert pairs
        for (_key, pending) in pending_converts {
            if pending.is_complete() {
                let txs = pending.resolve(wallet_name);
                for (line, tx) in txs {
                    result.items.push((line, tx));
                }
            } else {
                // Incomplete pair: emit individual rows as standalone
                for row in pending.outgoing.iter().chain(pending.incoming.iter()) {
                    if let Some(tx) = single_row_to_transaction(row, wallet_name) {
                        result.items.push((row.line_number, tx));
                    }
                }
            }
        }

        // Resolve Transaction Spend/Revenue pairs (treated like converts)
        for (_key, pending) in pending_spend_revenue {
            if pending.is_complete() {
                let txs = pending.resolve(wallet_name);
                for (line, tx) in txs {
                    result.items.push((line, tx));
                }
            } else {
                // Emit unpaired spend/revenue as standalone
                for row in pending.outgoing.iter().chain(pending.incoming.iter()) {
                    if let Some(tx) = single_row_to_transaction(row, wallet_name) {
                        result.items.push((row.line_number, tx));
                    }
                }
            }
        }

        // ── Phase 2: process standalone rows ──

        for row in &standalone_rows {
            if let Some(tx) = single_row_to_transaction(row, wallet_name) {
                result.items.push((row.line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::BinanceAllStatements
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Binance Spot Trade History Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct BinanceSpotParser;

impl ExchangeParser for BinanceSpotParser {
    fn parse(
        &self,
        content: &str,
        wallet_name: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError> {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());

        let headers = reader
            .headers()
            .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {}", e)))?
            .clone();

        let cols = resolve_spot_columns(&headers);

        for required in &["date", "side", "executed", "amount", "fee"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!(
                        "Missing required Binance Spot column: '{}'",
                        match *required {
                            "date" => "Date(UTC)",
                            "side" => "Side",
                            "executed" => "Executed",
                            "amount" => "Amount",
                            "fee" => "Fee",
                            other => other,
                        }
                    ),
                ));
            }
        }

        let mut result: ParseResult<ImportCryptoTransaction> = ParseResult::default();

        for (idx, record) in reader.records().enumerate() {
            let record = match record {
                Ok(r) => r,
                Err(err) => {
                    let line = err.position().map(|p| p.line()).unwrap_or((idx + 2) as u64);
                    result.errors.push(RowError::new(
                        line as usize,
                        None,
                        format!("Invalid CSV record: {}", err),
                    ));
                    continue;
                }
            };

            let line_number = record
                .position()
                .map(|p| p.line())
                .unwrap_or((idx + 2) as u64) as usize;

            let date_raw = get_field(&record, &cols, "date");
            let side_raw = get_field(&record, &cols, "side");
            let executed_raw = get_field(&record, &cols, "executed");
            let amount_raw = get_field(&record, &cols, "amount");
            let fee_raw = get_field(&record, &cols, "fee");
            let pair_raw = get_field(&record, &cols, "pair");

            let timestamp = match parse_timestamp(date_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Date(UTC)"),
                        format!("Invalid timestamp: '{}'", date_raw),
                    ));
                    continue;
                }
            };

            let date = format_datetime(timestamp);
            let is_buy = side_raw.eq_ignore_ascii_case("BUY");

            // Parse Executed (base currency amount, e.g. "0.5BTC")
            let (executed_qty, executed_unit) = match parse_amount_with_unit(executed_raw) {
                Some(v) => v,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Executed"),
                        format!("Cannot parse executed amount: '{}'", executed_raw),
                    ));
                    continue;
                }
            };

            // Parse Amount (quote currency amount, e.g. "25000USDT")
            let (amount_qty, amount_unit) = match parse_amount_with_unit(amount_raw) {
                Some(v) => v,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Amount"),
                        format!("Cannot parse amount: '{}'", amount_raw),
                    ));
                    continue;
                }
            };

            // Parse Fee (e.g. "0.001BNB")
            let (fee_qty, fee_unit) = match parse_amount_with_unit(fee_raw) {
                Some(v) => v,
                None => (0.0, String::new()),
            };

            // Normalise currencies
            let base_symbol = normalise_coin(&executed_unit, timestamp);
            let quote_symbol = normalise_coin(&amount_unit, timestamp);
            let fee_symbol = if !fee_unit.is_empty() {
                normalise_coin(&fee_unit, timestamp)
            } else {
                String::new()
            };

            let base_fiat = is_fiat(&base_symbol);
            let quote_fiat = is_fiat(&quote_symbol);

            let notes = if pair_raw.is_empty() {
                Some(format!(
                    "Binance Spot {} | {}/{}",
                    side_raw, base_symbol, quote_symbol
                ))
            } else {
                Some(format!("Binance Spot {} | {}", side_raw, pair_raw))
            };

            // Determine fee fields
            let (fee_usd, fee_coin_sym, fee_coin_amt) = if fee_qty.abs() > f64::EPSILON {
                if is_fiat(&fee_symbol) {
                    (Some(fee_qty), None, None)
                } else {
                    (None, Some(fee_symbol.clone()), Some(fee_qty))
                }
            } else {
                (None, None, None)
            };

            // Both fiat — skip
            if base_fiat && quote_fiat {
                continue;
            }

            if quote_fiat {
                // Standard pair: BTC/USD
                let price = if executed_qty > 0.0 {
                    Some(amount_qty / executed_qty)
                } else {
                    None
                };

                let subtype = if is_buy { "buy" } else { "sell" };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: base_symbol,
                    transaction_type: "trade".to_string(),
                    amount: executed_qty,
                    subtype: Some(subtype.to_string()),
                    price_per_coin: price,
                    fee: fee_usd,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: fee_coin_sym,
                    fee_amount: fee_coin_amt,
                    notes,
                };

                result.items.push((line_number, tx));
            } else if base_fiat {
                // Inverted pair: USD/BTC (rare but handle)
                let subtype = if is_buy { "sell" } else { "buy" };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: quote_symbol,
                    transaction_type: "trade".to_string(),
                    amount: amount_qty,
                    subtype: Some(subtype.to_string()),
                    price_per_coin: None,
                    fee: fee_usd,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: fee_coin_sym,
                    fee_amount: fee_coin_amt,
                    notes,
                };

                result.items.push((line_number, tx));
            } else {
                // Crypto-to-crypto: swap
                let (from_symbol, from_amount, to_symbol, to_amount) = if is_buy {
                    // Buying base with quote: out=quote, in=base
                    (quote_symbol, amount_qty, base_symbol, executed_qty)
                } else {
                    // Selling base for quote: out=base, in=quote
                    (base_symbol, executed_qty, quote_symbol, amount_qty)
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: from_symbol,
                    transaction_type: "trade".to_string(),
                    amount: from_amount,
                    subtype: Some("swap".to_string()),
                    price_per_coin: None,
                    fee: fee_usd,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: Some(to_symbol),
                    swap_to_amount: Some(to_amount),
                    fee_coin_symbol: fee_coin_sym,
                    fee_amount: fee_coin_amt,
                    notes,
                };

                result.items.push((line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::BinanceSpotTradeHistory
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── All Statements parser ──

    #[test]
    fn all_statements_buy_becomes_trade_buy() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:30:45,Spot,Buy,BTC,0.5,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
        assert_eq!(tx.wallet, "Binance");
    }

    #[test]
    fn all_statements_sell_becomes_trade_sell() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:30:45,Spot,Sell,ETH,-2.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn all_statements_deposit_becomes_transfer_deposit() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-02-01 08:00:00,Spot,Deposit,BTC,1.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
    }

    #[test]
    fn all_statements_withdraw_becomes_transfer_withdrawal() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-02-01 08:00:00,Spot,Withdraw,BTC,-0.5,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
    }

    #[test]
    fn all_statements_distribution_becomes_income_airdrop() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-03-01 00:00:00,Spot,Distribution,FLR,100.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("airdrop"));
    }

    #[test]
    fn all_statements_staking_rewards_becomes_income_staking() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-03-15 00:00:00,Spot,Staking Rewards,DOT,0.05,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("staking"));
    }

    #[test]
    fn all_statements_simple_earn_flexible_interest() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-03-15 00:00:00,Spot,Simple Earn Flexible Interest,USDT,0.12,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        // USDT is not fiat in our system, so it should be parsed
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("staking"));
    }

    #[test]
    fn all_statements_fee_becomes_expense_fee() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:30:45,Spot,Fee,BNB,-0.001,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "expense");
        assert_eq!(tx.subtype.as_deref(), Some("fee"));
        assert!((tx.amount - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn all_statements_convert_pair_becomes_swap() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-04-01 12:00:00,Spot,Binance Convert,ETH,-2.0,\n",
            "12345,2024-04-01 12:00:00,Spot,Binance Convert,SOL,50.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert!((tx.amount - 2.0).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("SOL"));
        assert!((tx.swap_to_amount.unwrap() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn all_statements_convert_fiat_to_crypto_becomes_buy() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-04-01 12:00:00,Spot,Binance Convert,USD,-1000.0,\n",
            "12345,2024-04-01 12:00:00,Spot,Binance Convert,BTC,0.02,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.02).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < 0.01);
    }

    #[test]
    fn all_statements_convert_crypto_to_fiat_becomes_sell() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-04-01 12:00:00,Spot,Binance Convert,BTC,-0.5,\n",
            "12345,2024-04-01 12:00:00,Spot,Binance Convert,USD,25000.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn all_statements_fiat_deposit_is_skipped() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:00:00,Spot,Fiat Deposit,USD,10000.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert!(result.items.is_empty());
    }

    #[test]
    fn all_statements_internal_transfer_is_skipped() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:00:00,Spot,Transfer Between Main and Funding Wallet,BTC,-0.5,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert!(result.items.is_empty());
    }

    #[test]
    fn all_statements_bcc_normalised_to_bch() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:00:00,Spot,Deposit,BCC,1.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].1.symbol, "BCH");
    }

    #[test]
    fn all_statements_luna_renamed_before_cutoff() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2022-05-01 10:00:00,Spot,Buy,LUNA,100.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].1.symbol, "LUNC");
    }

    #[test]
    fn all_statements_luna_not_renamed_after_cutoff() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2022-06-01 10:00:00,Spot,Buy,LUNA,100.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].1.symbol, "LUNA");
    }

    #[test]
    fn all_statements_card_cashback_becomes_income_rebate() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-05-01 10:00:00,Spot,Binance Card Cashback,BNB,0.01,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("rebate"));
    }

    #[test]
    fn all_statements_card_spending_becomes_expense_payment() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-05-01 10:00:00,Spot,Binance Card Spending,BNB,-0.5,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "expense");
        assert_eq!(tx.subtype.as_deref(), Some("payment"));
    }

    #[test]
    fn all_statements_airdrop_assets_becomes_income_airdrop() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-06-01 00:00:00,Spot,Airdrop Assets,ARB,50.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("airdrop"));
    }

    #[test]
    fn all_statements_spend_revenue_pair_becomes_trade() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-04-01 12:00:00,Spot,Transaction Spend,USDT,-100.0,\n",
            "12345,2024-04-01 12:00:00,Spot,Transaction Revenue,BTC,0.002,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        // USDT is not fiat, so this is a swap
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
    }

    #[test]
    fn all_statements_zero_change_is_skipped() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:00:00,Spot,Buy,BTC,0.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert!(result.items.is_empty());
    }

    #[test]
    fn all_statements_uses_custom_wallet_name() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:00:00,Spot,Deposit,BTC,1.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Mi Binance").unwrap();

        assert_eq!(result.items[0].1.wallet, "Mi Binance");
    }

    #[test]
    fn all_statements_crypto_fiat_deposit_ignored() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:00:00,Spot,Deposit,USD,1000.0,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        // USD deposit: not skipped by should_skip (that's only for transfers/fiat ops),
        // but is_fiat check in single_row_to_transaction filters it out
        assert!(result.items.is_empty());
    }

    #[test]
    fn all_statements_unknown_operation_handled() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:00:00,Spot,Super New Feature,BTC,0.01,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        // Positive change + unknown = income/other
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("other"));
    }

    // ── Spot Trade History parser ──

    #[test]
    fn spot_buy_btc_usdt() {
        let csv = concat!(
            "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
            "2024-01-15 10:30:45,BTCUSDT,BUY,50000.00,0.5BTC,25000USDT,0.001BNB\n",
        );

        let parser = BinanceSpotParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        // USDT is not fiat, so BTC/USDT is crypto-to-crypto = swap
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        // Buying BTC with USDT: outgoing=USDT, incoming=BTC
        assert_eq!(tx.symbol, "USDT");
        assert!((tx.amount - 25000.0).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("BTC"));
        assert!((tx.swap_to_amount.unwrap() - 0.5).abs() < f64::EPSILON);
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("BNB"));
        assert!((tx.fee_amount.unwrap() - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn spot_sell_eth_usd() {
        let csv = concat!(
            "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
            "2024-02-01 08:00:00,ETHUSD,SELL,2000.00,2.0ETH,4000USD,3.50USD\n",
        );

        let parser = BinanceSpotParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 2.0).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 2000.0).abs() < 0.01);
        assert!((tx.fee.unwrap() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn spot_buy_with_fiat_quote() {
        let csv = concat!(
            "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
            "2024-01-15 10:30:45,BTCUSD,BUY,50000.00,0.1BTC,5000USD,1.00USD\n",
        );

        let parser = BinanceSpotParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.1).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < 0.01);
        assert!((tx.fee.unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spot_crypto_to_crypto_becomes_swap() {
        let csv = concat!(
            "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
            "2024-03-15 14:00:00,ETHBTC,BUY,0.05,5.0ETH,0.25BTC,0.0001BTC\n",
        );

        let parser = BinanceSpotParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        // Buying ETH with BTC: outgoing=BTC, incoming=ETH
        assert_eq!(tx.symbol, "BTC");
        assert!((tx.amount - 0.25).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
        assert!((tx.swap_to_amount.unwrap() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spot_bcc_normalised_to_bch() {
        let csv = concat!(
            "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
            "2024-01-15 10:00:00,BCCUSD,BUY,300.00,1.0BCC,300USD,0.01BCC\n",
        );

        let parser = BinanceSpotParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].1.symbol, "BCH");
    }

    #[test]
    fn spot_uses_custom_wallet_name() {
        let csv = concat!(
            "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
            "2024-01-15 10:00:00,BTCUSD,BUY,50000,0.1BTC,5000USD,1USD\n",
        );

        let parser = BinanceSpotParser;
        let result = parser.parse(csv, "Mi Binance Spot").unwrap();

        assert_eq!(result.items[0].1.wallet, "Mi Binance Spot");
    }

    #[test]
    fn spot_empty_content() {
        let csv = "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n";

        let parser = BinanceSpotParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn spot_invalid_executed_is_error() {
        let csv = concat!(
            "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
            "2024-01-15 10:00:00,BTCUSD,BUY,50000,INVALID,5000USD,1USD\n",
        );

        let parser = BinanceSpotParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
    }

    // ── Edge cases ──

    #[test]
    fn all_statements_multiple_operations() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-10 08:00:00,Spot,Deposit,BTC,1.0,\n",
            "12345,2024-01-11 09:00:00,Spot,Buy,ETH,5.0,\n",
            "12345,2024-01-12 10:00:00,Spot,Staking Rewards,DOT,0.5,\n",
            "12345,2024-01-13 11:00:00,Spot,Withdraw,BTC,-0.3,\n",
            "12345,2024-01-14 12:00:00,Spot,Fee,BNB,-0.001,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 5);

        // Verify types
        let types: Vec<(&str, Option<&str>)> = result
            .items
            .iter()
            .map(|(_, tx)| (tx.transaction_type.as_str(), tx.subtype.as_deref()))
            .collect();

        // Note: ordering depends on HashMap iteration, so just check all are present
        assert!(types.contains(&("transfer", Some("deposit"))));
        assert!(types.contains(&("trade", Some("buy"))));
        assert!(types.contains(&("income", Some("staking"))));
        assert!(types.contains(&("transfer", Some("withdrawal"))));
        assert!(types.contains(&("expense", Some("fee"))));
    }

    #[test]
    fn all_statements_small_assets_exchange_multi_dust() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,DOGE,-100.0,\n",
            "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,ADA,-50.0,\n",
            "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,BNB,0.15,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        // Two swap transactions (DOGE->BNB and ADA->BNB)
        assert_eq!(result.items.len(), 2);

        for item in &result.items {
            let tx = &item.1;
            assert_eq!(tx.transaction_type, "trade");
            assert_eq!(tx.subtype.as_deref(), Some("swap"));
            assert_eq!(tx.swap_to_symbol.as_deref(), Some("BNB"));
            // Incoming BNB (0.15) split equally across 2 dust assets
            assert_eq!(tx.swap_to_amount, Some(0.075));
        }
    }

    #[test]
    fn all_statements_convert_same_symbol_is_skipped() {
        // A convert where both sides are the same coin (e.g. USDT -> USDT)
        // is a no-op and should produce zero transactions.
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:30:45,Spot,Binance Convert,USDT,-6.20000000,\n",
            "12345,2024-01-15 10:30:45,Spot,Binance Convert,USDT,6.20000000,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 0);
    }

    // ── P2P Trading tests ──

    #[test]
    fn all_statements_p2p_buy_becomes_trade_buy() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:30:45,Funding,P2P Trading,USDT,6.20000000,P2P - 12345678\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 6.2).abs() < f64::EPSILON);
        assert!(tx.notes.as_ref().unwrap().contains("P2P Trading"));
        assert!(tx.notes.as_ref().unwrap().contains("P2P - 12345678"));
    }

    #[test]
    fn all_statements_p2p_sell_becomes_trade_sell() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-02-20 14:00:00,Funding,P2P Trading,BTC,-0.01000000,P2P - 99887766\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn all_statements_c2c_transfer_becomes_trade_buy() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-03-10 08:15:00,Funding,C2C Transfer,ETH,1.50000000,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn all_statements_p2p_fiat_is_skipped() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:30:45,Funding,P2P Trading,USD,100.00000000,P2P - 12345678\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        // USD is fiat — should be skipped
        assert_eq!(result.items.len(), 0);
    }

    // ── Transfer between sub-accounts (UM Futures, etc.) ──

    #[test]
    fn all_statements_um_futures_transfer_is_skipped() {
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-15 10:30:45,USD-M Futures,Transfer Between Spot Account and UM Futures Account,USDT,-20.00000000,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        // Internal transfer — should be skipped
        assert_eq!(result.items.len(), 0);
    }

    #[test]
    fn all_statements_unknown_transfer_between_is_skipped() {
        // Any "Transfer Between..." string we haven't explicitly listed should
        // still be caught by the starts_with fallback.
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-06-01 12:00:00,Spot,Transfer Between Spot and Some New Account,BTC,0.10000000,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        assert_eq!(result.items.len(), 0);
    }

    #[test]
    fn all_statements_mixed_p2p_convert_transfer() {
        // Realistic scenario: P2P buy, then convert, then internal transfer
        let csv = concat!(
            "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
            "12345,2024-01-10 09:00:00,Funding,P2P Trading,USDT,100.00000000,P2P - 111\n",
            "12345,2024-01-10 10:00:00,Spot,Binance Convert,USDT,-50.00000000,\n",
            "12345,2024-01-10 10:00:00,Spot,Binance Convert,BTC,0.00120000,\n",
            "12345,2024-01-10 11:00:00,USD-M Futures,Transfer Between Spot Account and UM Futures Account,USDT,-20.00000000,\n",
        );

        let parser = BinanceAllStatementsParser;
        let result = parser.parse(csv, "Binance").unwrap();

        // P2P buy (1) + Convert pair becomes buy since USDT->BTC with USDT
        // not being fiat (1) + Transfer skipped (0) = 2 transactions
        assert_eq!(result.items.len(), 2);

        // Find the P2P trade
        let p2p = result
            .items
            .iter()
            .find(|(_, tx)| {
                tx.notes
                    .as_ref()
                    .map_or(false, |n| n.contains("P2P Trading"))
            })
            .map(|(_, tx)| tx)
            .expect("Should have a P2P transaction");
        assert_eq!(p2p.symbol, "USDT");
        assert_eq!(p2p.subtype.as_deref(), Some("buy"));
        assert!((p2p.amount - 100.0).abs() < f64::EPSILON);

        // Find the convert (USDT -> BTC = swap since neither is fiat)
        let convert = result
            .items
            .iter()
            .find(|(_, tx)| {
                tx.notes
                    .as_ref()
                    .map_or(false, |n| n.contains("Binance Convert"))
            })
            .map(|(_, tx)| tx)
            .expect("Should have a Convert transaction");
        assert_eq!(convert.symbol, "USDT");
        assert_eq!(convert.subtype.as_deref(), Some("swap"));
        assert_eq!(convert.swap_to_symbol.as_deref(), Some("BTC"));
    }
}
