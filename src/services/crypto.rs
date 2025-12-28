//! Cryptocurrency module for fetching market data from CoinGecko API
//!
//! Security considerations:
//! - Input validation and sanitization for coin IDs
//! - Request timeout to prevent hanging
//! - Response size limits to prevent DoS
//! - Client-side rate limiting
//! - Input deduplication
//! - Output sanitization
//! - No sensitive data exposure in errors

use crate::db::{Database, DbError};
use crate::models::{
    AggregatedAsset, CryptoAsset, CryptoCatalogCoin, CryptoTransaction, CryptoTransactionType,
    CryptoWallet,
};
use crate::security_log::{SecurityEvent, log_rate_limit, log_security_event};
use chrono::{Local, NaiveDate};
use futures::TryStreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// CoinGecko API base URL (free tier, no API key required)
const COINGECKO_API_BASE: &str = "https://api.coingecko.com/api/v3";

/// Maximum number of coins per request (CoinGecko limit)
const MAX_COINS_PER_REQUEST: usize = 50;

/// Request timeout in seconds
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Maximum length for a coin ID (security: prevent oversized inputs)
const MAX_COIN_ID_LENGTH: usize = 64;

/// Maximum response body size (1MB - prevents DoS via large responses)
const MAX_RESPONSE_SIZE: usize = 1024 * 1024;

/// Minimum interval between API requests in milliseconds (rate limiting)
const MIN_REQUEST_INTERVAL_MS: u64 = 1500;

/// Maximum length for sanitized string fields from API
const MAX_SANITIZED_STRING_LENGTH: usize = 128;

/// Last request timestamp for rate limiting (atomic for thread safety)
static LAST_REQUEST_TIME: AtomicU64 = AtomicU64::new(0);

/// List of popular tickers used for traffic padding/obfuscation.
/// These are mixed with user requests to make it harder to fingerprint specific portfolio holdings
/// (e.g. distinguishing if a user owns Bitcoin vs just generic market monitoring).
const PRIVACY_PRESERVING_PRICE_IDS: &[&str] = &[
    "bitcoin", "ethereum", "litecoin", "monero", "tether", "solana", "polkadot", "cardano",
    "dogecoin", "ripple",
];

/// Default tickers shown in the UI when no user selection exists.
const DEFAULT_TICKER_IDS: &[&str] = &["bitcoin", "litecoin", "monero", "ethereum", "tether"];

pub const SETTING_AUTO_FETCH: &str = "auto_fetch_crypto";
pub const SETTING_TICKER_COINS: &str = "ticker_coins";
pub const SETTING_CRYPTO_LAST_UPDATED: &str = "crypto_last_updated";
pub const SETTING_CRYPTO_CUSTOM_COINS: &str = "crypto_custom_coins";
pub const SETTING_CRYPTO_HIDDEN_COINS: &str = "crypto_hidden_coins";
pub const SETTING_CRYPTO_FAVORITE_COINS: &str = "crypto_favorite_coins";
pub const SETTING_CRYPTO_LAST_WALLET_ID: &str = "crypto_last_wallet_id";
pub const SETTING_CRYPTO_LAST_COIN_ID: &str = "crypto_last_coin_id";

const MAX_NOTES_LENGTH: usize = 1024;
const MAX_WALLET_NAME_LENGTH: usize = 128;
const MAX_SYMBOL_LENGTH: usize = 16;
const MAX_ICON_LENGTH: usize = 32;
const MAX_COIN_NAME_LENGTH: usize = 64;

/// Internal struct to deserialize CoinGecko API response
#[derive(Debug, Deserialize)]
struct CoinGeckoMarketData {
    id: String,
    symbol: String,
    name: String,
    current_price: Option<f64>,
    price_change_percentage_24h: Option<f64>,
    last_updated: Option<String>,
}

/// Internal struct for simple price response (used for CLP/USD rate)
#[derive(Debug, Deserialize)]
struct SimplePriceResponse {
    tether: Option<TetherPrice>,
}

#[derive(Debug, Deserialize)]
struct TetherPrice {
    clp: Option<f64>,
}

/// Validates a coin ID to prevent injection or malformed inputs
pub fn validate_coin_id(coin_id: &str) -> Result<String, String> {
    // Reject any whitespace/control characters outright (prevents hidden newlines/tabs)
    if coin_id.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("Coin ID contains invalid characters".to_string());
    }

    let trimmed = coin_id.trim().to_lowercase();

    if trimmed.is_empty() {
        return Err("Coin ID cannot be empty".to_string());
    }

    if trimmed.len() > MAX_COIN_ID_LENGTH {
        return Err(format!(
            "Coin ID exceeds maximum length of {} characters",
            MAX_COIN_ID_LENGTH
        ));
    }

    // Only allow alphanumeric characters and hyphens (CoinGecko format)
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err("Coin ID contains invalid characters".to_string());
    }

    // Additional check: must start and end with alphanumeric
    if let (Some(first), Some(last)) = (trimmed.chars().next(), trimmed.chars().last())
        && (!first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric())
    {
        return Err("Coin ID must start and end with alphanumeric characters".to_string());
    }

    // Prevent consecutive hyphens
    if trimmed.contains("--") {
        return Err("Coin ID contains invalid characters".to_string());
    }

    Ok(trimmed)
}

