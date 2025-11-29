use crate::models::Transaction;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use thiserror::Error;

/// Errores personalizados para operaciones de base de datos
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Error de SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Error al obtener el directorio de datos de la aplicación")]
    AppDataDir,

    #[error("Error de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("La contraseña de la base de datos es inválida")]
    InvalidPassword,

    #[error("Error al crear el directorio de datos")]
    DirectoryCreation,

    #[error("Tipo de transacción inválido")]
    InvalidTransactionType,
}

/// Struct principal que envuelve la conexión a la base de datos
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Inicializa la base de datos con encriptación SQLCipher
    pub fn init(app_handle: &AppHandle, password: &str) -> Result<Self, DbError> {
        // Obtener el directorio de datos de la aplicación
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|_| DbError::AppDataDir)?;

        // Crear el directorio si no existe
        if !app_data_dir.exists() {
            std::fs::create_dir_all(&app_data_dir).map_err(|_| DbError::DirectoryCreation)?;
        }

        // Construir la ruta completa de la base de datos
        let db_path = app_data_dir.join("sanctum.db");

        // Abrir conexión a la base de datos
        let conn = Connection::open(&db_path)?;

        // --- ZONA DE SEGURIDAD Y CONFIGURACIÓN ---

        // 1. Establecer la contraseña (Encriptación)
        // Usamos pragma_update para evitar SQL Injection de forma segura
        conn.pragma_update(None, "key", password)
            .map_err(|_| DbError::InvalidPassword)?;

        // 2. Activar modo WAL (Rendimiento)
        // Usamos pragma_update porque WAL retorna string "wal" y execute fallaría
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(DbError::Sqlite)?;

        // -----------------------------------------

        // Crear instancia de Database
        let db = Database { conn };

        // Ejecutar migraciones
        db.run_migrations()?;

        Ok(db)
    }

    /// Ejecuta las migraciones necesarias para crear las tablas
    fn run_migrations(&self) -> Result<(), DbError> {
        // Verificar si la tabla existe y tiene la columna 'type'
        let table_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transactions'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        if table_exists {
            // Verificar si la columna 'type' existe
            let has_type_column: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name='type'",
                    [],
                    |row| row.get::<_, i32>(0),
                )
                .unwrap_or(0)
                > 0;

            if !has_type_column {
                // Migrar tabla antigua a nueva estructura
                self.conn
                    .execute("ALTER TABLE transactions RENAME TO transactions_old", [])?;
                self.conn.execute(
                    "CREATE TABLE transactions (
                        id TEXT PRIMARY KEY NOT NULL,
                        amount INTEGER NOT NULL,
                        category TEXT NOT NULL,
                        description TEXT NOT NULL,
                        date TEXT NOT NULL,
                        type TEXT NOT NULL
                    )",
                    [],
                )?;
                // Copiar datos existentes (asumiendo 'expense' como default)
                self.conn.execute(
                    "INSERT INTO transactions (id, amount, category, description, date, type)
                     SELECT id, amount, category, description, date, 'expense' FROM transactions_old",
                    [],
                )?;
                self.conn.execute("DROP TABLE transactions_old", [])?;
            }
        } else {
            // Crear tabla nueva con la columna 'type'
            self.conn.execute(
                "CREATE TABLE transactions (
                    id TEXT PRIMARY KEY NOT NULL,
                    amount INTEGER NOT NULL,
                    category TEXT NOT NULL,
                    description TEXT NOT NULL,
                    date TEXT NOT NULL,
                    type TEXT NOT NULL
                )",
                [],
            )?;
        }

        // Crear índices para búsquedas rápidas
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(type)",
            [],
        )?;

        Ok(())
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Verifica que la base de datos esté correctamente configurada y accesible
    pub fn health_check(&self) -> Result<(), DbError> {
        // CORRECCIÓN CRÍTICA:
        // Usamos query_row en lugar de execute.
        // `execute` espera 0 filas afectadas. `SELECT 1` devuelve una fila.
        // `query_row` consume esa fila y retorna Ok, evitando el error.
        self.conn
            .query_row("SELECT 1", [], |_| Ok(()))
            .map_err(DbError::Sqlite)?;
        Ok(())
    }

    /// Crea una nueva transacción en la base de datos
    pub fn create_transaction(&self, transaction: &Transaction) -> Result<(), DbError> {
        // Validar tipo de transacción
        if !transaction.validate_type() {
            return Err(DbError::InvalidTransactionType);
        }

        self.conn.execute(
            "INSERT INTO transactions (id, amount, category, description, date, type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &transaction.id,
                &transaction.amount,
                &transaction.category,
                &transaction.description,
                &transaction.date,
                &transaction.transaction_type,
            ],
        )?;

        Ok(())
    }

    pub fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf, DbError> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|_| DbError::AppDataDir)?;

        Ok(app_data_dir.join("sanctum.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_error_display() {
        let error = DbError::InvalidPassword;
        assert_eq!(
            error.to_string(),
            "La contraseña de la base de datos es inválida"
        );
    }
}
