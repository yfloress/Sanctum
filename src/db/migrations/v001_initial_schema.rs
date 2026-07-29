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

//! Initial database schema (v1)
//!
//! Creates all tables for the Sanctum application:
//! - Finance: accounts, transactions, transaction_categories
//! - Crypto: wallets, transactions, price cache, portfolio snapshots
//! - System: settings, auth_attempts, session_info

use crate::db::DbError;
use chrono::Utc;
use rusqlite::{Connection, params};

pub fn up(conn: &Connection) -> Result<(), DbError> {
    // === FIAT Finance System ===
    create_accounts_table(conn)?;
    create_transactions_table(conn)?;
    create_transaction_categories_table(conn)?;
    initialize_default_categories(conn)?;

    // === Crypto Ledger System ===
    create_crypto_tables(conn)?;
    create_price_cache_tables(conn)?;

    // === System Tables ===
    create_settings_table(conn)?;
    create_security_tables(conn)?;

    Ok(())
}

// ==================== Finance Tables ====================

fn create_accounts_table(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_accounts_archived ON accounts(is_archived)",
        [],
    )?;

    Ok(())
}

fn create_transactions_table(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(type)",
        [],
    )?;

    Ok(())
}

fn create_transaction_categories_table(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transaction_categories_type ON transaction_categories(category_type, sort_order)",
        [],
    )?;

    Ok(())
}

fn initialize_default_categories(conn: &Connection) -> Result<(), DbError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM transaction_categories", [], |row| {
        row.get(0)
    })?;

    if count > 0 {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();

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
        conn.execute(
            "INSERT INTO transaction_categories (id, name, category_type, sort_order, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, "expense", idx as i32, 1, now],
        )?;
    }

    let income_categories = ["SALARY", "FREELANCE", "INVESTMENT", "GIFT", "OTHER"];

    for (idx, name) in income_categories.iter().enumerate() {
        let id = format!("inc_{}", uuid::Uuid::new_v4());
        conn.execute(
            "INSERT INTO transaction_categories (id, name, category_type, sort_order, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, "income", idx as i32, 1, now],
        )?;
    }

    Ok(())
}

// ==================== Crypto Tables ====================

fn create_crypto_tables(conn: &Connection) -> Result<(), DbError> {
    // Wallets
    conn.execute(
        "CREATE TABLE IF NOT EXISTS crypto_wallets (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            icon TEXT
        )",
        [],
    )?;

    // Transactions
    conn.execute(
        "CREATE TABLE IF NOT EXISTS crypto_transactions (
            id TEXT PRIMARY KEY NOT NULL,
            wallet_id TEXT NOT NULL,
            coin_id TEXT NOT NULL,
            symbol TEXT NOT NULL,
            type TEXT NOT NULL,
            amount REAL NOT NULL,
            subtype TEXT,
            price_per_coin REAL,
            fee REAL,
            fee_coin_id TEXT,
            fee_amount REAL,
            override_proceeds REAL,
            override_cost_basis REAL,
            date TEXT NOT NULL,
            notes TEXT,
            related_tx_id TEXT,
            FOREIGN KEY (wallet_id) REFERENCES crypto_wallets(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_crypto_wallets_category ON crypto_wallets(category)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_crypto_tx_wallet ON crypto_transactions(wallet_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_crypto_tx_coin ON crypto_transactions(coin_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_crypto_tx_date ON crypto_transactions(date)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_crypto_tx_type ON crypto_transactions(type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_crypto_tx_related ON crypto_transactions(related_tx_id)",
        [],
    )?;

    Ok(())
}

fn create_price_cache_tables(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS exchange_rate_cache (
            currency_pair TEXT PRIMARY KEY NOT NULL,
            rate REAL NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
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

    conn.execute(
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

// ==================== System Tables ====================

fn create_settings_table(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

fn create_security_tables(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS auth_attempts (
            vault_path TEXT PRIMARY KEY NOT NULL,
            failed_count INTEGER NOT NULL DEFAULT 0,
            locked_until TEXT,
            last_attempt TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_info (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_activity TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    // Initialize session info
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO session_info (id, last_activity, created_at) VALUES (1, ?1, ?1)",
        params![&now],
    )?;

    Ok(())
}
