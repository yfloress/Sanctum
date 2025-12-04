use crate::models::{
    AggregatedAsset, BalanceSummary, CryptoHolding, CryptoTransaction, CryptoTransactionType,
    CryptoWallet, Habit, HabitLog, Transaction,
};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, Error as RusqliteError, ErrorCode, params};
use secrecy::{ExposeSecret, SecretString};
use std::collections::{HashMap, HashSet};
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

    #[error("Wallet not found")]
    WalletNotFound,

    #[error("Invalid wallet category")]
    InvalidWalletCategory,

    #[error("Wallet has existing transactions")]
    WalletNotEmpty,

    #[error("Session expired due to inactivity")]
    SessionExpired,

    #[error("Too many failed attempts")]
    RateLimited,
}

// ==================== Security Constants ====================

/// Maximum failed authentication attempts before lockout
pub const MAX_FAILED_ATTEMPTS: u32 = 5;

/// Lockout duration in seconds after max failed attempts (5 minutes)
pub const LOCKOUT_DURATION_SECS: i64 = 300;

/// Time window to reset failed attempts counter (60 seconds)
pub const ATTEMPT_RESET_SECS: i64 = 60;

/// Session timeout duration in seconds (15 minutes of inactivity)
pub const SESSION_TIMEOUT_SECS: i64 = 900;

