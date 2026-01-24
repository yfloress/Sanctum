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
