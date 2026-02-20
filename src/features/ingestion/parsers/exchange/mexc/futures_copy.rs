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

use csv::{ReaderBuilder, Trim};

use super::super::common::{format_datetime, is_fiat, parse_decimal, parse_timestamp};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use super::futures_common::{
    get_field, missing_required, parse_pair_quote_symbol, pick_time, push_pnl_and_fee,
    resolve_columns, PnlFeeContext,
};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcFuturesCopyTradeOrderParser;

impl ExchangeParser for MexcFuturesCopyTradeOrderParser {
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
            ("pair", "futures"),
            ("create_time", "create_time(UTC+00:00)"),
            ("close_time", "close_time(UTC+00:00)"),
            ("pnl", "Position Profit/Loss(USDT)"),
            ("fee", "fee"),
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

            let pair_raw = get_field(&record, &cols, "pair");
            let time_raw = pick_time(
                get_field(&record, &cols, "close_time"),
                get_field(&record, &cols, "create_time"),
            );
            let pnl_raw = get_field(&record, &cols, "pnl");
            let fee_raw = get_field(&record, &cols, "fee");
            let state_raw = get_field(&record, &cols, "copy_state");
            let trader_raw = get_field(&record, &cols, "copy_trader_uid");

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("close_time(UTC+00:00)"),
                        format!("Invalid timestamp: '{time_raw}'"),
                    ));
                    continue;
                }
            };
            let date = format_datetime(timestamp);

            let quote_symbol = parse_pair_quote_symbol(pair_raw).unwrap_or_else(|| "USDT".to_string());
            if is_fiat(&quote_symbol) {
                continue;
            }

            let pnl = parse_decimal(pnl_raw);
            let fee = parse_decimal(fee_raw).map(f64::abs).filter(|v| *v > 0.0);

            let notes = format!(
                "MEXC futures copy order | pair={} | state={} | trader={}",
                pair_raw.trim(),
                state_raw.trim(),
                trader_raw.trim()
            );
            let ctx = PnlFeeContext {
                line_number,
                date: &date,
                wallet_name,
                pnl_symbol: &quote_symbol,
                fee_symbol: &quote_symbol,
                note_prefix: &notes,
            };
            push_pnl_and_fee(
                &mut result.items,
                &ctx,
                pnl,
                fee,
            );
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFuturesCopyTradeOrderHistory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COPY_HEADER: &str = "UID,copy_trader_uid,copy_state,futures,used_margin,leverage,open_type,vol(Cont),copy_amount(USDT),deal_avg_price,close_avg_price,Position Profit/Loss(USDT),fee,create_time(UTC+00:00),close_time(UTC+00:00)";

    #[test]
    fn futures_copy_trade_generates_pnl_and_fee_entries() {
        let csv = format!(
            "{}\nUSER_001,TRADER_001,Active,BTC-USDT,200,10,Market,0.5,1000,30000,31000,1000,5,2025-11-01 10:00:00,2025-11-02 12:00:00\n",
            COPY_HEADER
        );

        let parser = MexcFuturesCopyTradeOrderParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(result.items.iter().any(|(_, tx)| {
            tx.transaction_type == "income" && tx.subtype.as_deref() == Some("reward")
        }));
        assert!(result.items.iter().any(|(_, tx)| {
            tx.transaction_type == "expense" && tx.subtype.as_deref() == Some("fee")
        }));
    }
}