/// Sanitizes a string from external API to prevent XSS and other injection attacks
fn sanitize_api_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric()
                || *c == ' '
                || *c == '-'
                || *c == '.'
                || *c == ','
                || *c == ':'
        })
        .take(MAX_SANITIZED_STRING_LENGTH)
        .collect()
}

/// Validates that a float value is within reasonable bounds
fn validate_price(price: f64) -> f64 {
    if price.is_nan() || price.is_infinite() || price < 0.0 {
        0.0
    } else if price > 1e15 {
        // Cap at 1 quadrillion (sanity check)
        1e15
    } else {
        price
    }
}

/// Validates percentage change is within reasonable bounds
fn validate_percentage(pct: f64) -> f64 {
    if pct.is_nan() || pct.is_infinite() {
        0.0
    } else {
        pct.clamp(-100.0, 10000.0) // -100% to +10000%
    }
}

/// Returns current time in milliseconds since UNIX epoch
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Enforces client-side rate limiting with a wait instead of an error
async fn enforce_rate_limit() -> Result<(), String> {
    loop {
        let now = now_millis();
        let last_request = LAST_REQUEST_TIME.load(Ordering::SeqCst);

        if last_request == 0 || now >= last_request + MIN_REQUEST_INTERVAL_MS {
            if LAST_REQUEST_TIME
                .compare_exchange(last_request, now, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(());
            }
            // If another thread updated the timestamp, re-evaluate
            continue;
        }

        let wait_ms = last_request + MIN_REQUEST_INTERVAL_MS - now;
        let wait_secs = wait_ms.div_ceil(1000);
        log_rate_limit("coingecko_api", wait_secs);
        sleep(Duration::from_millis(wait_ms)).await;
    }
}

/// Creates a secure HTTP client with appropriate settings
fn create_secure_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("Sanctum/1.0")
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none()) // Prevent redirect attacks
        .build()
        .map_err(|_| "Failed to create HTTP client".to_string())
}

/// Fetches cryptocurrency prices from CoinGecko API
///
/// # Arguments
/// * `coin_ids` - Vector of coin IDs (e.g., ["bitcoin", "ethereum"])
///
/// # Returns
/// * `Ok(Vec<CryptoAsset>)` - List of crypto assets with current prices
/// * `Err(String)` - Error message if the request fails
///
/// # Security
/// - Validates all coin IDs before making requests
/// - Deduplicates input to prevent URL bloat
/// - Limits the number of coins per request
/// - Uses HTTPS only
/// - Implements request timeout
/// - Limits response size
/// - Client-side rate limiting
/// - Sanitizes all output from external API
pub async fn fetch_crypto_prices(coin_ids: Vec<String>) -> Result<Vec<CryptoAsset>, String> {
    // Validate input is not empty
    if coin_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Check rate limit before proceeding (waits instead of failing fast)
    enforce_rate_limit().await?;

    // Validate, sanitize, and deduplicate all coin IDs
    let mut seen = HashSet::new();
    let mut validated_ids = Vec::new();

    for id in coin_ids.iter() {
        let validated = validate_coin_id(id)?;
        if seen.insert(validated.clone()) {
            validated_ids.push(validated);
        }
    }

    // Limit number of coins to prevent abuse
    if validated_ids.len() > MAX_COINS_PER_REQUEST {
        return Err(format!(
            "Too many coins requested. Maximum is {}",
            MAX_COINS_PER_REQUEST
        ));
    }

    // Build the comma-separated list of IDs
    let ids_param = validated_ids.join(",");

    // Construct the API URL
    let url = format!(
        "{}/coins/markets?vs_currency=usd&ids={}&order=market_cap_desc&sparkline=false",
        COINGECKO_API_BASE, ids_param
    );

    // Log API request
    log_security_event(
        SecurityEvent::ExternalApiRequest,
        Some(&format!("coins={}", validated_ids.len())),
    );

    // Create secure HTTP client
    let client = create_secure_client()?;

    // Make the request
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(handle_request_error)?;

    // Check HTTP status
    let status = response.status();
    if !status.is_success() {
        if status.as_u16() == 429 {
            log_security_event(SecurityEvent::ExternalApiRateLimited, Some("coingecko"));
        }
        return Err(match status.as_u16() {
            429 => "Rate limit exceeded. Please try again later.".to_string(),
            404 => "Cryptocurrency data not found.".to_string(),
            500..=599 => "CoinGecko service is temporarily unavailable.".to_string(),
            _ => "API request failed. Please try again later.".to_string(),
        });
    }

    // Check content length before downloading
    if let Some(content_length) = response.content_length()
        && content_length as usize > MAX_RESPONSE_SIZE
    {
        return Err("Response too large".to_string());
    }

    // Download body with size limit (streaming to avoid loading unbounded data)
    let mut downloaded: usize = 0;
    let mut body: Vec<u8> = Vec::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream
        .try_next()
        .await
        .map_err(|_| "Failed to download response".to_string())?
    {
        downloaded += chunk.len();
        if downloaded > MAX_RESPONSE_SIZE {
            return Err("Response too large".to_string());
        }
        body.extend_from_slice(&chunk);
    }

    // Parse response
    let market_data: Vec<CoinGeckoMarketData> = serde_json::from_slice(&body)
        .map_err(|_| "Failed to parse cryptocurrency data".to_string())?;

    // Limit number of results (defense in depth)
    if market_data.len() > MAX_COINS_PER_REQUEST * 2 {
        return Err("Unexpected response format".to_string());
    }

    // Convert to our internal struct with sanitization
    let assets: Vec<CryptoAsset> = market_data
        .into_iter()
        .map(|data| CryptoAsset {
            id: sanitize_api_string(&data.id),
            symbol: sanitize_api_string(&data.symbol).to_uppercase(),
            name: sanitize_api_string(&data.name),
            current_price: validate_price(data.current_price.unwrap_or(0.0)),
            price_change_percentage_24h: validate_percentage(
                data.price_change_percentage_24h.unwrap_or(0.0),
            ),
            last_updated: sanitize_api_string(
                &data.last_updated.unwrap_or_else(|| "N/A".to_string()),
            ),
        })
        .collect();

    Ok(assets)
}

