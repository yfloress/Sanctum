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

use std::collections::{HashMap, HashSet};

use chrono::{Datelike, NaiveDate, Weekday};
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
    /// An earlier movement with the same description was a transfer with
    /// this account, so this one probably is too.
    pub transfer_account_id: Option<String>,
    /// The category or transfer came from an earlier movement at the same
    /// merchant.
    pub learned: bool,
    /// A movement already in the other account mirrors this row (same
    /// amount, opposite direction, a few days apart). Importing the row as
    /// a transfer turns that movement into the transfer instead of adding a
    /// second one.
    pub merge_transaction_id: Option<String>,
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
    /// The other account's movement to turn into this transfer, from the
    /// preview's `merge_transaction_id`.
    pub merge_transaction_id: Option<String>,
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
            let history = History::new(&transactions);
            let existing = signed_movements(&transactions, &account.id);
            let duplicates = mark_duplicates(&statement.rows, &existing);
            let others: Vec<Account> = db
                .get_accounts()?
                .into_iter()
                .filter(|a| {
                    a.id != account.id
                        && !a.is_archived
                        && a.currency.eq_ignore_ascii_case(STATEMENT_CURRENCY)
                })
                .collect();
            let suggestions: Vec<Option<Suggestion>> = statement
                .rows
                .iter()
                .map(|row| history.suggest(&row.description, &account.id, row.amount < 0))
                .collect();
            // Rows already settled (in the account, or a learned transfer)
            // must not claim another account's movement.
            let settled: Vec<bool> = duplicates
                .iter()
                .zip(&suggestions)
                .map(|(dup, s)| *dup || matches!(s, Some(Suggestion::Transfer(_))))
                .collect();
            let mirrors = match_mirrors(&statement.rows, &settled, &transactions, &others);
            let card = card_account(&account, &others);

            let rows = statement
                .rows
                .into_iter()
                .zip(duplicates)
                .zip(suggestions)
                .zip(mirrors)
                .map(|(((row, duplicate), learned), mirror)| {
                    let mut category = FALLBACK_CATEGORY.to_string();
                    let mut transfer_account_id = None;
                    match &learned {
                        Some(Suggestion::Category(name)) => category = name.clone(),
                        Some(Suggestion::Transfer(other)) => {
                            transfer_account_id = Some(other.clone())
                        }
                        None => {}
                    }
                    let mut merge_transaction_id = None;
                    if let Some((tx_id, other)) = mirror {
                        transfer_account_id = Some(other);
                        merge_transaction_id = Some(tx_id);
                    } else if transfer_account_id.is_none()
                        && !duplicate
                        && is_card_payment(&row)
                        && let Some(card) = card
                    {
                        transfer_account_id = Some(card.to_string());
                    }
                    PreviewRow {
                        duplicate,
                        learned: learned.is_some(),
                        category,
                        transfer_account_id,
                        merge_transaction_id,
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

    /// Writes the reviewed rows into the account, already reconciled. Every
    /// row is validated before the first one is written, so a bad row writes
    /// nothing.
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
            let transactions: HashMap<String, Transaction> =
                IngestionRepository::get_all_transactions(db)?
                    .into_iter()
                    .map(|tx| (tx.id.clone(), tx))
                    .collect();
            let mut merged: HashSet<&str> = HashSet::new();

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
                    if let Some(merge_id) = entry.merge_transaction_id.as_deref() {
                        // Money into this account left the other one as a
                        // spending there, and the other way round.
                        let mirrored_kind = if entry.amount > 0 {
                            "expense"
                        } else {
                            "income"
                        };
                        let mirror = transactions
                            .get(merge_id)
                            .filter(|tx| {
                                tx.account_id == other.id
                                    && tx.transaction_type == mirrored_kind
                                    && tx.amount == cents
                            })
                            .filter(|_| merged.insert(merge_id))
                            .ok_or_else(|| validation_error("import-statement-invalid-transfer"))?;
                        // This side is confirmed by the statement; the other
                        // keeps whatever its own reconciliation said.
                        let (from_reconciled, to_reconciled) = if entry.amount > 0 {
                            (mirror.reconciled, true)
                        } else {
                            (true, mirror.reconciled)
                        };
                        writes.push(Write::Merge {
                            id: mirror.id.clone(),
                            from,
                            to,
                            from_reconciled,
                            to_reconciled,
                        });
                        continue;
                    }
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

            let mut written = Vec::with_capacity(writes.len());
            for write in &writes {
                match write {
                    Write::Movement(transaction) => {
                        IngestionRepository::create_transaction(db, transaction)?;
                        written.push(transaction.id.clone());
                    }
                    Write::Transfer {
                        from,
                        to,
                        cents,
                        date,
                        description,
                    } => {
                        written.push(IngestionRepository::create_transfer(
                            db,
                            from,
                            to,
                            *cents,
                            description,
                            &date.format("%Y-%m-%d").to_string(),
                        )?);
                    }
                    Write::Merge {
                        id,
                        from,
                        to,
                        from_reconciled,
                        to_reconciled,
                    } => db.convert_to_transfer(id, from, to, *from_reconciled, *to_reconciled)?,
                }
            }
            // The bank's own statement is what reconciling checks against, so
            // these rows are confirmed on this account's side. The other side
            // of a transfer waits for that account's statement.
            db.set_reconciled(&account.id, &written)?;
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
    Merge {
        id: String,
        from: String,
        to: String,
        from_reconciled: bool,
        to_reconciled: bool,
    },
}

/// How far apart, in business days, the two sides of a transfer may be
/// booked. Banks clear between each other overnight, but weekends and
/// holidays push it out.
const MIRROR_WINDOW_BUSINESS_DAYS: u32 = 5;

/// Weekdays after the earlier date up to and including the later one.
/// Holidays are not known, so a long weekend counts a day short.
fn business_days_between(a: NaiveDate, b: NaiveDate) -> u32 {
    let (mut day, end) = if a <= b { (a, b) } else { (b, a) };
    let mut count = 0;
    while day < end {
        day = day.succ_opt().unwrap_or(end);
        if !matches!(day.weekday(), Weekday::Sat | Weekday::Sun) {
            count += 1;
        }
    }
    count
}

/// For each row, the movement in another of the user's accounts that looks
/// like its other half: an expense there for money that came in here, or an
/// income there for money that left, with the same amount and close in
/// time. The nearest one wins and each is used once. Rows marked `settled`
/// are skipped.
fn match_mirrors(
    rows: &[StatementRow],
    settled: &[bool],
    transactions: &[Transaction],
    others: &[Account],
) -> Vec<Option<(String, String)>> {
    let eligible: HashSet<&str> = others.iter().map(|a| a.id.as_str()).collect();
    let mut used: HashSet<&str> = HashSet::new();
    rows.iter()
        .zip(settled)
        .map(|(row, settled)| {
            if *settled {
                return None;
            }
            let kind = if row.amount > 0 { "expense" } else { "income" };
            let cents = row.amount.checked_abs()?.checked_mul(100)?;
            let best = transactions
                .iter()
                .filter(|tx| {
                    eligible.contains(tx.account_id.as_str())
                        && tx.transaction_type == kind
                        && tx.amount == cents
                        && !used.contains(tx.id.as_str())
                })
                .filter_map(|tx| {
                    let date = NaiveDate::parse_from_str(tx.date.get(..10)?, "%Y-%m-%d").ok()?;
                    let gap = business_days_between(date, row.date);
                    (gap <= MIRROR_WINDOW_BUSINESS_DAYS).then_some((
                        gap,
                        (date - row.date).num_days().abs(),
                        tx,
                    ))
                })
                .min_by_key(|(gap, days, _)| (*gap, *days))?;
            used.insert(best.2.id.as_str());
            Some((best.2.id.clone(), best.2.account_id.clone()))
        })
        .collect()
}

/// The user's only credit card account, when the statement is not from a
/// card itself. A card payment never shows on the card's own statement
/// (only purchases do), so it cannot be matched and is guessed instead.
fn card_account<'a>(account: &Account, others: &'a [Account]) -> Option<&'a str> {
    if account.account_type == "credit_card" {
        return None;
    }
    let mut cards = others.iter().filter(|a| a.account_type == "credit_card");
    match (cards.next(), cards.next()) {
        (Some(card), None) => Some(card.id.as_str()),
        _ => None,
    }
}

