//! Crypto repository
//!
//! Database operations for crypto feature.
//! Currently delegates to the main Database struct for backwards compatibility.

use crate::db::{Database, DbError};
// Use original models for compatibility with db.rs
use crate::models::{AggregatedAsset, CryptoTransaction, CryptoWallet};

/// Type alias for crypto price cache entries: (coin_id, symbol, name, price, change_24h, updated_at)
pub type CryptoPriceEntry = (String, String, String, f64, f64, String);

/// Repository for crypto-related database operations
pub struct CryptoRepository;

impl CryptoRepository {
    // Wallet operations
    pub fn create_wallet(db: &Database, wallet: &CryptoWallet) -> Result<(), DbError> {
        db.create_wallet(wallet)
    }

    pub fn get_wallets(db: &Database) -> Result<Vec<CryptoWallet>, DbError> {
        db.get_wallets()
    }

    pub fn get_wallet(db: &Database, id: &str) -> Result<Option<CryptoWallet>, DbError> {
        db.get_wallet(id)
    }

    pub fn update_wallet(db: &Database, wallet: &CryptoWallet) -> Result<(), DbError> {
        db.update_wallet(wallet)
    }

    pub fn delete_wallet(db: &Database, id: &str) -> Result<(), DbError> {
        db.delete_wallet(id)
    }

    // Transaction operations
    pub fn create_crypto_transaction(db: &Database, tx: &CryptoTransaction) -> Result<(), DbError> {
        db.create_crypto_transaction(tx)
    }

    pub fn get_wallet_transactions(db: &Database, wallet_id: &str) -> Result<Vec<CryptoTransaction>, DbError> {
        db.get_wallet_transactions(wallet_id)
    }

    pub fn get_all_crypto_transactions(db: &Database) -> Result<Vec<CryptoTransaction>, DbError> {
        db.get_all_crypto_transactions()
    }

    pub fn get_crypto_transactions_by_coin(db: &Database, coin_id: &str) -> Result<Vec<CryptoTransaction>, DbError> {
        db.get_crypto_transactions_by_coin(coin_id)
    }

    pub fn delete_crypto_transaction(db: &Database, id: &str) -> Result<(), DbError> {
        db.delete_crypto_transaction(id)
    }

    pub fn get_crypto_transaction(db: &Database, id: &str) -> Result<Option<CryptoTransaction>, DbError> {
        db.get_crypto_transaction(id)
    }

    // Portfolio operations
    pub fn get_aggregated_portfolio(db: &Database) -> Result<Vec<AggregatedAsset>, DbError> {
        db.get_aggregated_portfolio()
    }

    pub fn get_wallet_aggregated_holdings(db: &Database, wallet_id: &str) -> Result<Vec<AggregatedAsset>, DbError> {
        db.get_wallet_aggregated_holdings(wallet_id)
    }

    // Price cache operations
    pub fn save_crypto_price(
        db: &Database,
        coin_id: &str,
        symbol: &str,
        name: &str,
        price: f64,
        change_24h: f64,
    ) -> Result<(), DbError> {
        db.save_crypto_price(coin_id, symbol, name, price, change_24h)
    }

    /// Returns Vec of (coin_id, symbol, name, price, change_24h, updated_at)
    pub fn load_crypto_prices(db: &Database) -> Result<Vec<CryptoPriceEntry>, DbError> {
        db.load_crypto_prices()
    }

    pub fn load_crypto_price(db: &Database, coin_id: &str) -> Result<Option<(f64, String)>, DbError> {
        db.load_crypto_price(coin_id)
    }

    // Portfolio snapshots
    pub fn save_crypto_portfolio_snapshot(
        db: &Database,
        snapshot_date: &str,
        total_value: f64,
        total_pnl: f64,
    ) -> Result<(), DbError> {
        db.save_crypto_portfolio_snapshot(snapshot_date, total_value, total_pnl)
    }

    pub fn load_crypto_portfolio_snapshots(db: &Database, start_date: &str) -> Result<Vec<(String, f64, f64)>, DbError> {
        db.load_crypto_portfolio_snapshots(start_date)
    }

    // Settings
    pub fn get_setting(db: &Database, key: &str) -> Result<Option<String>, DbError> {
        db.get_setting(key)
    }

    pub fn set_setting(db: &Database, key: &str, value: &str) -> Result<(), DbError> {
        db.set_setting(key, value)
    }
}
