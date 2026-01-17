//! Data ingestion controller methods
//!
//! Handles importing transactions and habit logs from various file formats.

use super::{AppController, ControllerError};
use crate::features::ingestion::{IngestionError, ImportSummary, MAX_FILE_SIZE};

impl From<IngestionError> for ControllerError {
    fn from(err: IngestionError) -> Self {
        match err {
            IngestionError::Database(e) => ControllerError::Database(e),
            IngestionError::Parse(msg) => ControllerError::Validation(msg),
            IngestionError::Validation(msg) => ControllerError::Validation(msg),
            IngestionError::NoVaultOpen => ControllerError::NoVaultOpen,
            IngestionError::SessionExpired => ControllerError::SessionExpired,
            IngestionError::UnsupportedFormat(msg) => ControllerError::Validation(msg),
            IngestionError::FileTooLarge(msg) => ControllerError::Validation(msg),
        }
    }
}

impl AppController {
    // ==================== Data Import Operations ====================

    /// Imports data from file content
    ///
    /// Automatically detects format based on content and filename.
    /// Supported formats: JSON v1, CSV, Plain Text
    ///
    /// Returns an ImportSummary with counts and error details.
    pub fn import_data(
        &self,
        content: String,
        filename: String,
    ) -> Result<ImportSummary, ControllerError> {
        self.ingestion_service
            .import_from_content(&content, &filename)
            .map_err(ControllerError::from)
    }

    /// Returns the maximum allowed file size in bytes
    pub fn max_import_file_size(&self) -> usize {
        MAX_FILE_SIZE
    }
}
