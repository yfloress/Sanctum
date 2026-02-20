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
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "1111111,1610274075,2021-01-10 10:20:15,in,0.030000000000,30000000000,0.000000000000,abc123def456abc123def456abc123def456abc123def456abc123def456abcd,\"\",0,,\"\"\n",
        "2222222,1634753387,2021-10-20 20:29:47,out,0.100000000000,100000000000,0.000040000000,def789abc012def789abc012def789abc012def789abc012def789abc012defg,\"My Label\",0,,\"Payment for services\"\n",
    )
}

#[test]
fn incoming_becomes_transfer_deposit() {
    let parser = MoneroGuiParser;
    let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

    let tx = &result.items[0].1;
    assert_eq!(tx.symbol, "XMR");
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("deposit"));
    assert!((tx.amount - 0.03).abs() < f64::EPSILON);
    assert_eq!(tx.wallet, "Monero GUI");
    // No fee for incoming (fee = 0)
    assert!(tx.fee_coin_symbol.is_none());
    assert!(tx.fee_amount.is_none());
}

#[test]
fn outgoing_becomes_transfer_withdrawal() {
    let parser = MoneroGuiParser;
    let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

    let tx = &result.items[1].1;
    assert_eq!(tx.symbol, "XMR");
    assert_eq!(tx.transaction_type, "transfer");
    assert_eq!(tx.subtype.as_deref(), Some("withdrawal"));
    assert!((tx.amount - 0.1).abs() < f64::EPSILON);
    // Fee should be present for outgoing
    assert_eq!(tx.fee_coin_symbol.as_deref(), Some("XMR"));
    assert!((tx.fee_amount.unwrap() - 0.00004).abs() < f64::EPSILON);
}

#[test]
fn notes_contain_description_label_and_truncated_txid() {
    let parser = MoneroGuiParser;
    let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

    // Second tx has description and label
    let notes = result.items[1].1.notes.as_deref().unwrap();
    assert!(notes.contains("Monero GUI"));
    assert!(notes.contains("Desc: Payment for services"));
    assert!(notes.contains("Label: My Label"));
    assert!(notes.contains("TxID: def789ab...c012defg"));
    assert!(notes.contains("Block: 2222222"));
}

#[test]
fn notes_without_description_or_label() {
    let parser = MoneroGuiParser;
    let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

    // First tx has empty description and label
    let notes = result.items[0].1.notes.as_deref().unwrap();
    assert!(notes.contains("Monero GUI"));
    assert!(!notes.contains("Desc:"));
    assert!(!notes.contains("Label:"));
    assert!(notes.contains("TxID:"));
}

#[test]
fn uses_custom_wallet_name() {
    let parser = MoneroGuiParser;
    let result = parser.parse(sample_csv(), "Mi Monero").unwrap();

    assert_eq!(result.items[0].1.wallet, "Mi Monero");
    assert_eq!(result.items[1].1.wallet, "Mi Monero");
}

#[test]
fn date_format_is_iso() {
    let parser = MoneroGuiParser;
    let result = parser.parse(sample_csv(), "Monero GUI").unwrap();

    assert_eq!(result.items[0].1.date, "2021-01-10 10:20:15");
    assert_eq!(result.items[1].1.date, "2021-10-20 20:29:47");
}