/// KDF iterations for PBKDF2-HMAC-SHA512 (OWASP 2024 recommendation)
pub const KDF_ITERATIONS: i64 = 600_000;

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
        if let Some(parent) = db_path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|_| DbError::DirectoryCreation)?;
        }

        let is_new_db = !db_path.exists();

        // Abrir conexión a la base de datos
        let conn = Connection::open(&db_path)?;

        // Enforce foreign key constraints for the connection
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(DbError::Sqlite)?;

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

            // Fortalecer parámetros de derivación y layout (OWASP 2024: 600,000 para PBKDF2-SHA512)
            conn.pragma_update(None, "kdf_iter", KDF_ITERATIONS)
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
        // ==================== Financial Transactions Table ====================
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

        // ==================== Crypto Ledger System Migration ====================
        self.migrate_crypto_ledger()?;

        // ==================== Habits System ====================
        self.migrate_habits_tables()?;

        // ==================== Security Tables ====================
        self.create_security_tables()?;

        Ok(())
    }

    /// Creates habits tracking tables
    fn migrate_habits_tables(&self) -> Result<(), DbError> {
        // Habits table - stores habit definitions
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS habits (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT NOT NULL DEFAULT '#8b5cf6',
                created_at TEXT NOT NULL,
                archived INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // Habit logs table - stores completion records
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS habit_logs (
                id TEXT PRIMARY KEY NOT NULL,
                habit_id TEXT NOT NULL,
                completed_date TEXT NOT NULL,
                FOREIGN KEY (habit_id) REFERENCES habits(id) ON DELETE CASCADE,
                UNIQUE(habit_id, completed_date)
            )",
            [],
        )?;

        // Create indices for fast lookups
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_habits_archived ON habits(archived)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_habit_logs_habit_id ON habit_logs(habit_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_habit_logs_date ON habit_logs(completed_date)",
            [],
        )?;

        // Composite index for range queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_habit_logs_habit_date ON habit_logs(habit_id, completed_date)",
            [],
        )?;

        Ok(())
    }

    /// Creates security-related tables for rate limiting and session management
    fn create_security_tables(&self) -> Result<(), DbError> {
        // Rate limiting table - persists failed authentication attempts
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_attempts (
                vault_path TEXT PRIMARY KEY NOT NULL,
                failed_count INTEGER NOT NULL DEFAULT 0,
                locked_until TEXT,
                last_attempt TEXT NOT NULL
            )",
            [],
        )?;

        // Session tracking table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS session_info (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_activity TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Initialize session on vault open
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO session_info (id, last_activity, created_at) VALUES (1, ?1, ?1)",
            params![&now],
        )?;

        Ok(())
    }

    // ==================== Rate Limiting Functions ====================

    /// Records a failed authentication attempt for a vault path
    pub fn record_failed_attempt(
        conn: &Connection,
        vault_path: &str,
    ) -> Result<(u32, bool), DbError> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        // Get current state
        let current: Option<(i32, Option<String>, String)> = conn
            .query_row(
                "SELECT failed_count, locked_until, last_attempt FROM auth_attempts WHERE vault_path = ?1",
                params![vault_path],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        let (mut failed_count, locked_until, last_attempt) =
            current.unwrap_or((0, None, now_str.clone()));

        // Check if we should reset the counter (enough time has passed)
        if let Ok(last) = DateTime::parse_from_rfc3339(&last_attempt)
            && now.signed_duration_since(last.with_timezone(&Utc))
                > Duration::seconds(ATTEMPT_RESET_SECS)
        {
            failed_count = 0;
        }

        // Check if currently locked
        if let Some(ref locked_str) = locked_until
            && let Ok(locked) = DateTime::parse_from_rfc3339(locked_str)
        {
            if now < locked.with_timezone(&Utc) {
                // Still locked, reject the attempt
                return Err(DbError::RateLimited);
            }
            // Lock expired, reset
            failed_count = 0;
        }

        // Increment counter
        failed_count += 1;
        let is_locked = failed_count >= MAX_FAILED_ATTEMPTS as i32;

        let new_locked_until = if is_locked {
            Some((now + Duration::seconds(LOCKOUT_DURATION_SECS)).to_rfc3339())
        } else {
            None
        };

        // Upsert the record
        conn.execute(
            "INSERT INTO auth_attempts (vault_path, failed_count, locked_until, last_attempt)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(vault_path) DO UPDATE SET
                failed_count = ?2,
                locked_until = ?3,
                last_attempt = ?4",
            params![vault_path, failed_count, &new_locked_until, &now_str],
        )?;

        Ok((failed_count as u32, is_locked))
    }

    /// Checks if a vault path is currently rate limited
    pub fn check_rate_limit(conn: &Connection, vault_path: &str) -> Result<(), DbError> {
        let result: Option<(i32, Option<String>, String)> = conn
            .query_row(
                "SELECT failed_count, locked_until, last_attempt FROM auth_attempts WHERE vault_path = ?1",
                params![vault_path],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        if let Some((failed_count, locked_until, last_attempt)) = result {
            let now = Utc::now();

            // Check if locked
            if let Some(locked_str) = locked_until
                && let Ok(locked) = DateTime::parse_from_rfc3339(&locked_str)
                && now < locked.with_timezone(&Utc)
            {
                return Err(DbError::RateLimited);
            }

            // Check if we should still count previous attempts
            if let Ok(last) = DateTime::parse_from_rfc3339(&last_attempt)
                && now.signed_duration_since(last.with_timezone(&Utc))
                    <= Duration::seconds(ATTEMPT_RESET_SECS)
                && failed_count >= MAX_FAILED_ATTEMPTS as i32
            {
                return Err(DbError::RateLimited);
            }
        }

        Ok(())
    }

    /// Resets rate limit after successful authentication
    pub fn reset_rate_limit(conn: &Connection, vault_path: &str) -> Result<(), DbError> {
        conn.execute(
            "DELETE FROM auth_attempts WHERE vault_path = ?1",
            params![vault_path],
        )?;
        Ok(())
    }

    /// Gets remaining lockout time in seconds (0 if not locked)
    pub fn get_lockout_remaining(conn: &Connection, vault_path: &str) -> Result<u64, DbError> {
        let locked_until: Option<String> = conn
            .query_row(
                "SELECT locked_until FROM auth_attempts WHERE vault_path = ?1",
                params![vault_path],
                |row| row.get(0),
            )
            .ok()
            .flatten();

        if let Some(locked_str) = locked_until
            && let Ok(locked) = DateTime::parse_from_rfc3339(&locked_str)
        {
            let now = Utc::now();
            if now < locked.with_timezone(&Utc) {
                return Ok((locked.with_timezone(&Utc) - now).num_seconds() as u64);
            }
        }

        Ok(0)
    }

    // ==================== Session Management Functions ====================

    /// Updates the last activity timestamp
    pub fn touch_session(&self) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE session_info SET last_activity = ?1 WHERE id = 1",
            params![&now],
        )?;
        Ok(())
    }

    /// Checks if the session has expired due to inactivity
    pub fn check_session_timeout(&self) -> Result<(), DbError> {
        let last_activity: String = self.conn.query_row(
            "SELECT last_activity FROM session_info WHERE id = 1",
            [],
            |row| row.get(0),
        )?;

        if let Ok(last) = DateTime::parse_from_rfc3339(&last_activity) {
            let now = Utc::now();
            if now.signed_duration_since(last.with_timezone(&Utc))
                > Duration::seconds(SESSION_TIMEOUT_SECS)
            {
                return Err(DbError::SessionExpired);
            }
        }

        // Update last activity on successful check
        self.touch_session()?;
        Ok(())
    }

    /// Gets seconds until session expires (for UI display)
    pub fn get_session_remaining(&self) -> Result<i64, DbError> {
        let last_activity: String = self.conn.query_row(
            "SELECT last_activity FROM session_info WHERE id = 1",
            [],
            |row| row.get(0),
        )?;

        if let Ok(last) = DateTime::parse_from_rfc3339(&last_activity) {
            let now = Utc::now();
            let elapsed = now
                .signed_duration_since(last.with_timezone(&Utc))
                .num_seconds();
            return Ok((SESSION_TIMEOUT_SECS - elapsed).max(0));
        }

        Ok(SESSION_TIMEOUT_SECS)
    }

    /// Migrates from old crypto_holdings to new ledger system
    fn migrate_crypto_ledger(&self) -> Result<(), DbError> {
        // Check if old crypto_holdings table exists
        let old_table_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crypto_holdings'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        // Check if new tables exist
        let wallets_exist: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crypto_wallets'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        // Create new tables if they don't exist
        if !wallets_exist {
            // Create crypto_wallets table
            self.conn.execute(
                "CREATE TABLE crypto_wallets (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    category TEXT NOT NULL,
                    icon TEXT
                )",
                [],
            )?;

            // Create crypto_transactions table
            self.conn.execute(
                "CREATE TABLE crypto_transactions (
                    id TEXT PRIMARY KEY NOT NULL,
                    wallet_id TEXT NOT NULL,
                    coin_id TEXT NOT NULL,
                    symbol TEXT NOT NULL,
                    type TEXT NOT NULL,
                    amount REAL NOT NULL,
                    price_per_coin REAL,
                    fee REAL,
                    fee_coin_id TEXT,
                    fee_amount REAL,
                    date TEXT NOT NULL,
                    notes TEXT,
                    related_tx_id TEXT,
                    FOREIGN KEY (wallet_id) REFERENCES crypto_wallets(id) ON DELETE CASCADE
                )",
                [],
            )?;

            // Create indexes for crypto tables
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_crypto_wallets_category ON crypto_wallets(category)",
                [],
            )?;

            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_crypto_tx_wallet ON crypto_transactions(wallet_id)",
                [],
            )?;

            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_crypto_tx_coin ON crypto_transactions(coin_id)",
                [],
            )?;

            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_crypto_tx_date ON crypto_transactions(date)",
                [],
            )?;

            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_crypto_tx_type ON crypto_transactions(type)",
                [],
            )?;

            // Index for related_tx_id lookups (used in swap/transfer deletions)
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_crypto_tx_related ON crypto_transactions(related_tx_id)",
                [],
            )?;
        }

        // Migrate old data if exists
        if old_table_exists && wallets_exist {
            // Check if we have already migrated (by checking for legacy wallet)
            let legacy_wallet_exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM crypto_wallets WHERE id = 'legacy_portfolio'",
                    [],
                    |row| row.get::<_, i32>(0),
                )
                .unwrap_or(0)
                > 0;

            if !legacy_wallet_exists {
                // Check if there are holdings to migrate
                let holdings_count: i32 = self
                    .conn
                    .query_row("SELECT COUNT(*) FROM crypto_holdings", [], |row| row.get(0))
                    .unwrap_or(0);

                if holdings_count > 0 {
                    // Create a legacy wallet for migrated holdings
                    self.conn.execute(
                        "INSERT INTO crypto_wallets (id, name, category, icon) VALUES (?1, ?2, ?3, ?4)",
                        params!["legacy_portfolio", "Legacy Portfolio", "wallet_multi", "📦"],
                    )?;

                    // Migrate holdings as buy transactions
                    self.conn.execute(
                        "INSERT INTO crypto_transactions (id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, date, notes)
                         SELECT
                            'migrated_' || id,
                            'legacy_portfolio',
                            coin_id,
                            symbol,
                            'buy',
                            amount,
                            purchase_price,
                            NULL,
                            purchase_date,
                            'Migrated from legacy holdings'
                         FROM crypto_holdings",
                        [],
                    )?;
                }

                // Rename old table as backup
                self.conn.execute(
                    "ALTER TABLE crypto_holdings RENAME TO crypto_holdings_backup",
                    [],
                )?;
            }
        } else if !old_table_exists && !wallets_exist {
            // Fresh install - tables already created above
        }

        Ok(())
    }

    /// Verifica que la base de datos esté correctamente configurada y accesible
    pub fn health_check(&self) -> Result<(), DbError> {
        self.conn
            .query_row("SELECT 1", [], |_| Ok(()))
            .map_err(DbError::Sqlite)?;
        Ok(())
    }

    // ==================== Financial Transactions CRUD ====================

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
             ORDER BY date DESC, rowid DESC",
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

    /// Deletes a transaction by its ID
    pub fn delete_transaction(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM transactions WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== Legacy Crypto Holdings CRUD (for backwards compatibility) ====================

    /// Creates a new crypto holding in the database (LEGACY)
    pub fn create_crypto_holding(&self, holding: &CryptoHolding) -> Result<(), DbError> {
        if !holding.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        // Check if legacy backup table exists, if not check for original
        let backup_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crypto_holdings_backup'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        if backup_exists {
            // Insert into backup table for legacy support
            self.conn.execute(
                "INSERT INTO crypto_holdings_backup (id, coin_id, symbol, amount, purchase_price, purchase_date)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    &holding.id,
                    &holding.coin_id,
                    &holding.symbol,
                    &holding.amount,
                    &holding.purchase_price,
                    &holding.purchase_date,
                ],
            )?;
        }

        Ok(())
    }

    /// Gets all crypto holdings ordered by coin_id (LEGACY)
    pub fn get_crypto_holdings(&self) -> Result<Vec<CryptoHolding>, DbError> {
        // Try backup table first, then original
        let table_name = if self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crypto_holdings_backup'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0)
            > 0
        {
            "crypto_holdings_backup"
        } else if self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crypto_holdings'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0)
            > 0
        {
            "crypto_holdings"
        } else {
            return Ok(Vec::new());
        };

        let query = format!(
            "SELECT id, coin_id, symbol, amount, purchase_price, purchase_date
             FROM {}
             ORDER BY coin_id ASC, purchase_date DESC",
            table_name
        );

        let mut stmt = self.conn.prepare(&query)?;

        let holdings = stmt
            .query_map([], |row| {
                Ok(CryptoHolding {
                    id: row.get(0)?,
                    coin_id: row.get(1)?,
                    symbol: row.get(2)?,
                    amount: row.get(3)?,
                    purchase_price: row.get(4)?,
                    purchase_date: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(holdings)
    }

    /// Deletes a crypto holding by its ID (LEGACY)
    pub fn delete_crypto_holding(&self, id: &str) -> Result<(), DbError> {
        // Try backup table first
        let backup_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crypto_holdings_backup'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        if backup_exists {
            self.conn.execute(
                "DELETE FROM crypto_holdings_backup WHERE id = ?1",
                params![id],
            )?;
        } else {
            self.conn
                .execute("DELETE FROM crypto_holdings WHERE id = ?1", params![id])?;
        }
        Ok(())
    }

    // ==================== Crypto Wallets CRUD ====================

    /// Creates a new crypto wallet
    pub fn create_wallet(&self, wallet: &CryptoWallet) -> Result<(), DbError> {
        if !wallet.validate() {
            return Err(DbError::InvalidWalletCategory);
        }

        self.conn.execute(
            "INSERT INTO crypto_wallets (id, name, category, icon) VALUES (?1, ?2, ?3, ?4)",
            params![&wallet.id, &wallet.name, &wallet.category, &wallet.icon],
        )?;

        Ok(())
    }

    /// Gets all wallets
    pub fn get_wallets(&self) -> Result<Vec<CryptoWallet>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, category, icon FROM crypto_wallets ORDER BY name ASC")?;

        let wallets = stmt
            .query_map([], |row| {
                Ok(CryptoWallet {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    icon: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(wallets)
    }

    /// Gets a single wallet by ID
    pub fn get_wallet(&self, id: &str) -> Result<Option<CryptoWallet>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, name, category, icon FROM crypto_wallets WHERE id = ?1",
            params![id],
            |row| {
                Ok(CryptoWallet {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    icon: row.get(3)?,
                })
            },
        );

        match result {
            Ok(wallet) => Ok(Some(wallet)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Updates a wallet
    pub fn update_wallet(&self, wallet: &CryptoWallet) -> Result<(), DbError> {
        if !wallet.validate() {
            return Err(DbError::InvalidWalletCategory);
        }

        let rows = self.conn.execute(
            "UPDATE crypto_wallets SET name = ?2, category = ?3, icon = ?4 WHERE id = ?1",
            params![&wallet.id, &wallet.name, &wallet.category, &wallet.icon],
        )?;

        if rows == 0 {
            return Err(DbError::WalletNotFound);
        }

        Ok(())
    }

    /// Deletes a wallet and all its transactions
    pub fn delete_wallet(&self, id: &str) -> Result<(), DbError> {
        // Block deletion if wallet has existing transactions
        let tx_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM crypto_transactions WHERE wallet_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if tx_count > 0 {
            return Err(DbError::WalletNotEmpty);
        }

        let rows = self
            .conn
            .execute("DELETE FROM crypto_wallets WHERE id = ?1", params![id])?;

        if rows == 0 {
            return Err(DbError::WalletNotFound);
        }

        Ok(())
    }

    // ==================== Crypto Transactions CRUD ====================

    /// Creates a new crypto transaction
    pub fn create_crypto_transaction(&self, tx: &CryptoTransaction) -> Result<(), DbError> {
        if !tx.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        // Verify wallet exists
        let wallet_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM crypto_wallets WHERE id = ?1",
                params![&tx.wallet_id],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        if !wallet_exists {
            return Err(DbError::WalletNotFound);
        }

        self.conn.execute(
            "INSERT INTO crypto_transactions
             (id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                &tx.id,
                &tx.wallet_id,
                &tx.coin_id,
                &tx.symbol,
                &tx.transaction_type,
                &tx.amount,
                &tx.price_per_coin,
                &tx.fee,
                &tx.fee_coin_id,
                &tx.fee_amount,
                &tx.date,
                &tx.notes,
                &tx.related_tx_id,
            ],
        )?;

        Ok(())
    }

    /// Gets all transactions for a specific wallet
    pub fn get_wallet_transactions(
        &self,
        wallet_id: &str,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions
             WHERE wallet_id = ?1
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map(params![wallet_id], |row| {
                Ok(CryptoTransaction {
                    id: row.get(0)?,
                    wallet_id: row.get(1)?,
                    coin_id: row.get(2)?,
                    symbol: row.get(3)?,
                    transaction_type: row.get(4)?,
                    amount: row.get(5)?,
                    price_per_coin: row.get(6)?,
                    fee: row.get(7)?,
                    fee_coin_id: row.get(8)?,
                    fee_amount: row.get(9)?,
                    date: row.get(10)?,
                    notes: row.get(11)?,
                    related_tx_id: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Gets all crypto transactions across all wallets
    pub fn get_all_crypto_transactions(&self) -> Result<Vec<CryptoTransaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map([], |row| {
                Ok(CryptoTransaction {
                    id: row.get(0)?,
                    wallet_id: row.get(1)?,
                    coin_id: row.get(2)?,
                    symbol: row.get(3)?,
                    transaction_type: row.get(4)?,
                    amount: row.get(5)?,
                    price_per_coin: row.get(6)?,
                    fee: row.get(7)?,
                    fee_coin_id: row.get(8)?,
                    fee_amount: row.get(9)?,
                    date: row.get(10)?,
                    notes: row.get(11)?,
                    related_tx_id: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Deletes a crypto transaction by ID
    pub fn delete_crypto_transaction(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM crypto_transactions WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Gets a crypto transaction by ID
    pub fn get_crypto_transaction(&self, id: &str) -> Result<Option<CryptoTransaction>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions WHERE id = ?1",
            params![id],
            |row| {
                Ok(CryptoTransaction {
                    id: row.get(0)?,
                    wallet_id: row.get(1)?,
                    coin_id: row.get(2)?,
                    symbol: row.get(3)?,
                    transaction_type: row.get(4)?,
                    amount: row.get(5)?,
                    price_per_coin: row.get(6)?,
                    fee: row.get(7)?,
                    fee_coin_id: row.get(8)?,
                    fee_amount: row.get(9)?,
                    date: row.get(10)?,
                    notes: row.get(11)?,
                    related_tx_id: row.get(12)?,
                })
            },
        );

        match result {
            Ok(tx) => Ok(Some(tx)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    // ==================== Portfolio Aggregation ====================

    /// Applies a swap pair atomically to source and target assets to preserve cost basis
    fn apply_swap_pair(
        assets: &mut HashMap<String, AggregatedAsset>,
        source: &CryptoTransaction,
        target: &CryptoTransaction,
    ) {
        let source_entry = assets
            .entry(source.coin_id.clone())
            .or_insert_with(|| AggregatedAsset::new(source.coin_id.clone(), source.symbol.clone()));

        // Capture state before the swap to compute proportional cost
        let prev_amount = source_entry.total_amount;
        let prev_cost = source_entry.total_cost_basis;

        // Compute cost transferred using proportional cost basis of the source asset
        let proportion = if prev_amount > 0.0 {
            (source.amount / prev_amount).min(1.0)
        } else {
            0.0
        };
        let cost_transferred = prev_cost * proportion + source.fee.unwrap_or(0.0);

        // Apply outflow on source asset
        source_entry.total_amount -= source.amount;
        if source_entry.total_amount < 0.0 {
            source_entry.total_amount = 0.0;
        }
        source_entry.total_cost_basis = (source_entry.total_cost_basis - cost_transferred).max(0.0);

        // Apply inflow on target asset
        let target_entry = assets
            .entry(target.coin_id.clone())
            .or_insert_with(|| AggregatedAsset::new(target.coin_id.clone(), target.symbol.clone()));
        target_entry.total_amount += target.amount;
        target_entry.total_cost_basis += cost_transferred.max(0.0);
    }

    /// Calculates aggregated portfolio from all transactions across all wallets
    /// This is the CRITICAL function that computes total holdings per coin
    pub fn get_aggregated_portfolio(&self) -> Result<Vec<AggregatedAsset>, DbError> {
        let mut transactions = self.get_all_crypto_transactions()?;

        // Process transactions chronologically to keep cost basis adjustments consistent
        transactions.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));

        let tx_map: HashMap<String, CryptoTransaction> = transactions
            .iter()
            .cloned()
            .map(|tx| (tx.id.clone(), tx))
            .collect();

        let mut processed: HashSet<String> = HashSet::new();

        // Group transactions by coin_id and calculate totals
        let mut assets: HashMap<String, AggregatedAsset> = HashMap::new();

        for tx in transactions {
            if processed.contains(&tx.id) {
                continue;
            }

            // Handle swap pairs to carry over cost basis to the acquired asset
            if let Some(rel_id) = &tx.related_tx_id
                && let Some(counter) = tx_map.get(rel_id)
            {
                let is_swap_pair = (tx.transaction_type == "swap"
                    && counter.transaction_type == "transfer_in")
                    || (tx.transaction_type == "transfer_in" && counter.transaction_type == "swap");

                if is_swap_pair {
                    if processed.contains(rel_id) || processed.contains(&tx.id) {
                        continue;
                    }

                    processed.insert(tx.id.clone());
                    processed.insert(rel_id.clone());

                    // Determine which side is the source (swap out) and target (swap in)
                    if tx.transaction_type == "swap" {
                        Self::apply_swap_pair(&mut assets, &tx, counter);
                    } else {
                        Self::apply_swap_pair(&mut assets, counter, &tx);
                    }
                    continue;
                }
            }

            let tx_type = match tx.transaction_type.parse::<CryptoTransactionType>() {
                Ok(t) => t,
                Err(_) => continue, // Skip invalid transaction types
            };

            let entry = assets
                .entry(tx.coin_id.clone())
                .or_insert_with(|| AggregatedAsset::new(tx.coin_id.clone(), tx.symbol.clone()));

            match tx_type {
                CryptoTransactionType::Buy => {
                    entry.total_amount += tx.amount;
                    let cost = tx.amount * tx.price_per_coin.unwrap_or(0.0);
                    let fee = tx.fee.unwrap_or(0.0);
                    entry.total_cost_basis += cost + fee;
                }
                CryptoTransactionType::TransferIn => {
                    entry.total_amount += tx.amount;
                }
                CryptoTransactionType::Sell
                | CryptoTransactionType::TransferOut
                | CryptoTransactionType::Swap => {
                    let prev_amount = entry.total_amount;
                    entry.total_amount -= tx.amount;
                    if entry.total_amount < 0.0 {
                        entry.total_amount = 0.0;
                    }
                    if prev_amount > 0.0 {
                        let ratio = (tx.amount / prev_amount).min(1.0);
                        entry.total_cost_basis *= 1.0 - ratio;
                        entry.total_cost_basis = entry.total_cost_basis.max(0.0);
                    }
                }
            }
        }

        // Calculate average buy prices and filter out zero/negative balances
        let result: Vec<AggregatedAsset> = assets
            .into_values()
            .filter_map(|mut asset| {
                if asset.total_amount > 0.0001 {
                    // Small threshold to handle floating point errors
                    asset.calculate_avg_price();
                    Some(asset)
                } else {
                    None
                }
            })
            .collect();

        Ok(result)
    }

    /// Gets aggregated holdings for a specific wallet
    pub fn get_wallet_aggregated_holdings(
        &self,
        wallet_id: &str,
    ) -> Result<Vec<AggregatedAsset>, DbError> {
        let mut transactions = self.get_wallet_transactions(wallet_id)?;

        // Process chronologically
        transactions.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));

        let tx_map: HashMap<String, CryptoTransaction> = transactions
            .iter()
            .cloned()
            .map(|tx| (tx.id.clone(), tx))
            .collect();
        let mut processed: HashSet<String> = HashSet::new();

        let mut assets: HashMap<String, AggregatedAsset> = HashMap::new();

        for tx in transactions {
            if processed.contains(&tx.id) {
                continue;
            }

            if let Some(rel_id) = &tx.related_tx_id
                && let Some(counter) = tx_map.get(rel_id)
            {
                let is_swap_pair = (tx.transaction_type == "swap"
                    && counter.transaction_type == "transfer_in")
                    || (tx.transaction_type == "transfer_in" && counter.transaction_type == "swap");

                if is_swap_pair {
                    if processed.contains(rel_id) || processed.contains(&tx.id) {
                        continue;
                    }

                    processed.insert(tx.id.clone());
                    processed.insert(rel_id.clone());

                    if tx.transaction_type == "swap" {
                        Self::apply_swap_pair(&mut assets, &tx, counter);
                    } else {
                        Self::apply_swap_pair(&mut assets, counter, &tx);
                    }
                    continue;
                }
            }

            let tx_type = match tx.transaction_type.parse::<CryptoTransactionType>() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let entry = assets
                .entry(tx.coin_id.clone())
                .or_insert_with(|| AggregatedAsset::new(tx.coin_id.clone(), tx.symbol.clone()));

            if matches!(tx_type, CryptoTransactionType::Buy) {
                entry.total_amount += tx.amount;
                let cost = tx.amount * tx.price_per_coin.unwrap_or(0.0);
                entry.total_cost_basis += cost + tx.fee.unwrap_or(0.0);
            } else if matches!(tx_type, CryptoTransactionType::TransferIn) {
                entry.total_amount += tx.amount;
            } else if tx_type.is_outflow() || matches!(tx_type, CryptoTransactionType::Swap) {
                let prev_amount = entry.total_amount;
                entry.total_amount -= tx.amount;
                if entry.total_amount < 0.0 {
                    entry.total_amount = 0.0;
                }
                if prev_amount > 0.0 {
                    let ratio = (tx.amount / prev_amount).min(1.0);
                    entry.total_cost_basis *= 1.0 - ratio;
                    entry.total_cost_basis = entry.total_cost_basis.max(0.0);
                }
            }
        }

        let result: Vec<AggregatedAsset> = assets
            .into_values()
            .filter_map(|mut asset| {
                if asset.total_amount > 0.0001 {
                    asset.calculate_avg_price();
                    Some(asset)
                } else {
                    None
                }
            })
            .collect();

        Ok(result)
    }

    // ==================== Balance Summary ====================

    /// Gets the balance summary (income, expenses and total) in a single optimized query
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

    // ==================== Habits CRUD ====================

    /// Creates a new habit
    pub fn create_habit(&self, habit: &Habit) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO habits (id, name, description, color, created_at, archived)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &habit.id,
                &habit.name,
                &habit.description,
                &habit.color,
                &habit.created_at,
                habit.archived as i32,
            ],
        )?;
        Ok(())
    }

    /// Gets all active (non-archived) habits
    pub fn get_habits(&self) -> Result<Vec<Habit>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, color, created_at, archived
             FROM habits
             WHERE archived = 0
             ORDER BY created_at ASC",
        )?;

        let habits = stmt
            .query_map([], |row| {
                Ok(Habit {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    archived: row.get::<_, i32>(5)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(habits)
    }

    /// Gets a single habit by ID
    pub fn get_habit(&self, id: &str) -> Result<Option<Habit>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, name, description, color, created_at, archived
             FROM habits WHERE id = ?1",
            params![id],
            |row| {
                Ok(Habit {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    archived: row.get::<_, i32>(5)? != 0,
                })
            },
        );

        match result {
            Ok(habit) => Ok(Some(habit)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Updates an existing habit
    pub fn update_habit(&self, habit: &Habit) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE habits SET name = ?1, description = ?2, color = ?3 WHERE id = ?4",
            params![&habit.name, &habit.description, &habit.color, &habit.id],
        )?;
        Ok(())
    }

    /// Archives a habit (soft delete)
    pub fn archive_habit(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("UPDATE habits SET archived = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Permanently deletes a habit and all its logs
    pub fn delete_habit(&self, id: &str) -> Result<(), DbError> {
        // Logs are deleted via CASCADE
        self.conn
            .execute("DELETE FROM habits WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== Habit Logs CRUD ====================

    /// Creates a habit completion log
    pub fn create_habit_log(&self, log: &HabitLog) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO habit_logs (id, habit_id, completed_date)
             VALUES (?1, ?2, ?3)",
            params![&log.id, &log.habit_id, &log.completed_date],
        )?;
        Ok(())
    }

    /// Deletes a habit log (uncomplete)
    pub fn delete_habit_log(&self, habit_id: &str, date: &str) -> Result<bool, DbError> {
        let rows = self.conn.execute(
            "DELETE FROM habit_logs WHERE habit_id = ?1 AND completed_date = ?2",
            params![habit_id, date],
        )?;
        Ok(rows > 0)
    }

    /// Checks if a habit log exists for a given date
    pub fn habit_log_exists(&self, habit_id: &str, date: &str) -> Result<bool, DbError> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM habit_logs WHERE habit_id = ?1 AND completed_date = ?2",
            params![habit_id, date],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Gets all habit logs within a date range
    pub fn get_habit_logs(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<HabitLog>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, habit_id, completed_date
             FROM habit_logs
             WHERE completed_date >= ?1 AND completed_date <= ?2
             ORDER BY completed_date ASC",
        )?;

        let logs = stmt
            .query_map(params![start_date, end_date], |row| {
                Ok(HabitLog {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    completed_date: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Gets habit logs for a specific habit within a date range
    pub fn get_habit_logs_for_habit(
        &self,
        habit_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<HabitLog>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, habit_id, completed_date
             FROM habit_logs
             WHERE habit_id = ?1 AND completed_date >= ?2 AND completed_date <= ?3
             ORDER BY completed_date ASC",
        )?;

        let logs = stmt
            .query_map(params![habit_id, start_date, end_date], |row| {
                Ok(HabitLog {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    completed_date: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Toggles a habit completion for a specific date
    /// Returns (created: bool, log_id: Option<String>)
    /// - created=true means a new log was created, log_id contains the new ID
    /// - created=false means an existing log was deleted, log_id is None
    pub fn toggle_habit_log(
        &self,
        habit_id: &str,
        date: &str,
    ) -> Result<(bool, Option<String>), DbError> {
        if self.habit_log_exists(habit_id, date)? {
            self.delete_habit_log(habit_id, date)?;
            Ok((false, None))
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            let log = HabitLog::new(id.clone(), habit_id.to_string(), date.to_string());
            self.create_habit_log(&log)?;
            Ok((true, Some(id)))
        }
    }

    /// Gets completion statistics for habits in a date range
    pub fn get_habit_stats(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<(String, i32)>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT habit_id, COUNT(*) as completions
             FROM habit_logs
             WHERE completed_date >= ?1 AND completed_date <= ?2
             GROUP BY habit_id",
        )?;

        let stats = stmt
            .query_map(params![start_date, end_date], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(stats)
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
        assert_eq!(DbError::WalletNotFound.to_string(), "Wallet not found");
        assert_eq!(
            DbError::InvalidWalletCategory.to_string(),
            "Invalid wallet category"
        );
        assert_eq!(
            DbError::WalletNotEmpty.to_string(),
            "Wallet has existing transactions"
        );
        assert_eq!(
            DbError::SessionExpired.to_string(),
            "Session expired due to inactivity"
        );
        assert_eq!(DbError::RateLimited.to_string(), "Too many failed attempts");
    }
}
