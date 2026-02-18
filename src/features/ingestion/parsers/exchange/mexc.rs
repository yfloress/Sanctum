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

//! MEXC Spot Trade History CSV parser
//!
//! Handles the CSV export from MEXC's spot trading history.
//!
//! ## Expected columns
//!
//! ```text
//! UID, Pairs, Time, Type, Direction, Average Filled Price, Order Price,
//! Filled Quantity, Order Quantity, Order Amount, Status
//! ```
//!
//! ## Behaviour
//!
//! - Only rows with `Status == "Filled"` are processed; other statuses are
//!   skipped silently.
//! - `Pairs` uses underscore separator: `LTC_USDT`, `BTC_USDT`, `ETH_BTC`.
//! - `Direction` is `Buy` or `Sell`.
//! - `Filled Quantity` is the amount of the **base** asset actually filled.
//! - `Order Amount` is the total in the **quote** currency.
//! - `Average Filled Price` is the execution price.
//! - No fee information is available in this export format.
//! - When `Order Price` is `"Market"`, the order was a market order (noted in
//!   the transaction notes).
//! - If the quote asset is fiat, the transaction is a simple buy/sell trade.
//! - If both assets are crypto, the transaction becomes a swap.
//! - If both assets are fiat, the row is skipped.

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::common::{format_datetime, is_fiat, parse_decimal, parse_timestamp};
use super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

