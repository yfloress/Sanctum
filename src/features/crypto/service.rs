// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  Kyronix
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

//! Crypto service module
//!
//! Core service for crypto operations: wallets, prices, and portfolio management.
//! Transaction logic is in transactions.rs, catalog logic in catalog.rs.

use crate::db::{Database, DbError};
use crate::models::{AggregatedAsset, CryptoAsset, CryptoWallet};
use crate::security_log::{SecurityEvent, log_security_event};
use chrono::Local;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::api::{
    MAX_PROXY_URL_LENGTH, ProxyConfig, default_price_allowlist, fetch_crypto_prices,
    fetch_historical_price_usd, fetch_usd_fx_rate, validate_proxy_url,
};
use super::validation::{
    MAX_ICON_LENGTH, MAX_WALLET_NAME_LENGTH, sanitize_string, validate_coin_id_str, validate_date,
    validate_field_length, validate_uuid,
};

// ==================== Settings Constants ====================

pub const SETTING_AUTO_FETCH: &str = "auto_fetch_crypto";
pub const SETTING_TICKER_COINS: &str = "ticker_coins";
pub const SETTING_CRYPTO_LAST_UPDATED: &str = "crypto_last_updated";
pub const SETTING_CRYPTO_CUSTOM_COINS: &str = "crypto_custom_coins";
pub const SETTING_CRYPTO_HIDDEN_COINS: &str = "crypto_hidden_coins";
pub const SETTING_CRYPTO_FAVORITE_COINS: &str = "crypto_favorite_coins";
pub const SETTING_CRYPTO_LAST_WALLET_ID: &str = "crypto_last_wallet_id";
pub const SETTING_CRYPTO_LAST_COIN_ID: &str = "crypto_last_coin_id";
pub const SETTING_CRYPTO_PROXY_ENABLED: &str = "crypto_proxy_enabled";
pub const SETTING_CRYPTO_PROXY_URL: &str = "crypto_proxy_url";
pub const SETTING_CRYPTO_TAX_IPC_DATA: &str = "crypto_tax_ipc_data";
pub const SETTING_CRYPTO_TAX_IPC_UPDATED: &str = "crypto_tax_ipc_updated";
pub const SETTING_CRYPTO_TAX_SETTINGS: &str = "crypto_tax_settings";
// Re-export app-level settings from core for unified access from crypto flows.
pub use crate::core::settings::{
    SETTING_DARK_MODE, SETTING_PREFERRED_CURRENCY, SETTING_PREFERRED_LANGUAGE,
    SETTING_SESSION_TIMEOUT, SETTING_SIDEBAR_COLLAPSED,
};

// ==================== Error Types ====================

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error")]
    Internal,

    #[error("No vault is currently open")]
    NoVaultOpen,

    #[error("Session expired due to inactivity. Please unlock the vault again.")]
    SessionExpired,

    #[error("API error: {0}")]
    Api(String),
}

impl From<String> for CryptoError {
    fn from(s: String) -> Self {
        CryptoError::Validation(s)
    }
}

// ==================== Crypto Service ====================

pub struct CryptoService {
    db: Arc<Mutex<Option<Database>>>,
}

impl CryptoService {
    pub fn new(db: Arc<Mutex<Option<Database>>>) -> Self {
        Self { db }
    }

    pub(crate) fn with_db<T, F>(&self, f: F) -> Result<T, CryptoError>
    where
        F: FnOnce(&Database) -> Result<T, CryptoError>,
    {
        let db_lock = self.db.lock().map_err(|_| CryptoError::Internal)?;
        let db = db_lock.as_ref().ok_or(CryptoError::NoVaultOpen)?;

        db.check_session_timeout().map_err(|e| match e {
            DbError::SessionExpired => CryptoError::SessionExpired,
            _ => CryptoError::Database(e),
        })?;

        let result = f(db)?;
        db.touch_session().map_err(CryptoError::Database)?;
        Ok(result)
    }

    // ==================== Settings ====================

    pub fn get_app_setting(&self, key: &str) -> Result<String, CryptoError> {
        self.with_db(|db| {
            let val = db.get_setting(key).map_err(CryptoError::Database)?;
            Ok(val.unwrap_or_default())
        })
    }

    pub fn set_app_setting(&self, key: &str, value: &str) -> Result<(), CryptoError> {
        self.with_db(|db| db.set_setting(key, value).map_err(CryptoError::Database))
    }

    pub fn set_proxy_enabled(&self, enabled: bool) -> Result<(), CryptoError> {
        let value = if enabled { "true" } else { "false" };
        self.set_app_setting(SETTING_CRYPTO_PROXY_ENABLED, value)
    }

