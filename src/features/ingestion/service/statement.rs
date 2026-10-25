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

//! Ingestion service — bank statement PDFs.
//!
//! Two steps. The preview reads the PDF into one account, marks the rows
//! the account already has and suggests a category for each row from the
//! history: a merchant categorized once is categorized the same way next
//! month. The user reviews that and sends back the rows to keep, which the
//! import writes as they are.
//!
//! Duplicates are counted, not looked up in a set: two identical transfers
//! on the same day are two movements, and only as many statement rows as
//! the account already holds for that day and amount are marked.

use std::collections::HashMap;

use chrono::NaiveDate;
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use super::{IngestionError, IngestionService};
use crate::db::Database;
use crate::features::ingestion::parsers::statement::{
    PdfError, RowProblem, StatementError, StatementFormat, StatementRow, StatementWarning,
    detect_format, parse_statement, read_pdf,
};
use crate::features::ingestion::repository::IngestionRepository;
use crate::features::ingestion::validation::validate_file_size;
use crate::models::{Account, Transaction};
use crate::services::i18n::{t, t_args};

const STATEMENT_CURRENCY: &str = "CLP";
/// The category every vault starts with, for rows the history knows nothing about.
const FALLBACK_CATEGORY: &str = "OTHER";

pub enum StatementRead {
    NeedsPassword,
    WrongPassword,
    Ready(StatementPreview),
}

pub struct StatementPreview {
    pub format: StatementFormat,
    pub period: Option<(NaiveDate, NaiveDate)>,
    pub rows: Vec<PreviewRow>,
    pub problems: Vec<RowProblem>,
    pub warnings: Vec<StatementWarning>,
}

pub struct PreviewRow {
    pub row: StatementRow,
    /// The account already has this movement.
    pub duplicate: bool,
    pub category: String,
    /// The category came from an earlier movement at the same merchant.
    pub learned: bool,
}

/// A reviewed row, ready to write.
pub struct StatementEntry {
    pub date: String,
    pub description: String,
    /// Whole pesos; negative is money out of the account.
    pub amount: i64,
    pub category: String,
    /// Makes the row a transfer with this other account (a card payment,
    /// savings) instead of income or spending.
    pub transfer_account_id: Option<String>,
}

impl IngestionService {
    pub fn preview_statement(
        &self,
        bytes: &[u8],
        password: Option<&SecretString>,
        account_id: &str,
        format: Option<StatementFormat>,
    ) -> Result<StatementRead, IngestionError> {
        validate_file_size(bytes.len()).map_err(IngestionError::FileTooLarge)?;
        let doc = match read_pdf(bytes, password.map(|p| p.expose_secret())) {
            Ok(doc) => doc,
            Err(PdfError::PasswordRequired) => return Ok(StatementRead::NeedsPassword),
            Err(PdfError::WrongPassword) => return Ok(StatementRead::WrongPassword),
            Err(PdfError::Unreadable) => return Err(parse_error("import-statement-unreadable")),
            Err(PdfError::NoText) => return Err(parse_error("import-statement-no-text")),
        };
        let format = format
            .or_else(|| detect_format(&doc))
            .ok_or_else(|| parse_error("import-statement-unknown-bank"))?;
        let statement = parse_statement(&doc, format).map_err(|e| match e {
            StatementError::WrongFormat(_) => parse_error("import-statement-wrong-bank"),
            StatementError::InvalidPeriod | StatementError::MissingPeriod => {
                parse_error("import-statement-no-period")
            }
        })?;

        self.with_db_readonly(|db| {
            let account = statement_account(db, account_id)?;
            let transactions = IngestionRepository::get_all_transactions(db)?;
            let history = CategoryHistory::new(&transactions);
            let existing = signed_movements(&transactions, &account.id);
            let duplicates = mark_duplicates(&statement.rows, &existing);
            let rows = statement
                .rows
                .into_iter()
                .zip(duplicates)
                .map(|(row, duplicate)| {
                    let learned = history.suggest(&row.description, row.amount < 0);
                    PreviewRow {
                        duplicate,
                        learned: learned.is_some(),
                        category: learned.unwrap_or_else(|| FALLBACK_CATEGORY.to_string()),
                        row,
                    }
                })
                .collect();
            Ok(StatementRead::Ready(StatementPreview {
                format: statement.format,
                period: statement.period,
                rows,
                problems: statement.problems,
                warnings: statement.warnings,
            }))
        })
    }

