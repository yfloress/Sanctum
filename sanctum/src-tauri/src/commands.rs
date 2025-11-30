use crate::crypto;
use crate::db::Database;
use crate::models::{
    AggregatedAsset, BalanceSummary, CryptoAsset, CryptoHolding, CryptoTransaction, CryptoWallet,
    Transaction,
};
use crate::security_log::{SecurityEvent, log_auth_failure, log_security_event};
use chrono::NaiveDate;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

/// Configuración de rate limiting
const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION_SECS: u64 = 300; // 5 minutos
const ATTEMPT_RESET_SECS: u64 = 60; // Reset contador después de 1 minuto sin intentos

// ==================== Security: Field Length Limits ====================
const MAX_CATEGORY_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 512;
const MAX_NOTES_LENGTH: usize = 1024;
const MAX_WALLET_NAME_LENGTH: usize = 128;
const MAX_SYMBOL_LENGTH: usize = 16;
const MAX_COIN_ID_LENGTH: usize = 64;
const MAX_ICON_LENGTH: usize = 32;

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

/// Estado de rate limiting para una IP/sesión
#[derive(Debug, Clone)]
struct RateLimitState {
    failed_attempts: u32,
    last_attempt: Instant,
    locked_until: Option<Instant>,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            failed_attempts: 0,
            last_attempt: Instant::now(),
            locked_until: None,
        }
    }
}

/// Estado global para mantener la conexión a la base de datos
pub struct DbState {
    pub db: Mutex<Option<Database>>,
    rate_limit: Mutex<HashMap<String, RateLimitState>>,
}

