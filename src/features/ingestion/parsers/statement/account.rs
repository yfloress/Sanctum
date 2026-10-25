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

//! Checking, savings and wallet statements: a table of dated rows with
//! charge, credit and balance columns.
//!
//! The sign of each movement comes from the strongest evidence available,
//! in this order: the column the amount sits in (needs layout), an explicit
//! `+`/`-`, a zero placeholder in the other amount column, and last a
//! keyword in the description. Keyword guesses are flagged on the row so
//! the preview can ask for a look.

use std::collections::HashMap;

use super::clp::{Amount, RowDate, is_time, parse_amount, parse_row_date};
use super::document::{Document, Line, Word, fold};
use super::period::{Period, PeriodRule, find_period, infer_year, within_period};
use super::{
    ParsedStatement, RowIssue, RowProblem, StatementError, StatementFormat, StatementRow,
    StatementWarning,
};

/// Column titles, folded. Several titles per column average their centers,
/// for headers printed over two rows ("CHEQUES Y" above "CARGOS").
#[derive(Debug, Clone, Copy)]
pub struct ColumnHeaders {
    pub expense: &'static [&'static str],
    pub income: &'static [&'static str],
    pub balance: &'static [&'static str],
    pub description: &'static [&'static str],
}

/// Which of the two amount columns comes first, when the empty one holds `0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnOrder {
    ExpenseFirst,
    IncomeFirst,
}

/// Charges and credits declared in a statement's summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Totals {
    pub expense: i64,
    pub income: i64,
}

/// Reads the declared totals from the statement's folded tokens.
pub type SummaryReader = fn(&[String]) -> Option<Totals>;
/// Drops bank codes from the description words.
pub type DescriptionCleaner = fn(&[String]) -> Vec<String>;

pub struct AccountProfile {
    pub format: StatementFormat,
    /// Any of these, folded and without punctuation, identifies the bank.
    pub markers: &'static [&'static str],
    pub periods: &'static [PeriodRule],
    pub income_keywords: &'static [&'static str],
    pub expense_keywords: &'static [&'static str],
    /// Rows without a date continue the date of the row above.
    pub continuation_lines: bool,
    pub start_markers: &'static [&'static str],
    pub end_markers: &'static [&'static str],
    pub columns: Option<ColumnHeaders>,
    /// Rows print `0` in the empty one of the charge and credit columns.
    pub zero_filled: Option<ColumnOrder>,
    pub summary: Option<SummaryReader>,
    pub clean_description: Option<DescriptionCleaner>,
}

const SKIPPED_ROWS: &[&str] = &[
    "SALDO INICIAL",
    "SALDO FINAL",
    "TOTAL CARGOS",
    "TOTAL ABONOS",
];
const AMOUNT_LABELS: &[&str] = &[
    "SALDO", "CARGO", "CARGOS", "ABONO", "ABONOS", "DEBE", "HABER", "$", "CLP",
];
const HEADER_ROW_WORDS: &[&str] = &["DESCRIPCION", "DETALLE", "OPERACION", "COMERCIO"];

pub fn has_marker(doc: &Document, markers: &[&str]) -> bool {
    let compact: String = doc
        .folded_text()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();
    markers.iter().any(|marker| {
        let wanted: String = fold(marker)
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .collect();
        // Some PDFs draw bold headings twice, so the extracted text reads
        // "BBAANNCCOO". Accept that too.
        let doubled: String = wanted.chars().flat_map(|c| [c, c]).collect();
        compact.contains(&wanted) || compact.contains(&doubled)
    })
}

pub fn parse_account(
    doc: &Document,
    profile: &AccountProfile,
) -> Result<ParsedStatement, StatementError> {
    if !has_marker(doc, profile.markers) {
        return Err(StatementError::WrongFormat(profile.format));
    }
    let tokens = doc.folded_tokens();
    let period = find_period(&tokens, profile.periods)?;
    let ranges = profile
        .columns
        .map(|headers| column_ranges(doc, &headers))
        .unwrap_or_default();

    let mut statement = ParsedStatement::new(profile.format, period);
    for row in table_rows(doc, profile) {
        let line = &doc.lines[row.line];
        match parse_row(&row, line, profile, period, ranges.get(&line.page)) {
            Ok(Some(parsed)) => statement.rows.push(parsed),
            Ok(None) => {}
            Err(issue) => statement.problems.push(RowProblem {
                line: row.line,
                text: line.text(),
                issue,
            }),
        }
    }
    if statement.period.is_none() {
        statement.period = period_from_rows(&statement.rows);
    }
    if let Some(summary) = profile.summary {
        check_totals(&mut statement, summary(&tokens), doc);
    }
    Ok(statement)
}

