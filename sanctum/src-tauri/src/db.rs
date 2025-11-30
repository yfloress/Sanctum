use crate::models::{BalanceSummary, Transaction};
use rusqlite::{Connection, Error as RusqliteError, ErrorCode, params};
use secrecy::{ExposeSecret, SecretString};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use thiserror::Error;

/// Errores personalizados para operaciones de base de datos
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Could not access application data directory")]
    AppDataDir,

    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Could not open vault")]
    InvalidPassword,

    #[error("Could not create data directory")]
    DirectoryCreation,

    #[error("Invalid transaction type")]
    InvalidTransactionType,
}

/// Struct principal que envuelve la conexión a la base de datos
pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    /// Inicializa la base de datos con encriptación SQLCipher
    /// Usa SecretString para manejar la contraseña de forma segura
    pub fn init(
        app_handle: &AppHandle,
        password: &SecretString,
        db_path: Option<PathBuf>,
    ) -> Result<Self, DbError> {
        // Resolver la ruta objetivo
        let db_path = match db_path {
            Some(path) => path,
            None => Self::default_db_path(app_handle)?,
        };

        // Crear el directorio si no existe
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|_| DbError::DirectoryCreation)?;
            }
        }

        let is_new_db = !db_path.exists();

        // Abrir conexión a la base de datos
        let conn = Connection::open(&db_path)?;

        // --- ZONA DE SEGURIDAD Y CONFIGURACIÓN ---
        // 1. Establecer la contraseña (Encriptación)
        // Usamos pragma_update para evitar SQL Injection de forma segura
        // ExposeSecret permite acceder al valor interno de forma controlada
        conn.pragma_update(None, "key", password.expose_secret())
            .map_err(|_| DbError::InvalidPassword)?;

        // 1.0 Endurecer la configuración de SQLCipher una vez aplicada la clave
        Self::apply_sqlcipher_hardening(&conn, is_new_db)?;

        // 1.1 Validar contraseña con integrity check para fallar rápido en caso de clave incorrecta
        if !is_new_db {
            Self::verify_key(&conn)?;
        }

        // 2. Activar modo WAL (Rendimiento)
        // Usamos pragma_update porque WAL retorna string "wal" y execute fallaría
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(DbError::Sqlite)?;

        // -----------------------------------------

        // Crear instancia de Database
        let db = Database {
            conn,
            path: db_path,
        };

        // Ejecutar migraciones
        db.run_migrations()?;

        Ok(db)
    }

    /// Ajusta PRAGMAs defensivos de SQLCipher para la conexión
    fn apply_sqlcipher_hardening(conn: &Connection, is_new_db: bool) -> Result<(), DbError> {
        // Asegurar limpieza de buffers sensibles
        conn.pragma_update(None, "cipher_memory_security", true)
            .map_err(DbError::Sqlite)?;

        // Solo para bases nuevas configuramos parámetros que afectan el layout en disco
        if is_new_db {
            // Forzar algoritmos fuertes (defaults de SQLCipher 4, explícitos para evitar degradación)
            conn.pragma_update(None, "cipher_hmac_algorithm", "HMAC_SHA512")
                .map_err(DbError::Sqlite)?;
            conn.pragma_update(None, "cipher_kdf_algorithm", "PBKDF2_HMAC_SHA512")
                .map_err(DbError::Sqlite)?;

            // Fortalecer parámetros de derivación y layout
            conn.pragma_update(None, "kdf_iter", 256_000i64)
                .map_err(DbError::Sqlite)?;
            conn.pragma_update(None, "cipher_page_size", 4096i64)
                .map_err(DbError::Sqlite)?;
        }

        Ok(())
    }

    /// Valida que la clave sea correcta ejecutando cipher_integrity_check
    fn verify_key(conn: &Connection) -> Result<(), DbError> {
        let result = conn.pragma_query_value(None, "cipher_integrity_check", |row| {
            row.get::<_, String>(0)
        });

        match result {
            Ok(value) => {
                if value.to_lowercase() != "ok" {
                    Err(DbError::InvalidPassword)
                } else {
                    Ok(())
                }
            }
            // Algunos builds retornan QueryReturnedNoRows aunque la clave sea correcta.
            // En ese caso, hacemos una consulta segura a sqlite_master para validar acceso.
            Err(RusqliteError::QueryReturnedNoRows) => {
                conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
                    .map_err(|e| match e {
                        RusqliteError::SqliteFailure(ref code, _)
                            if matches!(
                                code.code,
                                ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt
                            ) =>
                        {
                            DbError::InvalidPassword
                        }
                        _ => DbError::InvalidPassword,
                    })?;
                Ok(())
            }
            Err(RusqliteError::SqliteFailure(ref code, _))
                if matches!(
                    code.code,
                    ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt
                ) =>
            {
                Err(DbError::InvalidPassword)
            }
            Err(_) => Err(DbError::InvalidPassword),
        }
    }

    /// Ruta por defecto en el directorio de datos de la aplicación
    pub fn default_db_path(app_handle: &AppHandle) -> Result<PathBuf, DbError> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|_| DbError::AppDataDir)?;
        Ok(app_data_dir.join("sanctum.db"))
    }

    /// Ruta actual de la conexión
    pub fn path(&self) -> &PathBuf {
        &self.path
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

    /// Verifica que la base de datos esté correctamente configurada y accesible
    pub fn health_check(&self) -> Result<(), DbError> {
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

    /// Obtiene todas las transacciones ordenadas por fecha descendente
    pub fn get_transactions(&self) -> Result<Vec<Transaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, category, description, date, type
             FROM transactions
             ORDER BY date DESC, id DESC",
        )?;

        let transactions = stmt
            .query_map([], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    amount: row.get(1)?,
                    category: row.get(2)?,
                    description: row.get(3)?,
                    date: row.get(4)?,
                    transaction_type: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Obtiene el resumen de balance (ingresos, gastos y total) en una sola query optimizada
    pub fn get_balance_summary(&self) -> Result<BalanceSummary, DbError> {
        let (total_income, total_expense): (i64, i64) = self
            .conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN type = 'income' THEN amount ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN type = 'expense' THEN amount ELSE 0 END), 0)
                 FROM transactions",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(DbError::Sqlite)?;

        let total_balance = total_income - total_expense;

        Ok(BalanceSummary {
            total_balance,
            total_income,
            total_expense,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_error_display() {
        let error = DbError::InvalidPassword;
        // Mensaje genérico que no revela información sensible
        assert_eq!(error.to_string(), "Could not open vault");
    }

    #[test]
    fn test_db_error_generic_messages() {
        // Verificar que los mensajes de error no revelan detalles internos
        assert_eq!(
            DbError::AppDataDir.to_string(),
            "Could not access application data directory"
        );
        assert_eq!(
            DbError::DirectoryCreation.to_string(),
            "Could not create data directory"
        );
        assert_eq!(
            DbError::InvalidTransactionType.to_string(),
            "Invalid transaction type"
        );
    }
}
