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

//! Plain text parser for semicolon-separated imports
//!
//! Supports mixed content with prefixes:
//! - T;... = Transaction (Fiat)
//! - H;... = Habit Log
//! - C;... = Crypto Transaction

use super::ParseResult;
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

impl TextParser {
    pub fn format_name(&self) -> &'static str {
        "Plain Text"
    }

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
    /// Format (standard): date;wallet;symbol;type;amount;price;fee;notes
    /// Format (swap): date;wallet;symbol;type;amount;swap_to_symbol;swap_to_amount;fee;fee_coin_symbol;fee_amount;notes
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

        let is_swap = tx_type.eq_ignore_ascii_case("swap");
        let (price_per_coin, fee, swap_to_symbol, swap_to_amount, fee_coin_symbol, fee_amount, notes) =
            if is_swap {
            let to_symbol = fields.get(5).map(|s| s.trim()).unwrap_or("");
            let to_amount_str = fields.get(6).map(|s| s.trim()).unwrap_or("");

            if to_symbol.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("swap_to_symbol"), "Swap target symbol is required")
                        .with_raw_data(format!("C;{}", line)),
                );
                return;
            }
            if to_amount_str.is_empty() {
                result.errors.push(
                    RowError::new(line_number, Some("swap_to_amount"), "Swap target amount is required")
                        .with_raw_data(format!("C;{}", line)),
                );
                return;
            }

            let to_amount: f64 = match to_amount_str.replace(',', ".").parse() {
                Ok(value) => value,
                Err(_) => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("swap_to_amount"),
                            format!("Invalid swap target amount: '{}'", to_amount_str),
                        )
                        .with_raw_data(format!("C;{}", line)),
                    );
                    return;
                }
            };

            let fee = fields
                .get(7)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());
            let fee_coin_symbol = fields
                .get(8)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(String::from);
            let fee_amount = fields
                .get(9)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());
            let notes = fields
                .get(10)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(String::from);
            (
                None,
                fee,
                Some(to_symbol.to_string()),
                Some(to_amount),
                fee_coin_symbol,
                fee_amount,
                notes,
            )
        } else {
            let price_per_coin = fields
                .get(5)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());
            let fee = fields
                .get(6)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());
            let notes = fields
                .get(7)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(String::from);
            (price_per_coin, fee, None, None, None, None, notes)
        };

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
                swap_to_symbol,
                swap_to_amount,
                fee_coin_symbol,
                fee_amount,
                notes,
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
    fn test_parse_mixed_crypto_swap() {
        let text = "C;2024-01-20;Binance;BTC;swap;0.1;ETH;2.5;0.01;BTC;0.0001;Swap note";

        let parser = TextParser;
        let result = parser.parse_mixed(text);

        assert!(result.crypto_transactions.errors.is_empty());
        assert_eq!(result.crypto_transactions.items.len(), 1);
        let tx = &result.crypto_transactions.items[0].1;
        assert_eq!(tx.transaction_type, "swap");
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
        assert_eq!(tx.swap_to_amount, Some(2.5));
        assert_eq!(tx.fee, Some(0.01));
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("BTC"));
        assert_eq!(tx.fee_amount, Some(0.0001));
        assert_eq!(tx.notes.as_deref(), Some("Swap note"));
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
