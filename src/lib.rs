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
