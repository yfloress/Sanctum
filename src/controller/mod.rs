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

//! Application Controller for Sanctum
//!
//! This module provides a pure Rust API that can be consumed by any UI framework (Slint, etc.)
//! Split into domain-specific submodules for maintainability.

mod crypto;
mod finance;
mod habits;
mod ingestion;
mod rewards;
mod settings;
mod vault;

use crate::db::{Database, DbError};
use crate::features::crypto::{CryptoError, CryptoService};
pub use crate::features::crypto::{
    SETTING_AUTO_FETCH, SETTING_CRYPTO_CUSTOM_COINS, SETTING_CRYPTO_FAVORITE_COINS,
    SETTING_CRYPTO_HIDDEN_COINS, SETTING_CRYPTO_LAST_COIN_ID, SETTING_CRYPTO_LAST_UPDATED,
    SETTING_CRYPTO_LAST_WALLET_ID, SETTING_CRYPTO_PROXY_ENABLED, SETTING_CRYPTO_PROXY_URL,
    SETTING_DARK_MODE, SETTING_PREFERRED_CURRENCY, SETTING_PREFERRED_LANGUAGE,
    SETTING_SESSION_TIMEOUT, SETTING_SIDEBAR_COLLAPSED, SETTING_TICKER_COINS,
};
pub use crate::features::finance::{DashboardData, ExpenseSlice};
use crate::features::finance::{FinanceError, FinanceService};
use crate::features::habits::{HabitService, RewardsService};
use crate::features::ingestion::IngestionService;
use crate::security_log::{SecurityEvent, log_auth_failure, log_security_event};
use crate::services::charts::ChartsService;
use rusqlite::Connection;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use slint::Image;
use std::fs::{self, Permissions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ==================== Error Types ====================

/// Errors that can occur in the controller layer
#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error")]
    Internal,

    #[error("No vault is currently open")]
    NoVaultOpen,

    #[error("A vault is already open. Close it first.")]
    VaultAlreadyOpen,

    #[error("Session expired due to inactivity. Please unlock the vault again.")]
    SessionExpired,

    #[error("Too many failed attempts. Try again in {0} seconds")]
    RateLimited(u64),

    #[error("Vault already exists at this location. Use unlock instead.")]
    VaultExists,

    #[error("No vault found at the specified location")]
    VaultNotFound,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(String),
}

impl From<String> for ControllerError {
    fn from(s: String) -> Self {
        ControllerError::Validation(s)
    }
}

impl From<FinanceError> for ControllerError {
    fn from(err: FinanceError) -> Self {
        match err {
            FinanceError::Database(e) => ControllerError::Database(e),
            FinanceError::Validation(message) => ControllerError::Validation(message),
            FinanceError::Internal => ControllerError::Internal,
            FinanceError::NoVaultOpen => ControllerError::NoVaultOpen,
            FinanceError::SessionExpired => ControllerError::SessionExpired,
        }
    }
}

impl From<CryptoError> for ControllerError {
    fn from(err: CryptoError) -> Self {
        match err {
            CryptoError::Database(e) => ControllerError::Database(e),
            CryptoError::Validation(message) => ControllerError::Validation(message),
            CryptoError::Internal => ControllerError::Internal,
            CryptoError::NoVaultOpen => ControllerError::NoVaultOpen,
            CryptoError::SessionExpired => ControllerError::SessionExpired,
            CryptoError::Api(message) => ControllerError::Api(message),
        }
    }
}

// ==================== Habit Analytics Types ====================

#[derive(Debug, Clone)]
pub struct WeekdayEfficiency {
    pub day_name: String,
    pub day_short: String,
    pub avg_count: f32,
    pub is_best: bool,
    pub bar_height_percent: f32,
}

#[derive(Debug, Clone)]
pub struct MonthlyTrendPoint {
    pub month_name: String,
    pub avg_per_day: f32,
    pub x_percent: f32,
    pub y_percent: f32,
}

