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

//! Kraken CSV parsers
//!
//! Supports two export formats:
//!
//! ## Ledger CSV (recommended — covers ALL transaction types)
//!
//! Each trade generates **two rows** linked by `refid` (one per asset).
//! `spend`/`receive` pairs with the same `refid` are also trades.
//! Internal staking transfers (subtypes like `spottostaking`) are skipped.
//!
//! ## Trades CSV (spot/margin trades only)
//!
//! Simpler format with one row per trade including pair, price, cost, volume.

use std::collections::HashMap;

use chrono::NaiveDateTime;
use csv::{ReaderBuilder, StringRecord, Trim};

use super::common::{
    format_datetime, is_fiat, normalize_kraken_currency, parse_decimal, parse_kraken_pair,
    parse_timestamp,
};
use super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

// ─── Kraken Ledger types ────────────────────────────────────────────────────

/// Known ledger entry types from Kraken's export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LedgerType {
    Trade,
    MarginTrade,
    Earn,
    Rollover,
    Deposit,
    Withdrawal,
    Transfer,
    Adjustment,
    Spend,
    Receive,
    Settled,
    Staking,
    InviteBonus,
    Unknown,
}

impl LedgerType {
    fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "trade" => LedgerType::Trade,
            "margin trade" => LedgerType::MarginTrade,
            "earn" => LedgerType::Earn,
            "rollover" => LedgerType::Rollover,
            "deposit" => LedgerType::Deposit,
            "withdrawal" => LedgerType::Withdrawal,
            "transfer" => LedgerType::Transfer,
            "adjustment" => LedgerType::Adjustment,
            "spend" => LedgerType::Spend,
            "receive" => LedgerType::Receive,
            "settled" => LedgerType::Settled,
            "staking" => LedgerType::Staking,
            "invite bonus" => LedgerType::InviteBonus,
            _ => LedgerType::Unknown,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            LedgerType::Trade => "trade",
            LedgerType::MarginTrade => "margin trade",
            LedgerType::Earn => "earn",
            LedgerType::Rollover => "rollover",
            LedgerType::Deposit => "deposit",
            LedgerType::Withdrawal => "withdrawal",
            LedgerType::Transfer => "transfer",
            LedgerType::Adjustment => "adjustment",
            LedgerType::Spend => "spend",
            LedgerType::Receive => "receive",
            LedgerType::Settled => "settled",
            LedgerType::Staking => "staking",
            LedgerType::InviteBonus => "invite bonus",
            LedgerType::Unknown => "unknown",
        }
    }
}

/// Known ledger subtypes (primarily staking-related).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LedgerSubtype {
    Allocation,
    Deallocation,
    Autoallocate,
    Reward,
    Migration,
    SpotToStaking,
    StakingFromSpot,
    StakingToSpot,
    SpotFromStaking,
    SpotToFutures,
    SpotFromFutures,
    Other,
}

impl LedgerSubtype {
    fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim().to_lowercase();
        if trimmed.is_empty() {
            return None;
        }
        Some(match trimmed.as_str() {
            "allocation" => LedgerSubtype::Allocation,
            "deallocation" => LedgerSubtype::Deallocation,
            "autoallocate" => LedgerSubtype::Autoallocate,
            "reward" => LedgerSubtype::Reward,
            "migration" => LedgerSubtype::Migration,
            "spottostaking" => LedgerSubtype::SpotToStaking,
            "stakingfromspot" => LedgerSubtype::StakingFromSpot,
            "stakingtospot" => LedgerSubtype::StakingToSpot,
            "spotfromstaking" => LedgerSubtype::SpotFromStaking,
            "spottofutures" => LedgerSubtype::SpotToFutures,
            "spotfromfutures" => LedgerSubtype::SpotFromFutures,
            _ => LedgerSubtype::Other,
        })
    }

    /// Returns `true` for subtypes that represent internal balance movements
    /// (spot <-> staking, spot <-> futures) which should be skipped.
    fn is_internal_transfer(&self) -> bool {
        matches!(
            self,
            LedgerSubtype::Allocation
                | LedgerSubtype::Deallocation
                | LedgerSubtype::Autoallocate
                | LedgerSubtype::Migration
                | LedgerSubtype::SpotToStaking
                | LedgerSubtype::StakingFromSpot
                | LedgerSubtype::StakingToSpot
                | LedgerSubtype::SpotFromStaking
                | LedgerSubtype::SpotToFutures
                | LedgerSubtype::SpotFromFutures
        )
    }
}