    pub fn set_proxy_url(&self, url: String) -> Result<(), CryptoError> {
        let trimmed = url.trim();
        if trimmed.len() > MAX_PROXY_URL_LENGTH {
            return Err(CryptoError::Validation("Proxy URL is too long".to_string()));
        }
        self.set_app_setting(SETTING_CRYPTO_PROXY_URL, trimmed)
    }

    pub fn validate_proxy_url(&self, url: &str) -> Result<String, CryptoError> {
        validate_proxy_url(url).map_err(CryptoError::Validation)
    }

    // ==================== Price Monitoring ====================

    pub fn get_monitored_coin_ids(&self) -> Result<Vec<String>, CryptoError> {
        let mut ids = Vec::new();
        let mut seen = HashSet::new();

        for id in self.get_active_ticker_ids() {
            if seen.insert(id.clone()) {
                ids.push(id);
            }
        }

        if let Ok(portfolio) = self.get_aggregated_portfolio() {
            for asset in portfolio {
                let coin_id = asset.coin_id;
                if seen.insert(coin_id.clone()) {
                    ids.push(coin_id);
                }
            }
        }

        Ok(ids)
    }

    pub async fn get_crypto_prices(
        &self,
        coins: Vec<String>,
    ) -> Result<Vec<CryptoAsset>, CryptoError> {
        const MAX_BATCH_SIZE: usize = 50;

        let mut final_list = Vec::new();
        let mut seen = HashSet::new();
        let mut truncated = false;

        for coin in coins {
            if seen.insert(coin.clone()) {
                if final_list.len() < MAX_BATCH_SIZE {
                    final_list.push(coin);
                } else {
                    truncated = true;
                }
            }
        }

        if final_list.len() < MAX_BATCH_SIZE {
            let padding = default_price_allowlist();

            for privacy_coin in padding {
                if final_list.len() >= MAX_BATCH_SIZE {
                    break;
                }
                if seen.insert(privacy_coin.clone()) {
                    final_list.push(privacy_coin);
                }
            }
        }

        if truncated {
            log::warn!(
                "Price request exceeds {} unique coins; truncating to limit",
                MAX_BATCH_SIZE
            );
        }

        let proxy = self.get_proxy_config()?;
        fetch_crypto_prices(final_list, proxy.as_ref())
            .await
            .map_err(CryptoError::Api)
    }

    pub async fn get_usd_fx_rate(&self, target_currency: String) -> Result<f64, CryptoError> {
        let proxy = self.get_proxy_config()?;
        fetch_usd_fx_rate(&target_currency, proxy.as_ref())
            .await
            .map_err(CryptoError::Api)
    }

    pub async fn get_historical_price_usd(
        &self,
        coin_id: String,
        date: String,
    ) -> Result<f64, CryptoError> {
        let validated_coin_id = validate_coin_id_str(&coin_id)?;
        let validated_date = validate_date(&date)?;
        let proxy = self.get_proxy_config()?;
        fetch_historical_price_usd(&validated_coin_id, &validated_date, proxy.as_ref())
            .await
            .map_err(CryptoError::Api)
    }

    pub fn save_crypto_prices(&self, prices: Vec<CryptoAsset>) -> Result<(), CryptoError> {
        self.with_db(|db| {
            for price in prices {
                db.save_crypto_price(
                    &price.id,
                    &price.symbol,
                    &price.name,
                    price.current_price,
                    price.price_change_percentage_24h,
                )?;
            }
            Ok(())
        })
    }

    pub fn load_crypto_prices(&self) -> Result<Vec<CryptoAsset>, CryptoError> {
        self.with_db(|db| {
            let cached = db.load_crypto_prices()?;
            Ok(cached
                .into_iter()
                .map(|(id, symbol, name, price, change, updated)| CryptoAsset {
                    id,
                    symbol,
                    name,
                    current_price: price,
                    price_change_percentage_24h: change,
                    last_updated: updated,
                })
                .collect())
        })
    }

    fn get_proxy_config(&self) -> Result<Option<ProxyConfig>, CryptoError> {
        let enabled = self
            .get_app_setting(SETTING_CRYPTO_PROXY_ENABLED)
            .unwrap_or_default()
            == "true";

        if !enabled {
            return Ok(None);
        }

        let url = self
            .get_app_setting(SETTING_CRYPTO_PROXY_URL)
            .unwrap_or_default();
        let url = validate_proxy_url(&url).map_err(CryptoError::Validation)?;
        Ok(Some(ProxyConfig { url }))
    }

