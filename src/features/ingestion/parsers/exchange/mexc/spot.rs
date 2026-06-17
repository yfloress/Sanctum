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

//! MEXC Spot Trade History CSV parser
//!
//! Handles the CSV export from MEXC's spot trading history.
//!
//! ## Expected columns
//!
//! ```text
//! UID, Pairs, Time, Type, Direction, Average Filled Price, Order Price,
//! Filled Quantity, Order Quantity, Order Amount, Status
//! ```
//!
//! ## Behaviour
//!
//! - Only rows with `Status == "Filled"` are processed; other statuses are
//!   skipped silently.
//! - `Pairs` usually uses separators (`LTC_USDT`, `BTC-USDT`, `ETH/BTC`)
//!   but compact forms like `BTCUSDT` are also accepted.
//! - `Direction` is `Buy` or `Sell`.
//! - `Filled Quantity` is the amount of the **base** asset actually filled.
//! - `Order Amount` is the total in the **quote** currency.
//! - `Average Filled Price` is the execution price.
//! - No fee information is available in this export format.
//! - When `Order Price` is `"Market"`, the order was a market order (noted in
//!   the transaction notes).
//! - If the quote asset is fiat, the transaction is a simple buy/sell trade.
//! - If both assets are crypto, the transaction becomes a swap.
//! - If both assets are fiat, the row is skipped.

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::super::common::{
    append_tax_non_usd_quote_reason, format_datetime, is_fiat, is_usd_valued_quote, parse_decimal,
    parse_timestamp,
};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

// ─── Column resolution ──────────────────────────────────────────────────────

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"').to_lowercase();
        match key.as_str() {
            "pairs" => {
                map.insert("pairs", i);
            }
            "time" => {
                map.insert("time", i);
            }
            "type" => {
                map.insert("type", i);
            }
            "direction" => {
                map.insert("direction", i);
            }
            "average filled price" => {
                map.insert("avg_price", i);
            }
            "order price" => {
                map.insert("order_price", i);
            }
            "filled quantity" => {
                map.insert("filled_qty", i);
            }
            "order amount" => {
                map.insert("order_amount", i);
            }
            "status" => {
                map.insert("status", i);
            }
            _ => {}
        }
    }
    map
}

fn get_field<'a>(record: &'a StringRecord, cols: &HashMap<&str, usize>, name: &str) -> &'a str {
    cols.get(name)
        .and_then(|&i| record.get(i))
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or("")
}

// ─── Pair parsing ───────────────────────────────────────────────────────────

/// Splits a MEXC pair into `(base, quote)`.
///
/// Supported forms:
/// - Separated: `LTC_USDT`, `BTC-USDT`, `ETH/USDC`
/// - Compact: `BTCUSDT`, `ETHBTC` (using known quote suffixes)
pub(super) fn parse_mexc_pair(pair: &str) -> Option<(String, String)> {
    let trimmed = pair.trim();
    if trimmed.is_empty() {
        return None;
    }

    for separator in ['_', '-', '/'] {
        if let Some((left, right)) = trimmed.split_once(separator) {
            let base = left.trim().to_uppercase();
            let quote = right.trim().to_uppercase();
            if !base.is_empty() && !quote.is_empty() {
                return Some((base, quote));
            }
        }
    }

    // Compact pair fallback (e.g. BTCUSDT, ETHBTC)
    let upper = trimmed.to_uppercase();
    const QUOTE_SUFFIXES: &[&str] = &[
        "USDT", "USDC", "BUSD", "FDUSD", "TUSD", "USDD", "USDP", "DAI", "BTC", "ETH", "USD", "EUR",
        "GBP", "JPY", "AUD", "CAD", "MXN", "BRL", "CLP", "ARS",
    ];
    for quote in QUOTE_SUFFIXES {
        if let Some(base_raw) = upper.strip_suffix(quote) {
            let base = base_raw.trim();
            if !base.is_empty() && base.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Some((base.to_string(), (*quote).to_string()));
            }
        }
    }

    None
}

