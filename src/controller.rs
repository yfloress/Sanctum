//! Application Controller for Sanctum
//!
//! This module provides a pure Rust API that can be consumed by any UI framework (Slint, etc.)
//! All Tauri-specific code has been removed.

use crate::crypto;
use crate::db::{Database, DbError};
use crate::models::{
    Account, AccountBalance, AggregatedAsset, BalanceSummary, CryptoAsset, CryptoTransaction,
    CryptoWallet, Habit, HabitLog, Transaction,
};
use crate::security_log::{SecurityEvent, log_auth_failure, log_security_event};
use crate::services::habit::HabitService;
use chrono::{Datelike, NaiveDate, Utc};
use rusqlite::Connection;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::fs::{self, Permissions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use regex::Regex;

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

// ==================== Analytics Types ====================

#[derive(Debug, Clone)]
pub struct ExpenseSlice {
    pub category: String,
    pub amount: i64,
    pub percentage: f32,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct AnalyticsSummary {
    pub chart_path: String,
    pub net_worth: String,
    pub max_value: String,
    pub min_value: String,
    pub expense_slices: Vec<ExpenseSlice>,
}

// ==================== Security: Field Length Limits ====================
const MAX_CATEGORY_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 512;
const MAX_NOTES_LENGTH: usize = 1024;
const MAX_WALLET_NAME_LENGTH: usize = 128;
const MAX_SYMBOL_LENGTH: usize = 16;
const MAX_ICON_LENGTH: usize = 32;
const MAX_PASSWORD_LENGTH: usize = 128;
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_ACCOUNT_NAME_LENGTH: usize = 64;
const MAX_CURRENCY_LENGTH: usize = 8;
const EXCHANGE_RATE_TTL_SECS: i64 = 6 * 60 * 60; // 6 hours

// ==================== Helper Functions ====================

/// Validates and truncates a string field to a maximum length
fn validate_field_length(
    value: &str,
    max_length: usize,
    field_name: &str,
) -> Result<String, ControllerError> {
    let trimmed = value.trim();
    if trimmed.len() > max_length {
        return Err(ControllerError::Validation(format!(
            "{} exceeds maximum length of {} characters",
            field_name, max_length
        )));
    }
    Ok(trimmed.to_string())
}

/// Sanitizes a string by removing potentially dangerous characters
fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            c.is_alphanumeric()
                || c.is_whitespace()
                || matches!(
                    *c,
                    '-' | '_'
                        | '.'
                        | ','
                        | ':'
                        | ';'
                        | '!'
                        | '?'
                        | '('
                        | ')'
                        | '@'
                        | '#'
                        | '$'
                        | '%'
                        | '&'
                        | '+'
                        | '='
                        | '/'
                        | '\''
                        | '"'
                )
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validates a CoinGecko coin ID using the crypto module constraints
fn validate_coin_id_str(coin_id: &str) -> Result<String, ControllerError> {
    crate::crypto::validate_coin_id(coin_id).map_err(ControllerError::Validation)
}

/// Validates a ticker/symbol (alphanumeric only)
fn validate_symbol(symbol: &str) -> Result<String, ControllerError> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return Err(ControllerError::Validation("Symbol cannot be empty".to_string()));
    }
    if trimmed.len() > MAX_SYMBOL_LENGTH {
        return Err(ControllerError::Validation(format!(
            "Symbol exceeds maximum length of {} characters",
            MAX_SYMBOL_LENGTH
        )));
    }
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ControllerError::Validation("Symbol must be alphanumeric".to_string()));
    }
    Ok(trimmed.to_uppercase())
}

/// Validates that a floating point value is finite and positive
fn validate_positive_amount(value: f64, field: &str) -> Result<f64, ControllerError> {
    if !value.is_finite() {
        return Err(ControllerError::Validation(format!("{} must be a finite number", field)));
    }
    if value <= 0.0 {
        return Err(ControllerError::Validation(format!("{} must be greater than zero", field)));
    }
    Ok(value)
}

/// Validates that an optional floating point value is finite and non-negative
fn validate_non_negative(value: Option<f64>, field: &str) -> Result<Option<f64>, ControllerError> {
    if let Some(v) = value {
        if !v.is_finite() {
            return Err(ControllerError::Validation(format!("{} must be a finite number", field)));
        }
        if v < 0.0 {
            return Err(ControllerError::Validation(format!("{} cannot be negative", field)));
        }
    }
    Ok(value)
}

/// Validates a UUID string format
fn validate_uuid(id: &str) -> Result<String, ControllerError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(ControllerError::Validation("ID cannot be empty".to_string()));
    }

    // Check if it's a valid UUID or a legacy ID format
    if Uuid::parse_str(trimmed).is_ok() {
        return Ok(trimmed.to_string());
    }

    // Allow legacy IDs that start with "migrated_" or "legacy_"
    if trimmed.starts_with("migrated_") || trimmed.starts_with("legacy_") {
        return Ok(trimmed.to_string());
    }

    Err(ControllerError::Validation("Invalid ID format".to_string()))
}

/// Valida la contraseña básica para abrir una bóveda existente
/// Solo verifica que no esté vacía y no exceda el límite
fn validate_password_basic(password: String) -> Result<SecretString, ControllerError> {
    let trimmed = password.trim();

    if trimmed.is_empty() {
        return Err(ControllerError::Validation("Password cannot be empty".to_string()));
    }

    if trimmed.len() > MAX_PASSWORD_LENGTH {
        return Err(ControllerError::Validation(format!(
            "Password cannot exceed {} characters",
            MAX_PASSWORD_LENGTH
        )));
    }

    // Crear SecretString que limpiará la memoria automáticamente
    Ok(SecretString::from(trimmed.to_string()))
}

