// Sanctum — a privacy-first personal finance and crypto vault.
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

//! JSON parser for Sanctum Web exports

use super::{ImportParser, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, ImportTransaction, RowError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JsonFileRaw {
    #[serde(default)]
    transactions: Vec<serde_json::Value>,
    #[serde(default)]
    crypto_transactions: Vec<serde_json::Value>,
}

#[derive(Debug)]
pub struct JsonParseResult {
    pub transactions: ParseResult<ImportTransaction>,
    pub crypto_transactions: ParseResult<ImportCryptoTransaction>,
}

pub struct JsonParser;

impl ImportParser for JsonParser {
    fn parse_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportTransaction>, RowError> {
        let file = self.parse_raw(content)?;
        Ok(parse_json_items(file.transactions, "transaction"))
    }

    fn format_name(&self) -> &'static str {
        "JSON"
    }
}

impl JsonParser {
    /// Parses the full JSON file and returns transactions and crypto
    pub fn parse_full(&self, content: &str) -> Result<JsonParseResult, RowError> {
        let file = self.parse_raw(content)?;
        Ok(JsonParseResult {
            transactions: parse_json_items(file.transactions, "transaction"),
            crypto_transactions: parse_json_items(file.crypto_transactions, "crypto transaction"),
        })
    }

    /// Parses crypto transactions from JSON content
    pub fn parse_crypto_transactions(
        &self,
        content: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError> {
        let file = self.parse_raw(content)?;
        Ok(parse_json_items(
            file.crypto_transactions,
            "crypto transaction",
        ))
    }

    fn parse_raw(&self, content: &str) -> Result<JsonFileRaw, RowError> {
        serde_json::from_str(content)
            .map_err(|e| RowError::new(1, None, format!("Invalid JSON: {}", e)))
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

        let parser = JsonParser;
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
    fn test_invalid_json() {
        let parser = JsonParser;
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

        let parser = JsonParser;
        let result = parser.parse_transactions(json);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.errors.len(), 1);
    }

    /// Simulates the exact JSON the web generator produces for every crypto
    /// scenario category and validates that the JSON parser + validation
    /// accept them all without errors.
    #[test]
    fn test_parse_generator_crypto_all_scenarios() {
        use crate::features::ingestion::validation::validate_import_crypto_transaction;

        // New format: type = transaction type category, subtype = specific action.
        // Legacy split fields are no longer supported.
        let json = r#"{
            "version": "1.0",
            "exported_at": "2026-02-10T12:00:00.000Z",
            "transactions": [],
            "habit_logs": [],
            "crypto_transactions": [
                {
                    "date": "2026-01-10",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "trade",
                    "subtype": "buy",
                    "amount": 0.5,
                    "price_per_coin": 97000.0,
                    "fee": 10.0,
                    "notes": null
                },
                {
                    "date": "2026-01-11",
                    "wallet": "Binance",
                    "symbol": "ETH",
                    "type": "trade",
                    "subtype": "sell",
                    "amount": 2.0,
                    "price_per_coin": 3200.0,
                    "fee": 5.0,
                    "notes": "Sold some ETH"
                },
                {
                    "date": "2026-01-12",
                    "wallet": "Binance",
                    "symbol": "BTC",
                    "type": "trade",
                    "subtype": "swap",
                    "amount": 0.1,
                    "price_per_coin": 97000.0,
                    "swap_to_symbol": "ETH",
                    "swap_to_amount": 3.0,
                    "fee_coin_symbol": "BNB",
                    "fee_amount": 0.01,
                    "notes": null
                },
                {
                    "date": "2026-01-13",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "trade",
                    "subtype": "other",
                    "amount": 0.001,
                    "notes": "OTC deal"
                },
                {
                    "date": "2026-01-14",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "transfer",
                    "subtype": "deposit",
                    "amount": 1.0,
                    "notes": null
                },
                {
                    "date": "2026-01-15",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "transfer",
                    "subtype": "withdrawal",
                    "amount": 0.5,
                    "notes": null
                },
                {
                    "date": "2026-02-01",
                    "wallet": "Ledger",
                    "symbol": "ETH",
                    "type": "income",
                    "subtype": "airdrop",
                    "amount": 0.5,
                    "price_per_coin": 3200.0,
                    "notes": "Free ETH airdrop"
                },
                {
                    "date": "2026-02-02",
                    "wallet": "Ledger",
                    "symbol": "SOL",
                    "type": "income",
                    "subtype": "staking",
                    "amount": 10.0,
                    "price_per_coin": 180.0,
                    "notes": null
                },
                {
                    "date": "2026-02-03",
                    "wallet": "Ledger",
                    "symbol": "XMR",
                    "type": "income",
                    "subtype": "mining",
                    "amount": 1.0,
                    "price_per_coin": 200.0,
                    "notes": null
                },
                {
                    "date": "2026-02-04",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "income",
                    "subtype": "interest",
                    "amount": 0.001,
                    "price_per_coin": 97000.0,
                    "notes": null
                },
                {
                    "date": "2026-02-05",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "income",
                    "subtype": "gift",
                    "amount": 0.01,
                    "notes": null
                },
                {
                    "date": "2026-02-06",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "income",
                    "subtype": "fork",
                    "amount": 0.005,
                    "notes": null
                },
                {
                    "date": "2026-02-07",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "income",
                    "subtype": "payment",
                    "amount": 0.002,
                    "price_per_coin": 97000.0,
                    "notes": null
                },
                {
                    "date": "2026-02-08",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "income",
                    "subtype": "rebate",
                    "amount": 0.0001,
                    "notes": null
                },
                {
                    "date": "2026-02-09",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "income",
                    "subtype": "reward",
                    "amount": 0.0005,
                    "notes": null
                },
                {
                    "date": "2026-02-10",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "income",
                    "subtype": "other",
                    "amount": 0.0003,
                    "notes": null
                },
                {
                    "date": "2026-03-01",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "expense",
                    "subtype": "payment",
                    "amount": 0.01,
                    "price_per_coin": 97000.0,
                    "notes": null
                },
                {
                    "date": "2026-03-02",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "expense",
                    "subtype": "gift",
                    "amount": 0.005,
                    "notes": null
                },
                {
                    "date": "2026-03-03",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "expense",
                    "subtype": "fee",
                    "amount": 0.001,
                    "notes": null
                },
                {
                    "date": "2026-03-04",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "expense",
                    "subtype": "lost",
                    "amount": 0.1,
                    "notes": null
                },
                {
                    "date": "2026-03-05",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "expense",
                    "subtype": "stolen",
                    "amount": 0.05,
                    "notes": null
                },
                {
                    "date": "2026-03-06",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "expense",
                    "subtype": "donation",
                    "amount": 0.002,
                    "notes": null
                },
                {
                    "date": "2026-03-07",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "expense",
                    "subtype": "sell",
                    "amount": 0.003,
                    "notes": null
                },
                {
                    "date": "2026-03-08",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "expense",
                    "subtype": "other",
                    "amount": 0.001,
                    "notes": null
                },
                {
                    "date": "2026-03-09",
                    "wallet": "Ledger",
                    "symbol": "BTC",
                    "type": "trade",
                    "subtype": "buy",
                    "amount": 0.01,
                    "price_per_coin": 97000.0,
                    "fee": 5.0,
                    "fee_coin_symbol": "ETH",
                    "fee_amount": 0.003,
                    "override_cost_basis": 980.0,
                    "notes": "Buy with crypto fee and cost basis override"
                }
            ]
        }"#;

        let parser = JsonParser;
        let result = parser.parse_full(json).expect("JSON parse should succeed");

        // All 25 crypto transactions should parse without errors
        assert!(
            result.crypto_transactions.errors.is_empty(),
            "Parse errors: {:?}",
            result
                .crypto_transactions
                .errors
                .iter()
                .map(|e| &e.message)
                .collect::<Vec<_>>()
        );
        assert_eq!(result.crypto_transactions.items.len(), 25);

        // Validate every parsed transaction passes ingestion validation
        for (line, tx) in &result.crypto_transactions.items {
            validate_import_crypto_transaction(tx, *line).unwrap_or_else(|e| {
                panic!(
                    "Validation failed for line {} (type={}, subtype={:?}): {}",
                    e.line_number, tx.transaction_type, tx.subtype, e.message
                )
            });
        }

        // Spot-check: trade:buy
        let buy = &result.crypto_transactions.items[0].1;
        assert_eq!(buy.transaction_type, "trade");
        assert_eq!(buy.subtype.as_deref(), Some("buy"));
        assert_eq!(buy.symbol, "BTC");
        assert_eq!(buy.amount, 0.5);
        assert_eq!(buy.mechanical_type(), "buy");

        // Spot-check: trade:swap with fee coin
        let swap = &result.crypto_transactions.items[2].1;
        assert_eq!(swap.transaction_type, "trade");
        assert_eq!(swap.subtype.as_deref(), Some("swap"));
        assert_eq!(swap.mechanical_type(), "swap");
        assert_eq!(swap.swap_to_symbol.as_deref(), Some("ETH"));
        assert_eq!(swap.swap_to_amount, Some(3.0));
        assert_eq!(swap.fee_coin_symbol.as_deref(), Some("BNB"));
        assert_eq!(swap.fee_amount, Some(0.01));

        // Spot-check: trade:other
        let trade_other = &result.crypto_transactions.items[3].1;
        assert_eq!(trade_other.transaction_type, "trade");
        assert_eq!(trade_other.subtype.as_deref(), Some("other"));
        assert_eq!(trade_other.mechanical_type(), "buy");

        // Spot-check: transfer:deposit
        let transfer_in = &result.crypto_transactions.items[4].1;
        assert_eq!(transfer_in.transaction_type, "transfer");
        assert_eq!(transfer_in.subtype.as_deref(), Some("deposit"));
        assert_eq!(transfer_in.mechanical_type(), "transfer_in");

        // Spot-check: transfer:withdrawal
        let transfer_out = &result.crypto_transactions.items[5].1;
        assert_eq!(transfer_out.transaction_type, "transfer");
        assert_eq!(transfer_out.subtype.as_deref(), Some("withdrawal"));
        assert_eq!(transfer_out.mechanical_type(), "transfer_out");

        // Spot-check: income:airdrop
        let airdrop = &result.crypto_transactions.items[6].1;
        assert_eq!(airdrop.transaction_type, "income");
        assert_eq!(airdrop.subtype.as_deref(), Some("airdrop"));
        assert_eq!(airdrop.mechanical_type(), "buy");

        // Spot-check: income:staking
        let staking = &result.crypto_transactions.items[7].1;
        assert_eq!(staking.transaction_type, "income");
        assert_eq!(staking.subtype.as_deref(), Some("staking"));

        // Spot-check: expense:stolen
        let stolen = &result.crypto_transactions.items[20].1;
        assert_eq!(stolen.transaction_type, "expense");
        assert_eq!(stolen.subtype.as_deref(), Some("stolen"));
        assert_eq!(stolen.mechanical_type(), "sell");

        // Spot-check: expense:sell (liquidation)
        let liquidation = &result.crypto_transactions.items[22].1;
        assert_eq!(liquidation.transaction_type, "expense");
        assert_eq!(liquidation.subtype.as_deref(), Some("sell"));
        assert_eq!(liquidation.mechanical_type(), "sell");

        // Spot-check: trade:buy with crypto fee + cost basis override
        let fee_override = &result.crypto_transactions.items[24].1;
        assert_eq!(fee_override.transaction_type, "trade");
        assert_eq!(fee_override.subtype.as_deref(), Some("buy"));
        assert_eq!(fee_override.fee_coin_symbol.as_deref(), Some("ETH"));
        assert_eq!(fee_override.fee_amount, Some(0.003));
        assert_eq!(fee_override.override_cost_basis, Some(980.0));
    }
}
