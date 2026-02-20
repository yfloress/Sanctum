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

//! Monero GUI Wallet CSV parser (Monero / XMR)
//!
//! The official Monero GUI wallet exports transaction history as CSV with the
//! following columns:
//!
//! ```text
//! blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description
//! ```
//!
//! - `direction` is either `"in"` or `"out"`.
//! - `amount` is in XMR (decimal). Value is absolute.
//! - `atomicAmount` is the amount in piconeros (integer).
//! - `fee` is the network fee in XMR. For incoming transactions, the fee was
//!   paid by the sender.
//! - `epoch` is the Unix timestamp (seconds) of the block.
//! - `date` is a human-readable datetime string (`YYYY-MM-DD HH:MM:SS`).
//! - `txid` is the Monero transaction hash.
//! - `label` is the address label (may be empty or `""`).
//! - `subaddrAccount` is the subaddress account index (usually 0).
//! - `paymentId` is the Monero payment ID (usually empty).
//! - `description` is the user-assigned transaction note (may be empty or `""`).
//!
//! Churn transactions (self-sends) appear as `direction = "out"` with
//! `amount = 0` — these are mapped to `transfer/churn`.
//!
//! All transactions are mapped to the `XMR` symbol. Incoming transactions
//! become `transfer/deposit`, outgoing become `transfer/withdrawal`.
//! The fee (when present) is recorded as `fee_coin_symbol = "XMR"`.

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::common::{format_datetime, non_empty, parse_decimal, parse_timestamp};
use super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

/// The symbol used for all Monero GUI Wallet transactions.
const XMR_SYMBOL: &str = "XMR";

// ─── Column index resolution ────────────────────────────────────────────────

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"').to_lowercase();
        match key.as_str() {
            "blockheight" => {
                map.insert("blockheight", i);
            }
            "epoch" => {
                map.insert("epoch", i);
            }
            "date" => {
                map.insert("date", i);
            }
            "direction" => {
                map.insert("direction", i);
            }
            "amount" => {
                map.insert("amount", i);
            }
            "atomicamount" => {
                map.insert("atomicamount", i);
            }
            "fee" => {
                map.insert("fee", i);
            }
            "txid" => {
                map.insert("txid", i);
            }
            "label" => {
                map.insert("label", i);
            }
            "subaddraccount" => {
                map.insert("subaddraccount", i);
            }
            "paymentid" => {
                map.insert("paymentid", i);
            }
            "description" => {
                map.insert("description", i);
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
//  Monero GUI Wallet Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct MoneroGuiParser;

impl ExchangeParser for MoneroGuiParser {
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
                    format!("Missing required Monero GUI Wallet column: '{}'", required),
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
            let label = get_field(&record, &cols, "label");
            let description = get_field(&record, &cols, "description");
            let blockheight = get_field(&record, &cols, "blockheight");

            // Skip empty rows
            if date_raw.is_empty() && direction_raw.is_empty() && amount_raw.is_empty() {
                continue;
            }

            // Parse date — Monero GUI uses "YYYY-MM-DD HH:MM:SS" format
            let timestamp = match parse_timestamp(date_raw) {
                Some(dt) => dt,
                None => {
                    // Try using epoch column as fallback (Unix timestamp)
                    let epoch_raw = get_field(&record, &cols, "epoch");
                    match parse_decimal(epoch_raw) {
                        Some(epoch) if epoch > 0.0 => {
                            match chrono::DateTime::from_timestamp(epoch as i64, 0) {
                                Some(dt) => dt.naive_utc(),
                                None => {
                                    result.errors.push(RowError::new(
                                        line_number,
                                        Some("date"),
                                        format!(
                                            "Invalid date '{}' and epoch '{}'",
                                            date_raw, epoch_raw
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

            // Parse amount (always positive / absolute in the CSV).
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

            // Parse fee (may be 0 or absent)
            let fee_value = parse_decimal(fee_raw).unwrap_or(0.0);
            let (fee_coin_symbol, fee_amount) = if fee_value.abs() > f64::EPSILON {
                (Some(XMR_SYMBOL.to_string()), Some(fee_value.abs()))
            } else {
                (None, None)
            };

            // Build notes from available metadata
            let mut notes_parts: Vec<String> = Vec::new();
            notes_parts.push("Monero GUI".to_string());

            // Description takes priority, then label
            if let Some(desc) = non_empty(description) {
                notes_parts.push(format!("Desc: {}", desc));
            }
            if let Some(lbl) = non_empty(label) {
                notes_parts.push(format!("Label: {}", lbl));
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
                symbol: XMR_SYMBOL.to_string(),
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
        ExchangeSource::MoneroGuiWallet
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════


#[cfg(test)]
mod tests;
