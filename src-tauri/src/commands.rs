use crate::crypto;
use crate::db::{Database, DbError};
use crate::models::{
    AggregatedAsset, BalanceSummary, CryptoAsset, CryptoHolding, CryptoTransaction, CryptoWallet,
    Habit, HabitLog, Transaction,
};
use crate::security_log::{SecurityEvent, log_auth_failure, log_security_event};
use chrono::NaiveDate;
use rusqlite::Connection;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::fs::{self, Permissions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

// ==================== Security: Field Length Limits ====================
const MAX_CATEGORY_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 512;
const MAX_NOTES_LENGTH: usize = 1024;
const MAX_WALLET_NAME_LENGTH: usize = 128;
const MAX_SYMBOL_LENGTH: usize = 16;
const MAX_ICON_LENGTH: usize = 32;
const MAX_PASSWORD_LENGTH: usize = 128;
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_HABIT_NAME_LENGTH: usize = 128;
const MAX_HABIT_DESCRIPTION_LENGTH: usize = 512;

/// Validates and truncates a string field to a maximum length
fn validate_field_length(
    value: &str,
    max_length: usize,
    field_name: &str,
) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.len() > max_length {
        return Err(format!(
            "{} exceeds maximum length of {} characters",
            field_name, max_length
        ));
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
fn validate_coin_id_str(coin_id: &str) -> Result<String, String> {
    crate::crypto::validate_coin_id(coin_id)
}

/// Validates a ticker/symbol (alphanumeric only)
fn validate_symbol(symbol: &str) -> Result<String, String> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return Err("Symbol cannot be empty".to_string());
    }
    if trimmed.len() > MAX_SYMBOL_LENGTH {
        return Err(format!(
            "Symbol exceeds maximum length of {} characters",
            MAX_SYMBOL_LENGTH
        ));
    }
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Symbol must be alphanumeric".to_string());
    }
    Ok(trimmed.to_uppercase())
}

/// Validates that a floating point value is finite and positive
fn validate_positive_amount(value: f64, field: &str) -> Result<f64, String> {
    if !value.is_finite() {
        return Err(format!("{} must be a finite number", field));
    }
    if value <= 0.0 {
        return Err(format!("{} must be greater than zero", field));
    }
    Ok(value)
}

/// Validates that an optional floating point value is finite and non-negative
fn validate_non_negative(value: Option<f64>, field: &str) -> Result<Option<f64>, String> {
    if let Some(v) = value {
        if !v.is_finite() {
            return Err(format!("{} must be a finite number", field));
        }
        if v < 0.0 {
            return Err(format!("{} cannot be negative", field));
        }
    }
    Ok(value)
}

/// Estado global para mantener la conexión a la base de datos
pub struct DbState {
    pub db: Mutex<Option<Database>>,
}

impl Default for DbState {
    fn default() -> Self {
        Self {
            db: Mutex::new(None),
        }
    }
}

impl DbState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the database is initialized, false otherwise (including on lock errors)
    pub fn is_initialized(&self) -> bool {
        self.db.lock().map(|guard| guard.is_some()).unwrap_or(false)
    }

    /// Checks session timeout and updates last activity
    pub fn check_session(&self) -> Result<(), String> {
        let db_lock = self.db.lock().map_err(|_| "Internal error".to_string())?;
        if let Some(db) = db_lock.as_ref() {
            db.check_session_timeout().map_err(|e| match e {
                DbError::SessionExpired => {
                    "Session expired due to inactivity. Please unlock the vault again.".to_string()
                }
                _ => e.to_string(),
            })?;
        }
        Ok(())
    }
}

/// Checks persistent rate limit using a temporary connection
fn check_persistent_rate_limit(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Ok(());
    }

    // Try to open without encryption to check rate limit table
    // This uses a separate unencrypted DB for rate limiting
    let rate_limit_path = db_path.with_extension("ratelimit");

    if let Ok(conn) = Connection::open(&rate_limit_path) {
        // Create table if not exists
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_attempts (
                vault_path TEXT PRIMARY KEY NOT NULL,
                failed_count INTEGER NOT NULL DEFAULT 0,
                locked_until TEXT,
                last_attempt TEXT NOT NULL
            )",
            [],
        );

        let vault_key = db_path.to_string_lossy().to_string();
        if let Err(DbError::RateLimited) = Database::check_rate_limit(&conn, &vault_key) {
            let remaining = Database::get_lockout_remaining(&conn, &vault_key).unwrap_or(0);
            return Err(format!(
                "Too many failed attempts. Try again in {} seconds",
                remaining
            ));
        }
    }

    Ok(())
}

/// Records a failed attempt in persistent storage
fn record_persistent_failed_attempt(db_path: &Path) {
    let rate_limit_path = db_path.with_extension("ratelimit");

    if let Ok(conn) = Connection::open(&rate_limit_path) {
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_attempts (
                vault_path TEXT PRIMARY KEY NOT NULL,
                failed_count INTEGER NOT NULL DEFAULT 0,
                locked_until TEXT,
                last_attempt TEXT NOT NULL
            )",
            [],
        );

        let vault_key = db_path.to_string_lossy().to_string();
        if let Ok((attempts, locked)) = Database::record_failed_attempt(&conn, &vault_key) {
            log_auth_failure(attempts, locked);
        }
    }
}

