//! Plain text parser for semicolon-separated imports
//!
//! Supports mixed content with prefixes:
//! - T;... = Transaction (Fiat)
//! - H;... = Habit Log
//! - C;... = Crypto Transaction

use super::{ImportParser, ParseResult};
use crate::features::ingestion::types::{
    ImportCryptoTransaction, ImportHabitLog, ImportTransaction, RowError,
};
use crate::features::ingestion::validation::parse_bool;

pub struct TextParser;

/// Result for parsing mixed text content
#[derive(Debug, Default)]
pub struct TextMixedParseResult {
    pub transactions: ParseResult<ImportTransaction>,
    pub habit_logs: ParseResult<ImportHabitLog>,
    pub crypto_transactions: ParseResult<ImportCryptoTransaction>,
}

impl ImportParser for TextParser {
    fn parse_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportTransaction>, RowError> {
        // For backwards compatibility, delegate to parse_mixed and extract transactions
        let result = self.parse_mixed(content);
        Ok(result.transactions)
    }

    fn parse_habit_logs(&self, content: &str) -> Result<ParseResult<ImportHabitLog>, RowError> {
        // For backwards compatibility, delegate to parse_mixed and extract habits
        let result = self.parse_mixed(content);
        Ok(result.habit_logs)
    }

    fn format_name(&self) -> &'static str {
        "Plain Text"
    }
}

impl TextParser {
    /// Parses mixed content with prefixes (T;, H;, C;)
    pub fn parse_mixed(&self, content: &str) -> TextMixedParseResult {
        let mut result = TextMixedParseResult::default();

        for (idx, line) in content.lines().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for prefix
            let upper = trimmed.to_uppercase();
            if upper.starts_with("T;") {
                self.parse_transaction_line(&trimmed[2..], line_number, &mut result.transactions);
            } else if upper.starts_with("H;") {
                self.parse_habit_line(&trimmed[2..], line_number, &mut result.habit_logs);
            } else if upper.starts_with("C;") {
                self.parse_crypto_line(&trimmed[2..], line_number, &mut result.crypto_transactions);
            } else {
                // No recognized prefix - report error
                result.transactions.errors.push(
                    RowError::new(
                        line_number,
                        None,
                        "Unrecognized line format. Expected prefix: T; (transaction), H; (habit), or C; (crypto)",
                    )
                    .with_raw_data(trimmed),
                );
            }
        }

