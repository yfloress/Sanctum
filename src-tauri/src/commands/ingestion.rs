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

//! Ingestion domain Tauri commands.

use sanctum::error::AppError;
use sanctum::features::crypto::CryptoService;
use sanctum::features::ingestion::parsers::CustomColumnMapping;
use sanctum::features::ingestion::parsers::detect_exchange_source as parse_exchange_source;
use sanctum::features::ingestion::parsers::statement::{
    RowIssue, StatementFormat, StatementWarning,
};
use sanctum::features::ingestion::service::{StatementEntry, StatementPreview, StatementRead};
use sanctum::features::ingestion::{IngestionService, MAX_FILE_SIZE};
use sanctum::services::i18n::{t, t_args};
use sanctum::ui::currency::format_money;
use sanctum::ui::dto::ingestion::{
    CsvAnalysisResult, CustomCsvMapping, ExchangeDetectionResult, ImportErrorDto,
    ImportResultsResponse, MissingAccountDto, MissingCategoryDto, StatementEntryInput,
    StatementFormatDto, StatementPreviewDto, StatementPreviewResponse, StatementProblemDto,
    StatementRowDto,
};
use secrecy::SecretString;
use tauri::State;

#[tauri::command]
pub fn preview_import(
    ingestion: State<'_, IngestionService>,
    content: String,
    filename: String,
) -> Result<ImportResultsResponse, AppError> {
    let summary = ingestion
        .preview_from_content(&content, &filename)
        .map_err(AppError::from)?;
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn import_data(
    ingestion: State<'_, IngestionService>,
    crypto: State<'_, CryptoService>,
    content: String,
    filename: String,
) -> Result<ImportResultsResponse, AppError> {
    let summary = ingestion
        .import_from_content(&content, &filename)
        .map_err(AppError::from)?;
    backfill_history(&crypto);
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
) -> Result<ImportResultsResponse, AppError> {
    let summary = ingestion
        .preview_exchange_csv_auto(&content, &wallet_name)
        .map_err(AppError::from)?;
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn import_exchange_csv(
    ingestion: State<'_, IngestionService>,
    crypto: State<'_, CryptoService>,
    content: String,
    wallet_name: String,
) -> Result<ImportResultsResponse, AppError> {
    let summary = ingestion
        .import_exchange_csv_auto(&content, &wallet_name)
        .map_err(AppError::from)?;
    backfill_history(&crypto);
    Ok(map_import_summary(summary))
}

#[tauri::command]
pub fn analyze_custom_csv(
    ingestion: State<'_, IngestionService>,
    content: String,
) -> Result<CsvAnalysisResult, AppError> {
    let structure = ingestion
        .analyze_custom_csv(&content)
        .map_err(AppError::from)?;
    Ok(CsvAnalysisResult {
        headers: structure.headers,
        sample_row: structure.sample_row,
    })
}

#[tauri::command]
pub fn import_custom_csv(
    ingestion: State<'_, IngestionService>,
    crypto: State<'_, CryptoService>,
    content: String,
    mapping: CustomCsvMapping,
    wallet_name: String,
) -> Result<ImportResultsResponse, AppError> {
    let summary = ingestion
        .import_custom_csv(&content, into_domain_mapping(mapping), &wallet_name)
        .map_err(AppError::from)?;
    backfill_history(&crypto);
    Ok(map_import_summary(summary))
}

/// Imported crypto history has no daily snapshots, so the charts would start
/// today. A failure here leaves the charts sparse but must not fail the import,
/// which is already committed.
fn backfill_history(crypto: &CryptoService) {
    if let Err(e) = crypto.backfill_portfolio_snapshots() {
        log::warn!("Portfolio history backfill failed: {e}");
    }
}

/// Maps the IPC mapping DTO into the ingestion feature's domain mapping,
/// keeping the service decoupled from the `ui::dto` layer.
fn into_domain_mapping(mapping: CustomCsvMapping) -> CustomColumnMapping {
    CustomColumnMapping {
        date_col: mapping.date_col,
        asset_col: mapping.asset_col,
        amount_col: mapping.amount_col,
        type_col: mapping.type_col,
        fee_col: mapping.fee_col,
        fee_currency_col: mapping.fee_currency_col,
        price_col: mapping.price_col,
        notes_col: mapping.notes_col,
    }
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
        missing_accounts: summary
            .missing_accounts
            .into_iter()
            .map(|a| MissingAccountDto {
                name: a.name,
                currency: a.currency,
            })
            .collect(),
        missing_categories: summary
            .missing_categories
            .into_iter()
            .map(|c| MissingCategoryDto {
                name: c.name,
                category_type: c.category_type,
            })
            .collect(),
        missing_wallets: summary.missing_wallets,
    }
}

// ─── Bank statements ─────────────────────────────────────────────────────────

fn format_label(format: StatementFormat) -> String {
    t(&format!("import-statement-format-{}", format.id()))
}

#[tauri::command]
pub fn statement_formats() -> Vec<StatementFormatDto> {
    StatementFormat::ALL
        .into_iter()
        .map(|f| StatementFormatDto {
            id: f.id().to_string(),
            label: format_label(f),
        })
        .collect()
}

/// Reads a statement PDF into a reviewable preview. The password, when the
/// PDF has one, is used for this call only.
#[tauri::command]
pub fn preview_statement(
    ingestion: State<'_, IngestionService>,
    bytes: Vec<u8>,
    password: Option<String>,
    account_id: String,
    format: Option<String>,
) -> Result<StatementPreviewResponse, AppError> {
    let password = password.filter(|p| !p.is_empty()).map(SecretString::from);
    let format = format.as_deref().and_then(StatementFormat::from_id);
    let read = ingestion
        .preview_statement(&bytes, password.as_ref(), &account_id, format)
        .map_err(AppError::from)?;
    let (status, preview) = match read {
        StatementRead::NeedsPassword => ("needs_password", None),
        StatementRead::WrongPassword => ("wrong_password", None),
        StatementRead::Ready(preview) => ("ready", Some(map_statement_preview(preview))),
    };
    Ok(StatementPreviewResponse {
        status: status.to_string(),
        preview,
    })
}

#[tauri::command]
pub fn import_statement(
    ingestion: State<'_, IngestionService>,
    account_id: String,
    rows: Vec<StatementEntryInput>,
) -> Result<usize, AppError> {
    let entries = rows
        .into_iter()
        .map(|r| StatementEntry {
            date: r.date,
            description: r.description,
            amount: r.amount,
            category: r.category,
            transfer_account_id: r.transfer_account_id.filter(|id| !id.is_empty()),
        })
        .collect();
    ingestion
        .import_statement(&account_id, entries)
        .map_err(AppError::from)
}

fn map_statement_preview(preview: StatementPreview) -> StatementPreviewDto {
    let pesos = |amount: i64| format_money(amount * 100, "CLP");
    let warnings = preview
        .warnings
        .iter()
        .map(|w| match w {
            StatementWarning::TotalsMismatch { declared, parsed } => t_args(
                "import-statement-warning-totals",
                &[
                    ("parsed_out", &pesos(parsed.expense)),
                    ("declared_out", &pesos(declared.expense)),
                    ("parsed_in", &pesos(parsed.income)),
                    ("declared_in", &pesos(declared.income)),
                ],
            ),
            StatementWarning::SectionMismatch {
                section,
                declared,
                parsed,
            } => t_args(
                "import-statement-warning-section",
                &[
                    ("section", section),
                    ("parsed", &pesos(*parsed)),
                    ("declared", &pesos(*declared)),
                ],
            ),
            StatementWarning::TotalsUnreadable => t("import-statement-warning-unreadable"),
        })
        .collect();
    StatementPreviewDto {
        format: preview.format.id().to_string(),
        format_label: format_label(preview.format),
        credit_card: preview.format.is_credit_card(),
        period_start: preview.period.map(|p| p.0.format("%Y-%m-%d").to_string()),
        period_end: preview.period.map(|p| p.1.format("%Y-%m-%d").to_string()),
        rows: preview
            .rows
            .into_iter()
            .map(|r| StatementRowDto {
                date: r.row.date.format("%Y-%m-%d").to_string(),
                description: r.row.description,
                amount: r.row.amount,
                category: r.category,
                learned: r.learned,
                duplicate: r.duplicate,
                sign_guessed: r.row.sign_from_keyword,
                installment: r.row.installment.map(|(n, c)| format!("{n}/{c}")),
            })
            .collect(),
        problems: preview
            .problems
            .into_iter()
            .map(|p| StatementProblemDto {
                text: p.text,
                reason: t(&format!("import-statement-issue-{}", issue_id(p.issue))),
            })
            .collect(),
        warnings,
    }
}

fn issue_id(issue: RowIssue) -> &'static str {
    match issue {
        RowIssue::MissingAmount | RowIssue::ZeroAmount => "amount",
        RowIssue::AmbiguousAmount => "ambiguous",
        RowIssue::UnknownSign => "sign",
        RowIssue::MissingDescription => "description",
        RowIssue::MissingYear | RowIssue::OutsidePeriod | RowIssue::InvalidDate => "date",
        RowIssue::UnreadableRow => "unreadable",
    }
}
