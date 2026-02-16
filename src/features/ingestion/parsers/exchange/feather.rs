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

use super::common::{format_date, non_empty, parse_decimal, parse_timestamp};
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

            let date = format_date(timestamp);

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

            // Parse amount (always positive in the CSV)
            let amount = match parse_decimal(amount_raw) {
                Some(v) if v > 0.0 => v,
                Some(v) if v < 0.0 => v.abs(), // handle negative just in case
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
            // Include fiat valuation in notes if present
            if let Some(fiat_val) = non_empty(fiat_amount) {
                let currency = non_empty(fiat_currency).unwrap_or("USD");
                notes_parts.push(format!("Fiat: {} {}", fiat_val, currency));
            }

            let notes = Some(notes_parts.join(" | "));

            // Map to Sanctum type/subtype
            let (tx_type, subtype) = if is_incoming {
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
mod tests {
    use super::*;

    fn sample_csv() -> &'static str {
        concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,in,0.500000000000,0.500000000000,0.000000000000,abc123def456abc123def456abc123def456abc123def456abc123def456abcd,,,82.50,USD\n",
            "3060000,1705398645,2024-01-16 10:30:45,0,out,-0.100030000000,0.100000000000,0.000030000000,def789abc012def789abc012def789abc012def789abc012def789abc012defg,Payment to Alice,,16.50,USD\n",
        )
    }

    #[test]
    fn incoming_becomes_transfer_deposit() {
        let parser = FeatherParser;
        let result = parser.parse(sample_csv(), "Feather").unwrap();

        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "XMR");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
        assert_eq!(tx.wallet, "Feather");
        // No fee for incoming
        assert!(tx.fee_coin_symbol.is_none());
        assert!(tx.fee_amount.is_none());
    }

    #[test]
    fn outgoing_becomes_transfer_withdrawal() {
        let parser = FeatherParser;
        let result = parser.parse(sample_csv(), "Feather").unwrap();

        let tx = &result.items[1].1;
        assert_eq!(tx.symbol, "XMR");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
        assert!((tx.amount - 0.1).abs() < f64::EPSILON);
        // Fee should be present for outgoing
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("XMR"));
        assert!((tx.fee_amount.unwrap() - 0.00003).abs() < f64::EPSILON);
    }

    #[test]
    fn notes_contain_description_and_truncated_txid() {
        let parser = FeatherParser;
        let result = parser.parse(sample_csv(), "Feather").unwrap();

        // Second tx has a description "Payment to Alice"
        let notes = result.items[1].1.notes.as_deref().unwrap();
        assert!(notes.contains("Feather Wallet"));
        assert!(notes.contains("Desc: Payment to Alice"));
        assert!(notes.contains("TxID: def789ab...c012defg"));
        assert!(notes.contains("Block: 3060000"));
    }

    #[test]
    fn notes_without_description() {
        let parser = FeatherParser;
        let result = parser.parse(sample_csv(), "Feather").unwrap();

        // First tx has no description
        let notes = result.items[0].1.notes.as_deref().unwrap();
        assert!(notes.contains("Feather Wallet"));
        assert!(!notes.contains("Desc:"));
        assert!(notes.contains("TxID:"));
    }

    #[test]
    fn notes_contain_fiat_valuation() {
        let parser = FeatherParser;
        let result = parser.parse(sample_csv(), "Feather").unwrap();

        let notes = result.items[0].1.notes.as_deref().unwrap();
        assert!(notes.contains("Fiat: 82.50 USD"));

        let notes = result.items[1].1.notes.as_deref().unwrap();
        assert!(notes.contains("Fiat: 16.50 USD"));
    }

    #[test]
    fn uses_custom_wallet_name() {
        let parser = FeatherParser;
        let result = parser.parse(sample_csv(), "Mi Monero").unwrap();

        assert_eq!(result.items[0].1.wallet, "Mi Monero");
        assert_eq!(result.items[1].1.wallet, "Mi Monero");
    }

    #[test]
    fn date_format_is_iso() {
        let parser = FeatherParser;
        let result = parser.parse(sample_csv(), "Feather").unwrap();

        assert_eq!(result.items[0].1.date, "2024-01-15");
        assert_eq!(result.items[1].1.date, "2024-01-16");
    }

    #[test]
    fn empty_content_produces_no_items() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn invalid_direction_produces_error() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,sideways,0.5,0.5,0,abc,,,,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("sideways"));
    }

    #[test]
    fn invalid_amount_produces_error() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,in,INVALID,INVALID,0,abc,,,,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn zero_amount_produces_error() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,in,0.0,0.0,0,abc,,,,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn missing_required_column_is_fatal_error() {
        // Missing "date" column
        let csv = "blockHeight,timestamp,direction,amount,fee,txid\n";

        let parser = FeatherParser;
        let err = parser.parse(csv, "Feather").unwrap_err();

        assert!(err.message.contains("date"));
    }

    #[test]
    fn timestamp_fallback_when_date_invalid() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,INVALID_DATE,0,in,0.5,0.5,0,abc,,,,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        // Should fall back to timestamp column
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].1.date, "2024-01-15");
    }

    #[test]
    fn negative_amount_is_handled() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,out,-0.5,-0.5,0.00003,abc,,,,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn incoming_with_zero_fee_has_no_fee_fields() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,in,1.0,1.0,0,abc,,,,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        let tx = &result.items[0].1;
        assert!(tx.fee_coin_symbol.is_none());
        assert!(tx.fee_amount.is_none());
    }

    #[test]
    fn multiple_transactions_parsed() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,in,1.0,1.0,0,abc1,,,165.00,USD\n",
            "3050001,1705312300,2024-01-15 10:31:40,0,out,-0.500030000000,0.5,0.00003,abc2,test,,82.50,USD\n",
            "3060000,1705398645,2024-01-16 10:30:45,0,in,2.5,2.5,0,abc3,mining,,412.50,EUR\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].1.subtype.as_deref(), Some("deposit"));
        assert_eq!(result.items[1].1.subtype.as_deref(), Some("withdrawal"));
        assert_eq!(result.items[2].1.subtype.as_deref(), Some("deposit"));
    }

    #[test]
    fn source_returns_feather_wallet() {
        let parser = FeatherParser;
        assert_eq!(parser.source(), ExchangeSource::FeatherWallet);
    }

    #[test]
    fn legacy_header_still_works() {
        // Ensure backward compatibility with the old header format
        let csv = concat!(
            "blockheight,epoch,date,direction,amount,fee,txid,address,description,paymentid\n",
            "3050000,1705312245,2024-01-15 10:30:45,in,0.5,0,abc,addr,,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].1.symbol, "XMR");
        assert_eq!(result.items[0].1.subtype.as_deref(), Some("deposit"));
    }

    #[test]
    fn fiat_without_currency_defaults_to_usd() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,in,0.5,0.5,0,abc,,,100.00,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        let notes = result.items[0].1.notes.as_deref().unwrap();
        assert!(notes.contains("Fiat: 100.00 USD"));
    }

    #[test]
    fn no_fiat_when_fiat_amount_empty() {
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "3050000,1705312245,2024-01-15 10:30:45,0,in,0.5,0.5,0,abc,,,,\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        let notes = result.items[0].1.notes.as_deref().unwrap();
        assert!(!notes.contains("Fiat:"));
    }

    #[test]
    fn real_feather_export_with_quoted_iso8601z_dates() {
        // Exact format from a real Feather Wallet export: quoted fields,
        // ISO 8601 dates with trailing Z, and an invalid date row that
        // must fall back to the timestamp (Unix epoch) column.
        let csv = concat!(
            "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
            "1111111,1761878329,\"2020-11-30T21:21:10Z\",0,\"in\",0.004450100000,0.034450000000,0.000000880000,\"4389432db3234234sjdhf32hjldsf5140ba29cc49234234fbdcd82b37c5957cceed\",\"\",\"\",\"10.68\",\"USD\"\n",
            "2222222,1761823846,\"2020-13-30T29:59:42Z\",0,\"out\",-0.014450000000,0.034419280000,0.000000000000,\"9e2353234234232342342349515b457c01155db5fc36ac67233bbd207c5367\",\"\",\"\",\"10.63\",\"USD\"\n",
        );

        let parser = FeatherParser;
        let result = parser.parse(csv, "Feather").unwrap();

        // Both rows should parse (second falls back to timestamp column)
        assert_eq!(result.items.len(), 2);
        assert!(result.errors.is_empty());

        // Row 1: valid ISO 8601 date with Z suffix
        let tx1 = &result.items[0].1;
        assert_eq!(tx1.date, "2020-11-30");
        assert_eq!(tx1.transaction_type, "transfer");
        assert_eq!(tx1.subtype.as_deref(), Some("deposit"));
        assert!((tx1.amount - 0.034450).abs() < 1e-6);
        assert_eq!(tx1.symbol, "XMR");
        // Fee present (0.000000880000)
        assert_eq!(tx1.fee_coin_symbol.as_deref(), Some("XMR"));
        assert!((tx1.fee_amount.unwrap() - 0.00000088).abs() < 1e-12);
        // Fiat valuation in notes
        let notes1 = tx1.notes.as_deref().unwrap();
        assert!(notes1.contains("Fiat: 10.68 USD"));
        assert!(notes1.contains("Block: 1111111"));

        // Row 2: invalid date (month 13, hour 29) => falls back to timestamp 1761823846
        let tx2 = &result.items[1].1;
        // timestamp 1761823846 = 2025-10-30 (approximately)
        assert!(!tx2.date.is_empty());
        assert_eq!(tx2.transaction_type, "transfer");
        assert_eq!(tx2.subtype.as_deref(), Some("withdrawal"));
        assert!((tx2.amount - 0.03441928).abs() < 1e-8);
        // Fee is zero => no fee fields
        assert!(tx2.fee_coin_symbol.is_none());
        assert!(tx2.fee_amount.is_none());
        let notes2 = tx2.notes.as_deref().unwrap();
        assert!(notes2.contains("Fiat: 10.63 USD"));
        assert!(notes2.contains("Block: 2222222"));
    }
}