/// A parsed row from the Kraken ledger CSV.
#[derive(Debug, Clone)]
struct LedgerRow {
    time: NaiveDateTime,
    ledger_type: LedgerType,
    /// Normalised ticker (e.g. `BTC` instead of `XXBT`).
    symbol: String,
    /// Signed amount: negative = debit, positive = credit.
    amount: f64,
    fee: f64,
    line_number: usize,
}

impl LedgerRow {
    fn is_incoming(&self) -> bool {
        self.amount >= 0.0
    }

    fn abs_amount(&self) -> f64 {
        self.amount.abs()
    }
}

/// Accumulates the two sides of a single Kraken trade (linked by `refid`).
///
/// A trade can manifest as:
/// - Two `trade` rows (one negative, one positive)
/// - A `spend` + `receive` pair
/// - Mixed combinations in edge cases
#[derive(Debug, Default)]
struct PendingTrade {
    spend: Option<LedgerRow>,
    receive: Option<LedgerRow>,
    trade_out: Option<LedgerRow>,
    trade_in: Option<LedgerRow>,
}

impl PendingTrade {
    fn insert(&mut self, row: LedgerRow) {
        match row.ledger_type {
            LedgerType::Spend => self.spend = Some(row),
            LedgerType::Receive => self.receive = Some(row),
            LedgerType::Trade | LedgerType::MarginTrade => {
                if row.is_incoming() {
                    self.trade_in = Some(row);
                } else {
                    self.trade_out = Some(row);
                }
            }
            _ => {}
        }
    }

    fn is_complete(&self) -> bool {
        (self.spend.is_some() && self.receive.is_some())
            || (self.trade_out.is_some() && self.trade_in.is_some())
    }

