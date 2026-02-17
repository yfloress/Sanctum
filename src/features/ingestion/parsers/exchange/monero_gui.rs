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

use super::common::{format_date, non_empty, parse_decimal, parse_timestamp};
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
mod tests {
    use super::*;

    fn sample_csv() -> &'static str {
        concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "1111111,1610274075,2021-01-10 10:20:15,in,0.030000000000,30000000000,0.000000000000,abc123def456abc123def456abc123def456abc123def456abc123def456abcd,\"\",0,,\"\"\n",
            "2222222,1634753387,2021-10-20 20:29:47,out,0.100000000000,100000000000,0.000040000000,def789abc012def789abc012def789abc012def789abc012def789abc012defg,\"My Label\",0,,\"Payment for services\"\n",
        )
    }

    #[test]
    fn incoming_becomes_transfer_deposit() {
        let parser = MoneroGuiParser;
        let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "XMR");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
        assert!((tx.amount - 0.03).abs() < f64::EPSILON);
        assert_eq!(tx.wallet, "Monero GUI");
        // No fee for incoming (fee = 0)
        assert!(tx.fee_coin_symbol.is_none());
        assert!(tx.fee_amount.is_none());
    }

    #[test]
    fn outgoing_becomes_transfer_withdrawal() {
        let parser = MoneroGuiParser;
        let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

        let tx = &result.items[1].1;
        assert_eq!(tx.symbol, "XMR");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
        assert!((tx.amount - 0.1).abs() < f64::EPSILON);
        // Fee should be present for outgoing
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("XMR"));
        assert!((tx.fee_amount.unwrap() - 0.00004).abs() < f64::EPSILON);
    }

    #[test]
    fn notes_contain_description_label_and_truncated_txid() {
        let parser = MoneroGuiParser;
        let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

        // Second tx has description and label
        let notes = result.items[1].1.notes.as_deref().unwrap();
        assert!(notes.contains("Monero GUI"));
        assert!(notes.contains("Desc: Payment for services"));
        assert!(notes.contains("Label: My Label"));
        assert!(notes.contains("TxID: def789ab...c012defg"));
        assert!(notes.contains("Block: 2222222"));
    }

    #[test]
    fn notes_without_description_or_label() {
        let parser = MoneroGuiParser;
        let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

        // First tx has empty description and label
        let notes = result.items[0].1.notes.as_deref().unwrap();
        assert!(notes.contains("Monero GUI"));
        assert!(!notes.contains("Desc:"));
        assert!(!notes.contains("Label:"));
        assert!(notes.contains("TxID:"));
    }

    #[test]
    fn uses_custom_wallet_name() {
        let parser = MoneroGuiParser;
        let result = parser.parse(sample_csv(), "Mi Monero").unwrap();

        assert_eq!(result.items[0].1.wallet, "Mi Monero");
        assert_eq!(result.items[1].1.wallet, "Mi Monero");
    }

    #[test]
    fn date_format_is_iso() {
        let parser = MoneroGuiParser;
        let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

        assert_eq!(result.items[0].1.date, "2021-01-10");
        assert_eq!(result.items[1].1.date, "2021-10-20");
    }

    #[test]
    fn empty_content_produces_no_items() {
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn invalid_direction_produces_error() {
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "3050000,1705312245,2024-01-15 10:30:45,sideways,0.5,500000000000,0,abc,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn invalid_amount_produces_error() {
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "3050000,1705312245,2024-01-15 10:30:45,in,INVALID,0,0,abc,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn zero_amount_incoming_is_skipped_silently() {
        // Zero amount for incoming is the receiving side of a churn/self-send;
        // it carries no value, so we skip it silently (no error, no item).
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "3050000,1705312245,2024-01-15 10:30:45,in,0.000000000000,0,0.00004,abc,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn churn_transaction_zero_amount_out() {
        // Churn = self-send: direction "out", amount 0, fee > 0
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "3050000,1705312245,2024-01-15 10:30:45,out,0.000000000000,0,0.000017740000,abc123,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());

        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("churn"));
        assert!((tx.amount - 0.0).abs() < f64::EPSILON);
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("XMR"));
        let notes = tx.notes.as_deref().unwrap();
        assert!(notes.contains("Churn"));
    }

    #[test]
    fn missing_required_column_is_fatal_error() {
        // Missing "date" column
        let csv = "blockHeight,epoch,direction,amount,fee,txid\n";

        let parser = MoneroGuiParser;
        let err = parser.parse(csv, "Monero GUI").unwrap_err();

        assert!(err.message.contains("date"));
    }

    #[test]
    fn epoch_fallback_when_date_invalid() {
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "3050000,1705312245,INVALID_DATE,in,0.5,500000000000,0,abc,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        // Should fall back to epoch timestamp
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].1.date, "2024-01-15");
    }

    #[test]
    fn negative_amount_is_handled() {
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "3050000,1705312245,2024-01-15 10:30:45,out,-0.5,500000000000,0.00003,abc,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn incoming_with_zero_fee_has_no_fee_fields() {
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "3050000,1705312245,2024-01-15 10:30:45,in,1.0,1000000000000,0,abc,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        let tx = &result.items[0].1;
        assert!(tx.fee_coin_symbol.is_none());
        assert!(tx.fee_amount.is_none());
    }

    #[test]
    fn multiple_transactions_parsed() {
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "3050000,1705312245,2024-01-15 10:30:45,in,1.0,1000000000000,0,abc1,\"\",0,,\"\"\n",
            "3050001,1705312300,2024-01-15 10:31:40,out,0.5,500000000000,0.00003,abc2,\"addr2\",0,,\"test\"\n",
            "3060000,1705398645,2024-01-16 10:30:45,in,2.5,2500000000000,0,abc3,\"\",0,,\"mining\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].1.subtype.as_deref(), Some("deposit"));
        assert_eq!(result.items[1].1.subtype.as_deref(), Some("withdrawal"));
        assert_eq!(result.items[2].1.subtype.as_deref(), Some("deposit"));
    }

    #[test]
    fn source_returns_monero_gui_wallet() {
        let parser = MoneroGuiParser;
        assert_eq!(parser.source(), ExchangeSource::MoneroGuiWallet);
    }

    #[test]
    fn real_monero_gui_export_with_quoted_fields() {
        // Simulates the exact format from a real Monero GUI export:
        // quoted empty strings, integer atomicAmount, etc.
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "1111111,2222222222,2021-01-10 10:20:15,in,0.030000000000,3000000000,0.000000000000,kjhsahf8923h98fh32fhoiuhsaf923hf98fjasdkjfk,\"\",0,,\"\"\n",
            "1111111,2222222222,2021-10-20 20:29:47,out,0.034419280000,34419280000,0.000040000000,9e2353234234232342342349515b457c01155db5fc36ac67233bbd207c5367,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(result.errors.is_empty());

        // Row 1: incoming
        let tx1 = &result.items[0].1;
        assert_eq!(tx1.date, "2021-01-10");
        assert_eq!(tx1.transaction_type, "transfer");
        assert_eq!(tx1.subtype.as_deref(), Some("deposit"));
        assert!((tx1.amount - 0.03).abs() < 1e-12);
        assert_eq!(tx1.symbol, "XMR");
        // Fee = 0 for incoming
        assert!(tx1.fee_coin_symbol.is_none());

        // Row 2: outgoing with fee
        let tx2 = &result.items[1].1;
        assert_eq!(tx2.date, "2021-10-20");
        assert_eq!(tx2.transaction_type, "transfer");
        assert_eq!(tx2.subtype.as_deref(), Some("withdrawal"));
        assert!((tx2.amount - 0.03441928).abs() < 1e-12);
        assert_eq!(tx2.fee_coin_symbol.as_deref(), Some("XMR"));
        assert!((tx2.fee_amount.unwrap() - 0.00004).abs() < 1e-12);
    }

    #[test]
    fn zero_amount_incoming_from_real_export() {
        // The user's real data had a row with amount=0 and direction=in.
        // This is the receiving side of a churn/self-send and should be skipped silently.
        let csv = concat!(
            "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
            "1111111,2222222222,2021-10-20 20:29:47,in,0.000000000000,0,0.0004000000,9e235abc,\"\",0,,\"\"\n",
        );

        let parser = MoneroGuiParser;
        let result = parser.parse(csv, "Monero GUI").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }
}