struct TableRow<'a> {
    line: usize,
    date: RowDate,
    rest: &'a [Word],
}

/// Rows inside the movements table. When the statement has a start marker
/// the table is off until it appears, and every end marker turns it off
/// again, so page footers and summaries are not read as movements.
fn table_rows<'a>(doc: &'a Document, profile: &AccountProfile) -> Vec<TableRow<'a>> {
    let text = doc.folded_text();
    let mut active = !profile.start_markers.iter().any(|m| text.contains(m));
    let mut current_date: Option<RowDate> = None;
    let mut rows = Vec::new();

    for (index, line) in doc.lines.iter().enumerate() {
        let folded = fold(&line.text());
        if profile.end_markers.iter().any(|m| folded.contains(m)) {
            active = false;
            current_date = None;
            continue;
        }
        if profile.start_markers.iter().any(|m| folded.contains(m)) {
            active = true;
            current_date = None;
            continue;
        }
        let Some(first) = line.words.first() else {
            continue;
        };
        if let Some(date) = parse_row_date(&first.text) {
            current_date = Some(date);
            let mut start = 1;
            if line.words.get(1).is_some_and(|w| is_time(&w.text)) {
                start = 2;
            }
            if active && line.words.len() > start {
                rows.push(TableRow {
                    line: index,
                    date,
                    rest: &line.words[start..],
                });
            }
            continue;
        }
        if profile.continuation_lines
            && active
            && !folded.contains("SALDO DIA")
            && line.words.iter().any(|w| parse_amount(&w.text).is_some())
            && let Some(date) = current_date
        {
            rows.push(TableRow {
                line: index,
                date,
                rest: &line.words,
            });
        }
    }
    rows
}

fn parse_row(
    row: &TableRow<'_>,
    line: &Line,
    profile: &AccountProfile,
    period: Option<Period>,
    ranges: Option<&ColumnRanges>,
) -> Result<Option<StatementRow>, RowIssue> {
    let folded: Vec<String> = row.rest.iter().map(|w| fold(&w.text)).collect();
    if SKIPPED_ROWS
        .iter()
        .any(|skip| folded.join(" ").starts_with(skip))
    {
        return Ok(None);
    }

    let positioned = ranges.filter(|_| line.is_positioned());
    let (description_end, amount, sign_from_keyword) =
        match positioned.and_then(|r| amount_by_column(row.rest, r).transpose()) {
            Some(found) => {
                let (end, amount) = found?;
                (end, amount, false)
            }
            None => amount_by_text(row.rest, &folded, profile)?,
        };

    let raw: Vec<String> = row.rest[..description_end]
        .iter()
        .map(|w| w.text.clone())
        .collect();
    let cleaned = match profile.clean_description {
        Some(clean) => clean(&raw),
        None => raw.clone(),
    };
    let description = trim_description(&cleaned.join(" "));
    let raw_description = trim_description(&raw.join(" "));
    let description = if description.is_empty() {
        raw_description.clone()
    } else {
        description
    };
    if description.is_empty() {
        return Err(RowIssue::MissingDescription);
    }

    let date = match (row.date.year, period) {
        (Some(year), _) => chrono::NaiveDate::from_ymd_opt(year, row.date.month, row.date.day)
            .ok_or(RowIssue::InvalidDate)?,
        (None, Some(period)) => infer_year(row.date, period).ok_or(RowIssue::OutsidePeriod)?,
        (None, None) => return Err(RowIssue::MissingYear),
    };
    if let Some(period) = period
        && !within_period(date, period)
    {
        return Err(RowIssue::OutsidePeriod);
    }

    Ok(Some(StatementRow {
        line: row.line,
        date,
        description,
        raw_description,
        amount,
        sign_from_keyword,
        installment: None,
    }))
}

fn trim_description(text: &str) -> String {
    text.trim_matches(|c: char| c.is_whitespace() || c == ':' || c == '-')
        .to_string()
}

/// Reads the amount from its column. Returns the index where the
/// description ends and the signed amount, or `None` when no amount sits in
/// a charge or credit column (the caller then falls back to the text).
fn amount_by_column(
    words: &[Word],
    ranges: &ColumnRanges,
) -> Result<Option<(usize, i64)>, RowIssue> {
    let mut first = None;
    let mut signed = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let Some(amount) = parse_amount(&word.text) else {
            continue;
        };
        let column = word.center().and_then(|x| ranges.classify(x));
        let sign = match column {
            Some(Column::Expense) => -1,
            Some(Column::Income) => 1,
            _ => continue,
        };
        first.get_or_insert(index);
        if amount.value != 0 {
            signed.push(sign * amount.value);
        }
    }
    let Some(first) = first else {
        return Ok(None);
    };
    signed.sort_unstable();
    signed.dedup();
    match signed.as_slice() {
        [] => Err(RowIssue::ZeroAmount),
        [amount] => Ok(Some((first, *amount))),
        _ => Err(RowIssue::AmbiguousAmount),
    }
}