    // ==================== Portfolio Snapshots ====================

    pub fn save_crypto_portfolio_snapshot(
        &self,
        total_value: f64,
        total_cost: f64,
    ) -> Result<(), CryptoError> {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.with_db(|db| {
            db.save_crypto_portfolio_snapshot(&date, total_value, total_cost)
                .map_err(CryptoError::Database)
        })
    }

    pub fn get_crypto_portfolio_snapshots(
        &self,
        days: i64,
    ) -> Result<Vec<(String, f64, f64)>, CryptoError> {
        let days = days.max(1);
        let start_date = Local::now()
            .date_naive()
            .checked_sub_signed(chrono::Duration::days(days - 1))
            .unwrap_or_else(|| Local::now().date_naive())
            .format("%Y-%m-%d")
            .to_string();
        self.with_db(|db| {
            db.load_crypto_portfolio_snapshots(&start_date)
                .map_err(CryptoError::Database)
        })
    }

    // ==================== Wallet Management ====================

    pub fn add_wallet(
        &self,
        name: String,
        category: String,
        icon: Option<String>,
    ) -> Result<String, CryptoError> {
        self.with_db(|db| {
            let name = validate_field_length(&name, MAX_WALLET_NAME_LENGTH, "Wallet name")?;
            let name = sanitize_string(&name);

            if name.is_empty() {
                return Err(CryptoError::Validation(
                    "Wallet name cannot be empty".to_string(),
                ));
            }

            let valid_categories = ["exchange", "hardware", "software"];
            if !valid_categories.contains(&category.as_str()) {
                return Err(CryptoError::Validation(format!(
                    "Invalid category. Must be one of: {}",
                    valid_categories.join(", ")
                )));
            }

            let icon = match icon {
                Some(i) => Some(validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?),
                None => None,
            };

            let existing_wallets = db.get_wallets()?;
            if existing_wallets
                .iter()
                .any(|w| w.name.eq_ignore_ascii_case(&name))
            {
                return Err(CryptoError::Validation(format!(
                    "A wallet named '{}' already exists. Please choose a different name.",
                    name
                )));
            }

            let id = Uuid::new_v4().to_string();
            log_security_event(SecurityEvent::WalletCreated, Some(&category));

            let wallet = CryptoWallet::new(id.clone(), name, category, icon);
            db.create_wallet(&wallet)?;
            Ok(id)
        })
    }

    pub fn get_wallets(&self) -> Result<Vec<CryptoWallet>, CryptoError> {
        self.with_db(|db| db.get_wallets().map_err(CryptoError::Database))
    }

    pub fn delete_wallet(&self, id: String, force: bool) -> Result<(), CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;

            if !force {
                let transactions = db.get_wallet_transactions(&validated_id)?;
                if !transactions.is_empty() {
                    return Err(CryptoError::Validation(format!(
                        "Cannot delete wallet with {} transaction{}. Please delete all transactions first.",
                        transactions.len(),
                        if transactions.len() == 1 { "" } else { "s" }
                    )));
                }
            }