/// Resets persistent rate limit after successful auth
fn reset_persistent_rate_limit(db_path: &Path) {
    let rate_limit_path = db_path.with_extension("ratelimit");

    if let Ok(conn) = Connection::open(&rate_limit_path) {
        let vault_key = db_path.to_string_lossy().to_string();
        let _ = Database::reset_rate_limit(&conn, &vault_key);
    }
}

/// Validates a UUID string format
fn validate_uuid(id: &str) -> Result<String, String> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err("ID cannot be empty".to_string());
    }

    // Check if it's a valid UUID or a legacy ID format
    if Uuid::parse_str(trimmed).is_ok() {
        return Ok(trimmed.to_string());
    }

    // Allow legacy IDs that start with "migrated_" or "legacy_"
    if trimmed.starts_with("migrated_") || trimmed.starts_with("legacy_") {
        return Ok(trimmed.to_string());
    }

    Err("Invalid ID format".to_string())
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    last_db_path: Option<String>,
}

/// Valida la contraseña básica para abrir una bóveda existente
/// Solo verifica que no esté vacía y no exceda el límite
fn validate_password_basic(password: String) -> Result<SecretString, String> {
    let trimmed = password.trim();

    if trimmed.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    if trimmed.len() > MAX_PASSWORD_LENGTH {
        return Err(format!(
            "Password cannot exceed {} characters",
            MAX_PASSWORD_LENGTH
        ));
    }

    // Crear SecretString que limpiará la memoria automáticamente
    Ok(SecretString::from(trimmed.to_string()))
}

/// Valida la contraseña con requisitos estrictos para crear una nueva bóveda
/// Requisitos:
/// - Mínimo 8 caracteres
/// - Máximo 128 caracteres
/// - Al menos una letra mayúscula
/// - Al menos una letra minúscula
/// - Al menos un número
/// - Al menos un carácter especial
fn validate_password_strict(password: String) -> Result<SecretString, String> {
    let trimmed = password.trim();

    if trimmed.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    if trimmed.len() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "Password must be at least {} characters",
            MIN_PASSWORD_LENGTH
        ));
    }

    if trimmed.len() > MAX_PASSWORD_LENGTH {
        return Err(format!(
            "Password cannot exceed {} characters",
            MAX_PASSWORD_LENGTH
        ));
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
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if !has_digit {
        return Err("Password must contain at least one number".to_string());
    }

    if !has_special {
        return Err(
            "Password must contain at least one special character (!@#$%^&*...)".to_string(),
        );
    }

    // Crear SecretString que limpiará la memoria automáticamente
    Ok(SecretString::from(trimmed.to_string()))
}

/// Valida que una fecha esté en formato ISO-8601 (YYYY-MM-DD)
fn validate_date(date: &str) -> Result<String, String> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return Err("Date cannot be empty".to_string());
    }

    // Intento 1: Formato DD-MM-YYYY (Preferido por el usuario)
    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%d-%m-%Y") {
        return Ok(parsed.format("%Y-%m-%d").to_string()); // NORMALIZAR A ISO
    }

    // Intento 2: Formato ISO (Estándar DB y fallback)
    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    Err("Invalid date format. Use DD-MM-YYYY or YYYY-MM-DD".to_string())
}

fn ensure_no_connection(state: &State<DbState>) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    if db_lock.is_some() {
        return Err("A vault is already open. Close it first.".to_string());
    }

    Ok(())
}

/// Helper to get database with session check
fn get_db_with_session_check<'a>(
    db_lock: &'a std::sync::MutexGuard<'_, Option<Database>>,
) -> Result<&'a Database, String> {
    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    // Check session timeout
    db.check_session_timeout().map_err(|e| match e {
        DbError::SessionExpired => {
            "Session expired due to inactivity. Please unlock the vault again.".to_string()
        }
        _ => e.to_string(),
    })?;

    Ok(db)
}

fn persist_last_db_path(app_handle: &AppHandle, path: &PathBuf) -> Result<(), String> {
    let mut config = load_config(app_handle)?;
    config.last_db_path = Some(path.to_string_lossy().to_string());
    save_config(app_handle, &config)
}

fn load_config(app_handle: &AppHandle) -> Result<AppConfig, String> {
    let path = config_path(app_handle)?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let data = fs::read_to_string(&path).map_err(|_| "Could not read configuration".to_string())?;

    serde_json::from_str(&data).map_err(|_| "Could not parse configuration".to_string())
}

fn save_config(app_handle: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app_handle)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| "Could not create configuration directory".to_string())?;
    }

    let data = serde_json::to_string_pretty(config)
        .map_err(|_| "Could not serialize configuration".to_string())?;

    fs::write(&path, &data).map_err(|_| "Could not save configuration".to_string())?;

    // Set restrictive permissions (owner read/write only - 0600)
    #[cfg(unix)]
    {
        fs::set_permissions(&path, Permissions::from_mode(0o600))
            .map_err(|_| "Could not set configuration file permissions".to_string())?;
    }

    Ok(())
}

