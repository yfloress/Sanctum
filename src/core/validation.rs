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

//! Shared validation utilities
//!
//! Common validation functions used across finance, crypto, and other domains.
//! Centralizing these prevents code duplication and ensures consistent validation rules.

use chrono::NaiveDate;
use uuid::Uuid;

/// Validates and trims a field to a maximum length
pub fn validate_field_length(
    value: &str,
    max_length: usize,
    field_name: &str,
) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.len() > max_length {
        return Err(format!(
            "{} exceeds maximum length of {} characters",
            field_name, max_length
        ));
    }
    Ok(trimmed.to_string())
}

/// Sanitizes a string by removing potentially dangerous characters
/// Allows alphanumeric, whitespace, and common punctuation
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric()
                || c.is_whitespace()
                || matches!(
                    c,
                    '!' | '@'
                        | '#'
                        | '$'
                        | '%'
                        | '^'
                        | '&'
                        | '*'
                        | '('
                        | ')'
                        | '-'
                        | '_'
                        | '+'
                        | '='
                        | '{'
                        | '}'
                        | '['
                        | ']'
                        | '|'
                        | '\\'
                        | ':'
                        | '\''
                        | '"'
                        | ','
                        | '.'
                        | '<'
                        | '>'
                        | '?'
                        | '/'
                        | '`'
                        | '~'
                )
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validates a UUID string
pub fn validate_uuid(id: &str) -> Result<String, String> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err("ID cannot be empty".to_string());
    }

    if Uuid::parse_str(trimmed).is_ok() {
        return Ok(trimmed.to_string());
    }

    Err("Invalid ID format".to_string())
}

/// Validates a date string, accepting DD-MM-YYYY or YYYY-MM-DD formats
/// Returns normalized YYYY-MM-DD format
pub fn validate_date(date: &str) -> Result<String, String> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return Err("Date cannot be empty".to_string());
    }

    // Try DD-MM-YYYY format first
    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%d-%m-%Y") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    // Try YYYY-MM-DD format
    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    Err("Invalid date format. Use DD-MM-YYYY or YYYY-MM-DD".to_string())
}

/// Validates a hex color string (#RRGGBB format)
pub fn validate_color(color: &str) -> Result<String, String> {
    let trimmed = color.trim();

    if trimmed.is_empty() {
        return Err("Color cannot be empty".to_string());
    }

    if trimmed.len() != 7 {
        return Err("Color must be in #RRGGBB format".to_string());
    }

    if !trimmed.starts_with('#') {
        return Err("Color must start with #".to_string());
    }

    if !trimmed[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Color must contain valid hex characters".to_string());
    }

    Ok(trimmed.to_lowercase())
}

/// Escapes a value for safe inclusion in a CSV field.
///
/// Wraps the value in double quotes if it contains commas, newlines, or quotes.
/// Any internal double quotes are doubled per RFC 4180.
pub fn csv_escape(value: &str) -> String {
    let needs_quotes = value.contains(',') || value.contains('\n') || value.contains('"');
    if !needs_quotes {
        return value.to_string();
    }

    let escaped = value.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

/// Formats money value (in cents) for display
pub fn format_money_display(value: i64) -> String {
    let abs = value.abs();
    let units = abs / 100;
    let cents = abs % 100;
    let sign = if value < 0 { "-" } else { "" };
    format!("{sign}$ {units}.{cents:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_uuid_valid() {
        let valid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(validate_uuid(valid).is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        assert!(validate_uuid("not-a-uuid").is_err());
        assert!(validate_uuid("").is_err());
    }

    #[test]
    fn test_validate_date_formats() {
        assert_eq!(validate_date("25-12-2024").unwrap(), "2024-12-25");
        assert_eq!(validate_date("2024-12-25").unwrap(), "2024-12-25");
        assert!(validate_date("invalid").is_err());
    }

    #[test]
    fn test_validate_color() {
        assert_eq!(validate_color("#FF0000").unwrap(), "#ff0000");
        assert!(validate_color("FF0000").is_err());
        assert!(validate_color("#GGG").is_err());
    }

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("Hello World!"), "Hello World!");
        assert_eq!(sanitize_string("  trimmed  "), "trimmed");
    }

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_with_comma() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
    }

    #[test]
    fn test_csv_escape_with_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_csv_escape_with_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }
}
