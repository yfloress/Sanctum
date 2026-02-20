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

//! MEXC statement-style CSV parser.
//!
//! Supports exports with the same columns used by:
//! - Earn Fixed
//! - Earn Flexible
//! - Spot Statement
//! - Futures Statement
//!
//! Header:
//! `UID,Creation Time(UTC+00:00),Crypto,Transaction Type,Direction,Quantity`

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::super::common::{format_datetime, is_fiat, parse_decimal, parse_timestamp};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcStatementParser;

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"');
        match key {
            "Creation Time(UTC+00:00)" => {
                map.insert("time", i);
            }
            "Crypto" => {
                map.insert("symbol", i);
            }
            "Transaction Type" => {
                map.insert("tx_type", i);
            }
            "Direction" => {
                map.insert("direction", i);
            }
            "Quantity" => {
                map.insert("quantity", i);
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

fn is_outflow(direction: &str) -> bool {
    let normalized = direction.trim().to_lowercase();
    normalized.contains("out") || normalized.contains("withdraw")
}

fn map_statement_kind(
    tx_type_raw: &str,
    direction_raw: &str,
    signed_quantity: f64,
) -> (String, Option<String>) {
    let tx_type = tx_type_raw.trim().to_lowercase();
    let outflow = is_outflow(direction_raw) || signed_quantity < 0.0;

    if tx_type.contains("interest") {
        return ("income".to_string(), Some("interest".to_string()));
    }
    if tx_type.contains("staking") {
        return ("income".to_string(), Some("staking".to_string()));
    }
    if tx_type.contains("airdrop") {
        return ("income".to_string(), Some("airdrop".to_string()));
    }
    if tx_type.contains("reward") || tx_type.contains("bonus") {
        return ("income".to_string(), Some("reward".to_string()));
    }
    if tx_type.contains("deposit") {
        return ("transfer".to_string(), Some("deposit".to_string()));
    }
    if tx_type.contains("withdraw") {
        return ("transfer".to_string(), Some("withdrawal".to_string()));
    }
    if tx_type.contains("fee") {
        return ("expense".to_string(), Some("fee".to_string()));
    }
    if tx_type.contains("transfer") {
        if outflow {
            return ("transfer".to_string(), Some("withdrawal".to_string()));
        }
        return ("transfer".to_string(), Some("deposit".to_string()));
    }

    if outflow {
        ("expense".to_string(), Some("other".to_string()))
    } else {
        ("income".to_string(), Some("other".to_string()))
    }
}

fn build_notes(tx_type: &str, direction: &str) -> Option<String> {
    let mut parts = vec!["MEXC statement".to_string()];
    if !tx_type.trim().is_empty() {
        parts.push(format!("type={}", tx_type.trim()));
    }
    if !direction.trim().is_empty() {
        parts.push(format!("direction={}", direction.trim()));
    }
    Some(parts.join(" | "))
}

impl ExchangeParser for MexcStatementParser {
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
            .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {e}")))?
            .clone();

        let cols = resolve_columns(&headers);

        for (internal, display) in &[
            ("time", "Creation Time(UTC+00:00)"),
            ("symbol", "Crypto"),
            ("tx_type", "Transaction Type"),
            ("direction", "Direction"),
            ("quantity", "Quantity"),
        ] {
            if !cols.contains_key(internal) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required MEXC column: '{display}'"),
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
                        format!("Invalid CSV record: {err}"),
                    ));
                    continue;
                }
            };

            let line_number = record
                .position()
                .map(|p| p.line())
                .unwrap_or((idx + 2) as u64) as usize;

            let time_raw = get_field(&record, &cols, "time");
            let symbol_raw = get_field(&record, &cols, "symbol");
            let tx_type_raw = get_field(&record, &cols, "tx_type");
            let direction_raw = get_field(&record, &cols, "direction");
            let quantity_raw = get_field(&record, &cols, "quantity");

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Creation Time(UTC+00:00)"),
                        format!("Invalid timestamp: '{time_raw}'"),
                    ));
                    continue;
                }
            };

            let symbol = symbol_raw.trim().to_uppercase();
            if symbol.is_empty() {
                result.errors.push(RowError::new(
                    line_number,
                    Some("Crypto"),
                    "Crypto symbol is required",
                ));
                continue;
            }
            if is_fiat(&symbol) {
                continue;
            }

            let signed_quantity = match parse_decimal(quantity_raw) {
                Some(value) if value.abs() > 0.0 => value,
                Some(_) => continue,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Quantity"),
                        format!("Invalid quantity: '{quantity_raw}'"),
                    ));
                    continue;
                }
            };

            let (transaction_type, subtype) =
                map_statement_kind(tx_type_raw, direction_raw, signed_quantity);

            result.items.push((
                line_number,
                ImportCryptoTransaction {
                    date: format_datetime(timestamp),
                    wallet: wallet_name.to_string(),
                    symbol,
                    transaction_type,
                    amount: signed_quantity.abs(),
                    subtype,
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes: build_notes(tx_type_raw, direction_raw),
                },
            ));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcStatementHistory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str =
        "UID,Creation Time(UTC+00:00),Crypto,Transaction Type,Direction,Quantity";

    #[test]
    fn interest_row_maps_to_income_interest() {
        let csv = format!(
            "{}\nUSER_001,2025-10-15 12:34:56,USDT,Interest,Inflow,12.34\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("interest"));
        assert!((tx.amount - 12.34).abs() < f64::EPSILON);
    }

    #[test]
    fn withdrawal_row_maps_to_transfer_out() {
        let csv = format!(
            "{}\nUSER_001,2025-11-01 08:20:10,USDT,Withdrawal,Outflow,-200.00\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
        assert!((tx.amount - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unknown_type_uses_direction_fallback() {
        let csv = format!(
            "{}\nUSER_001,2025-11-01 08:20:10,LTC,Adjustment,Outflow,1.5\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "expense");
        assert_eq!(tx.subtype.as_deref(), Some("other"));
        assert!((tx.amount - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn statement_deposit_with_negative_quantity_keeps_valid_transfer_in() {
        // Some anonymizers flip signs even for inflow rows; parser should stay resilient.
        let csv = format!(
            "{}\nUSER_001,2025-10-01 00:00:01,USDT,Deposit,Inflow,-1000.00\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
        assert!((tx.amount - 1000.0).abs() < f64::EPSILON);
    }
}