fn config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| "Could not access application data directory".to_string())?;

    Ok(dir.join("config.json"))
}

/// Sanitizes the requested vault path to ensure it stays inside the app data directory
fn sanitize_db_path(app_handle: &AppHandle, raw: &str) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| "Could not access application data directory".to_string())?;

    // Ensure the base directory exists so canonicalization behaves deterministically
    fs::create_dir_all(&app_data_dir)
        .map_err(|_| "Could not access application data directory".to_string())?;

    let base = app_data_dir.canonicalize().unwrap_or(app_data_dir.clone());

    let raw_trimmed = raw.trim();
    if raw_trimmed.is_empty() {
        return Err("Vault path cannot be empty".to_string());
    }

    let candidate = PathBuf::from(raw_trimmed);

    // If an absolute path is provided, ensure it resides within app_data_dir
    let relative = if candidate.is_absolute() {
        candidate
            .strip_prefix(&base)
            .map_err(|_| "Vault path must stay inside the app data directory".to_string())?
            .to_path_buf()
    } else {
        candidate
    };

    // Normalize the path while preventing traversal outside of base
    let mut normalized = base.clone();
    for comp in relative.components() {
        match comp {
            Component::Prefix(_) | Component::RootDir => {
                return Err("Vault path must stay inside the app data directory".to_string());
            }
            Component::ParentDir => {
                if !normalized.pop() || !normalized.starts_with(&base) {
                    return Err("Vault path must stay inside the app data directory".to_string());
                }
            }
            Component::CurDir => {}
            Component::Normal(c) => normalized.push(c),
        }
    }

    Ok(normalized)
}

/// Genera una clave única para rate limiting basada en la ruta de la bóveda

// ==================== Database Management Commands ====================

/// Comando para verificar si la base de datos está inicializada
#[tauri::command]
pub fn is_db_initialized(state: State<DbState>) -> Result<bool, String> {
    Ok(state.is_initialized())
}

/// Comando para crear una nueva base de datos en una ruta específica
#[tauri::command]
pub fn create_db(
    password: String,
    path: Option<String>,
    app_handle: AppHandle,
    state: State<DbState>,
) -> Result<String, String> {
    // Usar validación estricta para crear nueva bóveda
    let password = validate_password_strict(password)?;
    ensure_no_connection(&state)?;

    let db_path_raw = if let Some(p) = path {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            Database::default_db_path(&app_handle).map_err(|e| e.to_string())?
        } else {
            PathBuf::from(trimmed)
        }
    } else {
        Database::default_db_path(&app_handle).map_err(|e| e.to_string())?
    };

    let db_path = sanitize_db_path(&app_handle, db_path_raw.to_string_lossy().as_ref())?;

    if db_path.exists() {
        return Err("A vault already exists at this location. Use unlock instead.".to_string());
    }

    let database =
        Database::init(&app_handle, &password, Some(db_path.clone())).map_err(|e| e.to_string())?;

    database.health_check().map_err(|e| e.to_string())?;

    let mut db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    *db_lock = Some(database);

    persist_last_db_path(&app_handle, &db_path)?;

    // Reset any rate limiting for this path
    reset_persistent_rate_limit(&db_path);

    // Log vault creation
    log_security_event(SecurityEvent::VaultCreated, None);

    Ok("Vault created successfully".to_string())
}

/// Comando para abrir una base de datos existente con rate limiting
#[tauri::command]
pub fn open_db(
    password: String,
    path: Option<String>,
    app_handle: AppHandle,
    state: State<DbState>,
) -> Result<String, String> {
    // Usar validación básica para abrir bóveda existente (compatibilidad con contraseñas antiguas)
    let password = validate_password_basic(password)?;
    ensure_no_connection(&state)?;

    // Resolver la ruta y validarla contra app_data_dir
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
    .or_else(|| load_config(&app_handle).ok().and_then(|c| c.last_db_path));

    let db_path_raw = if let Some(p) = raw_path {
        PathBuf::from(p)
    } else {
        Database::default_db_path(&app_handle).map_err(|e| e.to_string())?
    };

    let db_path = sanitize_db_path(&app_handle, db_path_raw.to_string_lossy().as_ref())?;

    if !db_path.exists() {
        return Err("No vault found at the specified location".to_string());
    }

    // Persistent rate limiting check
    check_persistent_rate_limit(&db_path)?;

    // Intentar abrir la base de datos
    let database = match Database::init(&app_handle, &password, Some(db_path.clone())) {
        Ok(db) => {
            // Éxito - resetear rate limit
            reset_persistent_rate_limit(&db_path);
            db
        }
        Err(e) => {
            // Fallo - registrar intento fallido persistente
            record_persistent_failed_attempt(&db_path);
            log_security_event(SecurityEvent::VaultOpenFailed, None);
            return Err(e.to_string());
        }
    };

    database.health_check().map_err(|e| e.to_string())?;

    let mut db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    *db_lock = Some(database);

    persist_last_db_path(&app_handle, &db_path)?;

    // Log successful vault open
    log_security_event(SecurityEvent::VaultOpened, None);

    Ok("Vault unlocked successfully".to_string())
}

