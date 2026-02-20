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

use super::common::{format_datetime, is_fiat, is_quote_currency, parse_decimal, parse_timestamp};
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

            // Process orders that had actual fills: "Filled" and "Partially Filled".
            // MEXC exports partially filled orders (order was partially executed
            // then cancelled) with the real `Filled Quantity`. Dropping these
            // loses real trades and causes wrong balances.
            // Cancelled / Pending / Unfilled / other statuses are skipped.
            let status_lower = status.to_lowercase();
            let is_filled = status_lower == "filled" || status_lower.contains("partially filled");
            if !is_filled {
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

            // Parse filled quantity (base asset amount).
            // Negative values are accepted and normalised — exchange exports
            // covering a partial window of an account's history can
            // legitimately contain negative figures.
            let filled_qty = match parse_decimal(filled_qty_raw) {
                Some(q) if q.abs() > 0.0 => q.abs(),
                Some(_) => {
                    // Zero after abs — skip silently (nothing was filled)
                    continue;
                }
                _ => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Filled Quantity"),
                        format!("Invalid filled quantity: '{}'", filled_qty_raw),
                    ));
                    continue;
                }
            };

            // Parse order amount (quote asset total).  Normalise to absolute
            // value for the same partial-window reason.
            let order_amount = parse_decimal(order_amount_raw)
                .map(|v| v.abs())
                .unwrap_or(0.0);

            // Parse average filled price (normalise to positive)
            let avg_price = parse_decimal(avg_price_raw).map(|p| p.abs());

            let is_buy = direction_raw.eq_ignore_ascii_case("Buy");
            let base_fiat = is_fiat(&base_symbol);
            let quote_fiat = is_fiat(&quote_symbol);

            // Stablecoins (USDT, USDC, …) act as pricing currencies: when
            // one side is a stablecoin and the other is a regular crypto
            // asset, classify the trade as buy/sell (with price ≈ USD)
            // instead of a swap.  This avoids phantom stablecoin balances.
            let quote_is_pricing = is_quote_currency(&quote_symbol);
            let base_is_pricing = is_quote_currency(&base_symbol);

            let notes = Some(format!(
                "MEXC {} {} | {}/{}",
                order_type_raw, direction_raw, base_symbol, quote_symbol,
            ));

            // Compute the actual filled value in the quote currency.
            // Prefer `filled_qty * avg_price` (real execution cost) over
            // `order_amount` (which may be the *intended* order total and
            // can differ for market orders or partial fills).
            let filled_value = if let Some(price) = avg_price {
                let computed = filled_qty * price;
                if computed > 0.0 {
                    computed
                } else {
                    order_amount
                }
            } else {
                order_amount
            };

            // Both true fiat → skip entirely
            if base_fiat && quote_fiat {
                continue;
            }

            // Quote is a pricing currency (fiat or stablecoin) and base
            // is a regular crypto asset → standard buy/sell.
            // Also covers fiat quotes (BTC_USD) and stablecoin quotes
            // (BTC_USDT, LTC_USDC, etc.).
            if quote_is_pricing && !base_is_pricing {
                let price = if avg_price.is_some() {
                    avg_price
                } else if filled_qty > 0.0 && filled_value > 0.0 {
                    Some(filled_value / filled_qty)
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
            } else if base_is_pricing && !quote_is_pricing {
                // Inverted pair: USD/BTC, USDT/BTC (rare but handle)
                // Buying base(fiat-like) with quote(crypto) = selling crypto
                let subtype = if is_buy { "sell" } else { "buy" };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: quote_symbol,
                    transaction_type: "trade".to_string(),
                    amount: filled_value,
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
                // Crypto-to-crypto swap: both regular crypto, or both
                // stablecoins (e.g. USDT_USDC).
                // Guard: skip same-symbol pairs (e.g. BTC_BTC) — would
                // produce an invalid swap X→X that fails validation.
                if base_symbol.eq_ignore_ascii_case(&quote_symbol) {
                    continue;
                }

                let (from_symbol, from_amount, to_symbol, to_amount) = if is_buy {
                    // Buying base with quote: out=quote, in=base
                    (quote_symbol, filled_value, base_symbol, filled_qty)
                } else {
                    // Selling base for quote: out=base, in=quote
                    (base_symbol, filled_qty, quote_symbol, filled_value)
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
    fn buy_ltc_usdt_market_order_is_buy() {
        // USDT is a stablecoin → treated as pricing currency → buy
        let csv = format!(
            "{}\n11111111,LTC_USDT,2020-10-10 20:05:07,Market,Buy,122.87,Market,0.122,0,14.99164,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());

        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "LTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.122).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 122.87).abs() < 0.01);
        assert_eq!(tx.wallet, "MEXC");
        assert_eq!(tx.date, "2020-10-10 20:05:07");

        // No swap fields for a buy
        assert!(tx.swap_to_symbol.is_none());
        assert!(tx.swap_to_amount.is_none());

        // No fee info in MEXC exports
        assert!(tx.fee.is_none());
        assert!(tx.fee_coin_symbol.is_none());
        assert!(tx.fee_amount.is_none());
    }

    #[test]
    fn buy_ltc_usdt_limit_order_is_buy() {
        // USDT is a stablecoin → pricing currency → buy
        let csv = format!(
            "{}\n11111111,LTC_USDT,2020-10-20 10:04:01,Limit,Buy,133.91,85.50,0.1498,0.1498,20.0599,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "LTC");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.1498).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 133.91).abs() < 0.01);
    }

    #[test]
    fn sell_btc_usdt_is_sell() {
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:30:45,Limit,Sell,50000.00,50000.00,0.5,0.5,25000.00,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
        // No swap fields for a sell
        assert!(tx.swap_to_symbol.is_none());
        assert!(tx.swap_to_amount.is_none());
    }

    #[test]
    fn buy_btc_usd_fiat_is_trade_buy() {
        let csv = format!(
            "{}\n11111111,BTC_USD,2024-01-15 10:30:45,Limit,Buy,50000.00,50000.00,0.5,0.5,25000.00,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.5).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sell_btc_usd_fiat_is_trade_sell() {
        let csv = format!(
            "{}\n11111111,BTC_USD,2024-02-01 08:00:00,Market,Sell,40000.00,Market,2.0,2.0,80000.00,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn buy_usdc_usdt_stablecoin_pair_is_swap() {
        // Both sides are stablecoins → crypto-to-crypto swap
        let csv = format!(
            "{}\n11111111,USDC_USDT,2024-03-01 12:00:00,Limit,Buy,1.0001,1.0001,500.0,500.0,500.05,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        // Buying USDC with USDT: out=USDT, in=USDC
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDC"));
    }

    #[test]
    fn crypto_to_crypto_becomes_swap() {
        let csv = format!(
            "{}\n11111111,ETH_BTC,2024-03-15 14:00:00,Limit,Buy,0.05,0.05,5.0,5.0,0.25,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        // Buying ETH with BTC: out=BTC, in=ETH
        assert_eq!(tx.symbol, "BTC");
        assert!((tx.amount - 0.25).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
        assert!((tx.swap_to_amount.unwrap() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn crypto_to_crypto_sell_becomes_swap() {
        let csv = format!(
            "{}\n11111111,ETH_BTC,2024-03-20 09:00:00,Limit,Sell,0.05,0.05,3.0,3.0,0.15,Filled\n",
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
    fn filled_value_prefers_avg_price_over_order_amount() {
        // Average Filled Price = 100.0, but Order Amount = 999.0 (stale/mismatched)
        // The parser should compute filled_value = 0.5 * 100.0 = 50.0
        let csv = format!(
            "{}\n11111111,ETH_BTC,2024-04-01 10:00:00,Market,Buy,100.0,Market,0.5,0.5,999.0,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        // out=BTC (quote), in=ETH (base)
        assert_eq!(tx.symbol, "BTC");
        // filled_value = 0.5 * 100.0 = 50.0, NOT 999.0
        assert!((tx.amount - 50.0).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
        assert!((tx.swap_to_amount.unwrap() - 0.5).abs() < f64::EPSILON);
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
    fn unfilled_status_is_skipped_even_if_contains_word_filled() {
        // Guard against broad `contains(\"filled\")` matching "Unfilled".
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:00:00,Limit,Buy,50000,50000,0.01,0.01,500,Unfilled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn partially_filled_is_processed() {
        // MEXC exports partially filled orders with the actual Filled Quantity.
        // These represent real executed trades and must not be dropped.
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:00:00,Limit,Buy,50000,50000,0.005,0.01,250,Partially Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(
            result.items.len(),
            1,
            "Partially filled orders must be processed"
        );
        assert!(result.errors.is_empty());

        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        // Uses the actual filled quantity, not the full order quantity
        assert!((tx.amount - 0.005).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn partially_filled_sell_is_processed() {
        // Ensures partially filled sell orders are recorded so the balance
        // correctly decreases — this was the root cause of phantom holdings.
        let csv = format!(
            "{}\n11111111,ETH_USDT,2024-02-10 14:30:00,Limit,Sell,3200.00,3200.00,0.75,1.0,2400.00,Partially Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "ETH");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!((tx.amount - 0.75).abs() < f64::EPSILON);
        assert!((tx.price_per_coin.unwrap() - 3200.0).abs() < f64::EPSILON);
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
            "{}\n11111111,BTC_USDT,2024-01-15 10:30:45,Limit,Buy,50000.00,50000.00,0.5,0.5,25000.00,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        let tx = &result.items[0].1;
        // BTC_USDT buy is now a buy (not a swap)
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        let notes = tx.notes.as_deref().unwrap();
        assert!(notes.contains("Limit"));
        assert!(notes.contains("Buy"));
    }

    #[test]
    fn zero_filled_quantity_is_skipped() {
        let csv = format!(
            "{}\n11111111,BTC_USDT,2024-01-15 10:00:00,Market,Buy,50000,Market,0,0,0,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        // Zero filled quantity is silently skipped (nothing was filled)
        assert!(result.items.is_empty());
        assert!(result.errors.is_empty());
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

    // ── Negative amount normalisation ───────────────────────────────────

    #[test]
    fn negative_filled_qty_is_normalised() {
        // Exchange exports covering a partial window can contain negative
        // figures.  The parser should accept them and use the absolute value.
        let csv = format!(
            "{}\n11111111,LTC_USDT,2024-03-10 14:00:00,Market,Buy,50,Market,-0.1,0,5,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!(
            (tx.amount - 0.1).abs() < f64::EPSILON,
            "amount should be abs of -0.1"
        );
    }

    #[test]
    fn negative_order_amount_is_normalised() {
        // order_amount negative → normalised to positive for filled_value
        let csv = format!(
            "{}\n11111111,LTC_USDT,2024-03-10 14:00:00,Market,Sell,50,Market,0.2,0,-10,Filled\n",
            HEADER
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        // price_per_coin = filled_value / filled_qty = 10 / 0.2 = 50
        assert!((tx.amount - 0.2).abs() < f64::EPSILON);
    }

    // ── Same-symbol swap guard ──────────────────────────────────────────

    #[test]
    fn same_symbol_pair_is_skipped() {
        // A degenerate pair like BTC_BTC should be silently skipped instead
        // of producing an invalid swap X->X that fails downstream validation.
        let csv = format!(
            "{}\n{},{},{},{},{},{},{},{},{},{},{}",
            HEADER,
            "123456",
            "BTC_BTC",
            "2024-06-01 10:00:00",
            "Limit Order",
            "Buy",
            "1.00",
            "1.00",
            "0.5",
            "0.5",
            "0.5",
            "Filled",
        );

        let parser = MexcSpotParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(
            result.items.is_empty(),
            "Expected no transactions for same-symbol pair BTC_BTC, got {}",
            result.items.len()
        );
    }
}
