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

//! Crypto-related controller methods
//!
//! Wallets, transactions, prices, and portfolio aggregation.

use super::{AppController, ControllerError};
use crate::features::crypto::{
    IpcImportSummary, IpcSummary, TaxPeriodSettings, TaxReport, TaxSummaryPayload,
};
use crate::models::{AggregatedAsset, CryptoAsset, CryptoTransaction, CryptoWallet};

impl AppController {
    // ==================== Crypto Price Methods ====================

    /// Gets all unique coin IDs that need monitoring (Active Tickers + Wallet Holdings)
    pub fn get_monitored_coin_ids(&self) -> Result<Vec<String>, ControllerError> {
        self.crypto_service
            .get_monitored_coin_ids()
            .map_err(ControllerError::from)
    }

    /// Fetches cryptocurrency prices from CoinGecko
    /// Implements privacy padding: mixes requested coins with a default list up to the API limit (50).
    pub async fn get_crypto_prices(
        &self,
        coins: Vec<String>,
    ) -> Result<Vec<CryptoAsset>, ControllerError> {
        self.crypto_service
            .get_crypto_prices(coins)
            .await
            .map_err(ControllerError::from)
    }

    /// Fetches USD to target fiat exchange rate (e.g. USD/EUR).
    pub async fn get_usd_fx_rate(&self, target_currency: String) -> Result<f64, ControllerError> {
        self.crypto_service
            .get_usd_fx_rate(target_currency)
            .await
            .map_err(ControllerError::from)
    }

    /// Fetches historical USD price for a coin on a specific day.
    pub async fn get_crypto_historical_price_usd(
        &self,
        coin_id: String,
        date: String,
    ) -> Result<f64, ControllerError> {
        self.crypto_service
            .get_historical_price_usd(coin_id, date)
            .await
            .map_err(ControllerError::from)
    }

    /// Saves crypto prices to cache
    pub fn save_crypto_prices(&self, prices: Vec<CryptoAsset>) -> Result<(), ControllerError> {
        self.crypto_service
            .save_crypto_prices(prices)
            .map_err(ControllerError::from)
    }

    /// Loads cached crypto prices
    pub fn load_crypto_prices(&self) -> Result<Vec<CryptoAsset>, ControllerError> {
        self.crypto_service
            .load_crypto_prices()
            .map_err(ControllerError::from)
    }

    /// Saves a daily portfolio snapshot (upsert by date)
    pub fn save_crypto_portfolio_snapshot(
        &self,
        total_value: f64,
        total_cost: f64,
    ) -> Result<(), ControllerError> {
        self.crypto_service
            .save_crypto_portfolio_snapshot(total_value, total_cost)
            .map_err(ControllerError::from)
    }

    /// Loads portfolio snapshots for the last N days (inclusive)
    pub fn get_crypto_portfolio_snapshots(
        &self,
        days: i64,
    ) -> Result<Vec<(String, f64, f64)>, ControllerError> {
        self.crypto_service
            .get_crypto_portfolio_snapshots(days)
            .map_err(ControllerError::from)
    }

    // ==================== Crypto Tax (IPC) ====================

    pub fn import_ipc_csv(&self, content: &str) -> Result<IpcImportSummary, ControllerError> {
        self.crypto_service
            .import_ipc_csv(content)
            .map_err(ControllerError::from)
    }

    pub fn get_ipc_summary(&self) -> Result<Option<IpcSummary>, ControllerError> {
        self.crypto_service
            .get_ipc_summary()
            .map_err(ControllerError::from)
    }

    pub fn load_tax_settings(
        &self,
        period_id: String,
    ) -> Result<TaxPeriodSettings, ControllerError> {
        self.crypto_service
            .load_tax_settings(period_id)
            .map_err(ControllerError::from)
    }

    pub fn save_tax_settings(&self, settings: TaxPeriodSettings) -> Result<(), ControllerError> {
        self.crypto_service
            .save_tax_settings(settings)
            .map_err(ControllerError::from)
    }

    pub fn generate_tax_report(&self, period_id: String) -> Result<TaxReport, ControllerError> {
        self.crypto_service
            .generate_tax_report(period_id)
            .map_err(ControllerError::from)
    }

