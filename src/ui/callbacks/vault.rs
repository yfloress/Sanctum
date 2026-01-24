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

// File: vault.rs

//! Vault backup and restore callbacks
//!
//! Handles vault export and restore operations with native file dialogs.

use crate::controller::AppController;
use crate::{AppState, AppWindow, VaultAdapter};
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;

pub fn setup_vault_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // Export vault callback
    {
        let controller = controller.clone();
        let notify = notify.clone();

        ui.global::<VaultAdapter>().on_export_vault(move || {
            // Open file save dialog
            let file_path = rfd::FileDialog::new()
                .add_filter("Sanctum Vault", &["db"])
                .set_file_name("sanctum_backup.db")
                .save_file();

            if let Some(path) = file_path {
                let path_str = path.to_string_lossy().to_string();

                match controller.export_vault(path_str.clone()) {
                    Ok(_) => {
                        notify("Vault backup created successfully".to_string(), false);
                        log::info!("Vault exported to: {}", path_str);
                    }
                    Err(e) => {
                        notify(format!("Failed to export vault: {}", e), true);
                        log::error!("Export failed: {}", e);
                    }
                }
            }
        });
    }

    // Restore vault callback
    {
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<VaultAdapter>().on_restore_vault(move || {
            // Open file picker dialog
            let file_path = rfd::FileDialog::new()
                .add_filter("Sanctum Vault", &["db"])
                .pick_file();

            if let Some(path) = file_path {
                let path_str = path.to_string_lossy().to_string();

                // Set the backup path and show confirmation modal
                if let Some(ui) = ui_weak.upgrade() {
                    ui.global::<VaultAdapter>()
                        .set_restore_backup_path(SharedString::from(path_str.clone()));
                    ui.global::<AppState>().set_show_restore_vault(true);

                    log::info!("User selected backup file: {}", path_str);
                } else {
                    notify("UI error: failed to show restore dialog".to_string(), true);
                }
            }
        });
    }

    // Confirm restore callback (actual restore operation)
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<VaultAdapter>().on_confirm_restore(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let backup_path = ui
                    .global::<VaultAdapter>()
                    .get_restore_backup_path()
                    .to_string();

                if backup_path.is_empty() {
                    notify("No backup file selected".to_string(), true);
                    return;
                }

                match controller.restore_vault(backup_path.clone()) {
                    Ok(_) => {
                        notify(
                            "Vault restored successfully. Please log in with your backup password."
                                .to_string(),
                            false,
                        );

                        // Close vault modal
                        ui.global::<AppState>().set_show_restore_vault(false);

                        // Clear backup path
                        ui.global::<VaultAdapter>()
                            .set_restore_backup_path(SharedString::from(""));

                        // Redirect to login screen
                        ui.global::<AppState>().set_is_logged_in(false);

                        log::info!("Vault restored from: {}", backup_path);
                    }
                    Err(e) => {
                        notify(format!("Failed to restore vault: {}", e), true);
                        log::error!("Restore failed: {}", e);
                    }
                }
            }
        });
    }

    // Rollback restore callback
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<VaultAdapter>().on_rollback_restore(move || {
            match controller.rollback_restore() {
                Ok(_) => {
                    notify("Restore rolled back successfully".to_string(), false);

                    // Redirect to login screen
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.global::<AppState>().set_is_logged_in(false);
                    }

                    log::info!("Restore rolled back");
                }
                Err(e) => {
                    notify(format!("Failed to rollback restore: {}", e), true);
                    log::error!("Rollback failed: {}", e);
                }
            }
        });
    }
}