/// Comando para cerrar la conexión a la base de datos
#[tauri::command]
pub fn close_db(state: State<DbState>) -> Result<String, String> {
    let mut db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    if db_lock.is_none() {
        return Err("No vault is currently open".to_string());
    }

    // Eliminar la conexión (Drop se encarga de cerrarla)
    *db_lock = None;

    // Log vault close
    log_security_event(SecurityEvent::VaultClosed, None);

    Ok("Vault locked successfully".to_string())
}

/// Comando para obtener la ruta de la base de datos
#[tauri::command]
pub fn get_db_path(app_handle: AppHandle, state: State<DbState>) -> Result<String, String> {
    // Si hay conexión activa, usamos esa ruta
    {
        let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
        if let Some(db) = db_lock.as_ref() {
            return Ok(db.path().to_string_lossy().to_string());
        }
    }

    // Si no hay conexión, devolvemos la última ruta usada o la ruta por defecto
    if let Ok(config) = load_config(&app_handle) {
        if let Some(last) = config.last_db_path {
            return Ok(last);
        }
    }

    Ok(Database::default_db_path(&app_handle)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .to_string())
}

// ==================== Financial Transaction Commands ====================

/// Comando para agregar una transacción
#[tauri::command]
pub fn add_transaction(
    state: State<DbState>,
    amount: i64,
    category: String,
    description: String,
    date: String,
    is_expense: bool,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validar y sanitizar campos
    let category = validate_field_length(&category, MAX_CATEGORY_LENGTH, "Category")?;
    let category = sanitize_string(&category);

    if category.is_empty() {
        return Err("Category cannot be empty".to_string());
    }

    let description = validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
    let description = sanitize_string(&description);

    // Validar formato de fecha
    let date = validate_date(&date)?;

    if amount <= 0 {
        return Err("Amount must be greater than zero".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let transaction_type = if is_expense { "expense" } else { "income" };

    let transaction = Transaction::new(
        id.clone(),
        amount,
        category,
        description,
        date,
        transaction_type.to_string(),
    );

    db.create_transaction(&transaction)
        .map_err(|e| e.to_string())?;

    // Log transaction creation
    log_security_event(
        SecurityEvent::TransactionCreated,
        Some(if is_expense { "expense" } else { "income" }),
    );

    Ok(id)
}

/// Comando para obtener todas las transacciones
#[tauri::command]
pub fn get_transactions(state: State<DbState>) -> Result<Vec<Transaction>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let transactions = db.get_transactions().map_err(|e| e.to_string())?;

    Ok(transactions)
}

/// Comando para obtener el resumen de balance
#[tauri::command]
pub fn get_balance(state: State<DbState>) -> Result<BalanceSummary, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let balance = db.get_balance_summary().map_err(|e| e.to_string())?;

    Ok(balance)
}

/// Comando para eliminar una transacción
#[tauri::command]
pub fn delete_transaction(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID format
    let validated_id = validate_uuid(&id)?;

    db.delete_transaction(&validated_id)
        .map_err(|e| e.to_string())?;

    // Log transaction deletion
    log_security_event(SecurityEvent::TransactionDeleted, None);

    Ok(())
}

// ==================== Crypto Price Commands ====================

/// Command to fetch cryptocurrency prices from CoinGecko
#[tauri::command]
pub async fn get_crypto_prices(coins: Vec<String>) -> Result<Vec<CryptoAsset>, String> {
    crypto::fetch_crypto_prices(coins).await
}

// ==================== Legacy Crypto Holdings Commands (backwards compatibility) ====================

/// Command to add a crypto holding to the portfolio (LEGACY)
#[tauri::command]
pub fn add_crypto_holding(
    state: State<DbState>,
    coin_id: String,
    symbol: String,
    amount: f64,
    purchase_price: f64,
    purchase_date: String,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate and sanitize inputs
    let coin_id = validate_coin_id_str(&coin_id)?;
    let symbol = validate_symbol(&symbol)?;
    validate_positive_amount(amount, "Amount")?;
    validate_non_negative(Some(purchase_price), "Purchase price")?;

    // Validar formato de fecha
    let purchase_date = validate_date(&purchase_date)?;

    let id = Uuid::new_v4().to_string();

    let holding = CryptoHolding::new(
        id.clone(),
        coin_id.to_lowercase(),
        symbol.to_uppercase(),
        amount,
        purchase_price,
        purchase_date,
    );

    db.create_crypto_holding(&holding)
        .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Command to get all crypto holdings (LEGACY)
#[tauri::command]
pub fn get_crypto_holdings(state: State<DbState>) -> Result<Vec<CryptoHolding>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let holdings = db.get_crypto_holdings().map_err(|e| e.to_string())?;

    Ok(holdings)
}

