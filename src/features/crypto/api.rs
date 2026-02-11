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
use reqwest::{Client, Proxy, Url};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;

// ==================== Constants ====================

/// CoinGecko API base URL (free tier, no API key required)
const COINGECKO_API_BASE: &str = "https://api.coingecko.com/api/v3";
/// Mindicador API base URL (public Chilean market indicators)
const MINDICADOR_API_BASE: &str = "https://mindicador.cl/api";
/// Primary public currency API (USD base table, no API key required)
const CURRENCY_API_PRIMARY_URL: &str = "https://latest.currency-api.pages.dev/v1/currencies/usd.json";
/// Mirror endpoint for the same currency dataset
const CURRENCY_API_MIRROR_URL: &str =
    "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/usd.json";

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

/// Maximum allowed length for proxy URLs
pub const MAX_PROXY_URL_LENGTH: usize = 2048;

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

/// Internal struct for simple price response (used for USD fallback via USDT quote)
#[derive(Debug, Deserialize)]
struct SimplePriceResponse {
    tether: Option<HashMap<String, f64>>,
}

/// Mindicador response for USD observed value in CLP.
#[derive(Debug, Deserialize)]
struct MindicadorDollarResponse {
    valor: Option<f64>,
    serie: Option<Vec<MindicadorSeriesPoint>>,
}

#[derive(Debug, Deserialize)]
struct MindicadorSeriesPoint {
    valor: Option<f64>,
}

/// Public currency API response (base USD).
#[derive(Debug, Deserialize)]
struct CurrencyApiUsdResponse {
    usd: Option<HashMap<String, f64>>,
}

// ==================== Proxy Configuration ====================

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub url: String,
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
pub fn sanitize_api_string(input: &str) -> String {
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
pub fn validate_price(price: f64) -> f64 {
    if price.is_nan() || price.is_infinite() || price < 0.0 {
        0.0
    } else if price > 1e15 {
        1e15
    } else {
        price
    }
}

/// Validates percentage change is within reasonable bounds
pub fn validate_percentage(pct: f64) -> f64 {
    if pct.is_nan() || pct.is_infinite() {
        0.0
    } else {
        pct.clamp(-100.0, 10000.0)
    }
}

/// Validates a proxy URL and returns a normalized string
pub fn validate_proxy_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Proxy URL cannot be empty".to_string());
    }

    if trimmed.len() > MAX_PROXY_URL_LENGTH {
        return Err("Proxy URL is too long".to_string());
    }

    let parsed = Url::parse(trimmed).map_err(|_| "Invalid proxy URL".to_string())?;
    match parsed.scheme() {
        "http" | "https" | "socks5" | "socks5h" => {}
        _ => {
            return Err(
                "Proxy URL must use http://, https://, socks5://, or socks5h://".to_string(),
            )
        }
    }

    if parsed.host_str().is_none() {
        return Err("Proxy URL must include a host".to_string());
    }

    Ok(trimmed.to_string())
}

