//! Application Controller for Sanctum
//!
//! This module provides a pure Rust API that can be consumed by any UI framework (Slint, etc.)
//! All Tauri-specific code has been removed.

use crate::crypto;
use crate::db::{Database, DbError};
use crate::models::{
    Account, AccountBalance, AggregatedAsset, BalanceSummary, CryptoAsset, CryptoCatalogCoin,
    CryptoTransaction, CryptoTransactionType, CryptoWallet, Habit, HabitLog, Transaction,
    TransactionCategory,
};
use crate::security_log::{SecurityEvent, log_auth_failure, log_security_event};
use crate::services::habit::HabitService;
use chrono::{Datelike, Local, NaiveDate, Utc};
use regex::Regex;
use rusqlite::{Connection, Error as RusqliteError};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
const MAX_CATEGORY_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 512;
const MAX_NOTES_LENGTH: usize = 1024;
const MAX_WALLET_NAME_LENGTH: usize = 128;
const MAX_SYMBOL_LENGTH: usize = 16;
const MAX_ICON_LENGTH: usize = 32;
const MAX_PASSWORD_LENGTH: usize = 128;
const MIN_PASSWORD_LENGTH: usize = 8;
const PASSWORD_PASSPHRASE_LENGTH: usize = 16;
const MAX_ACCOUNT_NAME_LENGTH: usize = 64;
const MAX_CURRENCY_LENGTH: usize = 8;
const MAX_COIN_NAME_LENGTH: usize = 64;
const EXCHANGE_RATE_TTL_SECS: i64 = 6 * 60 * 60; // 6 hours
pub const SETTING_AUTO_FETCH: &str = "auto_fetch_crypto";
pub const SETTING_TICKER_COINS: &str = "ticker_coins";
pub const SETTING_CRYPTO_LAST_UPDATED: &str = "crypto_last_updated";
pub const SETTING_CRYPTO_CUSTOM_COINS: &str = "crypto_custom_coins";
pub const SETTING_CRYPTO_HIDDEN_COINS: &str = "crypto_hidden_coins";
pub const SETTING_CRYPTO_FAVORITE_COINS: &str = "crypto_favorite_coins";
pub const SETTING_CRYPTO_LAST_WALLET_ID: &str = "crypto_last_wallet_id";
pub const SETTING_CRYPTO_LAST_COIN_ID: &str = "crypto_last_coin_id";

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
        return Err(ControllerError::Validation(
            "Symbol cannot be empty".to_string(),
        ));
    }
    if trimmed.len() > MAX_SYMBOL_LENGTH {
        return Err(ControllerError::Validation(format!(
            "Symbol exceeds maximum length of {} characters",
            MAX_SYMBOL_LENGTH
        )));
    }
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ControllerError::Validation(
            "Symbol must be alphanumeric".to_string(),
        ));
    }
    Ok(trimmed.to_uppercase())
}


/// Validates that a floating point value is finite and positive
fn validate_positive_amount(value: f64, field: &str) -> Result<f64, ControllerError> {
    if !value.is_finite() {
        return Err(ControllerError::Validation(format!(
            "{} must be a finite number",
            field
        )));
    }
    if value <= 0.0 {
        return Err(ControllerError::Validation(format!(
            "{} must be greater than zero",
            field
        )));
    }
    Ok(value)
}

/// Validates that an optional floating point value is finite and non-negative
fn validate_non_negative(value: Option<f64>, field: &str) -> Result<Option<f64>, ControllerError> {
    if let Some(v) = value {
        if !v.is_finite() {
            return Err(ControllerError::Validation(format!(
                "{} must be a finite number",
                field
            )));
        }
        if v < 0.0 {
            return Err(ControllerError::Validation(format!(
                "{} cannot be negative",
                field
            )));
        }
    }
    Ok(value)
}

fn normalize_fee_coin(
    fee_coin_id: Option<String>,
    fee_amount: Option<f64>,
) -> Result<(Option<String>, Option<f64>), ControllerError> {
    let fee_coin_id = fee_coin_id.and_then(|id| {
        let trimmed = id.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    match (fee_coin_id, fee_amount) {
        (None, None) => Ok((None, None)),
        (Some(id), Some(amount)) => {
            let id = validate_coin_id_str(&id)?;
            let amount = validate_positive_amount(amount, "Fee amount")?;
            Ok((Some(id), Some(amount)))
        }
        (None, Some(_)) => Err(ControllerError::Validation(
            "Fee coin is required when fee amount is provided".to_string(),
        )),
        (Some(_), None) => Ok((None, None)),
    }
}

/// Validates sufficient balance for a transaction
/// Returns error if required_amount exceeds available balance
fn validate_sufficient_balance(
    db: &Database,
    wallet_id: &str,
    coin_id: &str,
    symbol: &str,
    required_amount: f64,
    date: &str,
    exclude_tx_id: Option<&str>,
) -> Result<(), ControllerError> {
    let balance = db
        .get_wallet_coin_balance_at(wallet_id, coin_id, date, exclude_tx_id)
        .map_err(ControllerError::Database)?;

    if required_amount > balance {
        return Err(ControllerError::Validation(format!(
            "Insufficient funds. Available: {:.8} {}",
            balance, symbol
        )));
    }
    Ok(())
}

/// Validates sufficient balance for transaction fee
/// Handles both same-coin fees and different-coin fees
struct FeeBalanceContext<'a> {
    db: &'a Database,
    wallet_id: &'a str,
    main_coin_id: &'a str,
    main_symbol: &'a str,
    main_amount: f64,
    is_outflow: bool,
    date: &'a str,
    exclude_tx_id: Option<&'a str>,
}

fn validate_fee_balance(
    ctx: FeeBalanceContext<'_>,
    fee_coin_id: Option<&str>,
    fee_amount: Option<f64>,
) -> Result<(), ControllerError> {
    if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id, fee_amount) {
        if fee_coin == ctx.main_coin_id {
            // Same coin fee
            if ctx.is_outflow {
                // For outflows, validate main_amount + fee <= balance
                let total_required = ctx.main_amount + fee_amt;
                validate_sufficient_balance(
                    ctx.db,
                    ctx.wallet_id,
                    ctx.main_coin_id,
                    ctx.main_symbol,
                    total_required,
                    ctx.date,
                    ctx.exclude_tx_id,
                )?;
            } else {
                // For inflows, validate fee <= (incoming + existing balance)
                let existing = ctx
                    .db
                    .get_wallet_coin_balance_at(
                        ctx.wallet_id,
                        ctx.main_coin_id,
                        ctx.date,
                        ctx.exclude_tx_id,
                    )
                    .map_err(ControllerError::Database)?;
                if fee_amt > ctx.main_amount + existing {
                    return Err(ControllerError::Validation(
                        "Fee amount exceeds the available balance for this asset".to_string(),
                    ));
                }
            }
        } else {
            // Different coin fee
            validate_sufficient_balance(
                ctx.db,
                ctx.wallet_id,
                fee_coin,
                fee_coin, // Using coin_id as symbol fallback
                fee_amt,
                ctx.date,
                ctx.exclude_tx_id,
            )?;
        }
    }
    Ok(())
}

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