    /// Resolves the pending trade into zero or more `ImportCryptoTransaction`s.
    fn resolve(mut self, refid: &str, wallet_name: &str) -> Vec<ImportCryptoTransaction> {
        // Prefer trade_out/trade_in pair, fall back to spend/receive.
        // Use `.take()` so every field is consumed at most once,
        // keeping the fallback branch valid after earlier checks.
        let (outgoing, incoming) =
            if let (Some(out), Some(inc)) = (self.trade_out.take(), self.trade_in.take()) {
                (out, inc)
            } else if let (Some(spend), Some(recv)) = (self.spend.take(), self.receive.take()) {
                (spend, recv)
            } else {
                // Incomplete pair — emit single-sided transactions for whatever we have
                let mut results = Vec::new();
                for row in [self.spend, self.receive, self.trade_out, self.trade_in]
                    .into_iter()
                    .flatten()
                {
                    if let Some(tx) = single_row_to_transaction(&row, wallet_name, refid) {
                        results.push(tx);
                    }
                }
                return results;
            };

        let out_fiat = is_fiat(&outgoing.symbol);
        let in_fiat = is_fiat(&incoming.symbol);

        // Both fiat — skip entirely
        if out_fiat && in_fiat {
            return Vec::new();
        }

        let date = format_datetime(outgoing.time);
        let notes = Some(format!(
            "Kraken {} | Ref: {}",
            outgoing.ledger_type.label(),
            refid
        ));

        // Fiat -> Crypto = buy
        if out_fiat && !in_fiat {
            let price = if incoming.abs_amount() > 0.0 {
                Some(outgoing.abs_amount() / incoming.abs_amount())
            } else {
                None
            };

            let fee = if outgoing.fee.abs() > f64::EPSILON {
                Some(outgoing.fee.abs())
            } else if incoming.fee.abs() > f64::EPSILON {
                // Fee in crypto
                None // handled below
            } else {
                None
            };

            let mut tx = ImportCryptoTransaction {
                date,
                wallet: wallet_name.to_string(),
                symbol: incoming.symbol.clone(),
                transaction_type: "trade".to_string(),
                amount: incoming.abs_amount(),
                subtype: Some("buy".to_string()),
                price_per_coin: price,
                fee,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: None,
                swap_to_amount: None,
                fee_coin_symbol: None,
                fee_amount: None,
                notes,
            };

            // If the fee is in the crypto asset
            if incoming.fee.abs() > f64::EPSILON && fee.is_none() {
                tx.fee_coin_symbol = Some(incoming.symbol.clone());
                tx.fee_amount = Some(incoming.fee.abs());
            }

            return vec![tx];
        }

        // Crypto -> Fiat = sell
        if !out_fiat && in_fiat {
            let price = if outgoing.abs_amount() > 0.0 {
                Some(incoming.abs_amount() / outgoing.abs_amount())
            } else {
                None
            };

            let fee = if incoming.fee.abs() > f64::EPSILON {
                Some(incoming.fee.abs())
            } else {
                None
            };

            let mut tx = ImportCryptoTransaction {
                date,
                wallet: wallet_name.to_string(),
                symbol: outgoing.symbol.clone(),
                transaction_type: "trade".to_string(),
                amount: outgoing.abs_amount(),
                subtype: Some("sell".to_string()),
                price_per_coin: price,
                fee,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: None,
                swap_to_amount: None,
                fee_coin_symbol: None,
                fee_amount: None,
                notes,
            };

            // If the fee is in the sold crypto
            if outgoing.fee.abs() > f64::EPSILON && fee.is_none() {
                tx.fee_coin_symbol = Some(outgoing.symbol.clone());
                tx.fee_amount = Some(outgoing.fee.abs());
            }

            return vec![tx];
        }

        // Crypto -> Crypto = swap
        let fee_coin_symbol;
        let fee_amount;
        if outgoing.fee.abs() > f64::EPSILON {
            fee_coin_symbol = Some(outgoing.symbol.clone());
            fee_amount = Some(outgoing.fee.abs());
        } else if incoming.fee.abs() > f64::EPSILON {
            fee_coin_symbol = Some(incoming.symbol.clone());
            fee_amount = Some(incoming.fee.abs());
        } else {
            fee_coin_symbol = None;
            fee_amount = None;
        }

        let tx = ImportCryptoTransaction {
            date,
            wallet: wallet_name.to_string(),
            symbol: outgoing.symbol.clone(),
            transaction_type: "trade".to_string(),
            amount: outgoing.abs_amount(),
            subtype: Some("swap".to_string()),
            price_per_coin: None,
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: Some(incoming.symbol.clone()),
            swap_to_amount: Some(incoming.abs_amount()),
            fee_coin_symbol,
            fee_amount,
            notes,
        };

        vec![tx]
    }
}

