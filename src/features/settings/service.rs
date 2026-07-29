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

//! Settings service
//!
//! Generic get/set over the encrypted `settings` table for application-level
//! keys. Shares the same `Arc<RwLock<Option<Database>>>` handle as every other
//! service, so it observes vault open/close transparently.

use crate::db::{Database, DbError};
use std::sync::{Arc, RwLock};

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Internal error")]
    Internal,

    #[error("No vault is currently open")]
    NoVaultOpen,

    #[error("Session expired due to inactivity. Please unlock the vault again.")]
    SessionExpired,
}

pub struct SettingsService {
    db: Arc<RwLock<Option<Database>>>,
}

impl SettingsService {
    pub fn new(db: Arc<RwLock<Option<Database>>>) -> Self {
        Self { db }
    }

    fn with_db<T, F>(&self, f: F) -> Result<T, SettingsError>
    where
        F: FnOnce(&Database) -> Result<T, SettingsError>,
    {
        let db_lock = self.db.read().map_err(|_| SettingsError::Internal)?;
        let db = db_lock.as_ref().ok_or(SettingsError::NoVaultOpen)?;

        db.check_session_timeout().map_err(|e| match e {
            DbError::SessionExpired => SettingsError::SessionExpired,
            _ => SettingsError::Database(e),
        })?;

        let result = f(db)?;
        db.touch_session().map_err(SettingsError::Database)?;
        Ok(result)
    }

    /// Reads an application setting, returning an empty string when unset.
    pub fn get_app_setting(&self, key: &str) -> Result<String, SettingsError> {
        self.with_db(|db| {
            let val = db.get_setting(key).map_err(SettingsError::Database)?;
            Ok(val.unwrap_or_default())
        })
    }

    /// Writes an application setting.
    pub fn set_app_setting(&self, key: &str, value: &str) -> Result<(), SettingsError> {
        self.with_db(|db| db.set_setting(key, value).map_err(SettingsError::Database))
    }
}