/// Returns the padding list used for default tickers and privacy obfuscation.
pub fn default_price_allowlist() -> Vec<String> {
    PRIVACY_PRESERVING_PRICE_IDS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Returns the default ticker IDs shown in the UI.
pub fn default_ticker_ids() -> Vec<String> {
    DEFAULT_TICKER_IDS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Returns the default coin catalog used for selection and ticker configuration.
pub fn default_coin_catalog() -> Vec<CryptoCatalogCoin> {
    let defaults = [
        ("bitcoin", "Bitcoin", "BTC"),
        ("litecoin", "Litecoin", "LTC"),
        ("monero", "Monero", "XMR"),
        ("ethereum", "Ethereum", "ETH"),
        ("tether", "Tether", "USDT"),
        ("binancecoin", "BNB", "BNB"),
        ("solana", "Solana", "SOL"),
        ("ripple", "XRP", "XRP"),
        ("usd-coin", "USDC", "USDC"),
        ("cardano", "Cardano", "ADA"),
        ("dogecoin", "Dogecoin", "DOGE"),
        ("tron", "TRON", "TRX"),
        ("polygon-ecosystem-token", "Polygon", "POL"),
        ("chainlink", "Chainlink", "LINK"),
        ("polkadot", "Polkadot", "DOT"),
        ("shiba-inu", "Shiba Inu", "SHIB"),
        ("avalanche-2", "Avalanche", "AVAX"),
        ("stellar", "Stellar", "XLM"),
        ("bitcoin-cash", "Bitcoin Cash", "BCH"),
        ("uniswap", "Uniswap", "UNI"),
        ("cosmos", "Cosmos Hub", "ATOM"),
        ("ethereum-classic", "Ethereum Classic", "ETC"),
        ("hedera-hashgraph", "Hedera", "HBAR"),
        ("aave", "Aave", "AAVE"),
        ("vechain", "VeChain", "VET"),
        ("near", "NEAR Protocol", "NEAR"),
        ("algorand", "Algorand", "ALGO"),
        ("quant-network", "Quant", "QNT"),
        ("arbitrum", "Arbitrum", "ARB"),
        ("sui", "Sui", "SUI"),
        ("aptos", "Aptos", "APT"),
        ("crypto-com-chain", "Cronos", "CRO"),
        ("zcash", "Zcash", "ZEC"),
        ("dai", "Dai", "DAI"),
        ("the-open-network", "Toncoin", "TON"),
        ("internet-computer", "Internet Computer", "ICP"),
        ("kaspa", "Kaspa", "KAS"),
        ("mantle", "Mantle", "MNT"),
        ("bittensor", "Bittensor", "TAO"),
        ("worldcoin-wld", "Worldcoin", "WLD"),
    ];

    defaults
        .iter()
        .map(|(id, name, symbol)| CryptoCatalogCoin {
            id: (*id).to_string(),
            name: (*name).to_string(),
            symbol: (*symbol).to_string(),
            custom: false,
        })
        .collect()
}

/// Fetches the current CLP to USD exchange rate using CoinGecko
/// Returns how many CLP equals 1 USD (e.g., ~950 CLP = 1 USD)
///
/// We use USDT/CLP as proxy since CoinGecko provides this pair.
/// The rate returned is CLP per 1 USD.
pub async fn fetch_clp_usd_rate() -> Result<f64, String> {
    // Check rate limit before proceeding (waits instead of failing fast)
    enforce_rate_limit().await?;

    // Use simple/price endpoint to get USDT price in CLP
    // USDT ≈ 1 USD, so this gives us CLP/USD rate
    let url = format!(
        "{}/simple/price?ids=tether&vs_currencies=clp",
        COINGECKO_API_BASE
    );

    log_security_event(SecurityEvent::ExternalApiRequest, Some("clp_usd_rate"));

    let client = create_secure_client()?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(handle_request_error)?;

    let status = response.status();
    if !status.is_success() {
        if status.as_u16() == 429 {
            log_security_event(SecurityEvent::ExternalApiRateLimited, Some("coingecko"));
        }
        return Err("Failed to fetch exchange rate".to_string());
    }

    // Limit response size to prevent abuse
    if let Some(content_length) = response.content_length()
        && content_length as usize > MAX_RESPONSE_SIZE
    {
        return Err("Response too large".to_string());
    }

    let mut downloaded: usize = 0;
    let mut body: Vec<u8> = Vec::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream
        .try_next()
        .await
        .map_err(|_| "Failed to download response".to_string())?
    {
        downloaded += chunk.len();
        if downloaded > MAX_RESPONSE_SIZE {
            return Err("Response too large".to_string());
        }
        body.extend_from_slice(&chunk);
    }

    let price_data: SimplePriceResponse =
        serde_json::from_slice(&body).map_err(|_| "Failed to parse exchange rate data")?;

    let rate = price_data
        .tether
        .and_then(|t| t.clp)
        .ok_or("CLP rate not available")?;

    // Validate the rate is reasonable (between 500 and 2000 CLP per USD)
    if !(100.0..=5000.0).contains(&rate) {
        return Err("Exchange rate out of expected range".to_string());
    }

    Ok(rate)
}

/// Handles request errors without exposing sensitive information
fn handle_request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "Request timed out. Please check your connection.".to_string()
    } else if error.is_connect() {
        "Could not connect to the server. Please check your internet connection.".to_string()
    } else if error.is_redirect() {
        "Request was redirected unexpectedly.".to_string()
    } else {
        // Generic error message to avoid leaking internal details
        "Failed to fetch cryptocurrency data. Please try again later.".to_string()
    }
}

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

