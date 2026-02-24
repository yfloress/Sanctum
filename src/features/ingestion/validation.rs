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
use crate::features::crypto::tax::types::normalize_subtype;
use chrono::NaiveDate;

/// Valid type categories for crypto transactions.
const VALID_CRYPTO_TYPES: [&str; 4] = ["trade", "income", "expense", "transfer"];

/// Maximum file size (10MB)
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
const MAX_PRICE_PER_COIN: f64 = 1_000_000_000_000.0;

/// Validates a date string in YYYY-MM-DD format
pub fn validate_date(date: &str) -> Result<String, String> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return Err("Date is required".to_string());
    }

    // Accept YYYY-MM-DD
    if NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").is_ok() {
        return Ok(trimmed.to_string());
    }

    // Accept YYYY-MM-DD HH:MM:SS (full datetime from exchange imports)
    if trimmed.len() >= 10 && NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d").is_ok() {
        return Ok(trimmed.to_string());
    }

    Err(format!(
        "Invalid date format: '{}'. Expected YYYY-MM-DD",
        trimmed
    ))
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

/// Validates a crypto transaction type (trade/income/expense/transfer).
pub fn validate_crypto_type(tx_type: &str) -> Result<String, String> {
    let normalized = tx_type.trim().to_lowercase();
    if VALID_CRYPTO_TYPES.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(format!(
            "Invalid crypto type: '{}'. Expected: trade, income, expense, or transfer",
            tx_type
        ))
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
pub fn validate_import_transaction(
    tx: &ImportTransaction,
    line_number: usize,
) -> Result<(), RowError> {
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
/// Validates the mechanical type derived from type + subtype.
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

/// Validates crypto amount (can be very small, like 0.00000001).
///
/// Negative amounts are accepted and normalised to their absolute value.
/// Exchange CSV exports covering a partial window of an account's history
/// can legitimately contain negative figures (e.g. sells that happened
/// before the export window's buys).  Rejecting them would block valid
/// imports, so we allow them and use the magnitude.
pub fn validate_crypto_amount(amount: f64) -> Result<f64, String> {
    if !amount.is_finite() {
        return Err("Amount must be a finite number".to_string());
    }
    let abs = amount.abs();
    if abs <= 0.0 {
        return Err("Amount must be non-zero".to_string());
    }
    if abs > 1_000_000_000.0 {
        return Err("Amount exceeds maximum allowed".to_string());
    }
    Ok(abs)
}

/// Validates optional price per coin.
///
/// Negative prices are accepted and normalised to their absolute value
/// so that exchange exports with sign-encoded direction are not rejected.
pub fn validate_price_per_coin(price: Option<f64>) -> Result<Option<f64>, String> {
    match price {
        Some(p) if !p.is_finite() => Err("Price per coin must be a finite number".to_string()),
        Some(p) if p.abs() > MAX_PRICE_PER_COIN => {
            Err("Price per coin exceeds maximum".to_string())
        }
        Some(p) => Ok(Some(p.abs())),
        None => Ok(None),
    }
}

/// Validates optional fee.
///
/// Negative fees are accepted and normalised to their absolute value
/// (they may represent rebates or sign-encoded values in exchange exports).
pub fn validate_fee(fee: Option<f64>) -> Result<Option<f64>, String> {
    match fee {
        Some(f) if !f.is_finite() => Err("Fee must be a finite number".to_string()),
        Some(f) if f.abs() > 1_000_000.0 => Err("Fee exceeds maximum".to_string()),
        Some(f) => Ok(Some(f.abs())),
        None => Ok(None),
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

    // Validate type (trade/income/expense/transfer)
    validate_crypto_type(&tx.transaction_type).map_err(|e| make_error("type", e))?;

    // Validate the derived mechanical type
    let mech = tx.mechanical_type();
    validate_crypto_tx_type(mech).map_err(|e| make_error("type", e))?;

    validate_crypto_amount(tx.amount).map_err(|e| make_error("amount", e))?;
    validate_price_per_coin(tx.price_per_coin).map_err(|e| make_error("price_per_coin", e))?;
    validate_fee(tx.fee).map_err(|e| make_error("fee", e))?;

    // Validate subtype against the transaction type category
    if let Some(ref sub) = tx.subtype
        && normalize_subtype(&tx.transaction_type, sub).is_none()
    {
        return Err(make_error(
            "subtype",
            format!(
                "Invalid subtype '{}' for type '{}'. Check SUBTYPES_* catalogs.",
                sub, tx.transaction_type
            ),
        ));
    }
    if let Some(value) = tx.override_proceeds
        && value < 0.0
    {
        return Err(make_error(
            "override_proceeds",
            "Override proceeds cannot be negative".to_string(),
        ));
    }
    if let Some(value) = tx.override_cost_basis
        && value < 0.0
    {
        return Err(make_error(
            "override_cost_basis",
            "Override cost basis cannot be negative".to_string(),
        ));
    }

    // Validate fee_coin_symbol/fee_amount pairing for all types
    let tx_type = mech;
    if tx_type != "swap" {
        let fee_coin_symbol = tx
            .fee_coin_symbol
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        if fee_coin_symbol.is_some() && tx.fee_amount.is_none() {
            // fee_coin without fee_amount: ignore (will be cleared during processing)
        }
        if fee_coin_symbol.is_none() && tx.fee_amount.is_some() {
            return Err(make_error(
                "fee_coin_symbol",
                "Fee coin symbol is required when fee amount is provided".to_string(),
            ));
        }
        if let (Some(symbol), Some(amount)) = (fee_coin_symbol, tx.fee_amount) {
            validate_crypto_symbol(symbol).map_err(|e| make_error("fee_coin_symbol", e))?;
            validate_crypto_amount(amount).map_err(|e| make_error("fee_amount", e))?;
        }
    }

    if tx_type == "swap" {
        let to_symbol = tx
            .swap_to_symbol
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                make_error(
                    "swap_to_symbol",
                    "Swap target symbol is required".to_string(),
                )
            })?;
        let to_amount = tx.swap_to_amount.ok_or_else(|| {
            make_error(
                "swap_to_amount",
                "Swap target amount is required".to_string(),
            )
        })?;

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
            subtype: None,
            price_per_coin: None,
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
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
        // Full datetime from exchange imports
        assert!(validate_date("2024-01-15 10:30:45").is_ok());
        assert_eq!(
            validate_date("2024-01-15 10:30:45").unwrap(),
            "2024-01-15 10:30:45"
        );
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
    fn test_validate_crypto_amount_allows_negatives() {
        // Positive values pass through as-is
        assert_eq!(validate_crypto_amount(0.5).unwrap(), 0.5);
        assert_eq!(validate_crypto_amount(0.00000001).unwrap(), 0.00000001);
        // Negative values are normalised to their absolute value
        assert_eq!(validate_crypto_amount(-0.5).unwrap(), 0.5);
        assert_eq!(validate_crypto_amount(-1.23).unwrap(), 1.23);
        // Zero and non-finite are still rejected
        assert!(validate_crypto_amount(0.0).is_err());
        assert!(validate_crypto_amount(f64::NAN).is_err());
        assert!(validate_crypto_amount(f64::INFINITY).is_err());
        assert!(validate_crypto_amount(f64::NEG_INFINITY).is_err());
        // Max boundary
        assert!(validate_crypto_amount(2_000_000_000.0).is_err());
        assert!(validate_crypto_amount(-2_000_000_000.0).is_err());
    }

    #[test]
    fn test_validate_price_per_coin_allows_negatives() {
        assert_eq!(validate_price_per_coin(Some(100.0)).unwrap(), Some(100.0));
        // Negative normalised to absolute value
        assert_eq!(validate_price_per_coin(Some(-50.0)).unwrap(), Some(50.0));
        assert_eq!(
            validate_price_per_coin(Some(95_000_000.0)).unwrap(),
            Some(95_000_000.0)
        );
        assert_eq!(validate_price_per_coin(None).unwrap(), None);
        assert!(validate_price_per_coin(Some(f64::NAN)).is_err());
        assert!(validate_price_per_coin(Some(2_000_000_000_000.0)).is_err());
    }

    #[test]
    fn test_validate_fee_allows_negatives() {
        assert_eq!(validate_fee(Some(0.5)).unwrap(), Some(0.5));
        // Negative normalised to absolute value (rebate)
        assert_eq!(validate_fee(Some(-0.1)).unwrap(), Some(0.1));
        assert_eq!(validate_fee(None).unwrap(), None);
        assert!(validate_fee(Some(f64::NAN)).is_err());
        assert!(validate_fee(Some(2_000_000.0)).is_err());
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
    fn test_validate_crypto_type() {
        assert_eq!(validate_crypto_type("trade").unwrap(), "trade");
        assert_eq!(validate_crypto_type("INCOME").unwrap(), "income");
        assert!(validate_crypto_type("buy").is_err());
        assert!(validate_crypto_type("banana").is_err());
    }

    #[test]
    fn test_validate_crypto_swap_requires_targets() {
        let mut tx = base_crypto_tx("trade");
        tx.subtype = Some("swap".to_string());
        assert!(validate_import_crypto_transaction(&tx, 1).is_err());

        tx.swap_to_symbol = Some("ETH".to_string());
        tx.swap_to_amount = Some(1.25);
        assert!(validate_import_crypto_transaction(&tx, 1).is_ok());
    }

    #[test]
    fn test_validate_crypto_rejects_mismatched_subtype_for_type() {
        let mut tx = base_crypto_tx("income");
        tx.subtype = Some("sell".to_string());
        let err = validate_import_crypto_transaction(&tx, 1).expect_err("must fail");
        assert_eq!(err.field.as_deref(), Some("subtype"));
    }

    #[test]
    fn test_validate_crypto_transfer_withdrawal_maps_to_mechanical_outflow() {
        let mut tx = base_crypto_tx("transfer");
        tx.subtype = Some("withdrawal".to_string());
        assert_eq!(tx.mechanical_type(), "transfer_out");
        assert!(validate_import_crypto_transaction(&tx, 1).is_ok());
    }

    #[test]
    fn test_validate_crypto_fee_amount_requires_fee_coin_symbol() {
        let mut tx = base_crypto_tx("trade");
        tx.subtype = Some("buy".to_string());
        tx.fee_amount = Some(0.002);
        let err = validate_import_crypto_transaction(&tx, 1).expect_err("must fail");
        assert_eq!(err.field.as_deref(), Some("fee_coin_symbol"));
    }

    #[test]
    fn test_validate_crypto_swap_rejects_same_asset_target() {
        let mut tx = base_crypto_tx("trade");
        tx.subtype = Some("swap".to_string());
        tx.swap_to_symbol = Some("BTC".to_string());
        tx.swap_to_amount = Some(0.1);

        let err = validate_import_crypto_transaction(&tx, 1).expect_err("must fail");
        assert_eq!(err.field.as_deref(), Some("swap_to_symbol"));
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
