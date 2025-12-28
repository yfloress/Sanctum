//! Core infrastructure module for Sanctum
//!
//! Contains shared infrastructure: database connection, error types, and security.

pub mod database;
pub mod error;
pub mod security;

pub use database::Database;
pub use error::DbError;
pub use security::{SecurityEvent, init_security_logger, log_auth_failure, log_rate_limit, log_security_event};
