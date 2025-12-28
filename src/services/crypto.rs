//! Cryptocurrency service module
//!
//! This module re-exports from features/crypto for backwards compatibility.
//! All implementation has been moved to features/crypto/service.rs and features/crypto/api.rs

// Re-export everything from features/crypto for backwards compatibility
pub use crate::features::crypto::{
    // Service and error types
    CryptoError, CryptoService,
    // Settings constants
    SETTING_AUTO_FETCH, SETTING_CRYPTO_CUSTOM_COINS, SETTING_CRYPTO_FAVORITE_COINS,
    SETTING_CRYPTO_HIDDEN_COINS, SETTING_CRYPTO_LAST_COIN_ID, SETTING_CRYPTO_LAST_UPDATED,
    SETTING_CRYPTO_LAST_WALLET_ID, SETTING_TICKER_COINS,
    // API functions
    default_coin_catalog, default_price_allowlist, default_ticker_ids,
    fetch_clp_usd_rate, fetch_crypto_prices, validate_coin_id,
};
