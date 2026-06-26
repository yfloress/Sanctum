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

//! MEXC funding parsers.
//!
//! Supported headers:
//! - Funding History - Other
//! - Funding History - Transfer History

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::super::common::{
    format_datetime, is_fiat, normalize_header, parse_decimal, parse_timestamp,
};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcFundingOtherParser;
pub struct MexcFundingTransferParser;

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = normalize_header(col);
        match key.as_str() {
            "time" => {
                map.insert("time", i);
            }
            "crypto" => {
                map.insert("symbol", i);
            }
            "type" => {
                map.insert("type", i);
            }
            "quantity" => {
                map.insert("quantity", i);
            }
            "status" => {
                map.insert("status", i);
            }
            "remark" => {
                map.insert("remark", i);
            }
            "fromsystem" => {
                map.insert("from_system", i);
            }
            "tosystem" => {
                map.insert("to_system", i);
            }
            "currency" => {
                map.insert("currency", i);
            }
            "amount" => {
                map.insert("amount", i);
            }
            "updatetimeutc0000" => {
                map.insert("update_time", i);
            }
            "createtimeutc0000" => {
                map.insert("create_time", i);
            }
            "transfertype" => {
                map.insert("transfer_type", i);
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

fn is_success_status(raw: &str) -> bool {
    let status = raw.trim().to_lowercase();
    if status.is_empty() {
        return false;
    }
    let rejected_terms = [
        "pending",
        "processing",
        "cancel",
        "fail",
        "reject",
        "review",
        "verification",
    ];
    if rejected_terms.iter().any(|term| status.contains(term)) {
        return false;
    }

    status == "completed"
        || status == "success"
        || status == "successful"
        || status.starts_with("completed ")
}

fn map_other_type(type_raw: &str, signed_quantity: f64) -> (String, Option<String>) {
    let normalized = type_raw.trim().to_lowercase();

    if normalized.contains("interest") {
        return ("income".to_string(), Some("interest".to_string()));
    }
    if normalized.contains("reward") || normalized.contains("bonus") {
        return ("income".to_string(), Some("reward".to_string()));
    }
    if normalized.contains("airdrop") {
        return ("income".to_string(), Some("airdrop".to_string()));
    }
    if normalized.contains("deposit") {
        return ("transfer".to_string(), Some("deposit".to_string()));
    }
    if normalized.contains("withdraw") {
        return ("transfer".to_string(), Some("withdrawal".to_string()));
    }
    if normalized.contains("fee") {
        return ("expense".to_string(), Some("fee".to_string()));
    }

    if signed_quantity < 0.0 {
        ("expense".to_string(), Some("other".to_string()))
    } else {
        ("income".to_string(), Some("other".to_string()))
    }
}

fn map_transfer_subtype(transfer_type_raw: &str, signed_amount: f64) -> String {
    let normalized = transfer_type_raw.trim().to_lowercase();
    let tokens: Vec<&str> = normalized
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    if normalized.contains("withdraw") || tokens.iter().any(|t| *t == "out" || *t == "outflow") {
        return "withdrawal".to_string();
    }
    if normalized.contains("deposit") || tokens.iter().any(|t| *t == "in" || *t == "inflow") {
        return "deposit".to_string();
    }
    if signed_amount < 0.0 {
        "withdrawal".to_string()
    } else {
        "deposit".to_string()
    }
}

fn is_internal_system_transfer(
    from_system_raw: &str,
    to_system_raw: &str,
    transfer_type_raw: &str,
) -> bool {
    let from = from_system_raw.trim().to_lowercase();
    let to = to_system_raw.trim().to_lowercase();
    if !from.is_empty() && !to.is_empty() && from != to {
        return true;
    }

    let transfer_type = transfer_type_raw.trim().to_lowercase();
    transfer_type.contains("internal") || transfer_type == "transfer"
}

impl ExchangeParser for MexcFundingOtherParser {
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
            ("time", "Time"),
            ("symbol", "Crypto"),
            ("type", "Type"),
            ("quantity", "Quantity"),
            ("status", "Status"),
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
            let type_raw = get_field(&record, &cols, "type");
            let quantity_raw = get_field(&record, &cols, "quantity");
            let remark_raw = get_field(&record, &cols, "remark");

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

            let signed_quantity = match parse_decimal(quantity_raw) {
                Some(v) if v.abs() > 0.0 => v,
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

            let (transaction_type, subtype) = map_other_type(type_raw, signed_quantity);

            let mut notes = vec!["MEXC funding other".to_string()];
            if !type_raw.trim().is_empty() {
                notes.push(format!("type={}", type_raw.trim()));
            }
            if !remark_raw.trim().is_empty() {
                notes.push(format!("remark={}", remark_raw.trim()));
            }

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
                    notes: Some(notes.join(" | ")),
                },
            ));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFundingOtherHistory
    }
}

