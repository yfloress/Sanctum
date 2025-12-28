//! Crypto feature module
//!
//! Handles cryptocurrency wallets, transactions, portfolio tracking, and API integration.

pub mod api;
pub mod models;
pub mod repository;
pub mod service;

// Re-export API functions
pub use api::{
    default_coin_catalog, default_price_allowlist, default_ticker_ids,
    fetch_clp_usd_rate, fetch_crypto_prices, validate_coin_id,
};

// Re-export service
pub use service::{
    CryptoError, CryptoService,
    SETTING_AUTO_FETCH, SETTING_CRYPTO_CUSTOM_COINS, SETTING_CRYPTO_FAVORITE_COINS,
    SETTING_CRYPTO_HIDDEN_COINS, SETTING_CRYPTO_LAST_COIN_ID, SETTING_CRYPTO_LAST_UPDATED,
    SETTING_CRYPTO_LAST_WALLET_ID, SETTING_TICKER_COINS,
};

pub use models::*;
pub use repository::CryptoRepository;
