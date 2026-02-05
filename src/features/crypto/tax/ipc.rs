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

//! IPC parsing utilities (offline-only).

use csv::{ReaderBuilder, StringRecord, Trim};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEntry {
    pub period: String, // YYYY-MM
    pub index: f64,
}

#[derive(Debug, Clone)]
pub struct IpcImportSummary {
    pub total_rows: usize,
    pub inserted: usize,
    pub skipped: usize,
    pub errors: usize,
    pub first_period: Option<String>,
    pub last_period: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IpcSummary {
    pub count: usize,
    pub first_period: String,
    pub last_period: String,
    pub updated_at: Option<String>,
}

pub struct IpcParsed {
    pub entries: BTreeMap<String, f64>,
    pub total_rows: usize,
    pub errors: usize,
    pub skipped: usize,
}

pub fn parse_ipc_csv(content: &str) -> Result<IpcParsed, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("IPC file is empty".to_string());
    }

    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(trimmed.as_bytes());

    let headers = reader
        .headers()
        .map_err(|e| format!("Invalid CSV header: {}", e))?
        .clone();

    if is_header_data_row(&headers) {
        return parse_ipc_csv_no_header(trimmed);
    }

    let (period_idx, index_idx) = detect_columns(&headers);
    let period_idx = period_idx.unwrap_or(0);
    let index_idx = index_idx.unwrap_or(1);

    let mut entries: BTreeMap<String, f64> = BTreeMap::new();
    let mut total_rows = 0usize;
    let mut errors = 0usize;
    let mut skipped = 0usize;

    for record in reader.records() {
        let record = match record {
            Ok(record) => record,
            Err(_) => {
                errors += 1;
                continue;
            }
        };

        if record.iter().all(|field| field.trim().is_empty()) {
            continue;
        }
        total_rows += 1;

        let period_raw = record.get(period_idx).unwrap_or("");
        let index_raw = record.get(index_idx).unwrap_or("");

        let Some(period) = parse_period(period_raw) else {
            errors += 1;
            continue;
        };
        let Some(index) = parse_index(index_raw) else {
            errors += 1;
            continue;
        };

        let existed = entries.insert(period, index).is_some();
        if existed {
            skipped += 1;
        }
    }

    Ok(IpcParsed {
        entries,
        total_rows,
        errors,
        skipped,
    })
}

fn parse_ipc_csv_no_header(content: &str) -> Result<IpcParsed, String> {
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .has_headers(false)
        .from_reader(content.as_bytes());

    let mut entries: BTreeMap<String, f64> = BTreeMap::new();
    let mut total_rows = 0usize;
    let mut errors = 0usize;
    let mut skipped = 0usize;

    for record in reader.records() {
        let record = match record {
            Ok(record) => record,
            Err(_) => {
                errors += 1;
                continue;
            }
        };

        if record.iter().all(|field| field.trim().is_empty()) {
            continue;
        }
        total_rows += 1;

        let period_raw = record.get(0).unwrap_or("");
        let index_raw = record.get(1).unwrap_or("");

        let Some(period) = parse_period(period_raw) else {
            errors += 1;
            continue;
        };
        let Some(index) = parse_index(index_raw) else {
            errors += 1;
            continue;
        };

        let existed = entries.insert(period, index).is_some();
        if existed {
            skipped += 1;
        }
    }

    Ok(IpcParsed {
        entries,
        total_rows,
        errors,
        skipped,
    })
}

pub fn summarize_ipc(entries: &[IpcEntry], updated_at: Option<String>) -> Option<IpcSummary> {
    if entries.is_empty() {
        return None;
    }

    let first_period = entries.first()?.period.clone();
    let last_period = entries.last()?.period.clone();

    Some(IpcSummary {
        count: entries.len(),
        first_period,
        last_period,
        updated_at,
    })
}

pub fn build_import_summary(parsed: &IpcParsed) -> IpcImportSummary {
    let periods: Vec<&String> = parsed.entries.keys().collect();
    let first_period = periods.first().map(|p| (*p).clone());
    let last_period = periods.last().map(|p| (*p).clone());

    IpcImportSummary {
        total_rows: parsed.total_rows,
        inserted: parsed.entries.len(),
        skipped: parsed.skipped,
        errors: parsed.errors,
        first_period,
        last_period,
    }
}

pub fn map_to_entries(map: BTreeMap<String, f64>) -> Vec<IpcEntry> {
    map.into_iter()
        .map(|(period, index)| IpcEntry { period, index })
        .collect()
}

pub fn normalize_header(input: &str) -> String {
    let lowered = input.trim().to_lowercase();
    let mut normalized = String::with_capacity(lowered.len());
    for ch in lowered.chars() {
        let mapped = match ch {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            _ => ch,
        };
        if mapped.is_ascii_alphanumeric() {
            normalized.push(mapped);
        }
    }
    normalized
}

