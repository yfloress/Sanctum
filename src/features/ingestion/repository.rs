//! Ingestion repository
//!
//! Database operations for entity resolution and lookups during import.

use crate::db::{Database, DbError};
use crate::models::{
    Account, CryptoCatalogCoin, CryptoTransaction, CryptoWallet, Habit, Transaction,
    TransactionCategory,
};
use std::collections::HashMap;

/// Repository for ingestion-related database lookups
pub struct IngestionRepository;

impl IngestionRepository {
    /// Builds a lookup map for accounts (name_lowercase -> Account)
    pub fn build_account_lookup(db: &Database) -> Result<HashMap<String, Account>, DbError> {
        let accounts = db.get_accounts()?;
        Ok(accounts
            .into_iter()
            .map(|a| (a.name.trim().to_lowercase(), a))
            .collect())
    }

    /// Builds a lookup map for categories ((name_lowercase, type) -> Category)
    pub fn build_category_lookup(
        db: &Database,
    ) -> Result<HashMap<(String, String), TransactionCategory>, DbError> {
        let mut map = HashMap::new();
        for cat_type in ["expense", "income"] {
            let categories = db.get_transaction_categories(cat_type)?;
            for cat in categories {
                map.insert((cat.name.trim().to_lowercase(), cat_type.to_string()), cat);
            }
        }
        Ok(map)
    }

    /// Builds a lookup map for habits (name_lowercase -> Habit)
    pub fn build_habit_lookup(db: &Database) -> Result<HashMap<String, Habit>, DbError> {
        let habits = db.get_habits()?;
        Ok(habits
            .into_iter()
            .map(|h| (h.name.trim().to_lowercase(), h))
            .collect())
    }

    /// Gets all existing transactions for deduplication
    pub fn get_all_transactions(db: &Database) -> Result<Vec<Transaction>, DbError> {
        db.get_transactions()
    }

    /// Checks if a habit log already exists for habit+date
    pub fn habit_log_exists(db: &Database, habit_id: &str, date: &str) -> Result<bool, DbError> {
        db.habit_log_exists(habit_id, date)
    }

    /// Creates a transaction
    pub fn create_transaction(db: &Database, transaction: &Transaction) -> Result<(), DbError> {
        db.create_transaction(transaction)
    }

    /// Creates a transfer (atomic operation creating linked transactions)
    pub fn create_transfer(
        db: &Database,
        from_id: &str,
        to_id: &str,
        amount: i64,
        description: &str,
        date: &str,
    ) -> Result<String, DbError> {
        db.create_transfer(from_id, to_id, amount, description, date)
    }

    /// Creates a habit log
    pub fn create_habit_log(db: &Database, log: &crate::models::HabitLog) -> Result<(), DbError> {
        db.create_habit_log(log)
    }

    // ==================== Crypto Operations ====================

    /// Builds a lookup map for crypto wallets (name_lowercase -> CryptoWallet)
    pub fn build_wallet_lookup(db: &Database) -> Result<HashMap<String, CryptoWallet>, DbError> {
        let wallets = db.get_wallets()?;
        Ok(wallets
            .into_iter()
            .map(|w| (w.name.trim().to_lowercase(), w))
            .collect())
    }

    /// Builds a lookup map for crypto coins (symbol_lowercase -> CryptoCatalogCoin)
    pub fn build_coin_lookup(db: &Database) -> Result<HashMap<String, CryptoCatalogCoin>, DbError> {
        // Get default coins
        let mut coins = crate::features::crypto::api::default_coin_catalog();

        // Add custom coins from settings
        if let Ok(Some(raw)) =
            db.get_setting(crate::features::crypto::service::SETTING_CRYPTO_CUSTOM_COINS)
            && !raw.trim().is_empty()
            && let Ok(custom_coins) = serde_json::from_str::<Vec<CryptoCatalogCoin>>(&raw)
        {
            for mut coin in custom_coins {
                coin.custom = true;
                // Avoid duplicates if a custom coin has the same ID as a default one
                if !coins.iter().any(|c| c.id == coin.id) {
                    coins.push(coin);
                }
            }
        }

        Ok(coins
            .into_iter()
            .map(|c| (c.symbol.trim().to_lowercase(), c))
            .collect())
    }

    /// Gets all existing crypto transactions for deduplication
    pub fn get_all_crypto_transactions(db: &Database) -> Result<Vec<CryptoTransaction>, DbError> {
        db.get_all_crypto_transactions()
    }

    /// Creates a crypto transaction
    pub fn create_crypto_transaction(
        db: &Database,
        transaction: &CryptoTransaction,
    ) -> Result<(), DbError> {
        db.create_crypto_transaction(transaction)
    }

    /// Gets the balance of a coin in a wallet at a specific date
    pub fn get_wallet_coin_balance(
        db: &Database,
        wallet_id: &str,
        coin_id: &str,
        date: &str,
    ) -> Result<f64, DbError> {
        db.get_wallet_coin_balance_at(wallet_id, coin_id, date, None)
    }
}
