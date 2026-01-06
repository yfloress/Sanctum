//! Database module
//!
//! Provides encrypted SQLite database operations using SQLCipher.
//! Split into domain-specific submodules for maintainability.

#![allow(
    clippy::collapsible_if,
    clippy::if_same_then_else,
    clippy::type_complexity
)]

mod crypto;
mod finance;
mod habits;

use crate::security_log::{SecurityEvent, log_security_event};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, Error as RusqliteError, ErrorCode, params};
use secrecy::{ExposeSecret, SecretString};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// ==================== Error Types ====================

/// Custom errors for database operations
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

    #[error("Account not found")]
    AccountNotFound,

    #[error("Account has existing transactions")]
    AccountNotEmpty,

    #[error("Invalid account type")]
    InvalidAccountType,

    #[error("Cannot transfer to the same account")]
    SameAccountTransfer,

    #[error("Transaction not found")]
    TransactionNotFound,
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

// ==================== Database Struct ====================

/// Main struct wrapping the database connection
pub struct Database {
    conn: Connection,
    path: PathBuf,
    session_timeout: i64,  // Configurable session timeout in seconds
}

impl Database {
    /// Initializes the database with SQLCipher encryption
    /// Uses SecretString to handle password securely
    ///
    /// # Arguments
    /// * `db_path` - Required path to database file
    /// * `password` - Password to encrypt/decrypt the database
    pub fn init(db_path: PathBuf, password: &SecretString) -> Result<Self, DbError> {
        // Create directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|_| DbError::DirectoryCreation)?;
            }
            #[cfg(unix)]
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
                .map_err(|_| DbError::DirectoryCreation)?;
        }

        let is_new_db = !db_path.exists();

        // Open database connection
        let conn = Connection::open(&db_path)?;

        // Ensure restrictive permissions on the vault file
        #[cfg(unix)]
        fs::set_permissions(&db_path, fs::Permissions::from_mode(0o600)).map_err(DbError::Io)?;

        // Enforce foreign key constraints for the connection
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(DbError::Sqlite)?;

        // --- SECURITY AND CONFIGURATION ZONE ---
        // 1. Set password (Encryption)
        // Use pragma_update to avoid SQL Injection safely
        // ExposeSecret allows controlled access to internal value
        conn.pragma_update(None, "key", password.expose_secret())
            .map_err(|_| DbError::InvalidPassword)?;

        // 1.0 Harden SQLCipher configuration once key is applied
        Self::apply_sqlcipher_hardening(&conn, is_new_db)?;

        // 1.1 Validate password with integrity check to fail fast on incorrect key
        if !is_new_db {
            Self::verify_key(&conn)?;
        }

        // 2. Enable WAL mode (Performance)
        // Use pragma_update because WAL returns string "wal" and execute would fail
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(DbError::Sqlite)?;

        // -----------------------------------------

        // Create Database instance
        let db = Database {
            conn,
            path: db_path,
            session_timeout: SESSION_TIMEOUT_SECS,  // Default 15 minutes
        };

        // Run migrations
        db.run_migrations()?;

        // Verify and display security settings
        db.verify_encryption_settings()?;

        Ok(db)
    }

    /// Adjusts defensive SQLCipher PRAGMAs for the connection
    /// IMPORTANT: Algorithm parameters must be applied BEFORE attempting to decrypt
    /// for both new and existing DBs, as they define how to interpret the key.
    fn apply_sqlcipher_hardening(conn: &Connection, is_new_db: bool) -> Result<(), DbError> {
        // Ensure cleanup of sensitive buffers
        conn.pragma_update(None, "cipher_memory_security", true)
            .map_err(DbError::Sqlite)?;

        // Encryption algorithms - MUST always match those used when creating the DB
        // These parameters affect how the key is derived and HMAC is verified
        conn.pragma_update(None, "cipher_hmac_algorithm", "HMAC_SHA512")
            .map_err(DbError::Sqlite)?;
        conn.pragma_update(None, "cipher_kdf_algorithm", "PBKDF2_HMAC_SHA512")
            .map_err(DbError::Sqlite)?;
        conn.pragma_update(None, "kdf_iter", KDF_ITERATIONS)
            .map_err(DbError::Sqlite)?;
        conn.pragma_update(None, "cipher_page_size", 4096i64)
            .map_err(DbError::Sqlite)?;

        // Log only on new DB creation
        if is_new_db {
            log_security_event(SecurityEvent::VaultCreated, Some("SQLCipher hardened"));
        }

        Ok(())
    }

    /// Verifies current SQLCipher encryption parameters
    /// Only available in debug builds for auditing
    #[cfg(debug_assertions)]
    pub fn verify_encryption_settings(&self) -> Result<(), DbError> {
        use log::debug;

        let cipher = self
            .conn
            .pragma_query_value(None, "cipher", |row| row.get::<_, String>(0))
            .unwrap_or_else(|_| "unknown".to_string());
        let kdf = self
            .conn
            .pragma_query_value(None, "cipher_kdf_algorithm", |row| row.get::<_, String>(0))
            .unwrap_or_else(|_| "unknown".to_string());
        let iterations = self
            .conn
            .pragma_query_value(None, "kdf_iter", |row| row.get::<_, i64>(0))
            .unwrap_or(0);

        debug!(
            "[CRYPTO] cipher={} kdf={} iterations={}",
            cipher, kdf, iterations
        );

        Ok(())
    }

    /// No-op en release builds
    #[cfg(not(debug_assertions))]
    pub fn verify_encryption_settings(&self) -> Result<(), DbError> {
        Ok(())
    }

    /// Validates that the key is correct by attempting to read from database
    fn verify_key(conn: &Connection) -> Result<(), DbError> {
        // If key is incorrect, SQLCipher will return "file is not a database"
        match conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i32>(0)
        }) {
            Ok(_) => Ok(()),
            Err(e) => {
                // Any error reading sqlite_master indicates incorrect key or corrupt DB
                match e {
                    RusqliteError::SqliteFailure(ref code, _) => {
                        if matches!(
                            code.code,
                            ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt
                        ) {
                            Err(DbError::InvalidPassword)
                        } else {
                            Err(DbError::InvalidPassword)
                        }
                    }
                    _ => Err(DbError::InvalidPassword),
                }
            }
        }
    }

    /// Current path of the connection
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Verifies that the database is correctly configured and accessible
    pub fn health_check(&self) -> Result<(), DbError> {
        self.conn
            .query_row("SELECT 1", [], |_| Ok(()))
            .map_err(DbError::Sqlite)?;
        Ok(())
    }

    // ==================== Settings Methods ====================

    /// Gets a setting value by key
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Sets a setting value (upsert)
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
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
        if let Ok(last) = DateTime::parse_from_rfc3339(&last_attempt) {
            if now.signed_duration_since(last.with_timezone(&Utc))
                > Duration::seconds(ATTEMPT_RESET_SECS)
            {
                failed_count = 0;
            }
        }

        // Check if currently locked
        if let Some(ref locked_str) = locked_until {
            if let Ok(locked) = DateTime::parse_from_rfc3339(locked_str) {
                if now < locked.with_timezone(&Utc) {
                    // Still locked, reject the attempt
                    return Err(DbError::RateLimited);
                }
                // Lock expired, reset
                failed_count = 0;
            }
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
            if let Some(locked_str) = locked_until {
                if let Ok(locked) = DateTime::parse_from_rfc3339(&locked_str) {
                    if now < locked.with_timezone(&Utc) {
                        return Err(DbError::RateLimited);
                    }
                }
            }

            // Check if we should still count previous attempts
            if let Ok(last) = DateTime::parse_from_rfc3339(&last_attempt) {
                if now.signed_duration_since(last.with_timezone(&Utc))
                    <= Duration::seconds(ATTEMPT_RESET_SECS)
                    && failed_count >= MAX_FAILED_ATTEMPTS as i32
                {
                    return Err(DbError::RateLimited);
                }
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

        if let Some(locked_str) = locked_until {
            if let Ok(locked) = DateTime::parse_from_rfc3339(&locked_str) {
                let now = Utc::now();
                if now < locked.with_timezone(&Utc) {
                    return Ok((locked.with_timezone(&Utc) - now).num_seconds() as u64);
                }
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
                > Duration::seconds(self.session_timeout)
            {
                return Err(DbError::SessionExpired);
            }
        }
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
            return Ok((self.session_timeout - elapsed).max(0));
        }

        Ok(self.session_timeout)
    }

    /// Sets the session timeout duration (in seconds)
    pub fn set_session_timeout(&mut self, timeout_secs: i64) {
        self.session_timeout = timeout_secs;
    }

    // ==================== Migrations ====================

    /// Executes necessary migrations to create tables
    fn run_migrations(&self) -> Result<(), DbError> {
        // FIAT Accounts Table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                type TEXT NOT NULL DEFAULT 'bank',
                currency TEXT NOT NULL DEFAULT 'USD',
                initial_balance INTEGER NOT NULL DEFAULT 0,
                color TEXT NOT NULL DEFAULT '#8b5cf6',
                icon TEXT,
                is_archived INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_accounts_archived ON accounts(is_archived)",
            [],
        )?;

        // Financial Transactions Table
        let has_account_id: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name='account_id'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0)
            > 0;

        if !has_account_id {
            self.conn.execute("DROP TABLE IF EXISTS transactions", [])?;
        }

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY NOT NULL,
                account_id TEXT NOT NULL,
                amount INTEGER NOT NULL,
                category TEXT NOT NULL,
                description TEXT NOT NULL,
                date TEXT NOT NULL,
                type TEXT NOT NULL,
                transfer_account_id TEXT,
                FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
                FOREIGN KEY (transfer_account_id) REFERENCES accounts(id) ON DELETE RESTRICT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id)",
            [],
        )?;
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

        // Crypto Ledger System
        self.create_crypto_ledger_tables()?;

        // Habits System
        self.migrate_habits_tables()?;

        // Security Tables
        self.create_security_tables()?;

        // Price Cache Tables
        self.create_price_cache_tables()?;

        // Transaction Categories Table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transaction_categories (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                category_type TEXT NOT NULL CHECK(category_type IN ('expense', 'income')),
                sort_order INTEGER NOT NULL DEFAULT 0,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transaction_categories_type ON transaction_categories(category_type, sort_order)",
            [],
        )?;

        self.initialize_default_categories()?;

        // Settings Table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    fn initialize_default_categories(&self) -> Result<(), DbError> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM transaction_categories", [], |row| {
                    row.get(0)
                })?;

        if count > 0 {
            return Ok(());
        }

        let now = chrono::Utc::now().to_rfc3339();

        let expense_categories = [
            "FOOD",
            "TRANSPORT",
            "UTILITIES",
            "ENTERTAINMENT",
            "HEALTH",
            "SHOPPING",
            "EDUCATION",
            "OTHER",
        ];

        for (idx, name) in expense_categories.iter().enumerate() {
            let id = format!("exp_{}", uuid::Uuid::new_v4());
            self.conn.execute(
                "INSERT INTO transaction_categories (id, name, category_type, sort_order, is_default, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, name, "expense", idx as i32, 1, now],
            )?;
        }

        let income_categories = ["SALARY", "FREELANCE", "INVESTMENT", "GIFT", "OTHER"];

        for (idx, name) in income_categories.iter().enumerate() {
            let id = format!("inc_{}", uuid::Uuid::new_v4());
            self.conn.execute(
                "INSERT INTO transaction_categories (id, name, category_type, sort_order, is_default, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, name, "income", idx as i32, 1, now],
            )?;
        }

        Ok(())
    }

    fn migrate_habits_tables(&self) -> Result<(), DbError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS habits (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT NOT NULL DEFAULT '#8b5cf6',
                category TEXT NOT NULL DEFAULT 'mind',
                created_at TEXT NOT NULL,
                archived INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

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
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_habit_logs_habit_date ON habit_logs(habit_id, completed_date)",
            [],
        )?;

        let has_category: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('habits') WHERE name='category'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0)
            > 0;

        if !has_category {
            self.conn.execute(
                "ALTER TABLE habits ADD COLUMN category TEXT NOT NULL DEFAULT 'mind'",
                [],
            )?;
        }

        Ok(())
    }

    fn create_security_tables(&self) -> Result<(), DbError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_attempts (
                vault_path TEXT PRIMARY KEY NOT NULL,
                failed_count INTEGER NOT NULL DEFAULT 0,
                locked_until TEXT,
                last_attempt TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS session_info (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_activity TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO session_info (id, last_activity, created_at) VALUES (1, ?1, ?1)",
            params![&now],
        )?;

        Ok(())
    }

    fn create_price_cache_tables(&self) -> Result<(), DbError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS exchange_rate_cache (
                currency_pair TEXT PRIMARY KEY NOT NULL,
                rate REAL NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS crypto_price_cache (
                coin_id TEXT PRIMARY KEY NOT NULL,
                symbol TEXT NOT NULL,
                name TEXT NOT NULL,
                price_usd REAL NOT NULL,
                price_change_24h REAL NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS crypto_portfolio_snapshots (
                snapshot_date TEXT PRIMARY KEY NOT NULL,
                total_value_usd REAL NOT NULL,
                total_cost_usd REAL NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    fn create_crypto_ledger_tables(&self) -> Result<(), DbError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS crypto_wallets (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                icon TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS crypto_transactions (
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
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_crypto_tx_related ON crypto_transactions(related_tx_id)",
            [],
        )?;

        Ok(())
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_error_display() {
        let error = DbError::InvalidPassword;
        assert_eq!(error.to_string(), "Could not open vault");
    }

    #[test]
    fn test_db_error_generic_messages() {
        assert_eq!(
            DbError::AppDataDir.to_string(),
            "Could not access application data directory"
        );
        assert_eq!(
            DbError::DirectoryCreation.to_string(),
            "Could not create data directory"
        );
        assert_eq!(DbError::WalletNotFound.to_string(), "Wallet not found");
        assert_eq!(
            DbError::SessionExpired.to_string(),
            "Session expired due to inactivity"
        );
        assert_eq!(DbError::RateLimited.to_string(), "Too many failed attempts");
    }
}
