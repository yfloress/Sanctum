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

//! Transaction ledger export to CSV.

use crate::core::csv_escape;
use crate::models::{Account, Transaction};
use std::collections::HashMap;

const HEADER: &str = "date,type,account,currency,category,description,amount\n";

/// Renders the ledger as CSV, resolving each row's account by id.
///
/// Amounts are signed so that the column sums to the real change in balance:
/// expenses and the outgoing leg of a transfer are negative, everything else is
/// positive. Rows whose account no longer exists keep their data with the
/// account and currency columns left empty.
pub fn transactions_to_csv(transactions: &[Transaction], accounts: &[Account]) -> String {
    let by_id: HashMap<&str, &Account> = accounts.iter().map(|a| (a.id.as_str(), a)).collect();

    let mut out = String::from(HEADER);
    for tx in transactions {
        match tx.transaction_type.as_str() {
            // A transfer is stored as one row moving money out of `account_id`
            // into `transfer_account_id`. Writing both legs keeps the column
            // summable — the pair nets to zero — and keeps per-account
            // subtotals correct.
            "transfer" => {
                push_row(&mut out, &by_id, tx, &tx.account_id, true);
                if let Some(destination) = &tx.transfer_account_id {
                    push_row(&mut out, &by_id, tx, destination, false);
                }
            }
            "expense" => push_row(&mut out, &by_id, tx, &tx.account_id, true),
            _ => push_row(&mut out, &by_id, tx, &tx.account_id, false),
        }
    }
    out
}

fn push_row(
    out: &mut String,
    accounts: &HashMap<&str, &Account>,
    tx: &Transaction,
    account_id: &str,
    outflow: bool,
) {
    let account = accounts.get(account_id);
    out.push_str(&format!(
        "{},{},{},{},{},{},{}\n",
        csv_escape(&tx.date),
        csv_escape(&tx.transaction_type),
        csv_escape(account.map(|a| a.name.as_str()).unwrap_or_default()),
        csv_escape(account.map(|a| a.currency.as_str()).unwrap_or_default()),
        csv_escape(&tx.category),
        csv_escape(&tx.description),
        format_cents(tx.amount, outflow),
    ));
}

/// Formats a cent amount as a signed decimal.
///
/// Uses `unsigned_abs` rather than negating, so `i64::MIN` cannot overflow.
fn format_cents(cents: i64, outflow: bool) -> String {
    let abs = cents.unsigned_abs();
    let negative = outflow != (cents < 0);
    let sign = if negative && abs != 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", abs / 100, abs % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, name: &str, currency: &str) -> Account {
        Account {
            id: id.to_string(),
            name: name.to_string(),
            account_type: "bank".to_string(),
            currency: currency.to_string(),
            initial_balance: 0,
            color: "#000000".to_string(),
            icon: None,
            is_archived: false,
            created_at: "2026-01-01".to_string(),
        }
    }

    fn transaction(
        account_id: &str,
        amount: i64,
        category: &str,
        description: &str,
    ) -> Transaction {
        Transaction {
            id: "tx".to_string(),
            account_id: account_id.to_string(),
            amount,
            category: category.to_string(),
            description: description.to_string(),
            date: "2026-07-29".to_string(),
            transaction_type: "expense".to_string(),
            transfer_account_id: None,
        }
    }

    #[test]
    fn writes_header_only_when_there_are_no_rows() {
        assert_eq!(transactions_to_csv(&[], &[]), HEADER);
    }

    #[test]
    fn writes_expenses_as_negative_amounts() {
        let accounts = vec![account("a1", "Checking", "CLP")];
        let rows = vec![transaction("a1", 123_456, "Food", "Lunch")];

        let csv = transactions_to_csv(&rows, &accounts);

        assert_eq!(
            csv,
            format!("{HEADER}2026-07-29,expense,Checking,CLP,Food,Lunch,-1234.56\n")
        );
    }

    #[test]
    fn writes_income_as_a_positive_amount() {
        let accounts = vec![account("a1", "Savings", "USD")];
        let mut tx = transaction("a1", 100_000, "Salary", "");
        tx.transaction_type = "income".to_string();

        let csv = transactions_to_csv(&[tx], &accounts);

        assert!(csv.ends_with("2026-07-29,income,Savings,USD,Salary,,1000.00\n"));
    }

    #[test]
    fn splits_a_transfer_into_two_legs_that_net_to_zero() {
        let accounts = vec![
            account("a1", "Checking", "USD"),
            account("a2", "Savings", "USD"),
        ];
        let mut tx = transaction("a1", 50_000, "Transfer", "Move savings");
        tx.transaction_type = "transfer".to_string();
        tx.transfer_account_id = Some("a2".to_string());

        let csv = transactions_to_csv(&[tx], &accounts);

        let rows: Vec<&str> = csv.lines().skip(1).collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            "2026-07-29,transfer,Checking,USD,Transfer,Move savings,-500.00"
        );
        assert_eq!(
            rows[1],
            "2026-07-29,transfer,Savings,USD,Transfer,Move savings,500.00"
        );
    }

    #[test]
    fn writes_only_the_outgoing_leg_when_the_transfer_has_no_destination() {
        let mut tx = transaction("a1", 100, "Transfer", "");
        tx.transaction_type = "transfer".to_string();

        let csv = transactions_to_csv(&[tx], &[]);

        assert_eq!(csv.lines().skip(1).count(), 1);
        assert!(csv.ends_with("-1.00\n"));
    }

    #[test]
    fn leaves_account_columns_empty_when_the_account_is_gone() {
        let rows = vec![transaction("missing", 100, "Food", "Lunch")];

        let csv = transactions_to_csv(&rows, &[]);

        assert!(csv.ends_with("2026-07-29,expense,,,Food,Lunch,-1.00\n"));
    }

    #[test]
    fn quotes_fields_containing_separators() {
        let accounts = vec![account("a1", "Bank, N.A.", "USD")];
        let rows = vec![transaction("a1", 50, "Misc", "He said \"hi\"")];

        let csv = transactions_to_csv(&rows, &accounts);

        assert!(csv.contains("\"Bank, N.A.\""));
        assert!(csv.contains("\"He said \"\"hi\"\"\""));
        assert!(csv.ends_with("-0.50\n"));
    }

    #[test]
    fn formats_cents_with_two_decimals() {
        assert_eq!(format_cents(0, false), "0.00");
        assert_eq!(format_cents(0, true), "0.00", "zero never carries a sign");
        assert_eq!(format_cents(5, false), "0.05");
        assert_eq!(format_cents(5, true), "-0.05");
        assert_eq!(format_cents(100, false), "1.00");
        assert_eq!(format_cents(i64::MIN, false), "-92233720368547758.08");
        assert_eq!(
            format_cents(i64::MIN, true),
            "92233720368547758.08",
            "an outflow of a negative amount is an inflow"
        );
    }
}
