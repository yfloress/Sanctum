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
use crate::features::ingestion::{ImportSummary, IngestionError, MAX_FILE_SIZE};

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

    // ==================== Exchange CSV Import ====================

    /// Imports an exchange CSV with auto-detection.
    ///
    /// The exchange format is identified from the CSV headers.
    /// `wallet_name` is the target wallet for all imported transactions.
    pub fn import_exchange_csv(
        &self,
        content: String,
        wallet_name: String,
    ) -> Result<ImportSummary, ControllerError> {
        self.ingestion_service
            .import_exchange_csv_auto(&content, &wallet_name)
            .map_err(ControllerError::from)
    }

    /// Previews an exchange CSV import without writing to the database.
    pub fn preview_exchange_csv(
        &self,
        content: &str,
        wallet_name: &str,
    ) -> Result<ImportSummary, ControllerError> {
        self.ingestion_service
            .preview_exchange_csv_auto(content, wallet_name)
            .map_err(ControllerError::from)
    }

    /// Detects the exchange source from CSV content.
    ///
    /// Returns a tuple of `(exchange_id, exchange_label, default_wallet_name)`
    /// or `None` if the format is not recognized.
    pub fn detect_exchange_source(&self, content: &str) -> Option<(String, String, String)> {
        use crate::features::ingestion::parsers::detect_exchange_source;
        detect_exchange_source(content).map(|source| {
            (
                source.id().to_string(),
                source.label().to_string(),
                source.default_wallet_name().to_string(),
            )
        })
    }
}