pub struct CryptoService {
    db: Arc<Mutex<Option<Database>>>,
}

impl CryptoService {
    pub fn new(db: Arc<Mutex<Option<Database>>>) -> Self {
        Self { db }
    }

    fn with_db<T, F>(&self, f: F) -> Result<T, CryptoError>
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

    pub fn get_app_setting(&self, key: &str) -> Result<String, CryptoError> {
        self.with_db(|db| {
            let val = db.get_setting(key).map_err(CryptoError::Database)?;
            Ok(val.unwrap_or_default())
        })
    }

    pub fn set_app_setting(&self, key: &str, value: &str) -> Result<(), CryptoError> {
        self.with_db(|db| {
            db.set_setting(key, value)
                .map_err(CryptoError::Database)
        })
    }

    pub fn get_active_ticker_ids(&self) -> Vec<String> {
        self.get_app_setting(SETTING_TICKER_COINS)
            .ok()
            .filter(|val| !val.is_empty())
            .and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
            .unwrap_or_else(default_ticker_ids)
    }

    pub fn save_active_ticker_ids(&self, ids: Vec<String>) -> Result<(), CryptoError> {
        let json =
            serde_json::to_string(&ids).map_err(|e| CryptoError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_TICKER_COINS, &json)
    }

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

    fn save_custom_coin_catalog(&self, coins: Vec<CryptoCatalogCoin>) -> Result<(), CryptoError> {
        let json = serde_json::to_string(&coins)
            .map_err(|e| CryptoError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_CRYPTO_CUSTOM_COINS, &json)
    }

    fn save_hidden_coin_ids(&self, ids: Vec<String>) -> Result<(), CryptoError> {
        let json =
            serde_json::to_string(&ids).map_err(|e| CryptoError::Validation(e.to_string()))?;
        self.set_app_setting(SETTING_CRYPTO_HIDDEN_COINS, &json)
    }

    fn save_favorite_coin_ids(&self, ids: Vec<String>) -> Result<(), CryptoError> {
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

        fetch_crypto_prices(final_list)
            .await
            .map_err(CryptoError::Api)
    }

