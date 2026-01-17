//! Plain text parser for semicolon-separated imports

use super::{ImportParser, ParseResult};
use crate::features::ingestion::types::{ImportHabitLog, ImportTransaction, RowError};
use crate::features::ingestion::validation::parse_bool;

pub struct TextParser;

impl ImportParser for TextParser {
    fn parse_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportTransaction>, RowError> {
        let mut result = ParseResult::default();

        for (idx, line) in content.lines().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Split by semicolon
            let fields: Vec<&str> = trimmed.split(';').collect();

            // Expected: date;account;type;amount;currency;category;description;transfer_to_account
            // Minimum 7 fields (transfer_to_account can be empty or missing)
            if fields.len() < 7 {
                result.errors.push(
                    RowError::new(
                        line_number,
                        None,
                        format!(
                            "Invalid line format: expected 7-8 fields (date;account;type;amount;currency;category;description;transfer_to_account), got {}",
                            fields.len()
                        ),
                    )
                    .with_raw_data(trimmed),
                );
                continue;
            }

            let date = fields[0].trim();
            let account = fields[1].trim();
            let tx_type = fields[2].trim();
            let amount_str = fields[3].trim();
            let currency = fields[4].trim();
            let category = fields[5].trim();
            let description = fields[6].trim();
            let transfer_to = fields.get(7).map(|s| s.trim()).filter(|s| !s.is_empty());

            // Validate required fields
            if date.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("date"), "Date is required").with_raw_data(trimmed),
                );
                continue;
            }
            if account.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("account"), "Account is required")
                        .with_raw_data(trimmed),
                );
                continue;
            }
            if tx_type.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("type"), "Type is required").with_raw_data(trimmed),
                );
                continue;
            }
            if amount_str.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("amount"), "Amount is required")
                        .with_raw_data(trimmed),
                );
                continue;
            }
            if currency.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("currency"), "Currency is required")
                        .with_raw_data(trimmed),
                );
                continue;
            }

            if !tx_type.trim().eq_ignore_ascii_case("transfer") && category.trim().is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("category"), "Category is required")
                        .with_raw_data(trimmed),
                );
                continue;
            }

            // Parse amount
            let amount: f64 = match amount_str.replace(',', ".").parse() {
                Ok(value) => value,
                Err(_) => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("amount"),
                            format!("Invalid amount: '{}'", amount_str),
                        )
                        .with_raw_data(trimmed),
                    );
                    continue;
                }
            };

            result.items.push((
                line_number,
                ImportTransaction {
                    date: date.to_string(),
                    account: account.to_string(),
                    transaction_type: tx_type.to_string(),
                    amount,
                    currency: currency.to_string(),
                    category: category.to_string(),
                    description: description.to_string(),
                    transfer_to_account: transfer_to.map(String::from),
                },
            ));
        }

        Ok(result)
    }

    fn parse_habit_logs(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportHabitLog>, RowError> {
        let mut result = ParseResult::default();

        for (idx, line) in content.lines().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Split by semicolon
            let fields: Vec<&str> = trimmed.split(';').collect();

            // Expected: habit;date;completed
            if fields.len() < 3 {
                result.errors.push(
                    RowError::new(
                        line_number,
                        None,
                        format!(
                            "Invalid line format: expected 3 fields (habit;date;completed), got {}",
                            fields.len()
                        ),
                    )
                    .with_raw_data(trimmed),
                );
                continue;
            }

            let habit = fields[0].trim();
            let date = fields[1].trim();
            let completed_str = fields[2].trim();

            // Validate required fields
            if habit.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("habit"), "Habit name is required")
                        .with_raw_data(trimmed),
                );
                continue;
            }
            if date.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("date"), "Date is required").with_raw_data(trimmed),
                );
                continue;
            }

            let completed = match parse_bool(completed_str) {
                Ok(value) => value,
                Err(e) => {
                    result.errors.push(
                        RowError::new(line_number, Some("completed"), e).with_raw_data(trimmed),
                    );
                    continue;
                }
            };

            result.items.push((
                line_number,
                ImportHabitLog {
                    habit: habit.to_string(),
                    date: date.to_string(),
                    completed,
                },
            ));
        }

        Ok(result)
    }

    fn format_name(&self) -> &'static str {
        "Plain Text"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_transactions() {
        let text = "# Transactions\n\
                    2024-01-15;Checking;expense;45.50;USD;Food;Groceries;\n\
                    2024-01-14;Checking;transfer;500.00;USD;Transfer;Monthly savings;Savings";

        let parser = TextParser;
        let result = parser.parse_transactions(text);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.items.len(), 2);

        assert_eq!(parsed.items[0].1.date, "2024-01-15");
        assert_eq!(parsed.items[0].1.amount, 45.50);
        assert!(parsed.items[0].1.transfer_to_account.is_none());

        assert_eq!(parsed.items[1].1.transaction_type, "transfer");
        assert_eq!(parsed.items[1].1.transfer_to_account, Some("Savings".to_string()));
    }

    #[test]
    fn test_parse_text_habit_logs() {
        let text = "# Habit Logs\n\
                    Meditate;2024-01-15;true\n\
                    Exercise;2024-01-15;false";

        let parser = TextParser;
        let result = parser.parse_habit_logs(text);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.items.len(), 2);
        assert!(parsed.items[0].1.completed);
        assert!(!parsed.items[1].1.completed);
    }

    #[test]
    fn test_skip_comments_and_empty() {
        let text = "# This is a comment\n\n   \n# Another comment\nMeditate;2024-01-15;true";
        let parser = TextParser;
        let result = parser.parse_habit_logs(text);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.items.len(), 1);
    }

    #[test]
    fn test_invalid_field_count() {
        let text = "Meditate;2024-01-15";
        let parser = TextParser;
        let result = parser.parse_habit_logs(text);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.items.len(), 0);
        assert_eq!(parsed.errors.len(), 1);
    }

    #[test]
    fn test_parse_text_transactions_best_effort() {
        let text = "2024-01-15;Checking;expense;invalid;USD;Food;Groceries;\n\
                    2024-01-16;Checking;expense;10.00;USD;Food;Lunch;";

        let parser = TextParser;
        let result = parser.parse_transactions(text);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.errors.len(), 1);
    }
}
