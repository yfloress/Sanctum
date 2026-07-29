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
/// Amounts are written in major units with two decimals so a spreadsheet reads
/// them as numbers. Rows whose account no longer exists keep their data with
/// the account and currency columns left empty.
pub fn transactions_to_csv(transactions: &[Transaction], accounts: &[Account]) -> String {
    let by_id: HashMap<&str, &Account> = accounts.iter().map(|a| (a.id.as_str(), a)).collect();

    let mut out = String::from(HEADER);
    for tx in transactions {
        let account = by_id.get(tx.account_id.as_str());
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            csv_escape(&tx.date),
            csv_escape(&tx.transaction_type),
            csv_escape(account.map(|a| a.name.as_str()).unwrap_or_default()),
            csv_escape(account.map(|a| a.currency.as_str()).unwrap_or_default()),
            csv_escape(&tx.category),
            csv_escape(&tx.description),
            format_cents(tx.amount),
        ));
    }
    out
}

/// Formats a cent amount as a plain signed decimal.
fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
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
    fn resolves_the_account_name_and_currency() {
        let accounts = vec![account("a1", "Checking", "CLP")];
        let rows = vec![transaction("a1", 123_456, "Food", "Lunch")];

        let csv = transactions_to_csv(&rows, &accounts);

        assert_eq!(
            csv,
            format!("{HEADER}2026-07-29,expense,Checking,CLP,Food,Lunch,1234.56\n")
        );
    }

    #[test]
    fn leaves_account_columns_empty_when_the_account_is_gone() {
        let rows = vec![transaction("missing", 100, "Food", "Lunch")];

        let csv = transactions_to_csv(&rows, &[]);

        assert!(csv.ends_with("2026-07-29,expense,,,Food,Lunch,1.00\n"));
    }

    #[test]
    fn quotes_fields_containing_separators() {
        let accounts = vec![account("a1", "Bank, N.A.", "USD")];
        let rows = vec![transaction("a1", -50, "Misc", "He said \"hi\"")];

        let csv = transactions_to_csv(&rows, &accounts);

        assert!(csv.contains("\"Bank, N.A.\""));
        assert!(csv.contains("\"He said \"\"hi\"\"\""));
        assert!(csv.ends_with("-0.50\n"));
    }

    #[test]
    fn formats_cents_with_two_decimals() {
        assert_eq!(format_cents(0), "0.00");
        assert_eq!(format_cents(5), "0.05");
        assert_eq!(format_cents(-5), "-0.05");
        assert_eq!(format_cents(100), "1.00");
        assert_eq!(format_cents(i64::MIN), "-92233720368547758.08");
    }
}
