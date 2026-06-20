// Sanctum — a privacy-first personal finance and crypto vault.
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

//! Ingestion domain Tauri commands.

use sanctum::features::ingestion::parsers::detect_exchange_source as parse_exchange_source;
use sanctum::features::ingestion::{IngestionService, MAX_FILE_SIZE};
use sanctum::ui::dto::ingestion::{ExchangeDetectionResult, ImportErrorDto, ImportResultsResponse};
use tauri::State;

#[tauri::command]
pub fn preview_import(
    ingestion: State<'_, IngestionService>,
    content: String,
    filename: String,
) -> Result<ImportResultsResponse, String> {
    let summary = ingestion
        .preview_from_content(&content, &filename)
        .map_err(|e| e.to_string())?;
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn import_data(
    ingestion: State<'_, IngestionService>,
    content: String,
    filename: String,
) -> Result<ImportResultsResponse, String> {
    let summary = ingestion
        .import_from_content(&content, &filename)
        .map_err(|e| e.to_string())?;
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn max_import_file_size() -> usize {
    MAX_FILE_SIZE
}

#[tauri::command]
pub fn detect_exchange_source(content: String) -> Option<ExchangeDetectionResult> {
    detect_exchange_source_inner(&content)
}

fn detect_exchange_source_inner(content: &str) -> Option<ExchangeDetectionResult> {
    parse_exchange_source(content).map(|source| ExchangeDetectionResult {
        exchange_id: source.id().to_string(),
        exchange: source.label().to_string(),
        suggested_wallet: source.default_wallet_name().to_string(),
        file_count: 1,
        total_records: content.lines().count().saturating_sub(1),
    })
}

#[tauri::command]
pub fn preview_exchange_csv(
    ingestion: State<'_, IngestionService>,
    content: String,
    wallet_name: String,
) -> Result<ImportResultsResponse, String> {
    let summary = ingestion
        .preview_exchange_csv_auto(&content, &wallet_name)
        .map_err(|e| e.to_string())?;
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn import_exchange_csv(
    ingestion: State<'_, IngestionService>,
    content: String,
    wallet_name: String,
) -> Result<ImportResultsResponse, String> {
    let summary = ingestion
        .import_exchange_csv_auto(&content, &wallet_name)
        .map_err(|e| e.to_string())?;
    Ok(map_import_summary(summary))
}

fn map_import_summary(
    summary: sanctum::features::ingestion::ImportSummary,
) -> ImportResultsResponse {
    ImportResultsResponse {
        total_processed: summary.total_processed,
        inserted: summary.inserted,
        skipped: summary.skipped,
        errors: summary
            .error_details
            .into_iter()
            .map(|e| ImportErrorDto {
                line: Some(e.line_number),
                message: e.message,
            })
            .collect(),
    }
}
