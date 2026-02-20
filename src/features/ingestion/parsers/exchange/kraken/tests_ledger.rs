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
fn ledger_deposit_becomes_transfer_deposit() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX1\",\"REF1\",\"2024-01-15 10:00:00\",\"deposit\",\"\",\"currency\",\"XXBT\",\"0.5\",\"0\",\"0.5\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("deposit"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
}

#[test]
fn ledger_withdrawal_becomes_transfer_withdrawal() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX1\",\"REF1\",\"2024-01-15 10:00:00\",\"withdrawal\",\"\",\"currency\",\"XETH\",\"-1.5\",\"0.001\",\"0\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
    assert!((tx.amount - 1.5).abs() < f64::EPSILON);
    assert_eq!(tx.fee_coin_symbol.as_deref(), Some("ETH"));
    assert!((tx.fee_amount.unwrap() - 0.001).abs() < f64::EPSILON);
}

#[test]
fn ledger_trade_pair_becomes_buy() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-OUT\",\"REF-TRADE\",\"2024-01-15 10:00:00\",\"trade\",\"\",\"currency\",\"ZUSD\",\"-25000\",\"5.00\",\"0\"\n",
        "\"TX-IN\",\"REF-TRADE\",\"2024-01-15 10:00:00\",\"trade\",\"\",\"currency\",\"XXBT\",\"0.5\",\"0\",\"0.5\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    // USD is fiat -> buying BTC
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
    assert!((tx.fee.unwrap() - 5.0).abs() < f64::EPSILON);
}

#[test]
fn ledger_usdt_trade_pair_becomes_buy() {
    // USDT is a stablecoin → treated as pricing currency → buy (not swap)
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-OUT\",\"REF-USDT\",\"2024-05-01 10:00:00\",\"trade\",\"\",\"currency\",\"USDT\",\"-25000\",\"5.00\",\"0\"\n",
        "\"TX-IN\",\"REF-USDT\",\"2024-05-01 10:00:00\",\"trade\",\"\",\"currency\",\"XXBT\",\"0.5\",\"0\",\"0.5\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
    assert!(tx.swap_to_symbol.is_none());
    assert!(tx.swap_to_amount.is_none());
}

#[test]
fn ledger_trade_pair_becomes_sell() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-OUT\",\"REF-SELL\",\"2024-02-01 08:00:00\",\"trade\",\"\",\"currency\",\"XETH\",\"-2.0\",\"0\",\"0\"\n",
        "\"TX-IN\",\"REF-SELL\",\"2024-02-01 08:00:00\",\"trade\",\"\",\"currency\",\"ZUSD\",\"4000\",\"3.50\",\"4000\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 2000.0).abs() < f64::EPSILON);
    assert!((tx.fee.unwrap() - 3.5).abs() < f64::EPSILON);
}

#[test]
fn ledger_sell_for_usdt_becomes_sell() {
    // Selling ETH for USDT: USDT is a stablecoin → sell (not swap)
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-OUT\",\"REF-SELL-USDT\",\"2024-02-01 08:00:00\",\"trade\",\"\",\"currency\",\"XETH\",\"-2.0\",\"0\",\"0\"\n",
        "\"TX-IN\",\"REF-SELL-USDT\",\"2024-02-01 08:00:00\",\"trade\",\"\",\"currency\",\"USDT\",\"4000\",\"3.50\",\"4000\"\n",
    );

    let parser = KrakenLedgerParser;
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
fn ledger_crypto_to_crypto_becomes_swap() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-OUT\",\"REF-SWAP\",\"2024-03-10 12:00:00\",\"trade\",\"\",\"currency\",\"XETH\",\"-5.0\",\"0.01\",\"0\"\n",
        "\"TX-IN\",\"REF-SWAP\",\"2024-03-10 12:00:00\",\"trade\",\"\",\"currency\",\"XXBT\",\"0.25\",\"0\",\"0.25\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert!((tx.amount - 5.0).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("BTC"));
    assert!((tx.swap_to_amount.unwrap() - 0.25).abs() < f64::EPSILON);
    assert_eq!(tx.fee_coin_symbol.as_deref(), Some("ETH"));
    assert!((tx.fee_amount.unwrap() - 0.01).abs() < f64::EPSILON);
}

