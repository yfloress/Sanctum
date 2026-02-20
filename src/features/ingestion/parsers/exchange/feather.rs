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

//! Feather Wallet CSV parser (Monero / XMR)
//!
//! Feather is a desktop Monero wallet. Its CSV export contains one row per
//! on-chain transaction with the following columns:
//!
//! ```text
//! blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency
//! ```
//!
//! - `direction` is either `"in"` or `"out"`.
//! - `amount` and `fee` are in XMR (piconero-precision decimals).
//! - `fee` is only meaningful for outgoing transactions.
//! - `txid` is the Monero transaction hash.
//! - `description` is the user-assigned label (may be empty).
//! - `timestamp` is a Unix epoch (seconds) used as fallback when `date` is invalid.
//! - `balanceDelta` is the signed balance change (positive for in, negative for out).
//! - `fiatAmount` and `fiatCurrency` are optional fiat valuation at time of tx.
//! - `accountIndex` is the Monero account index (usually 0).
//! - `paymentId` is the Monero payment ID (usually empty).
//!
//! All transactions are mapped to the `XMR` symbol. Incoming transactions
//! become `transfer/deposit`, outgoing become `transfer/withdrawal`.
//! The fee (when present) is recorded as `fee_coin_symbol = "XMR"`.

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::common::{format_datetime, non_empty, parse_decimal, parse_timestamp};
use super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

/// The symbol used for all Feather Wallet transactions.
const FEATHER_SYMBOL: &str = "XMR";

