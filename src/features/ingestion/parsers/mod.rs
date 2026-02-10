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

//! Parsers for different import formats
//!
//! Supports JSON, CSV, and plain text formats.

pub mod csv;
pub mod json;
pub mod text;

pub use self::csv::CsvParser;
pub use self::json::{JsonParseResult, JsonParser};
pub use self::text::{TextMixedParseResult, TextParser};

use super::types::{
    ImportFormat, ImportHabitLog, ImportTransaction, RowError,
};

#[derive(Debug)]
pub struct ParseResult<T> {
    pub items: Vec<(usize, T)>,
    pub errors: Vec<RowError>,
}

impl<T> Default for ParseResult<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Common parser trait for all formats
pub trait ImportParser {
    /// Parses transactions from raw content
    /// Returns parsed items plus row-level parse errors
    fn parse_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportTransaction>, RowError>;

    /// Parses habit logs from raw content
    /// Returns parsed items plus row-level parse errors
    fn parse_habit_logs(&self, content: &str)
        -> Result<ParseResult<ImportHabitLog>, RowError>;

    /// Returns the format name for reporting
    fn format_name(&self) -> &'static str;
}

/// Detects file format from content and filename
pub fn detect_format(content: &str, filename: &str) -> Option<ImportFormat> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Get file extension
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    // JSON detection
    if (ext == "json" || trimmed.starts_with('{'))
        && trimmed.starts_with('{')
        && trimmed.ends_with('}')
    {
        return Some(ImportFormat::Json);
    }

    // CSV detection (has comma-separated header on first line)
    if ext == "csv" {
        let first_line = trimmed.lines().next()?.to_lowercase();
        // Must have commas and not start with # (comment)
        if first_line.contains(',') && !first_line.starts_with('#') {
            // Crypto CSV: wallet, symbol columns
            if first_line.contains("wallet") && first_line.contains("symbol") {
                return Some(ImportFormat::CsvCrypto);
            }
            // Habit CSV
            if first_line.contains("habit")
                && first_line.contains("date")
                && first_line.contains("completed")
            {
                return Some(ImportFormat::CsvHabitLogs);
            }
            // Transaction CSV
            if first_line.contains("account") && first_line.contains("amount") {
                return Some(ImportFormat::CsvTransactions);
            }
        }
    }

    // Text detection (semicolon-separated with prefixes T;, H;, C;)
    if ext == "txt" || trimmed.contains(';') {
        // Check for prefix-based mixed format (T;, H;, C;)
        let has_prefixes = trimmed.lines().any(|line| {
            let t = line.trim().to_uppercase();
            t.starts_with("T;") || t.starts_with("H;") || t.starts_with("C;")
        });

        if has_prefixes {
            return Some(ImportFormat::TextMixed);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_json_format() {
        let json = r#"{"version": "1.0", "transactions": []}"#;
        assert_eq!(detect_format(json, "data.json"), Some(ImportFormat::Json));
    }

    #[test]
    fn test_detect_csv_transactions() {
        let csv = "date,account,type,amount,currency,category,description,transfer_to_account\n";
        assert_eq!(
            detect_format(csv, "data.csv"),
            Some(ImportFormat::CsvTransactions)
        );
    }

    #[test]
    fn test_detect_csv_habits() {
        let csv = "habit,date,completed\n";
        assert_eq!(
            detect_format(csv, "habits.csv"),
            Some(ImportFormat::CsvHabitLogs)
        );
    }

    #[test]
    fn test_detect_csv_crypto() {
        let csv = "date,wallet,symbol,type,amount,price_per_coin,fee,notes\n";
        assert_eq!(
            detect_format(csv, "crypto.csv"),
            Some(ImportFormat::CsvCrypto)
        );
    }

    #[test]
    fn test_detect_text_mixed() {
        let text = "T;2024-01-15;Account;expense;100;USD;Food;Groceries;\nH;Meditate;2024-01-15;true\nC;2024-01-15;Binance;BTC;trade;0.5;buy;45000;10;";
        assert_eq!(
            detect_format(text, "data.txt"),
            Some(ImportFormat::TextMixed)
        );
    }

    #[test]
    fn test_detect_empty() {
        assert_eq!(detect_format("", "file.json"), None);
        assert_eq!(detect_format("   ", "file.csv"), None);
    }
}