#[test]
fn ledger_stablecoin_to_stablecoin_becomes_swap() {
    // USDT → USDC: both stablecoins → crypto-to-crypto swap
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-OUT\",\"REF-SS\",\"2024-04-01 10:00:00\",\"trade\",\"\",\"currency\",\"USDT\",\"-500\",\"0.10\",\"0\"\n",
        "\"TX-IN\",\"REF-SS\",\"2024-04-01 10:00:00\",\"trade\",\"\",\"currency\",\"USDC\",\"499.90\",\"0\",\"499.90\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert_eq!(tx.symbol, "USDT");
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDC"));
}

#[test]
fn ledger_spend_receive_pair_becomes_trade() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-SPEND\",\"REF-SR\",\"2024-05-10 12:00:00\",\"spend\",\"\",\"currency\",\"ZUSD\",\"-250.00\",\"2.50\",\"0\"\n",
        "\"TX-RECV\",\"REF-SR\",\"2024-05-10 12:00:00\",\"receive\",\"\",\"currency\",\"SOL\",\"1.25\",\"0\",\"1.25\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "SOL");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 1.25).abs() < f64::EPSILON);
    // Price = 250 / 1.25 = 200
    assert!((tx.price_per_coin.unwrap() - 200.0).abs() < 0.01);
    assert!((tx.fee.unwrap() - 2.5).abs() < f64::EPSILON);
}

#[test]
fn ledger_staking_reward_becomes_income() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-STK\",\"REF-STK\",\"2024-01-20 00:00:00\",\"staking\",\"reward\",\"currency\",\"DOT.S\",\"0.05\",\"0\",\"10.05\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "DOT");
    assert_eq!(tx.transaction_type, "income");
    assert_eq!(tx.subtype.as_deref(), Some("staking"));
    assert!((tx.amount - 0.05).abs() < f64::EPSILON);
}

#[test]
fn ledger_internal_staking_transfer_is_skipped() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-A\",\"REF-INT\",\"2024-01-10 00:00:00\",\"transfer\",\"spottostaking\",\"currency\",\"DOT\",\"-10\",\"0\",\"0\"\n",
        "\"TX-B\",\"REF-INT\",\"2024-01-10 00:00:00\",\"transfer\",\"stakingfromspot\",\"currency\",\"DOT.S\",\"10\",\"0\",\"10\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert!(result.items.is_empty());
}

#[test]
fn ledger_fiat_deposit_is_skipped() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX1\",\"REF1\",\"2024-01-15 10:00:00\",\"deposit\",\"\",\"currency\",\"ZUSD\",\"10000\",\"0\",\"10000\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    // Fiat rows produce None from single_row_to_transaction
    assert!(result.items.is_empty());
}

#[test]
fn ledger_fiat_trade_pair_is_skipped() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-A\",\"REF-FIAT\",\"2024-01-15 10:00:00\",\"trade\",\"\",\"currency\",\"ZUSD\",\"-1000\",\"0\",\"0\"\n",
        "\"TX-B\",\"REF-FIAT\",\"2024-01-15 10:00:00\",\"trade\",\"\",\"currency\",\"ZEUR\",\"920\",\"0\",\"920\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert!(result.items.is_empty());
}

#[test]
fn ledger_v2_format_with_subclass_and_wallet() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"subclass\",\"asset\",\"wallet\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX1\",\"REF1\",\"2024-06-01 09:00:00\",\"deposit\",\"\",\"currency\",\"\",\"XXBT\",\"spot / main\",\"1.0\",\"0\",\"1.0\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("deposit"));
}

#[test]
fn ledger_earn_reward_becomes_income_staking() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX-EARN\",\"REF-EARN\",\"2024-04-01 00:00:00\",\"earn\",\"reward\",\"currency\",\"SOL\",\"0.1\",\"0\",\"5.1\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "Kraken").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "SOL");
    assert_eq!(tx.transaction_type, "income");
    assert_eq!(tx.subtype.as_deref(), Some("staking"));
}

#[test]
fn ledger_uses_custom_wallet_name() {
    let csv = concat!(
        "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n",
        "\"TX1\",\"REF1\",\"2024-01-15 10:00:00\",\"deposit\",\"\",\"currency\",\"XXBT\",\"0.5\",\"0\",\"0.5\"\n",
    );

    let parser = KrakenLedgerParser;
    let result = parser.parse(csv, "My Kraken Account").unwrap();

    assert_eq!(result.items[0].1.wallet, "My Kraken Account");
}

// ── Trades parser ──
