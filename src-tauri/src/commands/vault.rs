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

//! Vault domain Tauri commands.
//!
//! Covers: vault existence check, create, unlock, lock, password strength,
//! export, restore, and rollback.
//!
//! Error messages are sanitized to avoid leaking internal details to the frontend.

use sanctum::error::AppError;
use sanctum::features::settings::SettingsService;
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
pub fn create_vault(vault: State<'_, VaultManager>, password: String) -> Result<(), AppError> {
    vault.create_db(password, None).map(|_| ()).map_err(|e| {
        log::error!("Vault creation failed: {e}");
        // Sanitize the message but keep the kind so the frontend can react.
        AppError::new(AppError::from(e).kind, "Failed to create vault")
    })
}

/// Unlock an existing vault with the given master password.
#[tauri::command]
pub fn unlock_vault(vault: State<'_, VaultManager>, password: String) -> Result<(), AppError> {
    vault.open_db(password, None).map(|_| ()).map_err(|e| {
        log::error!("Vault unlock failed: {e}");
        AppError::new(
            AppError::from(e).kind,
            "Invalid password or vault not found",
        )
    })
}

/// Lock the currently open vault (close the DB connection).
#[tauri::command]
pub fn lock_vault(vault: State<'_, VaultManager>) -> Result<(), AppError> {
    vault.close_db().map(|_| ()).map_err(|e| {
        log::error!("Vault lock failed: {e}");
        AppError::new(AppError::from(e).kind, "Failed to lock vault")
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
pub fn export_vault(
    vault: State<'_, VaultManager>,
    settings: State<'_, SettingsService>,
    path: String,
) -> Result<(), AppError> {
    vault.export_vault(path).map_err(|e| {
        log::error!("Vault export failed: {e}");
        AppError::new(AppError::from(e).kind, "Vault export failed")
    })?;

    // Best effort: the backup is already on disk, so a failed stamp must not
    // report the export itself as failed.
    if let Err(e) = settings.record_backup_now() {
        log::warn!("Could not record the backup timestamp: {e}");
    }

    Ok(())
}

/// Change the master password, re-encrypting the vault.
///
/// Returns the path of the rollback copy written first, which keeps the OLD
/// password.
#[tauri::command]
pub fn change_vault_password(
    vault: State<'_, VaultManager>,
    current_password: String,
    new_password: String,
) -> Result<String, AppError> {
    vault
        .change_password(current_password, new_password)
        .map_err(|e| {
            log::error!("Password change failed: {e}");
            AppError::from(e)
        })
}

/// Restore a vault from a backup file.
#[tauri::command]
pub fn restore_vault(vault: State<'_, VaultManager>, backup_path: String) -> Result<(), AppError> {
    vault.restore_vault(backup_path).map_err(|e| {
        log::error!("Vault restore failed: {e}");
        AppError::new(AppError::from(e).kind, "Vault restore failed")
    })
}

/// Roll back the last vault restore operation.
#[tauri::command]
pub fn rollback_restore(vault: State<'_, VaultManager>) -> Result<(), AppError> {
    vault.rollback_restore().map_err(|e| {
        log::error!("Vault rollback failed: {e}");
        AppError::new(AppError::from(e).kind, "Rollback failed")
    })
}