    /// Writes the reviewed rows into the account. Every row is validated
    /// before the first one is written, so a bad row writes nothing.
    pub fn import_statement(
        &self,
        account_id: &str,
        entries: Vec<StatementEntry>,
    ) -> Result<usize, IngestionError> {
        self.with_db(|db| {
            let account = statement_account(db, account_id)?;
            let categories = IngestionRepository::build_category_lookup(db)?;
            let accounts: HashMap<String, Account> = db
                .get_accounts()?
                .into_iter()
                .map(|a| (a.id.clone(), a))
                .collect();

            let mut writes = Vec::with_capacity(entries.len());
            for entry in &entries {
                let date = NaiveDate::parse_from_str(entry.date.trim(), "%Y-%m-%d")
                    .map_err(|_| validation_error("import-statement-invalid-row"))?;
                if entry.amount == 0 || entry.description.trim().is_empty() {
                    return Err(validation_error("import-statement-invalid-row"));
                }
                let cents = entry
                    .amount
                    .checked_abs()
                    .and_then(|v| v.checked_mul(100))
                    .ok_or_else(|| validation_error("import-statement-invalid-row"))?;

                if let Some(other_id) = entry.transfer_account_id.as_deref() {
                    let other = accounts
                        .get(other_id)
                        .filter(|o| {
                            o.id != account.id && o.currency.eq_ignore_ascii_case(&account.currency)
                        })
                        .ok_or_else(|| validation_error("import-statement-invalid-transfer"))?;
                    let (from, to) = if entry.amount < 0 {
                        (account.id.clone(), other.id.clone())
                    } else {
                        (other.id.clone(), account.id.clone())
                    };
                    writes.push(Write::Transfer {
                        from,
                        to,
                        cents,
                        date,
                        description: entry.description.trim().to_string(),
                    });
                    continue;
                }

                let kind = if entry.amount < 0 {
                    "expense"
                } else {
                    "income"
                };
                let key = (entry.category.trim().to_lowercase(), kind.to_string());
                let Some(category) = categories.get(&key) else {
                    return Err(IngestionError::Validation(t_args(
                        "import-error-category-not-found-detail",
                        &[("name", entry.category.trim()), ("type", kind)],
                    )));
                };
                writes.push(Write::Movement(Transaction::new(
                    Uuid::new_v4().to_string(),
                    account.id.clone(),
                    cents,
                    category.name.clone(),
                    entry.description.trim().to_string(),
                    date.format("%Y-%m-%d").to_string(),
                    kind.to_string(),
                    None,
                )));
            }

            for write in &writes {
                match write {
                    Write::Movement(transaction) => {
                        IngestionRepository::create_transaction(db, transaction)?
                    }
                    Write::Transfer {
                        from,
                        to,
                        cents,
                        date,
                        description,
                    } => {
                        IngestionRepository::create_transfer(
                            db,
                            from,
                            to,
                            *cents,
                            description,
                            &date.format("%Y-%m-%d").to_string(),
                        )?;
                    }
                }
            }
            Ok(writes.len())
        })
    }
}

enum Write {
    Movement(Transaction),
    Transfer {
        from: String,
        to: String,
        cents: i64,
        date: NaiveDate,
        description: String,
    },
}

fn parse_error(key: &str) -> IngestionError {
    IngestionError::Parse(t(key))
}

fn validation_error(key: &str) -> IngestionError {
    IngestionError::Validation(t(key))
}

fn statement_account(db: &Database, account_id: &str) -> Result<Account, IngestionError> {
    let account = db
        .get_account(account_id)
        .map_err(|_| validation_error("import-statement-account-missing"))?;
    if !account.currency.eq_ignore_ascii_case(STATEMENT_CURRENCY) {
        return Err(validation_error("import-statement-account-not-clp"));
    }
    Ok(account)
}

/// Every movement of the account as (date, signed cents). A transfer out
/// is negative here and a transfer in positive, as on the bank's statement.
fn signed_movements(transactions: &[Transaction], account_id: &str) -> Vec<(String, i64)> {
    transactions
        .iter()
        .filter_map(|tx| {
            let signed = if tx.account_id == account_id {
                match tx.transaction_type.as_str() {
                    "income" => tx.amount,
                    _ => -tx.amount,
                }
            } else if tx.transfer_account_id.as_deref() == Some(account_id) {
                tx.amount
            } else {
                return None;
            };
            Some((tx.date.clone(), signed))
        })
        .collect()
}

/// Marks a row as already imported while the account still has an unused
/// movement with the same date and amount.
fn mark_duplicates(rows: &[StatementRow], existing: &[(String, i64)]) -> Vec<bool> {
    let mut available: HashMap<(String, i64), usize> = HashMap::new();
    for key in existing {
        *available.entry(key.clone()).or_default() += 1;
    }
    rows.iter()
        .map(|row| {
            let key = (row.date.format("%Y-%m-%d").to_string(), row.amount * 100);
            match available.get_mut(&key) {
                Some(left) if *left > 0 => {
                    *left -= 1;
                    true
                }
                _ => false,
            }
        })
        .collect()
}