/// Converts a single (unpaired) ledger row into an `ImportCryptoTransaction`.
/// Returns `None` for fiat-only rows or rows that should be skipped.
fn single_row_to_transaction(
    row: &LedgerRow,
    wallet_name: &str,
    refid: &str,
) -> Option<ImportCryptoTransaction> {
    if is_fiat(&row.symbol) {
        return None;
    }

    let date = format_datetime(row.time);
    let notes = Some(format!(
        "Kraken {} | Ref: {}",
        row.ledger_type.label(),
        refid
    ));

    let fee_coin_symbol = if row.fee.abs() > f64::EPSILON {
        Some(row.symbol.clone())
    } else {
        None
    };
    let fee_amount = if row.fee.abs() > f64::EPSILON {
        Some(row.fee.abs())
    } else {
        None
    };

    let (tx_type, subtype) = match row.ledger_type {
        LedgerType::Deposit | LedgerType::Receive | LedgerType::InviteBonus => {
            if row.is_incoming() {
                ("transfer".to_string(), Some("deposit".to_string()))
            } else {
                ("transfer".to_string(), Some("withdrawal".to_string()))
            }
        }
        LedgerType::Withdrawal | LedgerType::Spend => {
            ("transfer".to_string(), Some("withdrawal".to_string()))
        }
        LedgerType::Staking | LedgerType::Earn => {
            if row.is_incoming() {
                ("income".to_string(), Some("staking".to_string()))
            } else {
                // Unstaking / deallocation that wasn't filtered as internal
                ("transfer".to_string(), Some("withdrawal".to_string()))
            }
        }
        LedgerType::Transfer
        | LedgerType::Adjustment
        | LedgerType::Settled
        | LedgerType::Rollover
        | LedgerType::MarginTrade => {
            if row.is_incoming() {
                ("transfer".to_string(), Some("deposit".to_string()))
            } else {
                ("transfer".to_string(), Some("withdrawal".to_string()))
            }
        }
        _ => {
            if row.is_incoming() {
                ("income".to_string(), Some("other".to_string()))
            } else {
                ("expense".to_string(), Some("other".to_string()))
            }
        }
    };

    Some(ImportCryptoTransaction {
        date,
        wallet: wallet_name.to_string(),
        symbol: row.symbol.clone(),
        transaction_type: tx_type,
        amount: row.abs_amount(),
        subtype,
        price_per_coin: None,
        fee: None,
        override_proceeds: None,
        override_cost_basis: None,
        swap_to_symbol: None,
        swap_to_amount: None,
        fee_coin_symbol,
        fee_amount,
        notes,
    })
}

// ─── Column index resolution ────────────────────────────────────────────────