// ─── Column index resolution ────────────────────────────────────────────────

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"').to_lowercase();
        match key.as_str() {
            "blockheight" => {
                map.insert("blockheight", i);
            }
            // Real Feather exports use "timestamp"; keep "epoch" as legacy alias
            "timestamp" | "epoch" => {
                map.insert("timestamp", i);
            }
            "date" => {
                map.insert("date", i);
            }
            "accountindex" => {
                map.insert("accountindex", i);
            }
            "direction" => {
                map.insert("direction", i);
            }
            "balancedelta" => {
                map.insert("balancedelta", i);
            }
            "amount" => {
                map.insert("amount", i);
            }
            "fee" => {
                map.insert("fee", i);
            }
            "txid" => {
                map.insert("txid", i);
            }
            "description" => {
                map.insert("description", i);
            }
            "paymentid" => {
                map.insert("paymentid", i);
            }
            "fiatamount" => {
                map.insert("fiatamount", i);
            }
            "fiatcurrency" => {
                map.insert("fiatcurrency", i);
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
//  Feather Wallet Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct FeatherParser;

impl ExchangeParser for FeatherParser {
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

        let cols = resolve_columns(&headers);

        // Validate required columns
        for required in &["date", "direction", "amount"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required Feather Wallet column: '{}'", required),
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
            let direction_raw = get_field(&record, &cols, "direction");
            let amount_raw = get_field(&record, &cols, "amount");
            let fee_raw = get_field(&record, &cols, "fee");
            let txid = get_field(&record, &cols, "txid");
            let description = get_field(&record, &cols, "description");
            let blockheight = get_field(&record, &cols, "blockheight");
            let fiat_amount = get_field(&record, &cols, "fiatamount");
            let fiat_currency = get_field(&record, &cols, "fiatcurrency");

            // Skip empty rows
            if date_raw.is_empty() && direction_raw.is_empty() && amount_raw.is_empty() {
                continue;
            }

            // Parse date — Feather can emit full datetime or just a date
            let timestamp = match parse_timestamp(date_raw) {
                Some(dt) => dt,
                None => {
                    // Try using timestamp column as fallback (Unix epoch seconds)
                    let ts_raw = get_field(&record, &cols, "timestamp");
                    match parse_decimal(ts_raw) {
                        Some(epoch) if epoch > 0.0 => {
                            match chrono::DateTime::from_timestamp(epoch as i64, 0) {
                                Some(dt) => dt.naive_utc(),
                                None => {
                                    result.errors.push(RowError::new(
                                        line_number,
                                        Some("date"),
                                        format!(
                                            "Invalid date '{}' and timestamp '{}'",
                                            date_raw, ts_raw
                                        ),
                                    ));
                                    continue;
                                }
                            }
                        }
                        _ => {
                            result.errors.push(RowError::new(
                                line_number,
                                Some("date"),
                                format!("Invalid date: '{}'", date_raw),
                            ));
                            continue;
                        }
                    }
                }
            };

            let date = format_datetime(timestamp);

            // Parse direction
            let direction = direction_raw.to_lowercase();
            let is_incoming = match direction.as_str() {
                "in" => true,
                "out" => false,
                other => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("direction"),
                        format!("Unknown direction '{}', expected 'in' or 'out'", other),
                    ));
                    continue;
                }
            };

            // Parse amount (always positive in the CSV).
            // Churn transactions (self-sends) have amount = 0.
            // - direction "out" + amount 0 → churn (record with fee only)
            // - direction "in"  + amount 0 → receiving side of churn (skip silently)
            let amount_parsed = parse_decimal(amount_raw);
            let is_zero = matches!(amount_parsed, Some(v) if v.abs() < f64::EPSILON);
            let is_churn_out = !is_incoming && is_zero;
            let is_churn_in = is_incoming && is_zero;

            // Incoming zero-amount is the receiving side of a self-send;
            // it carries no value and no fee, so we skip it silently.
            if is_churn_in {
                continue;
            }

            let amount = match amount_parsed {
                Some(v) if v > 0.0 => v,
                Some(v) if v < 0.0 => v.abs(), // handle negative just in case
                Some(_) if is_churn_out => 0.0, // churn tx: amount = 0 is valid
                _ => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("amount"),
                        format!("Invalid or zero amount: '{}'", amount_raw),
                    ));
                    continue;
                }
            };

            // Parse fee (only relevant for outgoing; may be 0 or absent)
            let fee_value = parse_decimal(fee_raw).unwrap_or(0.0);
            let (fee_coin_symbol, fee_amount) = if fee_value.abs() > f64::EPSILON {
                (Some(FEATHER_SYMBOL.to_string()), Some(fee_value.abs()))
            } else {
                (None, None)
            };

            // Build notes from available metadata
            let mut notes_parts: Vec<String> = Vec::new();
            notes_parts.push("Feather Wallet".to_string());

            if let Some(desc) = non_empty(description) {
                notes_parts.push(format!("Desc: {}", desc));
            }
            if let Some(hash) = non_empty(txid) {
                // Truncate long tx hashes for readability
                let display_hash = if hash.len() > 16 {
                    format!("{}...{}", &hash[..8], &hash[hash.len() - 8..])
                } else {
                    hash.to_string()
                };
                notes_parts.push(format!("TxID: {}", display_hash));
            }
            if let Some(bh) = non_empty(blockheight) {
                notes_parts.push(format!("Block: {}", bh));
            }
            // Include fiat valuation in notes if present.
            // Feather outputs "?" when the fiat value cannot be calculated.
            if let Some(fiat_val) = non_empty(fiat_amount) {
                if fiat_val != "?" {
                    let currency = non_empty(fiat_currency).unwrap_or("USD");
                    notes_parts.push(format!("Fiat: {} {}", fiat_val, currency));
                }
            }

            if is_churn_out {
                notes_parts.push("Churn".to_string());
            }

            let notes = Some(notes_parts.join(" | "));

            // Map to Sanctum type/subtype
            let (tx_type, subtype) = if is_churn_out {
                ("transfer".to_string(), Some("churn".to_string()))
            } else if is_incoming {
                ("transfer".to_string(), Some("deposit".to_string()))
            } else {
                ("transfer".to_string(), Some("withdrawal".to_string()))
            };

            let tx = ImportCryptoTransaction {
                date,
                wallet: wallet_name.to_string(),
                symbol: FEATHER_SYMBOL.to_string(),
                transaction_type: tx_type,
                amount,
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
            };

            result.items.push((line_number, tx));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::FeatherWallet
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════


#[cfg(test)]
mod tests;