/// The same merchant from month to month: folded, without numbers (dates,
/// operation codes, installments) that change every time.
fn merchant_key(description: &str) -> String {
    let words: Vec<String> = description
        .split_whitespace()
        .filter(|w| !w.chars().any(|c| c.is_ascii_digit()))
        .map(|w| w.to_uppercase())
        .collect();
    words.join(" ")
}

/// The latest category given to each merchant, per direction.
struct CategoryHistory {
    latest: HashMap<(String, bool), (String, String)>,
}

impl CategoryHistory {
    fn new(transactions: &[Transaction]) -> Self {
        let mut latest: HashMap<(String, bool), (String, String)> = HashMap::new();
        for tx in transactions {
            let expense = match tx.transaction_type.as_str() {
                "expense" => true,
                "income" => false,
                _ => continue,
            };
            let key = merchant_key(&tx.description);
            if key.is_empty() {
                continue;
            }
            let entry = latest
                .entry((key, expense))
                .or_insert_with(|| (String::new(), String::new()));
            if tx.date >= entry.0 {
                *entry = (tx.date.clone(), tx.category.clone());
            }
        }
        Self { latest }
    }

    fn suggest(&self, description: &str, expense: bool) -> Option<String> {
        let key = merchant_key(description);
        self.latest
            .get(&(key, expense))
            .map(|(_, category)| category.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(date: &str, kind: &str, amount: i64, category: &str, description: &str) -> Transaction {
        Transaction::new(
            Uuid::new_v4().to_string(),
            "acc".into(),
            amount,
            category.into(),
            description.into(),
            date.into(),
            kind.into(),
            None,
        )
    }

    fn row(date: (i32, u32, u32), amount: i64) -> StatementRow {
        StatementRow {
            line: 0,
            date: NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap(),
            description: "Transf.".into(),
            raw_description: "Transf.".into(),
            amount,
            sign_from_keyword: false,
            installment: None,
        }
    }

    #[test]
    fn duplicates_are_counted_per_day_and_amount() {
        let existing = vec![("2026-03-19".to_string(), 1_000_000)];
        // The account has one of the two identical transfers already.
        let rows = [
            row((2026, 3, 19), 10_000),
            row((2026, 3, 19), 10_000),
            row((2026, 3, 20), 10_000),
        ];
        assert_eq!(mark_duplicates(&rows, &existing), [true, false, false]);
    }

    #[test]
    fn transfers_count_with_the_bank_sign() {
        let mut out = tx("2026-03-01", "transfer", 5_000, "", "to savings");
        out.transfer_account_id = Some("savings".into());
        let mut incoming = tx("2026-03-02", "transfer", 7_000, "", "from savings");
        incoming.account_id = "savings".into();
        incoming.transfer_account_id = Some("acc".into());
        let spent = tx("2026-03-03", "expense", 900, "FOOD", "lunch");
        let moves = signed_movements(&[out, incoming, spent], "acc");
        assert_eq!(
            moves,
            [
                ("2026-03-01".to_string(), -5_000),
                ("2026-03-02".to_string(), 7_000),
                ("2026-03-03".to_string(), -900)
            ]
        );
    }

    #[test]
    fn categories_are_learned_per_merchant_and_direction() {
        let history = CategoryHistory::new(&[
            tx(
                "2026-01-04",
                "expense",
                100,
                "SHOPPING",
                "Compra PAYU *UBER TRIP",
            ),
            tx(
                "2026-02-04",
                "expense",
                100,
                "TRANSPORT",
                "Compra PAYU *UBER TRIP",
            ),
            tx(
                "2026-02-05",
                "income",
                100,
                "GIFT",
                "Compra PAYU *UBER TRIP",
            ),
            tx("2026-02-06", "expense", 100, "FOOD", "Compra LIDER 1234"),
        ]);
        assert_eq!(
            history.suggest("Compra PAYU *UBER TRIP", true).as_deref(),
            Some("TRANSPORT")
        );
        assert_eq!(
            history.suggest("Compra PAYU *UBER TRIP", false).as_deref(),
            Some("GIFT")
        );
        // Numbers do not make a different merchant.
        assert_eq!(
            history.suggest("compra lider 9876", true).as_deref(),
            Some("FOOD")
        );
        assert_eq!(history.suggest("Compra JUMBO", true), None);
    }
}
