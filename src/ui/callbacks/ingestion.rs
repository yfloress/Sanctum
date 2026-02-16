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

//! Data ingestion callbacks
//!
//! Handles file selection and import result mapping for the UI.
//! Supports both generic data import (JSON/CSV/TXT) and exchange-specific
//! CSV import (Kraken, Binance, Feather Wallet).

use crate::controller::AppController;
use crate::features::ingestion::{ImportSummary, RowError};
use crate::services::i18n::t_args;
use crate::{AppState, AppWindow, ImportErrorData, ImportPreviewChange, IngestionAdapter};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

struct PendingImport {
    filename: String,
    display_name: String,
    content: String,
}

struct PendingExchangeImport {
    display_name: String,
    content: String,
    wallet_name: String,
}

pub fn setup_ingestion_callbacks(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
) {
    let pending_import: Rc<RefCell<Option<PendingImport>>> = Rc::new(RefCell::new(None));
    let pending_exchange: Rc<RefCell<Option<PendingExchangeImport>>> = Rc::new(RefCell::new(None));

    // ── Generic data import (JSON / CSV / TXT) ──────────────────────────────

    // Open file picker and import data
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let pending_import = pending_import.clone();

        ui.global::<IngestionAdapter>().on_import_data(move || {
            let file_path = rfd::FileDialog::new()
                .add_filter("Data Files", &["json", "csv", "txt"])
                .pick_file();

            let Some(path) = file_path else {
                return;
            };

            pending_import.borrow_mut().take();

            let display_name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());
            let filename = display_name.clone();
            let content = match std::fs::read_to_string(&path) {
                Ok(data) => data,
                Err(err) => {
                    pending_import.borrow_mut().take();
                    if let Some(ui) = ui_weak.upgrade() {
                        let summary = build_error_summary(format!("Failed to read file: {}", err));
                        set_import_summary(&ui, summary, Some(display_name.clone()));
                        ui.global::<AppState>().set_show_import_preview(true);
                    }
                    return;
                }
            };

            let summary = match controller.preview_data(&content, &filename) {
                Ok(summary) => summary,
                Err(err) => build_error_summary(err.to_string()),
            };

            if let Some(ui) = ui_weak.upgrade() {
                pending_import.borrow_mut().replace(PendingImport {
                    filename,
                    display_name: display_name.clone(),
                    content,
                });
                set_import_summary(&ui, summary, Some(display_name));
                ui.global::<AppState>().set_show_import_preview(true);
            }
        });
    }

    // Confirm import after preview
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let pending_import = pending_import.clone();

        ui.global::<IngestionAdapter>().on_confirm_import(move || {
            let pending = pending_import.borrow_mut().take();
            let Some(pending) = pending else {
                return;
            };

            let summary = match controller.import_data(pending.content, pending.filename) {
                Ok(summary) => summary,
                Err(err) => build_error_summary(err.to_string()),
            };

            if let Some(ui) = ui_weak.upgrade() {
                set_import_summary(&ui, summary, Some(pending.display_name));
                ui.global::<AppState>().set_show_import_preview(false);
                ui.global::<AppState>().set_show_import_results(true);
            }
        });
    }

    // Cancel preview
    {
        let ui_weak = ui_weak.clone();
        let pending_import = pending_import.clone();

        ui.global::<IngestionAdapter>().on_cancel_preview(move || {
            pending_import.borrow_mut().take();
            if let Some(ui) = ui_weak.upgrade() {
                clear_import_summary(&ui);
            }
        });
    }

    // Clear results
    {
        let ui_weak = ui_weak.clone();
        ui.global::<IngestionAdapter>().on_reset_results(move || {
            if let Some(ui) = ui_weak.upgrade() {
                clear_import_summary(&ui);
            }
        });
    }

    // ── Exchange CSV import ─────────────────────────────────────────────────

    // Open file picker for exchange CSV, auto-detect format, preview
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let pending_exchange = pending_exchange.clone();

        ui.global::<IngestionAdapter>()
            .on_import_exchange_csv(move || {
                let file_path = rfd::FileDialog::new()
                    .add_filter("CSV Files", &["csv"])
                    .pick_file();

                let Some(path) = file_path else {
                    return;
                };

                pending_exchange.borrow_mut().take();

                let display_name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                let content = match std::fs::read_to_string(&path) {
                    Ok(data) => data,
                    Err(err) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            let summary =
                                build_error_summary(format!("Failed to read file: {}", err));
                            set_import_summary(&ui, summary, Some(display_name.clone()));
                            ui.global::<AppState>().set_show_import_preview(true);
                        }
                        return;
                    }
                };

                // Detect exchange source from CSV headers
                let detected = controller.detect_exchange_source(&content);

                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };

                let adapter = ui.global::<IngestionAdapter>();

                match detected {
                    Some((_, exchange_label, default_wallet)) => {
                        adapter.set_exchange_detected_format(SharedString::from(&exchange_label));

                        // Use user-provided wallet name if set, otherwise use default
                        let current_wallet = adapter.get_exchange_wallet_name().to_string();
                        let wallet_name = if current_wallet.trim().is_empty() {
                            adapter.set_exchange_wallet_name(SharedString::from(&default_wallet));
                            default_wallet.clone()
                        } else {
                            current_wallet
                        };

                        adapter.set_exchange_file_loaded(true);

                        // Preview with the determined wallet name
                        let summary =
                            match controller.preview_exchange_csv(&content, &wallet_name) {
                                Ok(summary) => summary,
                                Err(err) => build_error_summary(err.to_string()),
                            };

                        pending_exchange
                            .borrow_mut()
                            .replace(PendingExchangeImport {
                                display_name: display_name.clone(),
                                content,
                                wallet_name,
                            });

                        set_import_summary(&ui, summary, Some(display_name));
                        adapter.set_is_exchange_import(true);
                        ui.global::<AppState>().set_show_import_preview(true);
                    }
                    None => {
                        adapter.set_exchange_detected_format(SharedString::from(""));
                        adapter.set_exchange_file_loaded(false);

                        let summary = build_error_summary(
                            "Could not detect exchange format. Supported: Kraken, Binance, Feather Wallet.".to_string(),
                        );
                        set_import_summary(&ui, summary, Some(display_name));
                        adapter.set_is_exchange_import(true);
                        ui.global::<AppState>().set_show_import_preview(true);
                    }
                }
            });
    }

    // Confirm exchange import after preview
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let pending_exchange = pending_exchange.clone();

        ui.global::<IngestionAdapter>()
            .on_confirm_exchange_import(move || {
                let pending = pending_exchange.borrow_mut().take();
                let Some(pending) = pending else {
                    return;
                };

                let summary =
                    match controller.import_exchange_csv(pending.content, pending.wallet_name) {
                        Ok(summary) => summary,
                        Err(err) => build_error_summary(err.to_string()),
                    };

                if let Some(ui) = ui_weak.upgrade() {
                    set_import_summary(&ui, summary, Some(pending.display_name));
                    ui.global::<AppState>().set_show_import_preview(false);
                    ui.global::<AppState>().set_show_import_results(true);

                    // Clear exchange-specific state
                    let adapter = ui.global::<IngestionAdapter>();
                    adapter.set_exchange_detected_format(SharedString::from(""));
                    adapter.set_exchange_wallet_name(SharedString::from(""));
                    adapter.set_exchange_file_loaded(false);
                    adapter.set_is_exchange_import(false);
                }
            });
    }

    // Cancel exchange preview
    {
        let ui_weak = ui_weak.clone();
        let pending_exchange = pending_exchange.clone();

        ui.global::<IngestionAdapter>()
            .on_cancel_exchange_preview(move || {
                pending_exchange.borrow_mut().take();
                if let Some(ui) = ui_weak.upgrade() {
                    clear_import_summary(&ui);
                    let adapter = ui.global::<IngestionAdapter>();
                    adapter.set_exchange_detected_format(SharedString::from(""));
                    adapter.set_exchange_wallet_name(SharedString::from(""));
                    adapter.set_exchange_file_loaded(false);
                    adapter.set_is_exchange_import(false);
                }
            });
    }

    // Wallet name changed — store it for the pending exchange import
    {
        let pending_exchange = pending_exchange.clone();

        ui.global::<IngestionAdapter>()
            .on_exchange_wallet_name_changed(move |name| {
                let name_str = name.to_string();
                if let Some(pending) = pending_exchange.borrow_mut().as_mut() {
                    pending.wallet_name = name_str;
                }
            });
    }
}