/// Command to delete a crypto holding (LEGACY)
#[tauri::command]
pub fn delete_crypto_holding(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID format
    let validated_id = validate_uuid(&id)?;

    db.delete_crypto_holding(&validated_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ==================== Crypto Wallet Commands ====================

/// Command to create a new crypto wallet
#[tauri::command]
pub fn add_wallet(
    state: State<DbState>,
    name: String,
    category: String,
    icon: Option<String>,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate and sanitize inputs
    let name = validate_field_length(&name, MAX_WALLET_NAME_LENGTH, "Wallet name")?;
    let name = sanitize_string(&name);

    if name.is_empty() {
        return Err("Wallet name cannot be empty".to_string());
    }

    // Validate category
    let valid_categories = ["exchange", "wallet_single", "wallet_multi"];
    if !valid_categories.contains(&category.as_str()) {
        return Err(format!(
            "Invalid category. Must be one of: {}",
            valid_categories.join(", ")
        ));
    }

    // Validate icon if provided
    let icon = match icon {
        Some(i) => Some(validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?),
        None => None,
    };

    let id = Uuid::new_v4().to_string();

    // Log wallet creation before moving category
    log_security_event(SecurityEvent::WalletCreated, Some(&category));

    let wallet = CryptoWallet::new(id.clone(), name, category, icon);

    db.create_wallet(&wallet).map_err(|e| e.to_string())?;

    Ok(id)
}

/// Command to get all wallets
#[tauri::command]
pub fn get_wallets(state: State<DbState>) -> Result<Vec<CryptoWallet>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let wallets = db.get_wallets().map_err(|e| e.to_string())?;

    Ok(wallets)
}

/// Command to get a single wallet by ID
#[tauri::command]
pub fn get_wallet(state: State<DbState>, id: String) -> Result<Option<CryptoWallet>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID format
    let validated_id = validate_uuid(&id)?;

    let wallet = db.get_wallet(&validated_id).map_err(|e| e.to_string())?;

    Ok(wallet)
}

/// Command to update a wallet
#[tauri::command]
pub fn update_wallet(
    state: State<DbState>,
    id: String,
    name: String,
    category: String,
    icon: Option<String>,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID format
    let validated_id = validate_uuid(&id)?;

    // Validate and sanitize inputs
    let name = validate_field_length(&name, MAX_WALLET_NAME_LENGTH, "Wallet name")?;
    let name = sanitize_string(&name);

    if name.is_empty() {
        return Err("Wallet name cannot be empty".to_string());
    }

    let valid_categories = ["exchange", "wallet_single", "wallet_multi"];
    if !valid_categories.contains(&category.as_str()) {
        return Err(format!(
            "Invalid category. Must be one of: {}",
            valid_categories.join(", ")
        ));
    }

    // Validate icon if provided
    let icon = match icon {
        Some(i) => Some(validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?),
        None => None,
    };

    let wallet = CryptoWallet::new(validated_id, name, category, icon);

    db.update_wallet(&wallet).map_err(|e| e.to_string())?;

    Ok(())
}

/// Command to delete a wallet and all its transactions
#[tauri::command]
pub fn delete_wallet(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID format
    let validated_id = validate_uuid(&id)?;

    db.delete_wallet(&validated_id).map_err(|e| e.to_string())?;

    // Log wallet deletion
    log_security_event(SecurityEvent::WalletDeleted, None);

    Ok(())
}

// ==================== Crypto Transaction Commands ====================