// ─── Column resolution ──────────────────────────────────────────────────────

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"');
        match key {
            "Pairs" => {
                map.insert("pairs", i);
            }
            "Time" => {
                map.insert("time", i);
            }
            "Type" => {
                map.insert("type", i);
            }
            "Direction" => {
                map.insert("direction", i);
            }
            "Average Filled Price" => {
                map.insert("avg_price", i);
            }
            "Order Price" => {
                map.insert("order_price", i);
            }
            "Filled Quantity" => {
                map.insert("filled_qty", i);
            }
            "Order Amount" => {
                map.insert("order_amount", i);
            }
            "Status" => {
                map.insert("status", i);
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

// ─── Pair parsing ───────────────────────────────────────────────────────────

/// Splits a MEXC pair like `LTC_USDT` into `("LTC", "USDT")`.
fn parse_mexc_pair(pair: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = pair.split('_').collect();
    if parts.len() == 2 {
        let base = parts[0].trim().to_uppercase();
        let quote = parts[1].trim().to_uppercase();
        if !base.is_empty() && !quote.is_empty() {
            return Some((base, quote));
        }
    }
    None
}

// ─── Parser ─────────────────────────────────────────────────────────────────

pub struct MexcSpotParser;

impl ExchangeParser for MexcSpotParser {
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
            .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {}", e)))?
            .clone();

        let cols = resolve_columns(&headers);

        // Validate required columns
        for (internal, display) in &[
            ("pairs", "Pairs"),
            ("time", "Time"),
            ("direction", "Direction"),
            ("filled_qty", "Filled Quantity"),
            ("order_amount", "Order Amount"),
            ("status", "Status"),
        ] {
            if !cols.contains_key(internal) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required MEXC column: '{}'", display),
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
                        format!("Invalid CSV record: {}", err),
                    ));
                    continue;
                }
            };

            let line_number = record
                .position()
                .map(|p| p.line())
                .unwrap_or((idx + 2) as u64) as usize;

            let status = get_field(&record, &cols, "status");

            // Only process filled orders
            if !status.eq_ignore_ascii_case("Filled") {
                continue;
            }

            let pair_raw = get_field(&record, &cols, "pairs");
            let time_raw = get_field(&record, &cols, "time");
            let direction_raw = get_field(&record, &cols, "direction");
            let order_type_raw = get_field(&record, &cols, "type");
            let avg_price_raw = get_field(&record, &cols, "avg_price");
            let filled_qty_raw = get_field(&record, &cols, "filled_qty");
            let order_amount_raw = get_field(&record, &cols, "order_amount");

            // Parse timestamp
            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Time"),
                        format!("Invalid timestamp: '{}'", time_raw),
                    ));
                    continue;
                }
            };

            let date = format_datetime(timestamp);

            // Parse pair
            let (base_symbol, quote_symbol) = match parse_mexc_pair(pair_raw) {
                Some(pair) => pair,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Pairs"),
                        format!("Cannot parse trading pair: '{}'", pair_raw),
                    ));
                    continue;
                }
            };

            // Parse filled quantity (base asset amount)
            let filled_qty = match parse_decimal(filled_qty_raw) {
                Some(q) if q > 0.0 => q,
                _ => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Filled Quantity"),
                        format!("Invalid filled quantity: '{}'", filled_qty_raw),
                    ));
                    continue;
                }
            };

            // Parse order amount (quote asset total)
            let order_amount = parse_decimal(order_amount_raw).unwrap_or(0.0);

            // Parse average filled price
            let avg_price = parse_decimal(avg_price_raw);

            let is_buy = direction_raw.eq_ignore_ascii_case("Buy");
            let base_fiat = is_fiat(&base_symbol);
            let quote_fiat = is_fiat(&quote_symbol);

            let notes = Some(format!(
                "MEXC {} {} | {}/{}",
                order_type_raw, direction_raw, base_symbol, quote_symbol,
            ));

            // Both fiat -- skip
            if base_fiat && quote_fiat {
                continue;
            }

            if quote_fiat {
                // Standard pair: BTC/USD, LTC/USDT, etc.
                let price = if avg_price.is_some() {
                    avg_price
                } else if filled_qty > 0.0 && order_amount > 0.0 {
                    Some(order_amount / filled_qty)
                } else {
                    None
                };

                let subtype = if is_buy { "buy" } else { "sell" };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: base_symbol,
                    transaction_type: "trade".to_string(),
                    amount: filled_qty,
                    subtype: Some(subtype.to_string()),
                    price_per_coin: price,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                result.items.push((line_number, tx));
            } else if base_fiat {
                // Inverted pair: USD/BTC (rare but handle)
                let subtype = if is_buy { "sell" } else { "buy" };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: quote_symbol,
                    transaction_type: "trade".to_string(),
                    amount: order_amount,
                    subtype: Some(subtype.to_string()),
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                result.items.push((line_number, tx));
            } else {
                // Crypto-to-crypto: swap
                let (from_symbol, from_amount, to_symbol, to_amount) = if is_buy {
                    // Buying base with quote: out=quote, in=base
                    (quote_symbol, order_amount, base_symbol, filled_qty)
                } else {
                    // Selling base for quote: out=base, in=quote
                    (base_symbol, filled_qty, quote_symbol, order_amount)
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: from_symbol,
                    transaction_type: "trade".to_string(),
                    amount: from_amount,
                    subtype: Some("swap".to_string()),
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: Some(to_symbol),
                    swap_to_amount: Some(to_amount),
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                result.items.push((line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcSpotTradeHistory
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "UID,Pairs,Time,Type,Direction,Average Filled Price,Order Price,Filled Quantity,Order Quantity,Order Amount,Status";

    #[test]
    fn buy_ltc_usdt_market_order_is_swap() {
        // USDT is a stablecoin (crypto), not fiat, so LTC_USDT is crypto-to-crypto = swap
        let csv = format!(
            "{}\n11111111,LTC_USDT,2020-10-10 20:05:07,Market,Buy,07.519999999999999999,Market,0.122,0,14.99164,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());

        let tx = &result.items[0].1;
        // Buying LTC with USDT: out=USDT, in=LTC
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert!((tx.amount - 14.99164).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("LTC"));
        assert!((tx.swap_to_amount.unwrap() - 0.122).abs() < f64::EPSILON);
        assert_eq!(tx.wallet, "MEXC");
        assert_eq!(tx.date, "2020-10-10 20:05:07");

        // No fee info in MEXC exports
        assert!(tx.fee.is_none());
        assert!(tx.fee_coin_symbol.is_none());
        assert!(tx.fee_amount.is_none());
    }

    #[test]
    fn buy_ltc_usdt_limit_order_is_swap() {
        // USDT is crypto, so this is also a swap
        let csv = format!(
            "{}\n11111111,LTC_USDT,2020-10-20 10:04:01,Limit,Buy,05.399999999999999999,85.50,0.1498,0.1498,20.0599,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert!((tx.amount - 20.0599).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("LTC"));
        assert!((tx.swap_to_amount.unwrap() - 0.1498).abs() < f64::EPSILON);
        assert_eq!(tx.date, "2020-10-20 10:04:01");
    }

    #[test]
    fn sell_btc_usdt_is_swap() {
        // USDT is crypto, so selling BTC for USDT is a swap
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-03-15 14:30:00,Limit,Sell,65000.00,65000.00,0.01,0.01,650.00,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        // Selling BTC for USDT: out=BTC, in=USDT
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert!((tx.amount - 0.01).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDT"));
        assert!((tx.swap_to_amount.unwrap() - 650.0).abs() < f64::EPSILON);
    }

    #[test]
    fn buy_btc_usd_fiat_is_trade_buy() {
        // USD is fiat, so BTC_USD is a standard buy trade
        let csv = format!(
            "{}\n11111111,BTC_USD,2024-01-15 10:00:00,Market,Buy,50000.00,Market,0.1,0,5000.00,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.1).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
        assert!(tx.swap_to_symbol.is_none());
    }

    #[test]
    fn sell_btc_usd_fiat_is_trade_sell() {
        // USD is fiat, so selling BTC_USD is a standard sell trade
        let csv = format!(
            "{}\n11111111,BTC_USD,2024-03-15 14:30:00,Limit,Sell,65000.00,65000.00,0.01,0.01,650.00,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 0.01).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 65000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn crypto_to_crypto_becomes_swap() {
        let csv = format!(
            "{}\n11111111,ETH_BTC,2024-01-15 12:00:00,Market,Buy,0.05,Market,2.0,0,0.1,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        // Buying ETH with BTC: out=BTC, in=ETH
        assert_eq!(tx.symbol, "BTC");
        assert!((tx.amount - 0.1).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
        assert!((tx.swap_to_amount.unwrap() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn crypto_to_crypto_sell_becomes_swap() {
        let csv = format!(
            "{}\n11111111,ETH_BTC,2024-01-15 12:00:00,Limit,Sell,0.05,0.05,3.0,3.0,0.15,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        // Selling ETH for BTC: out=ETH, in=BTC
        assert_eq!(tx.symbol, "ETH");
        assert!((tx.amount - 3.0).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("BTC"));
        assert!((tx.swap_to_amount.unwrap() - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn unfilled_orders_are_skipped() {
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:00:00,Limit,Buy,0,50000.00,0,0.01,0,Cancelled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn partially_filled_skipped() {
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:00:00,Limit,Buy,50000,50000,0.005,0.01,250,Partially Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        // Only "Filled" status is processed
        assert!(result.items.is_empty());
    }

    #[test]
    fn empty_csv_produces_no_items() {
        let csv = format!("{}\n", HEADER);

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn invalid_timestamp_produces_error() {
        let csv = format!(
            "{}\n11111111,BTC_USDT,not-a-date,Market,Buy,50000,Market,0.01,0,500,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("Invalid timestamp"));
    }

    #[test]
    fn invalid_pair_produces_error() {
        let csv = format!(
            "{}\n11111111,INVALIDPAIR,2024-01-15 10:00:00,Market,Buy,50000,Market,0.01,0,500,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(
            result.errors[0]
                .message
                .contains("Cannot parse trading pair")
        );
    }

    #[test]
    fn invalid_filled_quantity_produces_error() {
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:00:00,Market,Buy,50000,Market,INVALID,0,500,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("Invalid filled quantity"));
    }

    #[test]
    fn uses_custom_wallet_name() {
        let csv = format!(
            "{}\n11111111,LTC_USDT,2020-10-10 20:05:07,Market,Buy,50,Market,0.1,0,5,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "Mi MEXC").unwrap();

        assert_eq!(result.items[0].1.wallet, "Mi MEXC");
    }

    #[test]
    fn multiple_rows_parsed() {
        let csv = format!(
            "{}\n{}\n{}\n",
            HEADER,
            "11111111,LTC_USDT,2020-10-10 20:05:07,Market,Buy,07.52,Market,0.122,0,14.99,Filled",
            "11111111,LTC_USDT,2020-10-20 10:04:01,Limit,Buy,05.40,85.50,0.1498,0.1498,20.06,Filled",
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn missing_required_column_returns_error() {
        let csv = "UID,Time,Direction\n11111111,2024-01-01 00:00:00,Buy\n";

        let parser = MexcSpotParser;
        let err = parser.parse(csv, "MEXC").unwrap_err();
        assert!(err.message.contains("Missing required MEXC column"));
    }

    #[test]
    fn notes_contain_order_type_and_direction() {
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:00:00,Market,Buy,50000,Market,0.01,0,500,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        let notes = result.items[0].1.notes.as_ref().unwrap();
        assert!(notes.contains("Market"));
        assert!(notes.contains("Buy"));
        assert!(notes.contains("BTC/USDT"));
    }

    #[test]
    fn zero_filled_quantity_produces_error() {
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:00:00,Market,Buy,50000,Market,0,0,0,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn parse_pair_standard() {
        let (base, quote) = parse_mexc_pair("LTC_USDT").unwrap();
        assert_eq!(base, "LTC");
        assert_eq!(quote, "USDT");
    }

    #[test]
    fn parse_pair_invalid_no_underscore() {
        assert!(parse_mexc_pair("LTCUSDT").is_none());
    }

    #[test]
    fn parse_pair_invalid_empty_side() {
        assert!(parse_mexc_pair("_USDT").is_none());
        assert!(parse_mexc_pair("LTC_").is_none());
    }
}