            db.delete_wallet(&validated_id, true)?;
            log_security_event(SecurityEvent::WalletDeleted, None);
            Ok(())
        })
    }

    /// Returns the number of transactions in a wallet
    pub fn get_wallet_transaction_count(&self, id: String) -> Result<usize, CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            let transactions = db.get_wallet_transactions(&validated_id)?;
            Ok(transactions.len())
        })
    }

    pub fn update_wallet_name(&self, id: String, new_name: String) -> Result<(), CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            let validated_name =
                validate_field_length(&new_name, MAX_WALLET_NAME_LENGTH, "Wallet name")?;
            let sanitized_name = sanitize_string(&validated_name);

            if sanitized_name.is_empty() {
                return Err(CryptoError::Validation(
                    "Wallet name cannot be empty".to_string(),
                ));
            }

            let existing_wallets = db.get_wallets()?;
            for wallet in existing_wallets {
                if wallet.id != validated_id && wallet.name.eq_ignore_ascii_case(&sanitized_name) {
                    return Err(CryptoError::Validation(
                        "A wallet with this name already exists".to_string(),
                    ));
                }
            }

            let mut wallet = db
                .get_wallet(&validated_id)?
                .ok_or_else(|| CryptoError::Validation("Wallet not found".to_string()))?;

            wallet.name = sanitized_name;

            db.update_wallet(&wallet)?;
            Ok(())
        })
    }

    pub fn update_wallet_icon(&self, id: String, icon: Option<String>) -> Result<(), CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;

            let mut wallet = db
                .get_wallet(&validated_id)?
                .ok_or_else(|| CryptoError::Validation("Wallet not found".to_string()))?;

            let icon = match icon {
                Some(i) => Some(validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?),
                None => None,
            };
            wallet.icon = icon.filter(|value| !value.is_empty());

            db.update_wallet(&wallet)?;
            Ok(())
        })
    }

    // ==================== Portfolio ====================

    pub fn get_aggregated_portfolio(&self) -> Result<Vec<AggregatedAsset>, CryptoError> {
        self.with_db(|db| db.get_aggregated_portfolio().map_err(CryptoError::Database))
    }

    pub fn get_wallet_holdings(
        &self,
        wallet_id: String,
    ) -> Result<Vec<AggregatedAsset>, CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&wallet_id)?;
            db.get_wallet_aggregated_holdings(&validated_id)
                .map_err(CryptoError::Database)
        })
    }

    pub fn get_available_balance(
        &self,
        wallet_id: String,
        coin_id: String,
        date: String,
    ) -> Result<f64, CryptoError> {
        self.with_db(|db| {
            let validated_wallet_id = validate_uuid(&wallet_id)?;
            let validated_coin_id = validate_coin_id_str(&coin_id)?;
            let validated_date = validate_date(&date)?;

            db.get_wallet_coin_balance_at(
                &validated_wallet_id,
                &validated_coin_id,
                &validated_date,
                None,
            )
            .map_err(CryptoError::Database)
        })
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use crate::features::crypto::api::{
        sanitize_api_string, validate_coin_id, validate_percentage, validate_price,
    };

    #[test]
    fn test_validate_coin_id_valid() {
        assert!(validate_coin_id("bitcoin").is_ok());
        assert!(validate_coin_id("ethereum").is_ok());
        assert!(validate_coin_id("binance-coin").is_ok());
        assert!(validate_coin_id("BITCOIN").is_ok());
        assert!(validate_coin_id("shiba-inu").is_ok());
    }

    #[test]
    fn test_validate_coin_id_empty() {
        assert!(validate_coin_id("").is_err());
        assert!(validate_coin_id("   ").is_err());
    }

    #[test]
    fn test_validate_coin_id_invalid_chars() {
        assert!(validate_coin_id("bitcoin<script>").is_err());
        assert!(validate_coin_id("../etc/passwd").is_err());
        assert!(validate_coin_id("coin;DROP TABLE").is_err());
        assert!(validate_coin_id("coin\n").is_err());
        assert!(validate_coin_id("coin&param=value").is_err());
        assert!(validate_coin_id("coin?query").is_err());
    }

    #[test]
    fn test_validate_coin_id_boundary_cases() {
        assert!(validate_coin_id("-bitcoin").is_err());
        assert!(validate_coin_id("bitcoin-").is_err());
        assert!(validate_coin_id("bit--coin").is_err());
    }

    #[test]
    fn test_validate_coin_id_too_long() {
        let long_id = "a".repeat(65);
        assert!(validate_coin_id(&long_id).is_err());

        let max_id = "a".repeat(64);
        assert!(validate_coin_id(&max_id).is_ok());
    }

    #[test]
    fn test_sanitize_api_string() {
        assert_eq!(sanitize_api_string("Bitcoin"), "Bitcoin");
        assert_eq!(sanitize_api_string("Bitcoin <script>"), "Bitcoin script");
        assert_eq!(sanitize_api_string("Test\n\r\t"), "Test");
        assert_eq!(sanitize_api_string("Normal-Name.v2"), "Normal-Name.v2");
    }

    #[test]
    fn test_sanitize_api_string_length() {
        let long_string = "a".repeat(500);
        let sanitized = sanitize_api_string(&long_string);
        assert_eq!(sanitized.len(), 128);
    }

    #[test]
    fn test_validate_price() {
        assert_eq!(validate_price(100.0), 100.0);
        assert_eq!(validate_price(-50.0), 0.0);
        assert_eq!(validate_price(f64::NAN), 0.0);
        assert_eq!(validate_price(f64::INFINITY), 0.0);
        assert_eq!(validate_price(1e20), 1e15);
    }

    #[test]
    fn test_validate_percentage() {
        assert_eq!(validate_percentage(5.5), 5.5);
        assert_eq!(validate_percentage(-50.0), -50.0);
        assert_eq!(validate_percentage(-150.0), -100.0);
        assert_eq!(validate_percentage(50000.0), 10000.0);
        assert_eq!(validate_percentage(f64::NAN), 0.0);
    }
}
