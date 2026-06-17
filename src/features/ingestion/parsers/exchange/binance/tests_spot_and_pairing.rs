// Sanctum — a privacy-first personal finance and crypto vault.
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
fn spot_buy_btc_usdt() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-01-15 10:30:45,BTCUSDT,BUY,50000.00,0.5BTC,25000USDT,0.001BNB\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    // USDT is a stablecoin → treated as pricing currency → buy
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert_eq!(tx.symbol, "BTC");
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
    assert!(tx.swap_to_symbol.is_none());
    assert!(tx.swap_to_amount.is_none());
    assert_eq!(tx.fee_coin_symbol.as_deref(), Some("BNB"));
    assert!((tx.fee_amount.unwrap() - 0.001).abs() < f64::EPSILON);
}

#[test]
fn spot_sell_eth_usd() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-02-01 08:00:00,ETHUSD,SELL,2000.00,2.0ETH,4000USD,3.50USD\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 2000.0).abs() < 0.01);
    assert!((tx.fee.unwrap() - 3.5).abs() < f64::EPSILON);
}

#[test]
fn spot_sell_eth_eur_does_not_set_usd_price_or_fee() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-02-01 08:00:00,ETHEUR,SELL,2000.00,2.0ETH,4000EUR,3.50EUR\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 2.0).abs() < f64::EPSILON);
    assert!(tx.price_per_coin.is_none());
    assert!(tx.fee.is_none());
    let note = tx.notes.as_deref().unwrap_or_default();
    assert!(note.contains("tax_reason=non_usd_quote:EUR"));
}

#[test]
fn spot_buy_with_fiat_quote() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-01-15 10:30:45,BTCUSD,BUY,50000.00,0.1BTC,5000USD,1.00USD\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 0.1).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < 0.01);
    assert!((tx.fee.unwrap() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn spot_sell_btc_usdt_becomes_sell() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-02-10 12:00:00,BTCUSDT,SELL,50000.00,0.5BTC,25000USDT,0.001BNB\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    // USDT is pricing currency → sell
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert!((tx.price_per_coin.unwrap() - 50000.0).abs() < f64::EPSILON);
    assert!(tx.swap_to_symbol.is_none());
    assert!(tx.swap_to_amount.is_none());
}

#[test]
fn spot_stablecoin_to_stablecoin_becomes_swap() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-04-01 12:00:00,USDCUSDT,BUY,1.0001,500USDC,500.05USDT,0.00USD\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    // Both stablecoins → swap
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert_eq!(tx.symbol, "USDT");
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDC"));
}

#[test]
fn spot_crypto_to_crypto_becomes_swap() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-03-15 14:00:00,ETHBTC,BUY,0.05,5.0ETH,0.25BTC,0.001BNB\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    // Buying ETH with BTC: outgoing=BTC, incoming=ETH
    assert_eq!(tx.symbol, "BTC");
    assert!((tx.amount - 0.25).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
    assert!((tx.swap_to_amount.unwrap() - 5.0).abs() < f64::EPSILON);
}

#[test]
fn spot_bcc_normalised_to_bch() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-01-15 10:00:00,BCCUSD,BUY,300.00,1.0BCC,300USD,0.01BCC\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].1.symbol, "BCH");
}

#[test]
fn spot_uses_custom_wallet_name() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-01-15 10:00:00,BTCUSD,BUY,50000,0.1BTC,5000USD,1USD\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Mi Binance Spot").unwrap();

    assert_eq!(result.items[0].1.wallet, "Mi Binance Spot");
}