#[derive(Debug, Clone)]
pub struct HabitAnalytics {
    pub weekday_data: Vec<WeekdayEfficiency>,
    pub monthly_data: Vec<MonthlyTrendPoint>,
    pub monthly_path: String,
}

// ==================== Security: Field Length Limits ====================
const MAX_PASSWORD_LENGTH: usize = 128;
const MIN_PASSWORD_LENGTH: usize = 8;
const PASSWORD_PASSPHRASE_LENGTH: usize = 16;

// ==================== Helper Functions ====================

/// Validates a UUID string format
fn validate_uuid(id: &str) -> Result<String, ControllerError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(ControllerError::Validation(
            "ID cannot be empty".to_string(),
        ));
    }

    // Check if it's a valid UUID
    if Uuid::parse_str(trimmed).is_ok() {
        return Ok(trimmed.to_string());
    }

    Err(ControllerError::Validation("Invalid ID format".to_string()))
}

fn normalize_habit_category(category: &str) -> Option<String> {
    let normalized = category.trim().to_lowercase();
    match normalized.as_str() {
        "mind" => Some("mind".to_string()),
        "body" => Some("body".to_string()),
        "spirit" | "discipline" => Some("spirit".to_string()),
        _ => None,
    }
}

fn ensure_default_settings(db: &Database) -> Result<(), ControllerError> {
    let defaults = [
        (SETTING_PREFERRED_CURRENCY, "USD"),
        (SETTING_PREFERRED_LANGUAGE, "en"),
        (SETTING_SESSION_TIMEOUT, "900"),
        (SETTING_DARK_MODE, "true"),
        (SETTING_AUTO_FETCH, "false"),
        (SETTING_CRYPTO_PROXY_ENABLED, "false"),
        (SETTING_CRYPTO_PROXY_URL, ""),
        (SETTING_SIDEBAR_COLLAPSED, "false"),
    ];

    for (key, value) in defaults {
        let current = db
            .get_setting(key)
            .map_err(ControllerError::Database)?
            .unwrap_or_default();
        if current.trim().is_empty() {
            db.set_setting(key, value)
                .map_err(ControllerError::Database)?;
        }
    }

    Ok(())
}

/// Validates basic password for opening an existing vault
/// Only verifies it's not empty and doesn't exceed limit
fn validate_password_basic(password: String) -> Result<SecretString, ControllerError> {
    let trimmed = password.trim();

    if trimmed.is_empty() {
        return Err(ControllerError::Validation(
            "Password cannot be empty".to_string(),
        ));
    }

    if trimmed.len() > MAX_PASSWORD_LENGTH {
        return Err(ControllerError::Validation(format!(
            "Password cannot exceed {} characters",
            MAX_PASSWORD_LENGTH
        )));
    }

    // Create SecretString that clears memory automatically
    Ok(SecretString::from(trimmed.to_string()))
}

/// Returns a warning message if the password is weak (empty string means ok)
fn password_strength_warning(password: &str) -> Option<String> {
    let trimmed = password.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.len() >= PASSWORD_PASSPHRASE_LENGTH {
        return None;
    }

    let has_uppercase = trimmed.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = trimmed.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
    let has_special = trimmed.chars().any(|c| {
        matches!(
            c,
            '!' | '@'
                | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '('
                | ')'
                | '-'
                | '_'
                | '='
                | '+'
                | '['
                | ']'
                | '{'
                | '}'
                | '|'
                | ';'
                | ':'
                | '\''
                | '"'
                | ','
                | '.'
                | '<'
                | '>'
                | '?'
                | '/'
                | '`'
                | '~'
        )
    });

    if trimmed.len() < MIN_PASSWORD_LENGTH
        || !has_uppercase
        || !has_lowercase
        || !has_digit
        || !has_special
    {
        return Some("Weak password: use 16+ chars or add variety".to_string());
    }

    None
}

// ==================== Configuration ====================

#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    last_db_rel_path: Option<String>,
}

