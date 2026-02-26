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

//! Shared helpers for exchange-specific CSV parsers.
//!
//! Contains timestamp normalisation, currency name mapping, and small
//! utilities that are reused across multiple exchange modules.

use chrono::NaiveDateTime;

// ─── Timestamp parsing ──────────────────────────────────────────────────────

/// Common datetime formats found in exchange CSV exports (most specific first).
const DATETIME_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S%.f", // 2024-01-15 10:30:45.1234 (Kraken)
    "%Y-%m-%d %H:%M:%S",    // 2024-01-15 10:30:45      (most exchanges)
    "%Y-%m-%dT%H:%M:%S%.f", // 2024-01-15T10:30:45.000  (ISO-8601 fractional)
    "%Y-%m-%dT%H:%M:%S",    // 2024-01-15T10:30:45      (ISO-8601)
    "%Y/%m/%d %H:%M:%S",    // 2024/01/15 10:30:45
    "%d-%m-%Y %H:%M:%S",    // 15-01-2024 10:30:45
    "%m/%d/%Y %H:%M:%S",    // 01/15/2024 10:30:45
];

const DATE_ONLY_FORMATS: &[&str] = &[
    "%Y-%m-%d", // 2024-01-15
    "%Y/%m/%d", // 2024/01/15
    "%d-%m-%Y", // 15-01-2024
    "%m/%d/%Y", // 01/15/2024
];

/// Parses a timestamp string into a `NaiveDateTime`, trying several common
/// formats. Returns `None` if none of them match.
pub fn parse_timestamp(raw: &str) -> Option<NaiveDateTime> {
    let trimmed = raw.trim().trim_matches('"');
    if trimmed.is_empty() {
        return None;
    }

    // Strip trailing 'Z' (UTC indicator) — NaiveDateTime has no timezone info,
    // and Feather Wallet exports ISO-8601 dates like "2020-11-30T21:21:10Z".
    let trimmed = trimmed.trim_end_matches('Z');

    for fmt in DATETIME_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            return Some(dt);
        }
    }

    // Fall back to date-only formats (midnight)
    for fmt in DATE_ONLY_FORMATS {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, fmt) {
            return d.and_hms_opt(0, 0, 0);
        }
    }

    None
}

/// Formats a `NaiveDateTime` as an ISO-8601 date-only string (`YYYY-MM-DD`).
pub fn format_date(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%d").to_string()
}

