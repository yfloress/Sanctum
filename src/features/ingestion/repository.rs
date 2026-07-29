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

//! Ingestion repository
//!
//! Database operations for entity resolution and lookups during import.

use crate::db::{Database, DbError};
use crate::models::{
    Account, CryptoCatalogCoin, CryptoTransaction, CryptoWallet, Transaction, TransactionCategory,
};
use std::collections::HashMap;

fn coin_lookup_keys(coin: &CryptoCatalogCoin) -> Vec<String> {
    let symbol = coin.symbol.trim().to_lowercase();
    let id = coin.id.trim().to_lowercase();
    let name = coin.name.trim().to_lowercase();

    let compact = |value: &str| -> String {
        value
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect()
    };

    let mut keys = vec![symbol, id.clone(), name];
    let compact_id = compact(&id);
    if !compact_id.is_empty() {
        keys.push(compact_id);
    }

    if id == "tether" {
        keys.push("usdt".to_string());
        keys.push("tether".to_string());
    }
    if id == "usd-coin" {
        keys.push("usdc".to_string());
        keys.push("usdcoin".to_string());
    }
    if id == "mx-token" {
        keys.push("mx".to_string());
        keys.push("mxtoken".to_string());
    }

    keys.sort();
    keys.dedup();
    keys
}

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

    /// Gets all existing transactions for deduplication
    pub fn get_all_transactions(db: &Database) -> Result<Vec<Transaction>, DbError> {
        db.get_transactions()
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

        let mut lookup = HashMap::new();
        for coin in coins {
            for key in coin_lookup_keys(&coin) {
                lookup.entry(key).or_insert_with(|| coin.clone());
            }
        }
        Ok(lookup)
    }

    /// Gets all existing crypto transactions for deduplication
    pub fn get_all_crypto_transactions(db: &Database) -> Result<Vec<CryptoTransaction>, DbError> {
        db.get_all_crypto_transactions(0, i64::MAX)
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

#[cfg(test)]
mod tests {
    use super::coin_lookup_keys;
    use crate::models::CryptoCatalogCoin;

    fn coin(id: &str, name: &str, symbol: &str) -> CryptoCatalogCoin {
        CryptoCatalogCoin {
            id: id.to_string(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            custom: false,
        }
    }

    #[test]
    fn coin_lookup_keys_include_stablecoin_aliases() {
        let keys = coin_lookup_keys(&coin("tether", "Tether", "USDT"));
        assert!(keys.contains(&"usdt".to_string()));
        assert!(keys.contains(&"tether".to_string()));
    }

    #[test]
    fn coin_lookup_keys_include_mx_aliases() {
        let keys = coin_lookup_keys(&coin("mx-token", "MX Token", "MX"));
        assert!(keys.contains(&"mx".to_string()));
        assert!(keys.contains(&"mx-token".to_string()));
        assert!(keys.contains(&"mxtoken".to_string()));
    }
}