pub struct AppController {
    db: Arc<Mutex<Option<Database>>>,
    pub finance_service: FinanceService,
    charts_service: ChartsService,
    crypto_service: CryptoService,
    pub habit_service: HabitService,
    pub rewards_service: RewardsService,
    pub ingestion_service: IngestionService,
    app_data_dir: PathBuf,
}

impl AppController {
    pub fn new(data_dir: PathBuf) -> Self {
        // Initialize with None as vault is locked
        let db = Arc::new(Mutex::new(None));
        let finance_service = FinanceService::new(db.clone());
        let crypto_service = CryptoService::new(db.clone());
        let habit_service = HabitService::new(db.clone());
        let rewards_service = RewardsService::new(db.clone());
        let ingestion_service = IngestionService::new(db.clone());
        let charts_service = ChartsService::new();

        Self {
            db,
            finance_service,
            charts_service,
            crypto_service,
            habit_service,
            rewards_service,
            ingestion_service,
            app_data_dir: data_dir,
        }
    }

    /// Returns the default database path
    pub fn default_db_path(&self) -> PathBuf {
        self.app_data_dir.join("sanctum.db")
    }

    /// Returns the config file path
    fn config_path(&self) -> PathBuf {
        self.app_data_dir.join("config.toml")
    }

    fn app_data_base(&self) -> Result<PathBuf, ControllerError> {
        fs::create_dir_all(&self.app_data_dir).map_err(|_| {
            ControllerError::Config("Could not access application data directory".to_string())
        })?;

        Ok(self
            .app_data_dir
            .canonicalize()
            .unwrap_or(self.app_data_dir.clone()))
    }

    fn normalize_config_path(&self, raw: &str) -> Option<String> {
        let normalized = self.sanitize_db_path(raw).ok()?;
        let base = self.app_data_base().ok()?;
        normalized
            .strip_prefix(&base)
            .ok()
            .map(|rel| rel.to_string_lossy().to_string())
    }

    fn normalize_config(&self, config: &mut AppConfig) -> bool {
        let mut changed = false;
        if let Some(raw) = config.last_db_rel_path.clone() {
            match self.normalize_config_path(&raw) {
                Some(rel) if rel != raw => {
                    config.last_db_rel_path = Some(rel);
                    changed = true;
                }
                Some(_) => {}
                None => {
                    config.last_db_rel_path = None;
                    changed = true;
                }
            }
        }
        changed
    }

    fn resolve_config_path(&self, stored: &str) -> Option<PathBuf> {
        self.sanitize_db_path(stored).ok()
    }

