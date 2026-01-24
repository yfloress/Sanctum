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

//! Validation helpers for ingestion data
//!
//! Provides field-level validation for imported transactions and habit logs.

use super::types::{ImportCryptoTransaction, ImportHabitLog, ImportTransaction, RowError};
use chrono::NaiveDate;

/// Maximum file size (10MB)
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Validates a date string in YYYY-MM-DD format
pub fn validate_date(date: &str) -> Result<String, String> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return Err("Date is required".to_string());
    }

    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map(|_| trimmed.to_string())
        .map_err(|_| format!("Invalid date format: '{}'. Expected YYYY-MM-DD", trimmed))
}

/// Validates transaction type
pub fn validate_transaction_type(tx_type: &str) -> Result<String, String> {
    let normalized = tx_type.trim().to_lowercase();
    match normalized.as_str() {
        "income" | "expense" | "transfer" => Ok(normalized),
        _ => Err(format!(
            "Invalid transaction type: '{}'. Expected: income, expense, or transfer",
            tx_type
        )),
    }
}

/// Validates and converts amount to cents
pub fn validate_amount(amount: f64) -> Result<i64, String> {
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    if amount > 999_999_999.99 {
        return Err("Amount exceeds maximum allowed".to_string());
    }

    // Convert to cents (round to avoid floating point issues)
    let cents = (amount * 100.0).round() as i64;
    if cents <= 0 {
        return Err("Amount must be positive".to_string());
    }

    Ok(cents)
}

/// Validates currency code (3-letter ISO 4217)
pub fn validate_currency(currency: &str) -> Result<String, String> {
    let normalized = currency.trim().to_uppercase();
    if normalized.is_empty() {
        return Err("Currency is required".to_string());
    }
    if normalized.len() != 3 {
        return Err(format!(
            "Invalid currency code: '{}'. Expected 3-letter ISO code",
            currency
        ));
    }
    // Basic check: all alphabetic
    if !normalized.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(format!(
            "Invalid currency code: '{}'. Must contain only letters",
            currency
        ));
    }
    Ok(normalized)
}

/// Validates account name
pub fn validate_account_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Account name is required".to_string());
    }
    if trimmed.len() > 64 {
        return Err("Account name too long (max 64 characters)".to_string());
    }
    Ok(trimmed.to_string())
}

/// Validates habit name
pub fn validate_habit_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Habit name is required".to_string());
    }
    if trimmed.len() > 128 {
        return Err("Habit name too long (max 128 characters)".to_string());
    }
    Ok(trimmed.to_string())
}

/// Validates category name
pub fn validate_category_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Category is required".to_string());
    }
    if trimmed.len() > 64 {
        return Err("Category name too long (max 64 characters)".to_string());
    }
    Ok(trimmed.to_string())
}

/// Parses boolean from various string representations
pub fn parse_bool(value: &str) -> Result<bool, String> {
    let normalized = value.trim().to_lowercase();
    match normalized.as_str() {
        "true" | "1" | "yes" | "y" | "si" | "sí" => Ok(true),
        "false" | "0" | "no" | "n" | "" => Ok(false),
        _ => Err(format!(
            "Invalid boolean value: '{}'. Expected: true/false",
            value
        )),
    }
}

/// Validates file size
pub fn validate_file_size(size: usize) -> Result<(), String> {
    if size == 0 {
        return Err("File is empty".to_string());
    }
    if size > MAX_FILE_SIZE {
        return Err(format!(
            "File size ({:.2} MB) exceeds maximum allowed ({} MB)",
            size as f64 / 1024.0 / 1024.0,
            MAX_FILE_SIZE / 1024 / 1024
        ));
    }
    Ok(())
}