/// Command to add a crypto transaction
#[tauri::command]
pub fn add_crypto_transaction(
    state: State<DbState>,
    wallet_id: String,
    coin_id: String,
    symbol: String,
    transaction_type: String,
    amount: f64,
    price_per_coin: Option<f64>,
    fee: Option<f64>,
    date: String,
    notes: Option<String>,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate and sanitize inputs
    if wallet_id.trim().is_empty() {
        return Err("Wallet ID cannot be empty".to_string());
    }

    let coin_id = validate_coin_id_str(&coin_id)?;
    let symbol = validate_symbol(&symbol)?;

    // Validate and sanitize notes if provided
    let notes = match notes {
        Some(n) => {
            let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
            Some(sanitize_string(&validated))
        }
        None => None,
    };

    if amount <= 0.0 {
        return Err("Amount must be greater than zero".to_string());
    }

    let valid_types = ["buy", "sell", "transfer_in", "transfer_out", "swap"];
    if !valid_types.contains(&transaction_type.as_str()) {
        return Err(format!(
            "Invalid transaction type. Must be one of: {}",
            valid_types.join(", ")
        ));
    }

    // Validate numeric fields
    validate_positive_amount(amount, "Amount")?;
    let price_per_coin = validate_non_negative(price_per_coin, "Price per coin")?;
    let fee = validate_non_negative(fee, "Fee")?;

    // Validar formato de fecha
    let date = validate_date(&date)?;

    // Log crypto transaction creation before moving transaction_type
    log_security_event(
        SecurityEvent::CryptoTransactionCreated,
        Some(&transaction_type),
    );

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

    db.create_crypto_transaction(&transaction)
        .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Command to add a swap transaction (creates two linked transactions)
#[tauri::command]
pub fn add_swap_transaction(
    state: State<DbState>,
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
) -> Result<(String, String), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate and sanitize inputs
    if wallet_id.trim().is_empty() {
        return Err("Wallet ID cannot be empty".to_string());
    }

    let from_coin_id = validate_coin_id_str(&from_coin_id)?;
    let to_coin_id = validate_coin_id_str(&to_coin_id)?;
    let from_symbol = validate_symbol(&from_symbol)?;
    let to_symbol = validate_symbol(&to_symbol)?;

    // Validate and sanitize notes if provided
    let notes = match notes {
        Some(n) => {
            let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
            Some(sanitize_string(&validated))
        }
        None => None,
    };

    validate_positive_amount(from_amount, "From amount")?;
    validate_positive_amount(to_amount, "To amount")?;

    let fee = validate_non_negative(fee, "Fee")?;
    let fee_amount = validate_non_negative(fee_amount, "Fee amount")?;

    if let Some(ref coin) = fee_coin_id {
        let _ = validate_coin_id_str(coin)?;
    }

    if fee_amount.is_some() && fee_coin_id.is_none() {
        return Err("Fee coin ID is required when fee amount is provided".to_string());
    }

    // Validar formato de fecha
    let date = validate_date(&date)?;

    let from_tx_id = Uuid::new_v4().to_string();
    let to_tx_id = Uuid::new_v4().to_string();

    // Create the "from" transaction (swap out)
    let mut from_tx = CryptoTransaction::new(
        from_tx_id.clone(),
        wallet_id.trim().to_string(),
        from_coin_id.to_lowercase(),
        from_symbol.to_uppercase(),
        "swap".to_string(),
        from_amount,
        None, // Price will be calculated from the swap ratio
        fee,
        date.clone(),
        notes.clone(),
    );
    from_tx.related_tx_id = Some(to_tx_id.clone());
    from_tx.fee_coin_id = fee_coin_id.clone();
    from_tx.fee_amount = fee_amount;

    // Create the "to" transaction (swap in - treated as transfer_in)
    let mut to_tx = CryptoTransaction::new(
        to_tx_id.clone(),
        wallet_id.trim().to_string(),
        to_coin_id.to_lowercase(),
        to_symbol.to_uppercase(),
        "transfer_in".to_string(), // Swap in is treated as an inflow
        to_amount,
        None,
        None, // Fee only on the "from" side
        date,
        notes,
    );
    to_tx.related_tx_id = Some(from_tx_id.clone());

    // Create both transactions
    db.create_crypto_transaction(&from_tx)
        .map_err(|e| e.to_string())?;
    db.create_crypto_transaction(&to_tx)
        .map_err(|e| e.to_string())?;

    Ok((from_tx_id, to_tx_id))
}

/// Command to add a transfer between wallets (creates two linked transactions)
#[tauri::command]
pub fn add_transfer_transaction(
    state: State<DbState>,
    from_wallet_id: String,
    to_wallet_id: String,
    coin_id: String,
    symbol: String,
    amount: f64,
    fee: Option<f64>,
    date: String,
    notes: Option<String>,
) -> Result<(String, String), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate and sanitize inputs
    if from_wallet_id.trim().is_empty() || to_wallet_id.trim().is_empty() {
        return Err("Wallet IDs cannot be empty".to_string());
    }

    let coin_id = validate_coin_id_str(&coin_id)?;
    let symbol = validate_symbol(&symbol)?;

    // Validate and sanitize notes if provided
    let notes = match notes {
        Some(n) => {
            let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
            Some(sanitize_string(&validated))
        }
        None => None,
    };

    validate_positive_amount(amount, "Amount")?;
    let fee = validate_non_negative(fee, "Fee")?;

    // Validar formato de fecha
    let date = validate_date(&date)?;

    let from_tx_id = Uuid::new_v4().to_string();
    let to_tx_id = Uuid::new_v4().to_string();

    // Amount received after fee
    let amount_after_fee = amount - fee.unwrap_or(0.0);
    if amount_after_fee <= 0.0 {
        return Err("Fee cannot be greater than or equal to the amount".to_string());
    }

    // Create the "from" transaction (transfer out)
    let mut from_tx = CryptoTransaction::new(
        from_tx_id.clone(),
        from_wallet_id.trim().to_string(),
        coin_id.to_lowercase(),
        symbol.to_uppercase(),
        "transfer_out".to_string(),
        amount,
        None,
        fee,
        date.clone(),
        notes.clone(),
    );
    from_tx.related_tx_id = Some(to_tx_id.clone());

    // Create the "to" transaction (transfer in)
    let mut to_tx = CryptoTransaction::new(
        to_tx_id.clone(),
        to_wallet_id.trim().to_string(),
        coin_id.to_lowercase(),
        symbol.to_uppercase(),
        "transfer_in".to_string(),
        amount_after_fee, // Amount received is after network fee
        None,
        None,
        date,
        notes,
    );
    to_tx.related_tx_id = Some(from_tx_id.clone());

    // Create both transactions
    db.create_crypto_transaction(&from_tx)
        .map_err(|e| e.to_string())?;
    db.create_crypto_transaction(&to_tx)
        .map_err(|e| e.to_string())?;

    Ok((from_tx_id, to_tx_id))
}

/// Command to get all transactions for a specific wallet
#[tauri::command]
pub fn get_wallet_transactions(
    state: State<DbState>,
    wallet_id: String,
) -> Result<Vec<CryptoTransaction>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID format
    let validated_id = validate_uuid(&wallet_id)?;

    let transactions = db
        .get_wallet_transactions(&validated_id)
        .map_err(|e| e.to_string())?;

    Ok(transactions)
}