#[test]
fn spot_empty_content() {
    let csv = "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n";

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn spot_invalid_executed_is_error() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-01-15 10:00:00,BTCUSD,BUY,50000,INVALID,5000USD,1USD\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert!(result.items.is_empty());
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn spot_invalid_side_is_error() {
    let csv = concat!(
        "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n",
        "2024-01-15 10:00:00,BTCUSDT,HOLD,50000,0.1BTC,5000USDT,0.1USDT\n",
    );

    let parser = BinanceSpotParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert!(result.items.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field.as_deref(), Some("Side"));
}

// ── Edge cases ──

#[test]
fn all_statements_multiple_operations() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-10 08:00:00,Spot,Deposit,BTC,1.0,\n",
        "12345,2024-01-11 09:00:00,Spot,Buy,ETH,5.0,\n",
        "12345,2024-01-12 10:00:00,Spot,Staking Rewards,DOT,0.5,\n",
        "12345,2024-01-13 11:00:00,Spot,Withdraw,BTC,-0.3,\n",
        "12345,2024-01-14 12:00:00,Spot,Fee,BNB,-0.001,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 5);

    // Verify types
    let types: Vec<(&str, Option<&str>)> = result
        .items
        .iter()
        .map(|(_, tx)| (tx.transaction_type.as_str(), tx.subtype.as_deref()))
        .collect();

    // Order should be deterministic by line; still check presence by type.
    assert!(types.contains(&("transfer", Some("deposit"))));
    assert!(types.contains(&("trade", Some("buy"))));
    assert!(types.contains(&("income", Some("staking"))));
    assert!(types.contains(&("transfer", Some("withdrawal"))));
    assert!(types.contains(&("expense", Some("fee"))));
}

#[test]
fn all_statements_small_assets_exchange_multi_dust() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,DOGE,-100.0,\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,ADA,-50.0,\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,BNB,0.15,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    // Two swap transactions (DOGE->BNB and ADA->BNB)
    assert_eq!(result.items.len(), 2);

    for item in &result.items {
        let tx = &item.1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("BNB"));
        // Incoming BNB (0.15) split equally across 2 dust assets
        assert_eq!(tx.swap_to_amount, Some(0.075));
    }
}

#[test]
fn all_statements_convert_same_symbol_is_skipped() {
    // A convert where both sides are the same coin (e.g. USDT -> USDT)
    // is a no-op and should produce zero transactions.
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Spot,Binance Convert,USDT,-6.20000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Binance Convert,USDT,6.20000000,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 0);
}

#[test]
#[allow(clippy::approx_constant)]
fn all_statements_convert_with_internal_transfer() {
    // Binance logs an internal Funding->Spot transfer with the same
    // "Binance Convert" label and timestamp as the real conversion.
    // The parser must filter out the internal USDT transfer and
    // correctly produce a USDT -> USDC swap.
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Funding,Binance Convert,USDT,-6.28000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Binance Convert,USDT,6.28000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Binance Convert,USDC,6.27603293,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert_eq!(tx.symbol, "USDT");
    assert!((tx.amount - 6.28).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDC"));
    assert!((tx.swap_to_amount.unwrap() - 6.27603293).abs() < 1e-8);
}

#[test]
fn all_statements_convert_pure_internal_transfer_is_skipped() {
    // If after filtering internal transfers nothing real remains,
    // the entire group is a no-op.
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Funding,Binance Convert,USDT,-10.00000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Binance Convert,USDT,10.00000000,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 0);
}

#[test]
fn all_statements_multi_dust_with_target_as_dust_source() {
    // Edge case: user has a tiny amount of BNB dust that is also being
    // converted via SmallAssetsExchange alongside other dust tokens.
    // BNB appears on BOTH sides (outgoing dust + incoming target).
    // The parser must not discard the group; DOGE -> BNB should resolve.
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,BNB,-0.00010000,\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,DOGE,-100.00000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,BNB,0.15000000,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    // DOGE -> BNB swap should be emitted.
    // BNB dust on the outgoing side is filtered as internal (same symbol
    // on both sides), which is acceptable — the amount is negligible.
    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("swap"));
    assert_eq!(tx.symbol, "DOGE");
    assert!((tx.amount - 100.0).abs() < f64::EPSILON);
    assert_eq!(tx.swap_to_symbol.as_deref(), Some("BNB"));
    assert!((tx.swap_to_amount.unwrap() - 0.15).abs() < 1e-8);
}