fn validate_fiat_currency_code(raw: &str) -> Result<String, String> {
    let normalized = raw.trim().to_uppercase();
    if normalized.len() != 3 || !normalized.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("Invalid fiat currency code".to_string());
    }
    Ok(normalized)
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
fn create_secure_client(proxy: Option<&ProxyConfig>) -> Result<Client, String> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("Sanctum/1.0")
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none());

    if let Some(proxy_cfg) = proxy {
        let proxy_url = validate_proxy_url(&proxy_cfg.url)?;
        let proxy = Proxy::all(proxy_url).map_err(|_| "Invalid proxy URL".to_string())?;
        // If the user configured a proxy inside Sanctum, force using that route only.
        // If no in-app proxy is configured, we keep default reqwest behavior (env/system proxy).
        builder = builder.no_proxy().proxy(proxy);
    }

    builder
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
pub async fn fetch_crypto_prices(
    coin_ids: Vec<String>,
    proxy: Option<&ProxyConfig>,
) -> Result<Vec<CryptoAsset>, String> {
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

    let client = create_secure_client(proxy)?;
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

/// Fetches the current USD/target fiat exchange rate.
/// Primary source is Mindicador for CLP and Currency API for all other fiat targets.
/// CoinGecko USDT quote is used as fallback for all currencies.
pub async fn fetch_usd_fx_rate(
    target_currency: &str,
    proxy: Option<&ProxyConfig>,
) -> Result<f64, String> {
    let target = validate_fiat_currency_code(target_currency)?;
    if target == "USD" {
        return Ok(1.0);
    }

    enforce_rate_limit().await?;

    log_security_event(
        SecurityEvent::ExternalApiRequest,
        Some(&format!("usd_fx_rate={}", target)),
    );

    let client = create_secure_client(proxy)?;

    if target == "CLP" {
        if let Ok(rate) = fetch_clp_rate_from_mindicador(&client).await {
            return Ok(rate);
        }
    } else if let Ok(rate) = fetch_usd_rate_from_currency_api(&client, &target).await {
        return Ok(rate);
    }

    // Fallback source: CoinGecko USDT/fiat approximation.
    log_security_event(
        SecurityEvent::ExternalApiRequest,
        Some(&format!("usd_fx_rate_usdt_fallback={}", target)),
    );
    enforce_rate_limit().await?;
    fetch_usd_rate_from_usdt(&client, &target).await
}

async fn fetch_clp_rate_from_mindicador(client: &Client) -> Result<f64, String> {
    let url = format!("{}/dolar", MINDICADOR_API_BASE);
    let response = client.get(&url).send().await.map_err(handle_request_error)?;
    let status = response.status();
    if !status.is_success() {
        if status.as_u16() == 429 {
            log_security_event(SecurityEvent::ExternalApiRateLimited, Some("mindicador"));
        }
        return Err("Failed to fetch exchange rate".to_string());
    }

    let body = download_response_body(response).await?;
    parse_mindicador_rate(&body)
}

async fn fetch_usd_rate_from_currency_api(client: &Client, target: &str) -> Result<f64, String> {
    let mut parse_error: Option<String> = None;
    for url in [CURRENCY_API_PRIMARY_URL, CURRENCY_API_MIRROR_URL] {
        let response = match client.get(url).send().await {
            Ok(res) => res,
            Err(err) => {
                parse_error = Some(handle_request_error(err));
                continue;
            }
        };
        if !response.status().is_success() {
            continue;
        }

        let body = match download_response_body(response).await {
            Ok(bytes) => bytes,
            Err(err) => {
                parse_error = Some(err);
                continue;
            }
        };

        match parse_currency_api_rate(&body, target) {
            Ok(rate) => return Ok(rate),
            Err(err) => parse_error = Some(err),
        }
    }

    Err(parse_error.unwrap_or_else(|| "Failed to fetch exchange rate".to_string()))
}

async fn fetch_usd_rate_from_usdt(client: &Client, target: &str) -> Result<f64, String> {
    let url = format!(
        "{}/simple/price?ids=tether&vs_currencies={}",
        COINGECKO_API_BASE,
        target.to_lowercase()
    );
    let response = client.get(&url).send().await.map_err(handle_request_error)?;
    let status = response.status();
    if !status.is_success() {
        if status.as_u16() == 429 {
            log_security_event(SecurityEvent::ExternalApiRateLimited, Some("coingecko"));
        }
        return Err("Failed to fetch exchange rate".to_string());
    }

    let body = download_response_body(response).await?;
    parse_usdt_fiat_rate(&body, target)
}

async fn download_response_body(response: reqwest::Response) -> Result<Vec<u8>, String> {
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

    Ok(body)
}

fn parse_usdt_fiat_rate(body: &[u8], target: &str) -> Result<f64, String> {
    let price_data: SimplePriceResponse =
        serde_json::from_slice(body).map_err(|_| "Failed to parse exchange rate data")?;

    let rate = price_data
        .tether
        .and_then(|quotes| quotes.get(&target.to_lowercase()).copied())
        .ok_or("Requested rate not available")?;

    validate_exchange_rate_range(rate, target)
}

fn parse_currency_api_rate(body: &[u8], target: &str) -> Result<f64, String> {
    let parsed: CurrencyApiUsdResponse =
        serde_json::from_slice(body).map_err(|_| "Failed to parse exchange rate data")?;

    let rate = parsed
        .usd
        .and_then(|rates| rates.get(&target.to_lowercase()).copied())
        .ok_or("Requested rate not available")?;

    validate_exchange_rate_range(rate, target)
}

fn parse_mindicador_rate(body: &[u8]) -> Result<f64, String> {
    let parsed: MindicadorDollarResponse =
        serde_json::from_slice(body).map_err(|_| "Failed to parse exchange rate data")?;

    let rate = parsed.valor.or_else(|| {
        parsed
            .serie
            .and_then(|serie| serie.into_iter().find_map(|point| point.valor))
    });

    let rate = rate.ok_or("CLP rate not available")?;
    validate_exchange_rate_range(rate, "CLP")
}

fn validate_exchange_rate_range(rate: f64, _target: &str) -> Result<f64, String> {
    if !rate.is_finite() || rate <= 0.0 || rate > 1_000_000.0 {
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


#[cfg(test)]
mod tests;