    pub fn generate_tax_summary(
        &self,
        period_id: String,
    ) -> Result<TaxSummaryPayload, ControllerError> {
        self.crypto_service
            .generate_tax_summary(period_id)
            .map_err(ControllerError::from)
    }

    pub fn fill_missing_tax_price_fields(
        &self,
        tx_id: String,
        price_per_coin: Option<f64>,
        fee_usd: Option<f64>,
        override_proceeds: Option<f64>,
    ) -> Result<bool, ControllerError> {
        self.crypto_service
            .fill_missing_tax_price_fields(tx_id, price_per_coin, fee_usd, override_proceeds)
            .map_err(ControllerError::from)
    }

    pub fn export_tax_report_csv(
        &self,
        period_id: String,
        path: String,
    ) -> Result<(), ControllerError> {
        self.crypto_service
            .export_tax_report_csv(period_id, &path)
            .map_err(ControllerError::from)
    }

    pub fn export_tax_history_csv(
        &self,
        period_id: String,
        path: String,
    ) -> Result<(), ControllerError> {
        self.crypto_service
            .export_tax_history_csv(period_id, &path)
            .map_err(ControllerError::from)
    }

    // ==================== Crypto Wallet Methods ====================

    /// Creates a new crypto wallet
    pub fn add_wallet(
        &self,
        name: String,
        category: String,
        icon: Option<String>,
    ) -> Result<String, ControllerError> {
        self.crypto_service
            .add_wallet(name, category, icon)
            .map_err(ControllerError::from)
    }

    /// Gets all wallets
    pub fn get_wallets(&self) -> Result<Vec<CryptoWallet>, ControllerError> {
        self.crypto_service
            .get_wallets()
            .map_err(ControllerError::from)
    }

    /// Deletes a wallet
    /// When force=false, returns an error if the wallet has transactions.
    /// When force=true, deletes the wallet and cascades to its transactions.
    pub fn delete_wallet(&self, id: String, force: bool) -> Result<(), ControllerError> {
        self.crypto_service
            .delete_wallet(id, force)
            .map_err(ControllerError::from)
    }

    /// Returns the number of transactions in a wallet
    pub fn get_wallet_transaction_count(&self, id: String) -> Result<usize, ControllerError> {
        self.crypto_service
            .get_wallet_transaction_count(id)
            .map_err(ControllerError::from)
    }

    /// Updates a wallet's name
    pub fn update_wallet_name(&self, id: String, new_name: String) -> Result<(), ControllerError> {
        self.crypto_service
            .update_wallet_name(id, new_name)
            .map_err(ControllerError::from)
    }

    /// Updates a wallet's icon
    pub fn update_wallet_icon(
        &self,
        id: String,
        icon: Option<String>,
    ) -> Result<(), ControllerError> {
        self.crypto_service
            .update_wallet_icon(id, icon)
            .map_err(ControllerError::from)
    }

    // ==================== Crypto Transaction Methods ====================

    /// Adds a crypto transaction
    #[allow(clippy::too_many_arguments)]
    pub fn add_crypto_transaction(
        &self,
        wallet_id: String,
        coin_id: String,
        symbol: String,
        transaction_type: String,
        amount: f64,
        price_per_coin: Option<f64>,
        fee: Option<f64>,
        fee_coin_id: Option<String>,
        fee_amount: Option<f64>,
        date: String,
        notes: Option<String>,
        subtype: Option<String>,
        override_proceeds: Option<f64>,
        override_cost_basis: Option<f64>,
    ) -> Result<String, ControllerError> {
        self.crypto_service
            .add_crypto_transaction(
                wallet_id,
                coin_id,
                symbol,
                transaction_type,
                amount,
                price_per_coin,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
                subtype,
                override_proceeds,
                override_cost_basis,
            )
            .map_err(ControllerError::from)
    }

    /// Adds a transfer between two wallets as a paired outflow/inflow transaction
    #[allow(clippy::too_many_arguments)]
    pub fn add_crypto_transfer(
        &self,
        from_wallet_id: String,
        to_wallet_id: String,
        coin_id: String,
        symbol: String,
        from_amount: f64,
        to_amount: f64,
        fee: Option<f64>,
        fee_coin_id: Option<String>,
        fee_amount: Option<f64>,
        date: String,
        notes: Option<String>,
    ) -> Result<String, ControllerError> {
        self.crypto_service
            .add_crypto_transfer(
                from_wallet_id,
                to_wallet_id,
                coin_id,
                symbol,
                from_amount,
                to_amount,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
            )
            .map_err(ControllerError::from)
    }

