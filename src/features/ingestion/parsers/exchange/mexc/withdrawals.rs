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

//! MEXC Withdrawal History CSV parser.
//!
//! Expected columns:
//! `UID,Status,Time,Crypto,Network,Request Amount,Withdrawal Address,memo,TxID,Trading Fee,Settlement Amount,Withdrawal Descriptions`

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::super::common::{
    format_datetime, is_fiat, normalize_header, parse_decimal, parse_timestamp,
};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcWithdrawalParser;

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
            "requestamount" => {
                map.insert("request_amount", i);
            }
            "tradingfee" => {
                map.insert("trading_fee", i);
            }
            "settlementamount" => {
                map.insert("settlement_amount", i);
            }
            "network" => {
                map.insert("network", i);
            }
            "withdrawaladdress" => {
                map.insert("address", i);
            }
            "memo" => {
                map.insert("memo", i);
            }
            "txid" => {
                map.insert("txid", i);
            }
            "withdrawaldescriptions" => {
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
        "under review",
        "review",
        "verification",
        "confirmation",
        "cancel",
        "fail",
        "reject",
    ];
    if rejected_terms.iter().any(|term| status.contains(term)) {
        return false;
    }

    status == "completed"
        || status == "success"
        || status == "successful"
        || status.contains("withdrawal successful")
}

fn resolve_transfer_amount(
    request_amount: f64,
    settlement_amount: Option<f64>,
    fee: Option<f64>,
) -> f64 {
    let epsilon = 1e-12;

    if let Some(settlement) = settlement_amount
        && settlement > epsilon
        && settlement <= request_amount + epsilon
    {
        return settlement;
    }

    if let Some(fee_amount) = fee
        && request_amount > fee_amount + epsilon
    {
        return request_amount - fee_amount;
    }

    request_amount
}

fn build_notes(cols: &HashMap<&str, usize>, record: &StringRecord) -> Option<String> {
    let mut parts = vec!["MEXC withdrawal".to_string()];

    if let Some(network) = cleaned_optional(get_field(record, cols, "network")) {
        parts.push(format!("network={network}"));
    }
    if let Some(address) = cleaned_optional(get_field(record, cols, "address")) {
        parts.push(format!("address={address}"));
    }
    if let Some(memo) = cleaned_optional(get_field(record, cols, "memo")) {
        parts.push(format!("memo={memo}"));
    }
    if let Some(txid) = cleaned_optional(get_field(record, cols, "txid")) {
        parts.push(format!("txid={txid}"));
    }
    if let Some(desc) = cleaned_optional(get_field(record, cols, "description")) {
        parts.push(format!("desc={desc}"));
    }

    Some(parts.join(" | "))
}

impl ExchangeParser for MexcWithdrawalParser {
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
            ("request_amount", "Request Amount"),
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
            let request_amount_raw = get_field(&record, &cols, "request_amount");
            let trading_fee_raw = get_field(&record, &cols, "trading_fee");
            let settlement_amount_raw = get_field(&record, &cols, "settlement_amount");

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

            let request_amount = match parse_decimal(request_amount_raw) {
                Some(value) if value.abs() > 0.0 => value.abs(),
                Some(_) => continue,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Request Amount"),
                        format!("Invalid request amount: '{request_amount_raw}'"),
                    ));
                    continue;
                }
            };

            let fee_amount = parse_decimal(trading_fee_raw)
                .map(f64::abs)
                .filter(|v| *v > 0.0);
            let settlement_amount = parse_decimal(settlement_amount_raw)
                .map(f64::abs)
                .filter(|v| *v > 0.0);

            let amount = resolve_transfer_amount(request_amount, settlement_amount, fee_amount);
            if amount <= 0.0 {
                result.errors.push(RowError::new(
                    line_number,
                    Some("Request Amount"),
                    "Withdrawal amount must be greater than zero",
                ));
                continue;
            }

            let tx = ImportCryptoTransaction {
                date: format_datetime(timestamp),
                wallet: wallet_name.to_string(),
                symbol: symbol.clone(),
                transaction_type: "transfer".to_string(),
                amount,
                subtype: Some("withdrawal".to_string()),
                price_per_coin: None,
                fee: None,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: None,
                swap_to_amount: None,
                fee_coin_symbol: fee_amount.map(|_| symbol.clone()),
                fee_amount,
                notes: build_notes(&cols, &record),
            };

            result.items.push((line_number, tx));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcWithdrawalHistory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "UID,Status,Time,Crypto,Network,Request Amount,Withdrawal Address,memo,TxID,Trading Fee,Settlement Amount,Withdrawal Descriptions";

    #[test]
    fn successful_withdrawal_is_transfer_out_with_fee() {
        let csv = format!(
            "{}\n11111111,Withdrawal Successful,2025-03-14 09:27:31,LTC,Litecoin(LTC),0.322,addr_ltc_test,--,a1b2,0.0001,0.3129,-\n",
            HEADER
        );

        let parser = MexcWithdrawalParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());

        let tx = &result.items[0].1;
        assert_eq!(tx.wallet, "MEXC");
        assert_eq!(tx.date, "2025-03-14 09:27:31");
        assert_eq!(tx.symbol, "LTC");
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
        assert!((tx.amount - 0.3129).abs() < f64::EPSILON);
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("LTC"));
        assert!((tx.fee_amount.unwrap() - 0.0001).abs() < f64::EPSILON);
    }

    #[test]
    fn cancelled_withdrawal_is_skipped() {
        let csv = format!(
            "{}\n11111111,Cancel,2025-12-21 19:38:50,USDT,Polygon(MATIC),53.568283,0xaddr,--,--,0.01,53.558283,-\n",
            HEADER
        );

        let parser = MexcWithdrawalParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn withdrawal_without_settlement_uses_request_minus_fee() {
        let csv = format!(
            "{}\n11111111,Withdrawal Successful,2025-06-02 17:45:12,BTC,Bitcoin(BTC),0.00068164,addr_btc_test,--,b2c3,0.000014,,-\n",
            HEADER
        );

        let parser = MexcWithdrawalParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert!((tx.amount - (0.00068164 - 0.000014)).abs() < f64::EPSILON);
        assert!((tx.fee_amount.unwrap() - 0.000014).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_request_amount_produces_error() {
        let csv = format!(
            "{}\n11111111,Withdrawal Successful,2025-06-02 17:45:12,BTC,Bitcoin(BTC),abc,addr_btc_test,--,b2c3,0.000014,0.00066764,-\n",
            HEADER
        );

        let parser = MexcWithdrawalParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("Invalid request amount"));
    }

    #[test]
    fn missing_required_column_returns_error() {
        let csv =
            "UID,Status,Time,Crypto\n11111111,Withdrawal Successful,2025-06-02 17:45:12,BTC\n";

        let parser = MexcWithdrawalParser;
        let err = parser.parse(csv, "MEXC").unwrap_err();

        assert!(err.message.contains("Missing required MEXC column"));
    }

    #[test]
    fn status_detection_accepts_completed_and_rejects_failed() {
        assert!(is_success_status("Withdrawal Successful"));
        assert!(is_success_status("Completed"));
        assert!(!is_success_status("Cancel"));
        assert!(!is_success_status("Failed"));
        assert!(!is_success_status("Pending"));
    }
}
