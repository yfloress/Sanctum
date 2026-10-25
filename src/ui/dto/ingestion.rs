// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
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

//! Ingestion domain DTOs.
//!
//! Covers: generic data import, exchange CSV import, preview, results.

use serde::{Deserialize, Serialize};

// ==================== Generic Import ====================

/// Preview summary before confirming an import.
#[derive(Debug, Clone, Serialize)]
pub struct ImportPreviewResponse {
    pub source: String,
    pub total_records: usize,
    pub to_add: usize,
    pub to_skip: usize,
    pub changes: Vec<ImportChangeDto>,
}

/// A single change in the import preview.
#[derive(Debug, Clone, Serialize)]
pub struct ImportChangeDto {
    pub action: String,
    pub description: String,
}

/// Results after executing an import.
#[derive(Debug, Clone, Serialize)]
pub struct ImportResultsResponse {
    pub total_processed: usize,
    pub inserted: usize,
    pub skipped: usize,
    pub errors: Vec<ImportErrorDto>,
    /// Accounts the file refers to that do not exist yet.
    pub missing_accounts: Vec<MissingAccountDto>,
    /// Categories the file refers to that do not exist yet.
    pub missing_categories: Vec<MissingCategoryDto>,
    /// Crypto wallets the file refers to that do not exist yet.
    pub missing_wallets: Vec<String>,
}

/// An account to create before importing.
#[derive(Debug, Clone, Serialize)]
pub struct MissingAccountDto {
    pub name: String,
    pub currency: String,
}

/// A category to create before importing.
#[derive(Debug, Clone, Serialize)]
pub struct MissingCategoryDto {
    pub name: String,
    pub category_type: String,
}

/// An error from the import process.
#[derive(Debug, Clone, Serialize)]
pub struct ImportErrorDto {
    pub line: Option<usize>,
    pub message: String,
}

// ==================== Exchange Import ====================

/// Detected exchange source after CSV analysis.
#[derive(Debug, Clone, Serialize)]
pub struct ExchangeDetectionResult {
    pub exchange_id: String,
    pub exchange: String,
    pub suggested_wallet: String,
    pub file_count: usize,
    pub total_records: usize,
}

/// Wallet selection for exchange import.
#[derive(Debug, Clone, Serialize)]
pub struct ExchangeWalletOption {
    pub id: String,
    pub name: String,
    pub is_new: bool,
}

/// Input for selecting a wallet during exchange import.
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeWalletSelectInput {
    pub wallet_name: String,
}

/// Input for adding a missing coin during exchange import.
#[derive(Debug, Clone, Deserialize)]
pub struct MissingCoinInput {
    pub symbol: String,
}

// ==================== Custom CSV Mapping ====================

/// Result of analysing an arbitrary CSV before mapping.
///
/// Carries the detected header row and the first data row so the UI can show
/// the user a concrete example of what each column contains.
#[derive(Debug, Clone, Serialize)]
pub struct CsvAnalysisResult {
    pub headers: Vec<String>,
    pub sample_row: Vec<String>,
}

/// User-chosen column mapping for a custom (unknown-exchange) CSV import.
///
/// Each field holds the header name the user picked for that logical column.
/// Only `date_col`, `asset_col` and `amount_col` are mandatory; the rest are
/// optional and omitted when the source CSV has no such column.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomCsvMapping {
    pub date_col: String,
    pub asset_col: String,
    pub amount_col: String,
    #[serde(default)]
    pub type_col: Option<String>,
    #[serde(default)]
    pub fee_col: Option<String>,
    #[serde(default)]
    pub fee_currency_col: Option<String>,
    #[serde(default)]
    pub price_col: Option<String>,
    #[serde(default)]
    pub notes_col: Option<String>,
}

// ==================== Bank Statements ====================

/// Answer to a statement preview. `status` is `ready`, `needs_password` or
/// `wrong_password`; only `ready` carries a preview.
#[derive(Debug, Clone, Serialize)]
pub struct StatementPreviewResponse {
    pub status: String,
    pub preview: Option<StatementPreviewDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementPreviewDto {
    pub format: String,
    pub format_label: String,
    pub credit_card: bool,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub rows: Vec<StatementRowDto>,
    pub problems: Vec<StatementProblemDto>,
    /// Translated, ready to show.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementRowDto {
    pub date: String,
    pub description: String,
    /// Whole pesos; negative is money out.
    pub amount: i64,
    pub category: String,
    /// The category came from an earlier movement at the same merchant.
    pub learned: bool,
    /// The account already has this movement.
    pub duplicate: bool,
    /// The direction was guessed from the description.
    pub sign_guessed: bool,
    /// "2/6" for the second of six installments.
    pub installment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementProblemDto {
    pub text: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementFormatDto {
    pub id: String,
    pub label: String,
}

/// A reviewed statement row to write.
#[derive(Debug, Clone, Deserialize)]
pub struct StatementEntryInput {
    pub date: String,
    pub description: String,
    /// Whole pesos; negative is money out.
    pub amount: i64,
    pub category: String,
    pub transfer_account_id: Option<String>,
}
