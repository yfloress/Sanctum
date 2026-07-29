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

//! CSV parser for spreadsheet imports

use super::{ImportParser, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, ImportTransaction, RowError};
use csv::{ReaderBuilder, StringRecord, Trim};
use std::collections::HashMap;

pub struct CsvParser;

impl CsvParser {
    /// Parses CSV header and returns column indices
    fn parse_header(headers: &StringRecord) -> HashMap<String, usize> {
        headers
            .iter()
            .enumerate()
            .map(|(i, col)| (col.trim().to_lowercase(), i))
            .collect()
    }

    /// Gets a field value from a row by column name
    fn get_field<'a>(
        row: &'a StringRecord,
        columns: &HashMap<String, usize>,
        name: &str,
    ) -> Option<&'a str> {
        columns
            .get(name)
            .and_then(|&i| row.get(i).map(|s| s.trim()))
    }
}

impl ImportParser for CsvParser {
    fn parse_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportTransaction>, RowError> {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());
        let headers = reader
            .headers()
            .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {}", e)))?
            .clone();
        let columns = Self::parse_header(&headers);

        // Validate required columns exist
        let required = [
            "date",
            "account",
            "type",
            "amount",
            "currency",
            "category",
            "description",
        ];
        for col in required {
            if !columns.contains_key(col) {
                return Err(RowError::new(
                    1,
                    None,
                    format!(
                        "Missing required column: '{}'. Expected: {}",
                        col,
                        required.join(", ")
                    ),
                ));
            }
        }

        let mut result = ParseResult::default();

        for (idx, record) in reader.records().enumerate() {
            let record = match record {
                Ok(record) => record,
                Err(err) => {
                    let line = err.position().map(|p| p.line()).unwrap_or((idx + 2) as u64);
                    result.errors.push(RowError::new(
                        line as usize,
                        None,
                        format!("Invalid CSV record: {}", err),
                    ));
                    continue;
                }
            };

            let line_number = record
                .position()
                .map(|p| p.line())
                .unwrap_or((idx + 2) as u64) as usize;
            let raw_data = record.iter().collect::<Vec<_>>().join(",");

            // Skip empty lines
            if record.iter().all(|field| field.trim().is_empty()) {
                continue;
            }

            let date = match Self::get_field(&record, &columns, "date") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(line_number, Some("date"), "Missing required field: date")
                            .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let account = match Self::get_field(&record, &columns, "account") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("account"),
                            "Missing required field: account",
                        )
                        .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let tx_type = match Self::get_field(&record, &columns, "type") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(line_number, Some("type"), "Missing required field: type")
                            .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let amount_str = match Self::get_field(&record, &columns, "amount") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("amount"),
                            "Missing required field: amount",
                        )
                        .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let currency = match Self::get_field(&record, &columns, "currency") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("currency"),
                            "Missing required field: currency",
                        )
                        .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let description = Self::get_field(&record, &columns, "description").unwrap_or("");

            let tx_type_normalized = tx_type.trim().to_lowercase();
            let category_value = Self::get_field(&record, &columns, "category").unwrap_or("");
            if tx_type_normalized != "transfer" && category_value.trim().is_empty() {
                result.errors.push(
                    RowError::new(
                        line_number,
                        Some("category"),
                        "Missing required field: category",
                    )
                    .with_raw_data(raw_data),
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
                        .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            // Optional transfer_to_account
            let transfer_to_account = Self::get_field(&record, &columns, "transfer_to_account")
                .filter(|s| !s.is_empty())
                .map(String::from);

            result.items.push((
                line_number,
                ImportTransaction {
                    date: date.to_string(),
                    account: account.to_string(),
                    transaction_type: tx_type.to_string(),
                    amount,
                    currency: currency.to_string(),
                    category: category_value.to_string(),
                    description: description.to_string(),
                    transfer_to_account,
                },
            ));
        }

        Ok(result)
    }

    fn format_name(&self) -> &'static str {
        "CSV"
    }
}

impl CsvParser {
    /// Parses crypto transactions from CSV content
    pub fn parse_crypto_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError> {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());
        let headers = reader
            .headers()
            .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {}", e)))?
            .clone();
        let columns = Self::parse_header(&headers);

        // Validate required columns exist
        let required = ["date", "wallet", "symbol", "type", "amount"];
        for col in required {
            if !columns.contains_key(col) {
                return Err(RowError::new(
                    1,
                    None,
                    format!(
                        "Missing required column: '{}'. Expected: {}",
                        col,
                        required.join(", ")
                    ),
                ));
            }
        }

