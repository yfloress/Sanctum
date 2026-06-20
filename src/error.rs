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

//! Application-level error for the Tauri command boundary.
//!
//! Domain services expose rich, domain-specific error enums (`FinanceError`,
//! `CryptoError`, …). Those stay internal. At the IPC boundary every command
//! returns [`AppError`], a serializable `{ kind, message }` payload so the
//! frontend can both display a human message and branch on a stable [`ErrorKind`]
//! (e.g. lock the vault on `session_expired`). The `From` impls below collapse
//! each domain error into a kind + message; the original message text is
//! preserved so the user-facing strings do not change.
//!
//! The struct is intentionally extensible: a `field: Option<String>` will be
//! added in the DTO/CQRS refactor (critique #4), where validation gains field
//! identity. Adding it is non-breaking thanks to `skip_serializing_if`.

use serde::Serialize;

use crate::features::crypto::CryptoError;
use crate::features::finance::FinanceError;
use crate::features::ingestion::IngestionError;
use crate::features::settings::SettingsError;
use crate::vault_manager::ControllerError;

/// Stable, serializable classification of an error at the IPC boundary.
///
/// Serialized in `snake_case` (e.g. `"session_expired"`) so the frontend can
/// match on it without depending on Rust naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Invalid user input.
    Validation,
    /// A requested resource does not exist.
    NotFound,
    /// The request conflicts with current state (e.g. vault already open).
    Conflict,
    /// No vault is currently open.
    NoVaultOpen,
    /// The session expired due to inactivity.
    SessionExpired,
    /// Too many attempts; the caller is being throttled.
    RateLimited,
    /// A network / upstream API call failed.
    Network,
    /// Failed to parse external data (CSV, etc.).
    Parse,
    /// The provided format is not supported.
    UnsupportedFormat,
    /// The provided file exceeds the allowed size.
    FileTooLarge,
    /// Configuration could not be read or written.
    Config,
    /// An unexpected internal error (storage, locking, …).
    Internal,
}

/// The error type every Tauri command returns.
///
/// Serializes to `{ "kind": "...", "message": "..." }`.
#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    /// Stable machine-readable classification.
    pub kind: ErrorKind,
    /// Human-readable message, safe to show to the user.
    pub message: String,
    // Reserved for critique #4 (DTO/CQRS validation), kept non-breaking:
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub field: Option<String>,
}

impl AppError {
    /// Build an error from an explicit kind and message.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Invalid user input.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Validation, message)
    }

    /// A requested resource does not exist.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    /// A generic, non-leaking internal error.
    pub fn internal() -> Self {
        Self::new(ErrorKind::Internal, "Internal error")
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

// ==================== Ad-hoc string errors (command layer) ====================

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::new(ErrorKind::Validation, s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::new(ErrorKind::Validation, s)
    }
}

// ==================== Domain error conversions ====================

impl From<FinanceError> for AppError {
    fn from(e: FinanceError) -> Self {
        let kind = match &e {
            FinanceError::Validation(_) => ErrorKind::Validation,
            FinanceError::NoVaultOpen => ErrorKind::NoVaultOpen,
            FinanceError::SessionExpired => ErrorKind::SessionExpired,
            FinanceError::Database(_) | FinanceError::Internal => ErrorKind::Internal,
        };
        Self::new(kind, e.to_string())
    }
}

impl From<CryptoError> for AppError {
    fn from(e: CryptoError) -> Self {
        let kind = match &e {
            CryptoError::Validation(_) => ErrorKind::Validation,
            CryptoError::NoVaultOpen => ErrorKind::NoVaultOpen,
            CryptoError::SessionExpired => ErrorKind::SessionExpired,
            CryptoError::Api(_) => ErrorKind::Network,
            CryptoError::Database(_) | CryptoError::Internal => ErrorKind::Internal,
        };
        Self::new(kind, e.to_string())
    }
}

impl From<SettingsError> for AppError {
    fn from(e: SettingsError) -> Self {
        let kind = match &e {
            SettingsError::NoVaultOpen => ErrorKind::NoVaultOpen,
            SettingsError::SessionExpired => ErrorKind::SessionExpired,
            SettingsError::Database(_) | SettingsError::Internal => ErrorKind::Internal,
        };
        Self::new(kind, e.to_string())
    }
}

impl From<IngestionError> for AppError {
    fn from(e: IngestionError) -> Self {
        let kind = match &e {
            IngestionError::Validation(_) => ErrorKind::Validation,
            IngestionError::Parse(_) => ErrorKind::Parse,
            IngestionError::UnsupportedFormat(_) => ErrorKind::UnsupportedFormat,
            IngestionError::FileTooLarge(_) => ErrorKind::FileTooLarge,
            IngestionError::NoVaultOpen => ErrorKind::NoVaultOpen,
            IngestionError::SessionExpired => ErrorKind::SessionExpired,
            IngestionError::Database(_) => ErrorKind::Internal,
        };
        Self::new(kind, e.to_string())
    }
}

impl From<ControllerError> for AppError {
    fn from(e: ControllerError) -> Self {
        let kind = match &e {
            ControllerError::Validation(_) => ErrorKind::Validation,
            ControllerError::NoVaultOpen => ErrorKind::NoVaultOpen,
            ControllerError::SessionExpired => ErrorKind::SessionExpired,
            ControllerError::RateLimited(_) => ErrorKind::RateLimited,
            ControllerError::VaultAlreadyOpen | ControllerError::VaultExists => ErrorKind::Conflict,
            ControllerError::VaultNotFound => ErrorKind::NotFound,
            ControllerError::Config(_) => ErrorKind::Config,
            ControllerError::Database(_) | ControllerError::Internal => ErrorKind::Internal,
        };
        Self::new(kind, e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finance_validation_maps_to_validation_kind() {
        let err: AppError = FinanceError::Validation("bad name".to_string()).into();
        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(err.message, "Validation error: bad name");
    }

    #[test]
    fn controller_rate_limited_maps_to_rate_limited_kind() {
        let err: AppError = ControllerError::RateLimited(30).into();
        assert_eq!(err.kind, ErrorKind::RateLimited);
        assert!(err.message.contains("30"));
    }

    #[test]
    fn controller_vault_exists_maps_to_conflict() {
        let err: AppError = ControllerError::VaultExists.into();
        assert_eq!(err.kind, ErrorKind::Conflict);
    }

    #[test]
    fn ingestion_file_too_large_maps_to_kind() {
        let err: AppError = IngestionError::FileTooLarge("5 MB".to_string()).into();
        assert_eq!(err.kind, ErrorKind::FileTooLarge);
    }

    #[test]
    fn ad_hoc_str_maps_to_validation() {
        let err: AppError = "Account not found".into();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn serializes_to_kind_and_message_object() {
        let err = AppError::not_found("missing");
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v["kind"], "not_found");
        assert_eq!(v["message"], "missing");
        // `field` is not present yet (reserved for #4).
        assert!(v.get("field").is_none());
    }
}
