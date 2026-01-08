//! Validation helpers for crypto operations
//!
//! Contains input validation, sanitization, and balance checking functions.
//! Generic validation re-exported from core, crypto-specific validation defined here.

use crate::db::Database;

use super::api::validate_coin_id;
use super::service::CryptoError;

// Re-export sanitize_string from core (doesn't need error wrapping)
pub use crate::core::validation::sanitize_string;

// ==================== Constants ====================

pub const MAX_NOTES_LENGTH: usize = 1024;
pub const MAX_WALLET_NAME_LENGTH: usize = 128;
pub const MAX_SYMBOL_LENGTH: usize = 16;
pub const MAX_ICON_LENGTH: usize = 256;
pub const MAX_COIN_NAME_LENGTH: usize = 64;

// ==================== Wrappers for Core Validation ====================

/// Validates and trims a field to max length, returning CryptoError
pub fn validate_field_length(
    value: &str,
    max_length: usize,
    field_name: &str,
) -> Result<String, CryptoError> {
    crate::core::validation::validate_field_length(value, max_length, field_name)
        .map_err(CryptoError::Validation)
}

/// Validates UUID, returning CryptoError
pub fn validate_uuid(id: &str) -> Result<String, CryptoError> {
    crate::core::validation::validate_uuid(id).map_err(CryptoError::Validation)
}

/// Validates date, returning CryptoError
pub fn validate_date(date: &str) -> Result<String, CryptoError> {
    crate::core::validation::validate_date(date).map_err(CryptoError::Validation)
}

// ==================== Crypto-Specific Validation ====================

pub fn validate_coin_id_str(coin_id: &str) -> Result<String, CryptoError> {
    validate_coin_id(coin_id).map_err(CryptoError::Validation)
}

pub fn validate_symbol(symbol: &str) -> Result<String, CryptoError> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return Err(CryptoError::Validation(
            "Symbol cannot be empty".to_string(),
        ));
    }
    if trimmed.len() > MAX_SYMBOL_LENGTH {
        return Err(CryptoError::Validation(format!(
            "Symbol exceeds maximum length of {} characters",
            MAX_SYMBOL_LENGTH
        )));
    }
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(CryptoError::Validation(
            "Symbol must be alphanumeric".to_string(),
        ));
    }
    Ok(trimmed.to_uppercase())
}

pub fn validate_positive_amount(value: f64, field: &str) -> Result<f64, CryptoError> {
    if !value.is_finite() {
        return Err(CryptoError::Validation(format!(
            "{} must be a finite number",
            field
        )));
    }
    if value <= 0.0 {
        return Err(CryptoError::Validation(format!(
            "{} must be greater than zero",
            field
        )));
    }
    Ok(value)
}

pub fn validate_non_negative(value: Option<f64>, field: &str) -> Result<Option<f64>, CryptoError> {
    if let Some(v) = value {
        if !v.is_finite() {
            return Err(CryptoError::Validation(format!(
                "{} must be a finite number",
                field
            )));
        }
        if v < 0.0 {
            return Err(CryptoError::Validation(format!(
                "{} cannot be negative",
                field
            )));
        }
    }
    Ok(value)
}

pub fn normalize_fee_coin(
    fee_coin_id: Option<String>,
    fee_amount: Option<f64>,
) -> Result<(Option<String>, Option<f64>), CryptoError> {
    let fee_coin_id = fee_coin_id.and_then(|id| {
        let trimmed = id.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    match (fee_coin_id, fee_amount) {
        (None, None) => Ok((None, None)),
        (Some(id), Some(amount)) => {
            let id = validate_coin_id_str(&id)?;
            let amount = validate_positive_amount(amount, "Fee amount")?;
            Ok((Some(id), Some(amount)))
        }
        (None, Some(_)) => Err(CryptoError::Validation(
            "Fee coin is required when fee amount is provided".to_string(),
        )),
        (Some(_), None) => Ok((None, None)),
    }
}

pub fn validate_sufficient_balance(
    db: &Database,
    wallet_id: &str,
    coin_id: &str,
    symbol: &str,
    required_amount: f64,
    date: &str,
    exclude_tx_id: Option<&str>,
) -> Result<(), CryptoError> {
    let balance = db
        .get_wallet_coin_balance_at(wallet_id, coin_id, date, exclude_tx_id)
        .map_err(CryptoError::Database)?;

    if required_amount > balance {
        return Err(CryptoError::Validation(format!(
            "Insufficient funds. Available: {:.8} {}",
            balance, symbol
        )));
    }
    Ok(())
}

pub struct FeeBalanceContext<'a> {
    pub db: &'a Database,
    pub wallet_id: &'a str,
    pub main_coin_id: &'a str,
    pub main_symbol: &'a str,
    pub main_amount: f64,
    pub is_outflow: bool,
    pub date: &'a str,
    pub exclude_tx_id: Option<&'a str>,
}

pub fn validate_fee_balance(
    ctx: FeeBalanceContext<'_>,
    fee_coin_id: Option<&str>,
    fee_amount: Option<f64>,
) -> Result<(), CryptoError> {
    if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id, fee_amount) {
        if fee_coin == ctx.main_coin_id {
            if ctx.is_outflow {
                let total_required = ctx.main_amount + fee_amt;
                validate_sufficient_balance(
                    ctx.db,
                    ctx.wallet_id,
                    ctx.main_coin_id,
                    ctx.main_symbol,
                    total_required,
                    ctx.date,
                    ctx.exclude_tx_id,
                )?;
            } else {
                let existing = ctx
                    .db
                    .get_wallet_coin_balance_at(
                        ctx.wallet_id,
                        ctx.main_coin_id,
                        ctx.date,
                        ctx.exclude_tx_id,
                    )
                    .map_err(CryptoError::Database)?;
                if fee_amt > ctx.main_amount + existing {
                    return Err(CryptoError::Validation(
                        "Fee amount exceeds the available balance for this asset".to_string(),
                    ));
                }
            }
        } else {
            validate_sufficient_balance(
                ctx.db,
                ctx.wallet_id,
                fee_coin,
                fee_coin,
                fee_amt,
                ctx.date,
                ctx.exclude_tx_id,
            )?;
        }
    }
    Ok(())
}
