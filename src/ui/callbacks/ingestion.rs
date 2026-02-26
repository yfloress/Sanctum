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
//! CSV import (Kraken, Binance, MEXC, NotBank, Feather Wallet, Monero GUI).

use crate::controller::AppController;
use crate::features::ingestion::{ImportSummary, RowError};
use crate::services::i18n::t;
use crate::services::i18n::t_args;
use crate::{
    AppState, AppWindow, CryptoAdapter, ImportErrorData, ImportPreviewChange, IngestionAdapter,
    NotificationAdapter,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
struct PendingImport {
    filename: String,
    display_name: String,
    content: String,
}

#[derive(Clone)]
struct PendingExchangeImport {
    display_name: String,
    detected_format: String,
    files: Vec<PendingExchangeFile>,
    preflight_errors: Vec<RowError>,
    preflight_skips: Vec<String>,
    wallet_name: String,
}

#[derive(Clone)]
struct PendingExchangeFile {
    display_name: String,
    source_id: String,
    content: String,
}

pub fn setup_ingestion_callbacks(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
) {
    let pending_import: Rc<RefCell<Option<PendingImport>>> = Rc::new(RefCell::new(None));
    let pending_exchange: Rc<RefCell<Option<PendingExchangeImport>>> = Rc::new(RefCell::new(None));
    let last_exchange_import: Rc<RefCell<Option<PendingExchangeImport>>> =
        Rc::new(RefCell::new(None));

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
                refresh_crypto_views(&ui);
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
        let last_exchange_import = last_exchange_import.clone();

        ui.global::<IngestionAdapter>()
            .on_import_exchange_csv(move || {
                let file_paths = rfd::FileDialog::new()
                    .add_filter("CSV Files", &["csv"])
                    .pick_files();

                let Some(paths) = file_paths else {
                    return;
                };
                if paths.is_empty() {
                    return;
                }

                pending_exchange.borrow_mut().take();
                last_exchange_import.borrow_mut().take();

                let mut pending_files = Vec::new();
                let mut detected_labels = Vec::new();
                let mut preflight_errors = Vec::new();
                let mut selected_names = Vec::new();

                for path in paths {
                    let display_name = path
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.to_string_lossy().to_string());
                    selected_names.push(display_name.clone());

                    let content = match std::fs::read_to_string(&path) {
                        Ok(data) => data,
                        Err(err) => {
                            preflight_errors.push(RowError::new(
                                0,
                                None,
                                format!("{}: Failed to read file: {}", display_name, err),
                            ));
                            continue;
                        }
                    };

                    if let Some((exchange_id, exchange_label, _)) =
                        controller.detect_exchange_source(&content)
                    {
                        detected_labels.push(exchange_label);
                        pending_files.push(PendingExchangeFile {
                            display_name,
                            source_id: exchange_id,
                            content,
                        });
                    } else {
                        preflight_errors.push(RowError::new(
                            0,
                            None,
                            format!("{}: {}", display_name, t("import-exchange-not-detected")),
                        ));
                    }
                }

                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };

                let adapter = ui.global::<IngestionAdapter>();
                if pending_files.is_empty() {
                    adapter.set_exchange_detected_format(SharedString::from(""));
                    adapter.set_exchange_file_loaded(false);
                    let msg = t("import-exchange-not-detected");
                    ui.global::<NotificationAdapter>()
                        .invoke_show(SharedString::from(msg), true);
                    return;
                }

                let (pending_files, preflight_skips) = apply_exchange_batch_filters(pending_files);

                let combined_format = combine_exchange_labels(&detected_labels);
                let combined_display = build_batch_display_name(&selected_names);

                adapter.set_exchange_detected_format(SharedString::from(&combined_format));
                adapter.set_exchange_file_loaded(true);
                // Clear wallet name so user must choose in the next step
                adapter.set_exchange_wallet_name(SharedString::from(""));

                // Store CSV contents temporarily (wallet not chosen yet)
                pending_exchange
                    .borrow_mut()
                    .replace(PendingExchangeImport {
                        display_name: combined_display,
                        detected_format: combined_format,
                        files: pending_files,
                        preflight_errors,
                        preflight_skips,
                        wallet_name: String::new(),
                    });

                // Ensure wallet list is loaded (user may not have visited crypto module yet)
                ui.global::<CryptoAdapter>().invoke_fetch_wallets();

                // Show wallet selection modal instead of going to preview
                ui.global::<AppState>()
                    .set_show_exchange_wallet_select(true);
            });
    }

    // Continue exchange import after wallet selection
    // The wallet name has been set on IngestionAdapter.exchange-wallet-name
    // by the ExchangeWalletSelectModal before this callback fires.
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let pending_exchange = pending_exchange.clone();

        ui.global::<IngestionAdapter>()
            .on_continue_exchange_with_wallet(move || {
                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };

                let adapter = ui.global::<IngestionAdapter>();
                let wallet_name = adapter.get_exchange_wallet_name().to_string();

                if wallet_name.trim().is_empty() {
                    return;
                }

                let mut borrow = pending_exchange.borrow_mut();
                let Some(pending) = borrow.as_mut() else {
                    return;
                };

                // Update the wallet name chosen by the user
                pending.wallet_name = wallet_name.clone();
                let snapshot = pending.clone();
                drop(borrow);

                let summary = build_exchange_preview_summary(&controller, &snapshot);
                set_import_summary(&ui, summary, Some(snapshot.display_name));
                adapter.set_is_exchange_import(true);
                ui.global::<AppState>().set_show_import_preview(true);
            });
    }

    // Cancel exchange wallet selection
    {
        let ui_weak = ui_weak.clone();
        let pending_exchange = pending_exchange.clone();

        ui.global::<IngestionAdapter>()
            .on_cancel_exchange_wallet_select(move || {
                pending_exchange.borrow_mut().take();
                if let Some(ui) = ui_weak.upgrade() {
                    let adapter = ui.global::<IngestionAdapter>();
                    adapter.set_exchange_detected_format(SharedString::from(""));
                    adapter.set_exchange_wallet_name(SharedString::from(""));
                    adapter.set_exchange_file_loaded(false);
                    adapter.set_is_exchange_import(false);
                }
            });
    }

    // Confirm exchange import after preview
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let pending_exchange = pending_exchange.clone();
        let last_exchange_import = last_exchange_import.clone();

        ui.global::<IngestionAdapter>()
            .on_confirm_exchange_import(move || {
                let pending = pending_exchange.borrow_mut().take();
                let Some(pending) = pending else {
                    return;
                };
                let summary = build_exchange_import_summary(&controller, &pending);
                let display_name = pending.display_name.clone();
                last_exchange_import.borrow_mut().replace(pending);

                if let Some(ui) = ui_weak.upgrade() {
                    set_import_summary(&ui, summary, Some(display_name));
                    ui.global::<AppState>().set_show_import_preview(false);
                    ui.global::<AppState>().set_show_import_results(true);
                    refresh_crypto_views(&ui);

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

    // Add missing coin from exchange import error row and retry automatically.
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let pending_exchange = pending_exchange.clone();
        let last_exchange_import = last_exchange_import.clone();

        ui.global::<IngestionAdapter>()
            .on_add_missing_exchange_coin(move |symbol| {
                let requested_symbol = symbol.to_string();
                let Some(normalized_symbol) = normalize_missing_coin_symbol(&requested_symbol)
                else {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.global::<NotificationAdapter>().invoke_show(
                            SharedString::from(t_args(
                                "import-exchange-coin-invalid",
                                &[("symbol", requested_symbol.as_str())],
                            )),
                            true,
                        );
                    }
                    return;
                };

                let catalog = controller.get_coin_catalog_or_default();
                let symbol_exists = catalog
                    .iter()
                    .any(|coin| coin.symbol.eq_ignore_ascii_case(&normalized_symbol));
                let mut can_retry = symbol_exists;

                if !symbol_exists {
                    let coin_id = build_custom_coin_id(&catalog, &normalized_symbol);
                    match controller.add_custom_coin(
                        coin_id,
                        normalized_symbol.clone(),
                        normalized_symbol.clone(),
                    ) {
                        Ok(_) => {
                            can_retry = true;
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.global::<NotificationAdapter>().invoke_show(
                                    SharedString::from(t_args(
                                        "import-exchange-coin-added",
                                        &[("symbol", normalized_symbol.as_str())],
                                    )),
                                    false,
                                );
                            }
                        }
                        Err(err) => {
                            let reason = err.to_string();
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.global::<NotificationAdapter>().invoke_show(
                                    SharedString::from(t_args(
                                        "import-exchange-coin-add-failed",
                                        &[
                                            ("symbol", normalized_symbol.as_str()),
                                            ("reason", reason.as_str()),
                                        ],
                                    )),
                                    true,
                                );
                            }
                        }
                    }
                }

                if !can_retry {
                    return;
                }

                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };

                ui.global::<CryptoAdapter>().invoke_load_coin_catalog();

                if let Some(pending) = pending_exchange.borrow().as_ref().cloned() {
                    let summary = build_exchange_preview_summary(&controller, &pending);
                    set_import_summary(&ui, summary, Some(pending.display_name));
                    ui.global::<IngestionAdapter>().set_is_exchange_import(true);
                    ui.global::<AppState>().set_show_import_preview(true);
                    return;
                }

                if let Some(last) = last_exchange_import.borrow().as_ref().cloned() {
                    let summary = build_exchange_import_summary(&controller, &last);
                    set_import_summary(&ui, summary, Some(last.display_name));
                    ui.global::<AppState>().set_show_import_results(true);
                    refresh_crypto_views(&ui);
                    return;
                }

                ui.global::<NotificationAdapter>().invoke_show(
                    SharedString::from(t("import-exchange-coin-retry-unavailable")),
                    true,
                );
            });
    }
}

