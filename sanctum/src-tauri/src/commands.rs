use crate::db::Database;
use std::sync::Mutex;
use tauri::{AppHandle, State};

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