/// Resolves column indices from the CSV header, supporting both v1 and v2
/// Kraken ledger formats. Returns a mapping of logical field -> column index.
fn resolve_ledger_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"').to_lowercase();
        match key.as_str() {
            "txid" => {
                map.insert("txid", i);
            }
            "refid" => {
                map.insert("refid", i);
            }
            "time" => {
                map.insert("time", i);
            }
            "type" => {
                map.insert("type", i);
            }
            "subtype" => {
                map.insert("subtype", i);
            }
            "asset" => {
                map.insert("asset", i);
            }
            "amount" => {
                map.insert("amount", i);
            }
            "fee" => {
                map.insert("fee", i);
            }
            "balance" => {
                map.insert("balance", i);
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

// ═══════════════════════════════════════════════════════════════════════════
//  Kraken Ledger Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct KrakenLedgerParser;

impl ExchangeParser for KrakenLedgerParser {
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

        let cols = resolve_ledger_columns(&headers);

        // Validate required columns
        for required in &["txid", "refid", "time", "type", "asset", "amount", "fee"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required Kraken ledger column: '{}'", required),
                ));
            }
        }

        let mut result: ParseResult<ImportCryptoTransaction> = ParseResult::default();
        let mut pending: HashMap<String, PendingTrade> = HashMap::new();

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

            // Parse fields
            let refid = get_field(&record, &cols, "refid").to_string();
            let time_raw = get_field(&record, &cols, "time");
            let type_raw = get_field(&record, &cols, "type");
            let subtype_raw = get_field(&record, &cols, "subtype");
            let asset_raw = get_field(&record, &cols, "asset");
            let amount_raw = get_field(&record, &cols, "amount");
            let fee_raw = get_field(&record, &cols, "fee");

            // Parse timestamp
            let time = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("time"),
                        format!("Invalid timestamp: '{}'", time_raw),
                    ));
                    continue;
                }
            };

            let ledger_type = LedgerType::parse(type_raw);

            // Skip internal staking/futures transfers
            if LedgerSubtype::parse(subtype_raw)
                .as_ref()
                .is_some_and(|st| st.is_internal_transfer())
            {
                continue;
            }

            let symbol = normalize_kraken_currency(asset_raw).to_string();

            let amount = match parse_decimal(amount_raw) {
                Some(v) => v,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("amount"),
                        format!("Invalid amount: '{}'", amount_raw),
                    ));
                    continue;
                }
            };

            let fee = parse_decimal(fee_raw).unwrap_or(0.0);

            let row = LedgerRow {
                time,
                ledger_type,
                symbol,
                amount,
                fee,
                line_number,
            };

            // Determine whether this row participates in a paired trade
            let is_pairable = matches!(
                ledger_type,
                LedgerType::Trade
                    | LedgerType::MarginTrade
                    | LedgerType::Spend
                    | LedgerType::Receive
            );

            let has_refid = !refid.is_empty();

            if is_pairable && has_refid {
                let entry = pending.remove(&refid).unwrap_or_default();
                let mut entry = entry;
                let entry_line = row.line_number;
                entry.insert(row);

                if entry.is_complete() {
                    let txs = entry.resolve(&refid, wallet_name);
                    for tx in txs {
                        result.items.push((entry_line, tx));
                    }
                } else {
                    pending.insert(refid, entry);
                }
            } else {
                // Non-pairable row: emit directly as a single transaction
                if let Some(tx) = single_row_to_transaction(&row, wallet_name, &refid) {
                    result.items.push((line_number, tx));
                }
            }
        }

        // Drain remaining pending trades (incomplete pairs)
        for (refid, entry) in pending {
            let txs = entry.resolve(&refid, wallet_name);
            for tx in txs {
                // Use line 0 since we don't have a single authoritative line
                result.items.push((0, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::KrakenLedger
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Kraken Trades Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct KrakenTradesParser;

/// Resolves column indices for the Kraken trades CSV.
fn resolve_trades_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"').to_lowercase();
        match key.as_str() {
            "txid" => {
                map.insert("txid", i);
            }
            "ordertxid" => {
                map.insert("ordertxid", i);
            }
            "pair" => {
                map.insert("pair", i);
            }
            "time" => {
                map.insert("time", i);
            }
            "type" => {
                map.insert("type", i);
            }
            "ordertype" => {
                map.insert("ordertype", i);
            }
            "price" => {
                map.insert("price", i);
            }
            "cost" => {
                map.insert("cost", i);
            }
            "fee" => {
                map.insert("fee", i);
            }
            "vol" => {
                map.insert("vol", i);
            }
            "margin" => {
                map.insert("margin", i);
            }
            _ => {}
        }
    }
    map
}

impl ExchangeParser for KrakenTradesParser {
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

        let cols = resolve_trades_columns(&headers);

        for required in &["pair", "time", "type", "cost", "fee", "vol"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required Kraken trades column: '{}'", required),
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

            let pair_raw = get_field(&record, &cols, "pair");
            let time_raw = get_field(&record, &cols, "time");
            let type_raw = get_field(&record, &cols, "type");
            let cost_raw = get_field(&record, &cols, "cost");
            let fee_raw = get_field(&record, &cols, "fee");
            let vol_raw = get_field(&record, &cols, "vol");
            let ordertxid = get_field(&record, &cols, "ordertxid").to_string();

            let time = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("time"),
                        format!("Invalid timestamp: '{}'", time_raw),
                    ));
                    continue;
                }
            };

            let (base, quote) = match parse_kraken_pair(pair_raw) {
                Some(pair) => pair,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("pair"),
                        format!("Cannot parse pair: '{}'", pair_raw),
                    ));
                    continue;
                }
            };

            let volume = match parse_decimal(vol_raw) {
                Some(v) if v > 0.0 => v,
                _ => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("vol"),
                        format!("Invalid volume: '{}'", vol_raw),
                    ));
                    continue;
                }
            };

            let cost = parse_decimal(cost_raw).unwrap_or(0.0);
            let fee = parse_decimal(fee_raw).unwrap_or(0.0);

            let is_buy = type_raw.eq_ignore_ascii_case("buy");
            let side = if is_buy { "buy" } else { "sell" };
            let date = format_datetime(time);

            let base_fiat = is_fiat(&base);
            let quote_fiat = is_fiat(&quote);

            let notes = if ordertxid.is_empty() {
                Some(format!("Kraken trade | {}", pair_raw))
            } else {
                Some(format!(
                    "Kraken trade | {} | Order: {}",
                    pair_raw, ordertxid
                ))
            };

            // Determine the crypto asset, subtype, and amounts
            if base_fiat && quote_fiat {
                // Fiat-to-fiat trade — skip
                continue;
            }

            if quote_fiat {
                // Standard pair like BTC/USD
                let price = if volume > 0.0 {
                    Some(cost / volume)
                } else {
                    None
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: base.clone(),
                    transaction_type: "trade".to_string(),
                    amount: volume,
                    subtype: Some(side.to_string()),
                    price_per_coin: price,
                    fee: if fee.abs() > f64::EPSILON {
                        Some(fee)
                    } else {
                        None
                    },
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                result.items.push((line_number, tx));
            } else if base_fiat {
                // Inverted pair like USD/BTC (rare but possible)
                let subtype_str = if is_buy { "sell" } else { "buy" };
                let price = if cost > 0.0 {
                    Some(volume / cost)
                } else {
                    None
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: quote.clone(),
                    transaction_type: "trade".to_string(),
                    amount: cost,
                    subtype: Some(subtype_str.to_string()),
                    price_per_coin: price,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: Some(quote.clone()),
                    fee_amount: if fee.abs() > f64::EPSILON {
                        Some(fee)
                    } else {
                        None
                    },
                    notes,
                };

                result.items.push((line_number, tx));
            } else {
                // Crypto-to-crypto pair — swap
                let (from_symbol, from_amount, to_symbol, to_amount) = if is_buy {
                    // Buying base with quote: outgoing=quote, incoming=base
                    (quote.clone(), cost, base.clone(), volume)
                } else {
                    // Selling base for quote: outgoing=base, incoming=quote
                    (base.clone(), volume, quote.clone(), cost)
                };

                let fee_coin_symbol = if fee.abs() > f64::EPSILON {
                    Some(quote.clone())
                } else {
                    None
                };
                let fee_amount = if fee.abs() > f64::EPSILON {
                    Some(fee)
                } else {
                    None
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: from_symbol,
                    transaction_type: "trade".to_string(),
                    amount: from_amount,
                    subtype: Some("swap".to_string()),
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: Some(to_symbol),
                    swap_to_amount: Some(to_amount),
                    fee_coin_symbol,
                    fee_amount,
                    notes,
                };

                result.items.push((line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::KrakenTrades
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Ledger parser ──

    #[test]
    fn ledger_deposit_becomes_transfer_deposit() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX1\",\"REF1\",\"2024-01-15 10:00:00\",\"deposit\",\"\",\"currency\",\"XXBT\",\"0.5\",\"0\",\"0.5\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn ledger_withdrawal_becomes_transfer_withdrawal() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX1\",\"REF1\",\"2024-01-15 10:00:00\",\"withdrawal\",\"\",\"currency\",\"XETH\",\"-1.5\",\"0.001\",\"0\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
        assert!((tx.amount - 1.5).abs() < f64::EPSILON);
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("ETH"));
        assert!((tx.fee_amount.unwrap() - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn ledger_trade_pair_becomes_buy() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX-OUT\",\"REF-TRADE\",\"2024-01-15 10:00:00\",\"trade\",\"\",\"currency\",\"ZUSD\",\"-25000\",\"5.00\",\"0\"\n",
            "\"TX-IN\",\"REF-TRADE\",\"2024-01-15 10:00:00\",\"trade\",\"\",\"currency\",\"XXBT\",\"0.5\",\"0\",\"0.5\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
        // Price = 25000 / 0.5 = 50000
        assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < 0.01);
        // Fee in USD
        assert!((tx.fee.unwrap() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ledger_trade_pair_becomes_sell() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX-OUT\",\"REF-SELL\",\"2024-02-01 08:00:00\",\"trade\",\"\",\"currency\",\"XETH\",\"-2.0\",\"0\",\"0\"\n",
            "\"TX-IN\",\"REF-SELL\",\"2024-02-01 08:00:00\",\"trade\",\"\",\"currency\",\"ZUSD\",\"4000\",\"3.50\",\"4000\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 2.0).abs() < f64::EPSILON);
        // Price = 4000 / 2 = 2000
        assert!((tx.price_per_coin.unwrap() - 2000.0).abs() < 0.01);
        assert!((tx.fee.unwrap() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn ledger_crypto_to_crypto_becomes_swap() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX-OUT\",\"REF-SWAP\",\"2024-03-10 12:00:00\",\"trade\",\"\",\"currency\",\"XETH\",\"-5.0\",\"0.01\",\"0\"\n",
            "\"TX-IN\",\"REF-SWAP\",\"2024-03-10 12:00:00\",\"trade\",\"\",\"currency\",\"XXBT\",\"0.25\",\"0\",\"0.25\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert!((tx.amount - 5.0).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("BTC"));
        assert!((tx.swap_to_amount.unwrap() - 0.25).abs() < f64::EPSILON);
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("ETH"));
        assert!((tx.fee_amount.unwrap() - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn ledger_spend_receive_pair_becomes_trade() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX-SPEND\",\"REF-SR\",\"2024-05-10 12:00:00\",\"spend\",\"\",\"currency\",\"ZUSD\",\"-250.00\",\"2.50\",\"0\"\n",
            "\"TX-RECV\",\"REF-SR\",\"2024-05-10 12:00:00\",\"receive\",\"\",\"currency\",\"SOL\",\"1.25\",\"0\",\"1.25\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "SOL");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 1.25).abs() < f64::EPSILON);
        // Price = 250 / 1.25 = 200
        assert!((tx.price_per_coin.unwrap() - 200.0).abs() < 0.01);
        assert!((tx.fee.unwrap() - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn ledger_staking_reward_becomes_income() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX-STK\",\"REF-STK\",\"2024-01-20 00:00:00\",\"staking\",\"reward\",\"currency\",\"DOT.S\",\"0.05\",\"0\",\"10.05\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "DOT");
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("staking"));
        assert!((tx.amount - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn ledger_internal_staking_transfer_is_skipped() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX-A\",\"REF-INT\",\"2024-01-10 00:00:00\",\"transfer\",\"spottostaking\",\"currency\",\"DOT\",\"-10\",\"0\",\"0\"\n",
            "\"TX-B\",\"REF-INT\",\"2024-01-10 00:00:00\",\"transfer\",\"stakingfromspot\",\"currency\",\"DOT.S\",\"10\",\"0\",\"10\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert!(result.items.is_empty());
    }

    #[test]
    fn ledger_fiat_deposit_is_skipped() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX1\",\"REF1\",\"2024-01-15 10:00:00\",\"deposit\",\"\",\"currency\",\"ZUSD\",\"10000\",\"0\",\"10000\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        // Fiat rows produce None from single_row_to_transaction
        assert!(result.items.is_empty());
    }

    #[test]
    fn ledger_fiat_trade_pair_is_skipped() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX-A\",\"REF-FIAT\",\"2024-01-15 10:00:00\",\"trade\",\"\",\"currency\",\"ZUSD\",\"-1000\",\"0\",\"0\"\n",
            "\"TX-B\",\"REF-FIAT\",\"2024-01-15 10:00:00\",\"trade\",\"\",\"currency\",\"ZEUR\",\"920\",\"0\",\"920\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert!(result.items.is_empty());
    }

    #[test]
    fn ledger_v2_format_with_subclass_and_wallet() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"subclass\",\"asset\",\"wallet\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX1\",\"REF1\",\"2024-06-01 09:00:00\",\"deposit\",\"\",\"currency\",\"\",\"XXBT\",\"spot / main\",\"1.0\",\"0\",\"1.0\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
    }

    #[test]
    fn ledger_earn_reward_becomes_income_staking() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX-EARN\",\"REF-EARN\",\"2024-04-01 00:00:00\",\"earn\",\"reward\",\"currency\",\"SOL\",\"0.1\",\"0\",\"5.1\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "SOL");
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("staking"));
    }

    #[test]
    fn ledger_uses_custom_wallet_name() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX1\",\"REF1\",\"2024-01-15 10:00:00\",\"deposit\",\"\",\"currency\",\"XXBT\",\"0.5\",\"0\",\"0.5\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "My Kraken Account").unwrap();

        assert_eq!(result.items[0].1.wallet, "My Kraken Account");
    }

    // ── Trades parser ──

    #[test]
    fn trades_buy_btc_usd() {
        let csv = concat!(
            "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
            "\"TX1\",\"ORD1\",\"BTC/USD\",\"2024-01-15 10:30:45\",\"buy\",\"limit\",\"50000.00\",\"25000.00\",\"5.00\",\"0.5\",\"0\",\"\",\"L1,L2\"\n",
        );

        let parser = KrakenTradesParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < 0.01);
        assert!((tx.fee.unwrap() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn trades_sell_eth_eur() {
        let csv = concat!(
            "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
            "\"TX1\",\"ORD1\",\"ETH/EUR\",\"2024-02-01 08:00:00\",\"sell\",\"market\",\"2000.00\",\"4000.00\",\"3.50\",\"2.0\",\"0\",\"\",\"L1,L2\"\n",
        );

        let parser = KrakenTradesParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 2.0).abs() < f64::EPSILON);
        assert!((tx.fee.unwrap() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn trades_crypto_to_crypto_becomes_swap() {
        let csv = concat!(
            "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
            "\"TX1\",\"ORD1\",\"ETH/BTC\",\"2024-03-15 14:00:00\",\"buy\",\"limit\",\"0.05\",\"0.25\",\"0.0001\",\"5.0\",\"0\",\"\",\"L1,L2\"\n",
        );

        let parser = KrakenTradesParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        // Buying ETH with BTC: outgoing=BTC (cost=0.25), incoming=ETH (vol=5.0)
        assert_eq!(tx.symbol, "BTC");
        assert!((tx.amount - 0.25).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
        assert!((tx.swap_to_amount.unwrap() - 5.0).abs() < f64::EPSILON);
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("BTC"));
        assert!((tx.fee_amount.unwrap() - 0.0001).abs() < f64::EPSILON);
    }

    #[test]
    fn trades_with_old_concatenated_pair() {
        let csv = concat!(
            "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
            "\"TX1\",\"ORD1\",\"XXBTZUSD\",\"2024-01-02 03:04:05.1234\",\"buy\",\"limit\",\"50000.00\",\"25000.00\",\"5.00\",\"0.5\",\"0\",\"\",\"L1,L2\"\n",
        );

        let parser = KrakenTradesParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
    }

    #[test]
    fn trades_uses_custom_wallet_name() {
        let csv = concat!(
            "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
            "\"TX1\",\"ORD1\",\"BTC/USD\",\"2024-01-15 10:00:00\",\"buy\",\"limit\",\"50000\",\"5000\",\"1\",\"0.1\",\"0\",\"\",\"\"\n",
        );

        let parser = KrakenTradesParser;
        let result = parser.parse(csv, "My Kraken").unwrap();

        assert_eq!(result.items[0].1.wallet, "My Kraken");
    }

    #[test]
    fn trades_zero_volume_is_error() {
        let csv = concat!(
            "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
            "\"TX1\",\"ORD1\",\"BTC/USD\",\"2024-01-15 10:00:00\",\"buy\",\"limit\",\"50000\",\"0\",\"0\",\"0\",\"0\",\"\",\"\"\n",
        );

        let parser = KrakenTradesParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn trades_with_usdt_pair() {
        let csv = concat!(
            "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
            "\"TX1\",\"ORD1\",\"BTCUSDT\",\"2024-01-15 10:00:00\",\"buy\",\"limit\",\"50000\",\"5000\",\"1\",\"0.1\",\"0\",\"\",\"\"\n",
        );

        let parser = KrakenTradesParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        // USDT is not fiat, so this should be a swap
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
    }

    // ── Edge cases ──

    #[test]
    fn ledger_handles_empty_content() {
        let csv = "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n";

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();
        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn ledger_invite_bonus_becomes_deposit() {
        let csv = concat!(
            "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
            "\"TX1\",\"REF1\",\"2024-06-01 00:00:00\",\"invite bonus\",\"\",\"currency\",\"XXBT\",\"0.001\",\"0\",\"0.001\"\n",
        );

        let parser = KrakenLedgerParser;
        let result = parser.parse(csv, "Kraken").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
    }
}
