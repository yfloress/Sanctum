//! Parsers for different import formats
//!
//! Supports JSON v1, CSV, and plain text formats.

pub mod csv;
pub mod json;
pub mod text;

pub use self::csv::CsvParser;
pub use self::json::JsonV1Parser;
pub use self::text::TextParser;

use super::types::{ImportFormat, ImportHabitLog, ImportTransaction, RowError};

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
    if ext == "json" || trimmed.starts_with('{') {
        if trimmed.contains("\"version\"") && (trimmed.contains("\"1.0\"") || trimmed.contains("\"1\"")) {
            return Some(ImportFormat::JsonV1);
        }
        // Try to parse as JSON anyway if it looks like JSON
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return Some(ImportFormat::JsonV1);
        }
    }

    // CSV detection (has comma-separated header on first line)
    if ext == "csv" {
        let first_line = trimmed.lines().next()?.to_lowercase();
        // Must have commas and not start with # (comment)
        if first_line.contains(',') && !first_line.starts_with('#') {
            if first_line.contains("habit") && first_line.contains("date") && first_line.contains("completed") {
                return Some(ImportFormat::CsvHabitLogs);
            }
            if first_line.contains("account") && first_line.contains("amount") {
                return Some(ImportFormat::CsvTransactions);
            }
        }
    }

    // Text detection (semicolon-separated)
    if ext == "txt" || trimmed.contains(';') {
        // Look for comment hints
        for line in trimmed.lines() {
            let line_lower = line.trim().to_lowercase();
            if line_lower.starts_with("# transaction") {
                return Some(ImportFormat::TextTransactions);
            }
            if line_lower.starts_with("# habit") {
                return Some(ImportFormat::TextHabitLogs);
            }
        }

        // Fallback: check first data line structure
        for line in trimmed.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with('#') || line_trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line_trimmed.split(';').collect();
            // 3 fields = habit log (habit;date;completed)
            // 7-8 fields = transaction
            if parts.len() == 3 {
                return Some(ImportFormat::TextHabitLogs);
            } else if parts.len() >= 7 {
                return Some(ImportFormat::TextTransactions);
            }
            break;
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
        assert_eq!(detect_format(json, "data.json"), Some(ImportFormat::JsonV1));
    }

    #[test]
    fn test_detect_csv_transactions() {
        let csv = "date,account,type,amount,currency,category,description,transfer_to_account\n";
        assert_eq!(detect_format(csv, "data.csv"), Some(ImportFormat::CsvTransactions));
    }

    #[test]
    fn test_detect_csv_habits() {
        let csv = "habit,date,completed\n";
        assert_eq!(detect_format(csv, "habits.csv"), Some(ImportFormat::CsvHabitLogs));
    }

    #[test]
    fn test_detect_text_transactions() {
        let text = "# Transactions\n2024-01-15;Account;expense;100;USD;Food;Groceries;";
        assert_eq!(detect_format(text, "data.txt"), Some(ImportFormat::TextTransactions));
    }

    #[test]
    fn test_detect_text_habits() {
        let text = "# Habit Logs\nMeditate;2024-01-15;true";
        assert_eq!(detect_format(text, "habits.txt"), Some(ImportFormat::TextHabitLogs));
    }

    #[test]
    fn test_detect_empty() {
        assert_eq!(detect_format("", "file.json"), None);
        assert_eq!(detect_format("   ", "file.csv"), None);
    }
}
