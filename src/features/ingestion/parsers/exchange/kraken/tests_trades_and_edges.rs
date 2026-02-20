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

use super::*;

#[test]
fn trades_buy_btc_usd() {
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"BTC/USD\",\"2024-01-15 10:30:45\",\"buy\",\"limit\",\"50000.00\",\"25000.00\",\"5.00\",\"0.5\",\"0\",\"\",\"L1,L2\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
}

#[test]
fn trades_buy_btc_usdt_becomes_buy() {
    // USDT is a stablecoin → pricing currency → buy (not swap)
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"BTC/USDT\",\"2024-01-15 10:30:45\",\"buy\",\"limit\",\"50000.00\",\"25000.00\",\"5.00\",\"0.5\",\"0\",\"\",\"L1,L2\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
    assert!(tx.swap_to_symbol.is_none());
    assert!(tx.swap_to_amount.is_none());
}

#[test]
fn trades_sell_eth_usdt_becomes_sell() {
    // Selling ETH for USDT: stablecoin quote → sell (not swap)
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"ETH/USDT\",\"2024-02-01 08:00:00\",\"sell\",\"market\",\"2000.00\",\"4000.00\",\"3.50\",\"2.0\",\"0\",\"\",\"L1,L2\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 2000.0).abs() < f64::EPSILON);
    assert!(tx.swap_to_symbol.is_none());
    assert!(tx.swap_to_amount.is_none());
}

#[test]
fn trades_sell_eth_eur() {
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"ETH/EUR\",\"2024-02-01 08:00:00\",\"sell\",\"market\",\"2000.00\",\"4000.00\",\"3.50\",\"2.0\",\"0\",\"\",\"L1,L2\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    assert!((tx.fee.unwrap() - 3.5).abs() < f64::EPSILON);
}