    pub async fn get_clp_usd_rate(&self) -> Result<f64, CryptoError> {
        fetch_clp_usd_rate().await.map_err(CryptoError::Api)
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

            let valid_categories = ["exchange", "wallet_single", "wallet_multi"];
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
            if existing_wallets.iter().any(|w| w.name.eq_ignore_ascii_case(&name)) {
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

    pub fn delete_wallet(&self, id: String) -> Result<(), CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;

            let transactions = db.get_wallet_transactions(&validated_id)?;
            if !transactions.is_empty() {
                return Err(CryptoError::Validation(format!(
                    "Cannot delete wallet with {} transaction{}. Please delete all transactions first.",
                    transactions.len(),
                    if transactions.len() == 1 { "" } else { "s" }
                )));
            }

            db.delete_wallet(&validated_id)?;
            log_security_event(SecurityEvent::WalletDeleted, None);
            Ok(())
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
    ) -> Result<String, CryptoError> {
        self.with_db(|db| {
            let wallet_id = wallet_id.trim().to_string();
            if wallet_id.is_empty() {
                return Err(CryptoError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
            }

            let coin_id = validate_coin_id_str(&coin_id)?;
            let symbol = validate_symbol(&symbol)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            validate_positive_amount(amount, "Amount")?;

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let valid_types = ["buy", "sell", "transfer_in", "transfer_out", "swap"];
            if !valid_types.contains(&transaction_type.as_str()) {
                return Err(CryptoError::Validation(format!(
                    "Invalid transaction type. Must be one of: {}",
                    valid_types.join(", ")
                )));
            }

            if transaction_type == "swap" {
                return Err(CryptoError::Validation(
                    "Swap requires paired transactions. Use the swap flow.".to_string(),
                ));
            }

            let price = if transaction_type == "buy" || transaction_type == "sell" {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => {
                        return Err(CryptoError::Validation(
                            "Price per coin is required and must be greater than zero".to_string(),
                        ))
                    }
                }
            } else {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => None,
                }
            };

            let is_outflow = transaction_type == "sell"
                || transaction_type == "transfer_out"
                || transaction_type == "swap";

            if is_outflow {
                validate_sufficient_balance(
                    db,
                    &wallet_id,
                    &coin_id,
                    &symbol,
                    amount,
                    &date,
                    None,
                )?;
            }

            let fee_context = FeeBalanceContext {
                db,
                wallet_id: &wallet_id,
                main_coin_id: &coin_id,
                main_symbol: &symbol,
                main_amount: amount,
                is_outflow,
                date: &date,
                exclude_tx_id: None,
            };
            validate_fee_balance(fee_context, fee_coin_id.as_deref(), fee_amount)?;

            log_security_event(
                SecurityEvent::CryptoTransactionCreated,
                Some(&transaction_type),
            );

            let id = Uuid::new_v4().to_string();
            let mut transaction = CryptoTransaction::new(
                id.clone(),
                wallet_id,
                coin_id.to_lowercase(),
                symbol.to_uppercase(),
                transaction_type,
                amount,
                price,
                fee,
                date,
                notes,
            );
            transaction.fee_coin_id = fee_coin_id;
            transaction.fee_amount = fee_amount;

            db.create_crypto_transaction(&transaction)?;
            Ok(id)
        })
    }

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
    ) -> Result<String, CryptoError> {
        self.with_db(|db| {
            let from_wallet_id = from_wallet_id.trim().to_string();
            let to_wallet_id = to_wallet_id.trim().to_string();
            if from_wallet_id.is_empty() || to_wallet_id.is_empty() {
                return Err(CryptoError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
            }
            if from_wallet_id == to_wallet_id {
                return Err(CryptoError::Validation(
                    "Source and destination wallets must be different".to_string(),
                ));
            }

            if db.get_wallet(&from_wallet_id)?.is_none() {
                return Err(CryptoError::Validation(
                    "Source wallet not found".to_string(),
                ));
            }
            if db.get_wallet(&to_wallet_id)?.is_none() {
                return Err(CryptoError::Validation(
                    "Destination wallet not found".to_string(),
                ));
            }

            let coin_id = validate_coin_id_str(&coin_id)?;
            let symbol = validate_symbol(&symbol)?;
            validate_positive_amount(from_amount, "From amount")?;
            validate_positive_amount(to_amount, "To amount")?;
            if to_amount > from_amount {
                return Err(CryptoError::Validation(
                    "To amount cannot exceed from amount".to_string(),
                ));
            }

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            let current_balance =
                db.get_wallet_coin_balance_at(&from_wallet_id, &coin_id, &date, None)?;
            if from_amount > current_balance {
                return Err(CryptoError::Validation(format!(
                    "Insufficient funds. Available: {:.8} {}",
                    current_balance, symbol
                )));
            }

            let fee_context = FeeBalanceContext {
                db,
                wallet_id: &from_wallet_id,
                main_coin_id: &coin_id,
                main_symbol: &symbol,
                main_amount: from_amount,
                is_outflow: true,
                date: &date,
                exclude_tx_id: None,
            };
            validate_fee_balance(fee_context, fee_coin_id.as_deref(), fee_amount)?;

            if let (Some(fee_coin), Some(_)) = (fee_coin_id.as_deref(), fee_amount)
                && fee_coin == coin_id
                && to_amount < from_amount
            {
                return Err(CryptoError::Validation(
                    "When using a same-coin network fee, keep the TO amount equal to FROM (the fee is recorded separately)".to_string(),
                ));
            }

            let (total_amount, total_cost) =
                db.get_wallet_coin_state_at(&from_wallet_id, &coin_id, &date)?;
            let avg_price = if total_amount > 0.0 {
                total_cost / total_amount
            } else {
                0.0
            };
            let transfer_price = if avg_price > 0.0 { Some(avg_price) } else { None };

            log_security_event(SecurityEvent::CryptoTransactionCreated, Some("transfer"));

            let source_id = Uuid::new_v4().to_string();
            let target_id = Uuid::new_v4().to_string();

            let source = CryptoTransaction {
                id: source_id.clone(),
                wallet_id: from_wallet_id,
                coin_id: coin_id.clone(),
                symbol: symbol.clone(),
                transaction_type: "transfer_out".to_string(),
                amount: from_amount,
                price_per_coin: None,
                fee: None,
                fee_coin_id: fee_coin_id.clone(),
                fee_amount,
                date: date.clone(),
                notes: notes.clone(),
                related_tx_id: Some(target_id.clone()),
            };

            let target = CryptoTransaction {
                id: target_id.clone(),
                wallet_id: to_wallet_id,
                coin_id,
                symbol,
                transaction_type: "transfer_in".to_string(),
                amount: to_amount,
                price_per_coin: transfer_price,
                fee,
                fee_coin_id: None,
                fee_amount: None,
                date,
                notes,
                related_tx_id: Some(source_id.clone()),
            };

            db.create_crypto_transaction(&source)?;
            if let Err(err) = db.create_crypto_transaction(&target) {
                if let Err(rollback_err) = db.delete_crypto_transaction(&source_id) {
                    log::error!(
                        "Failed to rollback transfer source transaction {}: {:?}",
                        source_id, rollback_err
                    );
                }
                return Err(CryptoError::Database(err));
            }

            Ok(source_id)
        })
    }

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
    ) -> Result<String, CryptoError> {
        self.with_db(|db| {
            let wallet_id = wallet_id.trim().to_string();
            if wallet_id.is_empty() {
                return Err(CryptoError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
            }

            let from_coin_id = validate_coin_id_str(&from_coin_id)?;
            let to_coin_id = validate_coin_id_str(&to_coin_id)?;
            if from_coin_id == to_coin_id {
                return Err(CryptoError::Validation(
                    "Swap requires two different assets".to_string(),
                ));
            }

            let from_symbol = validate_symbol(&from_symbol)?;
            let to_symbol = validate_symbol(&to_symbol)?;
            validate_positive_amount(from_amount, "From amount")?;
            validate_positive_amount(to_amount, "To amount")?;

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            validate_sufficient_balance(
                db,
                &wallet_id,
                &from_coin_id,
                &from_symbol,
                from_amount,
                &date,
                None,
            )?;

            if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id.as_deref(), fee_amount) {
                if fee_coin == from_coin_id {
                    let total_required = from_amount + fee_amt;
                    validate_sufficient_balance(
                        db,
                        &wallet_id,
                        &from_coin_id,
                        &from_symbol,
                        total_required,
                        &date,
                        None,
                    )?;
                } else if fee_coin == to_coin_id {
                    let to_balance = db.get_wallet_coin_balance_at(&wallet_id, fee_coin, &date, None)?;
                    if fee_amt > to_amount + to_balance {
                        return Err(CryptoError::Validation(
                            "Fee amount exceeds available output balance".to_string(),
                        ));
                    }
                } else {
                    validate_sufficient_balance(
                        db,
                        &wallet_id,
                        fee_coin,
                        fee_coin,
                        fee_amt,
                        &date,
                        None,
                    )?;
                }
            }

            log_security_event(SecurityEvent::CryptoTransactionCreated, Some("swap"));

            let source_id = Uuid::new_v4().to_string();
            let target_id = Uuid::new_v4().to_string();

            let source = CryptoTransaction {
                id: source_id.clone(),
                wallet_id: wallet_id.clone(),
                coin_id: from_coin_id,
                symbol: from_symbol,
                transaction_type: "swap".to_string(),
                amount: from_amount,
                price_per_coin: None,
                fee,
                fee_coin_id: fee_coin_id.clone(),
                fee_amount,
                date: date.clone(),
                notes,
                related_tx_id: Some(target_id.clone()),
            };

            let target = CryptoTransaction {
                id: target_id.clone(),
                wallet_id,
                coin_id: to_coin_id,
                symbol: to_symbol,
                transaction_type: "transfer_in".to_string(),
                amount: to_amount,
                price_per_coin: None,
                fee: None,
                fee_coin_id: None,
                fee_amount: None,
                date,
                notes: None,
                related_tx_id: Some(source_id.clone()),
            };

            db.create_crypto_transaction(&source)?;
            if let Err(err) = db.create_crypto_transaction(&target) {
                if let Err(rollback_err) = db.delete_crypto_transaction(&source_id) {
                    log::error!(
                        "Failed to rollback swap source transaction {}: {:?}",
                        source_id, rollback_err
                    );
                }
                return Err(CryptoError::Database(err));
            }

            Ok(source_id)
        })
    }

    pub fn get_wallet_transactions(
        &self,
        wallet_id: String,
    ) -> Result<Vec<CryptoTransaction>, CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&wallet_id)?;
            db.get_wallet_transactions(&validated_id)
                .map_err(CryptoError::Database)
        })
    }

    pub fn get_crypto_transaction(
        &self,
        id: String,
    ) -> Result<Option<CryptoTransaction>, CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.get_crypto_transaction(&validated_id)
                .map_err(CryptoError::Database)
        })
    }

    pub fn get_crypto_transactions_by_coin(
        &self,
        coin_id: String,
    ) -> Result<Vec<CryptoTransaction>, CryptoError> {
        self.with_db(|db| {
            let validated = validate_coin_id_str(&coin_id)?;
            db.get_crypto_transactions_by_coin(&validated)
                .map_err(CryptoError::Database)
        })
    }

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
    ) -> Result<(), CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            let existing = db.get_crypto_transaction(&validated_id)?;
            let existing = match existing {
                Some(tx) => tx,
                None => {
                    return Err(CryptoError::Validation(
                        "Transaction not found".to_string(),
                    ))
                }
            };

            if existing.transaction_type == "swap" || existing.related_tx_id.is_some() {
                return Err(CryptoError::Validation(
                    "Editing paired transactions is not supported".to_string(),
                ));
            }

            validate_positive_amount(amount, "Amount")?;
            let price = if existing.transaction_type == "buy"
                || existing.transaction_type == "sell"
            {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => {
                        return Err(CryptoError::Validation(
                            "Price per coin is required and must be greater than zero".to_string(),
                        ))
                    }
                }
            } else {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => None,
                }
            };
            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            let is_outflow = existing.transaction_type == "sell"
                || existing.transaction_type == "transfer_out";

            let existing_type = existing.get_type().unwrap_or(CryptoTransactionType::Buy);

            let mut balance_excluding =
                db.get_wallet_coin_balance_at(&existing.wallet_id, &existing.coin_id, &date, None)?;
            match existing_type {
                CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                    balance_excluding -= existing.amount;
                }
                CryptoTransactionType::Sell
                | CryptoTransactionType::TransferOut
                | CryptoTransactionType::Swap => {
                    balance_excluding += existing.amount;
                }
            }
            if existing.fee_coin_id.as_deref() == Some(existing.coin_id.as_str())
                && let Some(fee_amt) = existing.fee_amount
            {
                balance_excluding += fee_amt;
            }

            if is_outflow && amount > balance_excluding {
                return Err(CryptoError::Validation(format!(
                    "Insufficient funds. Available: {:.8} {}",
                    balance_excluding, existing.symbol
                )));
            }

            if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id.as_deref(), fee_amount) {
                let mut fee_balance_excluding = if fee_coin == existing.coin_id {
                    balance_excluding
                } else {
                    db.get_wallet_coin_balance_at(&existing.wallet_id, fee_coin, &date, None)?
                };
                if existing.fee_coin_id.as_deref() == Some(fee_coin)
                    && let Some(existing_fee_amt) = existing.fee_amount
                {
                    fee_balance_excluding += existing_fee_amt;
                }
                if fee_coin == existing.coin_id {
                    if is_outflow {
                        let total_required = amount + fee_amt;
                        if total_required > fee_balance_excluding {
                            return Err(CryptoError::Validation(format!(
                                "Insufficient funds for fee. Available: {:.8} {}",
                                fee_balance_excluding, existing.symbol
                            )));
                        }
                    } else {
                        let total_available = fee_balance_excluding + amount;
                        if fee_amt > total_available {
                            return Err(CryptoError::Validation(
                                "Fee amount exceeds available balance".to_string(),
                            ));
                        }
                    }
                } else if fee_amt > fee_balance_excluding {
                    return Err(CryptoError::Validation(format!(
                        "Insufficient funds for fee. Available: {:.8} {}",
                        fee_balance_excluding, fee_coin
                    )));
                }
            }

            db.update_crypto_transaction_fields(
                &validated_id,
                amount,
                price,
                fee,
                fee_coin_id.as_deref(),
                fee_amount,
                &date,
                notes.as_deref(),
            )?;

            Ok(())
        })
    }

    pub fn delete_crypto_transaction(&self, id: String) -> Result<(), CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;

            if let Ok(Some(tx)) = db.get_crypto_transaction(&validated_id)
                && let Some(related_id) = tx.related_tx_id
            {
                let _ = db.delete_crypto_transaction(&related_id);
            }

            db.delete_crypto_transaction(&validated_id)?;
            Ok(())
        })
    }

    pub fn get_aggregated_portfolio(&self) -> Result<Vec<AggregatedAsset>, CryptoError> {
        self.with_db(|db| {
            db.get_aggregated_portfolio()
                .map_err(CryptoError::Database)
        })
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
        _date: String,
    ) -> Result<f64, CryptoError> {
        self.with_db(|db| {
            let validated_wallet_id = validate_uuid(&wallet_id)?;
            let validated_coin_id = validate_coin_id_str(&coin_id)?;

            let today = Local::now().format("%Y-%m-%d").to_string();

            db.get_wallet_coin_balance_at(
                &validated_wallet_id,
                &validated_coin_id,
                &today,
                None,
            )
            .map_err(CryptoError::Database)
        })
    }
}