/// Valida la contraseña con requisitos estrictos para crear una nueva bóveda
fn validate_password_strict(password: String) -> Result<SecretString, ControllerError> {
    let trimmed = password.trim();

    if trimmed.is_empty() {
        return Err(ControllerError::Validation("Password cannot be empty".to_string()));
    }

    if trimmed.len() < MIN_PASSWORD_LENGTH {
        return Err(ControllerError::Validation(format!(
            "Password must be at least {} characters",
            MIN_PASSWORD_LENGTH
        )));
    }

    if trimmed.len() > MAX_PASSWORD_LENGTH {
        return Err(ControllerError::Validation(format!(
            "Password cannot exceed {} characters",
            MAX_PASSWORD_LENGTH
        )));
    }

    // Verificar complejidad de contraseña
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

    if !has_uppercase {
        return Err(ControllerError::Validation(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if !has_lowercase {
        return Err(ControllerError::Validation(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if !has_digit {
        return Err(ControllerError::Validation(
            "Password must contain at least one number".to_string(),
        ));
    }

    if !has_special {
        return Err(ControllerError::Validation(
            "Password must contain at least one special character (!@#$%^&*...)".to_string(),
        ));
    }

    Ok(SecretString::from(trimmed.to_string()))
}

/// Valida que una fecha esté en formato ISO-8601 (YYYY-MM-DD)
fn validate_date(date: &str) -> Result<String, ControllerError> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return Err(ControllerError::Validation("Date cannot be empty".to_string()));
    }

    // Intento 1: Formato DD-MM-YYYY (Preferido por el usuario)
    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%d-%m-%Y") {
        return Ok(parsed.format("%Y-%m-%d").to_string()); // NORMALIZAR A ISO
    }

    // Intento 2: Formato ISO (Estándar DB y fallback)
    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    Err(ControllerError::Validation(
        "Invalid date format. Use DD-MM-YYYY or YYYY-MM-DD".to_string(),
    ))
}

/// Validates a hex color code
fn validate_color(color: &str) -> Result<String, ControllerError> {
    let trimmed = color.trim();

    if trimmed.is_empty() {
        return Err(ControllerError::Validation("Color cannot be empty".to_string()));
    }

    if trimmed.len() != 7 {
        return Err(ControllerError::Validation("Color must be in #RRGGBB format".to_string()));
    }

    if !trimmed.starts_with('#') {
        return Err(ControllerError::Validation("Color must start with #".to_string()));
    }

    // Validate hex characters
    if !trimmed[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ControllerError::Validation(
            "Color must contain valid hex characters".to_string(),
        ));
    }

    Ok(trimmed.to_lowercase())
}

// ==================== Configuration ====================

#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    last_db_path: Option<String>,
}



pub struct AppController {
    db: Arc<Mutex<Option<Database>>>,
    habit_service: HabitService,
    app_data_dir: PathBuf,
}

impl AppController {
    pub fn new(data_dir: PathBuf) -> Self {
        // Initialize with None as vault is locked
        let db = Arc::new(Mutex::new(None));
        // HabitService needs access to the same (potentially empty) db lock
        let habit_service = HabitService::new(db.clone());

        Self {
            db,
            habit_service,
            app_data_dir: data_dir,
        }
    }

    /// Returns the default database path
    pub fn default_db_path(&self) -> PathBuf {
        self.app_data_dir.join("sanctum.db")
    }

    /// Returns the config file path
    fn config_path(&self) -> PathBuf {
        self.app_data_dir.join("config.json")
    }

    /// Loads the application configuration
    fn load_config(&self) -> Result<AppConfig, ControllerError> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(AppConfig::default());
        }

        let data = fs::read_to_string(&path)
            .map_err(|_| ControllerError::Config("Could not read configuration".to_string()))?;

        serde_json::from_str(&data)
            .map_err(|_| ControllerError::Config("Could not parse configuration".to_string()))
    }

    /// Saves the application configuration
    fn save_config(&self, config: &AppConfig) -> Result<(), ControllerError> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| ControllerError::Config("Could not create configuration directory".to_string()))?;
        }

        let data = serde_json::to_string_pretty(config)
            .map_err(|_| ControllerError::Config("Could not serialize configuration".to_string()))?;

        fs::write(&path, &data)
            .map_err(|_| ControllerError::Config("Could not save configuration".to_string()))?;

        // Set restrictive permissions (owner read/write only - 0600)
        #[cfg(unix)]
        {
            fs::set_permissions(&path, Permissions::from_mode(0o600))
                .map_err(|_| ControllerError::Config("Could not set configuration file permissions".to_string()))?;
        }

        Ok(())
    }

    /// Persists the last used database path
    fn persist_last_db_path(&self, path: &Path) -> Result<(), ControllerError> {
        let mut config = self.load_config()?;
        config.last_db_path = Some(path.to_string_lossy().to_string());
        self.save_config(&config)
    }

    /// Opens (or creates) the persistent rate-limit store with restrictive permissions
    fn open_rate_limit_conn(&self, rate_limit_path: &Path) -> Result<Connection, ControllerError> {
        if let Some(parent) = rate_limit_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| ControllerError::Config("Could not create rate limit directory".to_string()))?;
        }

        let conn = Connection::open(rate_limit_path)
            .map_err(|_| ControllerError::Config("Could not open rate limit store".to_string()))?;

        // Restrict permissions to owner read/write only
        #[cfg(unix)]
        fs::set_permissions(rate_limit_path, Permissions::from_mode(0o600))
            .map_err(|_| ControllerError::Config("Could not set rate limit file permissions".to_string()))?;

        Ok(conn)
    }

    /// Sanitizes the requested vault path to ensure it stays inside the app data directory
    fn sanitize_db_path(&self, raw: &str) -> Result<PathBuf, ControllerError> {
        // Ensure the base directory exists so canonicalization behaves deterministically
        fs::create_dir_all(&self.app_data_dir)
            .map_err(|_| ControllerError::Config("Could not access application data directory".to_string()))?;

        let base = self.app_data_dir.canonicalize().unwrap_or(self.app_data_dir.clone());

        let raw_trimmed = raw.trim();
        if raw_trimmed.is_empty() {
            return Err(ControllerError::Validation("Vault path cannot be empty".to_string()));
        }

        let candidate = PathBuf::from(raw_trimmed);

        // If an absolute path is provided, ensure it resides within app_data_dir
        let relative = if candidate.is_absolute() {
            candidate
                .strip_prefix(&base)
                .map_err(|_| ControllerError::Validation(
                    "Vault path must stay inside the app data directory".to_string(),
                ))?
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
            let vault_key = db_path.to_string_lossy().to_string();
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
            let vault_key = db_path.to_string_lossy().to_string();
            if let Ok((attempts, locked)) = Database::record_failed_attempt(&conn, &vault_key) {
                log_auth_failure(attempts, locked);
            }
        }
    }

    /// Resets persistent rate limit after successful auth
    fn reset_persistent_rate_limit(&self, db_path: &Path) {
        let rate_limit_path = db_path.with_extension("ratelimit");

        if let Ok(conn) = self.open_rate_limit_conn(&rate_limit_path) {
            let vault_key = db_path.to_string_lossy().to_string();
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

    /// Helper to get database with session check
    fn with_db<T, F>(&self, f: F) -> Result<T, ControllerError>
    where
        F: FnOnce(&Database) -> Result<T, ControllerError>,
    {
        let db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;
        let db = db_lock.as_ref().ok_or(ControllerError::NoVaultOpen)?;

        // Check session timeout
        db.check_session_timeout().map_err(|e| match e {
            DbError::SessionExpired => ControllerError::SessionExpired,
            _ => ControllerError::Database(e),
        })?;

        // Run the operation
        let result = f(db)?;

        // Mark activity only after successful operations to avoid extending sessions from idle checks
        db.touch_session().map_err(ControllerError::Database)?;

        Ok(result)
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
    pub fn create_db(&self, password: String, path: Option<String>) -> Result<String, ControllerError> {
        let password = validate_password_strict(password)?;
        self.ensure_no_connection()?;

        let db_path_raw = if let Some(p) = path {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                self.default_db_path()
            } else {
                PathBuf::from(trimmed)
            }
        } else {
            self.default_db_path()
        };

        let db_path = self.sanitize_db_path(db_path_raw.to_string_lossy().as_ref())?;

        if db_path.exists() {
            return Err(ControllerError::VaultExists);
        }

        let database = Database::init(db_path.clone(), &password)?;
        database.health_check()?;

        let mut db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;
        *db_lock = Some(database);

        self.persist_last_db_path(&db_path)?;
        self.reset_persistent_rate_limit(&db_path);

        log_security_event(SecurityEvent::VaultCreated, None);

        Ok("Vault created successfully".to_string())
    }

    /// Opens an existing vault with the provided password
    pub fn open_db(&self, password: String, path: Option<String>) -> Result<String, ControllerError> {
        let password = validate_password_basic(password)?;
        self.ensure_no_connection()?;

        // Resolve path
        let raw_path = if let Some(p) = path {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        }
        .or_else(|| self.load_config().ok().and_then(|c| c.last_db_path));

        let db_path_raw = if let Some(p) = raw_path {
            PathBuf::from(p)
        } else {
            self.default_db_path()
        };

        let db_path = self.sanitize_db_path(db_path_raw.to_string_lossy().as_ref())?;

        if !db_path.exists() {
            return Err(ControllerError::VaultNotFound);
        }

        // Persistent rate limiting check
        self.check_persistent_rate_limit(&db_path)?;

        // Try to open the database
        let database = match Database::init(db_path.clone(), &password) {
            Ok(db) => {
                self.reset_persistent_rate_limit(&db_path);
                db
            }
            Err(e) => {
                self.record_persistent_failed_attempt(&db_path);
                log_security_event(SecurityEvent::VaultOpenFailed, None);
                return Err(ControllerError::Database(e));
            }
        };

        database.health_check()?;

        let mut db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;
        *db_lock = Some(database);

        self.persist_last_db_path(&db_path)?;

        log_security_event(SecurityEvent::VaultOpened, None);

        Ok("Vault unlocked successfully".to_string())
    }

    /// Closes the current vault connection
    pub fn close_db(&self) -> Result<String, ControllerError> {
        let mut db_lock = self.db.lock().map_err(|_| ControllerError::Internal)?;

        if db_lock.is_none() {
            return Err(ControllerError::NoVaultOpen);
        }

        *db_lock = None;

        log_security_event(SecurityEvent::VaultClosed, None);

        Ok("Vault locked successfully".to_string())
    }

    /// Returns the remaining session time in seconds
    pub fn get_session_remaining(&self) -> Result<i64, ControllerError> {
        self.with_db_no_touch(|db| {
            db.get_session_remaining().map_err(ControllerError::Database)
        })
    }

    /// Checks if a vault file exists
    pub fn check_vault_exists(&self) -> bool {
        // Check if custom path was used previously
        if let Ok(config) = self.load_config()
            && let Some(last_path) = config.last_db_path
        {
            let path = PathBuf::from(&last_path);
            if path.exists() {
                return true;
            }
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
            && let Some(last) = config.last_db_path
        {
            return Ok(last);
        }

        Ok(self.default_db_path().to_string_lossy().to_string())
    }

    // ==================== FIAT Account Methods ====================

    /// Creates a new account
    pub fn create_account(
        &self,
        name: String,
        account_type: String,
        currency: String,
        initial_balance: i64,
        color: String,
        icon: Option<String>,
    ) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let name = validate_field_length(&name, MAX_ACCOUNT_NAME_LENGTH, "Account name")?;
            let name = sanitize_string(&name);

            if name.is_empty() {
                return Err(ControllerError::Validation("Account name cannot be empty".to_string()));
            }

            let currency = validate_field_length(&currency, MAX_CURRENCY_LENGTH, "Currency")?;
            let currency = sanitize_string(&currency).to_uppercase();

            if currency.is_empty() {
                return Err(ControllerError::Validation("Currency cannot be empty".to_string()));
            }

            let color = validate_color(&color)?;

            let icon = if let Some(i) = icon {
                let i = validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?;
                if i.is_empty() { None } else { Some(i) }
            } else {
                None
            };

            let id = Uuid::new_v4().to_string();
            let created_at = chrono::Utc::now().to_rfc3339();

            let account = Account::new(
                id.clone(),
                name,
                account_type,
                currency,
                initial_balance,
                color,
                icon,
                created_at,
            );

            db.create_account(&account)?;

            log_security_event(SecurityEvent::TransactionCreated, Some("account_created"));

            Ok(id)
        })
    }

    /// Gets all accounts
    pub fn get_accounts(&self) -> Result<Vec<Account>, ControllerError> {
        self.with_db(|db| db.get_accounts().map_err(ControllerError::Database))
    }

    /// Gets all account balances
    pub fn get_account_balances(&self) -> Result<Vec<AccountBalance>, ControllerError> {
        self.with_db(|db| db.get_all_account_balances().map_err(ControllerError::Database))
    }

    /// Updates an account
    #[allow(clippy::too_many_arguments)]
    pub fn update_account(
        &self,
        id: String,
        name: String,
        account_type: String,
        currency: String,
        initial_balance: i64,
        color: String,
        icon: Option<String>,
    ) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;

            let name = validate_field_length(&name, MAX_ACCOUNT_NAME_LENGTH, "Account name")?;
            let name = sanitize_string(&name);

            if name.is_empty() {
                return Err(ControllerError::Validation("Account name cannot be empty".to_string()));
            }

            let currency = validate_field_length(&currency, MAX_CURRENCY_LENGTH, "Currency")?;
            let currency = sanitize_string(&currency).to_uppercase();

            let color = validate_color(&color)?;

            let icon = if let Some(i) = icon {
                let i = validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?;
                if i.is_empty() { None } else { Some(i) }
            } else {
                None
            };

            let existing = db.get_account(&validated_id)?;

            let account = Account {
                id: validated_id,
                name,
                account_type,
                currency,
                initial_balance,
                color,
                icon,
                is_archived: existing.is_archived,
                created_at: existing.created_at,
            };

            db.update_account(&account).map_err(ControllerError::Database)
        })
    }

    /// Archives an account (soft delete)
    pub fn archive_account(&self, id: String) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.archive_account(&validated_id).map_err(ControllerError::Database)
        })
    }

    /// Transfers funds between accounts
    pub fn transfer_funds(
        &self,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let from_id = validate_uuid(&from_account_id)?;
            let to_id = validate_uuid(&to_account_id)?;

            if amount <= 0 {
                return Err(ControllerError::Validation("Transfer amount must be greater than zero".to_string()));
            }

            let description = validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            let tx_id = db.create_transfer(&from_id, &to_id, amount, &description, &date)?;
            log_security_event(SecurityEvent::TransactionCreated, Some("transfer"));
            Ok(tx_id)
        })
    }

    // ==================== Financial Transaction Methods ====================

    /// Adds a transaction
    pub fn add_transaction(
        &self,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        is_expense: bool,
    ) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let account_id = validate_uuid(&account_id)?;
            let category = validate_field_length(&category, MAX_CATEGORY_LENGTH, "Category")?;
            let category = sanitize_string(&category);

            if category.is_empty() {
                return Err(ControllerError::Validation("Category cannot be empty".to_string()));
            }

            let description = validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            if amount <= 0 {
                return Err(ControllerError::Validation("Amount must be greater than zero".to_string()));
            }

            let id = Uuid::new_v4().to_string();
            let transaction_type = if is_expense { "expense" } else { "income" };

            let transaction = Transaction::new(
                id.clone(),
                account_id,
                amount,
                category,
                description,
                date,
                transaction_type.to_string(),
                None,
            );

            db.create_transaction(&transaction)?;
            log_security_event(SecurityEvent::TransactionCreated, Some(transaction_type));
            Ok(id)
        })
    }

    /// Gets all transactions
    pub fn get_transactions(&self) -> Result<Vec<Transaction>, ControllerError> {
        self.with_db(|db| db.get_transactions().map_err(ControllerError::Database))
    }

    /// Gets balance summary
    pub fn get_balance(&self) -> Result<BalanceSummary, ControllerError> {
        self.with_db(|db| db.get_balance_summary().map_err(ControllerError::Database))
    }

    /// Gets expenses aggregated by category
    pub fn get_expenses_by_category(&self) -> Result<Vec<(String, i64)>, ControllerError> {
        let transactions = self.get_transactions()?;
        let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for tx in transactions {
            if tx.transaction_type == "expense" {
                *map.entry(tx.category).or_default() += tx.amount;
            }
        }

        let mut result: Vec<(String, i64)> = map.into_iter().collect();
        // Sort by amount descending
        result.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(result)
    }

    /// Deletes a transaction
    pub fn delete_transaction(&self, id: String) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.delete_transaction(&validated_id)?;
            log_security_event(SecurityEvent::TransactionDeleted, None);
            Ok(())
        })
    }

    // ==================== Crypto Price Methods ====================

    /// Fetches cryptocurrency prices from CoinGecko
    pub async fn get_crypto_prices(&self, coins: Vec<String>) -> Result<Vec<CryptoAsset>, ControllerError> {
        crypto::fetch_crypto_prices(coins).await.map_err(ControllerError::Api)
    }

    /// Fetches CLP to USD exchange rate
    pub async fn get_clp_usd_rate(&self) -> Result<f64, ControllerError> {
        crypto::fetch_clp_usd_rate().await.map_err(ControllerError::Api)
    }

    /// Saves exchange rate to cache
    pub fn save_exchange_rate(&self, pair: String, rate: f64) -> Result<(), ControllerError> {
        self.with_db(|db| db.save_exchange_rate(&pair, rate).map_err(ControllerError::Database))
    }

    /// Loads cached exchange rate
    pub fn load_exchange_rate(&self, pair: String) -> Result<Option<(f64, String)>, ControllerError> {
        self.with_db(|db| {
            let cached = db.load_exchange_rate(&pair).map_err(ControllerError::Database)?;

            if let Some((rate, updated_at)) = cached {
                let is_fresh = chrono::DateTime::parse_from_rfc3339(&updated_at)
                    .map(|dt| {
                        let age = Utc::now().signed_duration_since(dt.with_timezone(&Utc));
                        age.num_seconds() <= EXCHANGE_RATE_TTL_SECS
                    })
                    .unwrap_or(false);

                if !is_fresh {
                    return Ok(None);
                }

                return Ok(Some((rate, updated_at)));
            }

            Ok(None)
        })
    }

    /// Saves crypto prices to cache
    pub fn save_crypto_prices(&self, prices: Vec<CryptoAsset>) -> Result<(), ControllerError> {
        self.with_db(|db| {
            for price in prices {
                db.save_crypto_price(
                    &price.id,
                    &price.symbol,
                    &price.name,
                    price.current_price,
                    price.price_change_percentage_24h,
                )?;
            }
            Ok(())
        })
    }

    /// Loads cached crypto prices
    pub fn load_crypto_prices(&self) -> Result<Vec<CryptoAsset>, ControllerError> {
        self.with_db(|db| {
            let cached = db.load_crypto_prices()?;
            Ok(cached
                .into_iter()
                .map(|(id, symbol, name, price, change, updated)| CryptoAsset {
                    id,
                    symbol,
                    name,
                    current_price: price,
                    price_change_percentage_24h: change,
                    last_updated: updated,
                })
                .collect())
        })
    }

    // ==================== Crypto Wallet Methods ====================

    /// Creates a new crypto wallet
    pub fn add_wallet(&self, name: String, category: String, icon: Option<String>) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let name = validate_field_length(&name, MAX_WALLET_NAME_LENGTH, "Wallet name")?;
            let name = sanitize_string(&name);

            if name.is_empty() {
                return Err(ControllerError::Validation("Wallet name cannot be empty".to_string()));
            }

            let valid_categories = ["exchange", "wallet_single", "wallet_multi"];
            if !valid_categories.contains(&category.as_str()) {
                return Err(ControllerError::Validation(format!(
                    "Invalid category. Must be one of: {}",
                    valid_categories.join(", ")
                )));
            }

            let icon = match icon {
                Some(i) => Some(validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?),
                None => None,
            };

            let id = Uuid::new_v4().to_string();
            log_security_event(SecurityEvent::WalletCreated, Some(&category));

            let wallet = CryptoWallet::new(id.clone(), name, category, icon);
            db.create_wallet(&wallet)?;
            Ok(id)
        })
    }

    /// Gets all wallets
    pub fn get_wallets(&self) -> Result<Vec<CryptoWallet>, ControllerError> {
        self.with_db(|db| db.get_wallets().map_err(ControllerError::Database))
    }

    /// Deletes a wallet
    pub fn delete_wallet(&self, id: String) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.delete_wallet(&validated_id)?;
            log_security_event(SecurityEvent::WalletDeleted, None);
            Ok(())
        })
    }

    // ==================== Crypto Transaction Methods ====================

    /// Adds a crypto transaction
    #[allow(clippy::too_many_arguments)]
    pub fn add_crypto_transaction(
        &self,
        wallet_id: String,
        coin_id: String,
        symbol: String,
        transaction_type: String,
        amount: f64,
        price_per_coin: Option<f64>,
        fee: Option<f64>,
        date: String,
        notes: Option<String>,
    ) -> Result<String, ControllerError> {
        self.with_db(|db| {
            if wallet_id.trim().is_empty() {
                return Err(ControllerError::Validation("Wallet ID cannot be empty".to_string()));
            }

            let coin_id = validate_coin_id_str(&coin_id)?;
            let symbol = validate_symbol(&symbol)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            validate_positive_amount(amount, "Amount")?;
            let price_per_coin = validate_non_negative(price_per_coin, "Price per coin")?;
            let fee = validate_non_negative(fee, "Fee")?;
            let date = validate_date(&date)?;

            let valid_types = ["buy", "sell", "transfer_in", "transfer_out", "swap"];
            if !valid_types.contains(&transaction_type.as_str()) {
                return Err(ControllerError::Validation(format!(
                    "Invalid transaction type. Must be one of: {}",
                    valid_types.join(", ")
                )));
            }

            log_security_event(SecurityEvent::CryptoTransactionCreated, Some(&transaction_type));

            let id = Uuid::new_v4().to_string();
            let transaction = CryptoTransaction::new(
                id.clone(),
                wallet_id.trim().to_string(),
                coin_id.to_lowercase(),
                symbol.to_uppercase(),
                transaction_type,
                amount,
                price_per_coin,
                fee,
                date,
                notes,
            );

            db.create_crypto_transaction(&transaction)?;
            Ok(id)
        })
    }

    /// Gets wallet transactions
    pub fn get_wallet_transactions(&self, wallet_id: String) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&wallet_id)?;
            db.get_wallet_transactions(&validated_id).map_err(ControllerError::Database)
        })
    }

    /// Gets crypto transactions for a specific coin
    pub fn get_crypto_transactions_by_coin(&self, coin_id: String) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.with_db(|db| {
            let validated = validate_coin_id_str(&coin_id)?;
            db.get_crypto_transactions_by_coin(&validated).map_err(ControllerError::Database)
        })
    }

    /// Deletes a crypto transaction
    pub fn delete_crypto_transaction(&self, id: String) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;

            // Check if this transaction has a related transaction (swap/transfer)
            if let Ok(Some(tx)) = db.get_crypto_transaction(&validated_id)
                && let Some(related_id) = tx.related_tx_id
            {
                let _ = db.delete_crypto_transaction(&related_id);
            }

            db.delete_crypto_transaction(&validated_id)?;
            Ok(())
        })
    }

    // ==================== Portfolio Aggregation Methods ====================

    /// Gets aggregated portfolio across all wallets
    pub fn get_aggregated_portfolio(&self) -> Result<Vec<AggregatedAsset>, ControllerError> {
        self.with_db(|db| db.get_aggregated_portfolio().map_err(ControllerError::Database))
    }

    /// Gets aggregated holdings for a specific wallet
    pub fn get_wallet_holdings(&self, wallet_id: String) -> Result<Vec<AggregatedAsset>, ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&wallet_id)?;
            db.get_wallet_aggregated_holdings(&validated_id).map_err(ControllerError::Database)
        })
    }

    // ==================== Habits Methods ====================

    // ==================== Habit Management ====================

    pub fn create_habit(
        &self,
        name: String,
        description: Option<String>,
        color: String,
    ) -> std::result::Result<String, ControllerError> {
        if name.trim().is_empty() {
            return Err(ControllerError::Validation("Habit name cannot be empty".to_string()));
        }

        // Validate color format (basic hex)
        let color_regex = Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap();
        if !color_regex.is_match(&color) {
            return Err(ControllerError::Validation("Invalid color format. Use #RRGGBB".to_string()));
        }

        self.habit_service
            .create_habit(name, description, color)
            .map_err(ControllerError::Database)
    }

    pub fn get_habits(&self) -> std::result::Result<Vec<Habit>, ControllerError> {
        self.habit_service.get_habits().map_err(ControllerError::Database)
    }

    /// Updates a habit
    pub fn update_habit(
        &self,
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        is_archived: bool,
    ) -> std::result::Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }

        if name.trim().is_empty() {
            return Err(ControllerError::Validation(
                "Habit name cannot be empty".to_string(),
            ));
        }
        
        // Validate color
        let color_regex = Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap();
        if !color_regex.is_match(&color) {
            return Err(ControllerError::Validation(
                "Invalid color format. Use #RRGGBB".to_string(),
            ));
        }

        self.habit_service
            .update_habit(id, name, description, color, is_archived)
            .map_err(ControllerError::Database)
    }

    pub fn archive_habit(&self, id: String) -> std::result::Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }
        self.habit_service.archive_habit(id).map_err(ControllerError::Database)
    }

    /// Deletes a habit
    pub fn delete_habit(&self, id: String) -> std::result::Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }
        self.habit_service.delete_habit(id).map_err(ControllerError::Database)
    }

    /// Toggles habit completion for a date
    pub fn toggle_habit_completion(
        &self,
        habit_id: String,
        date: String,
    ) -> std::result::Result<bool, ControllerError> {
        if validate_uuid(&habit_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }

        // Validate date format YYYY-MM-DD
        if NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
            return Err(ControllerError::Validation(
                "Invalid date format. Use YYYY-MM-DD".to_string(),
            ));
        }

        self.habit_service
            .toggle_habit_completion(habit_id, date)
            .map_err(ControllerError::Database)
    }

    pub fn get_habit_logs(
        &self,
        start_date: String,
        end_date: String,
    ) -> std::result::Result<Vec<HabitLog>, ControllerError> {
        // Validate dates
        if NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").is_err()
            || NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").is_err()
        {
            return Err(ControllerError::Validation(
                "Invalid date format".to_string(),
            ));
        }

        self.habit_service
            .get_habit_logs(start_date, end_date)
            .map_err(ControllerError::Database)
    }

    /// Provides analytics summary (net worth history + expense breakdown)
    pub fn get_analytics_summary(&self, range: String) -> Result<AnalyticsSummary, ControllerError> {
        let balances = self.get_account_balances()?;
        let current_balance: i64 = balances.iter().map(|b| b.current_balance).sum();
        let transactions = self.get_transactions()?;

        let today = chrono::Local::now().date_naive();
        let start_date = match range.as_str() {
            "1M" => today
                .checked_sub_signed(chrono::Duration::days(30))
                .unwrap_or(today),
            "3M" => today
                .checked_sub_signed(chrono::Duration::days(90))
                .unwrap_or(today),
            "6M" => today
                .checked_sub_signed(chrono::Duration::days(180))
                .unwrap_or(today),
            "1Y" => today
                .checked_sub_signed(chrono::Duration::days(365))
                .unwrap_or(today),
            _ => today,
        };

        // Build daily deltas
        let mut delta_by_day: std::collections::HashMap<NaiveDate, i64> = std::collections::HashMap::new();
        let mut earliest_tx: Option<NaiveDate> = None;
        for tx in &transactions {
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d") {
                let delta = match tx.transaction_type.as_str() {
                    "income" => tx.amount,
                    "expense" => -tx.amount,
                    _ => 0,
                };
                *delta_by_day.entry(date).or_insert(0) += delta;
                earliest_tx = Some(earliest_tx.map_or(date, |d| d.min(date)));
            }
        }

        let effective_start = if range == "ALL" {
            earliest_tx.unwrap_or(today)
        } else {
            start_date.min(today)
        };

        // Time travel backwards from today
        let mut cursor = today;
        let mut points_rev: Vec<(NaiveDate, i64)> = Vec::new();
        let mut balance = current_balance;

        loop {
            points_rev.push((cursor, balance));
            let delta = *delta_by_day.get(&cursor).unwrap_or(&0);
            balance -= delta;

            if cursor <= effective_start {
                break;
            }
            if let Some(prev) = cursor.pred_opt() {
                cursor = prev;
            } else {
                break;
            }
        }

        points_rev.reverse();
        if points_rev.is_empty() {
            points_rev.push((today, current_balance));
        }

        let values: Vec<i64> = points_rev.iter().map(|(_, v)| *v).collect();
        let min_val = *values.iter().min().unwrap_or(&0);
        let max_val = *values.iter().max().unwrap_or(&0);
        let is_flat = max_val == min_val;
        let safe_range = if is_flat { 1.0 } else { (max_val - min_val) as f32 };

        let mut path_cmd = String::new();
        let len = values.len() as f32;

        for (idx, val) in values.iter().enumerate() {
            let x = if len > 1.0 {
                (idx as f32) * (100.0 / (len - 1.0))
            } else {
                0.0
            };

            let ratio = if is_flat {
                0.5
            } else {
                ((*val - min_val) as f32 / safe_range).clamp(0.0, 1.0)
            };

            let y_norm = 100.0 - (5.0 + (ratio * 90.0));

            if idx == 0 {
                path_cmd.push_str(&format!("M {:.2} {:.2}", x, y_norm));
            } else {
                path_cmd.push_str(&format!(" L {:.2} {:.2}", x, y_norm));
            }
        }

        if path_cmd.is_empty() {
            path_cmd = "M 0 50 L 100 50".to_string();
        }

        // Expense donut (current month)
        let mut expenses: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        let current_month = today.month();
        let current_year = today.year();
        for tx in &transactions {
            if tx.transaction_type != "expense" {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d")
                && date.year() == current_year
                && date.month() == current_month
            {
                *expenses
                    .entry(tx.category.to_uppercase())
                    .or_insert(0) += tx.amount;
            }
        }

        let total_expense: i64 = expenses.values().sum();
        let mut expense_slices: Vec<ExpenseSlice> = Vec::new();
        if total_expense > 0 {
            let mut by_amount: Vec<(String, i64)> = expenses.into_iter().collect();
            by_amount.sort_by(|a, b| b.1.cmp(&a.1));

            let colors = [
                "#8b5cf6",
                "#ec4899",
                "#3b82f6",
                "#10b981",
                "#f59e0b",
                "#ef4444",
                "#6366f1",
                "#14b8a6",
            ];

            for (idx, (category, amount)) in by_amount.iter().enumerate() {
                if *amount <= 0 {
                    continue;
                }
                let percentage = *amount as f32 / total_expense as f32;
                let color = colors[idx % colors.len()].to_string();

                expense_slices.push(ExpenseSlice {
                    category: category.clone(),
                    amount: *amount,
                    percentage,
                    color,
                });
            }
        }

        Ok(AnalyticsSummary {
            chart_path: path_cmd,
            net_worth: self.format_money_display(current_balance),
            max_value: self.format_money_display(max_val),
            min_value: self.format_money_display(min_val),
            expense_slices,
        })
    }

    /// Returns normalized SVG path commands (0-100 space) for net worth history and current net worth formatted
    /// Also returns min and max values formatted for labels
    pub fn get_net_worth_history(&self, range: &str) -> Result<(String, String, String, String), ControllerError> {
        let accounts = self.get_accounts()?;
        let transactions = self.get_transactions()?;

        // Event definition to merge Account Creations and Transactions
        struct FinancialEvent {
            date: chrono::NaiveDate,
            amount_delta: i64,
        }

        let mut events: Vec<FinancialEvent> = Vec::new();

        // 1. Add Account Creation Events (Initial Balances)
        for acc in &accounts {
            if acc.initial_balance != 0 {
                // Try to parse created_at, fallback to today if invalid
                let date = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&acc.created_at) {
                    dt.date_naive()
                } else if let Ok(d) = NaiveDate::parse_from_str(&acc.created_at, "%Y-%m-%d") {
                    d
                } else {
                     chrono::Local::now().date_naive()
                };
                
                events.push(FinancialEvent {
                    date,
                    amount_delta: acc.initial_balance,
                });
            }
        }

        // 2. Add Transaction Events
        for tx in &transactions {
             let delta = match tx.transaction_type.as_str() {
                "income" => tx.amount,
                "expense" => -tx.amount,
                _ => 0, 
            };
            
            if delta == 0 { continue; }
            
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d") {
                events.push(FinancialEvent {
                    date,
                    amount_delta: delta,
                });
            }
        }

        // 3. Sort chronologically
        events.sort_by(|a, b| a.date.cmp(&b.date));

        // 4. Build Full History
        let mut full_history: Vec<(chrono::NaiveDate, i64)> = Vec::new();
        let mut current_balance = 0;
        
        // Add a "zero" start point if we have events, slightly before the first one
        if let Some(first) = events.first() {
             full_history.push((first.date.pred_opt().unwrap_or(first.date), 0));
        } else {
             // No events at all
             full_history.push((chrono::Local::now().date_naive(), 0));
        }

        for event in events {
            current_balance += event.amount_delta;
            full_history.push((event.date, current_balance));
        }
        
        // Ensure the last point reflects "today" (extend the line to now)
        let today = chrono::Local::now().date_naive();
        if full_history.last().is_some_and(|last| last.0 < today) {
             full_history.push((today, current_balance));
        }

        let net_worth_formatted = self.format_money_display(current_balance);

        // 5. Filter based on Range
        let start_date = match range {
            "1M" => Some(today - chrono::Duration::days(30)),
            "3M" => Some(today - chrono::Duration::days(90)),
            "6M" => Some(today - chrono::Duration::days(180)),
            "1Y" => Some(today - chrono::Duration::days(365)),
            _ => None, // ALL
        };

        let filtered_history: Vec<(chrono::NaiveDate, i64)> = if let Some(start) = start_date {
            // Find the balance *at* the start date to be the first point
            // This prevents the graph from starting at 0 if the user had money before the range
            let start_balance = full_history
                .iter()
                .rfind(|(d, _)| *d <= start)
                .map(|(_, b)| *b)
                .unwrap_or(0); // If no history before start, balance is 0
                
            let mut range_points: Vec<(chrono::NaiveDate, i64)> = Vec::new();
            // Add start point
            range_points.push((start, start_balance));
            
            // Add all points within range
            range_points.extend(full_history.into_iter().filter(|(d, _)| *d >= start));
            range_points
        } else {
            full_history
        };

        if filtered_history.is_empty() {
             return Ok(("M 0 50 L 100 50".to_string(), net_worth_formatted, "$ 0.00".to_string(), "$ 0.00".to_string()));
        }

        let balances: Vec<i64> = filtered_history.iter().map(|(_, b)| *b).collect();
        let min_val = *balances.iter().min().unwrap_or(&0);
        let max_val = *balances.iter().max().unwrap_or(&0);
        
        let min_formatted = self.format_money_display(min_val);
        let max_formatted = self.format_money_display(max_val);

        let len = balances.len() as f32;
        let mut path_cmd = String::new();
        
        // PADDING: Add 5% padding top and bottom
        let range = (max_val - min_val) as f32;
        let safe_range = if range == 0.0 { 1.0 } else { range };

        // Generate Path (Bezier Curve approximation or simple Line)
        // For simplicity and robustness, we stick to Line first. 
        // Slint Path supports cubic-bezier but calculating control points manually is verbose.
        // Let's stick to Lines but ensure they look good.
        for (idx, val) in balances.iter().enumerate() {
            let x = if len > 1.0 {
                (idx as f32) * (100.0 / (len - 1.0))
            } else {
                0.0
            };

            let y_norm = if max_val == min_val {
                50.0
            } else {
                let ratio = (*val - min_val) as f32 / safe_range;
                // Scale to 5..95 (inverted)
                100.0 - (5.0 + (ratio * 90.0)) 
            };

            if idx == 0 {
                path_cmd.push_str(&format!("M {:.2} {:.2}", x, y_norm));
            } else {
                path_cmd.push_str(&format!(" L {:.2} {:.2}", x, y_norm));
            }
        }

        if path_cmd.is_empty() {
            path_cmd = "M 0 50 L 100 50".to_string();
        }

        Ok((path_cmd, net_worth_formatted, min_formatted, max_formatted))
    }

    fn format_money_display(&self, value: i64) -> String {
        let abs = value.abs();
        let units = abs / 100;
        let cents = abs % 100;
        let sign = if value < 0 { "-" } else { "" };
        format!("{sign}$ {units}.{cents:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_uuid("migrated_test").is_ok());
        assert!(validate_uuid("legacy_portfolio").is_ok());
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

    #[test]
    fn test_validate_password_strict_valid() {
        let result = validate_password_strict("Password1!".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().expose_secret(), "Password1!");
    }

    #[test]
    fn test_validate_date_valid() {
        assert!(validate_date("2024-01-15").is_ok());
        assert_eq!(validate_date("15-01-2024").unwrap(), "2024-01-15");
    }

    #[test]
    fn test_validate_date_invalid() {
        assert!(validate_date("").is_err());
        assert!(validate_date("not-a-date").is_err());
    }
}
