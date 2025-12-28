//! Application Controller for Sanctum
//!
//! This module provides a pure Rust API that can be consumed by any UI framework (Slint, etc.)
//! All Tauri-specific code has been removed.

use crate::db::{Database, DbError};
use crate::models::{
    Account, AccountBalance, AggregatedAsset, BalanceSummary, CryptoAsset, CryptoCatalogCoin,
    CryptoTransaction, CryptoWallet, Habit, HabitLog, Transaction, TransactionCategory,
};
use crate::security_log::{SecurityEvent, log_auth_failure, log_security_event};
pub use crate::features::finance::{AnalyticsSummary, ExpenseSlice};
pub use crate::features::crypto::{
    SETTING_AUTO_FETCH, SETTING_CRYPTO_CUSTOM_COINS, SETTING_CRYPTO_FAVORITE_COINS,
    SETTING_CRYPTO_HIDDEN_COINS, SETTING_CRYPTO_LAST_COIN_ID, SETTING_CRYPTO_LAST_UPDATED,
    SETTING_CRYPTO_LAST_WALLET_ID, SETTING_TICKER_COINS,
};
use crate::services::charts::ChartsService;
use crate::features::crypto::{CryptoError, CryptoService};
use crate::features::finance::{FinanceError, FinanceService};
use crate::features::habits::HabitService;
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use rusqlite::Connection;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::fs::{self, Permissions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use slint::Image;

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
    last_db_path: Option<String>,
}

pub struct AppController {
    db: Arc<Mutex<Option<Database>>>,
    pub finance_service: FinanceService,
    charts_service: ChartsService,
    crypto_service: CryptoService,
    pub habit_service: HabitService,
    app_data_dir: PathBuf,
}