#[test]
fn trades_crypto_to_crypto_becomes_swap() {
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"ETH/BTC\",\"2024-03-15 14:00:00\",\"buy\",\"limit\",\"0.05\",\"0.25\",\"0.0001\",\"5.0\",\"0\",\"\",\"L1,L2\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    // Buying ETH with BTC: outgoing=BTC (cost=0.25), incoming=ETH (vol=5.0)
    assert_eq!(tx.symbol, "BTC");
    assert!((tx.amount - 0.25).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
    assert!((tx.swap_to_amount.unwrap() - 5.0).abs() < f64::EPSILON);
    assert_eq!(tx.fee_coin_symbol.as_deref(), Some("BTC"));
    assert!((tx.fee_amount.unwrap() - 0.0001).abs() < f64::EPSILON);
}

#[test]
fn trades_with_old_concatenated_pair() {
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"XXBTZUSD\",\"2024-01-02 03:04:05.1234\",\"buy\",\"limit\",\"50000.00\",\"25000.00\",\"5.00\",\"0.5\",\"0\",\"\",\"L1,L2\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
}

#[test]
fn trades_uses_custom_wallet_name() {
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"BTC/USD\",\"2024-01-15 10:00:00\",\"buy\",\"limit\",\"50000\",\"5000\",\"1\",\"0.1\",\"0\",\"\",\"\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "My Kraken").unwrap();

    assert_eq!(result.items[0].1.wallet, "My Kraken");
}

#[test]
fn trades_zero_volume_is_skipped() {
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"BTC/USD\",\"2024-01-15 10:00:00\",\"buy\",\"limit\",\"50000\",\"0\",\"0\",\"0\",\"0\",\"\",\"\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    // Zero volume is silently skipped (nothing was filled)
    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn trades_with_usdt_pair() {
    // USDT is now a pricing currency → buy (not swap)
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"BTC/USDT\",\"2024-04-01 12:00:00\",\"buy\",\"market\",\"60000.00\",\"30000.00\",\"10.00\",\"0.5\",\"0\",\"\",\"L1\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 60000.0).abs() < f64::EPSILON);
    assert!(tx.swap_to_symbol.is_none());
}

#[test]
fn trades_stablecoin_to_stablecoin_is_swap() {
    // Both stablecoins → neither is "pricing relative to the other" → swap
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"USDC/USDT\",\"2024-04-01 12:00:00\",\"buy\",\"limit\",\"1.0001\",\"500.05\",\"0.10\",\"500.0\",\"0\",\"\",\"L1\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert_eq!(tx.symbol, "USDT");
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDC"));
}

// ── Edge cases ──

#[test]
fn ledger_handles_empty_content() {
    let csv = "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n";

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();
    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn ledger_invite_bonus_becomes_deposit() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-BONUS\",\"REF-BONUS\",\"2024-07-01 12:00:00\",\"invite bonus\",\"\",\"currency\",\"XXBT\",\"0.001\",\"0\",\"0.001\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("deposit"));
    assert_eq!(tx.symbol, "BTC");
}

// ── Same-symbol swap guards ─────────────────────────────────────────

#[test]
fn ledger_same_symbol_trade_pair_is_skipped() {
    // If both sides of a trade pair resolve to the same symbol (e.g. an
    // internal movement that wasn't caught by subtype filtering), the
    // parser should silently skip it instead of producing an invalid
    // swap X→X that fails downstream validation.
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-OUT\",\"REF-SAME\",\"2024-06-01 10:00:00\",\"trade\",\"\",\"currency\",\"XXBT\",\"-0.5\",\"0\",\"0\"\n",
        "\"TX-IN\",\"REF-SAME\",\"2024-06-01 10:00:00\",\"trade\",\"\",\"currency\",\"XXBT\",\"0.5\",\"0\",\"0.5\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    // Both rows normalise to BTC — same-symbol swap should be skipped
    assert!(
        result.items.is_empty(),
        "Expected no transactions for same-symbol trade pair, got {}",
        result.items.len()
    );
}

// ── Negative amount normalisation ───────────────────────────────────

#[test]
fn trades_negative_volume_is_normalised() {
    // Exchange exports covering a partial window of an account's
    // history can legitimately contain negative figures.  The parser
    // should accept them and use the absolute value.
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"BTC/USD\",\"2024-01-15 10:00:00\",\"buy\",\"limit\",\"50000\",\"25000\",\"10\",\"-0.5\",\"0\",\"\",\"\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    assert!(result.errors.is_empty());
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!(
        (tx.amount - 0.5).abs() < f64::EPSILON,
        "amount should be abs of -0.5, got {}",
        tx.amount
    );
    // price = cost / volume = 25000 / 0.5 = 50000
    assert!(
        (tx.price_per_coin.unwrap() - 50000.0).abs() < 0.01,
        "price_per_coin should be 50000"
    );
}

#[test]
fn trades_negative_cost_is_normalised() {
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"ETH/USD\",\"2024-02-20 12:00:00\",\"sell\",\"market\",\"3000\",\"-6000\",\"-5\",\"2.0\",\"0\",\"\",\"\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    assert!(result.errors.is_empty());
    let tx = &result.items[0].1;
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    // price = abs(cost) / volume = 6000 / 2.0 = 3000
    assert!(
        (tx.price_per_coin.unwrap() - 3000.0).abs() < 0.01,
        "price_per_coin should be 3000"
    );
    // fee normalised to absolute value
    assert!(
        (tx.fee.unwrap() - 5.0).abs() < f64::EPSILON,
        "fee should be abs of -5"
    );
}

#[test]
fn trades_same_symbol_pair_is_skipped() {
    // Degenerate pair like BTC/BTC should be silently skipped.
    let csv = concat!(
        "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n",
        "\"TX1\",\"ORD1\",\"BTC/BTC\",\"2024-06-01 10:00:00\",\"buy\",\"limit\",\"1.00\",\"0.5\",\"0\",\"0.5\",\"0\",\"\",\"L1,L2\"\n",
    );

    let parser = KrakenTradesParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert!(
        result.items.is_empty(),
        "Expected no transactions for same-symbol pair, got {}",
        result.items.len()
    );
}
