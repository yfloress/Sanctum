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

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

// Note: SQLCipher encrypts the entire database file, including the header.
// We cannot validate using SQLite magic bytes because they are encrypted.
// Validation is limited to file existence and size checks.

/// Maximum reasonable vault file size (1GB)
const MAX_VAULT_SIZE: u64 = 1_000_000_000;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("File not found")]
    FileNotFound,

    #[error("Invalid backup file")]
    InvalidBackupFile,

    #[error("Backup file is too large (maximum 1GB)")]
    BackupTooLarge,

    #[error("Backup file is empty")]
    BackupEmpty,

    #[error("Permission denied accessing file")]
    PermissionDenied,

    #[error("Insufficient disk space")]
    InsufficientDiskSpace,

    #[error("File already exists")]
    FileExists,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Validate that a file exists and has reasonable size for import
///
/// Note: SQLCipher encrypts the entire database including the header,
/// so we cannot validate using SQLite magic bytes. The actual validation
/// of whether it's a valid encrypted database happens at login time.
pub fn validate_backup_file(path: &Path) -> Result<(), VaultError> {
    // 1. Check file exists
    if !path.exists() {
        return Err(VaultError::FileNotFound);
    }

    // 2. Check permissions and get metadata
    let metadata = fs::metadata(path).map_err(|_| VaultError::PermissionDenied)?;

    // 3. Check file size
    let file_size = metadata.len();
    if file_size == 0 {
        return Err(VaultError::BackupEmpty);
    }
    if file_size > MAX_VAULT_SIZE {
        return Err(VaultError::BackupTooLarge);
    }

    Ok(())
}

/// Export vault to a backup location
///
/// Copies the encrypted database file without any decryption.
/// The backup maintains full encryption with the original password.
pub fn export_vault(source: &Path, destination: &Path) -> Result<(), VaultError> {
    // 1. Validate source exists
    if !source.exists() {
        return Err(VaultError::FileNotFound);
    }

    // 2. Check source is not empty
    let source_size = fs::metadata(source)?.len();
    if source_size == 0 {
        return Err(VaultError::BackupEmpty);
    }

    // 3. Check if destination already exists (should be handled by file dialog)
    if destination.exists() {
        log::warn!("Destination file already exists, will overwrite");
    }

    // 4. Copy file (encrypted, no decryption)
    fs::copy(source, destination)?;

    // 5. Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(destination, fs::Permissions::from_mode(0o600))?;
    }

    // 6. Verify copy integrity
    let dest_size = fs::metadata(destination)?.len();
    if source_size != dest_size {
        // Cleanup failed copy
        let _ = fs::remove_file(destination);
        return Err(VaultError::Io(std::io::Error::other(
            "Copy integrity check failed",
        )));
    }

    log::info!("Vault exported successfully to {:?}", destination);
    Ok(())
}

/// Create a pre-restore backup of the current vault
pub fn create_backup_copy(path: &Path) -> Result<PathBuf, VaultError> {
    if !path.exists() {
        return Err(VaultError::FileNotFound);
    }

    let backup_path = path.with_extension("db.pre-restore");

    // If a pre-restore backup already exists, remove it
    if backup_path.exists() {
        fs::remove_file(&backup_path)?;
    }

    fs::copy(path, &backup_path)?;

    log::info!("Pre-restore backup created at {:?}", backup_path);
    Ok(backup_path)
}

/// Create a rollback copy before re-encrypting the vault
///
/// Written next to the vault rather than to a user-chosen path: this is the
/// file that gets the database back if the rekey is interrupted, so it must
/// not fail over a directory choice. It stays encrypted with the old password.
pub fn create_pre_rekey_backup(path: &Path) -> Result<PathBuf, VaultError> {
    if !path.exists() {
        return Err(VaultError::FileNotFound);
    }

    let backup_path = path.with_extension("db.pre-password-change");

    if backup_path.exists() {
        fs::remove_file(&backup_path)?;
    }

    fs::copy(path, &backup_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&backup_path, fs::Permissions::from_mode(0o600))?;
    }

    log::info!("Pre-rekey backup created at {:?}", backup_path);
    Ok(backup_path)
}

/// Import (restore) vault from a backup file
pub fn import_vault(backup: &Path, destination: &Path) -> Result<(), VaultError> {
    // 1. Validate backup file
    validate_backup_file(backup)?;

    // 2. Create pre-restore backup of current vault (if it exists)
    if destination.exists() {
        create_backup_copy(destination)?;
    }

    // 3. Replace vault file with backup
    fs::copy(backup, destination)?;

    // 4. Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(destination, fs::Permissions::from_mode(0o600))?;
    }

    // 5. Verify integrity
    let backup_size = fs::metadata(backup)?.len();
    let dest_size = fs::metadata(destination)?.len();
    if backup_size != dest_size {
        // Attempt rollback from pre-restore backup
        let pre_restore = destination.with_extension("db.pre-restore");
        if pre_restore.exists() {
            log::error!("Restore failed, attempting rollback");
            let _ = fs::copy(&pre_restore, destination);
        }
        return Err(VaultError::Io(std::io::Error::other(
            "Restore integrity check failed",
        )));
    }

    log::info!("Vault restored successfully from {:?}", backup);
    Ok(())
}

/// Rollback to pre-restore backup
pub fn rollback_restore(vault_path: &Path) -> Result<(), VaultError> {
    let pre_restore = vault_path.with_extension("db.pre-restore");

    if !pre_restore.exists() {
        return Err(VaultError::FileNotFound);
    }

    fs::copy(&pre_restore, vault_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(vault_path, fs::Permissions::from_mode(0o600))?;
    }

    log::info!("Rollback successful");
    Ok(())
}

/// Clean up pre-restore backup after successful login
pub fn cleanup_pre_restore_backup(vault_path: &Path) -> Result<(), VaultError> {
    let pre_restore = vault_path.with_extension("db.pre-restore");

    if pre_restore.exists() {
        fs::remove_file(&pre_restore)?;
        log::info!("Pre-restore backup cleaned up");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_validate_backup_file_exists() {
        let temp_dir = std::env::temp_dir();
        let non_existent = temp_dir.join("non_existent_test.db");

        // Test non-existent file
        assert!(matches!(
            validate_backup_file(&non_existent),
            Err(VaultError::FileNotFound)
        ));
    }

    #[test]
    fn test_validate_backup_file_empty() {
        let temp_dir = std::env::temp_dir();
        let empty_file = temp_dir.join("empty_test.db");

        // Create an empty file
        File::create(&empty_file).unwrap();

        // Test empty file
        assert!(matches!(
            validate_backup_file(&empty_file),
            Err(VaultError::BackupEmpty)
        ));

        // Cleanup
        let _ = fs::remove_file(empty_file);
    }

    #[test]
    fn test_validate_backup_file_valid() {
        let temp_dir = std::env::temp_dir();
        let valid_file = temp_dir.join("valid_test.db");

        // Create a file with some content (simulating encrypted database)
        let mut file = File::create(&valid_file).unwrap();
        file.write_all(b"encrypted content here").unwrap();
        drop(file);

        // Test valid file (any non-empty file passes validation)
        assert!(validate_backup_file(&valid_file).is_ok());

        // Cleanup
        let _ = fs::remove_file(valid_file);
    }
}