/// Reads the amounts at the end of the row: up to three (charge, credit,
/// balance), possibly separated by labels like "SALDO". Anything after the
/// last amount, such as Banco de Chile's trailing "D", is ignored.
fn amount_by_text(
    words: &[Word],
    folded: &[String],
    profile: &AccountProfile,
) -> Result<(usize, i64, bool), RowIssue> {
    let (last, last_amount) = words
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, w)| parse_amount(&w.text).map(|a| (i, a)))
        .ok_or(RowIssue::MissingAmount)?;
    let mut block: Vec<(usize, Amount)> = vec![(last, last_amount)];
    let mut index = last;
    while index > 0 && block.len() < 3 {
        index -= 1;
        // "ID 1111111111111 CLP 7,00": the currency opens the amounts, so
        // the number before it is a reference, not a charge.
        if folded[index] == "CLP" {
            break;
        }
        if let Some(amount) = parse_amount(&words[index].text) {
            block.insert(0, (index, amount));
        } else if !AMOUNT_LABELS.contains(&folded[index].as_str()) {
            break;
        }
    }
    // "COMPRA LIDER CARGO 12.345": the label belongs to the amount.
    let mut description_end = block[0].0;
    while description_end > 0 && AMOUNT_LABELS.contains(&folded[description_end - 1].as_str()) {
        description_end -= 1;
    }
    let amounts: Vec<Amount> = block.iter().map(|(_, a)| *a).collect();

    // A zero in one of two columns is a placeholder: the other one holds
    // the movement. With three amounts the last is the running balance.
    let movement: &[Amount] = match amounts.len() {
        1 | 2 if is_zero_filled_pair(&amounts) => &amounts,
        1 => &amounts,
        n => &amounts[..n - 1],
    };

    if let Some(amount) = movement
        .iter()
        .find(|a| a.negative.is_some() && a.value != 0)
    {
        return Ok((description_end, amount.signed(), false));
    }
    if let Some(order) = profile.zero_filled
        && is_zero_filled_pair(movement)
    {
        let value = movement[0].value + movement[1].value;
        let first_is_movement = movement[0].value != 0;
        let is_expense = first_is_movement == (order == ColumnOrder::ExpenseFirst);
        return Ok((
            description_end,
            if is_expense { -value } else { value },
            false,
        ));
    }

    let non_zero: Vec<i64> = movement
        .iter()
        .filter(|a| a.value != 0)
        .map(|a| a.value)
        .collect();
    let value = match non_zero.as_slice() {
        [] => return Err(RowIssue::ZeroAmount),
        [value] => *value,
        _ => return Err(RowIssue::AmbiguousAmount),
    };
    let description = folded[..description_end].join(" ");
    if profile
        .income_keywords
        .iter()
        .any(|k| has_keyword(&description, k))
    {
        return Ok((description_end, value, true));
    }
    if profile
        .expense_keywords
        .iter()
        .any(|k| has_keyword(&description, k))
    {
        return Ok((description_end, -value, true));
    }
    Err(RowIssue::UnknownSign)
}

/// Whole-word match, so "PAC" (automatic debit) does not fire on
/// "PACIFICO". A plural ending is accepted: "ABONO" matches "ABONOS".
/// Keywords ending in punctuation ("PAGO:") may run into the next word.
fn has_keyword(text: &str, keyword: &str) -> bool {
    let open_end = keyword.ends_with(|c: char| !c.is_alphanumeric());
    text.match_indices(keyword).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        if before.is_some_and(char::is_alphanumeric) {
            return false;
        }
        if open_end {
            return true;
        }
        let after = &text[at + keyword.len()..];
        let after = after
            .strip_prefix("ES")
            .or_else(|| after.strip_prefix('S'))
            .filter(|rest| !rest.starts_with(char::is_alphanumeric))
            .unwrap_or(after);
        !after.starts_with(char::is_alphanumeric)
    })
}

fn is_zero_filled_pair(amounts: &[Amount]) -> bool {
    amounts.len() == 2 && ((amounts[0].value == 0) != (amounts[1].value == 0))
}

fn period_from_rows(rows: &[StatementRow]) -> Option<Period> {
    let start = rows.iter().map(|r| r.date).min()?;
    let end = rows.iter().map(|r| r.date).max()?;
    Some((start, end))
}

