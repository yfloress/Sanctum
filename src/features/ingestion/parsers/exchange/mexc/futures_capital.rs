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

use csv::{ReaderBuilder, Trim};

use super::super::common::{format_datetime, is_fiat, parse_decimal, parse_timestamp};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use super::futures_common::{get_field, missing_required, resolve_columns};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcFuturesCapitalFlowParser;

impl ExchangeParser for MexcFuturesCapitalFlowParser {
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
            ("time", "Time(UTC+00:00)"),
            ("symbol", "Crypto"),
            ("fund_type", "Fund Type"),
            ("flow_type", "Fund Flow Type"),
            ("amount", "Amount"),
        ] {
            if !cols.contains_key(internal) {
                return Err(missing_required(display));
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
            let fund_type_raw = get_field(&record, &cols, "fund_type");
            let flow_type_raw = get_field(&record, &cols, "flow_type");
            let amount_raw = get_field(&record, &cols, "amount");
            let pair_raw = get_field(&record, &cols, "pair");

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Time(UTC+00:00)"),
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

            let fund_type = fund_type_raw.trim().to_lowercase();
            let flow_type = flow_type_raw.trim().to_lowercase();

            let (transaction_type, subtype) = if fund_type.contains("fee") {
                ("expense".to_string(), "fee".to_string())
            } else if flow_type.contains("out") || fund_type.contains("withdraw") {
                ("transfer".to_string(), "withdrawal".to_string())
            } else if flow_type.contains("in") || fund_type.contains("deposit") {
                ("transfer".to_string(), "deposit".to_string())
            } else if signed_amount < 0.0 {
                ("expense".to_string(), "other".to_string())
            } else {
                ("income".to_string(), "other".to_string())
            };

            result.items.push((
                line_number,
                ImportCryptoTransaction {
                    date: format_datetime(timestamp),
                    wallet: wallet_name.to_string(),
                    symbol,
                    transaction_type,
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
                    notes: Some(format!(
                        "MEXC futures capital flow | pair={} | fund_type={} | flow_type={}",
                        pair_raw.trim(),
                        fund_type_raw.trim(),
                        flow_type_raw.trim()
                    )),
                },
            ));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFuturesCapitalFlow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPITAL_FLOW_HEADER: &str =
        "UID,Time(UTC+00:00),Futures Trading Pair,Crypto,Fund Type,Fund Flow Type,Amount";

    #[test]
    fn futures_capital_flow_deposit_maps_to_transfer_in() {
        let csv = format!(
            "{}\nUSER_001,2025-11-15 14:30:00,BTC-USDT,USDT,Deposit,Inflow,5000\n",
            CAPITAL_FLOW_HEADER
        );

        let parser = MexcFuturesCapitalFlowParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "transfer");
        assert_eq!(tx.subtype.as_deref(), Some("deposit"));
        assert!((tx.amount - 5000.0).abs() < f64::EPSILON);
    }
}