impl ExchangeParser for MexcFundingTransferParser {
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
            ("currency", "Currency"),
            ("amount", "Amount"),
            ("status", "Status"),
            ("transfer_type", "Transfer Type"),
        ] {
            if !cols.contains_key(internal) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required MEXC column: '{display}'"),
                ));
            }
        }
        if !cols.contains_key("update_time") && !cols.contains_key("create_time") {
            return Err(RowError::new(
                1,
                None,
                "Missing required MEXC column: 'update_time(UTC+00:00)' or 'create_time(UTC+00:00)'",
            ));
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

            let time_raw = {
                let updated = get_field(&record, &cols, "update_time");
                if updated.trim().is_empty() {
                    get_field(&record, &cols, "create_time")
                } else {
                    updated
                }
            };
            let symbol_raw = get_field(&record, &cols, "currency");
            let amount_raw = get_field(&record, &cols, "amount");
            let transfer_type_raw = get_field(&record, &cols, "transfer_type");
            let from_system_raw = get_field(&record, &cols, "from_system");
            let to_system_raw = get_field(&record, &cols, "to_system");

            if is_internal_system_transfer(from_system_raw, to_system_raw, transfer_type_raw) {
                continue;
            }

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("update_time(UTC+00:00)"),
                        format!("Invalid timestamp: '{time_raw}'"),
                    ));
                    continue;
                }
            };

            let symbol = symbol_raw.trim().to_uppercase();
            if symbol.is_empty() {
                result.errors.push(RowError::new(
                    line_number,
                    Some("Currency"),
                    "Currency symbol is required",
                ));
                continue;
            }
            if is_fiat(&symbol) {
                continue;
            }

            let signed_amount = match parse_decimal(amount_raw) {
                Some(v) if v.abs() > 0.0 => v,
                Some(_) => continue,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Amount"),
                        format!("Invalid amount: '{amount_raw}'"),
                    ));
                    continue;
                }
            };

            let subtype = map_transfer_subtype(transfer_type_raw, signed_amount);

            let mut notes = vec!["MEXC funding transfer".to_string()];
            if !transfer_type_raw.trim().is_empty() {
                notes.push(format!("transfer_type={}", transfer_type_raw.trim()));
            }
            if !from_system_raw.trim().is_empty() {
                notes.push(format!("from={}", from_system_raw.trim()));
            }
            if !to_system_raw.trim().is_empty() {
                notes.push(format!("to={}", to_system_raw.trim()));
            }

            result.items.push((
                line_number,
                ImportCryptoTransaction {
                    date: format_datetime(timestamp),
                    wallet: wallet_name.to_string(),
                    symbol,
                    transaction_type: "transfer".to_string(),
                    amount: signed_amount.abs(),
                    subtype: Some(subtype),
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes: Some(notes.join(" | ")),
                },
            ));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFundingTransferHistory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OTHER_HEADER: &str = "UID,Time,Crypto,Type,Quantity,Status,Remark";
    const TRANSFER_HEADER: &str = "UID,From System,To System,Currency,Amount,Status,update_time(UTC+00:00),create_time(UTC+00:00),Transfer Type";

    #[test]
    fn funding_other_bonus_maps_to_income_reward() {
        let csv = format!(
            "{}\nUSER_001,2025-12-10 11:22:33,USDT,Bonus,10,Completed,Welcome bonus\n",
            OTHER_HEADER
        );

        let parser = MexcFundingOtherParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.subtype.as_deref(), Some("reward"));
        assert!((tx.amount - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn funding_transfer_internal_system_rows_are_skipped() {
        let csv = format!(
            "{}\nUSER_001,Internal,External,USDT,150,Completed,2025-12-15 08:00:00,2025-12-15 07:55:00,Withdrawal\n",
            TRANSFER_HEADER
        );

        let parser = MexcFundingTransferParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn funding_transfer_without_system_context_is_kept() {
        let csv = format!(
            "{}\nUSER_001,,,USDT,150,Completed,2025-12-15 08:00:00,2025-12-15 07:55:00,Withdrawal\n",
            TRANSFER_HEADER
        );

        let parser = MexcFundingTransferParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
        assert!((tx.amount - 150.0).abs() < f64::EPSILON);
    }
}
