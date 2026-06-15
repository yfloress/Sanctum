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

//! Ingestion domain Tauri commands.

use sanctum::controller::AppController;
use sanctum::ui::dto::ingestion::{ExchangeDetectionResult, ImportErrorDto, ImportResultsResponse};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn preview_import(
    controller: State<'_, Arc<AppController>>,
    content: String,
    filename: String,
) -> Result<ImportResultsResponse, String> {
    let summary = controller
        .preview_data(&content, &filename)
        .map_err(|e| e.to_string())?;
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn import_data(
    controller: State<'_, Arc<AppController>>,
    content: String,
    filename: String,
) -> Result<ImportResultsResponse, String> {
    let summary = controller
        .import_data(content, filename)
        .map_err(|e| e.to_string())?;
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn max_import_file_size(controller: State<'_, Arc<AppController>>) -> usize {
    controller.max_import_file_size()
}

#[tauri::command]
pub fn detect_exchange_source(
    controller: State<'_, Arc<AppController>>,
    content: String,
) -> Option<ExchangeDetectionResult> {
    controller
        .detect_exchange_source(&content)
        .map(|(id, label, wallet)| ExchangeDetectionResult {
            exchange_id: id,
            exchange: label,
            suggested_wallet: wallet,
            file_count: 1,
            total_records: content.lines().count().saturating_sub(1),
        })
}

#[tauri::command]
pub fn preview_exchange_csv(
    controller: State<'_, Arc<AppController>>,
    content: String,
    wallet_name: String,
) -> Result<ImportResultsResponse, String> {
    let summary = controller
        .preview_exchange_csv(&content, &wallet_name)
        .map_err(|e| e.to_string())?;
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn import_exchange_csv(
    controller: State<'_, Arc<AppController>>,
    content: String,
    wallet_name: String,
) -> Result<ImportResultsResponse, String> {
    let summary = controller
        .import_exchange_csv(content, wallet_name)
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