#[test]
fn all_statements_multi_dust_with_internal_transfer_rows() {
    // SmallAssetsExchange with an internal account transfer mixed in.
    // Funding USDT row is an internal movement — should be filtered out.
    // The two real dust conversions (DOGE, ADA -> BNB) must still resolve.
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,DOGE,-80.00000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,ADA,-40.00000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Small Assets Exchange BNB,BNB,0.20000000,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 2);
    for item in &result.items {
        let tx = &item.1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("BNB"));
        // 0.20 BNB split equally across 2 dust assets
        assert!((tx.swap_to_amount.unwrap() - 0.10).abs() < 1e-8);
    }

    let symbols: Vec<&str> = result
        .items
        .iter()
        .map(|(_, tx)| tx.symbol.as_str())
        .collect();
    assert!(symbols.contains(&"DOGE"));
    assert!(symbols.contains(&"ADA"));
}

#[test]
fn all_statements_convert_with_internal_transfer_fiat_to_crypto() {
    // Internal Funding->Spot transfer alongside a fiat-to-crypto convert.
    // USD -100 (outgoing) + USDT +100 (internal incoming) + BTC +0.002 (real).
    // After filtering: USD is non-internal outgoing (fiat), BTC is real
    // incoming. Since USD is fiat and BTC is crypto → buy.
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Funding,Binance Convert,USDT,-100.00000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Binance Convert,USDT,100.00000000,\n",
        "12345,2024-01-15 10:30:45,Spot,Binance Convert,BTC,0.00200000,\n",
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
fn all_statements_spend_revenue_same_second_distinct_remarks_do_not_cross_pair() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-04-01 12:00:00,Spot,Transaction Spend,USDT,-100.0,order-a\n",
        "12345,2024-04-01 12:00:00,Spot,Transaction Revenue,BTC,0.002,order-a\n",
        "12345,2024-04-01 12:00:00,Spot,Transaction Spend,ETH,-1.0,order-b\n",
        "12345,2024-04-01 12:00:00,Spot,Transaction Revenue,SOL,30.0,order-b\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 2);

    let btc_buy = result
        .items
        .iter()
        .find(|(_, tx)| tx.symbol == "BTC" && tx.subtype.as_deref() == Some("buy"))
        .map(|(_, tx)| tx)
        .expect("Expected BTC buy");
    assert!((btc_buy.amount - 0.002).abs() < 1e-8);
    assert!((btc_buy.price_per_coin.unwrap_or_default() - 50000.0).abs() < 0.01);

    let eth_sol_swap = result
        .items
        .iter()
        .find(|(_, tx)| {
            tx.symbol == "ETH"
                && tx.subtype.as_deref() == Some("swap")
                && tx.swap_to_symbol.as_deref() == Some("SOL")
        })
        .map(|(_, tx)| tx)
        .expect("Expected ETH->SOL swap");
    assert!((eth_sol_swap.amount - 1.0).abs() < 1e-8);
    assert!((eth_sol_swap.swap_to_amount.unwrap_or_default() - 30.0).abs() < 1e-8);
}

#[test]
fn all_statements_convert_same_second_multiple_pairs_without_remarks_pair_by_order() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-06-15 08:00:00,Spot,Binance Convert,USDT,-50.0,\n",
        "12345,2024-06-15 08:00:00,Spot,Binance Convert,BTC,0.001,\n",
        "12345,2024-06-15 08:00:00,Spot,Binance Convert,ETH,-1.0,\n",
        "12345,2024-06-15 08:00:00,Spot,Binance Convert,SOL,20.0,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 2);

    let btc_buy = result
        .items
        .iter()
        .find(|(_, tx)| tx.symbol == "BTC" && tx.subtype.as_deref() == Some("buy"))
        .map(|(_, tx)| tx)
        .expect("Expected BTC buy");
    assert!((btc_buy.amount - 0.001).abs() < 1e-8);

    let eth_sol_swap = result
        .items
        .iter()
        .find(|(_, tx)| {
            tx.symbol == "ETH"
                && tx.subtype.as_deref() == Some("swap")
                && tx.swap_to_symbol.as_deref() == Some("SOL")
        })
        .map(|(_, tx)| tx)
        .expect("Expected ETH->SOL swap");
    assert!((eth_sol_swap.amount - 1.0).abs() < 1e-8);
    assert!((eth_sol_swap.swap_to_amount.unwrap_or_default() - 20.0).abs() < 1e-8);
}

// ── P2P Trading tests ──

