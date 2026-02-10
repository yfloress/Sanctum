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
    /// Supported formats: JSON, CSV, Plain Text
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

    /// Previews import results without writing to the database
    pub fn preview_data(
        &self,
        content: &str,
        filename: &str,
    ) -> Result<ImportSummary, ControllerError> {
        self.ingestion_service
            .preview_from_content(content, filename)
            .map_err(ControllerError::from)
    }

    /// Returns the maximum allowed file size in bytes
    pub fn max_import_file_size(&self) -> usize {
        MAX_FILE_SIZE
    }
}
