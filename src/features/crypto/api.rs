//! CoinGecko API client for cryptocurrency market data
//!
//! Security considerations:
//! - Input validation and sanitization for coin IDs
//! - Request timeout to prevent hanging
//! - Response size limits to prevent DoS
//! - Client-side rate limiting
//! - Input deduplication
//! - Output sanitization
//! - No sensitive data exposure in errors

use crate::models::{CryptoAsset, CryptoCatalogCoin};
use crate::security_log::{SecurityEvent, log_rate_limit, log_security_event};
use futures::TryStreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;

// ==================== Constants ====================

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

/// List of popular tickers used for traffic padding/obfuscation
const PRIVACY_PRESERVING_PRICE_IDS: &[&str] = &[
    "bitcoin", "ethereum", "litecoin", "monero", "tether", "solana", "polkadot", "cardano",
    "dogecoin", "ripple",
];

/// Default tickers shown in the UI when no user selection exists
const DEFAULT_TICKER_IDS: &[&str] = &["bitcoin", "litecoin", "monero", "ethereum", "tether"];

// ==================== Internal Structs ====================

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

// ==================== Validation Functions ====================

/// Validates a coin ID to prevent injection or malformed inputs
pub fn validate_coin_id(coin_id: &str) -> Result<String, String> {
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

    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Coin ID contains invalid characters".to_string());
    }

    if let (Some(first), Some(last)) = (trimmed.chars().next(), trimmed.chars().last())
        && (!first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric())
    {
        return Err("Coin ID must start and end with alphanumeric characters".to_string());
    }

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
        pct.clamp(-100.0, 10000.0)
    }
}

// ==================== Rate Limiting ====================

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
            continue;
        }

        let wait_ms = last_request + MIN_REQUEST_INTERVAL_MS - now;
        let wait_secs = wait_ms.div_ceil(1000);
        log_rate_limit("coingecko_api", wait_secs);
        sleep(Duration::from_millis(wait_ms)).await;
    }
}

// ==================== HTTP Client ====================

/// Creates a secure HTTP client with appropriate settings
fn create_secure_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("Sanctum/1.0")
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Failed to create HTTP client".to_string())
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
        "Failed to fetch cryptocurrency data. Please try again later.".to_string()
    }
}

// ==================== API Functions ====================

/// Fetches cryptocurrency prices from CoinGecko API
pub async fn fetch_crypto_prices(coin_ids: Vec<String>) -> Result<Vec<CryptoAsset>, String> {
    if coin_ids.is_empty() {
        return Ok(Vec::new());
    }

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

    if validated_ids.len() > MAX_COINS_PER_REQUEST {
        return Err(format!(
            "Too many coins requested. Maximum is {}",
            MAX_COINS_PER_REQUEST
        ));
    }

    let ids_param = validated_ids.join(",");
    let url = format!(
        "{}/coins/markets?vs_currency=usd&ids={}&order=market_cap_desc&sparkline=false",
        COINGECKO_API_BASE, ids_param
    );

    log_security_event(
        SecurityEvent::ExternalApiRequest,
        Some(&format!("coins={}", validated_ids.len())),
    );

    let client = create_secure_client()?;
    let response = client.get(&url).send().await.map_err(handle_request_error)?;

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

    let market_data: Vec<CoinGeckoMarketData> = serde_json::from_slice(&body)
        .map_err(|_| "Failed to parse cryptocurrency data".to_string())?;

    if market_data.len() > MAX_COINS_PER_REQUEST * 2 {
        return Err("Unexpected response format".to_string());
    }

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

/// Fetches the current CLP to USD exchange rate using CoinGecko
pub async fn fetch_clp_usd_rate() -> Result<f64, String> {
    enforce_rate_limit().await?;

    let url = format!(
        "{}/simple/price?ids=tether&vs_currencies=clp",
        COINGECKO_API_BASE
    );

    log_security_event(SecurityEvent::ExternalApiRequest, Some("clp_usd_rate"));

    let client = create_secure_client()?;
    let response = client.get(&url).send().await.map_err(handle_request_error)?;

    let status = response.status();
    if !status.is_success() {
        if status.as_u16() == 429 {
            log_security_event(SecurityEvent::ExternalApiRateLimited, Some("coingecko"));
        }
        return Err("Failed to fetch exchange rate".to_string());
    }

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

    if !(100.0..=5000.0).contains(&rate) {
        return Err("Exchange rate out of expected range".to_string());
    }

    Ok(rate)
}

// ==================== Default Data ====================

/// Returns the padding list used for default tickers and privacy obfuscation
pub fn default_price_allowlist() -> Vec<String> {
    PRIVACY_PRESERVING_PRICE_IDS.iter().map(|s| s.to_string()).collect()
}

/// Returns the default ticker IDs shown in the UI
pub fn default_ticker_ids() -> Vec<String> {
    DEFAULT_TICKER_IDS.iter().map(|s| s.to_string()).collect()
}

/// Returns the default coin catalog used for selection and ticker configuration
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
