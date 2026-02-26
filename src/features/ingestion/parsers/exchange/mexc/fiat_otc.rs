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

use super::super::common::{format_datetime, parse_decimal, parse_timestamp};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use super::fiat_common::{
    get_field, is_completed_status, map_spent_received_to_tx, parse_is_buy, resolve_columns,
};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcFiatOtcParser;

impl ExchangeParser for MexcFiatOtcParser {
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
            ("start_time", "Start Time(UTC+00:00)"),
            ("trading_token", "Trading Token"),
            ("direction", "Trading Direction"),
            ("status", "Status"),
            ("order_quantity", "Order Quantity"),
            ("settlement_token", "Settlement Token"),
            ("order_amount", "Order Amount"),
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
            if !is_completed_status(status) {
                continue;
            }

            let time_raw = get_field(&record, &cols, "start_time");
            let trading_token_raw = get_field(&record, &cols, "trading_token");
            let direction_raw = get_field(&record, &cols, "direction");
            let order_quantity_raw = get_field(&record, &cols, "order_quantity");
            let settlement_token_raw = get_field(&record, &cols, "settlement_token");
            let order_amount_raw = get_field(&record, &cols, "order_amount");
            let order_id = get_field(&record, &cols, "order_id");
            let payment_method = get_field(&record, &cols, "payment_method");

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Start Time(UTC+00:00)"),
                        format!("Invalid timestamp: '{time_raw}'"),
                    ));
                    continue;
                }
            };

            let is_buy = match parse_is_buy(direction_raw) {
                Some(value) => value,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Trading Direction"),
                        format!("Invalid trading direction: '{direction_raw}'"),
                    ));
                    continue;
                }
            };

            let order_quantity = match parse_decimal(order_quantity_raw) {
                Some(v) if v.abs() > 0.0 => v.abs(),
                Some(_) => continue,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Order Quantity"),
                        format!("Invalid order quantity: '{order_quantity_raw}'"),
                    ));
                    continue;
                }
            };

            let order_amount = match parse_decimal(order_amount_raw) {
                Some(v) if v.abs() > 0.0 => v.abs(),
                Some(_) => continue,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Order Amount"),
                        format!("Invalid order amount: '{order_amount_raw}'"),
                    ));
                    continue;
                }
            };

            let trading_token = trading_token_raw.trim().to_uppercase();
            let settlement_token = settlement_token_raw.trim().to_uppercase();
            if trading_token.is_empty() || settlement_token.is_empty() {
                result.errors.push(RowError::new(
                    line_number,
                    Some("Trading Token"),
                    "Trading Token and Settlement Token are required",
                ));
                continue;
            }

            let (spent_symbol, spent_amount, recv_symbol, recv_amount) = if is_buy {
                (
                    trading_token,
                    order_quantity,
                    settlement_token,
                    order_amount,
                )
            } else {
                (
                    settlement_token,
                    order_amount,
                    trading_token,
                    order_quantity,
                )
            };

            let mut context = String::new();
            if !order_id.trim().is_empty() {
                context.push_str(&format!("order_id={}", order_id.trim()));
            }
            if !payment_method.trim().is_empty() {
                if !context.is_empty() {
                    context.push_str(" | ");
                }
                context.push_str(&format!("payment_method={}", payment_method.trim()));
            }
            if context.is_empty() {
                context.push_str("fiat_otc");
            }

            if let Some(tx) = map_spent_received_to_tx(
                format_datetime(timestamp),
                wallet_name,
                "fiat_otc",
                &context,
                spent_symbol,
                spent_amount,
                recv_symbol,
                recv_amount,
                None,
                None,
            ) {
                result.items.push((line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFiatOtcOrders
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OTC_HEADER: &str = "UID,Order ID,Start Time(UTC+00:00),End Time(UTC+00:00),Trading Token,Trading Direction,Status,Order Quantity,Settlement Token,Order Amount,Payment Method";

    #[test]
    fn otc_completed_buy_maps_to_trade_buy() {
        let csv = format!(
            "{}\nUSER_001,OTC-001,2025-10-05 10:00:00,2025-10-05 10:15:00,USD,Buy,Completed,1000,USDT,1000.00,Bank Transfer\n",
            OTC_HEADER
        );

        let parser = MexcFiatOtcParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert_eq!(tx.symbol, "USDT");
        assert!((tx.amount - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pending_otc_row_is_skipped() {
        let csv = format!(
            "{}\nUSER_001,OTC-002,2025-11-12 14:30:00,2025-11-12 14:45:00,EUR,Sell,Pending,800,USDT,960.00,PayPal\n",
            OTC_HEADER
        );

        let parser = MexcFiatOtcParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn otc_negative_numeric_fields_are_normalized() {
        let csv = format!(
            "{}\nUSER_001,OTC-003,2025-10-06 10:00:00,2025-10-06 10:05:00,USD,Buy,Completed,-500,USDT,-500.00,Bank Transfer\n",
            OTC_HEADER
        );

        let parser = MexcFiatOtcParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert_eq!(tx.symbol, "USDT");
        assert!((tx.amount - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn otc_non_usd_fiat_quote_does_not_set_usd_price() {
        let csv = format!(
            "{}\nUSER_001,OTC-004,2025-10-07 10:00:00,2025-10-07 10:05:00,EUR,Buy,Completed,900,USDT,1000.00,SEPA\n",
            OTC_HEADER
        );

        let parser = MexcFiatOtcParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert_eq!(tx.symbol, "USDT");
        assert!(tx.price_per_coin.is_none());
        let note = tx.notes.as_deref().unwrap_or_default();
        assert!(note.contains("tax_reason=non_usd_quote:EUR"));
    }
}
