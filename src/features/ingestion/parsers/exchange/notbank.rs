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

//! NotBank (ex-CryptoMarket) CSV parsers.
//!
//! Supported exports:
//! - `Transaction` report
//! - `Trade Activity` report
//! - `Profit And Loss` report (informational only; no transaction rows)

use std::collections::HashMap;

use csv::StringRecord;

use super::common::{
    append_tax_non_usd_quote_reason, format_datetime, is_fiat, parse_decimal, parse_timestamp,
};
use super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

fn get_field<'a>(record: &'a StringRecord, cols: &HashMap<&str, usize>, name: &str) -> &'a str {
    cols.get(name)
        .and_then(|&i| record.get(i))
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or("")
}

fn parse_non_negative_decimal(raw: &str) -> Result<f64, ()> {
    match parse_decimal(raw) {
        Some(v) if v >= 0.0 => Ok(v),
        Some(_) => Err(()),
        None if raw.trim().is_empty() => Ok(0.0),
        None => Err(()),
    }
}

fn normalize_symbol(raw: &str) -> String {
    raw.trim().trim_matches('"').to_uppercase()
}

fn maybe_symbol(raw: &str) -> Option<String> {
    let token = normalize_symbol(raw);
    if token.is_empty() || token.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if token.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some(token)
    } else {
        None
    }
}

fn parse_compact_pair(raw: &str) -> Option<(String, String)> {
    let trimmed = raw.trim().trim_matches('"');
    if trimmed.is_empty() {
        return None;
    }

    for sep in ['/', '-', '_'] {
        if let Some((left, right)) = trimmed.split_once(sep) {
            let base = normalize_symbol(left);
            let quote = normalize_symbol(right);
            if !base.is_empty() && !quote.is_empty() {
                return Some((base, quote));
            }
        }
    }

    let upper = normalize_symbol(trimmed);
    const QUOTE_SUFFIXES: &[&str] = &[
        "USDT", "USDC", "BUSD", "FDUSD", "TUSD", "USDD", "USDP", "DAI", "CLP", "ARS", "BRL", "MXN",
        "USD", "EUR", "GBP", "JPY", "BTC", "ETH", "LTC", "XMR",
    ];

    for quote in QUOTE_SUFFIXES {
        if let Some(base_raw) = upper.strip_suffix(quote)
            && !base_raw.is_empty()
            && base_raw.chars().all(|c| c.is_ascii_alphanumeric())
        {
            return Some((base_raw.to_string(), (*quote).to_string()));
        }
    }
    None
}

mod pnl;
mod trade;
mod transaction;

pub use pnl::NotBankPnlParser;
pub use trade::NotBankTradeParser;
pub use transaction::NotBankTransactionParser;
