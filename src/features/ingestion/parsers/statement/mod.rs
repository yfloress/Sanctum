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

//! Chilean bank statements (cartolas) and credit card statements.
//!
//! Input is the statement's text as positioned lines (see [`Document`]);
//! turning a PDF into that is the extractor's job. Output is one row per
//! movement in whole pesos, negative for money going out, plus the rows
//! that could not be read and any disagreement with the totals the
//! statement declares. Nothing is dropped silently: a row that fails to
//! parse is reported, and a statement whose rows do not add up to its own
//! summary carries a warning for the preview to show.
//!
//! Row text can hold personal financial data. It goes to the preview and
//! nowhere else; never log it.

mod account;
mod banks;
mod clp;
mod credit;
mod document;
mod pdf;
mod period;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_local;
#[cfg(test)]
mod tests_pdf;

use chrono::NaiveDate;

pub use account::Totals;
pub use document::{Document, Line, Word};
pub use pdf::{PdfError, read_pdf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatementFormat {
    BancoChileAccount,
    BancoChileCredit,
    SantanderAccount,
    SantanderCredit,
    CopecPay,
}

impl StatementFormat {
    pub fn is_credit_card(self) -> bool {
        matches!(self, Self::BancoChileCredit | Self::SantanderCredit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementRow {
    /// Index into [`Document::lines`].
    pub line: usize,
    pub date: NaiveDate,
    /// Without the bank's operation codes.
    pub description: String,
    /// As printed, codes included (a transfer's account number says who
    /// sent it).
    pub raw_description: String,
    /// Whole pesos; negative is money out.
    pub amount: i64,
    /// The sign was guessed from a word like "COMPRA" rather than read from
    /// the statement's layout. Worth a second look in the preview.
    pub sign_from_keyword: bool,
    /// Installment number and count, for purchases in installments.
    pub installment: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowIssue {
    MissingAmount,
    ZeroAmount,
    /// More than one amount could be the movement.
    AmbiguousAmount,
    /// Neither the layout nor the description says charge or credit.
    UnknownSign,
    MissingDescription,
    /// A `dd/mm` row in a statement that declares no period.
    MissingYear,
    OutsidePeriod,
    InvalidDate,
    UnreadableRow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowProblem {
    pub line: usize,
    pub text: String,
    pub issue: RowIssue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementWarning {
    /// Parsed charges and credits differ from the statement's summary.
    TotalsMismatch { declared: Totals, parsed: Totals },
    /// One section of a credit card statement does not add up.
    SectionMismatch {
        section: &'static str,
        declared: i64,
        parsed: i64,
    },
    /// The statement has a summary, but it could not be read.
    TotalsUnreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStatement {
    pub format: StatementFormat,
    pub period: Option<(NaiveDate, NaiveDate)>,
    pub rows: Vec<StatementRow>,
    pub problems: Vec<RowProblem>,
    pub warnings: Vec<StatementWarning>,
}

impl ParsedStatement {
    fn new(format: StatementFormat, period: Option<(NaiveDate, NaiveDate)>) -> Self {
        Self {
            format,
            period,
            rows: Vec::new(),
            problems: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StatementError {
    #[error("the document is not a {0:?} statement")]
    WrongFormat(StatementFormat),
    #[error("the statement period ends before it starts")]
    InvalidPeriod,
    #[error("the statement does not say which period it bills")]
    MissingPeriod,
}

/// How many lines from the top to look for the bank's name. Further down,
/// a transfer to another bank would name that bank too.
const DETECTION_LINES: usize = 25;

/// Guesses the format from the document's heading. Returns `None` when no
/// bank or more than one matches, so the user picks.
pub fn detect_format(doc: &Document) -> Option<StatementFormat> {
    let folded = doc.folded_text();
    if credit::SANTANDER_CREDIT_MARKERS
        .iter()
        .all(|m| folded.contains(m))
    {
        return Some(StatementFormat::SantanderCredit);
    }
    if credit::BANCO_CHILE_CREDIT_MARKERS
        .iter()
        .all(|m| folded.contains(m))
    {
        return Some(StatementFormat::BancoChileCredit);
    }
    let heading = Document {
        lines: doc.lines.iter().take(DETECTION_LINES).cloned().collect(),
    };
    let matches: Vec<StatementFormat> = [banks::BANCO_CHILE, banks::SANTANDER, banks::COPEC_PAY]
        .iter()
        .filter(|profile| account::has_marker(&heading, profile.markers))
        .map(|profile| profile.format)
        .collect();
    match matches.as_slice() {
        [only] => Some(*only),
        _ => None,
    }
}

pub fn parse_statement(
    doc: &Document,
    format: StatementFormat,
) -> Result<ParsedStatement, StatementError> {
    match format {
        StatementFormat::BancoChileAccount => account::parse_account(doc, &banks::BANCO_CHILE),
        StatementFormat::SantanderAccount => account::parse_account(doc, &banks::SANTANDER),
        StatementFormat::CopecPay => account::parse_account(doc, &banks::COPEC_PAY),
        StatementFormat::BancoChileCredit => credit::parse_banco_chile_credit(doc),
        StatementFormat::SantanderCredit => credit::parse_santander_credit(doc),
    }
}
