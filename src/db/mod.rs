// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

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
mod migrations;

use crate::security_log::{SecurityEvent, log_security_event};
use chrono::{DateTime, Duration, Utc};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Error as RusqliteError, ErrorCode, params};
use secrecy::{ExposeSecret, SecretString};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};
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

    #[error("Database mutex was poisoned")]
    MutexPoisoned,

    #[error("Database not open - vault is locked")]
    DatabaseNotOpen,

    #[error("Database pool error")]
    Pool,
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

/// Busy timeout (ms) applied to every connection so transient locks (e.g. WAL
/// checkpoints) retry instead of failing immediately with SQLITE_BUSY.
const BUSY_TIMEOUT_MS: u64 = 5_000;

/// Minimum seconds between persisting `session_info` to disk. Activity is tracked
/// in-memory (`last_activity`); the row is only rewritten this often to avoid a
/// disk write on every read.
const SESSION_PERSIST_INTERVAL_SECS: i64 = 30;

/// Number of read-only connections in the pool. Mirrors WAL's "many readers, one
/// writer" model, bounded so we never spawn an unreasonable number of connections.
fn reader_pool_size() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
        .clamp(2, 8)
}

// ==================== Database Struct ====================

/// Connection pool of read-only connections to the encrypted vault.
type ReaderPool = r2d2::Pool<SqliteConnectionManager>;

/// Main struct wrapping the database connections.
///
/// Mirrors SQLite's WAL concurrency model at the application level: a single
/// serialized writer connection plus a pool of read-only connections, so that
/// reads run concurrently (with each other and with the writer) instead of being
/// serialized behind one global lock.
pub struct Database {
    /// The single writer connection, serialized through a Mutex (SQLite only
    /// allows one writer at a time anyway).
    writer: Mutex<Connection>,
    /// Pool of read-only (`query_only`) connections for concurrent reads.
    readers: ReaderPool,
    path: PathBuf,
    session_timeout: i64, // Configurable session timeout in seconds
    /// Last activity timestamp (epoch secs), tracked in memory so reads don't
    /// trigger a disk write. Persisted to `session_info` periodically.
    last_activity: AtomicI64,
    /// Last time `session_info` was persisted to disk (epoch secs).
    last_persist: AtomicI64,
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

        // --- WRITER CONNECTION ---
        // Open the single writer connection and apply key + SQLCipher hardening.
        let writer = Connection::open(&db_path)?;

        // Ensure restrictive permissions on the vault file
        #[cfg(unix)]
        fs::set_permissions(&db_path, fs::Permissions::from_mode(0o600)).map_err(DbError::Io)?;

        // Apply key (encryption) + hardening pragmas. Algorithm parameters MUST be
        // applied before attempting to decrypt, as they define how the key is
        // interpreted. `query_only = false` since this is the writer.
        Self::configure_connection(&writer, password, false)
            .map_err(|_| DbError::InvalidPassword)?;

        if is_new_db {
            log_security_event(SecurityEvent::VaultCreated, Some("SQLCipher hardened"));
        }

        // Validate password with an integrity check to fail fast on incorrect key.
        if !is_new_db {
            Self::verify_key(&writer)?;
        }

        // Enable WAL mode (set once on the writer; it is persisted in the DB header).
        // Use pragma_update because WAL returns the string "wal" and execute would fail.
        writer
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(DbError::Sqlite)?;

        // --- READER POOL ---
        // Build a pool of read-only connections. Each connection re-applies the
        // SQLCipher key + hardening via `with_init`, then enables `query_only`.
        // The key is moved into a fresh SecretString so it is zeroized when the
        // pool (and its manager closure) are dropped on vault close.
        let pool_key: SecretString = SecretString::from(password.expose_secret().to_owned());
        let manager = SqliteConnectionManager::file(&db_path)
            .with_init(move |conn| Self::configure_connection(conn, &pool_key, true));
        let readers = r2d2::Pool::builder()
            .max_size(reader_pool_size())
            .build(manager)
            .map_err(|_| DbError::Pool)?;

        let now_ts = Utc::now().timestamp();
        let db = Database {
            writer: Mutex::new(writer),
            readers,
            path: db_path,
            session_timeout: SESSION_TIMEOUT_SECS, // Default 15 minutes
            last_activity: AtomicI64::new(now_ts),
            last_persist: AtomicI64::new(0),
        };

        // Run migrations (uses the writer connection).
        db.run_migrations()?;

        // Seed session_info to start a new active session.
        db.persist_session()?;

        // Verify and display security settings
        db.verify_encryption_settings()?;