    /// Adds a swap as a paired outflow/inflow transaction with shared cost basis
    #[allow(clippy::too_many_arguments)]
    pub fn add_crypto_swap(
        &self,
        wallet_id: String,
        from_coin_id: String,
        from_symbol: String,
        from_amount: f64,
        to_coin_id: String,
        to_symbol: String,
        to_amount: f64,
        fee: Option<f64>,
        fee_coin_id: Option<String>,
        fee_amount: Option<f64>,
        date: String,
        notes: Option<String>,
    ) -> Result<String, ControllerError> {
        self.crypto_service
            .add_crypto_swap(
                wallet_id,
                from_coin_id,
                from_symbol,
                from_amount,
                to_coin_id,
                to_symbol,
                to_amount,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
            )
            .map_err(ControllerError::from)
    }

    /// Gets wallet transactions
    pub fn get_wallet_transactions(
        &self,
        wallet_id: String,
    ) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.crypto_service
            .get_wallet_transactions(wallet_id)
            .map_err(ControllerError::from)
    }

    /// Gets a crypto transaction by ID
    pub fn get_crypto_transaction(
        &self,
        id: String,
    ) -> Result<Option<CryptoTransaction>, ControllerError> {
        self.crypto_service
            .get_crypto_transaction(id)
            .map_err(ControllerError::from)
    }

    /// Gets crypto transactions for a specific coin
    pub fn get_crypto_transactions_by_coin(
        &self,
        coin_id: String,
    ) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.crypto_service
            .get_crypto_transactions_by_coin(coin_id)
            .map_err(ControllerError::from)
    }

    /// Gets all crypto transactions across all wallets, paginated by offset/limit
    pub fn get_all_crypto_transactions(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<CryptoTransaction>, ControllerError> {
        self.crypto_service
            .get_all_crypto_transactions(offset, limit)
            .map_err(ControllerError::from)
    }

    /// Updates a crypto transaction's editable fields
    #[allow(clippy::too_many_arguments)]
    pub fn update_crypto_transaction(
        &self,
        id: String,
        amount: f64,
        price_per_coin: Option<f64>,
        fee: Option<f64>,
        fee_coin_id: Option<String>,
        fee_amount: Option<f64>,
        date: String,
        notes: Option<String>,
        subtype: Option<String>,
        override_proceeds: Option<f64>,
        override_cost_basis: Option<f64>,
    ) -> Result<(), ControllerError> {
        self.crypto_service
            .update_crypto_transaction(
                id,
                amount,
                price_per_coin,
                fee,
                fee_coin_id,
                fee_amount,
                date,
                notes,
                subtype,
                override_proceeds,
                override_cost_basis,
            )
            .map_err(ControllerError::from)
    }

    /// Deletes a crypto transaction
    pub fn delete_crypto_transaction(&self, id: String) -> Result<(), ControllerError> {
        self.crypto_service
            .delete_crypto_transaction(id)
            .map_err(ControllerError::from)
    }

    // ==================== Portfolio Aggregation Methods ====================

    /// Gets aggregated portfolio across all wallets
    pub fn get_aggregated_portfolio(&self) -> Result<Vec<AggregatedAsset>, ControllerError> {
        self.crypto_service
            .get_aggregated_portfolio()
            .map_err(ControllerError::from)
    }

    /// Gets aggregated holdings for a specific wallet
    pub fn get_wallet_holdings(
        &self,
        wallet_id: String,
    ) -> Result<Vec<AggregatedAsset>, ControllerError> {
        self.crypto_service
            .get_wallet_holdings(wallet_id)
            .map_err(ControllerError::from)
    }

    /// Gets the available balance for a specific coin in a wallet at a given date
    pub fn get_available_balance(
        &self,
        wallet_id: String,
        coin_id: String,
        _date: String, // Ignored - always uses current date
    ) -> Result<f64, ControllerError> {
        self.crypto_service
            .get_available_balance(wallet_id, coin_id, _date)
            .map_err(ControllerError::from)
    }
}
