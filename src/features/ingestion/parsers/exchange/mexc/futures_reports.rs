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

use csv::{ReaderBuilder, Trim};

use super::super::common::{format_datetime, is_fiat, parse_decimal, parse_timestamp};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use super::futures_common::{
    PnlFeeContext, get_field, missing_required, parse_pair_quote_symbol, pick_time,
    push_pnl_and_fee, resolve_columns, status_is_final,
};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcFuturesOrderHistoryParser;
pub struct MexcFuturesPositionHistoryParser;
pub struct MexcFuturesTradeHistoryParser;

impl ExchangeParser for MexcFuturesOrderHistoryParser {
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
            ("pair", "Futures Trading Pair"),
            ("pnl", "Closing PNL"),
            ("trading_fee", "Trading Fee"),
            ("fee_symbol", "Fee-payment Crypto"),
            ("status", "Status"),
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

            let status_raw = get_field(&record, &cols, "status");
            if !status_is_final(status_raw) {
                continue;
            }

            let time_raw = get_field(&record, &cols, "time");
            let pair_raw = get_field(&record, &cols, "pair");
            let pnl_raw = get_field(&record, &cols, "pnl");
            let trading_fee_raw = get_field(&record, &cols, "trading_fee");
            let fee_symbol_raw = get_field(&record, &cols, "fee_symbol");

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
            let date = format_datetime(timestamp);

            let quote_symbol = match parse_pair_quote_symbol(pair_raw) {
                Some(symbol) => symbol,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Futures Trading Pair"),
                        format!("Cannot parse futures pair: '{pair_raw}'"),
                    ));
                    continue;
                }
            };
            let fee_symbol = if fee_symbol_raw.trim().is_empty() {
                quote_symbol.clone()
            } else {
                fee_symbol_raw.trim().to_uppercase()
            };

            if is_fiat(&quote_symbol) && is_fiat(&fee_symbol) {
                continue;
            }

            let pnl = parse_decimal(pnl_raw);
            let fee = parse_decimal(trading_fee_raw)
                .map(f64::abs)
                .filter(|v| *v > 0.0);

            let notes = format!(
                "MEXC futures order | pair={} | status={}",
                pair_raw.trim(),
                status_raw.trim()
            );
            let ctx = PnlFeeContext {
                line_number,
                date: &date,
                wallet_name,
                pnl_symbol: &quote_symbol,
                fee_symbol: &fee_symbol,
                note_prefix: &notes,
            };
            push_pnl_and_fee(&mut result.items, &ctx, pnl, fee);
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFuturesOrderHistory
    }
}

impl ExchangeParser for MexcFuturesPositionHistoryParser {
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
            ("pair", "Futures"),
            ("open_time", "Open Time(UTC+00:00)"),
            ("close_time", "Close Time"),
            ("fee", "Fee"),
            ("pnl", "Realized PNL"),
            ("status", "Status"),
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

            let status_raw = get_field(&record, &cols, "status");
            if !status_is_final(status_raw) {
                continue;
            }

            let pair_raw = get_field(&record, &cols, "pair");
            let time_raw = pick_time(
                get_field(&record, &cols, "close_time"),
                get_field(&record, &cols, "open_time"),
            );
            let pnl_raw = get_field(&record, &cols, "pnl");
            let fee_raw = get_field(&record, &cols, "fee");

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Close Time"),
                        format!("Invalid timestamp: '{time_raw}'"),
                    ));
                    continue;
                }
            };
            let date = format_datetime(timestamp);

            let quote_symbol = match parse_pair_quote_symbol(pair_raw) {
                Some(symbol) => symbol,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Futures"),
                        format!("Cannot parse futures pair: '{pair_raw}'"),
                    ));
                    continue;
                }
            };
            if is_fiat(&quote_symbol) {
                continue;
            }

            let pnl = parse_decimal(pnl_raw);
            let fee = parse_decimal(fee_raw).map(f64::abs).filter(|v| *v > 0.0);

            let notes = format!(
                "MEXC futures position | pair={} | status={}",
                pair_raw.trim(),
                status_raw.trim()
            );
            let ctx = PnlFeeContext {
                line_number,
                date: &date,
                wallet_name,
                pnl_symbol: &quote_symbol,
                fee_symbol: &quote_symbol,
                note_prefix: &notes,
            };
            push_pnl_and_fee(&mut result.items, &ctx, pnl, fee);
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFuturesPositionHistory
    }
}

