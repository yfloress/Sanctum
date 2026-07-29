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

use super::*;

#[test]
fn parse_timestamp_standard() {
    let dt = parse_timestamp("2024-01-15 10:30:45").unwrap();
    assert_eq!(dt.to_string(), "2024-01-15 10:30:45");
}

#[test]
fn parse_timestamp_fractional() {
    let dt = parse_timestamp("2024-01-15 10:30:45.1234").unwrap();
    assert_eq!(
        dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        "2024-01-15 10:30:45"
    );
}

#[test]
fn parse_timestamp_iso8601() {
    let dt = parse_timestamp("2024-01-15T10:30:45").unwrap();
    assert_eq!(dt.to_string(), "2024-01-15 10:30:45");
}

#[test]
fn parse_timestamp_iso8601_with_z() {
    // Feather Wallet uses this format: "2020-11-30T21:21:10Z"
    let dt = parse_timestamp("2020-11-30T21:21:10Z").unwrap();
    assert_eq!(dt.to_string(), "2020-11-30 21:21:10");
}

#[test]
fn parse_timestamp_iso8601_with_z_quoted() {
    let dt = parse_timestamp("\"2020-11-30T21:21:10Z\"").unwrap();
    assert_eq!(dt.to_string(), "2020-11-30 21:21:10");
}

#[test]
fn parse_timestamp_date_only() {
    let dt = parse_timestamp("2024-01-15").unwrap();
    assert_eq!(dt.to_string(), "2024-01-15 00:00:00");
}

#[test]
fn parse_timestamp_quoted() {
    let dt = parse_timestamp("\"2024-01-15 10:30:45\"").unwrap();
    assert_eq!(dt.to_string(), "2024-01-15 10:30:45");
}

#[test]
fn parse_timestamp_empty_is_none() {
    assert!(parse_timestamp("").is_none());
    assert!(parse_timestamp("  ").is_none());
}

#[test]
fn parse_timestamp_garbage_is_none() {
    assert!(parse_timestamp("not-a-date").is_none());
}

// ── Kraken currency ──

#[test]
fn normalize_kraken_btc_variants() {
    assert_eq!(normalize_kraken_currency("XXBT"), "BTC");
    assert_eq!(normalize_kraken_currency("XBT"), "BTC");
}

#[test]
fn normalize_kraken_fiat() {
    assert_eq!(normalize_kraken_currency("ZUSD"), "USD");
    assert_eq!(normalize_kraken_currency("ZEUR"), "EUR");
}

#[test]
fn normalize_kraken_staked_suffix() {
    assert_eq!(normalize_kraken_currency("ETH2.S"), "ETH2");
    assert_eq!(normalize_kraken_currency("DOT.S"), "DOT");
    assert_eq!(normalize_kraken_currency("SOL.M"), "SOL");
}

#[test]
fn normalize_kraken_passthrough() {
    assert_eq!(normalize_kraken_currency("ADA"), "ADA");
    assert_eq!(normalize_kraken_currency("SOL"), "SOL");
}

#[test]
fn normalize_kraken_crypto_prefixed() {
    assert_eq!(normalize_kraken_currency("XETH"), "ETH");
    assert_eq!(normalize_kraken_currency("XXMR"), "XMR");
    assert_eq!(normalize_kraken_currency("XXRP"), "XRP");
    assert_eq!(normalize_kraken_currency("XXLM"), "XLM");
    assert_eq!(normalize_kraken_currency("XZEC"), "ZEC");
    assert_eq!(normalize_kraken_currency("XXDG"), "DOGE");
    assert_eq!(normalize_kraken_currency("XDG"), "DOGE");
    assert_eq!(normalize_kraken_currency("XETC"), "ETC");
    assert_eq!(normalize_kraken_currency("XMLN"), "MLN");
    assert_eq!(normalize_kraken_currency("XREP"), "REP");
    assert_eq!(normalize_kraken_currency("XLTC"), "LTC");
}

// ── Binance currency ──

#[test]
fn normalize_binance_bcc_to_bch() {
    assert_eq!(normalize_binance_currency("BCC"), "BCH");
}

#[test]
fn normalize_binance_nano_to_xno() {
    assert_eq!(normalize_binance_currency("NANO"), "XNO");
}

#[test]
fn normalize_binance_passthrough() {
    assert_eq!(normalize_binance_currency("BTC"), "BTC");
}

#[test]
fn luna_rename_before_cutoff() {
    let dt = parse_timestamp("2022-05-01 00:00:00").unwrap();
    assert!(should_rename_luna_to_lunc(dt));
}

#[test]
fn luna_rename_after_cutoff() {
    let dt = parse_timestamp("2022-06-01 00:00:00").unwrap();
    assert!(!should_rename_luna_to_lunc(dt));
}

// ── Fiat detection ──

#[test]
fn fiat_detected() {
    assert!(is_fiat("USD"));
    assert!(is_fiat("EUR"));
    assert!(is_fiat("CLP"));
    assert!(is_fiat("FEE"));
}

#[test]
fn crypto_not_fiat() {
    assert!(!is_fiat("BTC"));
    assert!(!is_fiat("ETH"));
    assert!(!is_fiat("XMR"));
}

// ── Stablecoin detection ──

#[test]
fn stablecoin_detected() {
    assert!(is_stablecoin("USDT"));
    assert!(is_stablecoin("USDC"));
    assert!(is_stablecoin("BUSD"));
    assert!(is_stablecoin("DAI"));
    assert!(is_stablecoin("FDUSD"));
}

#[test]
fn fiat_is_not_stablecoin() {
    assert!(!is_stablecoin("USD"));
    assert!(!is_stablecoin("EUR"));
}