impl Default for DbState {
    fn default() -> Self {
        Self {
            db: Mutex::new(None),
            rate_limit: Mutex::new(HashMap::new()),
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

    /// Verifica si el rate limit permite un intento
    fn check_rate_limit(&self, key: &str) -> Result<(), String> {
        let mut rate_limit = self
            .rate_limit
            .lock()
            .map_err(|_| "Internal error: rate limit lock poisoned".to_string())?;
        let state = rate_limit.entry(key.to_string()).or_default();

        // Verificar si está bloqueado
        if let Some(locked_until) = state.locked_until {
            if Instant::now() < locked_until {
                let remaining = locked_until.duration_since(Instant::now()).as_secs();
                return Err(format!(
                    "Too many failed attempts. Try again in {} seconds",
                    remaining
                ));
            } else {
                // El bloqueo expiró, resetear
                state.locked_until = None;
                state.failed_attempts = 0;
            }
        }

        // Reset de intentos si pasó suficiente tiempo
        if state.last_attempt.elapsed() > Duration::from_secs(ATTEMPT_RESET_SECS) {
            state.failed_attempts = 0;
        }

        Ok(())
    }

    /// Registra un intento fallido
    fn record_failed_attempt(&self, key: &str) {
        if let Ok(mut rate_limit) = self.rate_limit.lock() {
            let state = rate_limit.entry(key.to_string()).or_default();

            state.failed_attempts += 1;
            state.last_attempt = Instant::now();

            let locked = state.failed_attempts >= MAX_FAILED_ATTEMPTS;
            if locked {
                state.locked_until =
                    Some(Instant::now() + Duration::from_secs(LOCKOUT_DURATION_SECS));
            }

            // Log the authentication failure
            log_auth_failure(state.failed_attempts, locked);
        }
        // Si el lock falla, simplemente ignoramos (fail-open para rate limiting)
    }

    /// Resetea el contador de intentos fallidos tras éxito
    fn reset_rate_limit(&self, key: &str) {
        if let Ok(mut rate_limit) = self.rate_limit.lock() {
            rate_limit.remove(key);
        }
    }
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

    if trimmed.len() > 128 {
        return Err("Password cannot exceed 128 characters".to_string());
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
fn validate_password_strict(password: String) -> Result<SecretString, String> {
    let trimmed = password.trim();

    if trimmed.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    if trimmed.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    if trimmed.len() > 128 {
        return Err("Password cannot exceed 128 characters".to_string());
    }

    // Verificar complejidad de contraseña
    let has_uppercase = trimmed.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = trimmed.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if !has_digit {
        return Err("Password must contain at least one number".to_string());
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

    // Validar formato y que sea una fecha válida
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map(|_| trimmed.to_string())
        .map_err(|_| "Invalid date format. Use YYYY-MM-DD".to_string())
}

fn ensure_no_connection(state: &State<DbState>) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    if db_lock.is_some() {
        return Err("A vault is already open. Close it first.".to_string());
    }

    Ok(())
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

/// Genera una clave única para rate limiting basada en la ruta de la bóveda
fn get_rate_limit_key(db_path: &PathBuf) -> String {
    format!("vault:{}", db_path.to_string_lossy())
}

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

    let db_path = if let Some(p) = path {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            Database::default_db_path(&app_handle).map_err(|e| e.to_string())?
        } else {
            PathBuf::from(trimmed)
        }
    } else {
        Database::default_db_path(&app_handle).map_err(|e| e.to_string())?
    };

    if db_path.exists() {
        return Err("A vault already exists at this location. Use unlock instead.".to_string());
    }

    let database =
        Database::init(&app_handle, &password, Some(db_path.clone())).map_err(|e| e.to_string())?;

    database.health_check().map_err(|e| e.to_string())?;

    let mut db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    *db_lock = Some(database);

    persist_last_db_path(&app_handle, &db_path)?;

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

    // Resolver la ruta
    let db_path = if let Some(p) = path {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(PathBuf::from(trimmed))
        }
    } else {
        None
    }
    .or_else(|| {
        load_config(&app_handle)
            .ok()
            .and_then(|c| c.last_db_path.map(PathBuf::from))
    })
    .unwrap_or_else(|| PathBuf::from(""));

    let db_path = if db_path.as_os_str().is_empty() {
        Database::default_db_path(&app_handle).map_err(|e| e.to_string())?
    } else {
        db_path
    };

    if !db_path.exists() {
        return Err("No vault found at the specified location".to_string());
    }

    // Rate limiting check
    let rate_limit_key = get_rate_limit_key(&db_path);
    state.check_rate_limit(&rate_limit_key)?;

    // Intentar abrir la base de datos
    let database = match Database::init(&app_handle, &password, Some(db_path.clone())) {
        Ok(db) => {
            // Éxito - resetear rate limit
            state.reset_rate_limit(&rate_limit_key);
            db
        }
        Err(e) => {
            // Fallo - registrar intento fallido
            state.record_failed_attempt(&rate_limit_key);
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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let transactions = db.get_transactions().map_err(|e| e.to_string())?;

    Ok(transactions)
}

/// Comando para obtener el resumen de balance
#[tauri::command]
pub fn get_balance(state: State<DbState>) -> Result<BalanceSummary, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let balance = db.get_balance_summary().map_err(|e| e.to_string())?;

    Ok(balance)
}

/// Comando para eliminar una transacción
#[tauri::command]
pub fn delete_transaction(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let trimmed_id = id.trim();
    if trimmed_id.is_empty() {
        return Err("Transaction ID cannot be empty".to_string());
    }

    db.delete_transaction(trimmed_id)
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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    // Validate and sanitize inputs
    let coin_id = validate_field_length(&coin_id, MAX_COIN_ID_LENGTH, "Coin ID")?;
    if coin_id.is_empty() {
        return Err("Coin ID cannot be empty".to_string());
    }

    let symbol = validate_field_length(&symbol, MAX_SYMBOL_LENGTH, "Symbol")?;
    if symbol.is_empty() {
        return Err("Symbol cannot be empty".to_string());
    }

    if amount <= 0.0 {
        return Err("Amount must be greater than zero".to_string());
    }

    if purchase_price < 0.0 {
        return Err("Purchase price cannot be negative".to_string());
    }

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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let holdings = db.get_crypto_holdings().map_err(|e| e.to_string())?;

    Ok(holdings)
}

/// Command to delete a crypto holding (LEGACY)
#[tauri::command]
pub fn delete_crypto_holding(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let trimmed_id = id.trim();
    if trimmed_id.is_empty() {
        return Err("Holding ID cannot be empty".to_string());
    }

    db.delete_crypto_holding(trimmed_id)
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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let wallets = db.get_wallets().map_err(|e| e.to_string())?;

    Ok(wallets)
}

/// Command to get a single wallet by ID
#[tauri::command]
pub fn get_wallet(state: State<DbState>, id: String) -> Result<Option<CryptoWallet>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let wallet = db.get_wallet(&id).map_err(|e| e.to_string())?;

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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

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

    let wallet = CryptoWallet::new(id, name, category, icon);

    db.update_wallet(&wallet).map_err(|e| e.to_string())?;

    Ok(())
}

/// Command to delete a wallet and all its transactions
#[tauri::command]
pub fn delete_wallet(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let trimmed_id = id.trim();
    if trimmed_id.is_empty() {
        return Err("Wallet ID cannot be empty".to_string());
    }

    db.delete_wallet(trimmed_id).map_err(|e| e.to_string())?;

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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    // Validate and sanitize inputs
    if wallet_id.trim().is_empty() {
        return Err("Wallet ID cannot be empty".to_string());
    }

    let coin_id = validate_field_length(&coin_id, MAX_COIN_ID_LENGTH, "Coin ID")?;
    if coin_id.is_empty() {
        return Err("Coin ID cannot be empty".to_string());
    }

    let symbol = validate_field_length(&symbol, MAX_SYMBOL_LENGTH, "Symbol")?;
    if symbol.is_empty() {
        return Err("Symbol cannot be empty".to_string());
    }

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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    // Validate and sanitize inputs
    if wallet_id.trim().is_empty() {
        return Err("Wallet ID cannot be empty".to_string());
    }

    let from_coin_id = validate_field_length(&from_coin_id, MAX_COIN_ID_LENGTH, "From Coin ID")?;
    let to_coin_id = validate_field_length(&to_coin_id, MAX_COIN_ID_LENGTH, "To Coin ID")?;
    let from_symbol = validate_field_length(&from_symbol, MAX_SYMBOL_LENGTH, "From Symbol")?;
    let to_symbol = validate_field_length(&to_symbol, MAX_SYMBOL_LENGTH, "To Symbol")?;

    if from_coin_id.is_empty() || to_coin_id.is_empty() {
        return Err("Coin IDs cannot be empty".to_string());
    }

    // Validate and sanitize notes if provided
    let notes = match notes {
        Some(n) => {
            let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
            Some(sanitize_string(&validated))
        }
        None => None,
    };

    if from_amount <= 0.0 || to_amount <= 0.0 {
        return Err("Amounts must be greater than zero".to_string());
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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    // Validate and sanitize inputs
    if from_wallet_id.trim().is_empty() || to_wallet_id.trim().is_empty() {
        return Err("Wallet IDs cannot be empty".to_string());
    }

    let coin_id = validate_field_length(&coin_id, MAX_COIN_ID_LENGTH, "Coin ID")?;
    if coin_id.is_empty() {
        return Err("Coin ID cannot be empty".to_string());
    }

    let symbol = validate_field_length(&symbol, MAX_SYMBOL_LENGTH, "Symbol")?;
    if symbol.is_empty() {
        return Err("Symbol cannot be empty".to_string());
    }

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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let transactions = db
        .get_wallet_transactions(&wallet_id)
        .map_err(|e| e.to_string())?;

    Ok(transactions)
}

/// Command to get all crypto transactions across all wallets
#[tauri::command]
pub fn get_all_crypto_transactions(
    state: State<DbState>,
) -> Result<Vec<CryptoTransaction>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let transactions = db
        .get_all_crypto_transactions()
        .map_err(|e| e.to_string())?;

    Ok(transactions)
}

/// Command to delete a crypto transaction
#[tauri::command]
pub fn delete_crypto_transaction(state: State<DbState>, id: String) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let trimmed_id = id.trim();
    if trimmed_id.is_empty() {
        return Err("Transaction ID cannot be empty".to_string());
    }

    // Check if this transaction has a related transaction (swap/transfer)
    if let Ok(Some(tx)) = db.get_crypto_transaction(trimmed_id) {
        if let Some(related_id) = tx.related_tx_id {
            // Delete the related transaction too
            let _ = db.delete_crypto_transaction(&related_id);
        }
    }

    db.delete_crypto_transaction(trimmed_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ==================== Portfolio Aggregation Commands ====================

/// Command to get aggregated portfolio across all wallets (for Overview tab)
#[tauri::command]
pub fn get_aggregated_portfolio(state: State<DbState>) -> Result<Vec<AggregatedAsset>, String> {
    let db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

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

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No vault is currently open".to_string())?;

    let holdings = db
        .get_wallet_aggregated_holdings(&wallet_id)
        .map_err(|e| e.to_string())?;

    Ok(holdings)
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
        let result = validate_password_strict("Password1".to_string());
        assert!(result.is_ok());
        // Verificar que el SecretString contiene el valor correcto
        assert_eq!(result.unwrap().expose_secret(), "Password1");
    }

    #[test]
    fn test_validate_password_strict_complexity() {
        // Missing uppercase
        assert!(validate_password_strict("password1".to_string()).is_err());
        // Missing lowercase
        assert!(validate_password_strict("PASSWORD1".to_string()).is_err());
        // Missing digit
        assert!(validate_password_strict("Password".to_string()).is_err());
        // Valid password
        assert!(validate_password_strict("Password1".to_string()).is_ok());
        // Too long
        let long_pass = "A".repeat(129) + "a1";
        assert!(validate_password_strict(long_pass).is_err());
    }

    #[test]
    fn test_validate_date_valid() {
        assert!(validate_date("2024-01-15").is_ok());
        assert!(validate_date("2023-12-31").is_ok());
        assert_eq!(validate_date("  2024-01-15  ").unwrap(), "2024-01-15");
    }

    #[test]
    fn test_validate_date_invalid() {
        assert!(validate_date("").is_err());
        assert!(validate_date("2024/01/15").is_err());
        assert!(validate_date("15-01-2024").is_err());
        assert!(validate_date("2024-13-01").is_err()); // Invalid month
        assert!(validate_date("2024-02-30").is_err()); // Invalid day
        assert!(validate_date("not-a-date").is_err());
    }

    #[test]
    fn test_rate_limit_state_default() {
        let state = RateLimitState::default();
        assert_eq!(state.failed_attempts, 0);
        assert!(state.locked_until.is_none());
    }
}