impl ExchangeParser for MexcFuturesTradeHistoryParser {
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
            ("pair", "Futures Trading Pair"),
            ("pnl", "Closing PNL"),
            ("trading_fee", "Trading Fee"),
            ("fee_symbol", "Fee-payment Crypto"),
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
            let pair_raw = get_field(&record, &cols, "pair");
            let pnl_raw = get_field(&record, &cols, "pnl");
            let trading_fee_raw = get_field(&record, &cols, "trading_fee");
            let fee_symbol_raw = get_field(&record, &cols, "fee_symbol");

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
            let date = format_datetime(timestamp);

            let quote_symbol = match parse_pair_quote_symbol(pair_raw) {
                Some(symbol) => symbol,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Futures Trading Pair"),
                        format!("Cannot parse futures pair: '{pair_raw}'"),
                    ));
                    continue;
                }
            };
            let fee_symbol = if fee_symbol_raw.trim().is_empty() {
                quote_symbol.clone()
            } else {
                fee_symbol_raw.trim().to_uppercase()
            };

            if is_fiat(&quote_symbol) && is_fiat(&fee_symbol) {
                continue;
            }

            let pnl = parse_decimal(pnl_raw);
            let fee = parse_decimal(trading_fee_raw)
                .map(f64::abs)
                .filter(|v| *v > 0.0);

            let notes = format!("MEXC futures trade | pair={}", pair_raw.trim());
            let ctx = PnlFeeContext {
                line_number,
                date: &date,
                wallet_name,
                pnl_symbol: &quote_symbol,
                fee_symbol: &fee_symbol,
                note_prefix: &notes,
            };
            push_pnl_and_fee(&mut result.items, &ctx, pnl, fee);
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFuturesTradeHistory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORDER_HEADER: &str = "UID,Time(UTC+00:00),Futures Trading Pair,Direction,Leverage,Order Type,Order Qty (Cont.),Filled Qty (Cont.),Order Qty (Crypto),Filled Qty (Crypto),Order Qty (Amount),Filled Qty (Amount),Order Price,Average Filled Price,Closing PNL,Trading Fee,Fee-payment Crypto,Status";
    const POSITION_HEADER: &str = "UID,Futures,Open Time(UTC+00:00),Close Time,Margin Mode,Avg Entry Price,Avg Close Price,Direction,Closing Qty (Cont.),Fee,Realized PNL,Status";
    const TRADE_HEADER: &str = "UID,Time(UTC+00:00),Futures Trading Pair,Direction,Order Type,Filled Qty (Cont.),Filled Qty (Crypto),Filled Qty (Amount),Filled Price,Trading Fee,Fee-payment Crypto,Role,Closing PNL";

    #[test]
    fn futures_order_history_uses_fee_symbol() {
        let csv = format!(
            "{}\nUSER_001,2025-11-20 11:00:00,BTC-USDT,Long,5,Limit,1,1,0.03,0.03,1500,1500,30000,30000,200,5,USDT,Filled\n",
            ORDER_HEADER
        );

        let parser = MexcFuturesOrderHistoryParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(result.items.iter().all(|(_, tx)| tx.symbol == "USDT"));
    }

    #[test]
    fn futures_position_history_maps_realized_pnl() {
        let csv = format!(
            "{}\nUSER_001,BTC-USDT,2025-11-10 08:00:00,2025-11-12 09:30:00,Isolated,29500,30500,Long,1,5,1000,Closed\n",
            POSITION_HEADER
        );

        let parser = MexcFuturesPositionHistoryParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(
            result
                .items
                .iter()
                .any(|(_, tx)| tx.transaction_type == "income")
        );
    }

    #[test]
    fn futures_trade_history_maps_negative_pnl_to_expense() {
        let csv = format!(
            "{}\nUSER_001,2025-11-22 13:45:00,BTC-USDT,Long,Market,1,0.03,1500,30000,5,USDT,Taker,-200\n",
            TRADE_HEADER
        );

        let parser = MexcFuturesTradeHistoryParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(result.items.iter().any(|(_, tx)| {
            tx.transaction_type == "expense" && tx.subtype.as_deref() == Some("other")
        }));
    }
}