#[test]
fn crypto_is_not_stablecoin() {
    assert!(!is_stablecoin("BTC"));
    assert!(!is_stablecoin("ETH"));
    assert!(!is_stablecoin("BNB"));
}

// ── Quote currency detection ──

#[test]
fn quote_currency_includes_fiat() {
    assert!(is_quote_currency("USD"));
    assert!(is_quote_currency("EUR"));
}

#[test]
fn quote_currency_includes_stablecoins() {
    assert!(is_quote_currency("USDT"));
    assert!(is_quote_currency("USDC"));
    assert!(is_quote_currency("BUSD"));
}

#[test]
fn quote_currency_excludes_crypto() {
    assert!(!is_quote_currency("BTC"));
    assert!(!is_quote_currency("ETH"));
    assert!(!is_quote_currency("BNB"));
}

// ── Decimal parsing ──

#[test]
fn parse_decimal_standard() {
    assert_eq!(parse_decimal("1234.56"), Some(1234.56));
}

#[test]
fn parse_decimal_comma_separator() {
    assert_eq!(parse_decimal("1234,56"), Some(1234.56));
}

#[test]
fn parse_decimal_negative() {
    assert_eq!(parse_decimal("-0.5"), Some(-0.5));
}

#[test]
fn parse_decimal_quoted() {
    assert_eq!(parse_decimal("\"100.0\""), Some(100.0));
}

#[test]
fn parse_decimal_empty_is_none() {
    assert!(parse_decimal("").is_none());
    assert!(parse_decimal("  ").is_none());
}

// ── Amount with unit ──

#[test]
fn parse_amount_with_unit_no_space() {
    let (qty, unit) = parse_amount_with_unit("0.001BNB").unwrap();
    assert!((qty - 0.001).abs() < f64::EPSILON);
    assert_eq!(unit, "BNB");
}

#[test]
fn parse_amount_with_unit_with_space() {
    let (qty, unit) = parse_amount_with_unit("25000 USDT").unwrap();
    assert!((qty - 25000.0).abs() < f64::EPSILON);
    assert_eq!(unit, "USDT");
}

#[test]
fn parse_amount_with_unit_empty_is_none() {
    assert!(parse_amount_with_unit("").is_none());
}

// ── Kraken pair parsing ──

#[test]
fn parse_kraken_pair_slash_separated() {
    let (base, quote) = parse_kraken_pair("BTC/USD").unwrap();
    assert_eq!(base, "BTC");
    assert_eq!(quote, "USD");
}

#[test]
fn parse_kraken_pair_slash_with_prefixes() {
    let (base, quote) = parse_kraken_pair("XXBT/ZUSD").unwrap();
    assert_eq!(base, "BTC");
    assert_eq!(quote, "USD");
}

#[test]
fn parse_kraken_pair_concatenated() {
    let (base, quote) = parse_kraken_pair("XXBTZUSD").unwrap();
    assert_eq!(base, "BTC");
    assert_eq!(quote, "USD");
}

#[test]
fn parse_kraken_pair_eth_eur() {
    let (base, quote) = parse_kraken_pair("XETHZEUR").unwrap();
    assert_eq!(base, "ETH");
    assert_eq!(quote, "EUR");
}

#[test]
fn parse_kraken_pair_crypto_crypto() {
    let (base, quote) = parse_kraken_pair("XETHXXBT").unwrap();
    assert_eq!(base, "ETH");
    assert_eq!(quote, "BTC");
}

#[test]
fn parse_kraken_pair_simple_suffix() {
    let (base, quote) = parse_kraken_pair("ADAUSD").unwrap();
    assert_eq!(base, "ADA");
    assert_eq!(quote, "USD");
}

#[test]
fn parse_kraken_pair_usdt_suffix() {
    let (base, quote) = parse_kraken_pair("BTCUSDT").unwrap();
    assert_eq!(base, "BTC");
    assert_eq!(quote, "USDT");
}

// ── Miscellaneous ──

#[test]
fn non_empty_trims_and_strips_quotes() {
    assert_eq!(non_empty("  \"hello\"  "), Some("hello"));
    assert_eq!(non_empty(""), None);
    assert_eq!(non_empty("  "), None);
    assert_eq!(non_empty("\"\""), None);
}

#[test]
fn format_date_produces_iso() {
    let dt = parse_timestamp("2024-01-15 10:30:45").unwrap();
    assert_eq!(format_date(dt), "2024-01-15");
}

#[test]
fn format_datetime_produces_full() {
    let dt = parse_timestamp("2024-01-15 10:30:45").unwrap();
    assert_eq!(format_datetime(dt), "2024-01-15 10:30:45");
}

#[test]
fn normalize_header_collapses_case_space_and_punctuation() {
    // Spacing, casing and underscore variants all collapse to one key.
    assert_eq!(
        normalize_header("Average Filled Price"),
        "averagefilledprice"
    );
    assert_eq!(
        normalize_header("average_filled_price"),
        "averagefilledprice"
    );
    assert_eq!(
        normalize_header("  averagefilledprice  "),
        "averagefilledprice"
    );
    // Punctuation and parenthesised suffixes are stripped, digits are kept.
    assert_eq!(normalize_header("Date(UTC)"), "dateutc");
    assert_eq!(
        normalize_header("create_time(UTC+00:00)"),
        "createtimeutc0000"
    );
    assert_eq!(normalize_header("Fee-payment Crypto"), "feepaymentcrypto");
    assert_eq!(normalize_header("P/L"), "pl");
    // Quotes and empty input.
    assert_eq!(normalize_header("\"UID\""), "uid");
    assert_eq!(normalize_header(""), "");
}
