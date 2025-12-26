#![allow(
    clippy::collapsible_if,
    clippy::if_same_then_else,
    clippy::type_complexity
)]

use crate::models::{
    Account, AccountBalance, AggregatedAsset, BalanceSummary, CryptoTransaction,
    CryptoTransactionType, CryptoWallet, Habit, HabitLog, Transaction, TransactionCategory,
};
use crate::security_log::{SecurityEvent, log_security_event};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, Error as RusqliteError, ErrorCode, params};
use secrecy::{ExposeSecret, SecretString};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

/// Main struct wrapping the database connection
pub struct Database {
    conn: Connection,
    path: PathBuf,
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

    /// Executes necessary migrations to create tables
    fn run_migrations(&self) -> Result<(), DbError> {
        // ==================== FIAT Accounts Table ====================
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

        // Indices for accounts
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_accounts_archived ON accounts(is_archived)",
            [],
        )?;

        // ==================== Financial Transactions Table (v2 with accounts) ====================
        // Check if we need to migrate from old schema
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
            // Drop old transactions table (fresh start as per requirements)
            self.conn.execute("DROP TABLE IF EXISTS transactions", [])?;
        }

        // Create new transactions table with account support
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

        // Indices for transactions
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

        // ==================== Crypto Ledger System ====================
        self.create_crypto_ledger_tables()?;

        // ==================== Habits System ====================
        self.migrate_habits_tables()?;

        // ==================== Security Tables ====================
        self.create_security_tables()?;

        // ==================== Price Cache Tables ====================
        self.create_price_cache_tables()?;

        // ==================== Transaction Categories Table ====================
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

        // Index for category type queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transaction_categories_type ON transaction_categories(category_type, sort_order)",
            [],
        )?;

        // Initialize default categories if table is empty
        self.initialize_default_categories()?;

        // ==================== Settings Table ====================
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
        // Check if categories already exist
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM transaction_categories",
            [],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Ok(()); // Already initialized
        }

        let now = chrono::Utc::now().to_rfc3339();

        // Default expense categories
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

        // Default income categories
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

    /// Creates habits tracking tables
    fn migrate_habits_tables(&self) -> Result<(), DbError> {
        // Habits table - stores habit definitions
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

    /// Creates price cache tables for offline support
    fn create_price_cache_tables(&self) -> Result<(), DbError> {
        // Exchange rates cache (CLP/USD, etc.)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS exchange_rate_cache (
                currency_pair TEXT PRIMARY KEY NOT NULL,
                rate REAL NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Crypto prices cache
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

        // Daily crypto portfolio snapshots (for trend chart)
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

    // ==================== Price Cache Functions ====================

    /// Saves an exchange rate to cache (e.g., CLP_USD)
    pub fn save_exchange_rate(&self, pair: &str, rate: f64) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO exchange_rate_cache (currency_pair, rate, updated_at)
             VALUES (?1, ?2, ?3)",
            params![pair, rate, now],
        )?;
        Ok(())
    }

    /// Loads a cached exchange rate
    pub fn load_exchange_rate(&self, pair: &str) -> Result<Option<(f64, String)>, DbError> {
        let result = self.conn.query_row(
            "SELECT rate, updated_at FROM exchange_rate_cache WHERE currency_pair = ?1",
            params![pair],
            |row| Ok((row.get::<_, f64>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Ok((rate, updated_at)) => Ok(Some((rate, updated_at))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Saves a crypto price to cache
    pub fn save_crypto_price(
        &self,
        coin_id: &str,
        symbol: &str,
        name: &str,
        price_usd: f64,
        price_change_24h: f64,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO crypto_price_cache
             (coin_id, symbol, name, price_usd, price_change_24h, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![coin_id, symbol, name, price_usd, price_change_24h, now],
        )?;
        Ok(())
    }

    /// Loads all cached crypto prices
    pub fn load_crypto_prices(
        &self,
    ) -> Result<Vec<(String, String, String, f64, f64, String)>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT coin_id, symbol, name, price_usd, price_change_24h, updated_at
             FROM crypto_price_cache",
        )?;

        let prices = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        prices
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::Sqlite)
    }

    /// Loads a specific cached crypto price
    pub fn load_crypto_price(&self, coin_id: &str) -> Result<Option<(f64, String)>, DbError> {
        let result = self.conn.query_row(
            "SELECT price_usd, updated_at FROM crypto_price_cache WHERE coin_id = ?1",
            params![coin_id],
            |row| Ok((row.get::<_, f64>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Ok((price, updated_at)) => Ok(Some((price, updated_at))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Saves a daily crypto portfolio snapshot (upsert by date)
    pub fn save_crypto_portfolio_snapshot(
        &self,
        snapshot_date: &str,
        total_value_usd: f64,
        total_cost_usd: f64,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO crypto_portfolio_snapshots
             (snapshot_date, total_value_usd, total_cost_usd, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(snapshot_date) DO UPDATE SET
                total_value_usd = ?2,
                total_cost_usd = ?3,
                created_at = ?4",
            params![snapshot_date, total_value_usd, total_cost_usd, now],
        )?;
        Ok(())
    }

    /// Loads crypto portfolio snapshots from a starting date (inclusive)
    pub fn load_crypto_portfolio_snapshots(
        &self,
        start_date: &str,
    ) -> Result<Vec<(String, f64, f64)>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT snapshot_date, total_value_usd, total_cost_usd
             FROM crypto_portfolio_snapshots
             WHERE snapshot_date >= ?1
             ORDER BY snapshot_date ASC",
        )?;

        let snapshots = stmt.query_map(params![start_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })?;

        snapshots
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::Sqlite)
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
                > Duration::seconds(SESSION_TIMEOUT_SECS)
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
            return Ok((SESSION_TIMEOUT_SECS - elapsed).max(0));
        }

        Ok(SESSION_TIMEOUT_SECS)
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

    /// Verifies that the database is correctly configured and accessible
    pub fn health_check(&self) -> Result<(), DbError> {
        self.conn
            .query_row("SELECT 1", [], |_| Ok(()))
            .map_err(DbError::Sqlite)?;
        Ok(())
    }

    // ==================== FIAT Accounts CRUD ====================

    /// Creates a new account
    pub fn create_account(&self, account: &Account) -> Result<(), DbError> {
        if !account.validate() {
            return Err(DbError::InvalidAccountType);
        }

        self.conn.execute(
            "INSERT INTO accounts (id, name, type, currency, initial_balance, color, icon, is_archived, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &account.id,
                &account.name,
                &account.account_type,
                &account.currency,
                &account.initial_balance,
                &account.color,
                &account.icon,
                account.is_archived,
                &account.created_at,
            ],
        )?;

        Ok(())
    }

    /// Gets all non-archived accounts
    pub fn get_accounts(&self) -> Result<Vec<Account>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, currency, initial_balance, color, icon, is_archived, created_at
             FROM accounts
             WHERE is_archived = 0
             ORDER BY created_at ASC",
        )?;

        let accounts = stmt
            .query_map([], |row| {
                Ok(Account {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    account_type: row.get(2)?,
                    currency: row.get(3)?,
                    initial_balance: row.get(4)?,
                    color: row.get(5)?,
                    icon: row.get(6)?,
                    is_archived: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accounts)
    }

    /// Gets a single account by ID
    pub fn get_account(&self, id: &str) -> Result<Account, DbError> {
        self.conn
            .query_row(
                "SELECT id, name, type, currency, initial_balance, color, icon, is_archived, created_at
                 FROM accounts WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Account {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        account_type: row.get(2)?,
                        currency: row.get(3)?,
                        initial_balance: row.get(4)?,
                        color: row.get(5)?,
                        icon: row.get(6)?,
                        is_archived: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                },
            )
            .map_err(|e| match e {
                RusqliteError::QueryReturnedNoRows => DbError::AccountNotFound,
                _ => DbError::Sqlite(e),
            })
    }

    /// Updates an account
    pub fn update_account(&self, account: &Account) -> Result<(), DbError> {
        if !account.validate() {
            return Err(DbError::InvalidAccountType);
        }

        let rows = self.conn.execute(
            "UPDATE accounts SET name = ?1, type = ?2, currency = ?3, initial_balance = ?4, color = ?5, icon = ?6
             WHERE id = ?7 AND is_archived = 0",
            params![
                &account.name,
                &account.account_type,
                &account.currency,
                &account.initial_balance,
                &account.color,
                &account.icon,
                &account.id,
            ],
        )?;

        if rows == 0 {
            return Err(DbError::AccountNotFound);
        }

        Ok(())
    }

    /// Archives an account (soft delete) - only if it has no transactions
    pub fn archive_account(&self, id: &str) -> Result<(), DbError> {
        // Check if account has transactions
        let tx_count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE account_id = ?1 OR transfer_account_id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        if tx_count > 0 {
            return Err(DbError::AccountNotEmpty);
        }

        let rows = self.conn.execute(
            "UPDATE accounts SET is_archived = 1 WHERE id = ?1",
            params![id],
        )?;

        if rows == 0 {
            return Err(DbError::AccountNotFound);
        }

        Ok(())
    }

    /// Gets the calculated balance for an account
    pub fn get_account_balance(&self, account_id: &str) -> Result<AccountBalance, DbError> {
        // First get the account to verify it exists and get initial balance
        let account = self.get_account(account_id)?;

        // Calculate income (money coming IN to this account)
        let total_income: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE account_id = ?1 AND type = 'income'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Add transfers IN (where this account is the destination)
        let transfers_in: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE transfer_account_id = ?1 AND type = 'transfer'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Calculate expenses (money going OUT of this account)
        let total_expense: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE account_id = ?1 AND type = 'expense'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Add transfers OUT (where this account is the source)
        let transfers_out: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions
                 WHERE account_id = ?1 AND type = 'transfer'",
                params![account_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let current_balance =
            account.initial_balance + total_income + transfers_in - total_expense - transfers_out;

        Ok(AccountBalance {
            account_id: account_id.to_string(),
            account_name: account.name,
            current_balance,
            total_income: total_income + transfers_in,
            total_expense: total_expense + transfers_out,
        })
    }

    /// Gets balances for all non-archived accounts
    pub fn get_all_account_balances(&self) -> Result<Vec<AccountBalance>, DbError> {
        let accounts = self.get_accounts()?;
        let mut balances = Vec::with_capacity(accounts.len());

        for account in accounts {
            let balance = self.get_account_balance(&account.id)?;
            balances.push(balance);
        }

        Ok(balances)
    }

    // ==================== Financial Transactions CRUD ====================

    /// Creates a new transaction in the database
    pub fn create_transaction(&self, transaction: &Transaction) -> Result<(), DbError> {
        // Validate transaction
        if !transaction.validate() {
            return Err(DbError::InvalidTransactionType);
        }

        // Verify account exists
        self.get_account(&transaction.account_id)?;

        // For transfers, verify destination account exists and is different
        if let Some(ref transfer_id) = transaction.transfer_account_id {
            if transfer_id == &transaction.account_id {
                return Err(DbError::SameAccountTransfer);
            }
            self.get_account(transfer_id)?;
        }

        self.conn.execute(
            "INSERT INTO transactions (id, account_id, amount, category, description, date, type, transfer_account_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &transaction.id,
                &transaction.account_id,
                &transaction.amount,
                &transaction.category,
                &transaction.description,
                &transaction.date,
                &transaction.transaction_type,
                &transaction.transfer_account_id,
            ],
        )?;

        Ok(())
    }

    /// Creates a transfer between two accounts (atomic operation)
    pub fn create_transfer(
        &self,
        from_account_id: &str,
        to_account_id: &str,
        amount: i64,
        description: &str,
        date: &str,
    ) -> Result<String, DbError> {
        if from_account_id == to_account_id {
            return Err(DbError::SameAccountTransfer);
        }

        // Verify both accounts exist
        self.get_account(from_account_id)?;
        self.get_account(to_account_id)?;

        let tx_id = uuid::Uuid::new_v4().to_string();

        // Create a single transfer transaction (from source to destination)
        self.conn.execute(
            "INSERT INTO transactions (id, account_id, amount, category, description, date, type, transfer_account_id)
             VALUES (?1, ?2, ?3, 'Transfer', ?4, ?5, 'transfer', ?6)",
            params![
                &tx_id,
                from_account_id,
                amount,
                description,
                date,
                to_account_id,
            ],
        )?;

        Ok(tx_id)
    }

    /// Gets all transactions ordered by descending date
    pub fn get_transactions(&self) -> Result<Vec<Transaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, amount, category, description, date, type, transfer_account_id
             FROM transactions
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map([], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    amount: row.get(2)?,
                    category: row.get(3)?,
                    description: row.get(4)?,
                    date: row.get(5)?,
                    transaction_type: row.get(6)?,
                    transfer_account_id: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    /// Gets transactions for a specific account
    pub fn get_transactions_by_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<Transaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, amount, category, description, date, type, transfer_account_id
             FROM transactions
             WHERE account_id = ?1 OR transfer_account_id = ?1
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map(params![account_id], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    amount: row.get(2)?,
                    category: row.get(3)?,
                    description: row.get(4)?,
                    date: row.get(5)?,
                    transaction_type: row.get(6)?,
                    transfer_account_id: row.get(7)?,
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

    // ==================== Transaction Categories CRUD ====================

    /// Gets all categories of a specific type (expense or income)
    pub fn get_transaction_categories(
        &self,
        category_type: &str,
    ) -> Result<Vec<TransactionCategory>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, category_type, sort_order, is_default, created_at
             FROM transaction_categories
             WHERE category_type = ?1
             ORDER BY sort_order, name",
        )?;

        let categories = stmt
            .query_map(params![category_type], |row| {
                Ok(TransactionCategory {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category_type: row.get(2)?,
                    sort_order: row.get(3)?,
                    is_default: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(categories)
    }

    /// Adds a new transaction category
    pub fn add_transaction_category(
        &self,
        name: &str,
        category_type: &str,
    ) -> Result<String, DbError> {
        // Validate category type
        if category_type != "expense" && category_type != "income" {
            return Err(DbError::InvalidTransactionType);
        }

        // Check for duplicate names within the same type
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM transaction_categories WHERE name = ?1 AND category_type = ?2)",
            params![name, category_type],
            |row| row.get(0),
        )?;

        if exists {
            return Err(DbError::Sqlite(RusqliteError::ExecuteReturnedResults));
        }

        // Get max sort order for this type
        let max_sort: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM transaction_categories WHERE category_type = ?1",
            params![category_type],
            |row| row.get(0),
        )?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO transaction_categories (id, name, category_type, sort_order, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, category_type, max_sort + 1, 0, now],
        )?;

        Ok(id)
    }

    /// Updates a category name
    pub fn update_transaction_category(&self, id: &str, new_name: &str) -> Result<(), DbError> {
        // Check if category exists and get its type
        let category_type: String = self.conn.query_row(
            "SELECT category_type FROM transaction_categories WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        // Check for duplicate names within the same type (excluding current category)
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM transaction_categories WHERE name = ?1 AND category_type = ?2 AND id != ?3)",
            params![new_name, category_type, id],
            |row| row.get(0),
        )?;

        if exists {
            return Err(DbError::Sqlite(RusqliteError::ExecuteReturnedResults));
        }

        self.conn.execute(
            "UPDATE transaction_categories SET name = ?1 WHERE id = ?2",
            params![new_name, id],
        )?;

        Ok(())
    }

    /// Deletes a category (only if not default)
    pub fn delete_transaction_category(&self, id: &str) -> Result<(), DbError> {
        // Check if it's a default category
        let is_default: i32 = self.conn.query_row(
            "SELECT is_default FROM transaction_categories WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        if is_default != 0 {
            return Err(DbError::Sqlite(RusqliteError::ExecuteReturnedResults));
        }

        self.conn.execute(
            "DELETE FROM transaction_categories WHERE id = ?1",
            params![id],
        )?;

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

    /// Gets wallet transactions up to a given date (inclusive), ordered ascending.
    pub fn get_wallet_transactions_up_to_date(
        &self,
        wallet_id: &str,
        date: &str,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions
             WHERE wallet_id = ?1
               AND date <= ?2
             ORDER BY date ASC, rowid ASC",
        )?;

        let transactions = stmt
            .query_map(params![wallet_id, date], |row| {
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

    /// Gets the wallet balance for a coin at (or before) a given date.
    pub fn get_wallet_coin_balance_at(
        &self,
        wallet_id: &str,
        coin_id: &str,
        date: &str,
        exclude_tx_id: Option<&str>,
    ) -> Result<f64, DbError> {
        let mut balance = 0.0;
        if let Some(exclude) = exclude_tx_id {
            let mut stmt = self.conn.prepare(
                "SELECT coin_id, type, amount, fee_coin_id, fee_amount
                 FROM crypto_transactions
                 WHERE wallet_id = ?1
                   AND date <= ?2
                   AND id != ?3
                   AND (coin_id = ?4 OR fee_coin_id = ?4)",
            )?;
            let rows = stmt.query_map(params![wallet_id, date, exclude, coin_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<f64>>(4)?,
                ))
            })?;
            for row in rows {
                let (row_coin_id, tx_type, amount, fee_coin_id, fee_amount) = row?;
                if row_coin_id == coin_id {
                    if let Ok(kind) = tx_type.parse::<CryptoTransactionType>() {
                        match kind {
                            CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                                balance += amount
                            }
                            CryptoTransactionType::Sell
                            | CryptoTransactionType::TransferOut
                            | CryptoTransactionType::Swap => balance -= amount,
                        }
                    }
                }

                if let Some(fee_coin_id) = fee_coin_id {
                    if fee_coin_id == coin_id {
                        if let Some(fee_amount) = fee_amount {
                            balance -= fee_amount;
                        }
                    }
                }
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT coin_id, type, amount, fee_coin_id, fee_amount
                 FROM crypto_transactions
                 WHERE wallet_id = ?1
                   AND date <= ?2
                   AND (coin_id = ?3 OR fee_coin_id = ?3)",
            )?;
            let rows = stmt.query_map(params![wallet_id, date, coin_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<f64>>(4)?,
                ))
            })?;
            for row in rows {
                let (row_coin_id, tx_type, amount, fee_coin_id, fee_amount) = row?;
                if row_coin_id == coin_id {
                    if let Ok(kind) = tx_type.parse::<CryptoTransactionType>() {
                        match kind {
                            CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                                balance += amount
                            }
                            CryptoTransactionType::Sell
                            | CryptoTransactionType::TransferOut
                            | CryptoTransactionType::Swap => balance -= amount,
                        }
                    }
                }

                if let Some(fee_coin_id) = fee_coin_id {
                    if fee_coin_id == coin_id {
                        if let Some(fee_amount) = fee_amount {
                            balance -= fee_amount;
                        }
                    }
                }
            }
        }

        Ok(balance)
    }

    /// Gets the wallet balance and cost basis for a coin at (or before) a given date.
    pub fn get_wallet_coin_state_at(
        &self,
        wallet_id: &str,
        coin_id: &str,
        date: &str,
    ) -> Result<(f64, f64), DbError> {
        let mut transactions = self.get_wallet_transactions_up_to_date(wallet_id, date)?;

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

            if let Some(rel_id) = &tx.related_tx_id {
                if let Some(counter) = tx_map.get(rel_id) {
                    let is_transfer_pair = (tx.transaction_type == "transfer_out"
                        && counter.transaction_type == "transfer_in")
                        || (tx.transaction_type == "transfer_in"
                            && counter.transaction_type == "transfer_out");
                    let is_swap_pair = (tx.transaction_type == "swap"
                        && counter.transaction_type == "transfer_in")
                        || (tx.transaction_type == "transfer_in"
                            && counter.transaction_type == "swap");

                    if is_transfer_pair {
                        if processed.contains(rel_id) || processed.contains(&tx.id) {
                            continue;
                        }

                        let applied = if tx.transaction_type == "transfer_out" {
                            Self::apply_transfer_pair(&mut assets, &tx, counter)
                        } else {
                            Self::apply_transfer_pair(&mut assets, counter, &tx)
                        };

                        if applied {
                            processed.insert(tx.id.clone());
                            processed.insert(rel_id.clone());
                            continue;
                        }
                    }

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
                if let Some(price) = tx.price_per_coin {
                    let fee = tx.fee.unwrap_or(0.0);
                    entry.total_cost_basis += (tx.amount * price) + fee;
                }
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

            if let (Some(fee_coin_id), Some(fee_amount)) =
                (tx.fee_coin_id.as_deref(), tx.fee_amount)
            {
                let fee_symbol = if fee_coin_id == tx.coin_id {
                    Some(tx.symbol.as_str())
                } else {
                    None
                };
                Self::apply_fee_coin_outflow(&mut assets, fee_coin_id, fee_amount, fee_symbol);
            }
        }

        if let Some(asset) = assets.get(coin_id) {
            return Ok((asset.total_amount, asset.total_cost_basis));
        }

        Ok((0.0, 0.0))
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

    /// Gets all crypto transactions for a specific coin across all wallets
    pub fn get_crypto_transactions_by_coin(
        &self,
        coin_id: &str,
    ) -> Result<Vec<CryptoTransaction>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_id, coin_id, symbol, type, amount, price_per_coin, fee, fee_coin_id, fee_amount, date, notes, related_tx_id
             FROM crypto_transactions
             WHERE coin_id = ?1
             ORDER BY date DESC, rowid DESC",
        )?;

        let transactions = stmt
            .query_map(params![coin_id], |row| {
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

    /// Updates editable fields of a crypto transaction
    #[allow(clippy::too_many_arguments)]
    pub fn update_crypto_transaction_fields(
        &self,
        id: &str,
        amount: f64,
        price_per_coin: Option<f64>,
        fee: Option<f64>,
        fee_coin_id: Option<&str>,
        fee_amount: Option<f64>,
        date: &str,
        notes: Option<&str>,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE crypto_transactions
             SET amount = ?1,
                 price_per_coin = ?2,
                 fee = ?3,
                 fee_coin_id = ?4,
                 fee_amount = ?5,
                 date = ?6,
                 notes = ?7
             WHERE id = ?8",
            params![
                amount,
                price_per_coin,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
                id
            ],
        )?;
        Ok(())
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

        if let (Some(fee_coin_id), Some(fee_amount)) =
            (source.fee_coin_id.as_deref(), source.fee_amount)
        {
            let fee_symbol = if fee_coin_id == source.coin_id {
                Some(source.symbol.as_str())
            } else {
                None
            };
            Self::apply_fee_coin_outflow(assets, fee_coin_id, fee_amount, fee_symbol);
        }

        if let (Some(fee_coin_id), Some(fee_amount)) =
            (target.fee_coin_id.as_deref(), target.fee_amount)
        {
            let fee_symbol = if fee_coin_id == target.coin_id {
                Some(target.symbol.as_str())
            } else {
                None
            };
            Self::apply_fee_coin_outflow(assets, fee_coin_id, fee_amount, fee_symbol);
        }
    }

    fn apply_fee_coin_outflow(
        assets: &mut HashMap<String, AggregatedAsset>,
        fee_coin_id: &str,
        fee_amount: f64,
        fee_symbol: Option<&str>,
    ) {
        if fee_amount <= 0.0 {
            return;
        }

        let entry = assets.entry(fee_coin_id.to_string()).or_insert_with(|| {
            AggregatedAsset::new(
                fee_coin_id.to_string(),
                fee_symbol.unwrap_or(fee_coin_id).to_uppercase(),
            )
        });

        let prev_amount = entry.total_amount;
        entry.total_amount -= fee_amount;
        if entry.total_amount < 0.0 {
            entry.total_amount = 0.0;
        }

        if prev_amount > 0.0 {
            let ratio = (fee_amount / prev_amount).min(1.0);
            entry.total_cost_basis *= 1.0 - ratio;
            entry.total_cost_basis = entry.total_cost_basis.max(0.0);
        }
    }

    /// Applies a transfer pair for the same asset, reducing cost basis only for fee losses
    fn apply_transfer_pair(
        assets: &mut HashMap<String, AggregatedAsset>,
        source: &CryptoTransaction,
        target: &CryptoTransaction,
    ) -> bool {
        if source.coin_id != target.coin_id {
            return false;
        }

        let entry = assets
            .entry(source.coin_id.clone())
            .or_insert_with(|| AggregatedAsset::new(source.coin_id.clone(), source.symbol.clone()));

        let prev_amount = entry.total_amount;
        let prev_cost = entry.total_cost_basis;
        if prev_amount <= 0.0 {
            entry.total_amount = (entry.total_amount - source.amount).max(0.0) + target.amount;
            entry.total_cost_basis += target.fee.unwrap_or(0.0);
            return true;
        }

        let unit_cost = prev_cost / prev_amount;
        let cost_out = unit_cost * source.amount;

        entry.total_amount = (entry.total_amount - source.amount).max(0.0);
        entry.total_cost_basis = (entry.total_cost_basis - cost_out).max(0.0);

        entry.total_amount += target.amount;
        entry.total_cost_basis += unit_cost * target.amount;
        entry.total_cost_basis += target.fee.unwrap_or(0.0);

        if let (Some(fee_coin_id), Some(fee_amount)) =
            (source.fee_coin_id.as_deref(), source.fee_amount)
        {
            let fee_symbol = if fee_coin_id == source.coin_id {
                Some(source.symbol.as_str())
            } else {
                None
            };
            Self::apply_fee_coin_outflow(assets, fee_coin_id, fee_amount, fee_symbol);
        }

        if let (Some(fee_coin_id), Some(fee_amount)) =
            (target.fee_coin_id.as_deref(), target.fee_amount)
        {
            let fee_symbol = if fee_coin_id == target.coin_id {
                Some(target.symbol.as_str())
            } else {
                None
            };
            Self::apply_fee_coin_outflow(assets, fee_coin_id, fee_amount, fee_symbol);
        }
        true
    }

    fn aggregate_crypto_transactions(
        mut transactions: Vec<CryptoTransaction>,
    ) -> Vec<AggregatedAsset> {
        if transactions.is_empty() {
            return Vec::new();
        }

        // Process transactions chronologically to keep cost basis adjustments consistent
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

            // Handle swap/transfer pairs to carry over cost basis
            if let Some(rel_id) = &tx.related_tx_id {
                if let Some(counter) = tx_map.get(rel_id) {
                    let is_transfer_pair = (tx.transaction_type == "transfer_out"
                        && counter.transaction_type == "transfer_in")
                        || (tx.transaction_type == "transfer_in"
                            && counter.transaction_type == "transfer_out");
                    let is_swap_pair = (tx.transaction_type == "swap"
                        && counter.transaction_type == "transfer_in")
                        || (tx.transaction_type == "transfer_in"
                            && counter.transaction_type == "swap");

                    if is_transfer_pair {
                        if processed.contains(rel_id) || processed.contains(&tx.id) {
                            continue;
                        }

                        let applied = if tx.transaction_type == "transfer_out" {
                            Self::apply_transfer_pair(&mut assets, &tx, counter)
                        } else {
                            Self::apply_transfer_pair(&mut assets, counter, &tx)
                        };

                        if applied {
                            processed.insert(tx.id.clone());
                            processed.insert(rel_id.clone());
                            continue;
                        }
                    }

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
                    if let Some(price) = tx.price_per_coin {
                        let fee = tx.fee.unwrap_or(0.0);
                        entry.total_cost_basis += (tx.amount * price) + fee;
                    }
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

            if let (Some(fee_coin_id), Some(fee_amount)) =
                (tx.fee_coin_id.as_deref(), tx.fee_amount)
            {
                let fee_symbol = if fee_coin_id == tx.coin_id {
                    Some(tx.symbol.as_str())
                } else {
                    None
                };
                Self::apply_fee_coin_outflow(&mut assets, fee_coin_id, fee_amount, fee_symbol);
            }
        }

        assets
            .into_values()
            .filter_map(|mut asset| {
                if asset.total_amount > 0.0001 {
                    asset.calculate_avg_price();
                    Some(asset)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Calculates aggregated portfolio from all transactions across all wallets
    /// This is the CRITICAL function that computes total holdings per coin
    pub fn get_aggregated_portfolio(&self) -> Result<Vec<AggregatedAsset>, DbError> {
        let transactions = self.get_all_crypto_transactions()?;
        Ok(Self::aggregate_crypto_transactions(transactions))
    }

    /// Gets aggregated holdings for a specific wallet
    pub fn get_wallet_aggregated_holdings(
        &self,
        wallet_id: &str,
    ) -> Result<Vec<AggregatedAsset>, DbError> {
        let transactions = self.get_wallet_transactions(wallet_id)?;
        Ok(Self::aggregate_crypto_transactions(transactions))
    }

    // ==================== Balance Summary ====================

    /// Gets the balance summary (income, expenses and total) including account initial balances
    pub fn get_balance_summary(&self) -> Result<BalanceSummary, DbError> {
        // Get sum of all initial balances from non-archived accounts
        let initial_balances: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(initial_balance), 0) FROM accounts WHERE is_archived = 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Get income and expenses from transactions
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

        // Total balance = initial balances + income - expenses
        let total_balance = initial_balances + total_income - total_expense;

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
            "INSERT INTO habits (id, name, description, color, category, created_at, archived)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &habit.id,
                &habit.name,
                &habit.description,
                &habit.color,
                &habit.category,
                &habit.created_at,
                habit.archived as i32,
            ],
        )?;
        Ok(())
    }

    /// Gets all active (non-archived) habits
    pub fn get_habits(&self) -> Result<Vec<Habit>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, color, category, created_at, archived
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
                    category: row.get(4)?,
                    created_at: row.get(5)?,
                    archived: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(habits)
    }

    /// Gets a single habit by ID
    pub fn get_habit(&self, id: &str) -> Result<Option<Habit>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, name, description, color, category, created_at, archived
             FROM habits WHERE id = ?1",
            params![id],
            |row| {
                Ok(Habit {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    category: row.get(4)?,
                    created_at: row.get(5)?,
                    archived: row.get::<_, i32>(6)? != 0,
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
            "UPDATE habits SET name = ?1, description = ?2, color = ?3, category = ?4 WHERE id = ?5",
            params![
                &habit.name,
                &habit.description,
                &habit.color,
                &habit.category,
                &habit.id
            ],
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

    /// Toggles a habit completion for a specific date
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
}

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
