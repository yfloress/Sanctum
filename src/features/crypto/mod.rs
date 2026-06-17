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

//! Crypto feature module
//!
//! Handles cryptocurrency wallets, transactions, portfolio tracking, and API integration.
//!
//! ## Module Structure
//! - `api` - CoinGecko API client
//! - `catalog` - Coin catalog management (custom coins, favorites, hidden)
//! - `models` - Domain models
//! - `repository` - Database operations
//! - `service` - Core service (wallets, prices, portfolio)
//! - `transactions` - Transaction operations (buy, sell, transfer, swap)
//! - `validation` - Input validation helpers

pub mod api;
pub mod catalog;
pub mod service;
mod service_tax;
pub mod tax;
pub mod transactions;
pub mod validation;

// Re-export API functions
pub use api::{
    default_coin_catalog, default_price_allowlist, default_ticker_ids, fetch_crypto_prices,
    fetch_usd_fx_rate, validate_coin_id,
};

// Re-export service
pub use service::{
    CryptoError, CryptoService, SETTING_AUTO_FETCH, SETTING_CRYPTO_CUSTOM_COINS,
    SETTING_CRYPTO_FAVORITE_COINS, SETTING_CRYPTO_HIDDEN_COINS, SETTING_CRYPTO_LAST_COIN_ID,
    SETTING_CRYPTO_LAST_UPDATED, SETTING_CRYPTO_LAST_WALLET_ID, SETTING_CRYPTO_PROXY_ENABLED,
    SETTING_CRYPTO_PROXY_URL, SETTING_CRYPTO_TAX_IPC_DATA, SETTING_CRYPTO_TAX_IPC_UPDATED,
    SETTING_DARK_MODE, SETTING_PREFERRED_CURRENCY, SETTING_PREFERRED_LANGUAGE,
    SETTING_SESSION_TIMEOUT, SETTING_SIDEBAR_COLLAPSED, SETTING_TICKER_COINS,
};

pub use tax::{IpcImportSummary, IpcSummary};
pub use tax::{
    LotAllocation, TaxDisposal, TaxReadinessItem, TaxReport, TaxReportSummary, TaxSummaryPayload,
    TaxWarning,
};
pub use tax::{TaxJurisdiction, TaxMethod, TaxPeriodSettings, TaxSettingsStore, TaxTxType};