fn check_totals(statement: &mut ParsedStatement, declared: Option<Totals>, doc: &Document) {
    let Some(declared) = declared else {
        let text = doc.folded_text();
        if text.contains("SALDO INICIAL") && text.contains("SALDO FINAL") {
            statement.warnings.push(StatementWarning::TotalsUnreadable);
        }
        return;
    };
    let parsed = Totals {
        expense: statement
            .rows
            .iter()
            .filter(|r| r.amount < 0)
            .map(|r| -r.amount)
            .sum(),
        income: statement
            .rows
            .iter()
            .filter(|r| r.amount > 0)
            .map(|r| r.amount)
            .sum(),
    };
    if parsed != declared {
        statement
            .warnings
            .push(StatementWarning::TotalsMismatch { declared, parsed });
    }
}

// ─── Column layout ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Column {
    Expense,
    Income,
    Balance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ColumnRanges {
    expense: (f32, f32),
    income: (f32, f32),
    balance: (f32, f32),
}

impl ColumnRanges {
    fn classify(&self, x: f32) -> Option<Column> {
        let inside = |(left, right): (f32, f32)| left <= x && x < right;
        if inside(self.expense) {
            Some(Column::Expense)
        } else if inside(self.income) {
            Some(Column::Income)
        } else if inside(self.balance) {
            Some(Column::Balance)
        } else {
            None
        }
    }
}

/// Column ranges per page, from the movements header of each page.
fn column_ranges(doc: &Document, headers: &ColumnHeaders) -> HashMap<usize, ColumnRanges> {
    let mut by_page: HashMap<usize, Vec<&Line>> = HashMap::new();
    for line in doc.lines.iter().filter(|l| l.is_positioned()) {
        by_page.entry(line.page).or_default().push(line);
    }
    by_page
        .into_iter()
        .filter_map(|(page, lines)| page_ranges(&lines, headers).map(|r| (page, r)))
        .collect()
}

fn page_ranges(lines: &[&Line], headers: &ColumnHeaders) -> Option<ColumnRanges> {
    // The header row is the one naming the description column. Only rows
    // just below it count, so the summary box above the table (which also
    // says "cargos" and "saldo") cannot move the columns.
    let header = lines.iter().find(|line| {
        let folded = fold(&line.text());
        HEADER_ROW_WORDS.iter().any(|w| folded.contains(w))
    })?;
    let window: Vec<&Word> = lines
        .iter()
        .filter(|l| l.top >= header.top - 2.0 && l.top <= header.top + 16.0)
        .flat_map(|l| l.words.iter())
        .collect();
    let center = |titles: &[&str]| -> Option<f32> {
        let centers: Vec<f32> = window
            .iter()
            .filter(|w| titles.contains(&fold(&w.text).as_str()))
            .filter_map(|w| w.center())
            .collect();
        (!centers.is_empty()).then(|| centers.iter().sum::<f32>() / centers.len() as f32)
    };

    let mut columns = [
        (Column::Expense, center(headers.expense)?),
        (Column::Income, center(headers.income)?),
        (Column::Balance, center(headers.balance)?),
    ];
    columns.sort_by(|a, b| a.1.total_cmp(&b.1));
    let description = center(headers.description);

    let mut ranges = [(f32::NEG_INFINITY, f32::INFINITY); 3];
    for i in 0..3 {
        let here = columns[i].1;
        let mut left = if i == 0 {
            here - (columns[1].1 - here) / 2.0
        } else {
            (columns[i - 1].1 + here) / 2.0
        };
        if let Some(description) = description
            && description < here
        {
            left = left.max((description + here) / 2.0);
        }
        let right = if i == 2 {
            f32::INFINITY
        } else {
            (here + columns[i + 1].1) / 2.0
        };
        ranges[i] = (left, right);
    }
    let range_of = |column: Column| {
        let i = columns.iter().position(|c| c.0 == column).unwrap_or(0);
        ranges[i]
    };
    Some(ColumnRanges {
        expense: range_of(Column::Expense),
        income: range_of(Column::Income),
        balance: range_of(Column::Balance),
    })
}

/// Finds `sequence` in folded tokens and returns the next `count` amounts,
/// skipping `$` signs. Used by summary readers.
pub fn amounts_after(tokens: &[String], sequence: &[&str], count: usize) -> Option<Vec<i64>> {
    let start = tokens
        .windows(sequence.len())
        .position(|w| w.iter().zip(sequence).all(|(t, s)| t == s))?;
    let amounts: Vec<i64> = tokens[start + sequence.len()..]
        .iter()
        .filter(|t| *t != "$")
        .take(count)
        .map(|t| parse_amount(t).map(|a| a.value))
        .collect::<Option<_>>()?;
    (amounts.len() == count).then_some(amounts)
}