fn validate_coin_id_str(coin_id: &str) -> Result<String, CryptoError> {
    validate_coin_id(coin_id).map_err(CryptoError::Validation)
}

fn validate_symbol(symbol: &str) -> Result<String, CryptoError> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return Err(CryptoError::Validation(
            "Symbol cannot be empty".to_string(),
        ));
    }
    if trimmed.len() > MAX_SYMBOL_LENGTH {
        return Err(CryptoError::Validation(format!(
            "Symbol exceeds maximum length of {} characters",
            MAX_SYMBOL_LENGTH
        )));
    }
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(CryptoError::Validation(
            "Symbol must be alphanumeric".to_string(),
        ));
    }
    Ok(trimmed.to_uppercase())
}

fn validate_positive_amount(value: f64, field: &str) -> Result<f64, CryptoError> {
    if !value.is_finite() {
        return Err(CryptoError::Validation(format!(
            "{} must be a finite number",
            field
        )));
    }
    if value <= 0.0 {
        return Err(CryptoError::Validation(format!(
            "{} must be greater than zero",
            field
        )));
    }
    Ok(value)
}

fn validate_non_negative(value: Option<f64>, field: &str) -> Result<Option<f64>, CryptoError> {
    if let Some(v) = value {
        if !v.is_finite() {
            return Err(CryptoError::Validation(format!(
                "{} must be a finite number",
                field
            )));
        }
        if v < 0.0 {
            return Err(CryptoError::Validation(format!(
                "{} cannot be negative",
                field
            )));
        }
    }
    Ok(value)
}

