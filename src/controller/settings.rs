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

//! Settings-related controller methods
//!
//! Application settings and coin catalog management.

use super::{AppController, ControllerError};
use crate::features::crypto::default_coin_catalog;
use crate::models::CryptoCatalogCoin;

impl AppController {
    // ==================== Settings Methods ====================

    /// Gets an application setting
    pub fn get_app_setting(&self, key: &str) -> Result<String, ControllerError> {
        self.crypto_service
            .get_app_setting(key)
            .map_err(ControllerError::from)
    }

    /// Sets an application setting
    pub fn set_app_setting(&self, key: &str, value: &str) -> Result<(), ControllerError> {
        self.crypto_service
            .set_app_setting(key, value)
            .map_err(ControllerError::from)
    }

    /// Enables or disables proxy usage for crypto API calls
    pub fn set_crypto_proxy_enabled(&self, enabled: bool) -> Result<(), ControllerError> {
        self.crypto_service
            .set_proxy_enabled(enabled)
            .map_err(ControllerError::from)
    }

    /// Stores proxy URL for crypto API calls (empty clears)
    pub fn set_crypto_proxy_url(&self, url: String) -> Result<(), ControllerError> {
        self.crypto_service
            .set_proxy_url(url)
            .map_err(ControllerError::from)
    }

    /// Validates proxy URL for crypto API calls
    pub fn validate_crypto_proxy_url(&self, url: String) -> Result<String, ControllerError> {
        self.crypto_service
            .validate_proxy_url(&url)
            .map_err(ControllerError::from)
    }

    /// Gets active ticker IDs from settings or default
    pub fn get_active_ticker_ids(&self) -> Vec<String> {
        self.crypto_service.get_active_ticker_ids()
    }

    /// Saves active ticker IDs to settings
    pub fn save_active_ticker_ids(&self, ids: Vec<String>) -> Result<(), ControllerError> {
        self.crypto_service
            .save_active_ticker_ids(ids)
            .map_err(ControllerError::from)
    }

    // ==================== Coin Catalog Methods ====================

    /// Loads custom coins configured by the user
    pub fn get_custom_coin_catalog(&self) -> Result<Vec<CryptoCatalogCoin>, ControllerError> {
        self.crypto_service
            .get_custom_coin_catalog()
            .map_err(ControllerError::from)
    }

    /// Loads hidden coin IDs for the catalog UI
    pub fn get_hidden_coin_ids(&self) -> Vec<String> {
        self.crypto_service.get_hidden_coin_ids()
    }

    /// Loads favorite coin IDs for the catalog UI
    pub fn get_favorite_coin_ids(&self) -> Vec<String> {
        self.crypto_service.get_favorite_coin_ids()
    }

    /// Marks or unmarks a coin as favorite
    pub fn set_favorite_coin(&self, id: String, favorite: bool) -> Result<(), ControllerError> {
        self.crypto_service
            .set_favorite_coin(id, favorite)
            .map_err(ControllerError::from)
    }

    /// Returns the full coin catalog (defaults + custom)
    pub fn get_coin_catalog(&self) -> Result<Vec<CryptoCatalogCoin>, ControllerError> {
        self.crypto_service
            .get_coin_catalog()
            .map_err(ControllerError::from)
    }

    /// Returns coin catalog, falling back to defaults on error
    /// Use this in UI to avoid needing to import features module
    pub fn get_coin_catalog_or_default(&self) -> Vec<CryptoCatalogCoin> {
        self.crypto_service
            .get_coin_catalog()
            .unwrap_or_else(|_| default_coin_catalog())
    }

    /// Adds a custom coin to the catalog
    pub fn add_custom_coin(
        &self,
        id: String,
        name: String,
        symbol: String,
    ) -> Result<(), ControllerError> {
        self.crypto_service
            .add_custom_coin(id, name, symbol)
            .map_err(ControllerError::from)
    }

    /// Deletes a custom coin from the catalog
    pub fn delete_custom_coin(&self, id: String) -> Result<(), ControllerError> {
        self.crypto_service
            .delete_custom_coin(id)
            .map_err(ControllerError::from)
    }
}
