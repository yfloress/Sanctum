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

use crate::models::{CryptoAsset, CryptoCatalogCoin};
use crate::security_log::{SecurityEvent, log_rate_limit, log_security_event};
use futures::TryStreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;

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