fn normalize_fee_coin(
    fee_coin_id: Option<String>,
    fee_amount: Option<f64>,
) -> Result<(Option<String>, Option<f64>), CryptoError> {
    let fee_coin_id = fee_coin_id.and_then(|id| {
        let trimmed = id.trim().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    });

    match (fee_coin_id, fee_amount) {
        (None, None) => Ok((None, None)),
        (Some(id), Some(amount)) => {
            let id = validate_coin_id_str(&id)?;
            let amount = validate_positive_amount(amount, "Fee amount")?;
            Ok((Some(id), Some(amount)))
        }
        (None, Some(_)) => Err(CryptoError::Validation(
            "Fee coin is required when fee amount is provided".to_string(),
        )),
        (Some(_), None) => Ok((None, None)),
    }
}

fn validate_sufficient_balance(
    db: &Database,
    wallet_id: &str,
    coin_id: &str,
    symbol: &str,
    required_amount: f64,
    date: &str,
    exclude_tx_id: Option<&str>,
) -> Result<(), CryptoError> {
    let balance = db
        .get_wallet_coin_balance_at(wallet_id, coin_id, date, exclude_tx_id)
        .map_err(CryptoError::Database)?;

    if required_amount > balance {
        return Err(CryptoError::Validation(format!(
            "Insufficient funds. Available: {:.8} {}",
            balance, symbol
        )));
    }
    Ok(())
}

