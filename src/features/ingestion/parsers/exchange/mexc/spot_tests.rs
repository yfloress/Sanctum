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

use super::spot::{MexcSpotParser, parse_mexc_pair};
use crate::features::ingestion::parsers::exchange::ExchangeParser;

const HEADER: &str = "UID,Pairs,Time,Type,Direction,Average Filled Price,Order Price,Filled Quantity,Order Quantity,Order Amount,Status";

#[test]
fn buy_ltc_usdt_market_order_is_swap() {
    let csv = format!(
        "{}\n11111111,LTC_USDT,2020-10-10 20:05:07,Market,Buy,122.87,Market,0.122,0,14.99164,Filled\n",
        HEADER
    );

    let parser = MexcSpotParser;
    let result = parser.parse(&csv, "MEXC").unwrap();

    assert_eq!(result.items.len(), 1);
    assert!(result.errors.is_empty());

    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "USDT");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert!((tx.amount - 14.99014).abs() < 0.01);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("LTC"));
    assert!((tx.swap_to_amount.unwrap() - 0.122).abs() < f64::EPSILON);
    assert_eq!(tx.wallet, "MEXC");
    assert_eq!(tx.date, "2020-10-10 20:05:07");

    assert!(tx.price_per_coin.is_none());

    // No fee info in MEXC exports
    assert!(tx.fee.is_none());
    assert!(tx.fee_coin_symbol.is_none());
    assert!(tx.fee_amount.is_none());
}

#[test]
fn buy_ltc_usdt_limit_order_is_swap() {
    let csv = format!(
        "{}\n11111111,LTC_USDT,2020-10-20 10:04:01,Limit,Buy,133.91,85.50,0.1498,0.1498,20.0599,Filled\n",
        HEADER
    );

    let parser = MexcSpotParser;
    let result = parser.parse(&csv, "MEXC").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "USDT");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert!((tx.amount - (0.1498 * 133.91)).abs() < 0.01);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("LTC"));
    assert!((tx.swap_to_amount.unwrap() - 0.1498).abs() < f64::EPSILON);
}

#[test]
fn sell_btc_usdt_is_swap() {
    let csv = format!(
        "{}\n11111111,BTC_USDT,2024-01-15 10:30:45,Limit,Sell,50000.00,50000.00,0.5,0.5,25000.00,Filled\n",
        HEADER
    );

    let parser = MexcSpotParser;
    let result = parser.parse(&csv, "MEXC").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDT"));
    assert!((tx.swap_to_amount.unwrap() - 25000.0).abs() < f64::EPSILON);
    assert!(tx.price_per_coin.is_none());
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
    assert_eq!(tx.symbol, "USDT");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    // Uses the actual filled quantity, not the full order quantity.
    assert!((tx.amount - 250.0).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("BTC"));
    assert!((tx.swap_to_amount.unwrap() - 0.005).abs() < f64::EPSILON);
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
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert!((tx.amount - 0.75).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDT"));
    assert!((tx.swap_to_amount.unwrap() - 2400.0).abs() < f64::EPSILON);
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
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
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
fn parse_pair_hyphen_separator() {
    let (base, quote) = parse_mexc_pair("BTC-USDT").unwrap();
    assert_eq!(base, "BTC");
    assert_eq!(quote, "USDT");
}

#[test]
fn parse_pair_slash_separator() {
    let (base, quote) = parse_mexc_pair("ETH/USDC").unwrap();
    assert_eq!(base, "ETH");
    assert_eq!(quote, "USDC");
}

#[test]
fn parse_pair_compact_suffix() {
    let (base, quote) = parse_mexc_pair("LTCUSDT").unwrap();
    assert_eq!(base, "LTC");
    assert_eq!(quote, "USDT");
}

#[test]
fn parse_pair_invalid_unknown_compact_symbol() {
    assert!(parse_mexc_pair("LTCQWERTY").is_none());
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
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert!(
        (tx.amount - 5.0).abs() < f64::EPSILON,
        "USDT outflow should use abs-filled value"
    );
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("LTC"));
    assert!((tx.swap_to_amount.unwrap() - 0.1).abs() < f64::EPSILON);
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
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert!((tx.amount - 0.2).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDT"));
    assert!((tx.swap_to_amount.unwrap() - 10.0).abs() < f64::EPSILON);
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
