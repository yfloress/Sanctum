use crate::db::Database;
use crate::models::{BalanceSummary, Transaction};
use secrecy::SecretString; // QUITADO: ExposeSecret (ya no se usa aquí)
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

/// Configuración de rate limiting
const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION_SECS: u64 = 300; // 5 minutos
const ATTEMPT_RESET_SECS: u64 = 60; // Reset contador después de 1 minuto sin intentos

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

    pub fn is_initialized(&self) -> bool {
        self.db.lock().unwrap().is_some()
    }

    /// Verifica si el rate limit permite un intento
    fn check_rate_limit(&self, key: &str) -> Result<(), String> {
        let mut rate_limit = self.rate_limit.lock().unwrap();
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
        let mut rate_limit = self.rate_limit.lock().unwrap();
        let state = rate_limit.entry(key.to_string()).or_default();

        state.failed_attempts += 1;
        state.last_attempt = Instant::now();

        if state.failed_attempts >= MAX_FAILED_ATTEMPTS {
            state.locked_until = Some(Instant::now() + Duration::from_secs(LOCKOUT_DURATION_SECS));
        }
    }

    /// Resetea el contador de intentos fallidos tras éxito
    fn reset_rate_limit(&self, key: &str) {
        let mut rate_limit = self.rate_limit.lock().unwrap();
        rate_limit.remove(key);
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    last_db_path: Option<String>,
}

/// Valida la contraseña y retorna un SecretString para manejo seguro
fn validate_password(password: String) -> Result<SecretString, String> {
    let trimmed = password.trim();

    if trimmed.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    if trimmed.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    // Crear SecretString que limpiará la memoria automáticamente
    Ok(SecretString::from(trimmed.to_string()))
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

    fs::write(&path, data).map_err(|_| "Could not save configuration".to_string())
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
    let password = validate_password(password)?;
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
    let password = validate_password(password)?;
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
            return Err(e.to_string());
        }
    };

    database.health_check().map_err(|e| e.to_string())?;

    let mut db_lock = state.db.lock().map_err(|_| "Internal error".to_string())?;
    *db_lock = Some(database);

    persist_last_db_path(&app_handle, &db_path)?;

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

    // Validar campos
    if category.trim().is_empty() {
        return Err("Category cannot be empty".to_string());
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret; // AGREGADO: Se mueve aquí porque solo se usa en tests

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
    fn test_validate_password_empty() {
        let result = validate_password("".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password cannot be empty");
    }

    #[test]
    fn test_validate_password_too_short() {
        let result = validate_password("1234567".to_string());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Password must be at least 8 characters"
        );
    }

    #[test]
    fn test_validate_password_valid() {
        let result = validate_password("12345678".to_string());
        assert!(result.is_ok());
        // Verificar que el SecretString contiene el valor correcto
        assert_eq!(result.unwrap().expose_secret(), "12345678");
    }

    #[test]
    fn test_rate_limit_state_default() {
        let state = RateLimitState::default();
        assert_eq!(state.failed_attempts, 0);
        assert!(state.locked_until.is_none());
    }
}