struct FeeBalanceContext<'a> {
    db: &'a Database,
    wallet_id: &'a str,
    main_coin_id: &'a str,
    main_symbol: &'a str,
    main_amount: f64,
    is_outflow: bool,
    date: &'a str,
    exclude_tx_id: Option<&'a str>,
}

fn validate_fee_balance(
    ctx: FeeBalanceContext<'_>,
    fee_coin_id: Option<&str>,
    fee_amount: Option<f64>,
) -> Result<(), CryptoError> {
    if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id, fee_amount) {
        if fee_coin == ctx.main_coin_id {
            if ctx.is_outflow {
                let total_required = ctx.main_amount + fee_amt;
                validate_sufficient_balance(
                    ctx.db,
                    ctx.wallet_id,
                    ctx.main_coin_id,
                    ctx.main_symbol,
                    total_required,
                    ctx.date,
                    ctx.exclude_tx_id,
                )?;
            } else {
                let existing = ctx
                    .db
                    .get_wallet_coin_balance_at(
                        ctx.wallet_id,
                        ctx.main_coin_id,
                        ctx.date,
                        ctx.exclude_tx_id,
                    )
                    .map_err(CryptoError::Database)?;
                if fee_amt > ctx.main_amount + existing {
                    return Err(CryptoError::Validation(
                        "Fee amount exceeds the available balance for this asset".to_string(),
                    ));
                }
            }
        } else {
            validate_sufficient_balance(
                ctx.db,
                ctx.wallet_id,
                fee_coin,
                fee_coin,
                fee_amt,
                ctx.date,
                ctx.exclude_tx_id,
            )?;
        }
    }
    Ok(())
}

fn validate_field_length(
    value: &str,
    max_length: usize,
    field_name: &str,
) -> Result<String, CryptoError> {
    let trimmed = value.trim();
    if trimmed.len() > max_length {
        return Err(CryptoError::Validation(format!(
            "{} exceeds maximum length of {} characters",
            field_name, max_length
        )));
    }
    Ok(trimmed.to_string())
}

fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric()
                || c.is_whitespace()
                || matches!(
                    c,
                    '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '-' | '_' | '+'
                        | '=' | '{' | '}' | '[' | ']' | '|' | '\\' | ':' | '\'' | '"' | ',' | '.'
                        | '<' | '>' | '?' | '/' | '`' | '~'
                )
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn validate_uuid(id: &str) -> Result<String, CryptoError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(CryptoError::Validation("ID cannot be empty".to_string()));
    }

    if Uuid::parse_str(trimmed).is_ok() {
        return Ok(trimmed.to_string());
    }

    Err(CryptoError::Validation("Invalid ID format".to_string()))
}

fn validate_date(date: &str) -> Result<String, CryptoError> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return Err(CryptoError::Validation("Date cannot be empty".to_string()));
    }

    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%d-%m-%Y") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    Err(CryptoError::Validation(
        "Invalid date format. Use DD-MM-YYYY or YYYY-MM-DD".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_coin_id_valid() {
        assert!(validate_coin_id("bitcoin").is_ok());
        assert!(validate_coin_id("ethereum").is_ok());
        assert!(validate_coin_id("binance-coin").is_ok());
        assert!(validate_coin_id("BITCOIN").is_ok()); // Should lowercase
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
        assert!(validate_coin_id("-bitcoin").is_err()); // Starts with hyphen
        assert!(validate_coin_id("bitcoin-").is_err()); // Ends with hyphen
        assert!(validate_coin_id("bit--coin").is_err()); // Consecutive hyphens
    }

    #[test]
    fn test_validate_coin_id_too_long() {
        let long_id = "a".repeat(MAX_COIN_ID_LENGTH + 1);
        assert!(validate_coin_id(&long_id).is_err());

        let max_id = "a".repeat(MAX_COIN_ID_LENGTH);
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
        assert_eq!(sanitized.len(), MAX_SANITIZED_STRING_LENGTH);
    }

    #[test]
    fn test_validate_price() {
        assert_eq!(validate_price(100.0), 100.0);
        assert_eq!(validate_price(-50.0), 0.0);
        assert_eq!(validate_price(f64::NAN), 0.0);
        assert_eq!(validate_price(f64::INFINITY), 0.0);
        assert_eq!(validate_price(1e20), 1e15); // Capped
    }

    #[test]
    fn test_validate_percentage() {
        assert_eq!(validate_percentage(5.5), 5.5);
        assert_eq!(validate_percentage(-50.0), -50.0);
        assert_eq!(validate_percentage(-150.0), -100.0); // Clamped
        assert_eq!(validate_percentage(50000.0), 10000.0); // Clamped
        assert_eq!(validate_percentage(f64::NAN), 0.0);
    }
}
