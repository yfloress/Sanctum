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
use super::fiat_common::{
    get_field, is_completed_status, map_spent_received_to_tx, parse_is_buy, resolve_columns,
};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcFiatP2pParser;

impl ExchangeParser for MexcFiatP2pParser {
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
            let p2p_type_raw = get_field(&record, &cols, "p2p_type");
            let order_quantity_raw = get_field(&record, &cols, "order_quantity");
            let settlement_token_raw = get_field(&record, &cols, "settlement_token");
            let order_amount_raw = get_field(&record, &cols, "order_amount");
            let fee_raw = get_field(&record, &cols, "fee");

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

            let direction_source = if direction_raw.trim().is_empty() {
                p2p_type_raw
            } else {
                direction_raw
            };
            let is_buy = match parse_is_buy(direction_source) {
                Some(value) => value,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Trading Direction"),
                        format!("Invalid P2P direction: '{direction_source}'"),
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

            let fee_amount = parse_decimal(fee_raw).map(f64::abs).filter(|v| *v > 0.0);

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
                    settlement_token.clone(),
                    order_amount,
                    trading_token.clone(),
                    order_quantity,
                )
            } else {
                (
                    trading_token.clone(),
                    order_quantity,
                    settlement_token.clone(),
                    order_amount,
                )
            };

            let fee_coin_symbol = if fee_amount.is_some() && !is_fiat(&settlement_token) {
                Some(settlement_token.clone())
            } else {
                None
            };

            let mut context = format!("direction={}", direction_source.trim());
            if let Some(fee) = fee_amount {
                context.push_str(&format!(" | fee={fee}"));
                if is_fiat(&settlement_token) {
                    context.push_str(&format!(" {}", settlement_token));
                }
            }

            if let Some(tx) = map_spent_received_to_tx(
                format_datetime(timestamp),
                wallet_name,
                "fiat_p2p",
                &context,
                spent_symbol,
                spent_amount,
                recv_symbol,
                recv_amount,
                fee_coin_symbol,
                fee_amount.filter(|_| !is_fiat(&settlement_token)),
            ) {
                result.items.push((line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcFiatP2pOrders
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P2P_HEADER: &str = "UID,P2P Type,User UID,Opponent UID,Start Time(UTC+00:00),End Time(UTC+00:00),Trading Token,Trading Direction,Status,Order Quantity,Price,Fee,Settlement Token,Order Amount";

    #[test]
    fn p2p_completed_sell_maps_to_trade_sell() {
        let csv = format!(
            "{}\nUSER_001,Sell,USER_001,OPP_002,2025-11-20 13:20:00,2025-11-20 13:35:00,USDT,Sell,Completed,300,0.98,3,USD,297\n",
            P2P_HEADER
        );

        let parser = MexcFiatP2pParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert_eq!(tx.symbol, "USDT");
        assert!((tx.amount - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn p2p_pending_row_is_skipped() {
        let csv = format!(
            "{}\nUSER_001,Buy,USER_001,OPP_001,2025-10-07 09:00:00,2025-10-07 09:10:00,USDT,Buy,Pending,500,1.00,5,USD,505\n",
            P2P_HEADER
        );

        let parser = MexcFiatP2pParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn p2p_negative_numeric_fields_are_normalized() {
        let csv = format!(
            "{}\nUSER_001,Sell,USER_001,OPP_002,2025-11-20 13:20:00,2025-11-20 13:35:00,USDT,Sell,Completed,-300,0.98,-3,USD,-297\n",
            P2P_HEADER
        );

        let parser = MexcFiatP2pParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert_eq!(tx.symbol, "USDT");
        assert!((tx.amount - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn p2p_non_usd_fiat_quote_does_not_set_usd_price() {
        let csv = format!(
            "{}\nUSER_001,Sell,USER_001,OPP_002,2025-11-20 13:20:00,2025-11-20 13:35:00,USDT,Sell,Completed,300,0.98,3,EUR,294\n",
            P2P_HEADER
        );

        let parser = MexcFiatP2pParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert_eq!(tx.symbol, "USDT");
        assert!(tx.price_per_coin.is_none());
        let note = tx.notes.as_deref().unwrap_or_default();
        assert!(note.contains("tax_reason=non_usd_quote:EUR"));
    }
}
