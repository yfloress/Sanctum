use crate::db::Database;
use crate::models::{BalanceSummary, Transaction};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

/// Estado global para mantener la conexión a la base de datos
///
/// Usa un Mutex para permitir acceso seguro desde múltiples threads
/// y Option para indicar si la DB está inicializada o no
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
    /// Crea una nueva instancia de DbState
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifica si la base de datos está inicializada
    pub fn is_initialized(&self) -> bool {
        self.db.lock().unwrap().is_some()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    last_db_path: Option<String>,
}

fn validate_password(password: String) -> Result<String, String> {
    let password = password.trim().to_string();

    if password.is_empty() {
        return Err("La contraseña no puede estar vacía".to_string());
    }

    if password.len() < 8 {
        return Err("La contraseña debe tener al menos 8 caracteres".to_string());
    }

    Ok(password)
}

fn ensure_no_connection(state: &State<DbState>) -> Result<(), String> {
    let db_lock = state
        .db
        .lock()
        .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;

    if db_lock.is_some() {
        return Err(
            "La base de datos ya está inicializada. Cierra la conexión actual primero.".to_string(),
        );
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

    let data = fs::read_to_string(&path)
        .map_err(|e| format!("No se pudo leer la configuración: {}", e))?;

    serde_json::from_str(&data).map_err(|e| format!("No se pudo parsear la configuración: {}", e))
}

fn save_config(app_handle: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app_handle)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("No se pudo crear el directorio de configuración: {}", e))?;
    }

    let data = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Error serializando config: {}", e))?;

    fs::write(&path, data).map_err(|e| format!("No se pudo guardar la configuración: {}", e))
}

fn config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| "No se pudo obtener el directorio de datos de la aplicación".to_string())?;

    Ok(dir.join("config.json"))
}

/// Comando Tauri para inicializar la base de datos con encriptación
///
/// # Argumentos
/// * `password` - Contraseña para encriptar/desencriptar la base de datos
/// * `app_handle` - Handle de la aplicación Tauri
/// * `state` - Estado global que contendrá la conexión
///
/// # Retorna
/// * `Result<String, String>` - Mensaje de éxito o error
///
/// # Ejemplo de llamada desde el frontend (JavaScript/TypeScript)
/// ```javascript
/// import { invoke } from '@tauri-apps/api/core';
///
/// try {
///   const result = await invoke('init_db', { password: 'mi_password_segura' });
///   console.log(result); // "Base de datos inicializada correctamente"
/// } catch (error) {
///   console.error('Error:', error);
/// }
/// ```
#[tauri::command]
pub fn init_db(
    password: String,
    app_handle: AppHandle,
    state: State<DbState>,
) -> Result<String, String> {
    let password = validate_password(password)?;

    // Verificar si ya existe una conexión inicializada
    ensure_no_connection(&state)?;

    // Intentar inicializar la base de datos
    let database = Database::init(&app_handle, &password, None)
        .map_err(|e| format!("Error al inicializar la base de datos: {}", e))?;

    // Verificar la salud de la conexión
    database
        .health_check()
        .map_err(|e| format!("Error en health check: {}", e))?;

    // Guardar la conexión en el estado global
    let mut db_lock = state
        .db
        .lock()
        .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;

    *db_lock = Some(database);

    // Persistir la última ruta usada
    let default_path = Database::default_db_path(&app_handle)
        .map_err(|e| format!("No se pudo obtener la ruta por defecto: {}", e))?;
    persist_last_db_path(&app_handle, &default_path)?;

    Ok("Base de datos inicializada correctamente".to_string())
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

    let db_path = if let Some(path) = path {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            Database::default_db_path(&app_handle)
                .map_err(|e| format!("No se pudo obtener la ruta por defecto: {}", e))?
        } else {
            PathBuf::from(trimmed)
        }
    } else {
        Database::default_db_path(&app_handle)
            .map_err(|e| format!("No se pudo obtener la ruta por defecto: {}", e))?
    };

    if db_path.exists() {
        return Err("Ya existe una base de datos en esa ruta. Usa \"Abrir bóveda\".".to_string());
    }

    let database = Database::init(&app_handle, &password, Some(db_path.clone()))
        .map_err(|e| format!("Error al crear la base de datos: {}", e))?;

    database
        .health_check()
        .map_err(|e| format!("Error en health check: {}", e))?;

    let mut db_lock = state
        .db
        .lock()
        .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;
    *db_lock = Some(database);

    persist_last_db_path(&app_handle, &db_path)?;

    Ok("Base de datos creada y abierta correctamente".to_string())
}

