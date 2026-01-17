//! Data ingestion feature module
//!
//! Handles importing transactions and habit logs from external files.
//!
//! Supported formats:
//! - JSON v1 (Sanctum Web export)
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
