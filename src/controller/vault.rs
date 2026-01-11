use super::ControllerError;
use crate::services::vault::{self, VaultError};
use std::path::{Path, PathBuf};

impl From<VaultError> for ControllerError {
    fn from(err: VaultError) -> Self {
        match err {
            VaultError::FileNotFound => {
                ControllerError::Validation("Backup file not found".to_string())
            }
            VaultError::InvalidBackupFile => {
                ControllerError::Validation("Invalid backup file: not a SQLite database".to_string())
            }
            VaultError::BackupTooSmall => {
                ControllerError::Validation("Backup file is too small (minimum 1MB)".to_string())
            }
            VaultError::BackupTooLarge => {
                ControllerError::Validation("Backup file is too large (maximum 1GB)".to_string())
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
            VaultError::Io(e) => {
                ControllerError::Validation(format!("IO error: {}", e))
            }
        }
    }
}

impl super::AppController {
    /// Export current vault to a backup location
    ///
    /// This copies the encrypted database file to the specified destination
    /// without decrypting it. The vault is temporarily closed during the operation.
    ///
    /// # Arguments
    /// * `destination` - Path where the backup will be saved
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ControllerError)` if export fails
    pub fn export_vault(&self, destination: String) -> Result<(), ControllerError> {
        // Get the current vault path
        let vault_path = self.default_db_path();

        if !vault_path.exists() {
            return Err(ControllerError::VaultNotFound);
        }

        // Validate and sanitize destination path
        let dest_path = PathBuf::from(destination);

        // Close vault temporarily to ensure no corruption
        let was_open = {
            let db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;
            db_lock.is_some()
        };

        if was_open {
            let _ = self.close_db();
        }

        // Perform export
        let result = vault::export_vault(&vault_path, &dest_path);

        // Note: We don't reopen the vault after export
        // User can continue their session after export

        result.map_err(ControllerError::from)
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