/// "PAGO TARJETA CREDITO", "PAGO TC 4321", "PAGO TARJ VISA".
fn is_card_payment(row: &StatementRow) -> bool {
    if row.amount >= 0 {
        return false;
    }
    let words: Vec<String> = row
        .description
        .split(|c: char| !c.is_alphanumeric())
        .map(str::to_uppercase)
        .collect();
    words.iter().any(|w| w == "PAGO")
        && words
            .iter()
            .any(|w| matches!(w.as_str(), "TARJETA" | "TARJ" | "TC"))
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Suggestion {
    Category(String),
    /// A transfer with this other account.
    Transfer(String),
}

/// What each merchant was last time, per direction. Categories apply to
/// every account (Uber is transport on the card and on the checking
/// account); a transfer only to the two accounts it joined.
struct History {
    categories: HashMap<(String, bool), (String, String)>,
    transfers: HashMap<(String, String, bool), (String, String)>,
}

impl History {
    fn new(transactions: &[Transaction]) -> Self {
        fn keep<K: std::hash::Hash + Eq>(
            map: &mut HashMap<K, (String, String)>,
            key: K,
            date: &str,
            value: &str,
        ) {
            let entry = map
                .entry(key)
                .or_insert_with(|| (String::new(), String::new()));
            if date >= entry.0.as_str() {
                *entry = (date.to_string(), value.to_string());
            }
        }
        let mut categories = HashMap::new();
        let mut transfers = HashMap::new();
        for tx in transactions {
            let key = merchant_key(&tx.description);
            if key.is_empty() {
                continue;
            }
            match tx.transaction_type.as_str() {
                "expense" => keep(&mut categories, (key, true), &tx.date, &tx.category),
                "income" => keep(&mut categories, (key, false), &tx.date, &tx.category),
                "transfer" => {
                    let Some(to) = tx.transfer_account_id.as_deref() else {
                        continue;
                    };
                    keep(
                        &mut transfers,
                        (key.clone(), tx.account_id.clone(), true),
                        &tx.date,
                        to,
                    );
                    keep(
                        &mut transfers,
                        (key, to.to_string(), false),
                        &tx.date,
                        &tx.account_id,
                    );
                }
                _ => {}
            }
        }
        Self {
            categories,
            transfers,
        }
    }

    fn suggest(&self, description: &str, account_id: &str, out: bool) -> Option<Suggestion> {
        let key = merchant_key(description);
        if let Some((_, other)) = self
            .transfers
            .get(&(key.clone(), account_id.to_string(), out))
        {
            return Some(Suggestion::Transfer(other.clone()));
        }
        self.categories
            .get(&(key, out))
            .map(|(_, category)| Suggestion::Category(category.clone()))
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
        let history = History::new(&[
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
        let category = |name: &str| Some(Suggestion::Category(name.into()));
        assert_eq!(
            history.suggest("Compra PAYU *UBER TRIP", "acc", true),
            category("TRANSPORT")
        );
        assert_eq!(
            history.suggest("Compra PAYU *UBER TRIP", "acc", false),
            category("GIFT")
        );
        // Numbers do not make a different merchant, and categories hold on
        // every account.
        assert_eq!(
            history.suggest("compra lider 9876", "card", true),
            category("FOOD")
        );
        assert_eq!(history.suggest("Compra JUMBO", "acc", true), None);
    }

    #[test]
    fn transfers_are_learned_for_both_accounts() {
        let mut payment = tx(
            "2026-02-05",
            "transfer",
            300_000,
            "",
            "PAGO TARJETA CREDITO 4321",
        );
        payment.transfer_account_id = Some("card".into());
        let history = History::new(&[payment]);
        let card = Some(Suggestion::Transfer("card".into()));
        let checking = Some(Suggestion::Transfer("acc".into()));
        assert_eq!(
            history.suggest("PAGO TARJETA CREDITO 9999", "acc", true),
            card
        );
        assert_eq!(
            history.suggest("PAGO TARJETA CREDITO", "card", false),
            checking
        );
        // Another account paying the same description is not that transfer.
        assert_eq!(
            history.suggest("PAGO TARJETA CREDITO", "savings", true),
            None
        );
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn account(id: &str, kind: &str) -> Account {
        Account::new(
            id.into(),
            id.into(),
            kind.into(),
            "CLP".into(),
            0,
            "#8b5cf6".into(),
            None,
            "2026-01-01".into(),
        )
    }

    #[test]
    fn business_days_skip_weekends() {
        // Friday 2026-03-20 to Monday 2026-03-23 is one business day.
        assert_eq!(
            business_days_between(date(2026, 3, 20), date(2026, 3, 23)),
            1
        );
        assert_eq!(
            business_days_between(date(2026, 3, 23), date(2026, 3, 20)),
            1
        );
        assert_eq!(
            business_days_between(date(2026, 3, 16), date(2026, 3, 23)),
            5
        );
        assert_eq!(
            business_days_between(date(2026, 3, 16), date(2026, 3, 16)),
            0
        );
    }

    #[test]
    fn mirrors_pair_opposite_movements_once() {
        let mut out_of_savings = tx(
            "2026-03-20",
            "expense",
            5_000_000,
            "OTHER",
            "Transf. a cuenta",
        );
        out_of_savings.account_id = "savings".into();
        let mut too_late = tx("2026-04-10", "expense", 5_000_000, "OTHER", "Transf.");
        too_late.account_id = "savings".into();
        let mut wrong_side = tx("2026-03-20", "income", 5_000_000, "OTHER", "Transf.");
        wrong_side.account_id = "savings".into();
        let others = [account("savings", "savings")];
        let rows = [
            row((2026, 3, 23), 50_000),
            row((2026, 3, 23), 50_000),
            row((2026, 3, 23), -50_000),
        ];

        let found = match_mirrors(
            &rows,
            &[false, false, false],
            &[out_of_savings.clone(), too_late, wrong_side.clone()],
            &others,
        );
        // Money in here pairs with the expense there, only once; money out
        // pairs with the income.
        assert_eq!(
            found[0],
            Some((out_of_savings.id.clone(), "savings".to_string()))
        );
        assert_eq!(found[1], None);
        assert_eq!(
            found[2],
            Some((wrong_side.id.clone(), "savings".to_string()))
        );

        let skipped = match_mirrors(&rows, &[true, true, true], &[out_of_savings], &others);
        assert!(skipped.iter().all(Option::is_none));
    }

    #[test]
    fn card_payments_go_to_the_only_card() {
        let mut payment = row((2026, 3, 5), -300_000);
        payment.description = "PAGO TARJETA CREDITO".into();
        assert!(is_card_payment(&payment));
        payment.description = "PAGO:GOOGLE PLAY".into();
        assert!(!is_card_payment(&payment));

        let checking = account("acc", "bank");
        let one_card = [
            account("card", "credit_card"),
            account("savings", "savings"),
        ];
        assert_eq!(card_account(&checking, &one_card), Some("card"));
        let two_cards = [
            account("card", "credit_card"),
            account("card2", "credit_card"),
        ];
        assert_eq!(card_account(&checking, &two_cards), None);
        assert_eq!(
            card_account(&account("card", "credit_card"), &one_card[1..]),
            None
        );
    }

    mod with_vault {
        use super::*;
        use std::sync::{Arc, RwLock};

        struct Vault {
            service: IngestionService,
            db: Arc<RwLock<Option<Database>>>,
            dir: std::path::PathBuf,
        }

        impl Drop for Vault {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.dir);
            }
        }

        fn vault() -> Vault {
            let dir =
                std::env::temp_dir().join(format!("sanctum-statement-test-{}", Uuid::new_v4()));
            std::fs::create_dir_all(&dir).unwrap();
            let password = SecretString::from("test-password-123".to_string());
            let db = Database::init(dir.join("vault.db"), &password).unwrap();
            for (id, kind) in [("acc", "bank"), ("savings", "savings")] {
                db.create_account(&account(id, kind)).unwrap();
            }
            let db = Arc::new(RwLock::new(Some(db)));
            Vault {
                service: IngestionService::new(db.clone()),
                db,
                dir,
            }
        }

        fn entry(amount: i64, category: &str) -> StatementEntry {
            StatementEntry {
                date: "2026-03-23".into(),
                description: "Transf. de ahorro".into(),
                amount,
                category: category.into(),
                transfer_account_id: None,
                merge_transaction_id: None,
            }
        }

        #[test]
        fn imported_rows_are_reconciled_and_mirrors_become_one_transfer() {
            let v = vault();
            // The savings statement went in first: the money left as a spending.
            v.service
                .import_statement(
                    "savings",
                    vec![StatementEntry {
                        date: "2026-03-20".into(),
                        ..entry(-50_000, "OTHER")
                    }],
                )
                .unwrap();
            let spending = {
                let lock = v.db.read().unwrap();
                lock.as_ref().unwrap().get_transactions().unwrap().remove(0)
            };
            assert!(spending.reconciled);

            // The checking statement shows it arriving, and the user keeps the
            // suggested transfer with savings.
            let written = v
                .service
                .import_statement(
                    "acc",
                    vec![
                        StatementEntry {
                            transfer_account_id: Some("savings".into()),
                            merge_transaction_id: Some(spending.id.clone()),
                            ..entry(50_000, "")
                        },
                        entry(-3_990, "TRANSPORT"),
                    ],
                )
                .unwrap();
            assert_eq!(written, 2);

            let lock = v.db.read().unwrap();
            let txs = lock.as_ref().unwrap().get_transactions().unwrap();
            assert_eq!(txs.len(), 2, "the transfer replaced the spending: {txs:?}");
            let transfer = txs.iter().find(|t| t.id == spending.id).unwrap();
            assert_eq!(transfer.transaction_type, "transfer");
            assert_eq!(transfer.account_id, "savings");
            assert_eq!(transfer.transfer_account_id.as_deref(), Some("acc"));
            assert!(transfer.reconciled && transfer.transfer_reconciled);
            let uber = txs
                .iter()
                .find(|t| t.transaction_type == "expense")
                .unwrap();
            assert!(uber.reconciled);

            // The same movement cannot be merged twice.
            drop(lock);
            let again = v.service.import_statement(
                "acc",
                vec![StatementEntry {
                    transfer_account_id: Some("savings".into()),
                    merge_transaction_id: Some(spending.id.clone()),
                    ..entry(50_000, "")
                }],
            );
            assert!(again.is_err());
        }
    }
}