fn build_error_summary(message: String) -> ImportSummary {
    let mut summary = ImportSummary::new("Unknown", "Unknown");
    summary.record_error(RowError::new(0, None, message));
    summary
}

fn clear_import_summary(ui: &AppWindow) {
    let adapter = ui.global::<IngestionAdapter>();
    adapter.set_import_file_name(SharedString::from(""));
    adapter.set_import_format(SharedString::from(""));
    adapter.set_import_data_type(SharedString::from(""));
    adapter.set_import_total_processed(0);
    adapter.set_import_inserted(0);
    adapter.set_import_skipped(0);
    adapter.set_import_errors(0);
    adapter.set_import_error_details(ModelRc::new(VecModel::from(Vec::<ImportErrorData>::new())));
    adapter.set_import_skipped_reasons(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
    adapter.set_import_preview_changes(ModelRc::new(VecModel::from(
        Vec::<ImportPreviewChange>::new(),
    )));
}

fn set_import_summary(ui: &AppWindow, summary: ImportSummary, file_name: Option<String>) {
    let adapter = ui.global::<IngestionAdapter>();
    adapter.set_import_file_name(SharedString::from(file_name.unwrap_or_default()));
    adapter.set_import_format(SharedString::from(summary.format.clone()));
    adapter.set_import_data_type(SharedString::from(summary.data_type.clone()));
    adapter.set_import_total_processed(summary.total_processed as i32);
    adapter.set_import_inserted(summary.inserted as i32);
    adapter.set_import_skipped(summary.skipped as i32);
    adapter.set_import_errors(summary.errors as i32);

    let errors: Vec<ImportErrorData> = summary.error_details.into_iter().map(map_error).collect();
    adapter.set_import_error_details(ModelRc::new(VecModel::from(errors)));

    let skipped_reasons: Vec<SharedString> = summary
        .skipped_reasons
        .into_iter()
        .map(SharedString::from)
        .collect();
    adapter.set_import_skipped_reasons(ModelRc::new(VecModel::from(skipped_reasons)));

    let changes: Vec<ImportPreviewChange> = summary
        .preview_changes
        .into_iter()
        .map(|c| ImportPreviewChange {
            change_type: c.change_type.into(),
            summary: c.summary.into(),
            details: c.details.into(),
        })
        .collect();
    adapter.set_import_preview_changes(ModelRc::new(VecModel::from(changes)));
}

fn map_error(err: RowError) -> ImportErrorData {
    let line_label = if err.line_number > 0 {
        let line_value = err.line_number.to_string();
        t_args("import-line", &[("line", line_value.as_str())])
    } else {
        String::new()
    };
    let field_label = err
        .field
        .as_deref()
        .filter(|field| !field.trim().is_empty())
        .map(|field| t_args("import-field", &[("field", field)]))
        .unwrap_or_default();

    ImportErrorData {
        line: err.line_number as i32,
        field: err.field.unwrap_or_default().into(),
        line_label: line_label.into(),
        field_label: field_label.into(),
        message: err.message.into(),
        raw_data: err.raw_data.unwrap_or_default().into(),
    }
}
