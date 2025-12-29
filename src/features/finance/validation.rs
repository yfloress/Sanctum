//! Finance validation helpers
//!
//! Re-exports shared validation from core and adds finance-specific constants.

// Re-export shared validation functions
pub use crate::core::validation::{
    format_money_display, sanitize_string, validate_color, validate_date, validate_field_length,
    validate_uuid,
};

// Finance-specific validation constants
pub const MAX_CATEGORY_LENGTH: usize = 64;
pub const MAX_DESCRIPTION_LENGTH: usize = 512;
pub const MAX_ACCOUNT_NAME_LENGTH: usize = 64;
pub const MAX_CURRENCY_LENGTH: usize = 8;
pub const MAX_ICON_LENGTH: usize = 32;
pub const EXCHANGE_RATE_TTL_SECS: i64 = 6 * 60 * 60;