fn build_exchange_preview_summary(
    controller: &AppController,
    pending: &PendingExchangeImport,
) -> ImportSummary {
    let mut summary = ImportSummary::new(&pending.detected_format, "Crypto");
    for err in pending.preflight_errors.iter().cloned() {
        summary.record_error(err);
    }
    for reason in &pending.preflight_skips {
        summary.record_skipped(reason);
    }
    for file in &pending.files {
        match controller.preview_exchange_csv(&file.content, &pending.wallet_name) {
            Ok(file_summary) => summary.merge(file_summary),
            Err(err) => summary.record_error(RowError::new(
                0,
                None,
                format!("{}: {}", file.display_name, err),
            )),
        }
    }
    summary
}

fn build_exchange_import_summary(
    controller: &AppController,
    pending: &PendingExchangeImport,
) -> ImportSummary {
    let mut summary = ImportSummary::new(&pending.detected_format, "Crypto");
    for err in pending.preflight_errors.iter().cloned() {
        summary.record_error(err);
    }
    for reason in &pending.preflight_skips {
        summary.record_skipped(reason);
    }
    for file in &pending.files {
        match controller.import_exchange_csv(file.content.clone(), pending.wallet_name.clone()) {
            Ok(file_summary) => summary.merge(file_summary),
            Err(err) => summary.record_error(RowError::new(
                0,
                None,
                format!("{}: {}", file.display_name, err),
            )),
        }
    }
    summary
}