#[test]
fn empty_content_produces_no_items() {
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn invalid_direction_produces_error() {
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "3050000,1705312245,2024-01-15 10:30:45,sideways,0.5,500000000000,0,abc,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    assert!(result.items.is_empty());
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn invalid_amount_produces_error() {
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "3050000,1705312245,2024-01-15 10:30:45,in,INVALID,0,0,abc,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    assert!(result.items.is_empty());
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn zero_amount_incoming_is_skipped_silently() {
    // Zero amount for incoming is the receiving side of a churn/self-send;
    // it carries no value, so we skip it silently (no error, no item).
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "3050000,1705312245,2024-01-15 10:30:45,in,0.000000000000,0,0.00004,abc,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn churn_transaction_zero_amount_out() {
    // Churn = self-send: direction "out", amount 0, fee > 0
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "3050000,1705312245,2024-01-15 10:30:45,out,0.000000000000,0,0.000017740000,abc123,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

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
    let csv = "blockHeight,epoch,direction,amount,fee,txid\n";

    let parser = MoneroGuiParser;
    let err = parser.parse(csv, "Monero GUI").unwrap_err();

    assert!(err.message.contains("date"));
}

#[test]
fn epoch_fallback_when_date_invalid() {
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "3050000,1705312245,INVALID_DATE,in,0.5,500000000000,0,abc,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    // Should fall back to epoch timestamp
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].1.date, "2024-01-15 09:50:45");
}

#[test]
fn negative_amount_is_handled() {
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "3050000,1705312245,2024-01-15 10:30:45,out,-0.5,500000000000,0.00003,abc,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    assert_eq!(result.items.len(), 1);
    let tx = &result.items[0].1;
    assert!((tx.amount - 0.5).abs() < f64::EPSILON);
}

#[test]
fn incoming_with_zero_fee_has_no_fee_fields() {
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "3050000,1705312245,2024-01-15 10:30:45,in,1.0,1000000000000,0,abc,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    let tx = &result.items[0].1;
    assert!(tx.fee_coin_symbol.is_none());
    assert!(tx.fee_amount.is_none());
}

#[test]
fn multiple_transactions_parsed() {
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "3050000,1705312245,2024-01-15 10:30:45,in,1.0,1000000000000,0,abc1,\"\",0,,\"\"\n",
        "3050001,1705312300,2024-01-15 10:31:40,out,0.5,500000000000,0.00003,abc2,\"addr2\",0,,\"test\"\n",
        "3060000,1705398645,2024-01-16 10:30:45,in,2.5,2500000000000,0,abc3,\"\",0,,\"mining\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0].1.subtype.as_deref(), Some("deposit"));
    assert_eq!(result.items[1].1.subtype.as_deref(), Some("withdrawal"));
    assert_eq!(result.items[2].1.subtype.as_deref(), Some("deposit"));
}

#[test]
fn source_returns_monero_gui_wallet() {
    let parser = MoneroGuiParser;
    assert_eq!(parser.source(), ExchangeSource::MoneroGuiWallet);
}

#[test]
fn real_monero_gui_export_with_quoted_fields() {
    // Simulates the exact format from a real Monero GUI export:
    // quoted empty strings, integer atomicAmount, etc.
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "1111111,2222222222,2021-01-10 10:20:15,in,0.030000000000,3000000000,0.000000000000,kjhsahf8923h98fh32fhoiuhsaf923hf98fjasdkjfk,\"\",0,,\"\"\n",
        "1111111,2222222222,2021-10-20 20:29:47,out,0.034419280000,34419280000,0.000040000000,9e2353234234232342342349515b457c01155db5fc36ac67233bbd207c5367,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    assert_eq!(result.items.len(), 2);
    assert!(result.errors.is_empty());

    // Row 1: incoming
    let tx1 = &result.items[0].1;
    assert_eq!(tx1.date, "2021-01-10 10:20:15");
    assert_eq!(tx1.transaction_type, "transfer");
    assert_eq!(tx1.subtype.as_deref(), Some("deposit"));
    assert!((tx1.amount - 0.03).abs() < 1e-12);
    assert_eq!(tx1.symbol, "XMR");
    // Fee = 0 for incoming
    assert!(tx1.fee_coin_symbol.is_none());

    // Row 2: outgoing with fee
    let tx2 = &result.items[1].1;
    assert_eq!(tx2.date, "2021-10-20 20:29:47");
    assert_eq!(tx2.transaction_type, "transfer");
    assert_eq!(tx2.subtype.as_deref(), Some("withdrawal"));
    assert!((tx2.amount - 0.03441928).abs() < 1e-12);
    assert_eq!(tx2.fee_coin_symbol.as_deref(), Some("XMR"));
    assert!((tx2.fee_amount.unwrap() - 0.00004).abs() < 1e-12);
}

#[test]
fn zero_amount_incoming_from_real_export() {
    // The user's real data had a row with amount=0 and direction=in.
    // This is the receiving side of a churn/self-send and should be skipped silently.
    let csv = concat!(
        "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n",
        "1111111,2222222222,2021-10-20 20:29:47,in,0.000000000000,0,0.0004000000,9e235abc,\"\",0,,\"\"\n",
    );

    let parser = MoneroGuiParser;
    let result = parser.parse(csv, "Monero GUI").unwrap();

    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}
