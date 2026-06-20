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

//! Vault domain Tauri commands.
//!
//! Covers: vault existence check, create, unlock, lock, password strength,
//! export, restore, and rollback.
//!
//! Error messages are sanitized to avoid leaking internal details to the frontend.

use sanctum::ui::dto::vault::{PasswordStrengthResult, VaultStatus};
use sanctum::vault_manager::VaultManager;
use tauri::State;

/// Check whether a vault file exists on disk.
#[tauri::command]
pub fn check_vault_exists(vault: State<'_, VaultManager>) -> VaultStatus {
    VaultStatus {
        exists: vault.check_vault_exists(),
    }
}

/// Create a new vault with the given master password.
#[tauri::command]
pub fn create_vault(vault: State<'_, VaultManager>, password: String) -> Result<(), String> {
    vault.create_db(password, None).map(|_| ()).map_err(|e| {
        log::error!("Vault creation failed: {e}");
        "Failed to create vault".to_string()
    })
}

/// Unlock an existing vault with the given master password.
#[tauri::command]
pub fn unlock_vault(vault: State<'_, VaultManager>, password: String) -> Result<(), String> {
    vault.open_db(password, None).map(|_| ()).map_err(|e| {
        log::error!("Vault unlock failed: {e}");
        "Invalid password or vault not found".to_string()
    })
}

/// Lock the currently open vault (close the DB connection).
#[tauri::command]
pub fn lock_vault(vault: State<'_, VaultManager>) -> Result<(), String> {
    vault.close_db().map(|_| ()).map_err(|e| {
        log::error!("Vault lock failed: {e}");
        "Failed to lock vault".to_string()
    })
}

/// Check password strength. Returns a warning message if weak, empty if strong.
#[tauri::command]
pub fn check_password_strength(
    vault: State<'_, VaultManager>,
    password: String,
) -> PasswordStrengthResult {
    PasswordStrengthResult {
        warning: vault.check_password_strength(password),
    }
}

/// Export the current vault to a backup file at the given path.
#[tauri::command]
pub fn export_vault(vault: State<'_, VaultManager>, path: String) -> Result<(), String> {
    vault.export_vault(path).map(|_| ()).map_err(|e| {
        log::error!("Vault export failed: {e}");
        "Vault export failed".to_string()
    })
}

/// Restore a vault from a backup file.
#[tauri::command]
pub fn restore_vault(vault: State<'_, VaultManager>, backup_path: String) -> Result<(), String> {
    vault.restore_vault(backup_path).map_err(|e| {
        log::error!("Vault restore failed: {e}");
        "Vault restore failed".to_string()
    })
}

/// Roll back the last vault restore operation.
#[tauri::command]
pub fn rollback_restore(vault: State<'_, VaultManager>) -> Result<(), String> {
    vault.rollback_restore().map_err(|e| {
        log::error!("Vault rollback failed: {e}");
        "Rollback failed".to_string()
    })
}