/// Command to get all crypto transactions across all wallets
#[tauri::command]
pub fn get_all_crypto_transactions(
    state: State<DbState>,
) -> Result<Vec<CryptoTransaction>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let transactions = db
        .get_all_crypto_transactions()
        .map_err(|e| e.to_string())?;

    Ok(transactions)
}

/// Command to delete a crypto transaction
#[tauri::command]
pub fn delete_crypto_transaction(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID format
    let validated_id = validate_uuid(&id)?;

    // Check if this transaction has a related transaction (swap/transfer)
    if let Ok(Some(tx)) = db.get_crypto_transaction(&validated_id) {
        if let Some(related_id) = tx.related_tx_id {
            // Delete the related transaction too
            let _ = db.delete_crypto_transaction(&related_id);
        }
    }

    db.delete_crypto_transaction(&validated_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ==================== Portfolio Aggregation Commands ====================

/// Command to get aggregated portfolio across all wallets (for Overview tab)
#[tauri::command]
pub fn get_aggregated_portfolio(state: State<DbState>) -> Result<Vec<AggregatedAsset>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let portfolio = db.get_aggregated_portfolio().map_err(|e| e.to_string())?;

    Ok(portfolio)
}

/// Command to get aggregated holdings for a specific wallet
#[tauri::command]
pub fn get_wallet_holdings(
    state: State<DbState>,
    wallet_id: String,
) -> Result<Vec<AggregatedAsset>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID format
    let validated_id = validate_uuid(&wallet_id)?;

    let holdings = db
        .get_wallet_aggregated_holdings(&validated_id)
        .map_err(|e| e.to_string())?;

    Ok(holdings)
}

/// Command to get remaining session time in seconds
#[tauri::command]
pub fn get_session_remaining(state: State<DbState>) -> Result<i64, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    db.get_session_remaining().map_err(|e| e.to_string())
}

// ==================== Habits Commands ====================

/// Validates a hex color code
fn validate_color(color: &str) -> Result<String, String> {
    let trimmed = color.trim();

    if trimmed.is_empty() {
        return Err("Color cannot be empty".to_string());
    }

    if trimmed.len() != 7 {
        return Err("Color must be in #RRGGBB format".to_string());
    }

    if !trimmed.starts_with('#') {
        return Err("Color must start with #".to_string());
    }

    // Validate hex characters
    if !trimmed[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Color must contain valid hex characters".to_string());
    }

    Ok(trimmed.to_lowercase())
}

/// Command to create a new habit
#[tauri::command]
pub fn create_habit(
    state: State<DbState>,
    name: String,
    description: Option<String>,
    color: String,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate and sanitize name
    let name = validate_field_length(&name, MAX_HABIT_NAME_LENGTH, "Habit name")?;
    let name = sanitize_string(&name);

    if name.is_empty() {
        return Err("Habit name cannot be empty".to_string());
    }

    // Validate and sanitize description if provided
    let description = match description {
        Some(d) => {
            let validated = validate_field_length(&d, MAX_HABIT_DESCRIPTION_LENGTH, "Description")?;
            let sanitized = sanitize_string(&validated);
            if sanitized.is_empty() {
                None
            } else {
                Some(sanitized)
            }
        }
        None => None,
    };

    // Validate color
    let color = validate_color(&color)?;

    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let habit = Habit::new(id.clone(), name, description, color, created_at);

    db.create_habit(&habit).map_err(|e| e.to_string())?;

    Ok(id)
}

/// Command to get all active habits
#[tauri::command]
pub fn get_habits(state: State<DbState>) -> Result<Vec<Habit>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let habits = db.get_habits().map_err(|e| e.to_string())?;

    Ok(habits)
}

