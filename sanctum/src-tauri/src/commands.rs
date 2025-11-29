use crate::db::Database;
use crate::models::Transaction;
use std::sync::Mutex;
use tauri::{AppHandle, State};
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
    // Validar que la contraseña no esté vacía
    if password.trim().is_empty() {
        return Err("La contraseña no puede estar vacía".to_string());
    }

    // Verificar si ya existe una conexión inicializada
    {
        let db_lock = state
            .db
            .lock()
            .map_err(|e| format!("Error al obtener el lock del estado: {}", e))?;

        if db_lock.is_some() {
            return Err(
                "La base de datos ya está inicializada. Cierra la conexión actual primero."
                    .to_string(),
            );
        }
    }

    // Intentar inicializar la base de datos
    let database = Database::init(&app_handle, &password)
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

    Ok("Base de datos inicializada correctamente".to_string())
}

/// Comando para verificar si la base de datos está inicializada
#[tauri::command]
pub fn is_db_initialized(state: State<DbState>) -> Result<bool, String> {
    Ok(state.is_initialized())
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
pub fn get_db_path(app_handle: AppHandle) -> Result<String, String> {
    let path = Database::get_db_path(&app_handle)
        .map_err(|e| format!("Error al obtener la ruta: {}", e))?;

    Ok(path.to_string_lossy().to_string())
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

    if description.trim().is_empty() {
        return Err("La descripción no puede estar vacía".to_string());
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