        result
    }

    /// Parses crypto transactions from content (without prefix handling)
    pub fn parse_crypto_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError> {
        let result = self.parse_mixed(content);
        Ok(result.crypto_transactions)
    }

    /// Parse a transaction line (after T; prefix is removed)
    /// Format: date;account;type;amount;currency;category;description;transfer_to
    fn parse_transaction_line(
        &self,
        line: &str,
        line_number: usize,
        result: &mut ParseResult<ImportTransaction>,
    ) {
        let fields: Vec<&str> = line.split(';').collect();

        // Minimum 7 fields
        if fields.len() < 7 {
            result.errors.push(
                RowError::new(
                    line_number,
                    None,
                    format!(
                        "Invalid transaction: expected 7-8 fields (date;account;type;amount;currency;category;description;transfer_to), got {}",
                        fields.len()
                    ),
                )
                .with_raw_data(format!("T;{}", line)),
            );
            return;
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
                RowError::new(line_number, Some("date"), "Date is required")
                    .with_raw_data(format!("T;{}", line)),
            );
            return;
        }
        if account.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("account"), "Account is required")
                    .with_raw_data(format!("T;{}", line)),
            );
            return;
        }
        if tx_type.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("type"), "Type is required")
                    .with_raw_data(format!("T;{}", line)),
            );
            return;
        }
        if amount_str.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("amount"), "Amount is required")
                    .with_raw_data(format!("T;{}", line)),
            );
            return;
        }
        if currency.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("currency"), "Currency is required")
                    .with_raw_data(format!("T;{}", line)),
            );
            return;
        }
        if !tx_type.eq_ignore_ascii_case("transfer") && category.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("category"), "Category is required")
                    .with_raw_data(format!("T;{}", line)),
            );
            return;
        }

        let amount: f64 = match amount_str.replace(',', ".").parse() {
            Ok(value) => value,
            Err(_) => {
                result.errors.push(
                    RowError::new(
                        line_number,
                        Some("amount"),
                        format!("Invalid amount: '{}'", amount_str),
                    )
                    .with_raw_data(format!("T;{}", line)),
                );
                return;
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

    /// Parse a habit line (after H; prefix is removed)
    /// Format: habit;date;completed
    fn parse_habit_line(
        &self,
        line: &str,
        line_number: usize,
        result: &mut ParseResult<ImportHabitLog>,
    ) {
        let fields: Vec<&str> = line.split(';').collect();

        if fields.len() < 3 {
            result.errors.push(
                RowError::new(
                    line_number,
                    None,
                    format!(
                        "Invalid habit log: expected 3 fields (habit;date;completed), got {}",
                        fields.len()
                    ),
                )
                .with_raw_data(format!("H;{}", line)),
            );
            return;
        }

        let habit = fields[0].trim();
        let date = fields[1].trim();
        let completed_str = fields[2].trim();

        if habit.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("habit"), "Habit name is required")
                    .with_raw_data(format!("H;{}", line)),
            );
            return;
        }
        if date.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("date"), "Date is required")
                    .with_raw_data(format!("H;{}", line)),
            );
            return;
        }

        let completed = match parse_bool(completed_str) {
            Ok(value) => value,
            Err(e) => {
                result.errors.push(
                    RowError::new(line_number, Some("completed"), e)
                        .with_raw_data(format!("H;{}", line)),
                );
                return;
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

    /// Parse a crypto line (after C; prefix is removed)
    /// Format: date;wallet;symbol;type;amount;price;fee;notes
    fn parse_crypto_line(
        &self,
        line: &str,
        line_number: usize,
        result: &mut ParseResult<ImportCryptoTransaction>,
    ) {
        let fields: Vec<&str> = line.split(';').collect();

        // Minimum 5 required fields
        if fields.len() < 5 {
            result.errors.push(
                RowError::new(
                    line_number,
                    None,
                    format!(
                        "Invalid crypto transaction: expected at least 5 fields (date;wallet;symbol;type;amount;[price];[fee];[notes]), got {}",
                        fields.len()
                    ),
                )
                .with_raw_data(format!("C;{}", line)),
            );
            return;
        }

        let date = fields[0].trim();
        let wallet = fields[1].trim();
        let symbol = fields[2].trim();
        let tx_type = fields[3].trim();
        let amount_str = fields[4].trim();
        let price_str = fields.get(5).map(|s| s.trim()).filter(|s| !s.is_empty());
        let fee_str = fields.get(6).map(|s| s.trim()).filter(|s| !s.is_empty());
        let notes = fields.get(7).map(|s| s.trim()).filter(|s| !s.is_empty());

        // Validate required fields
        if date.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("date"), "Date is required")
                    .with_raw_data(format!("C;{}", line)),
            );
            return;
        }
        if wallet.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("wallet"), "Wallet is required")
                    .with_raw_data(format!("C;{}", line)),
            );
            return;
        }
        if symbol.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("symbol"), "Symbol is required")
                    .with_raw_data(format!("C;{}", line)),
            );
            return;
        }
        if tx_type.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("type"), "Type is required")
                    .with_raw_data(format!("C;{}", line)),
            );
            return;
        }
        if amount_str.is_empty() {
            result.errors.push(
                RowError::new(line_number, Some("amount"), "Amount is required")
                    .with_raw_data(format!("C;{}", line)),
            );
            return;
        }

        let amount: f64 = match amount_str.replace(',', ".").parse() {
            Ok(value) => value,
            Err(_) => {
                result.errors.push(
                    RowError::new(
                        line_number,
                        Some("amount"),
                        format!("Invalid amount: '{}'", amount_str),
                    )
                    .with_raw_data(format!("C;{}", line)),
                );
                return;
            }
        };

        let price_per_coin = price_str.and_then(|s| s.replace(',', ".").parse().ok());
        let fee = fee_str.and_then(|s| s.replace(',', ".").parse().ok());

        result.items.push((
            line_number,
            ImportCryptoTransaction {
                date: date.to_string(),
                wallet: wallet.to_string(),
                symbol: symbol.to_string(),
                transaction_type: tx_type.to_string(),
                amount,
                price_per_coin,
                fee,
                notes: notes.map(String::from),
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mixed_transactions() {
        let text = "T;2024-01-15;Checking;expense;45.50;USD;Food;Groceries;\n\
                    T;2024-01-14;Checking;transfer;500.00;USD;Transfer;Monthly savings;Savings";

        let parser = TextParser;
        let result = parser.parse_mixed(text);

        assert!(result.transactions.errors.is_empty());
        assert_eq!(result.transactions.items.len(), 2);
        assert_eq!(result.transactions.items[0].1.date, "2024-01-15");
        assert_eq!(result.transactions.items[0].1.amount, 45.50);
    }

    #[test]
    fn test_parse_mixed_habits() {
        let text = "H;Meditate;2024-01-15;true\n\
                    H;Exercise;2024-01-15;false";

        let parser = TextParser;
        let result = parser.parse_mixed(text);

        assert!(result.habit_logs.errors.is_empty());
        assert_eq!(result.habit_logs.items.len(), 2);
        assert!(result.habit_logs.items[0].1.completed);
        assert!(!result.habit_logs.items[1].1.completed);
    }

    #[test]
    fn test_parse_mixed_crypto() {
        let text = "C;2024-01-15;Binance;BTC;buy;0.5;45000;10;First BTC purchase\n\
                    C;2024-01-16;Coinbase;ETH;sell;2.0;2500;;";

        let parser = TextParser;
        let result = parser.parse_mixed(text);

        assert!(result.crypto_transactions.errors.is_empty());
        assert_eq!(result.crypto_transactions.items.len(), 2);
        assert_eq!(result.crypto_transactions.items[0].1.symbol, "BTC");
        assert_eq!(result.crypto_transactions.items[0].1.amount, 0.5);
        assert_eq!(result.crypto_transactions.items[0].1.price_per_coin, Some(45000.0));
        assert_eq!(result.crypto_transactions.items[1].1.symbol, "ETH");
        assert!(result.crypto_transactions.items[1].1.fee.is_none());
    }

    #[test]
    fn test_parse_mixed_all_types() {
        let text = "# My import file\n\
                    T;2024-01-15;Checking;expense;100;USD;Food;Groceries;\n\
                    H;Meditate;2024-01-15;true\n\
                    C;2024-01-15;Binance;BTC;buy;0.1;42000;5;\n\
                    \n\
                    # More entries\n\
                    T;2024-01-16;Savings;income;500;USD;Salary;Monthly pay;";

        let parser = TextParser;
        let result = parser.parse_mixed(text);

        assert_eq!(result.transactions.items.len(), 2);
        assert_eq!(result.habit_logs.items.len(), 1);
        assert_eq!(result.crypto_transactions.items.len(), 1);
    }

    #[test]
    fn test_case_insensitive_prefix() {
        let text = "t;2024-01-15;Checking;expense;100;USD;Food;Groceries;\n\
                    h;Meditate;2024-01-15;true\n\
                    c;2024-01-15;Binance;BTC;buy;0.1;42000;5;";

        let parser = TextParser;
        let result = parser.parse_mixed(text);

        assert_eq!(result.transactions.items.len(), 1);
        assert_eq!(result.habit_logs.items.len(), 1);
        assert_eq!(result.crypto_transactions.items.len(), 1);
    }

    #[test]
    fn test_skip_comments_and_empty() {
        let text = "# This is a comment\n\n   \n# Another comment\nH;Meditate;2024-01-15;true";
        let parser = TextParser;
        let result = parser.parse_mixed(text);
        assert_eq!(result.habit_logs.items.len(), 1);
    }

    #[test]
    fn test_unrecognized_prefix() {
        let text = "X;some;data;here";
        let parser = TextParser;
        let result = parser.parse_mixed(text);
        assert_eq!(result.transactions.errors.len(), 1);
    }
}