impl AppController {
    pub fn new(data_dir: PathBuf) -> Self {
        // Initialize with None as vault is locked
        let db = Arc::new(Mutex::new(None));
        let finance_service = FinanceService::new(db.clone());
        let crypto_service = CryptoService::new(db.clone());
        // HabitService needs access to the same (potentially empty) db lock
        let habit_service = HabitService::new(db.clone());
        let charts_service = ChartsService::new();

        Self {
            db,
            finance_service,
            charts_service,
            crypto_service,
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

    // ==================== Chart Rendering ====================

    pub fn render_habit_radar_chart(
        &self,
        categories: &[(String, String, f32)],
    ) -> Option<Image> {
        self.charts_service.render_habit_radar_chart(categories)
    }

    pub fn render_weekday_efficiency_chart(
        &self,
        weekdays: &[(String, f32, bool)],
    ) -> Option<Image> {
        self.charts_service.render_weekday_efficiency_chart(weekdays)
    }

    pub fn render_portfolio_distribution_chart(&self, data: &[(String, f64)]) -> Option<Image> {
        self.charts_service.render_portfolio_distribution_chart(data)
    }

    pub fn render_portfolio_trend_chart(
        &self,
        data: &[(String, f64, f64)],
    ) -> Option<Image> {
        self.charts_service.render_portfolio_trend_chart(data)
    }

    pub fn chart_color_for_symbol(&self, symbol: &str, index: usize) -> (u8, u8, u8) {
        self.charts_service.chart_color_for_symbol(symbol, index)
    }

    // ==================== Settings Methods ====================

    /// Gets an application setting
    pub fn get_app_setting(&self, key: &str) -> Result<String, ControllerError> {
        self.crypto_service
            .get_app_setting(key)
            .map_err(ControllerError::from)
    }

    /// Sets an application setting
    pub fn set_app_setting(&self, key: &str, value: &str) -> Result<(), ControllerError> {
        self.crypto_service
            .set_app_setting(key, value)
            .map_err(ControllerError::from)
    }

    /// Gets active ticker IDs from settings or default
    pub fn get_active_ticker_ids(&self) -> Vec<String> {
        self.crypto_service.get_active_ticker_ids()
    }

    /// Saves active ticker IDs to settings
    pub fn save_active_ticker_ids(&self, ids: Vec<String>) -> Result<(), ControllerError> {
        self.crypto_service
            .save_active_ticker_ids(ids)
            .map_err(ControllerError::from)
    }

    /// Loads custom coins configured by the user
    pub fn get_custom_coin_catalog(&self) -> Result<Vec<CryptoCatalogCoin>, ControllerError> {
        self.crypto_service
            .get_custom_coin_catalog()
            .map_err(ControllerError::from)
    }

    /// Loads hidden coin IDs for the catalog UI
    pub fn get_hidden_coin_ids(&self) -> Vec<String> {
        self.crypto_service.get_hidden_coin_ids()
    }

    /// Loads favorite coin IDs for the catalog UI
    pub fn get_favorite_coin_ids(&self) -> Vec<String> {
        self.crypto_service.get_favorite_coin_ids()
    }

    /// Marks or unmarks a coin as favorite
    pub fn set_favorite_coin(&self, id: String, favorite: bool) -> Result<(), ControllerError> {
        self.crypto_service
            .set_favorite_coin(id, favorite)
            .map_err(ControllerError::from)
    }

    /// Returns the full coin catalog (defaults + custom)
    pub fn get_coin_catalog(&self) -> Result<Vec<CryptoCatalogCoin>, ControllerError> {
        self.crypto_service
            .get_coin_catalog()
            .map_err(ControllerError::from)
    }

    /// Adds a custom coin to the catalog
    pub fn add_custom_coin(
        &self,
        id: String,
        name: String,
        symbol: String,
    ) -> Result<(), ControllerError> {
        self.crypto_service
            .add_custom_coin(id, name, symbol)
            .map_err(ControllerError::from)
    }

    /// Deletes a custom coin from the catalog
    pub fn delete_custom_coin(&self, id: String) -> Result<(), ControllerError> {
        self.crypto_service
            .delete_custom_coin(id)
            .map_err(ControllerError::from)
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
        self.finance_service
            .create_account(
                name,
                account_type,
                currency,
                initial_balance,
                color,
                icon,
            )
            .map_err(ControllerError::from)
    }

    /// Gets all accounts
    pub fn get_accounts(&self) -> Result<Vec<Account>, ControllerError> {
        self.finance_service
            .get_accounts()
            .map_err(ControllerError::from)
    }

    /// Gets all account balances
    pub fn get_account_balances(&self) -> Result<Vec<AccountBalance>, ControllerError> {
        self.finance_service
            .get_account_balances()
            .map_err(ControllerError::from)
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
        self.finance_service
            .update_account(
                id,
                name,
                account_type,
                currency,
                initial_balance,
                color,
                icon,
            )
            .map_err(ControllerError::from)
    }

    /// Archives an account (soft delete)
    pub fn archive_account(&self, id: String) -> Result<(), ControllerError> {
        self.finance_service
            .archive_account(id)
            .map_err(ControllerError::from)
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
        self.finance_service
            .transfer_funds(from_account_id, to_account_id, amount, description, date)
            .map_err(ControllerError::from)
    }

    /// Updates an existing transfer between accounts
    pub fn update_transfer(
        &self,
        id: String,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<(), ControllerError> {
        self.finance_service
            .update_transfer(id, from_account_id, to_account_id, amount, description, date)
            .map_err(ControllerError::from)
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
        self.finance_service
            .add_transaction(account_id, amount, category, description, date, is_expense)
            .map_err(ControllerError::from)
    }

    /// Updates a transaction
    #[allow(clippy::too_many_arguments)]
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
        self.finance_service
            .update_transaction(
                id,
                account_id,
                amount,
                category,
                description,
                date,
                is_expense,
            )
            .map_err(ControllerError::from)
    }

    /// Gets all transactions
    pub fn get_transactions(&self) -> Result<Vec<Transaction>, ControllerError> {
        self.finance_service
            .get_transactions()
            .map_err(ControllerError::from)
    }

    /// Gets balance summary
    pub fn get_balance(&self) -> Result<BalanceSummary, ControllerError> {
        self.finance_service
            .get_balance()
            .map_err(ControllerError::from)
    }

    /// Gets expenses aggregated by category
    pub fn get_expenses_by_category(&self) -> Result<Vec<(String, i64)>, ControllerError> {
        self.finance_service
            .get_expenses_by_category()
            .map_err(ControllerError::from)
    }

    /// Deletes a transaction
    pub fn delete_transaction(&self, id: String) -> Result<(), ControllerError> {
        self.finance_service
            .delete_transaction(id)
            .map_err(ControllerError::from)
    }

    // ==================== Transaction Category Methods ====================

    /// Gets all categories of a specific type (expense or income)
    pub fn get_transaction_categories(
        &self,
        category_type: String,
    ) -> Result<Vec<TransactionCategory>, ControllerError> {
        self.finance_service
            .get_transaction_categories(category_type)
            .map_err(ControllerError::from)
    }

    /// Adds a new transaction category
    pub fn add_transaction_category(
        &self,
        name: String,
        category_type: String,
    ) -> Result<String, ControllerError> {
        self.finance_service
            .add_transaction_category(name, category_type)
            .map_err(ControllerError::from)
    }

    /// Updates a category name
    pub fn update_transaction_category(
        &self,
        id: String,
        new_name: String,
    ) -> Result<(), ControllerError> {
        self.finance_service
            .update_transaction_category(id, new_name)
            .map_err(ControllerError::from)
    }

    /// Deletes a category
    pub fn delete_transaction_category(&self, id: String) -> Result<(), ControllerError> {
        self.finance_service
            .delete_transaction_category(id)
            .map_err(ControllerError::from)
    }

    // ==================== Crypto Price Methods ====================

    /// Gets all unique coin IDs that need monitoring (Active Tickers + Wallet Holdings)
    pub fn get_monitored_coin_ids(&self) -> Result<Vec<String>, ControllerError> {
        self.crypto_service
            .get_monitored_coin_ids()
            .map_err(ControllerError::from)
    }

    /// Fetches cryptocurrency prices from CoinGecko
    /// Implements privacy padding: mixes requested coins with a default list up to the API limit (50).
    pub async fn get_crypto_prices(
        &self,
        coins: Vec<String>,
    ) -> Result<Vec<CryptoAsset>, ControllerError> {
        self.crypto_service
            .get_crypto_prices(coins)
            .await
            .map_err(ControllerError::from)
    }

    /// Fetches CLP to USD exchange rate
    pub async fn get_clp_usd_rate(&self) -> Result<f64, ControllerError> {
        self.crypto_service
            .get_clp_usd_rate()
            .await
            .map_err(ControllerError::from)
    }

    /// Saves exchange rate to cache
    pub fn save_exchange_rate(&self, pair: String, rate: f64) -> Result<(), ControllerError> {
        self.finance_service
            .save_exchange_rate(pair, rate)
            .map_err(ControllerError::from)
    }

    /// Loads cached exchange rate, even if stale
    pub fn load_exchange_rate_allow_stale(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, ControllerError> {
        self.finance_service
            .load_exchange_rate_allow_stale(pair)
            .map_err(ControllerError::from)
    }

    /// Loads cached exchange rate
    pub fn load_exchange_rate(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, ControllerError> {
        self.finance_service
            .load_exchange_rate(pair)
            .map_err(ControllerError::from)
    }

    /// Saves crypto prices to cache
    pub fn save_crypto_prices(&self, prices: Vec<CryptoAsset>) -> Result<(), ControllerError> {
        self.crypto_service
            .save_crypto_prices(prices)
            .map_err(ControllerError::from)
    }

    /// Loads cached crypto prices
    pub fn load_crypto_prices(&self) -> Result<Vec<CryptoAsset>, ControllerError> {
        self.crypto_service
            .load_crypto_prices()
            .map_err(ControllerError::from)
    }

    /// Saves a daily portfolio snapshot (upsert by date)
    pub fn save_crypto_portfolio_snapshot(
        &self,
        total_value: f64,
        total_cost: f64,
    ) -> Result<(), ControllerError> {
        self.crypto_service
            .save_crypto_portfolio_snapshot(total_value, total_cost)
            .map_err(ControllerError::from)
    }

    /// Loads portfolio snapshots for the last N days (inclusive)
    pub fn get_crypto_portfolio_snapshots(
        &self,
        days: i64,
    ) -> Result<Vec<(String, f64, f64)>, ControllerError> {
        self.crypto_service
            .get_crypto_portfolio_snapshots(days)
            .map_err(ControllerError::from)
    }

    // ==================== Crypto Wallet Methods ====================

    /// Creates a new crypto wallet
    pub fn add_wallet(
        &self,
        name: String,
        category: String,
        icon: Option<String>,
    ) -> Result<String, ControllerError> {
        self.crypto_service
            .add_wallet(name, category, icon)
            .map_err(ControllerError::from)
    }

    /// Gets all wallets
    pub fn get_wallets(&self) -> Result<Vec<CryptoWallet>, ControllerError> {
        self.crypto_service
            .get_wallets()
            .map_err(ControllerError::from)
    }

    /// Deletes a wallet
    /// Returns an error if the wallet has transactions
    pub fn delete_wallet(&self, id: String) -> Result<(), ControllerError> {
        self.crypto_service
            .delete_wallet(id)
            .map_err(ControllerError::from)
    }

    /// Updates a wallet's name
    pub fn update_wallet_name(&self, id: String, new_name: String) -> Result<(), ControllerError> {
        self.crypto_service
            .update_wallet_name(id, new_name)
            .map_err(ControllerError::from)
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
        self.crypto_service
            .add_crypto_transaction(
                wallet_id,
                coin_id,
                symbol,
                transaction_type,
                amount,
                price_per_coin,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
            )
            .map_err(ControllerError::from)
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
        self.crypto_service
            .add_crypto_transfer(
                from_wallet_id,
                to_wallet_id,
                coin_id,
                symbol,
                from_amount,
                to_amount,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
            )
            .map_err(ControllerError::from)
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
        self.crypto_service
            .add_crypto_swap(
                wallet_id,
                from_coin_id,
                from_symbol,
                from_amount,
                to_coin_id,
                to_symbol,
                to_amount,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
            )
            .map_err(ControllerError::from)
    }

    /// Gets wallet transactions
    pub fn get_wallet_transactions(
        &self,
        wallet_id: String,
    ) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.crypto_service
            .get_wallet_transactions(wallet_id)
            .map_err(ControllerError::from)
    }

    /// Gets a crypto transaction by ID
    pub fn get_crypto_transaction(
        &self,
        id: String,
    ) -> Result<Option<CryptoTransaction>, ControllerError> {
        self.crypto_service
            .get_crypto_transaction(id)
            .map_err(ControllerError::from)
    }

    /// Gets crypto transactions for a specific coin
    pub fn get_crypto_transactions_by_coin(
        &self,
        coin_id: String,
    ) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.crypto_service
            .get_crypto_transactions_by_coin(coin_id)
            .map_err(ControllerError::from)
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
        self.crypto_service
            .update_crypto_transaction(
                id,
                amount,
                price_per_coin,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
            )
            .map_err(ControllerError::from)
    }

    /// Deletes a crypto transaction
    pub fn delete_crypto_transaction(&self, id: String) -> Result<(), ControllerError> {
        self.crypto_service
            .delete_crypto_transaction(id)
            .map_err(ControllerError::from)
    }

    // ==================== Portfolio Aggregation Methods ====================

    /// Gets aggregated portfolio across all wallets
    pub fn get_aggregated_portfolio(&self) -> Result<Vec<AggregatedAsset>, ControllerError> {
        self.crypto_service
            .get_aggregated_portfolio()
            .map_err(ControllerError::from)
    }

    /// Gets aggregated holdings for a specific wallet
    pub fn get_wallet_holdings(
        &self,
        wallet_id: String,
    ) -> Result<Vec<AggregatedAsset>, ControllerError> {
        self.crypto_service
            .get_wallet_holdings(wallet_id)
            .map_err(ControllerError::from)
    }

    /// Gets the available balance for a specific coin in a wallet at a given date
    pub fn get_available_balance(
        &self,
        wallet_id: String,
        coin_id: String,
        _date: String, // Ignored - always uses current date
    ) -> Result<f64, ControllerError> {
        self.crypto_service
            .get_available_balance(wallet_id, coin_id, _date)
            .map_err(ControllerError::from)
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
        self.finance_service
            .get_analytics_summary(range)
            .map_err(ControllerError::from)
    }

    /// Returns normalized SVG path commands (0-100 space) for net worth history and current net worth formatted
    /// Also returns min and max values formatted for labels
    pub fn get_net_worth_history(
        &self,
        range: &str,
    ) -> Result<(String, String, String, String), ControllerError> {
        self.finance_service
            .get_net_worth_history(range)
            .map_err(ControllerError::from)
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

}
