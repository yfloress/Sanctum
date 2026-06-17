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

//! Finance validation helpers
//!
//! Re-exports shared validation from core and adds finance-specific constants.

// Re-export shared validation functions
pub use crate::core::validation::{
    format_money_display, sanitize_string, validate_color, validate_date, validate_field_length,
    validate_uuid,
};
use uuid::Uuid;

// Finance-specific validation constants
pub const MAX_CATEGORY_LENGTH: usize = 64;
pub const MAX_DESCRIPTION_LENGTH: usize = 512;
pub const MAX_ACCOUNT_NAME_LENGTH: usize = 64;
pub const MAX_CURRENCY_LENGTH: usize = 8;
pub const MAX_ICON_LENGTH: usize = 256;
pub const EXCHANGE_RATE_TTL_SECS: i64 = 6 * 60 * 60;

pub fn validate_category_id(id: &str) -> Result<String, String> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err("ID cannot be empty".to_string());
    }

    if Uuid::parse_str(trimmed).is_ok() {
        return Ok(trimmed.to_string());
    }

    if let Some(rest) = trimmed
        .strip_prefix("exp_")
        .or_else(|| trimmed.strip_prefix("inc_"))
    {
        validate_uuid(rest)?;
        return Ok(trimmed.to_string());
    }

    Err("Invalid ID format".to_string())
}
