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

//! MEXC Deposit History CSV parser.
//!
//! Expected columns:
//! `UID,Status,Time,Crypto,Network,Deposit Amount,TxID,Progress`

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::super::common::{
    format_datetime, is_fiat, normalize_header, parse_decimal, parse_timestamp,
};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcDepositParser;

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = normalize_header(col);
        match key.as_str() {
            "status" => {
                map.insert("status", i);
            }
            "time" => {
                map.insert("time", i);
            }
            "crypto" => {
                map.insert("symbol", i);
            }
            "network" => {
                map.insert("network", i);
            }
            "depositamount" => {
                map.insert("amount", i);
            }
            "txid" => {
                map.insert("txid", i);
            }
            "progress" => {
                map.insert("progress", i);
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

fn cleaned_optional(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed == "--" {
        None
    } else {
        Some(trimmed)
    }
}

fn is_success_status(raw: &str) -> bool {
    let status = raw.trim().to_lowercase();
    if status.is_empty() {
        return false;
    }
    let rejected_terms = [
        "pending",
        "processing",
        "pre-crediting",
        "pre crediting",
        "precrediting",
        "cancel",
        "fail",
        "reject",
        "invalid",
        "restricted",
        "return",
    ];
    if rejected_terms.iter().any(|term| status.contains(term)) {
        return false;
    }

    status == "completed"
        || status == "success"
        || status == "successful"
        || status == "credit"
        || status == "credited"
        || status.contains("credited successfully")
}

fn build_notes(cols: &HashMap<&str, usize>, record: &StringRecord) -> Option<String> {
    let mut parts = vec!["MEXC deposit".to_string()];

    if let Some(network) = cleaned_optional(get_field(record, cols, "network")) {
        parts.push(format!("network={network}"));
    }
    if let Some(txid) = cleaned_optional(get_field(record, cols, "txid")) {
        parts.push(format!("txid={txid}"));
    }
    if let Some(progress) = cleaned_optional(get_field(record, cols, "progress")) {
        parts.push(format!("progress={progress}"));
    }

    Some(parts.join(" | "))
}

impl ExchangeParser for MexcDepositParser {
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
            ("status", "Status"),
            ("time", "Time"),
            ("symbol", "Crypto"),
            ("amount", "Deposit Amount"),
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

            let status = get_field(&record, &cols, "status");
            if !is_success_status(status) {
                continue;
            }

            let time_raw = get_field(&record, &cols, "time");
            let symbol_raw = get_field(&record, &cols, "symbol");
            let amount_raw = get_field(&record, &cols, "amount");

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Time"),
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

            let amount = match parse_decimal(amount_raw) {
                Some(value) if value.abs() > 0.0 => value.abs(),
                Some(_) => continue,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Deposit Amount"),
                        format!("Invalid deposit amount: '{amount_raw}'"),
                    ));
                    continue;
                }
            };

            let tx = ImportCryptoTransaction {
                date: format_datetime(timestamp),
                wallet: wallet_name.to_string(),
                symbol,
                transaction_type: "transfer".to_string(),
                amount,
                subtype: Some("deposit".to_string()),
                price_per_coin: None,
                fee: None,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: None,
                swap_to_amount: None,
                fee_coin_symbol: None,
                fee_amount: None,
                notes: build_notes(&cols, &record),
            };

            result.items.push((line_number, tx));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcDepositHistory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "UID,Status,Time,Crypto,Network,Deposit Amount,TxID,Progress";

    #[test]
    fn successful_deposit_is_transfer_in() {
        let csv = format!(
            "{}\n10000000,Credited Successfully,2025-12-19 21:39:59,USDT,Polygon(MATIC),27,0xa1b2:0,(465/450)\n",
            HEADER
        );

        let parser = MexcDepositParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());

        let tx = &result.items[0].1;
        assert_eq!(tx.wallet, "MEXC");
        assert_eq!(tx.date, "2025-12-19 21:39:59");
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
        assert!((tx.amount - 27.0).abs() < f64::EPSILON);
        assert!(tx.fee_amount.is_none());
    }

    #[test]
    fn pending_or_cancelled_deposit_is_skipped() {
        let csv = format!(
            "{}\n10000000,Pending,2025-12-19 21:39:59,USDT,Polygon(MATIC),27,0xa1b2:0,(10/450)\n10000000,Cancel,2025-12-20 10:00:00,USDT,Polygon(MATIC),11,0xb2c3:0,(0/450)\n",
            HEADER
        );

        let parser = MexcDepositParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn invalid_deposit_amount_produces_error() {
        let csv = format!(
            "{}\n10000000,Credited Successfully,2025-12-19 21:39:59,USDT,Polygon(MATIC),abc,0xa1b2:0,(465/450)\n",
            HEADER
        );

        let parser = MexcDepositParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("Invalid deposit amount"));
    }

    #[test]
    fn missing_required_column_returns_error() {
        let csv =
            "UID,Status,Time,Crypto\n10000000,Credited Successfully,2025-12-19 21:39:59,USDT\n";

        let parser = MexcDepositParser;
        let err = parser.parse(csv, "MEXC").unwrap_err();

        assert!(err.message.contains("Missing required MEXC column"));
    }

    #[test]
    fn status_detection_accepts_credit_and_rejects_pending() {
        assert!(is_success_status("Credited Successfully"));
        assert!(is_success_status("Completed"));
        assert!(is_success_status("Credit"));
        assert!(!is_success_status("Pending"));
        assert!(!is_success_status("Cancel"));
        assert!(!is_success_status("Failed"));
    }
}
