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

//! Sanctum - Personal Finance Manager
//!
//! Core library modules for database, models, and business logic.
//!
//! # Architecture
//!
//! The codebase is organized into feature-based modules:
//!
//! - `core/` - Shared infrastructure (database, errors, security)
//! - `features/` - Domain modules (finance, crypto, habits)
//! - `services/` - Legacy services (being migrated to features/)
//! - `ui/` - UI layer (Slint helpers, data, callbacks)

// Generate Slint UI types - makes them available crate-wide
slint::include_modules!();

// New modular architecture
pub mod core;
pub mod features;
pub mod ui;

// Legacy modules (kept for backwards compatibility during migration)
pub mod controller;
pub mod db;
pub mod models;
pub mod security_log;
pub mod services;

// Re-exports for backwards compatibility
pub use core::{Database, DbError, SecurityEvent, init_security_logger, log_security_event};
