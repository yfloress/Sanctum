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

//! Tax period helpers.

use super::types::TaxPeriod;
use chrono::{Datelike, NaiveDate};

pub(super) fn parse_period(period_id: &str) -> Result<TaxPeriod, String> {
    let trimmed = period_id.trim();
    if trimmed.len() != 4 {
        return Err("Tax period must be a 4-digit year".to_string());
    }
    let year: i32 = trimmed
        .parse()
        .map_err(|_| "Tax period must be a valid year".to_string())?;

    let start = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| "Invalid tax period start".to_string())?;
    let end = NaiveDate::from_ymd_opt(year, 12, 31)
        .ok_or_else(|| "Invalid tax period end".to_string())?;

    Ok(TaxPeriod {
        id: trimmed.to_string(),
        start,
        end,
    })
}

pub(super) fn parse_date(raw: &str) -> Option<NaiveDate> {
    if raw.len() >= 10 {
        if let Ok(date) = NaiveDate::parse_from_str(&raw[..10], "%Y-%m-%d") {
            return Some(date);
        }
    }
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok()
}

pub(super) fn is_in_period(period: &TaxPeriod, date: NaiveDate) -> bool {
    date >= period.start && date <= period.end
}

pub(super) fn prev_month_key(date: NaiveDate) -> String {
    let mut year = date.year();
    let mut month = date.month();
    if month == 1 {
        year -= 1;
        month = 12;
    } else {
        month -= 1;
    }
    format!("{:04}-{:02}", year, month)
}
