// Sanctum — a privacy-first personal finance and crypto vault.
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

use std::collections::{HashMap, VecDeque};

use csv::{ReaderBuilder, StringRecord, Trim};

use super::super::common::{format_datetime, is_fiat, parse_decimal, parse_timestamp};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcStatementParser;

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"').to_lowercase();
        match key.as_str() {
            "creation time(utc+00:00)" => {
                map.insert("time", i);
            }
            "crypto" => {
                map.insert("symbol", i);
            }
            "transaction type" => {
                map.insert("tx_type", i);
            }
            "direction" => {
                map.insert("direction", i);
            }
            "quantity" => {
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

fn should_skip_statement_row(tx_type_raw: &str) -> bool {
    let tx_type = tx_type_raw.trim().to_lowercase();
    // Statement rows that mirror operations already represented in other
    // dedicated exports, or internal account moves that should not change
    // net wallet holdings in Sanctum.
    tx_type.contains("spot trading") || tx_type.contains("futures trading")
}

fn is_convert_row(tx_type_raw: &str) -> bool {
    tx_type_raw.trim().to_lowercase().contains("convert")
}

fn is_withdraw_row(tx_type_raw: &str) -> bool {
    let tx_type = tx_type_raw.trim().to_lowercase();
    tx_type.contains("withdraw") && !tx_type.contains("fee")
}

fn is_withdraw_fee_row(tx_type_raw: &str) -> bool {
    let tx_type = tx_type_raw.trim().to_lowercase();
    tx_type.contains("withdraw") && tx_type.contains("fee")
}

#[derive(Debug, Clone)]
struct ConvertLeg {
    line_number: usize,
    date: String,
    symbol: String,
    amount: f64,
}

#[derive(Default)]
struct ConvertBucket {
    outflow: VecDeque<ConvertLeg>,
    inflow: VecDeque<ConvertLeg>,
}

#[derive(Debug, Clone)]
struct PendingWithdraw {
    line_number: usize,
    date: String,
    symbol: String,
    amount: f64,
    notes: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingWithdrawFee {
    line_number: usize,
    date: String,
    symbol: String,
    amount: f64,
}

fn drain_convert_pairs(
    bucket: &mut ConvertBucket,
    wallet_name: &str,
    result: &mut ParseResult<ImportCryptoTransaction>,
) {
    while !bucket.outflow.is_empty() && !bucket.inflow.is_empty() {
        let out = match bucket.outflow.pop_front() {
            Some(v) => v,
            None => break,
        };
        let input = match bucket.inflow.pop_front() {
            Some(v) => v,
            None => break,
        };

        if out.symbol.eq_ignore_ascii_case(&input.symbol) {
            result.errors.push(RowError::new(
                out.line_number,
                Some("Crypto"),
                format!(
                    "Convert rows require two different assets: '{}' and '{}'",
                    out.symbol, input.symbol
                ),
            ));
            continue;
        }

        result.items.push((
            out.line_number,
            ImportCryptoTransaction {
                date: out.date.clone(),
                wallet: wallet_name.to_string(),
                symbol: out.symbol.clone(),
                transaction_type: "trade".to_string(),
                amount: out.amount,
                subtype: Some("swap".to_string()),
                price_per_coin: None,
                fee: None,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: Some(input.symbol.clone()),
                swap_to_amount: Some(input.amount),
                fee_coin_symbol: None,
                fee_amount: None,
                notes: Some(format!(
                    "MEXC statement | type=Convert | {} {} -> {} {}",
                    out.amount, out.symbol, input.amount, input.symbol
                )),
            },
        ));
    }
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

fn withdraw_key(date: &str, symbol: &str) -> String {
    format!("{}|{}", date.trim(), symbol.trim().to_uppercase())
}

fn emit_withdraw_tx(
    result: &mut ParseResult<ImportCryptoTransaction>,
    wallet_name: &str,
    pending: PendingWithdraw,
    fee: Option<PendingWithdrawFee>,
) {
    let (fee_coin_symbol, fee_amount) = match fee {
        Some(f) if f.amount > 0.0 => (Some(pending.symbol.clone()), Some(f.amount)),
        _ => (None, None),
    };

    result.items.push((
        pending.line_number,
        ImportCryptoTransaction {
            date: pending.date,
            wallet: wallet_name.to_string(),
            symbol: pending.symbol,
            transaction_type: "transfer".to_string(),
            amount: pending.amount,
            subtype: Some("withdrawal".to_string()),
            price_per_coin: None,
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: None,
            swap_to_amount: None,
            fee_coin_symbol,
            fee_amount,
            notes: pending.notes,
        },
    ));
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
        let mut pending_convert: HashMap<String, ConvertBucket> = HashMap::new();
        let mut pending_withdrawals: HashMap<String, VecDeque<PendingWithdraw>> = HashMap::new();
        let mut pending_withdraw_fees: HashMap<String, VecDeque<PendingWithdrawFee>> =
            HashMap::new();

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
            let date = format_datetime(timestamp);

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

            if should_skip_statement_row(tx_type_raw) {
                continue;
            }

            if is_withdraw_fee_row(tx_type_raw) {
                let key = withdraw_key(&date, &symbol);
                let fee = PendingWithdrawFee {
                    line_number,
                    date: date.clone(),
                    symbol: symbol.clone(),
                    amount: signed_quantity.abs(),
                };
                let paired_withdraw = pending_withdrawals
                    .get_mut(&key)
                    .and_then(|q| q.pop_front());
                if let Some(withdraw) = paired_withdraw {
                    emit_withdraw_tx(&mut result, wallet_name, withdraw, Some(fee));
                } else {
                    pending_withdraw_fees.entry(key).or_default().push_back(fee);
                }
                continue;
            }

            if is_withdraw_row(tx_type_raw) {
                let key = withdraw_key(&date, &symbol);
                let withdraw = PendingWithdraw {
                    line_number,
                    date: date.clone(),
                    symbol: symbol.clone(),
                    amount: signed_quantity.abs(),
                    notes: build_notes(tx_type_raw, direction_raw),
                };
                let paired_fee = pending_withdraw_fees
                    .get_mut(&key)
                    .and_then(|q| q.pop_front());
                if let Some(fee) = paired_fee {
                    emit_withdraw_tx(&mut result, wallet_name, withdraw, Some(fee));
                } else {
                    pending_withdrawals
                        .entry(key)
                        .or_default()
                        .push_back(withdraw);
                }
                continue;
            }

            if is_convert_row(tx_type_raw) {
                let outflow = is_outflow(direction_raw) || signed_quantity < 0.0;
                let leg = ConvertLeg {
                    line_number,
                    date: date.clone(),
                    symbol: symbol.clone(),
                    amount: signed_quantity.abs(),
                };
                let bucket = pending_convert.entry(date.clone()).or_default();
                if outflow {
                    bucket.outflow.push_back(leg);
                } else {
                    bucket.inflow.push_back(leg);
                }
                drain_convert_pairs(bucket, wallet_name, &mut result);
                continue;
            }

            let (transaction_type, subtype) =
                map_statement_kind(tx_type_raw, direction_raw, signed_quantity);

            result.items.push((
                line_number,
                ImportCryptoTransaction {
                    date,
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

        for bucket in pending_convert.into_values() {
            for leg in bucket.outflow.into_iter().chain(bucket.inflow) {
                result.errors.push(RowError::new(
                    leg.line_number,
                    Some("Transaction Type"),
                    format!("Unpaired Convert row at '{}'", leg.date),
                ));
            }
        }

        for queue in pending_withdrawals.into_values() {
            for pending in queue {
                emit_withdraw_tx(&mut result, wallet_name, pending, None);
            }
        }

        for queue in pending_withdraw_fees.into_values() {
            for fee in queue {
                if fee.amount <= 0.0 {
                    continue;
                }
                result.errors.push(RowError::new(
                    fee.line_number,
                    Some("Transaction Type"),
                    format!(
                        "Unpaired withdrawal fee row at '{}' for {}",
                        fee.date, fee.symbol
                    ),
                ));
            }
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

    const HEADER: &str = "UID,Creation Time(UTC+00:00),Crypto,Transaction Type,Direction,Quantity";

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
    fn withdrawal_rows_are_parsed_as_transfer_with_fee() {
        let csv = format!(
            "{}\nUSER_001,2025-11-01 08:20:10,USDT,Withdrawal Fees,Outflow,-0.50\nUSER_001,2025-11-01 08:20:10,USDT,Withdraw,Outflow,-10.00\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
        assert!((tx.amount - 10.0).abs() < f64::EPSILON);
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("USDT"));
        assert!((tx.fee_amount.unwrap() - 0.50).abs() < f64::EPSILON);
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
    fn statement_deposit_rows_are_mapped_to_transfer_in() {
        let csv = format!(
            "{}\nUSER_001,2025-10-01 00:00:01,USDT,Deposit,Inflow,1000.00\n",
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

    #[test]
    fn spot_trading_rows_are_skipped_to_avoid_duplicates_with_trade_exports() {
        let csv = format!(
            "{}\nUSER_001,2025-11-04 02:17:27,ICP,Spot Trading,Inflow,1.02\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn convert_rows_are_paired_into_swap() {
        let csv = format!(
            "{}\nUSER_001,2026-01-10 10:00:00,AAA,Convert,Outflow,-1.25\nUSER_001,2026-01-10 10:00:00,BBB,Convert,Inflow,250.5\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert_eq!(tx.symbol, "AAA");
        assert!((tx.amount - 1.25).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("BBB"));
        assert!((tx.swap_to_amount.unwrap() - 250.5).abs() < f64::EPSILON);
    }

    #[test]
    fn unpaired_convert_row_produces_error() {
        let csv = format!(
            "{}\nUSER_001,2026-01-10 10:00:00,AAA,Convert,Outflow,-1.25\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("Unpaired Convert row"));
    }

    #[test]
    fn unpaired_withdrawal_fee_row_produces_error() {
        let csv = format!(
            "{}\nUSER_001,2026-01-10 10:00:00,AAA,Withdrawal Fees,Outflow,-0.1\n",
            HEADER
        );

        let parser = MexcStatementParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(
            result.errors[0]
                .message
                .contains("Unpaired withdrawal fee row")
        );
    }
}