fn normalize_missing_coin_symbol(raw: &str) -> Option<String> {
    let mut compact: String = raw.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if compact.is_empty() {
        return None;
    }
    if compact.len() > 10 {
        compact.truncate(10);
    }
    Some(compact.to_uppercase())
}

fn extract_missing_coin_symbol(err: &RowError) -> Option<String> {
    let field = err.field.as_deref()?.trim().to_ascii_lowercase();
    if field != "symbol" && field != "swap_to_symbol" && field != "fee_coin_symbol" {
        return None;
    }

    let candidate = err.message.rsplit(':').next().unwrap_or("").trim();
    normalize_missing_coin_symbol(candidate)
}

fn build_custom_coin_id(catalog: &[crate::models::CryptoCatalogCoin], symbol: &str) -> String {
    let base = symbol.to_ascii_lowercase();
    let existing: HashSet<String> = catalog
        .iter()
        .map(|coin| coin.id.to_ascii_lowercase())
        .collect();

    if !existing.contains(&base) {
        return base;
    }

    for suffix in 2..=100 {
        let candidate = format!("{}-{}", base, suffix);
        if !existing.contains(&candidate) {
            return candidate;
        }
    }

    format!("{}-custom", base)
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

fn refresh_crypto_views(ui: &AppWindow) {
    let crypto = ui.global::<CryptoAdapter>();
    crypto.invoke_fetch_portfolio();
    crypto.invoke_fetch_wallets();

    let selected_wallet_id = crypto.get_selected_wallet_id().to_string();
    if !selected_wallet_id.trim().is_empty() {
        crypto.invoke_fetch_wallet_details(SharedString::from(selected_wallet_id));
    }

    let selected_asset_id = crypto.get_selected_asset().id.to_string();
    if !selected_asset_id.trim().is_empty() {
        crypto.invoke_fetch_asset_details(SharedString::from(selected_asset_id));
    }
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
    let missing_coin_symbol = extract_missing_coin_symbol(&err).unwrap_or_default();
    let can_add_missing_coin = !missing_coin_symbol.is_empty();

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
        can_add_missing_coin,
        missing_coin_symbol: missing_coin_symbol.into(),
    }
}

