//! Crypto feature module
//!
//! Handles cryptocurrency wallets, transactions, portfolio tracking, and API integration.

pub mod models;
pub mod repository;

// Re-export from original service for backwards compatibility
// TODO: Move service.rs logic here gradually
pub use crate::services::crypto::{
    CryptoError, CryptoService,
    SETTING_AUTO_FETCH, SETTING_CRYPTO_CUSTOM_COINS, SETTING_CRYPTO_FAVORITE_COINS,
    SETTING_CRYPTO_HIDDEN_COINS, SETTING_CRYPTO_LAST_COIN_ID, SETTING_CRYPTO_LAST_UPDATED,
    SETTING_CRYPTO_LAST_WALLET_ID, SETTING_TICKER_COINS,
    validate_coin_id,
};

pub use models::*;
pub use repository::CryptoRepository;
