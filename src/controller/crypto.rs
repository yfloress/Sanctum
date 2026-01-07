//! Crypto-related controller methods
//!
//! Wallets, transactions, prices, and portfolio aggregation.

use super::{AppController, ControllerError};
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

    /// Fetches CLP to USD exchange rate
    pub async fn get_clp_usd_rate(&self) -> Result<f64, ControllerError> {
        self.crypto_service
            .get_clp_usd_rate()
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
    /// Returns an error if the wallet has transactions
    pub fn delete_wallet(&self, id: String) -> Result<(), ControllerError> {
        self.crypto_service
            .delete_wallet(id)
            .map_err(ControllerError::from)
    }

    /// Updates a wallet's name
    pub fn update_wallet_name(&self, id: String, new_name: String) -> Result<(), ControllerError> {
        self.crypto_service
            .update_wallet_name(id, new_name)
            .map_err(ControllerError::from)
    }

    /// Updates a wallet's icon
    pub fn update_wallet_icon(&self, id: String, icon: Option<String>) -> Result<(), ControllerError> {
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
