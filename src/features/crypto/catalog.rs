// Sanctum — a privacy-first personal finance, crypto, and habits vault.
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

//! Coin catalog management
//!
//! Handles custom coins, hidden coins, favorites, and ticker configuration.

use crate::models::CryptoCatalogCoin;
use std::collections::HashSet;

use super::api::default_coin_catalog;
use super::service::{
    CryptoError, CryptoService, SETTING_CRYPTO_CUSTOM_COINS, SETTING_CRYPTO_FAVORITE_COINS,
    SETTING_CRYPTO_HIDDEN_COINS, SETTING_TICKER_COINS,
};
use super::validation::{
    MAX_COIN_NAME_LENGTH, sanitize_string, validate_coin_id_str, validate_field_length,
    validate_symbol,
};

impl CryptoService {
    // ==================== Ticker Configuration ====================

    pub fn get_active_ticker_ids(&self) -> Vec<String> {
        self.get_app_setting(SETTING_TICKER_COINS)
            .ok()
            .filter(|val| !val.is_empty())
            .and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
            .unwrap_or_else(super::api::default_ticker_ids)
    }

    pub fn save_active_ticker_ids(&self, ids: Vec<String>) -> Result<(), CryptoError> {
        let json =
            serde_json::to_string(&ids).map_err(|e| CryptoError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_TICKER_COINS, &json)
    }

    // ==================== Coin Catalog ====================

    pub fn get_custom_coin_catalog(&self) -> Result<Vec<CryptoCatalogCoin>, CryptoError> {
        let raw = self.get_app_setting(SETTING_CRYPTO_CUSTOM_COINS)?;
        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut coins: Vec<CryptoCatalogCoin> =
            serde_json::from_str(&raw).map_err(|e| CryptoError::Validation(e.to_string()))?;
        for coin in &mut coins {
            coin.custom = true;
        }
        Ok(coins)
    }

    pub fn get_hidden_coin_ids(&self) -> Vec<String> {
        self.get_app_setting(SETTING_CRYPTO_HIDDEN_COINS)
            .ok()
            .filter(|val| !val.is_empty())
            .and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
            .unwrap_or_default()
    }

    pub fn get_favorite_coin_ids(&self) -> Vec<String> {
        self.get_app_setting(SETTING_CRYPTO_FAVORITE_COINS)
            .ok()
            .filter(|val| !val.is_empty())
            .and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
            .unwrap_or_default()
    }

    pub(crate) fn save_custom_coin_catalog(
        &self,
        coins: Vec<CryptoCatalogCoin>,
    ) -> Result<(), CryptoError> {
        let json =
            serde_json::to_string(&coins).map_err(|e| CryptoError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_CRYPTO_CUSTOM_COINS, &json)
    }

    pub(crate) fn save_hidden_coin_ids(&self, ids: Vec<String>) -> Result<(), CryptoError> {
        let json =
            serde_json::to_string(&ids).map_err(|e| CryptoError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_CRYPTO_HIDDEN_COINS, &json)
    }

    pub(crate) fn save_favorite_coin_ids(&self, ids: Vec<String>) -> Result<(), CryptoError> {
        let json =
            serde_json::to_string(&ids).map_err(|e| CryptoError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_CRYPTO_FAVORITE_COINS, &json)
    }

    pub fn set_favorite_coin(&self, id: String, favorite: bool) -> Result<(), CryptoError> {
        let id = validate_coin_id_str(&id)?;
        let mut favorites = self.get_favorite_coin_ids();
        let had_id = favorites.iter().any(|coin| coin == &id);

        if favorite && !had_id {
            favorites.push(id);
            favorites.sort();
            favorites.dedup();
            self.save_favorite_coin_ids(favorites)?;
        } else if !favorite && had_id {
            favorites.retain(|coin| coin != &id);
            self.save_favorite_coin_ids(favorites)?;
        }

        Ok(())
    }

    pub fn get_coin_catalog(&self) -> Result<Vec<CryptoCatalogCoin>, CryptoError> {
        let mut catalog = default_coin_catalog();
        let custom = self.get_custom_coin_catalog()?;
        let mut ids: HashSet<String> = catalog.iter().map(|c| c.id.clone()).collect();

        for coin in custom {
            if ids.insert(coin.id.clone()) {
                catalog.push(coin);
            }
        }

        let hidden = self.get_hidden_coin_ids();
        if !hidden.is_empty() {
            let hidden: HashSet<String> = hidden.into_iter().collect();
            catalog.retain(|coin| !hidden.contains(&coin.id));
        }

        Ok(catalog)
    }

    pub fn add_custom_coin(
        &self,
        id: String,
        name: String,
        symbol: String,
    ) -> Result<(), CryptoError> {
        let id = validate_coin_id_str(&id)?;
        let symbol = validate_symbol(&symbol)?;
        let name = validate_field_length(&name, MAX_COIN_NAME_LENGTH, "Coin name")?;
        let name = sanitize_string(&name);

        if name.is_empty() {
            return Err(CryptoError::Validation(
                "Coin name cannot be empty".to_string(),
            ));
        }

        let mut custom = self.get_custom_coin_catalog()?;

        if custom.iter().any(|coin| coin.id == id)
            || default_coin_catalog().iter().any(|coin| coin.id == id)
        {
            return Err(CryptoError::Validation(
                "Coin ID already exists".to_string(),
            ));
        }

        custom.push(CryptoCatalogCoin {
            id,
            name,
            symbol,
            custom: true,
        });

        self.save_custom_coin_catalog(custom)
    }

    pub fn delete_custom_coin(&self, id: String) -> Result<(), CryptoError> {
        let id = validate_coin_id_str(&id)?;
        let mut custom = self.get_custom_coin_catalog()?;
        let before = custom.len();
        custom.retain(|coin| coin.id != id);
        let removed_custom = custom.len() != before;

        if removed_custom {
            self.save_custom_coin_catalog(custom)?;
        }

        let is_default = default_coin_catalog().iter().any(|coin| coin.id == id);
        let mut hidden_updated = false;
        if is_default {
            let mut hidden = self.get_hidden_coin_ids();
            if !hidden.iter().any(|coin| coin == &id) {
                hidden.push(id.clone());
                hidden.sort();
                hidden.dedup();
                self.save_hidden_coin_ids(hidden)?;
                hidden_updated = true;
            }
        }

        if !removed_custom && !hidden_updated {
            return Err(CryptoError::Validation("Coin not found".to_string()));
        }

        let mut active = self.get_active_ticker_ids();
        if active.iter().any(|coin| coin == &id) {
            active.retain(|coin| coin != &id);
            let _ = self.save_active_ticker_ids(active);
        }

        let mut favorites = self.get_favorite_coin_ids();
        if favorites.iter().any(|coin| coin == &id) {
            favorites.retain(|coin| coin != &id);
            let _ = self.save_favorite_coin_ids(favorites);
        }

        Ok(())
    }
}