    fn vault_metadata_key(&self, db_path: &Path) -> String {
        let base = self.app_data_base().unwrap_or(self.app_data_dir.clone());
        if let Ok(rel) = db_path.strip_prefix(&base) {
            return rel.to_string_lossy().to_string();
        }
        db_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| db_path.to_string_lossy().to_string())
    }

    /// Loads the application configuration
    fn load_config(&self) -> Result<AppConfig, ControllerError> {
        let path = self.config_path();
        if path.exists() {
            let data = fs::read_to_string(&path)
                .map_err(|_| ControllerError::Config("Could not read configuration".to_string()))?;
            let mut config: AppConfig = toml::from_str(&data).map_err(|_| {
                ControllerError::Config("Could not parse configuration".to_string())
            })?;
            if self.normalize_config(&mut config) {
                let _ = self.save_config(&config);
            }
            return Ok(config);
        }

        Ok(AppConfig::default())
    }

    /// Saves the application configuration
    fn save_config(&self, config: &AppConfig) -> Result<(), ControllerError> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| {
                ControllerError::Config("Could not create configuration directory".to_string())
            })?;
        }

        let data = toml::to_string_pretty(config).map_err(|_| {
            ControllerError::Config("Could not serialize configuration".to_string())
        })?;

        fs::write(&path, &data)
            .map_err(|_| ControllerError::Config("Could not save configuration".to_string()))?;

        // Set restrictive permissions (owner read/write only - 0600)
        #[cfg(unix)]
        {
            fs::set_permissions(&path, Permissions::from_mode(0o600)).map_err(|_| {
                ControllerError::Config("Could not set configuration file permissions".to_string())
            })?;
        }

        Ok(())
    }

    /// Persists the last used database path
    fn persist_last_db_path(&self, path: &Path) -> Result<(), ControllerError> {
        let mut config = self.load_config()?;
        config.last_db_rel_path =
            self.normalize_config_path(&path.to_string_lossy())
                .or_else(|| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().to_string())
                });
        self.save_config(&config)
    }

    /// Opens (or creates) the persistent rate-limit store with restrictive permissions
    fn open_rate_limit_conn(&self, rate_limit_path: &Path) -> Result<Connection, ControllerError> {
        if let Some(parent) = rate_limit_path.parent() {
            fs::create_dir_all(parent).map_err(|_| {
                ControllerError::Config("Could not create rate limit directory".to_string())
            })?;
        }

        let conn = Connection::open(rate_limit_path)
            .map_err(|_| ControllerError::Config("Could not open rate limit store".to_string()))?;

        // Restrict permissions to owner read/write only
        #[cfg(unix)]
        fs::set_permissions(rate_limit_path, Permissions::from_mode(0o600)).map_err(|_| {
            ControllerError::Config("Could not set rate limit file permissions".to_string())
        })?;

        Ok(conn)
    }

    /// Sanitizes the requested vault path to ensure it stays inside the app data directory
    fn sanitize_db_path(&self, raw: &str) -> Result<PathBuf, ControllerError> {
        let base = self.app_data_base()?;

        let raw_trimmed = raw.trim();
        if raw_trimmed.is_empty() {
            return Err(ControllerError::Validation(
                "Vault path cannot be empty".to_string(),
            ));
        }

        let candidate = PathBuf::from(raw_trimmed);

        // If an absolute path is provided, ensure it resides within app_data_dir
        let relative = if candidate.is_absolute() {
            candidate
                .strip_prefix(&base)
                .map_err(|_| {
                    ControllerError::Validation(
                        "Vault path must stay inside the app data directory".to_string(),
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
                        "Vault path must stay inside the app data directory".to_string(),
                    ));
                }
                Component::ParentDir => {
                    if !normalized.pop() || !normalized.starts_with(&base) {
                        return Err(ControllerError::Validation(
                            "Vault path must stay inside the app data directory".to_string(),
                        ));
                    }
                }
                Component::CurDir => {}
                Component::Normal(c) => normalized.push(c),
            }
        }

        Ok(normalized)
    }

    /// Checks persistent rate limit using a temporary connection
    fn check_persistent_rate_limit(&self, db_path: &Path) -> Result<(), ControllerError> {
        if !db_path.exists() {
            return Ok(());
        }

        // Try to open without encryption to check rate limit table
        // This uses a separate unencrypted DB for rate limiting
        let rate_limit_path = db_path.with_extension("ratelimit");

        if let Ok(conn) = self.open_rate_limit_conn(&rate_limit_path) {
            let vault_key = self.vault_metadata_key(db_path);
            if let Err(DbError::RateLimited) = Database::check_rate_limit(&conn, &vault_key) {
                let remaining = Database::get_lockout_remaining(&conn, &vault_key).unwrap_or(0);
                return Err(ControllerError::RateLimited(remaining));
            }
        }

        Ok(())
    }

    /// Records a failed attempt in persistent storage
    fn record_persistent_failed_attempt(&self, db_path: &Path) {
        let rate_limit_path = db_path.with_extension("ratelimit");

        if let Ok(conn) = self.open_rate_limit_conn(&rate_limit_path) {
            let vault_key = self.vault_metadata_key(db_path);
            if let Ok((attempts, locked)) = Database::record_failed_attempt(&conn, &vault_key) {
                log_auth_failure(attempts, locked);
            }
        }
    }

    /// Resets persistent rate limit after successful auth
    fn reset_persistent_rate_limit(&self, db_path: &Path) {
        let rate_limit_path = db_path.with_extension("ratelimit");

        if let Ok(conn) = self.open_rate_limit_conn(&rate_limit_path) {
            let vault_key = self.vault_metadata_key(db_path);
            let _ = Database::reset_rate_limit(&conn, &vault_key);
        }
    }

    /// Helper to ensure no connection is currently open
    fn ensure_no_connection(&self) -> Result<(), ControllerError> {
        let db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;

        if db_lock.is_some() {
            return Err(ControllerError::VaultAlreadyOpen);
        }

        Ok(())
    }

    /// Helper that does not refresh session activity (for passive checks like countdown timers)
    fn with_db_no_touch<T, F>(&self, f: F) -> Result<T, ControllerError>
    where
        F: FnOnce(&Database) -> Result<T, ControllerError>,
    {
        let db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;
        let db = db_lock.as_ref().ok_or(ControllerError::NoVaultOpen)?;

        // Only validate expiration; do not extend activity timestamp
        db.check_session_timeout().map_err(|e| match e {
            DbError::SessionExpired => ControllerError::SessionExpired,
            _ => ControllerError::Database(e),
        })?;

        f(db)
    }

    // ==================== Database Management ====================

    /// Returns true if the database is initialized, false otherwise
    pub fn is_db_initialized(&self) -> bool {
        self.db.lock().map(|guard| guard.is_some()).unwrap_or(false)
    }

    /// Creates a new vault at the specified path
    pub fn create_db(
        &self,
        password: String,
        path: Option<String>,
    ) -> Result<String, ControllerError> {
        self.ensure_no_connection()?;

        // Determine and sanitize path
        let db_path = if let Some(raw) = path {
            self.sanitize_db_path(&raw)?
        } else {
            self.default_db_path()
        };

        // Prevent overwriting an existing vault
        if db_path.exists() {
            return Err(ControllerError::VaultExists);
        }

        // Validate password (for creation, stricter rules can be applied)
        let secret = validate_password_basic(password)?;

        // Initialize vault
        let mut db = Database::init(db_path.clone(), &secret).map_err(ControllerError::Database)?;

        ensure_default_settings(&db)?;

        // Apply configured session timeout (default 15 min)
        let timeout = db
            .get_setting(SETTING_SESSION_TIMEOUT)
            .map_err(ControllerError::Database)?
            .unwrap_or_else(|| "900".to_string())
            .parse::<i64>()
            .unwrap_or(900); // Default to 15 minutes
        db.set_session_timeout(timeout);

        // Store reference
        {
            let mut db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;
            *db_lock = Some(db);
        }

        // Persist last used path
        let _ = self.persist_last_db_path(&db_path);

        log_security_event(SecurityEvent::VaultCreated, None);

        Ok(db_path.to_string_lossy().to_string())
    }

    /// Checks the password strength and returns a warning message if weak
    pub fn check_password_strength(&self, password: String) -> String {
        password_strength_warning(&password).unwrap_or_default()
    }

    /// Opens an existing vault
    pub fn open_db(
        &self,
        password: String,
        path: Option<String>,
    ) -> Result<String, ControllerError> {
        self.ensure_no_connection()?;

        // Determine and sanitize path
        let db_path = if let Some(raw) = path {
            self.sanitize_db_path(&raw)?
        } else {
            self.default_db_path()
        };

        // Ensure vault exists
        if !db_path.exists() {
            return Err(ControllerError::VaultNotFound);
        }

        // Check rate limit before attempting
        self.check_persistent_rate_limit(&db_path)?;

        // Validate password
        let secret = validate_password_basic(password)?;

        // Attempt to open
        match Database::init(db_path.clone(), &secret) {
            Ok(mut db) => {
                // Success - reset rate limit
                self.reset_persistent_rate_limit(&db_path);

                ensure_default_settings(&db)?;

                // Apply configured session timeout (default 15 min)
                let timeout = db
                    .get_setting(SETTING_SESSION_TIMEOUT)
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(900); // Default to 15 minutes
                db.set_session_timeout(timeout);

                // Store reference
                {
                    let mut db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;
                    *db_lock = Some(db);
                }

                // Persist last used path
                let _ = self.persist_last_db_path(&db_path);

                log_security_event(SecurityEvent::VaultOpened, None);

                Ok(db_path.to_string_lossy().to_string())
            }
            Err(DbError::InvalidPassword) => {
                // Record failed attempt
                self.record_persistent_failed_attempt(&db_path);
                Err(ControllerError::Database(DbError::InvalidPassword))
            }
            Err(e) => Err(ControllerError::Database(e)),
        }
    }

    /// Closes the current vault
    pub fn close_db(&self) -> Result<String, ControllerError> {
        let mut db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;

        if db_lock.is_none() {
            return Err(ControllerError::NoVaultOpen);
        }

        *db_lock = None;

        log_security_event(SecurityEvent::VaultClosed, None);

        Ok("Vault closed successfully".to_string())
    }

    /// Gets the remaining session time in seconds
    pub fn get_session_remaining(&self) -> Result<i64, ControllerError> {
        self.with_db_no_touch(|db| {
            db.get_session_remaining()
                .map_err(ControllerError::Database)
        })
    }

    /// Sets the session timeout duration (in seconds)
    pub fn set_session_timeout(&self, timeout_secs: i64) -> Result<(), ControllerError> {
        // Save to settings (will be applied on next vault open or can be applied now if vault is open)
        self.set_app_setting(SETTING_SESSION_TIMEOUT, &timeout_secs.to_string())?;
        Ok(())
    }

    // ==================== Chart Rendering ====================

    pub fn render_habit_radar_chart(&self, data: &[(String, String, f32)]) -> Option<Image> {
        self.charts_service.render_habit_radar_chart(data)
    }

    pub fn render_weekday_efficiency_chart(&self, data: &[(String, f32, bool)]) -> Option<Image> {
        self.charts_service.render_weekday_efficiency_chart(data)
    }

    pub fn render_portfolio_distribution_chart(&self, data: &[(String, f64)]) -> Option<Image> {
        self.charts_service
            .render_portfolio_distribution_chart(data)
    }

    pub fn render_portfolio_trend_chart(&self, data: &[(String, f64, f64)]) -> Option<Image> {
        self.charts_service.render_portfolio_trend_chart(data)
    }

    pub fn chart_color_for_symbol(&self, symbol: &str, index: usize) -> (u8, u8, u8) {
        self.charts_service.chart_color_for_symbol(symbol, index)
    }

    pub fn render_net_worth_chart(&self, values: &[i64]) -> Option<Image> {
        self.charts_service.render_net_worth_chart(values)
    }

    // ==================== Vault Path Methods ====================

    /// Checks if a vault file exists
    pub fn check_vault_exists(&self) -> bool {
        // Check if custom path was used previously
        if let Ok(config) = self.load_config()
            && let Some(last_path) = config.last_db_rel_path
            && let Some(path) = self.resolve_config_path(&last_path)
            && path.exists()
        {
            return true;
        }

        // Check default path
        self.default_db_path().exists()
    }

    /// Returns the current vault path
    pub fn get_db_path(&self) -> Result<String, ControllerError> {
        // If there's an active connection, use that path
        {
            let db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;
            if let Some(db) = db_lock.as_ref() {
                return Ok(db.path().to_string_lossy().to_string());
            }
        }

        // Otherwise, return the last used path or default
        if let Ok(config) = self.load_config()
            && let Some(last) = config.last_db_rel_path
            && let Some(path) = self.resolve_config_path(&last)
        {
            return Ok(path.to_string_lossy().to_string());
        }

        Ok(self.default_db_path().to_string_lossy().to_string())
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        assert!(validate_uuid("").is_err());
        assert!(validate_uuid("   ").is_err());
        assert!(validate_uuid("not-a-uuid").is_err());
    }

    #[test]
    fn test_validate_password_basic_empty() {
        let result = validate_password_basic("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_basic_valid() {
        let result = validate_password_basic("simple".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().expose_secret(), "simple");
    }
}
