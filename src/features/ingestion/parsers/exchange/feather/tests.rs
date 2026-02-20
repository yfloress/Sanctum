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

fn sample_csv() -> &'static str {
    concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,in,0.500000000000,0.500000000000,0.000000000000,abc123def456abc123def456abc123def456abc123def456abc123def456abcd,,,82.50,USD\n",
        "3060000,1705398645,2024-01-16 10:30:45,0,out,-0.100030000000,0.100000000000,0.000030000000,def789abc012def789abc012def789abc012def789abc012def789abc012defg,Payment to Alice,,16.50,USD\n",
    )
}

#[test]
fn incoming_becomes_transfer_deposit() {
    let parser = FeatherParser;
    let result = parser.parse(sample_csv(), "Feather").unwrap();

    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "XMR");
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("deposit"));
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
    assert_eq!(tx.wallet, "Feather");
    // No fee for incoming
    assert!(tx.fee_coin_symbol.is_none());
    assert!(tx.fee_amount.is_none());
}

#[test]
fn outgoing_becomes_transfer_withdrawal() {
    let parser = FeatherParser;
    let result = parser.parse(sample_csv(), "Feather").unwrap();

    let tx = &result.items[1].1;
    assert_eq!(tx.symbol, "XMR");
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
    assert!((tx.amount - 0.1).abs() < f64::EPSILON);
    // Fee should be present for outgoing
    assert_eq!(tx.fee_coin_symbol.as_deref(), Some("XMR"));
    assert!((tx.fee_amount.unwrap() - 0.00003).abs() < f64::EPSILON);
}

#[test]
fn notes_contain_description_and_truncated_txid() {
    let parser = FeatherParser;
    let result = parser.parse(sample_csv(), "Feather").unwrap();

    // Second tx has a description "Payment to Alice"
    let notes = result.items[1].1.notes.as_deref().unwrap();
    assert!(notes.contains("Feather Wallet"));
    assert!(notes.contains("Desc: Payment to Alice"));
    assert!(notes.contains("TxID: def789ab...c012defg"));
    assert!(notes.contains("Block: 3060000"));
}

#[test]
fn notes_without_description() {
    let parser = FeatherParser;
    let result = parser.parse(sample_csv(), "Feather").unwrap();

    // First tx has no description
    let notes = result.items[0].1.notes.as_deref().unwrap();
    assert!(notes.contains("Feather Wallet"));
    assert!(!notes.contains("Desc:"));
    assert!(notes.contains("TxID:"));
}

#[test]
fn notes_contain_fiat_valuation() {
    let parser = FeatherParser;
    let result = parser.parse(sample_csv(), "Feather").unwrap();

    let notes = result.items[0].1.notes.as_deref().unwrap();
    assert!(notes.contains("Fiat: 82.50 USD"));

    let notes = result.items[1].1.notes.as_deref().unwrap();
    assert!(notes.contains("Fiat: 16.50 USD"));
}

#[test]
fn uses_custom_wallet_name() {
    let parser = FeatherParser;
    let result = parser.parse(sample_csv(), "Mi Monero").unwrap();

    assert_eq!(result.items[0].1.wallet, "Mi Monero");
    assert_eq!(result.items[1].1.wallet, "Mi Monero");
}

#[test]
fn date_format_is_iso() {
    let parser = FeatherParser;
    let result = parser.parse(sample_csv(), "Feather").unwrap();

    assert_eq!(result.items[0].1.date, "2024-01-15 10:30:45");
    assert_eq!(result.items[1].1.date, "2024-01-16 10:30:45");
}

#[test]
fn empty_content_produces_no_items() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn invalid_direction_produces_error() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,sideways,0.5,0.5,0,abc,,,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    assert!(result.items.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].message.contains("sideways"));
}

#[test]
fn invalid_amount_produces_error() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,in,INVALID,INVALID,0,abc,,,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    assert!(result.items.is_empty());
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn zero_amount_incoming_is_silently_skipped() {
    // Incoming with amount=0 is the receiving side of a churn/self-send.
    // It carries no value, so we skip it silently (no error, no item).
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,in,0.0,0.0,0,abc,,,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    assert!(result.items.is_empty(), "no items should be produced");
    assert!(result.errors.is_empty(), "no errors should be produced");
}

#[test]
fn churn_transaction_zero_amount_out() {
    // Churn = self-send: direction "out", amount 0, fee > 0
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,out,-0.000017740000,0.000000000000,0.000017740000,abc123,,,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    assert_eq!(result.items.len(), 1);
    assert!(result.errors.is_empty());

    let tx = &result.items[0].1;
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("churn"));
    assert!((tx.amount - 0.0).abs() < f64::EPSILON);
    assert_eq!(tx.fee_coin_symbol.as_deref(), Some("XMR"));
    let notes = tx.notes.as_deref().unwrap();
    assert!(notes.contains("Churn"));
}

#[test]
fn missing_required_column_is_fatal_error() {
    // Missing "date" column
    let csv = "blockHeight,timestamp,direction,amount,fee,txid\n";

    let parser = FeatherParser;
    let err = parser.parse(csv, "Feather").unwrap_err();

    assert!(err.message.contains("date"));
}

#[test]
fn timestamp_fallback_when_date_invalid() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,INVALID_DATE,0,in,0.5,0.5,0,abc,,,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    // Should fall back to timestamp column
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].1.date, "2024-01-15 09:50:45");
}

#[test]
fn negative_amount_is_handled() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,out,-0.5,-0.5,0.00003,abc,,,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
}

