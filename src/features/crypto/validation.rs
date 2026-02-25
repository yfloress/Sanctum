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

//! Validation helpers for crypto operations
//!
//! Contains input validation, sanitization, and balance checking functions.
//! Generic validation re-exported from core, crypto-specific validation defined here.

use crate::db::Database;

use super::api::validate_coin_id;
use super::service::CryptoError;
use super::tax::types::{TaxTxType, normalize_subtype};

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

/// Validates and normalises a subtype value against its transaction type category.
///
/// `tx_type` is the transaction type category (trade/income/expense/transfer).
/// `value` is the raw subtype string (e.g. "buy", "airdrop", "deposit").
pub fn validate_subtype(
    tx_type: Option<&str>,
    value: Option<String>,
) -> Result<Option<String>, CryptoError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let resolved = tx_type.and_then(TaxTxType::parse).ok_or_else(|| {
        CryptoError::Validation(
            "Subtype requires a valid type (trade, income, expense, or transfer)".to_string(),
        )
    })?;
    let normalized = normalize_subtype(resolved.as_str(), trimmed).ok_or_else(|| {
        CryptoError::Validation(format!(
            "Invalid subtype '{}' for type '{}'. Check allowed subtypes.",
            trimmed,
            resolved.as_str()
        ))
    })?;
    Ok(Some(normalized))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_symbol_normalizes_uppercase() {
        assert_eq!(validate_symbol(" btc ").expect("valid symbol"), "BTC");
    }

    #[test]
    fn validate_symbol_rejects_invalid_values() {
        assert!(validate_symbol("").is_err());
        assert!(validate_symbol("a$").is_err());
        assert!(validate_symbol(&"a".repeat(MAX_SYMBOL_LENGTH + 1)).is_err());
    }

    #[test]
    fn validate_positive_amount_rejects_non_finite_or_non_positive() {
        assert!(validate_positive_amount(0.0, "Amount").is_err());
        assert!(validate_positive_amount(-1.0, "Amount").is_err());
        assert!(validate_positive_amount(f64::NAN, "Amount").is_err());
        assert!(validate_positive_amount(f64::INFINITY, "Amount").is_err());
        assert_eq!(
            validate_positive_amount(1.23, "Amount").expect("positive"),
            1.23
        );
    }

    #[test]
    fn validate_non_negative_handles_optional_values() {
        assert_eq!(
            validate_non_negative(None, "Fee").expect("none allowed"),
            None
        );
        assert_eq!(
            validate_non_negative(Some(0.0), "Fee").expect("zero allowed"),
            Some(0.0)
        );
        assert!(validate_non_negative(Some(-0.1), "Fee").is_err());
        assert!(validate_non_negative(Some(f64::NAN), "Fee").is_err());
    }

    #[test]
    fn validate_subtype_accepts_matching_type_catalog() {
        assert_eq!(
            validate_subtype(Some("trade"), Some("  SWAP  ".to_string()))
                .expect("valid")
                .as_deref(),
            Some("swap")
        );
        assert_eq!(
            validate_subtype(Some("income"), Some("airdrop".to_string()))
                .expect("valid")
                .as_deref(),
            Some("airdrop")
        );
        assert_eq!(
            validate_subtype(Some("transfer"), Some("withdrawal".to_string()))
                .expect("valid")
                .as_deref(),
            Some("withdrawal")
        );
    }

    #[test]
    fn validate_subtype_rejects_invalid_or_missing_type() {
        assert!(validate_subtype(Some("income"), Some("sell".to_string())).is_err());
        assert!(validate_subtype(Some("banana"), Some("buy".to_string())).is_err());
        assert_eq!(
            validate_subtype(Some("trade"), Some("   ".to_string())).expect("empty to none"),
            None
        );
        assert_eq!(
            validate_subtype(Some("trade"), None).expect("none to none"),
            None
        );
    }

    #[test]
    fn normalize_fee_coin_requires_both_fields_or_none() {
        let normalized =
            normalize_fee_coin(Some(" bitcoin ".to_string()), Some(0.001)).expect("valid fee pair");
        assert_eq!(normalized.0.as_deref(), Some("bitcoin"));
        assert_eq!(normalized.1, Some(0.001));

        assert_eq!(
            normalize_fee_coin(Some("bitcoin".to_string()), None).expect("coin-only clears"),
            (None, None)
        );
        assert!(normalize_fee_coin(None, Some(0.001)).is_err());
        assert!(normalize_fee_coin(Some("invalid symbol".to_string()), Some(0.001)).is_err());
        assert!(normalize_fee_coin(Some("bitcoin".to_string()), Some(0.0)).is_err());
    }
}
