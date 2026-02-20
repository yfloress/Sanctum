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
fn all_statements_buy_becomes_trade_buy() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Spot,Buy,BTC,0.5,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert_eq!(tx.wallet, "Binance");
}

#[test]
fn all_statements_sell_becomes_trade_sell() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Spot,Sell,ETH,-2.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 2.0).abs() < f64::EPSILON);
}

#[test]
fn all_statements_deposit_becomes_transfer_deposit() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-02-01 08:00:00,Spot,Deposit,BTC,1.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("deposit"));
}

#[test]
fn all_statements_withdraw_becomes_transfer_withdrawal() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-02-01 08:00:00,Spot,Withdraw,BTC,-0.5,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
}

#[test]
fn all_statements_distribution_becomes_income_airdrop() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-03-01 00:00:00,Spot,Distribution,FLR,100.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "income");
    assert_eq!(tx.subtype.as_deref(), Some("airdrop"));
}

#[test]
fn all_statements_staking_rewards_becomes_income_staking() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-03-15 00:00:00,Spot,Staking Rewards,DOT,0.05,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "income");
    assert_eq!(tx.subtype.as_deref(), Some("staking"));
}

#[test]
fn all_statements_simple_earn_flexible_interest() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-03-15 00:00:00,Spot,Simple Earn Flexible Interest,USDT,0.12,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    // USDT is not fiat in our system, so it should be parsed
    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "USDT");
    assert_eq!(tx.transaction_type, "income");
    assert_eq!(tx.subtype.as_deref(), Some("staking"));
}

#[test]
fn all_statements_fee_becomes_expense_fee() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Spot,Fee,BNB,-0.001,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "expense");
    assert_eq!(tx.subtype.as_deref(), Some("fee"));
    assert!((tx.amount - 0.001).abs() < f64::EPSILON);
}

#[test]
fn all_statements_convert_pair_becomes_swap() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,ETH,-2.0,\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,SOL,50.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("SOL"));
    assert!((tx.swap_to_amount.unwrap() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn all_statements_convert_fiat_to_crypto_becomes_buy() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,USD,-1000.0,\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,BTC,0.02,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.02).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < 0.01);
}

#[test]
fn all_statements_convert_crypto_to_fiat_becomes_sell() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,BTC,-0.5,\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,USD,25000.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
}

#[test]
fn all_statements_unpaired_convert_positive_crypto_uses_buy_fallback() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,BTC,0.02,partial export\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    assert!(result.errors.is_empty());
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.02).abs() < f64::EPSILON);
    assert!(tx.price_per_coin.is_none());
}

#[test]
fn all_statements_unpaired_convert_negative_crypto_uses_sell_fallback() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,ETH,-1.5,partial export\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    assert!(result.errors.is_empty());
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 1.5).abs() < f64::EPSILON);
    assert!(tx.price_per_coin.is_none());
}

#[test]
fn all_statements_unpaired_convert_fiat_row_is_skipped() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-04-01 12:00:00,Spot,Binance Convert,USD,-1000.0,partial export\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn all_statements_fiat_deposit_is_skipped() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:00:00,Spot,Fiat Deposit,USD,10000.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert!(result.items.is_empty());
}

#[test]
fn all_statements_internal_transfer_is_skipped() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:00:00,Spot,Transfer Between Main and Funding Wallet,BTC,-0.5,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert!(result.items.is_empty());
}

#[test]
fn all_statements_bcc_normalised_to_bch() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:00:00,Spot,Deposit,BCC,1.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].1.symbol, "BCH");
}

#[test]
fn all_statements_luna_renamed_before_cutoff() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2022-05-01 10:00:00,Spot,Buy,LUNA,100.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].1.symbol, "LUNC");
}

#[test]
fn all_statements_luna_not_renamed_after_cutoff() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2022-06-01 10:00:00,Spot,Buy,LUNA,100.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].1.symbol, "LUNA");
}

#[test]
fn all_statements_card_cashback_becomes_income_rebate() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-05-01 10:00:00,Spot,Binance Card Cashback,BNB,0.01,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "income");
    assert_eq!(tx.subtype.as_deref(), Some("rebate"));
}

#[test]
fn all_statements_card_spending_becomes_expense_payment() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-05-01 10:00:00,Spot,Binance Card Spending,BNB,-0.5,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "expense");
    assert_eq!(tx.subtype.as_deref(), Some("payment"));
}

#[test]
fn all_statements_airdrop_assets_becomes_income_airdrop() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-06-01 00:00:00,Spot,Airdrop Assets,ARB,50.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "income");
    assert_eq!(tx.subtype.as_deref(), Some("airdrop"));
}

#[test]
fn all_statements_spend_revenue_pair_becomes_trade() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-04-01 12:00:00,Spot,Transaction Spend,USDT,-100.0,\n",
        "12345,2024-04-01 12:00:00,Spot,Transaction Revenue,BTC,0.002,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    // USDT is a stablecoin (pricing currency) → USDT->BTC = buy
    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert_eq!(tx.symbol, "BTC");
    assert!((tx.amount - 0.002).abs() < 1e-8);
    // price = 100 USDT / 0.002 BTC = 50000
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < 0.01);
    assert!(tx.swap_to_symbol.is_none());
    assert!(tx.swap_to_amount.is_none());
}

#[test]
fn all_statements_zero_change_is_skipped() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:00:00,Spot,Buy,BTC,0.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert!(result.items.is_empty());
}

#[test]
fn all_statements_uses_custom_wallet_name() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:00:00,Spot,Deposit,BTC,1.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Mi Binance").unwrap();

    assert_eq!(result.items[0].1.wallet, "Mi Binance");
}

#[test]
fn all_statements_crypto_fiat_deposit_ignored() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:00:00,Spot,Deposit,USD,1000.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    // USD deposit: not skipped by should_skip (that's only for transfers/fiat ops),
    // but is_fiat check in single_row_to_transaction filters it out
    assert!(result.items.is_empty());
}

#[test]
fn all_statements_unknown_operation_handled() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:00:00,Spot,Super New Feature,BTC,0.01,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    // Positive change + unknown = income/other
    assert_eq!(tx.transaction_type, "income");
    assert_eq!(tx.subtype.as_deref(), Some("other"));
}

// ── Spot Trade History parser ──
