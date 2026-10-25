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

//! Chilean peso amounts and statement dates, one token at a time.

use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount {
    /// Absolute value in whole pesos.
    pub value: i64,
    /// `Some` when the token carried an explicit `+` or `-`.
    pub negative: Option<bool>,
}

impl Amount {
    pub fn signed(self) -> i64 {
        if self.negative == Some(true) {
            -self.value
        } else {
            self.value
        }
    }
}

/// Parses `12.345`, `-$12.345`, `$+1.000`, `CLP 5.490,00` style tokens.
///
/// Dots are thousands separators. A decimal part is only accepted when it
/// is zero, because a peso amount with cents means the token is something
/// else (or the statement is not in CLP).
pub fn parse_amount(token: &str) -> Option<Amount> {
    let mut rest = token.trim();
    if let Some(stripped) = rest
        .strip_prefix("CLP")
        .or_else(|| rest.strip_prefix("clp"))
    {
        rest = stripped.trim_start();
    }
    let mut negative = None;
    for _ in 0..2 {
        if let Some(stripped) = rest.strip_prefix('$') {
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix('-') {
            negative = Some(true);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix('+') {
            negative = Some(false);
            rest = stripped;
        }
    }

    let (integer, decimals) = match rest.split_once(',') {
        Some((integer, decimals)) => (integer, Some(decimals)),
        None => (rest, None),
    };
    if let Some(decimals) = decimals
        && (decimals.is_empty() || decimals.len() > 2 || !decimals.bytes().all(|b| b == b'0'))
    {
        return None;
    }
    if !is_grouped_integer(integer) {
        return None;
    }
    let digits: String = integer.chars().filter(char::is_ascii_digit).collect();
    if digits.len() > 15 {
        return None;
    }
    Some(Amount {
        value: digits.parse().ok()?,
        negative,
    })
}

/// `123`, `1.234`, `12.345.678`; not `1.23` or `.123`.
fn is_grouped_integer(text: &str) -> bool {
    let mut groups = text.split('.');
    let Some(first) = groups.next() else {
        return false;
    };
    if first.is_empty() || !first.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let mut grouped = false;
    for group in groups {
        if group.len() != 3 || !group.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        grouped = true;
    }
    !grouped || first.len() <= 3
}

/// A movement date as printed: the year is often left out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowDate {
    pub day: u32,
    pub month: u32,
    pub year: Option<i32>,
}

/// `dd/mm`, `dd-mm`, `dd/mm/yyyy` or `dd-mm-yyyy`.
pub fn parse_row_date(token: &str) -> Option<RowDate> {
    let parts: Vec<&str> = token.split(['/', '-']).collect();
    let (day, month, year) = match parts.as_slice() {
        [d, m] => (*d, *m, None),
        [d, m, y] if y.len() == 4 => (*d, *m, Some(*y)),
        _ => return None,
    };
    if day.len() != 2 || month.len() != 2 {
        return None;
    }
    let day: u32 = digits(day)?;
    let month: u32 = digits(month)?;
    if !(1..=31).contains(&day) || !(1..=12).contains(&month) {
        return None;
    }
    let year = match year {
        Some(y) => Some(digits::<i32>(y)?),
        None => None,
    };
    Some(RowDate { day, month, year })
}

pub fn parse_full_date(token: &str) -> Option<NaiveDate> {
    let date = parse_row_date(token)?;
    NaiveDate::from_ymd_opt(date.year?, date.month, date.day)
}

/// `dd/mm/yy`, used by credit card statements.
pub fn parse_short_year_date(token: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = token.split('/').collect();
    let [d, m, y] = parts.as_slice() else {
        return None;
    };
    if d.len() != 2 || m.len() != 2 || y.len() != 2 {
        return None;
    }
    NaiveDate::from_ymd_opt(2000 + digits::<i32>(y)?, digits(m)?, digits(d)?)
}

/// `hh:mm` or `hh:mm:ss`.
pub fn is_time(token: &str) -> bool {
    let parts: Vec<&str> = token.split(':').collect();
    (2..=3).contains(&parts.len())
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_digit()))
}

/// `nn/nn`, an installment number over the installment count.
pub fn parse_installment(token: &str) -> Option<(u32, u32)> {
    let (number, count) = token.split_once('/')?;
    if number.len() != 2 || count.len() != 2 {
        return None;
    }
    Some((digits(number)?, digits(count)?))
}

fn digits<T: std::str::FromStr>(text: &str) -> Option<T> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

pub fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}
