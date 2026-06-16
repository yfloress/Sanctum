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

use super::ControllerError;
use crate::services::vault::{self, VaultError};
use std::path::PathBuf;
use std::path::Component;

impl From<VaultError> for ControllerError {
    fn from(err: VaultError) -> Self {
        match err {
            VaultError::FileNotFound => {
                ControllerError::Validation("Backup file not found".to_string())
            }
            VaultError::InvalidBackupFile => {
                ControllerError::Validation("Invalid backup file".to_string())
            }
            VaultError::BackupTooLarge => {
                ControllerError::Validation("Backup file is too large (maximum 1GB)".to_string())
            }
            VaultError::BackupEmpty => {
                ControllerError::Validation("Backup file is empty".to_string())
            }
            VaultError::PermissionDenied => {
                ControllerError::Validation("Permission denied accessing file".to_string())
            }
            VaultError::InsufficientDiskSpace => {
                ControllerError::Validation("Insufficient disk space".to_string())
            }
            VaultError::FileExists => {
                ControllerError::Validation("File already exists".to_string())
            }
            VaultError::Io(e) => ControllerError::Validation(format!("IO error: {}", e)),
        }
    }
}

impl super::AppController {
    fn sanitize_export_path(&self, raw: &str) -> Result<PathBuf, ControllerError> {
        let base = self.app_data_base()?;

        let raw_trimmed = raw.trim();
        if raw_trimmed.is_empty() {
            return Err(ControllerError::Validation(
                "Export path cannot be empty".to_string(),
            ));
        }

        let candidate = PathBuf::from(raw_trimmed);

        // If an absolute path is provided, ensure it resides within app_data_dir
        let relative = if candidate.is_absolute() {
            candidate
                .strip_prefix(&base)
                .map_err(|_| {
                    ControllerError::Validation(
                        "Export path must stay inside the app data directory".to_string(),
                    )
                })?
                .to_path_buf()
        } else {
            candidate
        };

        // Normalize the path while preventing traversal outside of base
        let mut normalized = base.clone();
        for comp in relative.components() {
            match comp {
                Component::Prefix(_) | Component::RootDir => {
                    return Err(ControllerError::Validation(
                        "Export path must stay inside the app data directory".to_string(),
                    ));
                }
                Component::ParentDir => {
                    if !normalized.pop() || !normalized.starts_with(&base) {
                        return Err(ControllerError::Validation(
                            "Export path must stay inside the app data directory".to_string(),
                        ));
                    }
                }
                Component::CurDir => {}
                Component::Normal(c) => normalized.push(c),
            }
        }

        Ok(normalized)
    }

    /// Export current vault to a backup location
    ///
    /// This copies the encrypted database file to the specified destination
    /// without decrypting it. A WAL checkpoint is performed first to ensure
    /// all pending changes are included in the backup.
    ///
    /// # Arguments
    /// * `destination` - Path where the backup will be saved
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ControllerError)` if export fails
    pub fn export_vault(&self, destination: String) -> Result<(), ControllerError> {
        let vault_path = self.default_db_path();

        if !vault_path.exists() {
            return Err(ControllerError::VaultNotFound);
        }

        // Force WAL checkpoint to ensure all changes are in the main db file
        self.with_db_no_touch(|db| db.checkpoint().map_err(ControllerError::Database))?;

        // Sanitize destination path to prevent path traversal
        let dest_path = self.sanitize_export_path(&destination)?;

        vault::export_vault(&vault_path, &dest_path).map_err(ControllerError::from)
    }

    /// Restore vault from a backup file
    ///
    /// This replaces the current vault with the backup file. A pre-restore
    /// backup is created automatically. The vault is closed and the user
    /// must log in again with the backup's password.
    ///
    /// # Arguments
    /// * `backup_path` - Path to the backup file
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ControllerError)` if restore fails
    pub fn restore_vault(&self, backup_path: String) -> Result<(), ControllerError> {
        // Validate backup path
        let backup = PathBuf::from(backup_path);

        // Validate the backup file first (before closing vault)
        vault::validate_backup_file(&backup)?;

        // Close current vault if open
        let _ = self.close_db();

        // Get destination path (current vault location)
        let vault_path = self.default_db_path();

        // Import vault (creates pre-restore backup automatically)
        vault::import_vault(&backup, &vault_path)?;

        // User must now log in with the backup's password
        Ok(())
    }

    /// Check if a pre-restore backup exists
    pub fn has_pre_restore_backup(&self) -> bool {
        let vault_path = self.default_db_path();
        let pre_restore = vault_path.with_extension("db.pre-restore");
        pre_restore.exists()
    }

    /// Rollback to pre-restore backup
    ///
    /// This restores the vault to the state before the last restore operation.
    /// Useful if the user cannot log in with the restored vault's password.
    pub fn rollback_restore(&self) -> Result<(), ControllerError> {
        // Close current vault if open
        let _ = self.close_db();

        let vault_path = self.default_db_path();

        vault::rollback_restore(&vault_path)?;

        Ok(())
    }

    /// Clean up pre-restore backup after successful login
    pub fn cleanup_pre_restore_backup(&self) -> Result<(), ControllerError> {
        let vault_path = self.default_db_path();

        vault::cleanup_pre_restore_backup(&vault_path)?;

        Ok(())
    }
}