#[test]
fn all_statements_p2p_buy_becomes_trade_buy() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Funding,P2P Trading,USDT,6.20000000,P2P - 12345678\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "USDT");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 6.2).abs() < f64::EPSILON);
    assert!(tx.notes.as_ref().unwrap().contains("P2P Trading"));
    assert!(tx.notes.as_ref().unwrap().contains("P2P - 12345678"));
}

#[test]
fn all_statements_p2p_sell_becomes_trade_sell() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-02-20 14:00:00,Funding,P2P Trading,BTC,-0.01000000,P2P - 99887766\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "BTC");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("sell"));
    assert!((tx.amount - 0.01).abs() < f64::EPSILON);
}

#[test]
fn all_statements_c2c_transfer_becomes_trade_buy() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-03-10 08:15:00,Funding,C2C Transfer,ETH,1.50000000,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "ETH");
    assert_eq!(tx.transaction_type, "trade");
    assert_eq!(tx.subtype.as_deref(), Some("buy"));
    assert!((tx.amount - 1.5).abs() < f64::EPSILON);
}

#[test]
fn all_statements_p2p_fiat_is_skipped() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,Funding,P2P Trading,USD,100.00000000,P2P - 12345678\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    // USD is fiat — should be skipped
    assert_eq!(result.items.len(), 0);
}

// ── Transfer between sub-accounts (UM Futures, etc.) ──

#[test]
fn all_statements_um_futures_transfer_is_skipped() {
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-15 10:30:45,USD-M Futures,Transfer Between Spot Account and UM Futures Account,USDT,-20.00000000,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    // Internal transfer — should be skipped
    assert_eq!(result.items.len(), 0);
}

#[test]
fn all_statements_unknown_transfer_between_is_skipped() {
    // Any "Transfer Between..." string we haven't explicitly listed should
    // still be caught by the starts_with fallback.
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-06-01 12:00:00,Spot,Transfer Between Spot and Some New Account,BTC,0.10000000,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    assert_eq!(result.items.len(), 0);
}

#[test]
fn all_statements_mixed_p2p_convert_transfer() {
    // Realistic scenario: P2P buy, then convert, then internal transfer
    let csv = concat!(
        "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n",
        "12345,2024-01-10 09:00:00,Funding,P2P Trading,USDT,100.00000000,P2P - 111\n",
        "12345,2024-01-10 10:00:00,Spot,Binance Convert,USDT,-50.00000000,\n",
        "12345,2024-01-10 10:00:00,Spot,Binance Convert,BTC,0.00120000,\n",
        "12345,2024-01-10 11:00:00,USD-M Futures,Transfer Between Spot Account and UM Futures Account,USDT,-20.00000000,\n",
    );

    let parser = BinanceAllStatementsParser;
    let result = parser.parse(csv, "Binance").unwrap();

    // P2P buy (1) + Convert USDT->BTC = buy (1) + Transfer skipped (0) = 2
    assert_eq!(result.items.len(), 2);

    // Find the P2P trade
    let p2p = result
        .items
        .iter()
        .find(|(_, tx)| tx.notes.as_ref().is_some_and(|n| n.contains("P2P Trading")))
        .map(|(_, tx)| tx)
        .expect("Should have a P2P transaction");
    assert_eq!(p2p.symbol, "USDT");
    assert_eq!(p2p.subtype.as_deref(), Some("buy"));
    assert!((p2p.amount - 100.0).abs() < f64::EPSILON);

    // Find the convert (USDT is a pricing currency → USDT->BTC = buy)
    let convert = result
        .items
        .iter()
        .find(|(_, tx)| {
            tx.notes
                .as_ref()
                .is_some_and(|n| n.contains("Binance Convert"))
        })
        .map(|(_, tx)| tx)
        .expect("Should have a Convert transaction");
    assert_eq!(convert.symbol, "BTC");
    assert_eq!(convert.subtype.as_deref(), Some("buy"));
    assert!((convert.amount - 0.0012).abs() < 1e-8);
    // price = 50 USDT / 0.0012 BTC ≈ 41666.67
    assert!(convert.price_per_coin.is_some());
    assert!(convert.swap_to_symbol.is_none());
    assert!(convert.swap_to_amount.is_none());
}
