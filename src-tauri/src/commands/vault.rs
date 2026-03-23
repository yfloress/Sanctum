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

//! Vault domain Tauri commands.
//!
//! Covers: vault existence check, create, unlock, lock, password strength,
//! export, restore, and rollback.

use sanctum::controller::AppController;
use sanctum::ui::dto::vault::{PasswordStrengthResult, VaultExportResult, VaultStatus};
use std::sync::Arc;
use tauri::State;

/// Check whether a vault file exists on disk.
#[tauri::command]
pub fn check_vault_exists(controller: State<'_, Arc<AppController>>) -> VaultStatus {
    VaultStatus {
        exists: controller.check_vault_exists(),
    }
}

/// Create a new vault with the given master password.
///
/// Returns `Ok(())` on success or an error message string.
#[tauri::command]
pub fn create_vault(
    controller: State<'_, Arc<AppController>>,
    password: String,
) -> Result<(), String> {
    controller
        .create_db(password, None)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Unlock an existing vault with the given master password.
///
/// Returns `Ok(())` on success or an error message string.
#[tauri::command]
pub fn unlock_vault(
    controller: State<'_, Arc<AppController>>,
    password: String,
) -> Result<(), String> {
    controller
        .open_db(password, None)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Lock the currently open vault (close the DB connection).
///
/// Returns `Ok(())` on success or an error message string.
#[tauri::command]
pub fn lock_vault(controller: State<'_, Arc<AppController>>) -> Result<(), String> {
    controller.close_db().map(|_| ()).map_err(|e| e.to_string())
}

/// Check password strength. Returns a warning message if weak, empty if strong.
#[tauri::command]
pub fn check_password_strength(
    controller: State<'_, Arc<AppController>>,
    password: String,
) -> PasswordStrengthResult {
    PasswordStrengthResult {
        warning: controller.check_password_strength(password),
    }
}

/// Export the current vault to a backup file at the given path.
///
/// The frontend is responsible for showing the file save dialog and passing
/// the selected path here via `tauri-plugin-dialog`.
#[tauri::command]
pub fn export_vault(
    controller: State<'_, Arc<AppController>>,
    path: String,
) -> Result<VaultExportResult, String> {
    controller
        .export_vault(path.clone())
        .map(|_| VaultExportResult { path })
        .map_err(|e| e.to_string())
}

/// Restore a vault from a backup file.
///
/// The frontend is responsible for showing the file picker and passing
/// the selected path here via `tauri-plugin-dialog`.
/// After restore, the user must log in with the backup's password.
#[tauri::command]
pub fn restore_vault(
    controller: State<'_, Arc<AppController>>,
    backup_path: String,
) -> Result<(), String> {
    controller
        .restore_vault(backup_path)
        .map_err(|e| e.to_string())
}

/// Roll back the last vault restore operation.
///
/// Restores the vault to the state before the last restore. Useful if the
/// user cannot log in with the restored vault's password.
#[tauri::command]
pub fn rollback_restore(controller: State<'_, Arc<AppController>>) -> Result<(), String> {
    controller.rollback_restore().map_err(|e| e.to_string())
}