/// Formats a `NaiveDateTime` as a full ISO-8601 datetime (`YYYY-MM-DD HH:MM:SS`).
pub fn format_datetime(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

// ─── Kraken currency normalisation ──────────────────────────────────────────

/// Normalises Kraken's proprietary asset names to standard tickers.
///
/// Kraken prefixes crypto assets with `X` and fiat with `Z`, and uses `XBT`
/// instead of `BTC`. Staked variants carry a `.S` or `.M` suffix.
///
/// See <https://www.reddit.com/r/KrakenSupport/comments/1i5vwd0/comment/m873xad/>
pub fn normalize_kraken_currency(raw: &str) -> &str {
    // Strip staking/margin suffixes first (e.g. "ETH2.S" -> "ETH2", "DOT.S" -> "DOT")
    let base = match raw.split_once('.') {
        Some((prefix, _suffix)) => prefix,
        None => raw,
    };

    match base {
        "KFEE" => "FEE",
        "XETC" => "ETC",
        "XETH" => "ETH",
        "XLTC" => "LTC",
        "XMLN" => "MLN",
        "XREP" => "REP",
        "XXBT" | "XBT" => "BTC",
        "XXDG" | "XDG" => "DOGE",
        "XXLM" => "XLM",
        "XXMR" => "XMR",
        "XXRP" => "XRP",
        "XZEC" => "ZEC",
        "ZAUD" => "AUD",
        "ZCAD" => "CAD",
        "ZEUR" => "EUR",
        "ZGBP" => "GBP",
        "ZJPY" => "JPY",
        "ZUSD" => "USD",
        other => other,
    }
}

/// Returns `true` if the (already-normalised) ticker represents a fiat
/// currency that Sanctum should skip when importing crypto transactions.
pub fn is_fiat(symbol: &str) -> bool {
    matches!(
        symbol,
        "USD" | "EUR" | "GBP" | "JPY" | "CAD" | "AUD" | "CLP" | "ARS" | "BRL" | "MXN" | "FEE"
    )
}

/// Returns `true` for common USD-pegged stablecoins.
///
/// These are **not** fiat — they are tracked crypto assets — but they serve
/// as quote / pricing currencies on most exchanges.  When one side of a
/// trade is a stablecoin and the other is a regular crypto asset, we
/// classify the trade as a buy/sell (with `price_per_coin` ≈ USD) instead
/// of a swap, which avoids phantom stablecoin balances the user never
/// explicitly deposited.
pub fn is_stablecoin(symbol: &str) -> bool {
    matches!(
        symbol,
        "USDT"
            | "USDC"
            | "BUSD"
            | "DAI"
            | "TUSD"
            | "FDUSD"
            | "USDD"
            | "USDP"
            | "PYUSD"
            | "UST"
            | "FRAX"
    )
}

/// Returns `true` if the symbol acts as a quote / pricing currency —
/// either real fiat **or** a USD-pegged stablecoin.
///
/// Use this (instead of `is_fiat`) when deciding whether a trade leg is
/// the "money" side (buy/sell) vs the "asset" side.  `is_fiat` should
/// still be used when the question is "should we skip this row entirely?"
/// (e.g. fiat-only deposits/withdrawals).
pub fn is_quote_currency(symbol: &str) -> bool {
    is_fiat(symbol) || is_stablecoin(symbol)
}

/// Returns `true` when a quote/fee currency can be treated as USD-valued in
/// the current tax pipeline.
///
/// At the moment, tax valuation expects USD-denominated price inputs.
/// We therefore accept:
/// - `USD` fiat
/// - USD-pegged stablecoins
///
/// Other fiat currencies (EUR, CLP, etc.) must be normalized elsewhere
/// before they can be used as `price_per_coin`/fiat fee in taxes.
pub fn is_usd_valued_quote(symbol: &str) -> bool {
    symbol.eq_ignore_ascii_case("USD") || is_stablecoin(symbol)
}

/// Appends a structured tax annotation indicating that price normalization to
/// USD could not be derived from the exchange quote currency.
pub fn append_tax_non_usd_quote_reason(
    notes: Option<String>,
    quote_symbol: &str,
) -> Option<String> {
    let normalized_quote = quote_symbol.trim().to_uppercase();
    let marker = if normalized_quote.is_empty() {
        "tax_reason=non_usd_quote".to_string()
    } else {
        format!("tax_reason=non_usd_quote:{normalized_quote}")
    };

    match notes {
        Some(mut existing) => {
            if !existing.contains("tax_reason=non_usd_quote") {
                if !existing.trim().is_empty() {
                    existing.push_str(" | ");
                }
                existing.push_str(&marker);
            }
            Some(existing)
        }
        None => Some(marker),
    }
}

// ─── Binance currency normalisation ─────────────────────────────────────────

/// Normalises Binance-specific asset names.
///
/// * `BCC` was Binance's ticker for Bitcoin Cash before they switched to `BCH`.
/// * `NANO` was renamed to `XNO`.
/// * `LUNA` should become `LUNC` for transactions before the Terra 2.0 rename
///   (handled by the caller with a date check).
pub fn normalize_binance_currency(raw: &str) -> &str {
    match raw.trim() {
        "BCC" => "BCH",
        "NANO" => "XNO",
        other => other,
    }
}

/// Checks whether a Binance `LUNA` entry should be mapped to `LUNC`.
/// The rename happened between 2022-05-26 and 2022-05-30; we use 2022-05-27
/// as the cutoff consistent with Binance's announcement.
pub fn should_rename_luna_to_lunc(dt: NaiveDateTime) -> bool {
    let cutoff = chrono::NaiveDate::from_ymd_opt(2022, 5, 27).and_then(|d| d.and_hms_opt(0, 0, 0));
    match cutoff {
        Some(c) => dt < c,
        None => false,
    }
}

// ─── Amount / field parsing helpers ─────────────────────────────────────────

/// Parses a decimal string into `f64`, handling both `.` and `,` as decimal
/// separators and stripping whitespace/quotes.
pub fn parse_decimal(raw: &str) -> Option<f64> {
    let cleaned = raw.trim().trim_matches('"').replace(',', ".");
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

/// Parses a Binance amount+currency string like `"0.001BNB"` or `"25000 USDT"`.
/// Returns `(amount, currency)`.
pub fn parse_amount_with_unit(raw: &str) -> Option<(f64, String)> {
    let trimmed = raw.trim().trim_matches('"');
    if trimmed.is_empty() {
        return None;
    }

    // Find the boundary between digits and the currency code.
    // The currency code always starts with an ASCII letter.
    let first_alpha = trimmed.find(|c: char| c.is_ascii_alphabetic())?;
    let (qty_str, currency) = trimmed.split_at(first_alpha);
    let qty_str = qty_str.trim();
    let currency = currency.trim();

    if qty_str.is_empty() || currency.is_empty() {
        return None;
    }

    let quantity: f64 = qty_str.replace(',', ".").parse().ok()?;
    Some((quantity, currency.to_uppercase()))
}

/// Extracts base and quote currencies from a Kraken pair string.
///
/// Kraken uses two formats:
/// - Slash-separated: `"BTC/USD"`, `"ETH/EUR"`
/// - Concatenated with X/Z prefixes: `"XXBTZUSD"`, `"XETHZEUR"`
///
/// Returns `(base_normalised, quote_normalised)`.
pub fn parse_kraken_pair(pair: &str) -> Option<(String, String)> {
    let trimmed = pair.trim().trim_matches('"');

    // Try slash-separated first
    if let Some((base, quote)) = trimmed.split_once('/') {
        let b = normalize_kraken_currency(base.trim());
        let q = normalize_kraken_currency(quote.trim());
        return Some((b.to_string(), q.to_string()));
    }

    // Try concatenated form: find a known fiat prefix boundary
    // The heuristic: fiat codes are prefixed with Z (ZUSD, ZEUR, etc.)
    // Crypto codes are prefixed with X (XXBT, XETH, etc.)
    // We look for a Z-prefixed fiat in the latter portion.
    let fiat_prefixes = ["ZUSD", "ZEUR", "ZGBP", "ZJPY", "ZCAD", "ZAUD"];
    for prefix in fiat_prefixes {
        if let Some(pos) = trimmed.find(prefix)
            && pos > 0
        {
            let base_raw = &trimmed[..pos];
            let quote_raw = &trimmed[pos..];
            let b = normalize_kraken_currency(base_raw);
            let q = normalize_kraken_currency(quote_raw);
            return Some((b.to_string(), q.to_string()));
        }
    }

    // Last resort: try splitting crypto pairs (e.g. "XETHXXBT")
    // Look for double-X pattern as separator
    let crypto_prefixes = [
        "XXBT", "XETH", "XLTC", "XXMR", "XXRP", "XXLM", "XZEC", "XETC",
    ];
    for prefix in crypto_prefixes {
        if trimmed.starts_with(prefix) && trimmed.len() > prefix.len() {
            let base_raw = prefix;
            let quote_raw = &trimmed[prefix.len()..];
            let b = normalize_kraken_currency(base_raw);
            let q = normalize_kraken_currency(quote_raw);
            return Some((b.to_string(), q.to_string()));
        }
    }

    // Ultra last resort: split in half for shorter pairs like "ADAUSD"
    // Try known 3-4 letter fiat suffixes
    let fiat_suffixes = ["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "USDT", "USDC"];
    for suffix in fiat_suffixes {
        if trimmed.ends_with(suffix) && trimmed.len() > suffix.len() {
            let base_raw = &trimmed[..trimmed.len() - suffix.len()];
            return Some((base_raw.to_uppercase(), suffix.to_uppercase()));
        }
    }

    None
}

/// Returns a non-empty trimmed string, or `None`.
pub fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim().trim_matches('"');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