/// Validates that a date is in ISO-8601 format (YYYY-MM-DD)
fn validate_date(date: &str) -> Result<String, ControllerError> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return Err(ControllerError::Validation(
            "Date cannot be empty".to_string(),
        ));
    }

    // Attempt 1: DD-MM-YYYY format (preferred by the user)
    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%d-%m-%Y") {
        return Ok(parsed.format("%Y-%m-%d").to_string()); // NORMALIZAR A ISO
    }

    // Attempt 2: ISO format (DB standard and fallback)
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
        return Err(ControllerError::Validation(
            "Color cannot be empty".to_string(),
        ));
    }

    if trimmed.len() != 7 {
        return Err(ControllerError::Validation(
            "Color must be in #RRGGBB format".to_string(),
        ));
    }

    if !trimmed.starts_with('#') {
        return Err(ControllerError::Validation(
            "Color must start with #".to_string(),
        ));
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
            fs::create_dir_all(parent).map_err(|_| {
                ControllerError::Config("Could not create configuration directory".to_string())
            })?;
        }

        let data = serde_json::to_string_pretty(config).map_err(|_| {
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
        config.last_db_path = Some(path.to_string_lossy().to_string());
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
        // Ensure the base directory exists so canonicalization behaves deterministically
        fs::create_dir_all(&self.app_data_dir).map_err(|_| {
            ControllerError::Config("Could not access application data directory".to_string())
        })?;

        let base = self
            .app_data_dir
            .canonicalize()
            .unwrap_or(self.app_data_dir.clone());

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
    pub fn create_db(
        &self,
        password: String,
        path: Option<String>,
    ) -> Result<String, ControllerError> {
        let password = validate_password_basic(password)?;
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

    /// Returns a warning string for weak passwords (empty if strong/ok)
    pub fn check_password_strength(&self, password: String) -> String {
        password_strength_warning(&password).unwrap_or_default()
    }

    /// Opens an existing vault with the provided password
    pub fn open_db(
        &self,
        password: String,
        path: Option<String>,
    ) -> Result<String, ControllerError> {
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
            db.get_session_remaining()
                .map_err(ControllerError::Database)
        })
    }

    // ==================== Settings Methods ====================

    /// Gets an application setting
    pub fn get_app_setting(&self, key: &str) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let val = db.get_setting(key).map_err(ControllerError::Database)?;
            Ok(val.unwrap_or_default())
        })
    }

    /// Sets an application setting
    pub fn set_app_setting(&self, key: &str, value: &str) -> Result<(), ControllerError> {
        self.with_db(|db| {
            db.set_setting(key, value)
                .map_err(ControllerError::Database)
        })
    }

    /// Gets active ticker IDs from settings or default
    pub fn get_active_ticker_ids(&self) -> Vec<String> {
        self.get_app_setting(SETTING_TICKER_COINS)
            .ok()
            .filter(|val| !val.is_empty())
            .and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
            .unwrap_or_else(crypto::default_ticker_ids)
    }

    /// Saves active ticker IDs to settings
    pub fn save_active_ticker_ids(&self, ids: Vec<String>) -> Result<(), ControllerError> {
        let json =
            serde_json::to_string(&ids).map_err(|e| ControllerError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_TICKER_COINS, &json)
    }

    /// Loads custom coins configured by the user
    pub fn get_custom_coin_catalog(&self) -> Result<Vec<CryptoCatalogCoin>, ControllerError> {
        let raw = self.get_app_setting(SETTING_CRYPTO_CUSTOM_COINS)?;
        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut coins: Vec<CryptoCatalogCoin> =
            serde_json::from_str(&raw).map_err(|e| ControllerError::Validation(e.to_string()))?;
        for coin in &mut coins {
            coin.custom = true;
        }
        Ok(coins)
    }

    /// Loads hidden coin IDs for the catalog UI
    pub fn get_hidden_coin_ids(&self) -> Vec<String> {
        self.get_app_setting(SETTING_CRYPTO_HIDDEN_COINS)
            .ok()
            .filter(|val| !val.is_empty())
            .and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
            .unwrap_or_default()
    }

    /// Loads favorite coin IDs for the catalog UI
    pub fn get_favorite_coin_ids(&self) -> Vec<String> {
        self.get_app_setting(SETTING_CRYPTO_FAVORITE_COINS)
            .ok()
            .filter(|val| !val.is_empty())
            .and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
            .unwrap_or_default()
    }

    /// Saves custom coins to settings
    fn save_custom_coin_catalog(
        &self,
        coins: Vec<CryptoCatalogCoin>,
    ) -> Result<(), ControllerError> {
        let json = serde_json::to_string(&coins)
            .map_err(|e| ControllerError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_CRYPTO_CUSTOM_COINS, &json)
    }

    /// Saves hidden coin IDs to settings
    fn save_hidden_coin_ids(&self, ids: Vec<String>) -> Result<(), ControllerError> {
        let json =
            serde_json::to_string(&ids).map_err(|e| ControllerError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_CRYPTO_HIDDEN_COINS, &json)
    }

    /// Saves favorite coin IDs to settings
    fn save_favorite_coin_ids(&self, ids: Vec<String>) -> Result<(), ControllerError> {
        let json =
            serde_json::to_string(&ids).map_err(|e| ControllerError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_CRYPTO_FAVORITE_COINS, &json)
    }

    /// Marks or unmarks a coin as favorite
    pub fn set_favorite_coin(&self, id: String, favorite: bool) -> Result<(), ControllerError> {
        let id = validate_coin_id_str(&id)?;
        let mut favorites = self.get_favorite_coin_ids();
        let had_id = favorites.iter().any(|coin| coin == &id);

        if favorite && !had_id {
            favorites.push(id);
            favorites.sort();
            favorites.dedup();
            self.save_favorite_coin_ids(favorites)?;
        } else if !favorite && had_id {
            favorites.retain(|coin| coin != &id);
            self.save_favorite_coin_ids(favorites)?;
        }

        Ok(())
    }

    /// Returns the full coin catalog (defaults + custom)
    pub fn get_coin_catalog(&self) -> Result<Vec<CryptoCatalogCoin>, ControllerError> {
        let mut catalog = crypto::default_coin_catalog();
        let custom = self.get_custom_coin_catalog()?;
        let mut ids: HashSet<String> = catalog.iter().map(|c| c.id.clone()).collect();

        for coin in custom {
            if ids.insert(coin.id.clone()) {
                catalog.push(coin);
            }
        }

        let hidden = self.get_hidden_coin_ids();
        if !hidden.is_empty() {
            let hidden: HashSet<String> = hidden.into_iter().collect();
            catalog.retain(|coin| !hidden.contains(&coin.id));
        }

        Ok(catalog)
    }

    /// Adds a custom coin to the catalog
    pub fn add_custom_coin(
        &self,
        id: String,
        name: String,
        symbol: String,
    ) -> Result<(), ControllerError> {
        let id = validate_coin_id_str(&id)?;
        let symbol = validate_symbol(&symbol)?;
        let name = validate_field_length(&name, MAX_COIN_NAME_LENGTH, "Coin name")?;
        let name = sanitize_string(&name);

        if name.is_empty() {
            return Err(ControllerError::Validation(
                "Coin name cannot be empty".to_string(),
            ));
        }

        let mut custom = self.get_custom_coin_catalog()?;

        if custom.iter().any(|coin| coin.id == id)
            || crypto::default_coin_catalog()
                .iter()
                .any(|coin| coin.id == id)
        {
            return Err(ControllerError::Validation(
                "Coin ID already exists".to_string(),
            ));
        }

        custom.push(CryptoCatalogCoin {
            id,
            name,
            symbol,
            custom: true,
        });

        self.save_custom_coin_catalog(custom)
    }

    /// Deletes a custom coin from the catalog
    pub fn delete_custom_coin(&self, id: String) -> Result<(), ControllerError> {
        let id = validate_coin_id_str(&id)?;
        let mut custom = self.get_custom_coin_catalog()?;
        let before = custom.len();
        custom.retain(|coin| coin.id != id);
        let removed_custom = custom.len() != before;

        if removed_custom {
            self.save_custom_coin_catalog(custom)?;
        }

        let is_default = crypto::default_coin_catalog()
            .iter()
            .any(|coin| coin.id == id);
        let mut hidden_updated = false;
        if is_default {
            let mut hidden = self.get_hidden_coin_ids();
            if !hidden.iter().any(|coin| coin == &id) {
                hidden.push(id.clone());
                hidden.sort();
                hidden.dedup();
                self.save_hidden_coin_ids(hidden)?;
                hidden_updated = true;
            }
        }

        if !removed_custom && !hidden_updated {
            return Err(ControllerError::Validation("Coin not found".to_string()));
        }

        let mut active = self.get_active_ticker_ids();
        if active.iter().any(|coin| coin == &id) {
            active.retain(|coin| coin != &id);
            let _ = self.save_active_ticker_ids(active);
        }

        let mut favorites = self.get_favorite_coin_ids();
        if favorites.iter().any(|coin| coin == &id) {
            favorites.retain(|coin| coin != &id);
            let _ = self.save_favorite_coin_ids(favorites);
        }

        Ok(())
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
                return Err(ControllerError::Validation(
                    "Account name cannot be empty".to_string(),
                ));
            }

            let currency = validate_field_length(&currency, MAX_CURRENCY_LENGTH, "Currency")?;
            let currency = sanitize_string(&currency).to_uppercase();

            if currency.is_empty() {
                return Err(ControllerError::Validation(
                    "Currency cannot be empty".to_string(),
                ));
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
        self.with_db(|db| {
            db.get_all_account_balances()
                .map_err(ControllerError::Database)
        })
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
                return Err(ControllerError::Validation(
                    "Account name cannot be empty".to_string(),
                ));
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

            db.update_account(&account)
                .map_err(ControllerError::Database)
        })
    }

    /// Archives an account (soft delete)
    pub fn archive_account(&self, id: String) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.archive_account(&validated_id)
                .map_err(ControllerError::Database)
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
                return Err(ControllerError::Validation(
                    "Transfer amount must be greater than zero".to_string(),
                ));
            }

            let from_account = db.get_account(&from_id)?;
            let to_account = db.get_account(&to_id)?;
            if from_account.currency.to_uppercase() != to_account.currency.to_uppercase() {
                return Err(ControllerError::Validation(
                    "Transfers require both accounts to use the same currency".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
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
                return Err(ControllerError::Validation(
                    "Category cannot be empty".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            if amount <= 0 {
                return Err(ControllerError::Validation(
                    "Amount must be greater than zero".to_string(),
                ));
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

    /// Updates a transaction
    pub fn update_transaction(
        &self,
        id: String,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        is_expense: bool,
    ) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let id = validate_uuid(&id)?;
            let account_id = validate_uuid(&account_id)?;
            let category = validate_field_length(&category, MAX_CATEGORY_LENGTH, "Category")?;
            let category = sanitize_string(&category);

            if category.is_empty() {
                return Err(ControllerError::Validation(
                    "Category cannot be empty".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            if amount <= 0 {
                return Err(ControllerError::Validation(
                    "Amount must be greater than zero".to_string(),
                ));
            }

            let transaction_type = if is_expense { "expense" } else { "income" };

            let transaction = Transaction::new(
                id,
                account_id,
                amount,
                category,
                description,
                date,
                transaction_type.to_string(),
                None,
            );

            db.update_transaction(&transaction)?;
            Ok(())
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
        let accounts = self.get_accounts()?;

        let currency_map: std::collections::HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let clp_rate = match self.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
            Ok(Some((r, _))) => r,
            _ => 1.0,
        };
        let rate = if clp_rate > 0.0 { clp_rate } else { 1.0 };

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
        };

        let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for tx in transactions {
            if tx.transaction_type == "expense" {
                let amount = normalize(tx.amount, &tx.account_id);
                *map.entry(tx.category).or_default() += amount;
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

    // ==================== Transaction Category Methods ====================

    /// Gets all categories of a specific type (expense or income)
    pub fn get_transaction_categories(
        &self,
        category_type: String,
    ) -> Result<Vec<TransactionCategory>, ControllerError> {
        if category_type != "expense" && category_type != "income" {
            return Err(ControllerError::Validation(
                "Category type must be 'expense' or 'income'".to_string(),
            ));
        }

        self.with_db(|db| {
            db.get_transaction_categories(&category_type)
                .map_err(ControllerError::Database)
        })
    }

    /// Adds a new transaction category
    pub fn add_transaction_category(
        &self,
        name: String,
        category_type: String,
    ) -> Result<String, ControllerError> {
        let validated_name = validate_field_length(&name, MAX_CATEGORY_LENGTH, "Category name")?;

        if validated_name.is_empty() {
            return Err(ControllerError::Validation(
                "Category name cannot be empty".to_string(),
            ));
        }

        if category_type != "expense" && category_type != "income" {
            return Err(ControllerError::Validation(
                "Category type must be 'expense' or 'income'".to_string(),
            ));
        }

        self.with_db(|db| {
            db.add_transaction_category(&validated_name, &category_type)
                .map_err(|e| match e {
                    DbError::Sqlite(RusqliteError::ExecuteReturnedResults) => {
                        ControllerError::Validation(
                            "Category with this name already exists".to_string(),
                        )
                    }
                    _ => ControllerError::Database(e),
                })
        })
    }

    /// Updates a category name
    pub fn update_transaction_category(
        &self,
        id: String,
        new_name: String,
    ) -> Result<(), ControllerError> {
        let validated_id = validate_uuid(&id)?;
        let validated_name = validate_field_length(&new_name, MAX_CATEGORY_LENGTH, "Category name")?;

        if validated_name.is_empty() {
            return Err(ControllerError::Validation(
                "Category name cannot be empty".to_string(),
            ));
        }

        self.with_db(|db| {
            db.update_transaction_category(&validated_id, &validated_name)
                .map_err(|e| match e {
                    DbError::Sqlite(RusqliteError::ExecuteReturnedResults) => {
                        ControllerError::Validation(
                            "Category with this name already exists".to_string(),
                        )
                    }
                    _ => ControllerError::Database(e),
                })
        })
    }

    /// Deletes a category (only if not default)
    pub fn delete_transaction_category(&self, id: String) -> Result<(), ControllerError> {
        let validated_id = validate_uuid(&id)?;

        self.with_db(|db| {
            db.delete_transaction_category(&validated_id)
                .map_err(|e| match e {
                    DbError::Sqlite(RusqliteError::ExecuteReturnedResults) => {
                        ControllerError::Validation(
                            "Cannot delete default category".to_string(),
                        )
                    }
                    _ => ControllerError::Database(e),
                })
        })
    }

    // ==================== Crypto Price Methods ====================

    /// Gets all unique coin IDs that need monitoring (Active Tickers + Wallet Holdings)
    pub fn get_monitored_coin_ids(&self) -> Result<Vec<String>, ControllerError> {
        // Preserve priority: tickers first, then wallets.
        let mut ids = Vec::new();
        let mut seen = HashSet::new();

        for id in self.get_active_ticker_ids() {
            if seen.insert(id.clone()) {
                ids.push(id);
            }
        }

        if let Ok(portfolio) = self.get_aggregated_portfolio() {
            for asset in portfolio {
                let coin_id = asset.coin_id;
                if seen.insert(coin_id.clone()) {
                    ids.push(coin_id);
                }
            }
        }

        Ok(ids)
    }

    /// Fetches cryptocurrency prices from CoinGecko
    /// Implements privacy padding: mixes requested coins with a default list up to the API limit (50).
    pub async fn get_crypto_prices(
        &self,
        coins: Vec<String>,
    ) -> Result<Vec<CryptoAsset>, ControllerError> {
        // CoinGecko limit is 50
        const MAX_BATCH_SIZE: usize = 50;

        let mut final_list = Vec::new();
        let mut seen = HashSet::new();
        let mut truncated = false;

        for coin in coins {
            if seen.insert(coin.clone()) {
                if final_list.len() < MAX_BATCH_SIZE {
                    final_list.push(coin);
                } else {
                    truncated = true;
                }
            }
        }

        if final_list.len() < MAX_BATCH_SIZE {
            // Smart padding: fill remaining slots with privacy coins.
            let padding = crypto::default_price_allowlist();

            for privacy_coin in padding {
                if final_list.len() >= MAX_BATCH_SIZE {
                    break;
                }
                if seen.insert(privacy_coin.clone()) {
                    final_list.push(privacy_coin);
                }
            }
        }

        if truncated {
            log::warn!(
                "Price request exceeds {} unique coins; truncating to limit",
                MAX_BATCH_SIZE
            );
        }

        crypto::fetch_crypto_prices(final_list)
            .await
            .map_err(ControllerError::Api)
    }

    /// Fetches CLP to USD exchange rate
    pub async fn get_clp_usd_rate(&self) -> Result<f64, ControllerError> {
        crypto::fetch_clp_usd_rate()
            .await
            .map_err(ControllerError::Api)
    }

    /// Saves exchange rate to cache
    pub fn save_exchange_rate(&self, pair: String, rate: f64) -> Result<(), ControllerError> {
        self.with_db(|db| {
            db.save_exchange_rate(&pair, rate)
                .map_err(ControllerError::Database)
        })
    }

    /// Loads cached exchange rate, even if stale
    pub fn load_exchange_rate_allow_stale(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, ControllerError> {
        self.with_db(|db| {
            db.load_exchange_rate(&pair)
                .map_err(ControllerError::Database)
        })
    }

    /// Loads cached exchange rate
    pub fn load_exchange_rate(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, ControllerError> {
        self.with_db(|db| {
            let cached = db
                .load_exchange_rate(&pair)
                .map_err(ControllerError::Database)?;

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

    /// Saves a daily portfolio snapshot (upsert by date)
    pub fn save_crypto_portfolio_snapshot(
        &self,
        total_value: f64,
        total_cost: f64,
    ) -> Result<(), ControllerError> {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.with_db(|db| {
            db.save_crypto_portfolio_snapshot(&date, total_value, total_cost)
                .map_err(ControllerError::Database)
        })
    }

    /// Loads portfolio snapshots for the last N days (inclusive)
    pub fn get_crypto_portfolio_snapshots(
        &self,
        days: i64,
    ) -> Result<Vec<(String, f64, f64)>, ControllerError> {
        let days = days.max(1);
        let start_date = Local::now()
            .date_naive()
            .checked_sub_signed(chrono::Duration::days(days - 1))
            .unwrap_or_else(|| Local::now().date_naive())
            .format("%Y-%m-%d")
            .to_string();
        self.with_db(|db| {
            db.load_crypto_portfolio_snapshots(&start_date)
                .map_err(ControllerError::Database)
        })
    }

    // ==================== Crypto Wallet Methods ====================

    /// Creates a new crypto wallet
    pub fn add_wallet(
        &self,
        name: String,
        category: String,
        icon: Option<String>,
    ) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let name = validate_field_length(&name, MAX_WALLET_NAME_LENGTH, "Wallet name")?;
            let name = sanitize_string(&name);

            if name.is_empty() {
                return Err(ControllerError::Validation(
                    "Wallet name cannot be empty".to_string(),
                ));
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

            // Check for duplicate wallet names
            let existing_wallets = db.get_wallets()?;
            if existing_wallets.iter().any(|w| w.name.eq_ignore_ascii_case(&name)) {
                return Err(ControllerError::Validation(format!(
                    "A wallet named '{}' already exists. Please choose a different name.",
                    name
                )));
            }

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
    /// Returns an error if the wallet has transactions
    pub fn delete_wallet(&self, id: String) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;

            // Check if wallet has transactions
            let transactions = db.get_wallet_transactions(&validated_id)?;
            if !transactions.is_empty() {
                return Err(ControllerError::Validation(format!(
                    "Cannot delete wallet with {} transaction{}. Please delete all transactions first.",
                    transactions.len(),
                    if transactions.len() == 1 { "" } else { "s" }
                )));
            }

            db.delete_wallet(&validated_id)?;
            log_security_event(SecurityEvent::WalletDeleted, None);
            Ok(())
        })
    }

    /// Updates a wallet's name
    pub fn update_wallet_name(&self, id: String, new_name: String) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            let validated_name = validate_field_length(&new_name, MAX_WALLET_NAME_LENGTH, "Wallet name")?;
            let sanitized_name = sanitize_string(&validated_name);

            if sanitized_name.is_empty() {
                return Err(ControllerError::Validation(
                    "Wallet name cannot be empty".to_string(),
                ));
            }

            // Check for duplicate names
            let existing_wallets = db.get_wallets()?;
            for wallet in existing_wallets {
                if wallet.id != validated_id && wallet.name.eq_ignore_ascii_case(&sanitized_name) {
                    return Err(ControllerError::Validation(
                        "A wallet with this name already exists".to_string(),
                    ));
                }
            }

            // Get current wallet to preserve other fields
            let mut wallet = db.get_wallet(&validated_id)?
                .ok_or_else(|| ControllerError::Validation("Wallet not found".to_string()))?;

            // Update only the name
            wallet.name = sanitized_name;

            db.update_wallet(&wallet)?;
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
        fee_coin_id: Option<String>,
        fee_amount: Option<f64>,
        date: String,
        notes: Option<String>,
    ) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let wallet_id = wallet_id.trim().to_string();
            if wallet_id.is_empty() {
                return Err(ControllerError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
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

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let valid_types = ["buy", "sell", "transfer_in", "transfer_out", "swap"];
            if !valid_types.contains(&transaction_type.as_str()) {
                return Err(ControllerError::Validation(format!(
                    "Invalid transaction type. Must be one of: {}",
                    valid_types.join(", ")
                )));
            }

            if transaction_type == "swap" {
                return Err(ControllerError::Validation(
                    "Swap requires paired transactions. Use the swap flow.".to_string(),
                ));
            }

            let price = if transaction_type == "buy" || transaction_type == "sell" {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => {
                        return Err(ControllerError::Validation(
                            "Price per coin is required and must be greater than zero".to_string(),
                        ))
                    }
                }
            } else {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => None,
                }
            };

            // 2. Validate Sufficient Funds (Prevent Negative Balance)
            let is_outflow = transaction_type == "sell"
                || transaction_type == "transfer_out"
                || transaction_type == "swap";

            if is_outflow {
                validate_sufficient_balance(db, &wallet_id, &coin_id, &symbol, amount, &date, None)?;
            }

            // Validate fee balance
            let fee_context = FeeBalanceContext {
                db,
                wallet_id: &wallet_id,
                main_coin_id: &coin_id,
                main_symbol: &symbol,
                main_amount: amount,
                is_outflow,
                date: &date,
                exclude_tx_id: None,
            };
            validate_fee_balance(fee_context, fee_coin_id.as_deref(), fee_amount)?;

            log_security_event(
                SecurityEvent::CryptoTransactionCreated,
                Some(&transaction_type),
            );

            let id = Uuid::new_v4().to_string();
            let mut transaction = CryptoTransaction::new(
                id.clone(),
                wallet_id,
                coin_id.to_lowercase(),
                symbol.to_uppercase(),
                transaction_type,
                amount,
                price,
                fee,
                date,
                notes,
            );
            transaction.fee_coin_id = fee_coin_id;
            transaction.fee_amount = fee_amount;

            db.create_crypto_transaction(&transaction)?;
            Ok(id)
        })
    }

    /// Adds a transfer between two wallets as a paired outflow/inflow transaction
    #[allow(clippy::too_many_arguments)]
    pub fn add_crypto_transfer(
        &self,
        from_wallet_id: String,
        to_wallet_id: String,
        coin_id: String,
        symbol: String,
        from_amount: f64,
        to_amount: f64,
        fee: Option<f64>,
        fee_coin_id: Option<String>,
        fee_amount: Option<f64>,
        date: String,
        notes: Option<String>,
    ) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let from_wallet_id = from_wallet_id.trim().to_string();
            let to_wallet_id = to_wallet_id.trim().to_string();
            if from_wallet_id.is_empty() || to_wallet_id.is_empty() {
                return Err(ControllerError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
            }
            if from_wallet_id == to_wallet_id {
                return Err(ControllerError::Validation(
                    "Source and destination wallets must be different".to_string(),
                ));
            }

            if db
                .get_wallet(&from_wallet_id)
                .map_err(ControllerError::Database)?
                .is_none()
            {
                return Err(ControllerError::Validation(
                    "Source wallet not found".to_string(),
                ));
            }
            if db
                .get_wallet(&to_wallet_id)
                .map_err(ControllerError::Database)?
                .is_none()
            {
                return Err(ControllerError::Validation(
                    "Destination wallet not found".to_string(),
                ));
            }

            let coin_id = validate_coin_id_str(&coin_id)?;
            let symbol = validate_symbol(&symbol)?;
            validate_positive_amount(from_amount, "From amount")?;
            validate_positive_amount(to_amount, "To amount")?;
            if to_amount > from_amount {
                return Err(ControllerError::Validation(
                    "To amount cannot exceed from amount".to_string(),
                ));
            }

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            let current_balance = db
                .get_wallet_coin_balance_at(&from_wallet_id, &coin_id, &date, None)
                .map_err(ControllerError::Database)?;
            if from_amount > current_balance {
                return Err(ControllerError::Validation(format!(
                    "Insufficient funds. Available: {:.8} {}",
                    current_balance, symbol
                )));
            }

            // Validate fee balance
            let fee_context = FeeBalanceContext {
                db,
                wallet_id: &from_wallet_id,
                main_coin_id: &coin_id,
                main_symbol: &symbol,
                main_amount: from_amount,
                is_outflow: true, // transfer_out is always outflow
                date: &date,
                exclude_tx_id: None,
            };
            validate_fee_balance(fee_context, fee_coin_id.as_deref(), fee_amount)?;

            // Specific validation for transfer: TO amount should match FROM when using same-coin fee
            if let (Some(fee_coin), Some(_)) = (fee_coin_id.as_deref(), fee_amount)
                && fee_coin == coin_id
                && to_amount < from_amount
            {
                return Err(ControllerError::Validation(
                    "When using a same-coin network fee, keep the TO amount equal to FROM (the fee is recorded separately)".to_string(),
                ));
            }

            let (total_amount, total_cost) = db
                .get_wallet_coin_state_at(&from_wallet_id, &coin_id, &date)
                .map_err(ControllerError::Database)?;
            let avg_price = if total_amount > 0.0 {
                total_cost / total_amount
            } else {
                0.0
            };
            let transfer_price = if avg_price > 0.0 {
                Some(avg_price)
            } else {
                None
            };

            log_security_event(
                SecurityEvent::CryptoTransactionCreated,
                Some("transfer"),
            );

            let source_id = Uuid::new_v4().to_string();
            let target_id = Uuid::new_v4().to_string();

            let source = CryptoTransaction {
                id: source_id.clone(),
                wallet_id: from_wallet_id,
                coin_id: coin_id.clone(),
                symbol: symbol.clone(),
                transaction_type: "transfer_out".to_string(),
                amount: from_amount,
                price_per_coin: None,
                fee: None,
                fee_coin_id: fee_coin_id.clone(),
                fee_amount,
                date: date.clone(),
                notes: notes.clone(),
                related_tx_id: Some(target_id.clone()),
            };

            let target = CryptoTransaction {
                id: target_id.clone(),
                wallet_id: to_wallet_id,
                coin_id,
                symbol,
                transaction_type: "transfer_in".to_string(),
                amount: to_amount,
                price_per_coin: transfer_price,
                fee,
                fee_coin_id: None,
                fee_amount: None,
                date,
                notes,
                related_tx_id: Some(source_id.clone()),
            };

            db.create_crypto_transaction(&source)?;
            if let Err(err) = db.create_crypto_transaction(&target) {
                // Attempt rollback
                if let Err(rollback_err) = db.delete_crypto_transaction(&source_id) {
                    log::error!(
                        "Failed to rollback transfer source transaction {}: {:?}",
                        source_id, rollback_err
                    );
                }
                return Err(ControllerError::Database(err));
            }

            Ok(source_id)
        })
    }

    /// Adds a swap as a paired outflow/inflow transaction with shared cost basis
    #[allow(clippy::too_many_arguments)]
    pub fn add_crypto_swap(
        &self,
        wallet_id: String,
        from_coin_id: String,
        from_symbol: String,
        from_amount: f64,
        to_coin_id: String,
        to_symbol: String,
        to_amount: f64,
        fee: Option<f64>,
        fee_coin_id: Option<String>,
        fee_amount: Option<f64>,
        date: String,
        notes: Option<String>,
    ) -> Result<String, ControllerError> {
        self.with_db(|db| {
            let wallet_id = wallet_id.trim().to_string();
            if wallet_id.is_empty() {
                return Err(ControllerError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
            }

            let from_coin_id = validate_coin_id_str(&from_coin_id)?;
            let to_coin_id = validate_coin_id_str(&to_coin_id)?;
            if from_coin_id == to_coin_id {
                return Err(ControllerError::Validation(
                    "Swap requires two different assets".to_string(),
                ));
            }

            let from_symbol = validate_symbol(&from_symbol)?;
            let to_symbol = validate_symbol(&to_symbol)?;
            validate_positive_amount(from_amount, "From amount")?;
            validate_positive_amount(to_amount, "To amount")?;

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            // Validate sufficient funds for the source asset
            validate_sufficient_balance(db, &wallet_id, &from_coin_id, &from_symbol, from_amount, &date, None)?;

            // Validate fee balance (swap has special logic for fee_coin == to_coin)
            if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id.as_deref(), fee_amount) {
                if fee_coin == from_coin_id {
                    // Fee in source coin: validate from_amount + fee <= from_balance
                    let total_required = from_amount + fee_amt;
                    validate_sufficient_balance(db, &wallet_id, &from_coin_id, &from_symbol, total_required, &date, None)?;
                } else if fee_coin == to_coin_id {
                    // Fee in target coin: validate fee <= to_amount + existing_to_balance
                    let to_balance = db
                        .get_wallet_coin_balance_at(&wallet_id, fee_coin, &date, None)
                        .map_err(ControllerError::Database)?;
                    if fee_amt > to_amount + to_balance {
                        return Err(ControllerError::Validation(
                            "Fee amount exceeds available output balance".to_string(),
                        ));
                    }
                } else {
                    // Fee in different coin: validate fee <= fee_balance
                    validate_sufficient_balance(db, &wallet_id, fee_coin, fee_coin, fee_amt, &date, None)?;
                }
            }

            log_security_event(
                SecurityEvent::CryptoTransactionCreated,
                Some("swap"),
            );

            let source_id = Uuid::new_v4().to_string();
            let target_id = Uuid::new_v4().to_string();

            let source = CryptoTransaction {
                id: source_id.clone(),
                wallet_id: wallet_id.clone(),
                coin_id: from_coin_id,
                symbol: from_symbol,
                transaction_type: "swap".to_string(),
                amount: from_amount,
                price_per_coin: None,
                fee,
                fee_coin_id: fee_coin_id.clone(),
                fee_amount,
                date: date.clone(),
                notes,
                related_tx_id: Some(target_id.clone()),
            };

            let target = CryptoTransaction {
                id: target_id.clone(),
                wallet_id,
                coin_id: to_coin_id,
                symbol: to_symbol,
                transaction_type: "transfer_in".to_string(),
                amount: to_amount,
                price_per_coin: None,
                fee: None,
                fee_coin_id: None,
                fee_amount: None,
                date,
                notes: None,
                related_tx_id: Some(source_id.clone()),
            };

            db.create_crypto_transaction(&source)?;
            if let Err(err) = db.create_crypto_transaction(&target) {
                // Attempt rollback
                if let Err(rollback_err) = db.delete_crypto_transaction(&source_id) {
                    log::error!(
                        "Failed to rollback swap source transaction {}: {:?}",
                        source_id, rollback_err
                    );
                }
                return Err(ControllerError::Database(err));
            }

            Ok(source_id)
        })
    }

    /// Gets wallet transactions
    pub fn get_wallet_transactions(
        &self,
        wallet_id: String,
    ) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&wallet_id)?;
            db.get_wallet_transactions(&validated_id)
                .map_err(ControllerError::Database)
        })
    }

    /// Gets a crypto transaction by ID
    pub fn get_crypto_transaction(
        &self,
        id: String,
    ) -> Result<Option<CryptoTransaction>, ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.get_crypto_transaction(&validated_id)
                .map_err(ControllerError::Database)
        })
    }

    /// Gets crypto transactions for a specific coin
    pub fn get_crypto_transactions_by_coin(
        &self,
        coin_id: String,
    ) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.with_db(|db| {
            let validated = validate_coin_id_str(&coin_id)?;
            db.get_crypto_transactions_by_coin(&validated)
                .map_err(ControllerError::Database)
        })
    }

    /// Updates a crypto transaction's editable fields
    #[allow(clippy::too_many_arguments)]
    pub fn update_crypto_transaction(
        &self,
        id: String,
        amount: f64,
        price_per_coin: Option<f64>,
        fee: Option<f64>,
        fee_coin_id: Option<String>,
        fee_amount: Option<f64>,
        date: String,
        notes: Option<String>,
    ) -> Result<(), ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            let existing = db
                .get_crypto_transaction(&validated_id)
                .map_err(ControllerError::Database)?;
            let existing = match existing {
                Some(tx) => tx,
                None => {
                    return Err(ControllerError::Validation(
                        "Transaction not found".to_string(),
                    ))
                }
            };

            if existing.transaction_type == "swap" || existing.related_tx_id.is_some() {
                return Err(ControllerError::Validation(
                    "Editing paired transactions is not supported".to_string(),
                ));
            }

            validate_positive_amount(amount, "Amount")?;
            let price = if existing.transaction_type == "buy"
                || existing.transaction_type == "sell"
            {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => {
                        return Err(ControllerError::Validation(
                            "Price per coin is required and must be greater than zero".to_string(),
                        ))
                    }
                }
            } else {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => None,
                }
            };
            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            let is_outflow = existing.transaction_type == "sell"
                || existing.transaction_type == "transfer_out";

            let existing_type = existing.get_type().unwrap_or(CryptoTransactionType::Buy);

            let mut balance_excluding = db
                .get_wallet_coin_balance_at(&existing.wallet_id, &existing.coin_id, &date, None)
                .map_err(ControllerError::Database)?;
            match existing_type {
                CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                    balance_excluding -= existing.amount;
                }
                CryptoTransactionType::Sell
                | CryptoTransactionType::TransferOut
                | CryptoTransactionType::Swap => {
                    balance_excluding += existing.amount;
                }
            }
            if existing.fee_coin_id.as_deref() == Some(existing.coin_id.as_str())
                && let Some(fee_amt) = existing.fee_amount
            {
                balance_excluding += fee_amt;
            }

            // Validate sufficient balance for outflows
            if is_outflow && amount > balance_excluding {
                return Err(ControllerError::Validation(format!(
                    "Insufficient funds. Available: {:.8} {}",
                    balance_excluding, existing.symbol
                )));
            }

            // Validate fee balance (excluding this transaction)
            if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id.as_deref(), fee_amount) {
                let mut fee_balance_excluding = if fee_coin == existing.coin_id {
                    balance_excluding
                } else {
                    db.get_wallet_coin_balance_at(&existing.wallet_id, fee_coin, &date, None)
                        .map_err(ControllerError::Database)?
                };
                if existing.fee_coin_id.as_deref() == Some(fee_coin)
                    && let Some(existing_fee_amt) = existing.fee_amount
                {
                    fee_balance_excluding += existing_fee_amt;
                }
                if fee_coin == existing.coin_id {
                    if is_outflow {
                        let total_required = amount + fee_amt;
                        if total_required > fee_balance_excluding {
                            return Err(ControllerError::Validation(format!(
                                "Insufficient funds for fee. Available: {:.8} {}",
                                fee_balance_excluding, existing.symbol
                            )));
                        }
                    } else {
                        let total_available = fee_balance_excluding + amount;
                        if fee_amt > total_available {
                            return Err(ControllerError::Validation(
                                "Fee amount exceeds available balance".to_string(),
                            ));
                        }
                    }
                } else if fee_amt > fee_balance_excluding {
                    return Err(ControllerError::Validation(format!(
                        "Insufficient funds for fee. Available: {:.8} {}",
                        fee_balance_excluding, fee_coin
                    )));
                }
            }

            db.update_crypto_transaction_fields(
                &validated_id,
                amount,
                price,
                fee,
                fee_coin_id.as_deref(),
                fee_amount,
                &date,
                notes.as_deref(),
            )
            .map_err(ControllerError::Database)?;

            Ok(())
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
        self.with_db(|db| {
            db.get_aggregated_portfolio()
                .map_err(ControllerError::Database)
        })
    }

    /// Gets aggregated holdings for a specific wallet
    pub fn get_wallet_holdings(
        &self,
        wallet_id: String,
    ) -> Result<Vec<AggregatedAsset>, ControllerError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&wallet_id)?;
            db.get_wallet_aggregated_holdings(&validated_id)
                .map_err(ControllerError::Database)
        })
    }

    /// Gets the available balance for a specific coin in a wallet at a given date
    pub fn get_available_balance(
        &self,
        wallet_id: String,
        coin_id: String,
        _date: String, // Ignored - always uses current date
    ) -> Result<f64, ControllerError> {
        self.with_db(|db| {
            let validated_wallet_id = validate_uuid(&wallet_id)?;
            let validated_coin_id = validate_coin_id_str(&coin_id)?; // coin_id is NOT a UUID

            // Use current date to get balance up to today
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();

            db.get_wallet_coin_balance_at(
                &validated_wallet_id,
                &validated_coin_id,
                &today,
                None, // Don't exclude any transactions
            )
            .map_err(ControllerError::Database)
        })
    }

    // ==================== Habits Methods ====================

    // ==================== Habit Management ====================

    pub fn create_habit(
        &self,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
    ) -> std::result::Result<String, ControllerError> {
        if name.trim().is_empty() {
            return Err(ControllerError::Validation(
                "Habit name cannot be empty".to_string(),
            ));
        }

        // Validate color format (basic hex)
        let color_regex = Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap();
        if !color_regex.is_match(&color) {
            return Err(ControllerError::Validation(
                "Invalid color format. Use #RRGGBB".to_string(),
            ));
        }

        let category = normalize_habit_category(&category).ok_or_else(|| {
            ControllerError::Validation("Invalid habit category".to_string())
        })?;

        self.habit_service
            .create_habit(name, description, color, category)
            .map_err(ControllerError::Database)
    }

    pub fn get_habits(&self) -> std::result::Result<Vec<Habit>, ControllerError> {
        self.habit_service
            .get_habits()
            .map_err(ControllerError::Database)
    }

    /// Updates a habit
    pub fn update_habit(
        &self,
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
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

        let category = normalize_habit_category(&category).ok_or_else(|| {
            ControllerError::Validation("Invalid habit category".to_string())
        })?;

        self.habit_service
            .update_habit(id, name, description, color, category, is_archived)
            .map_err(ControllerError::Database)
    }

    pub fn archive_habit(&self, id: String) -> std::result::Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }
        self.habit_service
            .archive_habit(id)
            .map_err(ControllerError::Database)
    }

    /// Deletes a habit
    pub fn delete_habit(&self, id: String) -> std::result::Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }
        self.habit_service
            .delete_habit(id)
            .map_err(ControllerError::Database)
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

    /// Gets all habit logs (optimized for bulk operations like streak calculation)
    /// This avoids N+1 query problems by fetching all logs at once
    pub fn get_all_habit_logs(&self) -> std::result::Result<Vec<HabitLog>, ControllerError> {
        // Use a wide date range to cover all reasonable dates
        self.get_habit_logs("1970-01-01".to_string(), "2100-01-01".to_string())
    }

    /// Gets habit analytics: weekday efficiency and monthly trend
    pub fn get_habit_analytics(&self, days: i32) -> Result<HabitAnalytics, ControllerError> {
        let today = chrono::Local::now().date_naive();
        let start_date = today
            .checked_sub_signed(chrono::Duration::days(days as i64))
            .unwrap_or(today);

        let logs = self.get_habit_logs(
            start_date.format("%Y-%m-%d").to_string(),
            today.format("%Y-%m-%d").to_string(),
        )?;

        // ==================== Weekday Efficiency ====================
        // Count completions per weekday
        let mut weekday_counts: [i32; 7] = [0; 7]; // Mon=0, Tue=1, ..., Sun=6
        let mut weekday_occurrences: [i32; 7] = [0; 7];

        // Count how many times each weekday appears in the range
        let mut cursor = start_date;
        while cursor <= today {
            let weekday_idx = cursor.weekday().num_days_from_monday() as usize;
            weekday_occurrences[weekday_idx] += 1;
            cursor = cursor.succ_opt().unwrap_or(cursor);
            if cursor == today && cursor == start_date {
                break; // Single day edge case
            }
            if cursor > today {
                break;
            }
        }

        // Count habit completions per weekday
        for log in &logs {
            if let Ok(date) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d") {
                let weekday_idx = date.weekday().num_days_from_monday() as usize;
                weekday_counts[weekday_idx] += 1;
            }
        }

        // Calculate averages
        let weekday_avgs: Vec<(usize, f32)> = (0..7)
            .map(|i| {
                let avg = if weekday_occurrences[i] > 0 {
                    weekday_counts[i] as f32 / weekday_occurrences[i] as f32
                } else {
                    0.0
                };
                (i, avg)
            })
            .collect();

        let max_avg = weekday_avgs
            .iter()
            .map(|(_, avg)| *avg)
            .fold(0.0_f32, f32::max);

        let day_names = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        let day_shorts = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

        let weekday_data: Vec<WeekdayEfficiency> = weekday_avgs
            .iter()
            .map(|(i, avg)| {
                let bar_height = if max_avg > 0.0 {
                    (*avg / max_avg) * 100.0
                } else {
                    0.0
                };
                WeekdayEfficiency {
                    day_name: day_names[*i].to_string(),
                    day_short: day_shorts[*i].to_string(),
                    avg_count: *avg,
                    is_best: (*avg - max_avg).abs() < 0.001 && max_avg > 0.0,
                    bar_height_percent: bar_height,
                }
            })
            .collect();

        // ==================== Monthly Trend ====================
        // Show last 12 months including current month (rolling 12-month window)
        // Example: Dec 2024 shows Jan 2024 - Dec 2024, Jan 2025 shows Feb 2024 - Jan 2025
        let current_year = today.year();
        let current_month = today.month();

        // Calculate start month (11 months back)
        let (start_year, start_month) = if current_month <= 11 {
            (current_year - 1, current_month + 1)
        } else {
            (current_year, current_month - 11)
        };

        let twelve_months_ago_start = NaiveDate::from_ymd_opt(start_year, start_month, 1)
            .unwrap_or(start_date);

        // Group by month and calculate average habits per day
        let mut monthly_data_map: std::collections::BTreeMap<(i32, u32), (i32, i32)> =
            std::collections::BTreeMap::new(); // (year, month) -> (habit_count, days_in_range)

        // Count days per month in the last 12 months
        let mut cursor = twelve_months_ago_start;
        while cursor <= today {
            let key = (cursor.year(), cursor.month());
            monthly_data_map.entry(key).or_insert((0, 0)).1 += 1;
            cursor = cursor.succ_opt().unwrap_or(cursor);
            if cursor > today {
                break;
            }
        }

        // Count habits per month in the last 12 months
        for log in &logs {
            if let Ok(date) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d")
                && date >= twelve_months_ago_start
            {
                let key = (date.year(), date.month());
                if let Some(entry) = monthly_data_map.get_mut(&key) {
                    entry.0 += 1;
                }
            }
        }

        // Convert to sorted vec with averages
        let month_names = [
            "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];

        let monthly_avgs: Vec<((i32, u32), f32, String)> = monthly_data_map
            .iter()
            .map(|((year, month), (count, days))| {
                let avg = if *days > 0 {
                    *count as f32 / *days as f32
                } else {
                    0.0
                };
                let label = format!("{} {}", month_names[*month as usize], year % 100);
                ((*year, *month), avg, label)
            })
            .collect();

        if monthly_avgs.is_empty() {
            return Ok(HabitAnalytics {
                weekday_data,
                monthly_data: vec![],
                monthly_path: "M 0 50 L 100 50".to_string(),
            });
        }

        let min_avg = monthly_avgs
            .iter()
            .map(|(_, avg, _)| *avg)
            .fold(f32::MAX, f32::min);
        let max_monthly_avg = monthly_avgs
            .iter()
            .map(|(_, avg, _)| *avg)
            .fold(0.0_f32, f32::max);

        let range = (max_monthly_avg - min_avg).max(0.1); // Avoid division by zero
        let len = monthly_avgs.len();

        let monthly_data: Vec<MonthlyTrendPoint> = monthly_avgs
            .iter()
            .enumerate()
            .map(|(i, (_, avg, label))| {
                let x = if len > 1 {
                    (i as f32 / (len - 1) as f32) * 100.0
                } else {
                    50.0
                };
                let y_ratio = (*avg - min_avg) / range;
                let y = 100.0 - (10.0 + y_ratio * 80.0); // 10% padding top/bottom

                MonthlyTrendPoint {
                    month_name: label.clone(),
                    avg_per_day: *avg,
                    x_percent: x,
                    y_percent: y,
                }
            })
            .collect();

        // Generate SVG path for monthly trend (simple lines)
        let mut path = String::new();
        for (i, point) in monthly_data.iter().enumerate() {
            if i == 0 {
                path.push_str(&format!("M {:.2} {:.2}", point.x_percent, point.y_percent));
            } else {
                path.push_str(&format!(" L {:.2} {:.2}", point.x_percent, point.y_percent));
            }
        }

        if path.is_empty() {
            path = "M 0 50 L 100 50".to_string();
        }

        Ok(HabitAnalytics {
            weekday_data,
            monthly_data,
            monthly_path: path,
        })
    }

    /// Provides analytics summary (net worth history + expense breakdown)
    pub fn get_analytics_summary(
        &self,
        range: String,
    ) -> Result<AnalyticsSummary, ControllerError> {
        let balances = self.get_account_balances()?;
        let accounts = self.get_accounts()?;
        let transactions = self.get_transactions()?;

        // Currency Normalization Setup
        let currency_map: std::collections::HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let clp_rate = match self.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
            Ok(Some((r, _))) => r,
            _ => 1.0,
        };
        let rate = if clp_rate > 0.0 { clp_rate } else { 1.0 };

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
        };

        // Calculate Normalized Current Balance
        let current_balance: i64 = balances
            .iter()
            .map(|b| normalize(b.current_balance, &b.account_id))
            .sum();

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

        // Build daily deltas (Normalized)
        let mut delta_by_day: std::collections::HashMap<NaiveDate, i64> =
            std::collections::HashMap::new();
        let mut earliest_tx: Option<NaiveDate> = None;
        for tx in &transactions {
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d") {
                let raw_delta = match tx.transaction_type.as_str() {
                    "income" => tx.amount,
                    "expense" => -tx.amount,
                    _ => 0,
                };
                let delta = normalize(raw_delta, &tx.account_id);

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
        let safe_range = ((max_val - min_val) as f64).max(1.0);

        // Generate Smooth Path (Catmull-Rom Spline -> Cubic Bezier)
        // Normalize points to 0..100 space
        let points: Vec<(f64, f64)> = values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = (i as f64 / (values.len().max(2) - 1) as f64) * 100.0;
                let y_ratio = (v - min_val) as f64 / safe_range;
                let y = 100.0 - (5.0 + (y_ratio * 90.0)); // 5% padding
                (x, y)
            })
            .collect();

        let mut path_cmd = String::new();
        if !points.is_empty() {
            path_cmd.push_str(&format!("M {:.2} {:.2}", points[0].0, points[0].1));

            for i in 0..points.len() - 1 {
                let p0 = if i == 0 { points[0] } else { points[i - 1] };
                let p1 = points[i];
                let p2 = points[i + 1];
                let p3 = if i + 2 < points.len() {
                    points[i + 2]
                } else {
                    p2
                };

                let cp1x = p1.0 + (p2.0 - p0.0) / 6.0;
                let cp1y = p1.1 + (p2.1 - p0.1) / 6.0;

                let cp2x = p2.0 - (p3.0 - p1.0) / 6.0;
                let cp2y = p2.1 - (p3.1 - p1.1) / 6.0;

                path_cmd.push_str(&format!(
                    " C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                    cp1x, cp1y, cp2x, cp2y, p2.0, p2.1
                ));
            }
        } else {
            path_cmd = "M 0 50 L 100 50".to_string();
        }

        // Expense donut (Normalized)
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
                let amount = normalize(tx.amount, &tx.account_id);
                *expenses.entry(tx.category.to_uppercase()).or_insert(0) += amount;
            }
        }

        let total_expense: i64 = expenses.values().sum();
        let mut expense_slices: Vec<ExpenseSlice> = Vec::new();
        if total_expense > 0 {
            let mut by_amount: Vec<(String, i64)> = expenses.into_iter().collect();
            by_amount.sort_by(|a, b| b.1.cmp(&a.1));

            let colors = [
                "#8b5cf6", "#ec4899", "#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#6366f1",
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
    pub fn get_net_worth_history(
        &self,
        range: &str,
    ) -> Result<(String, String, String, String), ControllerError> {
        let accounts = self.get_accounts()?;
        let transactions = self.get_transactions()?;

        // Currency Normalization Setup
        let currency_map: std::collections::HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let clp_rate = match self.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
            Ok(Some((r, _))) => r,
            _ => 1.0,
        };
        let rate = if clp_rate > 0.0 { clp_rate } else { 1.0 };

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
        };

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

                let amount_delta = normalize(acc.initial_balance, &acc.id);

                events.push(FinancialEvent { date, amount_delta });
            }
        }

        // 2. Add Transaction Events
        for tx in &transactions {
            let raw_delta = match tx.transaction_type.as_str() {
                "income" => tx.amount,
                "expense" => -tx.amount,
                _ => 0,
            };

            if raw_delta == 0 {
                continue;
            }

            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d") {
                let amount_delta = normalize(raw_delta, &tx.account_id);
                events.push(FinancialEvent { date, amount_delta });
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
            return Ok((
                "M 0 50 L 100 50".to_string(),
                net_worth_formatted,
                "$ 0.00".to_string(),
                "$ 0.00".to_string(),
            ));
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