fn detect_columns(headers: &StringRecord) -> (Option<usize>, Option<usize>) {
    let mut period_idx = None;
    let mut index_idx = None;

    for (idx, raw) in headers.iter().enumerate() {
        let key = normalize_header(raw);
        if period_idx.is_none() && is_period_header(&key) {
            period_idx = Some(idx);
            continue;
        }
        if index_idx.is_none() && is_index_header(&key) {
            index_idx = Some(idx);
        }
    }

    (period_idx, index_idx)
}

fn is_header_data_row(headers: &StringRecord) -> bool {
    if headers.len() < 2 {
        return false;
    }
    let left = headers.get(0).unwrap_or("");
    let right = headers.get(1).unwrap_or("");
    parse_period(left).is_some() && parse_index(right).is_some()
}

fn is_period_header(key: &str) -> bool {
    matches!(
        key,
        "periodo"
            | "period"
            | "fecha"
            | "mes"
            | "month"
            | "time"
            | "date"
            | "anio"
            | "ano"
    )
}

fn is_index_header(key: &str) -> bool {
    matches!(
        key,
        "ipc" | "indice" | "indiceipc" | "indiceprecios" | "index" | "valor"
    )
}

fn parse_period(raw: &str) -> Option<String> {
    let cleaned = raw.trim();
    if cleaned.is_empty() {
        return None;
    }

    let normalized = normalize_header(cleaned);

    if let Some(period) = parse_numeric_period(cleaned) {
        return Some(period);
    }

    if let Some(period) = parse_month_name_period(&normalized) {
        return Some(period);
    }

    None
}

fn parse_numeric_period(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let mut sep = '-';
    if trimmed.contains('/') {
        sep = '/';
    }

    if trimmed.contains(sep) {
        let parts: Vec<&str> = trimmed.split(sep).collect();
        if parts.len() >= 2 {
            let left = parts[0].trim();
            let right = parts[1].trim();
            if left.len() == 4 {
                let year = left.parse::<i32>().ok()?;
                let month = right.parse::<u32>().ok()?;
                return format_period(year, month);
            }
            if right.len() == 4 {
                let year = right.parse::<i32>().ok()?;
                let month = left.parse::<u32>().ok()?;
                return format_period(year, month);
            }
        }
    }

    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 6 {
        let year = digits[0..4].parse::<i32>().ok()?;
        let month = digits[4..6].parse::<u32>().ok()?;
        return format_period(year, month);
    }
    if digits.len() == 8 {
        let year = digits[0..4].parse::<i32>().ok()?;
        let month = digits[4..6].parse::<u32>().ok()?;
        return format_period(year, month);
    }

    None
}

fn parse_month_name_period(normalized: &str) -> Option<String> {
    let year = extract_year(normalized)?;
    let month = extract_month(normalized)?;
    format_period(year, month)
}

fn extract_year(input: &str) -> Option<i32> {
    let digits: Vec<char> = input.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 4 {
        return None;
    }
    for idx in 0..=digits.len().saturating_sub(4) {
        let slice: String = digits[idx..idx + 4].iter().collect();
        if let Ok(year) = slice.parse::<i32>() {
            if (1900..=2100).contains(&year) {
                return Some(year);
            }
        }
    }
    None
}

fn extract_month(input: &str) -> Option<u32> {
    let month_map = [
        ("enero", 1u32),
        ("febrero", 2u32),
        ("marzo", 3u32),
        ("abril", 4u32),
        ("mayo", 5u32),
        ("junio", 6u32),
        ("julio", 7u32),
        ("agosto", 8u32),
        ("septiembre", 9u32),
        ("setiembre", 9u32),
        ("octubre", 10u32),
        ("noviembre", 11u32),
        ("diciembre", 12u32),
        ("ene", 1u32),
        ("feb", 2u32),
        ("mar", 3u32),
        ("abr", 4u32),
        ("may", 5u32),
        ("jun", 6u32),
        ("jul", 7u32),
        ("ago", 8u32),
        ("sep", 9u32),
        ("oct", 10u32),
        ("nov", 11u32),
        ("dic", 12u32),
    ];

    for (name, value) in month_map {
        if input.contains(name) {
            return Some(value);
        }
    }

    None
}

fn format_period(year: i32, month: u32) -> Option<String> {
    if !(1900..=2100).contains(&year) {
        return None;
    }
    if !(1..=12).contains(&month) {
        return None;
    }
    Some(format!("{:04}-{:02}", year, month))
}

fn parse_index(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut value = trimmed.to_string();

    if value.contains(',') && value.contains('.') {
        value = value.replace('.', "");
        value = value.replace(',', ".");
    } else if value.contains(',') {
        value = value.replace(',', ".");
    }

    let filtered: String = value
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();

    if filtered.is_empty() {
        return None;
    }

    filtered.parse::<f64>().ok()
}