fn parse_direction_is_buy(direction: &str) -> Option<bool> {
    match direction.trim().to_lowercase().as_str() {
        "buy" => Some(true),
        "sell" => Some(false),
        _ => None,
    }
}

// ─── Parser ─────────────────────────────────────────────────────────────────

pub struct MexcSpotParser;

impl ExchangeParser for MexcSpotParser {
    fn parse(
        &self,
        content: &str,
        wallet_name: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError> {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());

        let headers = reader
            .headers()
            .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {}", e)))?
            .clone();

        let cols = resolve_columns(&headers);

        // Validate required columns
        for (internal, display) in &[
            ("pairs", "Pairs"),
            ("time", "Time"),
            ("direction", "Direction"),
            ("filled_qty", "Filled Quantity"),
            ("order_amount", "Order Amount"),
            ("status", "Status"),
        ] {
            if !cols.contains_key(internal) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required MEXC column: '{}'", display),
                ));
            }
        }

        let mut result: ParseResult<ImportCryptoTransaction> = ParseResult::default();

        for (idx, record) in reader.records().enumerate() {
            let record = match record {
                Ok(r) => r,
                Err(err) => {
                    let line = err.position().map(|p| p.line()).unwrap_or((idx + 2) as u64);
                    result.errors.push(RowError::new(
                        line as usize,
                        None,
                        format!("Invalid CSV record: {}", err),
                    ));
                    continue;
                }
            };

            let line_number = record
                .position()
                .map(|p| p.line())
                .unwrap_or((idx + 2) as u64) as usize;

            let status = get_field(&record, &cols, "status");

            // Process orders that had actual fills: "Filled" and "Partially Filled".
            // MEXC exports partially filled orders (order was partially executed
            // then cancelled) with the real `Filled Quantity`. Dropping these
            // loses real trades and causes wrong balances.
            // Cancelled / Pending / Unfilled / other statuses are skipped.
            let status_lower = status.to_lowercase();
            let is_filled = status_lower == "filled" || status_lower.contains("partially filled");
            if !is_filled {
                continue;
            }

            let pair_raw = get_field(&record, &cols, "pairs");
            let time_raw = get_field(&record, &cols, "time");
            let direction_raw = get_field(&record, &cols, "direction");
            let order_type_raw = get_field(&record, &cols, "type");
            let avg_price_raw = get_field(&record, &cols, "avg_price");
            let filled_qty_raw = get_field(&record, &cols, "filled_qty");
            let order_amount_raw = get_field(&record, &cols, "order_amount");

            // Parse timestamp
            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Time"),
                        format!("Invalid timestamp: '{}'", time_raw),
                    ));
                    continue;
                }
            };

            let date = format_datetime(timestamp);

            // Parse pair
            let (base_symbol, quote_symbol) = match parse_mexc_pair(pair_raw) {
                Some(pair) => pair,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Pairs"),
                        format!("Cannot parse trading pair: '{}'", pair_raw),
                    ));
                    continue;
                }
            };

            // Parse filled quantity (base asset amount).
            // Negative values are accepted and normalised — exchange exports
            // covering a partial window of an account's history can
            // legitimately contain negative figures.
            let filled_qty = match parse_decimal(filled_qty_raw) {
                Some(q) if q.abs() > 0.0 => q.abs(),
                Some(_) => {
                    // Zero after abs — skip silently (nothing was filled)
                    continue;
                }
                _ => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Filled Quantity"),
                        format!("Invalid filled quantity: '{}'", filled_qty_raw),
                    ));
                    continue;
                }
            };

            // Parse order amount (quote asset total).  Normalise to absolute
            // value for the same partial-window reason.
            let order_amount = parse_decimal(order_amount_raw)
                .map(|v| v.abs())
                .unwrap_or(0.0);

            // Parse average filled price (normalise to positive)
            let avg_price = parse_decimal(avg_price_raw).map(|p| p.abs());

            let is_buy = match parse_direction_is_buy(direction_raw) {
                Some(v) => v,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Direction"),
                        format!("Invalid direction value: '{}'", direction_raw),
                    ));
                    continue;
                }
            };
            let base_fiat = is_fiat(&base_symbol);
            let quote_fiat = is_fiat(&quote_symbol);

            // Only true fiat pairs (USD/BTC) are treated as buy/sell pricing
            // operations. Stablecoin-quoted pairs (BTC/USDT) are swaps so
            // stablecoin balances are updated correctly.
            let quote_is_pricing = quote_fiat;
            let base_is_pricing = base_fiat;

            let mut notes = Some(format!(
                "MEXC {} {} | {}/{}",
                order_type_raw, direction_raw, base_symbol, quote_symbol,
            ));

            // Compute the actual filled value in the quote currency.
            // Prefer `filled_qty * avg_price` (real execution cost) over
            // `order_amount` (which may be the *intended* order total and
            // can differ for market orders or partial fills).
            let filled_value = if let Some(price) = avg_price {
                let computed = filled_qty * price;
                if computed > 0.0 {
                    computed
                } else {
                    order_amount
                }
            } else {
                order_amount
            };

            // Both true fiat → skip entirely
            if base_fiat && quote_fiat {
                continue;
            }

            // Quote is true fiat and base is crypto -> standard buy/sell.
            if quote_is_pricing && !base_is_pricing {
                let quote_is_usd_valued = is_usd_valued_quote(&quote_symbol);
                let price = if avg_price.is_some() {
                    if quote_is_usd_valued { avg_price } else { None }
                } else if filled_qty > 0.0 && filled_value > 0.0 {
                    if quote_is_usd_valued {
                        Some(filled_value / filled_qty)
                    } else {
                        None
                    }
                } else {
                    None
                };
                if !quote_is_usd_valued {
                    notes = append_tax_non_usd_quote_reason(notes, &quote_symbol);
                }

                let subtype = if is_buy { "buy" } else { "sell" };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: base_symbol,
                    transaction_type: "trade".to_string(),
                    amount: filled_qty,
                    subtype: Some(subtype.to_string()),
                    price_per_coin: price,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                result.items.push((line_number, tx));
            } else if base_is_pricing && !quote_is_pricing {
                // Inverted fiat pair (rare): USD/BTC.
                // Buying base(fiat-like) with quote(crypto) = selling crypto
                let subtype = if is_buy { "sell" } else { "buy" };
                if !is_usd_valued_quote(&base_symbol) {
                    notes = append_tax_non_usd_quote_reason(notes, &base_symbol);
                }

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: quote_symbol,
                    transaction_type: "trade".to_string(),
                    amount: filled_value,
                    subtype: Some(subtype.to_string()),
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                result.items.push((line_number, tx));
            } else {
                // Crypto-to-crypto swap: both regular crypto, or both
                // stablecoins (e.g. USDT_USDC).
                // Guard: skip same-symbol pairs (e.g. BTC_BTC) — would
                // produce an invalid swap X→X that fails validation.
                if base_symbol.eq_ignore_ascii_case(&quote_symbol) {
                    continue;
                }

                let (from_symbol, from_amount, to_symbol, to_amount) = if is_buy {
                    // Buying base with quote: out=quote, in=base
                    (quote_symbol, filled_value, base_symbol, filled_qty)
                } else {
                    // Selling base for quote: out=base, in=quote
                    (base_symbol, filled_qty, quote_symbol, filled_value)
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: from_symbol,
                    transaction_type: "trade".to_string(),
                    amount: from_amount,
                    subtype: Some("swap".to_string()),
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: Some(to_symbol),
                    swap_to_amount: Some(to_amount),
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                result.items.push((line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcSpotTradeHistory
    }
}
