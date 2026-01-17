//! JSON v1 parser for Sanctum Web exports

use super::{ImportParser, ParseResult};
use crate::features::ingestion::types::{
    ImportCryptoTransaction, ImportHabitLog, ImportTransaction, RowError,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JsonV1FileRaw {
    version: String,
    #[serde(default)]
    transactions: Vec<serde_json::Value>,
    #[serde(default)]
    habit_logs: Vec<serde_json::Value>,
    #[serde(default)]
    crypto_transactions: Vec<serde_json::Value>,
}

#[derive(Debug)]
pub struct JsonV1ParseResult {
    pub transactions: ParseResult<ImportTransaction>,
    pub habit_logs: ParseResult<ImportHabitLog>,
    pub crypto_transactions: ParseResult<ImportCryptoTransaction>,
}

pub struct JsonV1Parser;

impl ImportParser for JsonV1Parser {
    fn parse_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportTransaction>, RowError> {
        let file = self.parse_raw(content)?;
        Ok(parse_json_items(file.transactions, "transaction"))
    }

    fn parse_habit_logs(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportHabitLog>, RowError> {
        let file = self.parse_raw(content)?;
        Ok(parse_json_items(file.habit_logs, "habit log"))
    }

    fn format_name(&self) -> &'static str {
        "JSON v1"
    }
}

impl JsonV1Parser {
    /// Parses the full JSON file and returns transactions, habit logs, and crypto
    pub fn parse_full(&self, content: &str) -> Result<JsonV1ParseResult, RowError> {
        let file = self.parse_raw(content)?;
        Ok(JsonV1ParseResult {
            transactions: parse_json_items(file.transactions, "transaction"),
            habit_logs: parse_json_items(file.habit_logs, "habit log"),
            crypto_transactions: parse_json_items(file.crypto_transactions, "crypto transaction"),
        })
    }

    /// Parses crypto transactions from JSON content
    pub fn parse_crypto_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError> {
        let file = self.parse_raw(content)?;
        Ok(parse_json_items(file.crypto_transactions, "crypto transaction"))
    }

    fn parse_raw(&self, content: &str) -> Result<JsonV1FileRaw, RowError> {
        let file: JsonV1FileRaw =
            serde_json::from_str(content).map_err(|e| RowError::new(1, None, format!("Invalid JSON: {}", e)))?;

        // Validate version
        let version = file.version.trim();
        if version != "1.0" && version != "1" {
            return Err(RowError::new(
                1,
                Some("version"),
                format!("Unsupported JSON version: '{}'. Expected: 1.0", version),
            ));
        }

        Ok(file)
    }
}

fn parse_json_items<T>(items: Vec<serde_json::Value>, label: &str) -> ParseResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut result = ParseResult::default();
    for (index, value) in items.into_iter().enumerate() {
        match serde_json::from_value::<T>(value.clone()) {
            Ok(item) => result.items.push((index + 1, item)),
            Err(err) => {
                let raw = serde_json::to_string(&value).unwrap_or_default();
                let error = RowError::new(index + 1, None, format!("Invalid {}: {}", label, err))
                    .with_raw_data(raw);
                result.errors.push(error);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transactions() {
        let json = r#"{
            "version": "1.0",
            "transactions": [
                {
                    "date": "2024-01-15",
                    "account": "Checking",
                    "type": "expense",
                    "amount": 45.50,
                    "currency": "USD",
                    "category": "Food",
                    "description": "Groceries",
                    "transfer_to_account": null
                }
            ],
            "habit_logs": []
        }"#;

        let parser = JsonV1Parser;
        let result = parser.parse_transactions(json);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].0, 1); // line number
        assert_eq!(parsed.items[0].1.amount, 45.50);
        assert_eq!(parsed.items[0].1.account, "Checking");
    }

    #[test]
    fn test_parse_habit_logs() {
        let json = r#"{
            "version": "1.0",
            "transactions": [],
            "habit_logs": [
                {
                    "habit": "Meditate",
                    "date": "2024-01-15",
                    "completed": true
                },
                {
                    "habit": "Exercise",
                    "date": "2024-01-15",
                    "completed": false
                }
            ]
        }"#;

        let parser = JsonV1Parser;
        let result = parser.parse_habit_logs(json);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.items.len(), 2);
        assert!(parsed.items[0].1.completed);
        assert!(!parsed.items[1].1.completed);
    }

    #[test]
    fn test_invalid_version() {
        let json = r#"{"version": "2.0", "transactions": [], "habit_logs": []}"#;
        let parser = JsonV1Parser;
        let result = parser.parse_transactions(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json() {
        let parser = JsonV1Parser;
        let result = parser.parse_transactions("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_transactions_best_effort() {
        let json = r#"{
            "version": "1.0",
            "transactions": [
                { "date": "2024-01-15", "account": "Checking" },
                {
                    "date": "2024-01-16",
                    "account": "Checking",
                    "type": "expense",
                    "amount": 10.0,
                    "currency": "USD",
                    "category": "Food",
                    "description": "Lunch",
                    "transfer_to_account": null
                }
            ],
            "habit_logs": []
        }"#;

        let parser = JsonV1Parser;
        let result = parser.parse_transactions(json);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.errors.len(), 1);
    }
}