/// Validates an import transaction
pub fn validate_import_transaction(tx: &ImportTransaction, line_number: usize) -> Result<(), RowError> {
    let make_error = |field: &str, msg: String| RowError::new(line_number, Some(field), msg);

    validate_date(&tx.date).map_err(|e| make_error("date", e))?;
    validate_account_name(&tx.account).map_err(|e| make_error("account", e))?;
    validate_transaction_type(&tx.transaction_type).map_err(|e| make_error("type", e))?;
    validate_amount(tx.amount).map_err(|e| make_error("amount", e))?;
    validate_currency(&tx.currency).map_err(|e| make_error("currency", e))?;

    // Category validation (not required for transfers)
    let tx_type = tx.transaction_type.trim().to_lowercase();
    if tx_type != "transfer" {
        validate_category_name(&tx.category).map_err(|e| make_error("category", e))?;
    }

    // Transfer validation
    if tx_type == "transfer" {
        let has_dest = tx
            .transfer_to_account
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !has_dest {
            return Err(make_error(
                "transfer_to_account",
                "Transfer transactions require a destination account".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validates an import habit log
pub fn validate_import_habit_log(log: &ImportHabitLog, line_number: usize) -> Result<(), RowError> {
    let make_error = |field: &str, msg: String| RowError::new(line_number, Some(field), msg);

    validate_habit_name(&log.habit).map_err(|e| make_error("habit", e))?;
    validate_date(&log.date).map_err(|e| make_error("date", e))?;

    Ok(())
}

// ==================== Crypto Validation ====================

/// Validates crypto transaction type
pub fn validate_crypto_tx_type(tx_type: &str) -> Result<String, String> {
    let normalized = tx_type.trim().to_lowercase();
    match normalized.as_str() {
        "buy" | "sell" | "transfer_in" | "transfer_out" | "swap" => Ok(normalized),
        _ => Err(format!(
            "Invalid crypto transaction type: '{}'. Expected: buy, sell, transfer_in, transfer_out, or swap",
            tx_type
        )),
    }
}

/// Validates crypto symbol (1-10 uppercase letters)
pub fn validate_crypto_symbol(symbol: &str) -> Result<String, String> {
    let normalized = symbol.trim().to_uppercase();
    if normalized.is_empty() {
        return Err("Crypto symbol is required".to_string());
    }
    if normalized.len() > 10 {
        return Err("Crypto symbol too long (max 10 characters)".to_string());
    }
    if !normalized.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!(
            "Invalid crypto symbol: '{}'. Must contain only letters and numbers",
            symbol
        ));
    }
    Ok(normalized)
}

/// Validates wallet name
pub fn validate_wallet_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Wallet name is required".to_string());
    }
    if trimmed.len() > 64 {
        return Err("Wallet name too long (max 64 characters)".to_string());
    }
    Ok(trimmed.to_string())
}

/// Validates crypto amount (can be very small, like 0.00000001)
pub fn validate_crypto_amount(amount: f64) -> Result<f64, String> {
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    if amount > 1_000_000_000.0 {
        return Err("Amount exceeds maximum allowed".to_string());
    }
    Ok(amount)
}

/// Validates optional price per coin
pub fn validate_price_per_coin(price: Option<f64>) -> Result<Option<f64>, String> {
    match price {
        Some(p) if p < 0.0 => Err("Price per coin cannot be negative".to_string()),
        Some(p) if p > 10_000_000.0 => Err("Price per coin exceeds maximum".to_string()),
        other => Ok(other),
    }
}

/// Validates optional fee
pub fn validate_fee(fee: Option<f64>) -> Result<Option<f64>, String> {
    match fee {
        Some(f) if f < 0.0 => Err("Fee cannot be negative".to_string()),
        Some(f) if f > 1_000_000.0 => Err("Fee exceeds maximum".to_string()),
        other => Ok(other),
    }
}

/// Validates an import crypto transaction
pub fn validate_import_crypto_transaction(
    tx: &ImportCryptoTransaction,
    line_number: usize,
) -> Result<(), RowError> {
    let make_error = |field: &str, msg: String| RowError::new(line_number, Some(field), msg);

    validate_date(&tx.date).map_err(|e| make_error("date", e))?;
    validate_wallet_name(&tx.wallet).map_err(|e| make_error("wallet", e))?;
    validate_crypto_symbol(&tx.symbol).map_err(|e| make_error("symbol", e))?;
    validate_crypto_tx_type(&tx.transaction_type).map_err(|e| make_error("type", e))?;
    validate_crypto_amount(tx.amount).map_err(|e| make_error("amount", e))?;
    validate_price_per_coin(tx.price_per_coin).map_err(|e| make_error("price_per_coin", e))?;
    validate_fee(tx.fee).map_err(|e| make_error("fee", e))?;

    let tx_type = tx.transaction_type.trim().to_lowercase();
    if tx_type == "swap" {
        let to_symbol = tx
            .swap_to_symbol
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| make_error("swap_to_symbol", "Swap target symbol is required".to_string()))?;
        let to_amount = tx
            .swap_to_amount
            .ok_or_else(|| make_error("swap_to_amount", "Swap target amount is required".to_string()))?;

        validate_crypto_symbol(to_symbol).map_err(|e| make_error("swap_to_symbol", e))?;
        validate_crypto_amount(to_amount).map_err(|e| make_error("swap_to_amount", e))?;

        let from_symbol = tx.symbol.trim().to_uppercase();
        let to_symbol_norm = to_symbol.trim().to_uppercase();
        if from_symbol == to_symbol_norm {
            return Err(make_error(
                "swap_to_symbol",
                "Swap requires two different assets".to_string(),
            ));
        }

        let fee_coin_symbol = tx
            .fee_coin_symbol
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        if fee_coin_symbol.is_some() ^ tx.fee_amount.is_some() {
            return Err(make_error(
                "fee_coin_symbol",
                "Fee coin symbol and fee amount must be provided together".to_string(),
            ));
        }
        if let (Some(symbol), Some(amount)) = (fee_coin_symbol, tx.fee_amount) {
            validate_crypto_symbol(symbol).map_err(|e| make_error("fee_coin_symbol", e))?;
            validate_crypto_amount(amount).map_err(|e| make_error("fee_amount", e))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_crypto_tx(tx_type: &str) -> ImportCryptoTransaction {
        ImportCryptoTransaction {
            date: "2024-01-10".to_string(),
            wallet: "Ledger".to_string(),
            symbol: "BTC".to_string(),
            transaction_type: tx_type.to_string(),
            amount: 0.5,
            price_per_coin: None,
            fee: None,
            swap_to_symbol: None,
            swap_to_amount: None,
            fee_coin_symbol: None,
            fee_amount: None,
            notes: None,
        }
    }

    #[test]
    fn test_validate_date_valid() {
        assert!(validate_date("2024-01-15").is_ok());
        assert!(validate_date("  2024-12-31  ").is_ok());
    }

    #[test]
    fn test_validate_date_invalid() {
        assert!(validate_date("01-15-2024").is_err());
        assert!(validate_date("2024/01/15").is_err());
        assert!(validate_date("").is_err());
        assert!(validate_date("invalid").is_err());
    }

    #[test]
    fn test_validate_amount() {
        assert_eq!(validate_amount(45.50).unwrap(), 4550);
        assert_eq!(validate_amount(0.01).unwrap(), 1);
        assert_eq!(validate_amount(100.0).unwrap(), 10000);
        assert!(validate_amount(0.0).is_err());
        assert!(validate_amount(-10.0).is_err());
    }

    #[test]
    fn test_validate_transaction_type() {
        assert_eq!(validate_transaction_type("income").unwrap(), "income");
        assert_eq!(validate_transaction_type("EXPENSE").unwrap(), "expense");
        assert_eq!(validate_transaction_type("Transfer").unwrap(), "transfer");
        assert!(validate_transaction_type("invalid").is_err());
    }

    #[test]
    fn test_validate_currency() {
        assert_eq!(validate_currency("usd").unwrap(), "USD");
        assert_eq!(validate_currency("CLP").unwrap(), "CLP");
        assert!(validate_currency("US").is_err());
        assert!(validate_currency("USDD").is_err());
        assert!(validate_currency("").is_err());
    }

    #[test]
    fn test_validate_crypto_tx_type_includes_swap() {
        assert_eq!(validate_crypto_tx_type("swap").unwrap(), "swap");
        assert!(validate_crypto_tx_type("stake").is_err());
    }

    #[test]
    fn test_validate_crypto_swap_requires_targets() {
        let mut tx = base_crypto_tx("swap");
        assert!(validate_import_crypto_transaction(&tx, 1).is_err());

        tx.swap_to_symbol = Some("ETH".to_string());
        tx.swap_to_amount = Some(1.25);
        assert!(validate_import_crypto_transaction(&tx, 1).is_ok());
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("true").unwrap());
        assert!(parse_bool("1").unwrap());
        assert!(parse_bool("yes").unwrap());
        assert!(!parse_bool("false").unwrap());
        assert!(!parse_bool("0").unwrap());
        assert!(!parse_bool("").unwrap());
    }
}