#[test]
fn incoming_with_zero_fee_has_no_fee_fields() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,in,1.0,1.0,0,abc,,,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    let tx = &result.items[0].1;
    assert!(tx.fee_coin_symbol.is_none());
    assert!(tx.fee_amount.is_none());
}

#[test]
fn multiple_transactions_parsed() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,in,1.0,1.0,0,abc1,,,165.00,USD\n",
        "3050001,1705312300,2024-01-15 10:31:40,0,out,-0.500030000000,0.5,0.00003,abc2,test,,82.50,USD\n",
        "3060000,1705398645,2024-01-16 10:30:45,0,in,2.5,2.5,0,abc3,mining,,412.50,EUR\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0].1.subtype.as_deref(), Some("deposit"));
    assert_eq!(result.items[1].1.subtype.as_deref(), Some("withdrawal"));
    assert_eq!(result.items[2].1.subtype.as_deref(), Some("deposit"));
}

#[test]
fn source_returns_feather_wallet() {
    let parser = FeatherParser;
    assert_eq!(parser.source(), ExchangeSource::FeatherWallet);
}

#[test]
fn legacy_header_still_works() {
    // Ensure backward compatibility with the old header format
    let csv = concat!(
        "blockheight,epoch,date,direction,amount,fee,txid,address,description,paymentid\n",
        "3050000,1705312245,2024-01-15 10:30:45,in,0.5,0,abc,addr,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].1.symbol, "XMR");
    assert_eq!(result.items[0].1.subtype.as_deref(), Some("deposit"));
}

#[test]
fn fiat_without_currency_defaults_to_usd() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,in,0.5,0.5,0,abc,,,100.00,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    let notes = result.items[0].1.notes.as_deref().unwrap();
    assert!(notes.contains("Fiat: 100.00 USD"));
}

#[test]
fn no_fiat_when_fiat_amount_empty() {
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,in,0.5,0.5,0,abc,,,,\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    let notes = result.items[0].1.notes.as_deref().unwrap();
    assert!(!notes.contains("Fiat:"));
}

#[test]
fn fiat_question_mark_is_skipped() {
    // Feather outputs "?" when fiat value cannot be calculated
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "3050000,1705312245,2024-01-15 10:30:45,0,in,0.5,0.5,0,abc,,,?,USD\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    let notes = result.items[0].1.notes.as_deref().unwrap();
    assert!(!notes.contains("Fiat:"));
}

#[test]
fn real_feather_export_with_quoted_iso8601z_dates() {
    // Exact format from a real Feather Wallet export: quoted fields,
    // ISO 8601 dates with trailing Z, and an invalid date row that
    // must fall back to the timestamp (Unix epoch) column.
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "1111111,1761878329,\"2020-11-30T21:21:10Z\",0,\"in\",0.004450100000,0.034450000000,0.000000880000,\"4389432db3234234sjdhf32hjldsf5140ba29cc49234234fbdcd82b37c5957cceed\",\"\",\"\",\"10.68\",\"USD\"\n",
        "2222222,1761823846,\"2020-13-30T29:59:42Z\",0,\"out\",-0.014450000000,0.034419280000,0.000000000000,\"9e2353234234232342342349515b457c01155db5fc36ac67233bbd207c5367\",\"\",\"\",\"10.63\",\"USD\"\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    // Both rows should parse (second falls back to timestamp column)
    assert_eq!(result.items.len(), 2);
    assert!(result.errors.is_empty());

    // Row 1: valid ISO 8601 date with Z suffix
    let tx1 = &result.items[0].1;
    assert_eq!(tx1.date, "2020-11-30 21:21:10");
    assert_eq!(tx1.transaction_type, "transfer");
    assert_eq!(tx1.subtype.as_deref(), Some("deposit"));
    assert!((tx1.amount - 0.034450).abs() < 1e-6);
    assert_eq!(tx1.symbol, "XMR");
    // Fee present (0.000000880000)
    assert_eq!(tx1.fee_coin_symbol.as_deref(), Some("XMR"));
    assert!((tx1.fee_amount.unwrap() - 0.00000088).abs() < 1e-12);
    // Fiat valuation in notes
    let notes1 = tx1.notes.as_deref().unwrap();
    assert!(notes1.contains("Fiat: 10.68 USD"));
    assert!(notes1.contains("Block: 1111111"));

    // Row 2: invalid date (month 13, hour 29) => falls back to timestamp 1761823846
    let tx2 = &result.items[1].1;
    assert_eq!(tx2.date, "2025-10-30 11:30:46");
    assert_eq!(tx2.transaction_type, "transfer");
    assert_eq!(tx2.subtype.as_deref(), Some("withdrawal"));
    assert!((tx2.amount - 0.03441928).abs() < 1e-8);
    // Fee is zero => no fee fields
    assert!(tx2.fee_coin_symbol.is_none());
    assert!(tx2.fee_amount.is_none());
    let notes2 = tx2.notes.as_deref().unwrap();
    assert!(notes2.contains("Fiat: 10.63 USD"));
    assert!(notes2.contains("Block: 2222222"));
}

#[test]
fn real_feather_churn_in_is_skipped_silently() {
    // Real Feather format: quoted ISO-8601 date with Z, direction "in",
    // amount 0 → receiving side of a churn/self-send, must be skipped.
    let csv = concat!(
        "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n",
        "1111111,111111111,\"2020-01-10T21:20:50Z\",0,\"in\",0.000000000000,0.000000000000,0.000490880000,\"291283921399821jhdsfkjaskjdfh92382839\",\"\",\"\",\"0.00\",\"USD\"\n",
    );

    let parser = FeatherParser;
    let result = parser.parse(csv, "Feather").unwrap();

    // Zero-amount incoming is churn-in: no items, no errors
    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}
