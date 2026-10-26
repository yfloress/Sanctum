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

//! The period a statement covers, and the year of rows printed as `dd/mm`.

use chrono::{Datelike, NaiveDate};

use super::StatementError;
use super::clp::{RowDate, parse_full_date};

pub type Period = (NaiveDate, NaiveDate);

/// How a statement prints its period. Tokens are folded (see `fold`).
#[derive(Debug, Clone, Copy)]
pub enum PeriodRule {
    /// `DESDE: 01/05/2026 HASTA: 31/05/2026`, also `DESDE 01/05/2026 AL ...`.
    DesdeHasta,
    /// `PERIODO: 01/05/2026 - 31/05/2026` or `PERIODO 01/05/2026 AL ...`.
    PeriodoRange,
    /// A header row followed, somewhere after it, by the two dates.
    AfterHeader(&'static [&'static str]),
}

pub fn find_period(
    tokens: &[String],
    rules: &[PeriodRule],
) -> Result<Option<Period>, StatementError> {
    for rule in rules {
        let found = match rule {
            PeriodRule::DesdeHasta => desde_hasta(tokens),
            PeriodRule::PeriodoRange => periodo_range(tokens),
            PeriodRule::AfterHeader(header) => after_header(tokens, header),
        };
        if let Some((start, end)) = found {
            if end < start {
                return Err(StatementError::InvalidPeriod);
            }
            return Ok(Some((start, end)));
        }
    }
    Ok(None)
}

/// Reads the date after a keyword at `i`, written as `KW date`, `KW : date`,
/// `KW: date` or `KW:date`. Returns the date and the index after it.
fn date_after_keyword(
    tokens: &[String],
    i: usize,
    keywords: &[&str],
) -> Option<(NaiveDate, usize)> {
    let token = tokens.get(i)?;
    for keyword in keywords {
        let Some(rest) = token.strip_prefix(keyword) else {
            continue;
        };
        if let Some(glued) = rest.strip_prefix(':')
            && !glued.is_empty()
        {
            return parse_full_date(glued).map(|d| (d, i + 1));
        }
        if !rest.is_empty() && rest != ":" {
            continue;
        }
        let mut j = i + 1;
        if tokens.get(j).is_some_and(|t| t == ":") {
            j += 1;
        }
        return tokens
            .get(j)
            .and_then(|t| parse_full_date(t))
            .map(|d| (d, j + 1));
    }
    None
}

fn desde_hasta(tokens: &[String]) -> Option<Period> {
    for i in 0..tokens.len() {
        let Some((start, next)) = date_after_keyword(tokens, i, &["DESDE"]) else {
            continue;
        };
        for k in next..tokens.len() {
            if let Some((end, _)) = date_after_keyword(tokens, k, &["HASTA", "AL"]) {
                return Some((start, end));
            }
        }
    }
    None
}

fn periodo_range(tokens: &[String]) -> Option<Period> {
    for i in 0..tokens.len() {
        let Some((start, next)) = date_after_keyword(tokens, i, &["PERIODO"]) else {
            continue;
        };
        let separated = tokens.get(next).is_some_and(|t| t == "-" || t == "AL");
        if separated && let Some(end) = tokens.get(next + 1).and_then(|t| parse_full_date(t)) {
            return Some((start, end));
        }
    }
    None
}

fn after_header(tokens: &[String], header: &[&str]) -> Option<Period> {
    let start = tokens
        .windows(header.len())
        .position(|window| window.iter().zip(header).all(|(t, h)| t == h))?;
    let mut dates = tokens[start + header.len()..]
        .iter()
        .filter_map(|t| parse_full_date(t));
    Some((dates.next()?, dates.next()?))
}

/// Places a `dd/mm` row inside the statement period. A day of slack on each
/// side covers rows booked the day before or after the declared range.
pub fn infer_year(date: RowDate, period: Period) -> Option<NaiveDate> {
    let (start, end) = period;
    let years = [start.year() - 1, start.year(), end.year(), end.year() + 1];
    let candidates: Vec<NaiveDate> = years
        .iter()
        .filter_map(|&y| NaiveDate::from_ymd_opt(y, date.month, date.day))
        .collect();
    if let Some(inside) = candidates.iter().find(|d| (start..=end).contains(*d)) {
        return Some(*inside);
    }
    candidates.into_iter().find(|d| within_period(*d, period))
}

pub fn within_period(date: NaiveDate, (start, end): Period) -> bool {
    let slack = chrono::Duration::days(1);
    date >= start - slack && date <= end + slack
}