fn combine_exchange_labels(labels: &[String]) -> String {
    let mut unique = Vec::<String>::new();
    for label in labels {
        if !unique.iter().any(|item| item == label) {
            unique.push(label.clone());
        }
    }
    match unique.len() {
        0 => "CSV".to_string(),
        1..=3 => unique.join(" / "),
        _ => {
            let head = unique[..3].join(" / ");
            let extra = unique.len() - 3;
            format!("{} +{}", head, extra)
        }
    }
}

fn apply_exchange_batch_filters(
    files: Vec<PendingExchangeFile>,
) -> (Vec<PendingExchangeFile>, Vec<String>) {
    let has_kraken_ledger = files.iter().any(|f| f.source_id == "kraken_ledger");
    let has_mexc_futures_trades = files.iter().any(|f| f.source_id == "mexc_futures_trades");
    let has_mexc_trade_history = files.iter().any(|f| f.source_id == "mexc_trades");
    let has_binance_all = files.iter().any(|f| f.source_id == "binance_all");

    let mut filtered = Vec::with_capacity(files.len());
    let mut skipped = Vec::new();

    for file in files {
        if has_kraken_ledger && file.source_id == "kraken_trades" {
            skipped.push(format!(
                "{}: skipped overlapping source (covered by Kraken Ledger)",
                file.display_name
            ));
            continue;
        }
        if has_mexc_trade_history && file.source_id == "mexc_spot" {
            skipped.push(format!(
                "{}: skipped overlapping source (covered by MEXC Trade History)",
                file.display_name
            ));
            continue;
        }
        if has_binance_all && file.source_id == "binance_spot" {
            skipped.push(format!(
                "{}: skipped overlapping source (covered by Binance All Statements)",
                file.display_name
            ));
            continue;
        }
        if has_mexc_futures_trades
            && (file.source_id == "mexc_futures_orders"
                || file.source_id == "mexc_futures_positions")
        {
            skipped.push(format!(
                "{}: skipped duplicate source (covered by MEXC Futures Trade History)",
                file.display_name
            ));
            continue;
        }
        filtered.push(file);
    }

    filtered.sort_by_key(|file| match file.source_id.as_str() {
        "kraken_ledger" => 0_i32,
        "notbank_transaction" => 0_i32,
        "kraken_trades" => 1_i32,
        "notbank_trade" => 1_i32,
        _ => 2_i32,
    });

    (filtered, skipped)
}