        let mut result = ParseResult::default();

        for (idx, record) in reader.records().enumerate() {
            let record = match record {
                Ok(record) => record,
                Err(err) => {
                    let line = err.position().map(|p| p.line()).unwrap_or((idx + 2) as u64);
                    result.errors.push(RowError::new(
                        line as usize,
                        None,
                        format!("Invalid CSV record: {}", err),
                    ));
                    continue;
                }
            };

            let line_number = record
                .position()
                .map(|p| p.line())
                .unwrap_or((idx + 2) as u64) as usize;
            let raw_data = record.iter().collect::<Vec<_>>().join(",");

            // Skip empty lines
            if record.iter().all(|field| field.trim().is_empty()) {
                continue;
            }

            let date = match Self::get_field(&record, &columns, "date") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(line_number, Some("date"), "Missing required field: date")
                            .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let wallet = match Self::get_field(&record, &columns, "wallet") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("wallet"),
                            "Missing required field: wallet",
                        )
                        .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let symbol = match Self::get_field(&record, &columns, "symbol") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("symbol"),
                            "Missing required field: symbol",
                        )
                        .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let tx_type = match Self::get_field(&record, &columns, "type") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(line_number, Some("type"), "Missing required field: type")
                            .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let amount_str = match Self::get_field(&record, &columns, "amount") {
                Some(value) if !value.is_empty() => value,
                _ => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("amount"),
                            "Missing required field: amount",
                        )
                        .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            let amount: f64 = match amount_str.replace(',', ".").parse() {
                Ok(value) => value,
                Err(_) => {
                    result.errors.push(
                        RowError::new(
                            line_number,
                            Some("amount"),
                            format!("Invalid amount: '{}'", amount_str),
                        )
                        .with_raw_data(raw_data),
                    );
                    continue;
                }
            };

            // Optional fields
            let price_per_coin = Self::get_field(&record, &columns, "price_per_coin")
                .or_else(|| Self::get_field(&record, &columns, "price"))
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());

            let fee = Self::get_field(&record, &columns, "fee")
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());

            // Subtype column (e.g. "buy", "airdrop", "deposit")
            let subtype = Self::get_field(&record, &columns, "subtype")
                .filter(|s| !s.is_empty())
                .map(String::from);

            let override_proceeds = Self::get_field(&record, &columns, "override_proceeds")
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());

            let override_cost_basis = Self::get_field(&record, &columns, "override_cost_basis")
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());

            let swap_to_symbol = Self::get_field(&record, &columns, "swap_to_symbol")
                .or_else(|| Self::get_field(&record, &columns, "to_symbol"))
                .filter(|s| !s.is_empty())
                .map(String::from);

            let swap_to_amount = Self::get_field(&record, &columns, "swap_to_amount")
                .or_else(|| Self::get_field(&record, &columns, "to_amount"))
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());

            let fee_coin_symbol = Self::get_field(&record, &columns, "fee_coin_symbol")
                .or_else(|| Self::get_field(&record, &columns, "fee_coin"))
                .filter(|s| !s.is_empty())
                .map(String::from);

            let fee_amount = Self::get_field(&record, &columns, "fee_amount")
                .filter(|s| !s.is_empty())
                .and_then(|s| s.replace(',', ".").parse().ok());

            let notes = Self::get_field(&record, &columns, "notes")
                .filter(|s| !s.is_empty())
                .map(String::from);

            result.items.push((
                line_number,
                ImportCryptoTransaction {
                    date: date.to_string(),
                    wallet: wallet.to_string(),
                    symbol: symbol.to_string(),
                    transaction_type: tx_type.to_string(),
                    amount,
                    subtype,
                    price_per_coin,
                    fee,
                    override_proceeds,
                    override_cost_basis,
                    swap_to_symbol,
                    swap_to_amount,
                    fee_coin_symbol,
                    fee_amount,
                    notes,
                },
            ));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_transactions() {
        let csv = "date,account,type,amount,currency,category,description,transfer_to_account\n\
                   2024-01-15,Checking,expense,45.50,USD,Food,Groceries,\n\
                   2024-01-14,Checking,transfer,500.00,USD,Transfer,Monthly savings,Savings";

        let parser = CsvParser;
        let result = parser.parse_transactions(csv);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.items.len(), 2);

        assert_eq!(parsed.items[0].1.date, "2024-01-15");
        assert_eq!(parsed.items[0].1.amount, 45.50);
        assert!(parsed.items[0].1.transfer_to_account.is_none());

        assert_eq!(parsed.items[1].1.transaction_type, "transfer");
        assert_eq!(
            parsed.items[1].1.transfer_to_account,
            Some("Savings".to_string())
        );
    }

    #[test]
    fn test_parse_csv_transactions_best_effort() {
        let csv = "date,account,type,amount,currency,category,description,transfer_to_account\n\
                   2024-01-15,Checking,expense,invalid,USD,Food,Groceries,\n\
                   2024-01-16,Checking,expense,10.00,USD,Food,Lunch,";

        let parser = CsvParser;
        let result = parser.parse_transactions(csv);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.errors.len(), 1);
    }

    #[test]
    fn test_missing_column() {
        let csv = "date,account,amount\n2024-01-15,Checking,100";
        let parser = CsvParser;
        let result = parser.parse_transactions(csv);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_file() {
        let parser = CsvParser;
        let result = parser.parse_transactions("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_csv_crypto_transactions_with_subtype_and_swap_fields() {
        let csv = "date,wallet,symbol,type,subtype,amount,price_per_coin,fee,swap_to_symbol,swap_to_amount,fee_coin_symbol,fee_amount,notes\n\
                   2026-01-10,Ledger,BTC,trade,buy,0.5,97000,10,,,,,Spot buy\n\
                   2026-01-11,Binance,BTC,trade,swap,0.1,97000,5,ETH,3.0,BNB,0.01,Swap BTC to ETH";

        let parser = CsvParser;
        let result = parser.parse_crypto_transactions(csv);
        assert!(result.is_ok());

        let parsed = result.expect("csv crypto parse");
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.items.len(), 2);

        let buy = &parsed.items[0].1;
        assert_eq!(buy.transaction_type, "trade");
        assert_eq!(buy.subtype.as_deref(), Some("buy"));
        assert_eq!(buy.amount, 0.5);
        assert_eq!(buy.notes.as_deref(), Some("Spot buy"));

        let swap = &parsed.items[1].1;
        assert_eq!(swap.transaction_type, "trade");
        assert_eq!(swap.subtype.as_deref(), Some("swap"));
        assert_eq!(swap.swap_to_symbol.as_deref(), Some("ETH"));
        assert_eq!(swap.swap_to_amount, Some(3.0));
        assert_eq!(swap.fee_coin_symbol.as_deref(), Some("BNB"));
        assert_eq!(swap.fee_amount, Some(0.01));
    }

    #[test]
    fn test_parse_csv_crypto_alias_columns_for_price_and_swap_target() {
        let csv = "date,wallet,symbol,type,subtype,amount,price,to_symbol,to_amount,fee_coin,fee_amount\n\
                   2026-01-12,Exchange,BTC,trade,swap,0.2,98000,ETH,6.2,USDT,1.5";

        let parser = CsvParser;
        let result = parser.parse_crypto_transactions(csv);
        assert!(result.is_ok());

        let parsed = result.expect("csv crypto parse");
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.items.len(), 1);

        let tx = &parsed.items[0].1;
        assert_eq!(tx.price_per_coin, Some(98000.0));
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
        assert_eq!(tx.swap_to_amount, Some(6.2));
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("USDT"));
        assert_eq!(tx.fee_amount, Some(1.5));
    }

    #[test]
    fn test_parse_csv_crypto_best_effort_with_invalid_amount() {
        let csv = "date,wallet,symbol,type,subtype,amount\n\
                   2026-01-10,Ledger,BTC,trade,buy,invalid\n\
                   2026-01-11,Ledger,ETH,income,airdrop,1.25";

        let parser = CsvParser;
        let result = parser.parse_crypto_transactions(csv);
        assert!(result.is_ok());

        let parsed = result.expect("csv crypto parse");
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.items[0].1.symbol, "ETH");
        assert_eq!(parsed.items[0].1.subtype.as_deref(), Some("airdrop"));
    }

    #[test]
    fn test_parse_csv_crypto_missing_required_column() {
        let csv = "date,wallet,type,amount\n2026-01-10,Ledger,trade,0.5";
        let parser = CsvParser;
        let result = parser.parse_crypto_transactions(csv);
        assert!(result.is_err());
    }
}
