//! Core infrastructure module for Sanctum
//!
//! Contains shared infrastructure: database connection, error types, and validation.
//! Security logging is provided by the crate-root `security_log` module.

pub mod database;
pub mod error;
pub mod validation;

pub use database::Database;
pub use error::DbError;
pub use validation::{
    format_money_display, sanitize_string, validate_color, validate_date, validate_field_length,
    validate_uuid,
};

// Re-export security logging from crate root for convenience
pub use crate::security_log::{
    init_security_logger, log_auth_failure, log_rate_limit, log_security_event, SecurityEvent,
};