/// Comando para abrir una base de datos existente
#[tauri::command]
pub fn open_db(
    password: String,
    path: Option<String>,
    app_handle: AppHandle,
    state: State<DbState>,
) -> Result<String, String> {
    let password = validate_password(password)?;
    ensure_no_connection(&state)?;

    // Si no se pasa ruta, intentamos con la última usada o la ruta por defecto
    let db_path = if let Some(path) = path {
        let trimmed = path.trim();
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
        Database::default_db_path(&app_handle)
            .map_err(|e| format!("No se pudo obtener la ruta por defecto: {}", e))?
    } else {
        db_path
    };

    if !db_path.exists() {
        return Err(format!(
            "No se encontró una base de datos en la ruta especificada: {}",
            db_path.to_string_lossy()
        ));
    }

    let database = Database::init(&app_handle, &password, Some(db_path.clone()))
        .map_err(|e| format!("Error al abrir la base de datos: {}", e))?;

    database
        .health_check()
        .map_err(|e| format!("Error en health check: {}", e))?;

    let mut db_lock = state
        .db
        .lock()
        .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;
    *db_lock = Some(database);

    persist_last_db_path(&app_handle, &db_path)?;

    Ok("Bóveda abierta correctamente".to_string())
}

/// Comando para cerrar la conexión a la base de datos
#[tauri::command]
pub fn close_db(state: State<DbState>) -> Result<String, String> {
    let mut db_lock = state
        .db
        .lock()
        .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;

    if db_lock.is_none() {
        return Err("No hay ninguna base de datos abierta".to_string());
    }

    // Eliminar la conexión (Drop se encarga de cerrarla)
    *db_lock = None;

    Ok("Base de datos cerrada correctamente".to_string())
}

/// Comando para obtener la ruta de la base de datos
#[tauri::command]
pub fn get_db_path(app_handle: AppHandle, state: State<DbState>) -> Result<String, String> {
    // Si hay conexión activa, usamos esa ruta
    {
        let db_lock = state
            .db
            .lock()
            .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;
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
        .map_err(|e| format!("No se pudo obtener la ruta por defecto: {}", e))?
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
    // Validar que haya una conexión activa
    let db_lock = state
        .db
        .lock()
        .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No hay conexión a la base de datos. Inicializa primero.".to_string())?;

    // Validar campos
    if category.trim().is_empty() {
        return Err("La categoría no puede estar vacía".to_string());
    }

    if amount <= 0 {
        return Err("El monto debe ser mayor a cero".to_string());
    }

    // Generar UUID
    let id = Uuid::new_v4().to_string();

    // Determinar tipo de transacción
    let transaction_type = if is_expense { "expense" } else { "income" };

    // Crear transacción
    let transaction = Transaction::new(
        id.clone(),
        amount,
        category,
        description,
        date,
        transaction_type.to_string(),
    );

    // Guardar en la base de datos
    db.create_transaction(&transaction)
        .map_err(|e| format!("Error al crear transacción: {}", e))?;

    Ok(id)
}

/// Comando para obtener todas las transacciones
#[tauri::command]
pub fn get_transactions(state: State<DbState>) -> Result<Vec<Transaction>, String> {
    let db_lock = state
        .db
        .lock()
        .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No hay conexión a la base de datos. Inicializa primero.".to_string())?;

    let transactions = db
        .get_transactions()
        .map_err(|e| format!("Error al obtener transacciones: {}", e))?;

    Ok(transactions)
}

/// Comando para obtener el resumen de balance
#[tauri::command]
pub fn get_balance(state: State<DbState>) -> Result<BalanceSummary, String> {
    let db_lock = state
        .db
        .lock()
        .map_err(|e| format!("Error getting state lock: {}", e))?;

    let db = db_lock
        .as_ref()
        .ok_or_else(|| "No database connection. Initialize first.".to_string())?;

    let balance = db
        .get_balance_summary()
        .map_err(|e| format!("Error getting balance: {}", e))?;

    Ok(balance)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