/// Command to update an existing habit
#[tauri::command]
pub fn update_habit(
    state: State<DbState>,
    id: String,
    name: String,
    description: Option<String>,
    color: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate ID
    let validated_id = validate_uuid(&id)?;

    // Validate and sanitize name
    let name = validate_field_length(&name, MAX_HABIT_NAME_LENGTH, "Habit name")?;
    let name = sanitize_string(&name);

    if name.is_empty() {
        return Err("Habit name cannot be empty".to_string());
    }

    // Validate and sanitize description
    let description = match description {
        Some(d) => {
            let validated = validate_field_length(&d, MAX_HABIT_DESCRIPTION_LENGTH, "Description")?;
            let sanitized = sanitize_string(&validated);
            if sanitized.is_empty() {
                None
            } else {
                Some(sanitized)
            }
        }
        None => None,
    };

    // Validate color
    let color = validate_color(&color)?;

    // Get existing habit to preserve created_at
    let existing = db
        .get_habit(&validated_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Habit not found".to_string())?;

    let habit = Habit {
        id: validated_id,
        name,
        description,
        color,
        created_at: existing.created_at,
        archived: existing.archived,
    };

    db.update_habit(&habit).map_err(|e| e.to_string())?;

    Ok(())
}

/// Command to archive a habit (soft delete)
#[tauri::command]
pub fn archive_habit(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let validated_id = validate_uuid(&id)?;

    db.archive_habit(&validated_id).map_err(|e| e.to_string())?;

    Ok(())
}

/// Command to permanently delete a habit
#[tauri::command]
pub fn delete_habit(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let validated_id = validate_uuid(&id)?;

    db.delete_habit(&validated_id).map_err(|e| e.to_string())?;

    Ok(())
}

/// Command to toggle habit completion for a date
/// Returns { created: bool, log_id: Option<String> }
#[tauri::command]
pub fn toggle_habit_completion(
    state: State<DbState>,
    habit_id: String,
    date: String,
) -> Result<(bool, Option<String>), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate habit ID
    let validated_habit_id = validate_uuid(&habit_id)?;

    // Validate date format (YYYY-MM-DD)
    let validated_date = validate_date(&date)?;

    let result = db
        .toggle_habit_log(&validated_habit_id, &validated_date)
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Command to get habit logs for a date range
#[tauri::command]
pub fn get_habit_logs(
    state: State<DbState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<HabitLog>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    // Validate date formats
    let start = validate_date(&start_date)?;
    let end = validate_date(&end_date)?;

    let logs = db.get_habit_logs(&start, &end).map_err(|e| e.to_string())?;

    Ok(logs)
}

/// Command to get habit completion statistics for a date range
#[tauri::command]
pub fn get_habit_stats(
    state: State<DbState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<(String, i32)>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    let db = get_db_with_session_check(&db_lock)?;

    let start = validate_date(&start_date)?;
    let end = validate_date(&end_date)?;

    let stats = db
        .get_habit_stats(&start, &end)
        .map_err(|e| e.to_string())?;

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn test_db_state_default() {
        let state = DbState::default();
        assert!(!state.is_initialized());
    }

    #[test]
    fn test_db_state_new() {
        let state = DbState::new();
        assert!(!state.is_initialized());
    }

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
        assert_eq!(result.unwrap_err(), "Password cannot be empty");
    }

    #[test]
    fn test_validate_password_basic_valid() {
        // Contraseña simple debe funcionar para abrir bóvedas antiguas
        let result = validate_password_basic("simple".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().expose_secret(), "simple");
    }

    #[test]
    fn test_validate_password_basic_too_long() {
        let long_pass = "a".repeat(129);
        assert!(validate_password_basic(long_pass).is_err());
    }

    #[test]
    fn test_validate_password_strict_empty() {
        let result = validate_password_strict("".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password cannot be empty");
    }

    #[test]
    fn test_validate_password_strict_too_short() {
        let result = validate_password_strict("1234567".to_string());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Password must be at least 8 characters"
        );
    }

    #[test]
    fn test_validate_password_strict_valid() {
        // Password with all requirements: uppercase, lowercase, digit, special char
        let result = validate_password_strict("Password1!".to_string());
        assert!(result.is_ok());
        // Verificar que el SecretString contiene el valor correcto
        assert_eq!(result.unwrap().expose_secret(), "Password1!");
    }

    #[test]
    fn test_validate_password_strict_complexity() {
        // Missing uppercase
        assert!(validate_password_strict("password1!".to_string()).is_err());
        // Missing lowercase
        assert!(validate_password_strict("PASSWORD1!".to_string()).is_err());
        // Missing digit
        assert!(validate_password_strict("Password!".to_string()).is_err());
        // Missing special character
        assert!(validate_password_strict("Password1".to_string()).is_err());
        // Valid password with all requirements
        assert!(validate_password_strict("Password1!".to_string()).is_ok());
        assert!(validate_password_strict("MyP@ssw0rd".to_string()).is_ok());
        assert!(validate_password_strict("Test#123abc".to_string()).is_ok());
        // Too long
        let long_pass = "A".repeat(129) + "a1!";
        assert!(validate_password_strict(long_pass).is_err());
    }

    #[test]
    fn test_validate_date_valid() {
        // Formato ISO (fallback)
        assert!(validate_date("2024-01-15").is_ok());
        assert!(validate_date("2023-12-31").is_ok());
        assert_eq!(validate_date("  2024-01-15  ").unwrap(), "2024-01-15");
        // Formato DD-MM-YYYY (preferido por el usuario)
        assert_eq!(validate_date("15-01-2024").unwrap(), "2024-01-15");
        assert_eq!(validate_date("31-12-2023").unwrap(), "2023-12-31");
        assert_eq!(validate_date("01-06-2025").unwrap(), "2025-06-01");
    }

    #[test]
    fn test_validate_date_invalid() {
        assert!(validate_date("").is_err());
        assert!(validate_date("2024/01/15").is_err()); // Formato con barras no soportado
        assert!(validate_date("2024-13-01").is_err()); // Invalid month
        assert!(validate_date("2024-02-30").is_err()); // Invalid day
        assert!(validate_date("30-02-2024").is_err()); // Invalid day in DD-MM-YYYY
        assert!(validate_date("not-a-date").is_err());
        assert!(validate_date("15-13-2024").is_err()); // Invalid month in DD-MM-YYYY
    }
}