        Ok(db)
    }

    /// Runs a closure with a pooled read-only connection.
    ///
    /// IMPORTANT: never call `read`/`write` again from within `f` — that would hold
    /// two pooled connections at once and can exhaust the pool under concurrency.
    /// Use `*_on(conn, …)` helpers for composition within a single block instead.
    fn read<T, F>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Connection) -> Result<T, DbError>,
    {
        let conn = self.readers.get().map_err(|_| DbError::Pool)?;
        f(&conn)
    }

    /// Runs a closure with the serialized writer connection.
    fn write<T, F>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Connection) -> Result<T, DbError>,
    {
        let conn = self.writer.lock().map_err(|_| DbError::MutexPoisoned)?;
        f(&conn)
    }

    /// Runs a closure inside an IMMEDIATE write transaction on the writer connection.
    ///
    /// Use this when a sequence of reads-then-writes must be atomic.
    pub fn with_transaction<T, F>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Connection) -> Result<T, DbError>,
    {
        let conn = self.writer.lock().map_err(|_| DbError::MutexPoisoned)?;
        conn.execute("BEGIN IMMEDIATE", [])?;
        match f(&conn) {
            Ok(value) => {
                conn.execute("COMMIT", [])?;
                Ok(value)
            }
            Err(err) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(err)
            }
        }
    }

    /// Applies the SQLCipher key and hardening PRAGMAs to a connection.
    ///
    /// IMPORTANT: The key and algorithm parameters must be applied BEFORE any read,
    /// for both new and existing DBs, as they define how the key is interpreted.
    /// When `query_only` is true the connection is restricted to reads (reader pool).
    fn configure_connection(
        conn: &Connection,
        password: &SecretString,
        query_only: bool,
    ) -> Result<(), RusqliteError> {
        // 1. Key (encryption). ExposeSecret grants controlled access; pragma_update
        //    binds the value safely (no SQL injection).
        conn.pragma_update(None, "key", password.expose_secret())?;

        // 2. SQLCipher hardening. These MUST match the values used when the DB was
        //    created, as they affect key derivation and HMAC verification.
        conn.pragma_update(None, "cipher_memory_security", true)?;
        conn.pragma_update(None, "cipher_hmac_algorithm", "HMAC_SHA512")?;
        conn.pragma_update(None, "cipher_kdf_algorithm", "PBKDF2_HMAC_SHA512")?;
        conn.pragma_update(None, "kdf_iter", KDF_ITERATIONS)?;
        conn.pragma_update(None, "cipher_page_size", 4096i64)?;

        // 3. Behavior: retry on transient locks, enforce FKs, WAL-friendly sync.
        conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        // 4. Read-only enforcement for pooled reader connections.
        if query_only {
            conn.pragma_update(None, "query_only", true)?;
        }

        Ok(())
    }

    /// Verifies current SQLCipher encryption parameters
    /// Only available in debug builds for auditing
    #[cfg(debug_assertions)]
    pub fn verify_encryption_settings(&self) -> Result<(), DbError> {
        use log::debug;

        self.read(|conn| {
            let cipher = conn
                .pragma_query_value(None, "cipher", |row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "unknown".to_string());
            let kdf = conn
                .pragma_query_value(None, "cipher_kdf_algorithm", |row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "unknown".to_string());
            let iterations = conn
                .pragma_query_value(None, "kdf_iter", |row| row.get::<_, i64>(0))
                .unwrap_or(0);

            debug!(
                "[CRYPTO] cipher={} kdf={} iterations={}",
                cipher, kdf, iterations
            );

            Ok(())
        })
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
        self.read(|conn| {
            conn.query_row("SELECT 1", [], |_| Ok(()))
                .map_err(DbError::Sqlite)?;
            Ok(())
        })
    }

    /// Forces a WAL checkpoint to write all changes to the main database file
    ///
    /// This is necessary before creating backups to ensure all pending
    /// changes in the WAL file are merged into the main database file.
    /// Uses TRUNCATE mode which writes all frames and resets the WAL file.
    pub fn checkpoint(&self) -> Result<(), DbError> {
        // Checkpoint must run on the writer connection (it writes the main DB file).
        // Use query_row instead of pragma_update because wal_checkpoint
        // is a command that returns values, not a setting pragma
        self.write(|conn| {
            conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_row| Ok(()))
                .map_err(DbError::Sqlite)?;
            Ok(())
        })
    }

    // ==================== Settings Methods ====================

    /// Gets a setting value by key
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        self.read(|conn| {
            let result = conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            );

            match result {
                Ok(value) => Ok(Some(value)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DbError::Sqlite(e)),
            }
        })
    }

    /// Sets a setting value (upsert)
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.write(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = ?2",
                params![key, value],
            )?;
            Ok(())
        })
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

    /// Updates the last activity timestamp (in memory) and persists it to disk at
    /// most once per [`SESSION_PERSIST_INTERVAL_SECS`], so reads don't trigger a write.
    pub fn touch_session(&self) -> Result<(), DbError> {
        let now_ts = Utc::now().timestamp();
        self.last_activity.store(now_ts, Ordering::Relaxed);

        // Throttle disk persistence so frequent activity doesn't write every time.
        let last = self.last_persist.load(Ordering::Relaxed);
        if now_ts - last >= SESSION_PERSIST_INTERVAL_SECS {
            self.persist_session_at(now_ts)?;
        }
        Ok(())
    }

    /// Persists the current activity timestamp to `session_info` now.
    fn persist_session(&self) -> Result<(), DbError> {
        self.persist_session_at(Utc::now().timestamp())
    }

    /// Persists the given activity timestamp to `session_info`.
    fn persist_session_at(&self, now_ts: i64) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.write(|conn| {
            conn.execute(
                "INSERT INTO session_info (id, last_activity, created_at)
                 VALUES (1, ?1, ?1)
                 ON CONFLICT(id) DO UPDATE SET last_activity = ?1",
                params![&now],
            )?;
            Ok(())
        })?;
        self.last_persist.store(now_ts, Ordering::Relaxed);
        Ok(())
    }

    /// Checks if the session has expired due to inactivity (reads in-memory state).
    pub fn check_session_timeout(&self) -> Result<(), DbError> {
        self.check_session_timeout_readonly()
    }

    /// Checks session timeout from the in-memory activity timestamp.
    pub fn check_session_timeout_readonly(&self) -> Result<(), DbError> {
        if self.session_timeout < 0 {
            return Ok(());
        }

        let last = self.last_activity.load(Ordering::Relaxed);
        let now = Utc::now().timestamp();
        if now - last > self.session_timeout {
            return Err(DbError::SessionExpired);
        }
        Ok(())
    }

    /// Gets seconds until session expires (for UI display).
    pub fn get_session_remaining(&self) -> Result<i64, DbError> {
        if self.session_timeout < 0 {
            return Ok(i64::MAX);
        }

        let last = self.last_activity.load(Ordering::Relaxed);
        let now = Utc::now().timestamp();
        Ok((self.session_timeout - (now - last)).max(0))
    }

    /// Sets the session timeout duration (in seconds)
    pub fn set_session_timeout(&mut self, timeout_secs: i64) {
        self.session_timeout = timeout_secs;
    }

    // ==================== Migrations ====================

    /// Executes pending database migrations
    fn run_migrations(&self) -> Result<(), DbError> {
        self.write(|conn| {
            let current_version = migrations::get_current_version(conn)?;

            if current_version < migrations::SCHEMA_VERSION {
                migrations::run_pending(conn, current_version, migrations::SCHEMA_VERSION)?;
            }

            Ok(())
        })
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

    /// Exercises the writer + reader-pool design end to end: many concurrent
    /// readers run alongside a continuous writer with no deadlock and no
    /// unhandled `SQLITE_BUSY`. Validates the WAL "many readers, one writer"
    /// model holds at the application level.
    #[test]
    fn test_concurrent_reads_with_writer() {
        use crate::models::Account;
        use secrecy::SecretString;
        use std::sync::Arc;
        use std::thread;

        let dir =
            std::env::temp_dir().join(format!("sanctum-concurrency-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create test dir");
        let db_path = dir.join("vault.db");
        let password = SecretString::from("concurrency-test-pw-123".to_string());

        let db = Arc::new(Database::init(db_path, &password).expect("init db"));

        // Seed an account so reads return data.
        let account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Checking".to_string(),
            account_type: "bank".to_string(),
            currency: "USD".to_string(),
            initial_balance: 1000,
            color: "#8b5cf6".to_string(),
            icon: None,
            is_archived: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        db.create_account(&account).expect("seed account");

        let mut handles = Vec::new();

        // Writer thread: continuously inserts income transactions.
        {
            let db = Arc::clone(&db);
            let account_id = account.id.clone();
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    let tx = crate::models::Transaction {
                        id: uuid::Uuid::new_v4().to_string(),
                        account_id: account_id.clone(),
                        amount: 10 + i,
                        category: "Salary".to_string(),
                        description: "concurrent".to_string(),
                        date: "2024-06-15".to_string(),
                        transaction_type: "income".to_string(),
                        transfer_account_id: None,
                    };
                    db.create_transaction(&tx).expect("concurrent write");
                }
            }));
        }

        // Reader threads: hammer reads that pull pooled connections.
        for _ in 0..8 {
            let db = Arc::clone(&db);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    db.get_accounts().expect("concurrent get_accounts");
                    db.get_balance_summary().expect("concurrent balance");
                    db.get_all_account_balances().expect("concurrent balances");
                }
            }));
        }

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        // All 50 writes landed and reads stayed consistent.
        let txs = db.get_transactions().expect("final read");
        assert_eq!(txs.len(), 50);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