fn build_batch_display_name(file_names: &[String]) -> String {
    match file_names {
        [] => String::new(),
        [single] => single.clone(),
        [first, rest @ ..] => format!("{} +{}", first, rest.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PendingExchangeFile, apply_exchange_batch_filters, build_batch_display_name,
        build_custom_coin_id, combine_exchange_labels, extract_missing_coin_symbol,
        normalize_missing_coin_symbol,
    };
    use crate::features::ingestion::RowError;
    use crate::models::CryptoCatalogCoin;

    fn coin(id: &str, symbol: &str) -> CryptoCatalogCoin {
        CryptoCatalogCoin {
            id: id.to_string(),
            name: symbol.to_string(),
            symbol: symbol.to_string(),
            custom: true,
        }
    }

    #[test]
    fn combine_exchange_labels_deduplicates_preserving_order() {
        let labels = vec!["MEXC".to_string(), "Kraken".to_string(), "MEXC".to_string()];
        assert_eq!(combine_exchange_labels(&labels), "MEXC / Kraken");
    }

    #[test]
    fn combine_exchange_labels_limits_long_lists() {
        let labels = vec![
            "MEXC Spot".to_string(),
            "MEXC Funding".to_string(),
            "MEXC Futures".to_string(),
            "MEXC Earn".to_string(),
        ];
        assert_eq!(
            combine_exchange_labels(&labels),
            "MEXC Spot / MEXC Funding / MEXC Futures +1"
        );
    }

    #[test]
    fn build_batch_display_name_uses_first_name_and_count() {
        let files = vec![
            "spot.csv".to_string(),
            "withdrawals.csv".to_string(),
            "deposits.csv".to_string(),
        ];
        assert_eq!(build_batch_display_name(&files), "spot.csv +2");
    }

    #[test]
    fn batch_filter_skips_mexc_spot_when_trade_history_present() {
        let files = vec![
            PendingExchangeFile {
                display_name: "Spot-Spot Order History.csv".to_string(),
                source_id: "mexc_spot".to_string(),
                content: "spot".to_string(),
            },
            PendingExchangeFile {
                display_name: "Spot-Spot Trade History.csv".to_string(),
                source_id: "mexc_trades".to_string(),
                content: "trades".to_string(),
            },
        ];

        let (filtered, skipped) = apply_exchange_batch_filters(files);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source_id, "mexc_trades");
        assert_eq!(skipped.len(), 1);
    }

    #[test]
    fn batch_filter_skips_overlapping_mexc_futures_reports() {
        let files = vec![
            PendingExchangeFile {
                display_name: "Futures-Futures Order History.csv".to_string(),
                source_id: "mexc_futures_orders".to_string(),
                content: "orders".to_string(),
            },
            PendingExchangeFile {
                display_name: "Futures-Futures Position History.csv".to_string(),
                source_id: "mexc_futures_positions".to_string(),
                content: "positions".to_string(),
            },
            PendingExchangeFile {
                display_name: "Futures-Futures Trade History.csv".to_string(),
                source_id: "mexc_futures_trades".to_string(),
                content: "trades".to_string(),
            },
        ];

        let (filtered, skipped) = apply_exchange_batch_filters(files);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source_id, "mexc_futures_trades");
        assert_eq!(skipped.len(), 2);
    }

    #[test]
    fn batch_filter_skips_binance_spot_when_all_statements_present() {
        let files = vec![
            PendingExchangeFile {
                display_name: "binance-all.csv".to_string(),
                source_id: "binance_all".to_string(),
                content: "all".to_string(),
            },
            PendingExchangeFile {
                display_name: "binance-spot.csv".to_string(),
                source_id: "binance_spot".to_string(),
                content: "spot".to_string(),
            },
        ];

        let (filtered, skipped) = apply_exchange_batch_filters(files);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source_id, "binance_all");
        assert_eq!(skipped.len(), 1);
    }

    #[test]
    fn batch_filter_skips_kraken_trades_when_ledger_present() {
        let files = vec![
            PendingExchangeFile {
                display_name: "kraken-ledger.csv".to_string(),
                source_id: "kraken_ledger".to_string(),
                content: "ledger".to_string(),
            },
            PendingExchangeFile {
                display_name: "kraken-trades.csv".to_string(),
                source_id: "kraken_trades".to_string(),
                content: "trades".to_string(),
            },
        ];

        let (filtered, skipped) = apply_exchange_batch_filters(files);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source_id, "kraken_ledger");
        assert_eq!(skipped.len(), 1);
    }

    #[test]
    fn batch_filter_prefers_kraken_ledger_over_kraken_trades() {
        let files = vec![
            PendingExchangeFile {
                display_name: "trades.csv".to_string(),
                source_id: "kraken_trades".to_string(),
                content: "trades".to_string(),
            },
            PendingExchangeFile {
                display_name: "ledgers.csv".to_string(),
                source_id: "kraken_ledger".to_string(),
                content: "ledger".to_string(),
            },
        ];

        let (filtered, skipped) = apply_exchange_batch_filters(files);
        assert_eq!(skipped.len(), 1);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source_id, "kraken_ledger");
    }

    #[test]
    fn batch_filter_keeps_notbank_trade_when_transaction_present() {
        let files = vec![
            PendingExchangeFile {
                display_name: "Trade Activity.csv".to_string(),
                source_id: "notbank_trade".to_string(),
                content: "trade".to_string(),
            },
            PendingExchangeFile {
                display_name: "Transaction.csv".to_string(),
                source_id: "notbank_transaction".to_string(),
                content: "transaction".to_string(),
            },
        ];

        let (filtered, skipped) = apply_exchange_batch_filters(files);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].source_id, "notbank_transaction");
        assert_eq!(filtered[1].source_id, "notbank_trade");
        assert!(skipped.is_empty());
    }

    #[test]
    fn normalize_missing_coin_symbol_keeps_alphanumeric_uppercase() {
        assert_eq!(
            normalize_missing_coin_symbol(" pepe-usdt "),
            Some("PEPEUSDT".to_string())
        );
        assert_eq!(normalize_missing_coin_symbol(""), None);
    }

    #[test]
    fn extract_missing_coin_symbol_from_catalog_error() {
        let err = RowError::new(7, Some("symbol"), "Crypto asset not found in catalog: pepe");
        assert_eq!(extract_missing_coin_symbol(&err), Some("PEPE".to_string()));
    }

    #[test]
    fn extract_missing_coin_symbol_ignores_unrelated_errors() {
        let err = RowError::new(7, Some("amount"), "Amount must be positive");
        assert_eq!(extract_missing_coin_symbol(&err), None);
    }

    #[test]
    fn build_custom_coin_id_avoids_existing_ids() {
        let catalog = vec![coin("pepe", "PEPE"), coin("pepe-2", "PEP2")];
        assert_eq!(build_custom_coin_id(&catalog, "PEPE"), "pepe-3");
    }
}
