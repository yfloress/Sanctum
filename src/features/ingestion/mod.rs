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

//! Data ingestion feature module
//!
//! Handles importing transactions and habit logs from external files.
//!
//! Supported formats:
//! - JSON (Sanctum Web export)
//! - CSV (Excel/Google Sheets)
//! - Plain text (semicolon-separated)
//!
//! Split into focused submodules:
//! - `types` - Data structures for import operations
//! - `validation` - Input validation helpers
//! - `parsers` - Format-specific parsers
//! - `repository` - Database lookups for entity resolution
//! - `service` - Main orchestration service

pub mod parsers;
pub mod repository;
pub mod service;
pub mod types;
pub mod validation;

pub use repository::IngestionRepository;
pub use service::{IngestionError, IngestionService};
pub use types::{ImportFormat, ImportSummary, RowError};
pub use validation::MAX_FILE_SIZE;
