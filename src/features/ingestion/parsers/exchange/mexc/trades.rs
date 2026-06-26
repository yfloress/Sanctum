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

//! MEXC Trade History CSV parser.
//!
//! Expected columns:
//! `UID,Pairs,Time,Side,Filled Price,Executed Amount,Total,Fee,Role`

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::super::common::{
    append_tax_non_usd_quote_reason, format_datetime, is_fiat, is_usd_valued_quote,
    normalize_header, parse_decimal, parse_timestamp,
};
use super::super::{ExchangeParser, ExchangeSource, ParseResult};
use super::spot::parse_mexc_pair;
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub struct MexcTradeParser;

fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = normalize_header(col);
        match key.as_str() {
            "pairs" => {
                map.insert("pairs", i);
            }
            "time" => {
                map.insert("time", i);
            }
            "side" => {
                map.insert("side", i);
            }
            "filledprice" => {
                map.insert("filled_price", i);
            }
            "executedamount" => {
                map.insert("executed_amount", i);
            }
            "total" => {
                map.insert("total", i);
            }
            "fee" => {
                map.insert("fee", i);
            }
            "role" => {
                map.insert("role", i);
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

fn parse_side_is_buy(side: &str) -> Option<bool> {
    let value = side.trim().to_lowercase();
    match value.as_str() {
        "buy" => Some(true),
        "sell" => Some(false),
        _ => None,
    }
}

fn parse_fee_field(raw: &str) -> Option<(String, f64)> {
    let trimmed = raw.trim().trim_matches('"');
    if trimmed.is_empty() || trimmed == "-" || trimmed == "--" {
        return None;
    }

    let split_idx = trimmed
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(trimmed.len());
    let (amount_part, symbol_part) = trimmed.split_at(split_idx);
    let amount = parse_decimal(amount_part)?.abs();
    if amount <= 0.0 {
        return None;
    }

    let symbol = symbol_part.trim().to_uppercase();
    if symbol.is_empty() {
        return None;
    }

    Some((symbol, amount))
}

fn build_notes(pair: &str, side: &str, role: &str) -> Option<String> {
    let mut parts = vec![format!("MEXC trade {side} | {pair}")];
    if !role.trim().is_empty() {
        parts.push(format!("role={}", role.trim()));
    }
    Some(parts.join(" | "))
}

impl ExchangeParser for MexcTradeParser {
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
            .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {e}")))?
            .clone();

        let cols = resolve_columns(&headers);
        for (internal, display) in &[
            ("pairs", "Pairs"),
            ("time", "Time"),
            ("side", "Side"),
            ("filled_price", "Filled Price"),
            ("executed_amount", "Executed Amount"),
            ("total", "Total"),
        ] {
            if !cols.contains_key(internal) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required MEXC column: '{display}'"),
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
                        format!("Invalid CSV record: {err}"),
                    ));
                    continue;
                }
            };

            let line_number = record
                .position()
                .map(|p| p.line())
                .unwrap_or((idx + 2) as u64) as usize;

            let pair_raw = get_field(&record, &cols, "pairs");
            let time_raw = get_field(&record, &cols, "time");
            let side_raw = get_field(&record, &cols, "side");
            let filled_price_raw = get_field(&record, &cols, "filled_price");
            let executed_amount_raw = get_field(&record, &cols, "executed_amount");
            let total_raw = get_field(&record, &cols, "total");
            let fee_raw = get_field(&record, &cols, "fee");
            let role_raw = get_field(&record, &cols, "role");

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Time"),
                        format!("Invalid timestamp: '{time_raw}'"),
                    ));
                    continue;
                }
            };
            let date = format_datetime(timestamp);

            let (base_symbol, quote_symbol) = match parse_mexc_pair(pair_raw) {
                Some(pair) => pair,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Pairs"),
                        format!("Cannot parse trading pair: '{pair_raw}'"),
                    ));
                    continue;
                }
            };

            let is_buy = match parse_side_is_buy(side_raw) {
                Some(value) => value,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Side"),
                        format!("Invalid side value: '{side_raw}'"),
                    ));
                    continue;
                }
            };

            let executed_amount = match parse_decimal(executed_amount_raw) {
                Some(value) if value.abs() > 0.0 => value.abs(),
                Some(_) => continue,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Executed Amount"),
                        format!("Invalid executed amount: '{executed_amount_raw}'"),
                    ));
                    continue;
                }
            };

            let filled_price = parse_decimal(filled_price_raw).map(f64::abs);
            let total_amount = parse_decimal(total_raw).map(f64::abs).unwrap_or(0.0);

            let quote_amount = if total_amount > 0.0 {
                total_amount
            } else if let Some(price) = filled_price {
                executed_amount * price
            } else {
                result.errors.push(RowError::new(
                    line_number,
                    Some("Total"),
                    "Cannot derive quote amount from Total or Filled Price",
                ));
                continue;
            };

            let base_fiat = is_fiat(&base_symbol);
            let quote_fiat = is_fiat(&quote_symbol);
            if base_fiat && quote_fiat {
                continue;
            }

            // Only true fiat pairs are imported as buy/sell. Stablecoin pairs
            // are imported as swaps to keep stablecoin balances consistent.
            let quote_is_pricing = quote_fiat;
            let base_is_pricing = base_fiat;

            let (fee_coin_symbol, fee_amount) = match parse_fee_field(fee_raw) {
                Some((coin, amount)) => (Some(coin), Some(amount)),
                None => (None, None),
            };

            let mut notes = build_notes(pair_raw, side_raw, role_raw);

            if quote_is_pricing && !base_is_pricing {
                let quote_is_usd_valued = is_usd_valued_quote(&quote_symbol);
                let price = if quote_is_usd_valued {
                    filled_price.or_else(|| Some(quote_amount / executed_amount))
                } else {
                    None
                };
                if !quote_is_usd_valued {
                    notes = append_tax_non_usd_quote_reason(notes, &quote_symbol);
                }
                let subtype = if is_buy { "buy" } else { "sell" };

                result.items.push((
                    line_number,
                    ImportCryptoTransaction {
                        date,
                        wallet: wallet_name.to_string(),
                        symbol: base_symbol,
                        transaction_type: "trade".to_string(),
                        amount: executed_amount,
                        subtype: Some(subtype.to_string()),
                        price_per_coin: price,
                        fee: None,
                        override_proceeds: None,
                        override_cost_basis: None,
                        swap_to_symbol: None,
                        swap_to_amount: None,
                        fee_coin_symbol,
                        fee_amount,
                        notes,
                    },
                ));
            } else if base_is_pricing && !quote_is_pricing {
                let subtype = if is_buy { "sell" } else { "buy" };
                if !is_usd_valued_quote(&base_symbol) {
                    notes = append_tax_non_usd_quote_reason(notes, &base_symbol);
                }

                result.items.push((
                    line_number,
                    ImportCryptoTransaction {
                        date,
                        wallet: wallet_name.to_string(),
                        symbol: quote_symbol,
                        transaction_type: "trade".to_string(),
                        amount: quote_amount,
                        subtype: Some(subtype.to_string()),
                        price_per_coin: None,
                        fee: None,
                        override_proceeds: None,
                        override_cost_basis: None,
                        swap_to_symbol: None,
                        swap_to_amount: None,
                        fee_coin_symbol,
                        fee_amount,
                        notes,
                    },
                ));
            } else {
                if base_symbol.eq_ignore_ascii_case(&quote_symbol) {
                    continue;
                }

                let (from_symbol, from_amount, to_symbol, to_amount) = if is_buy {
                    (quote_symbol, quote_amount, base_symbol, executed_amount)
                } else {
                    (base_symbol, executed_amount, quote_symbol, quote_amount)
                };

                result.items.push((
                    line_number,
                    ImportCryptoTransaction {
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
                        fee_coin_symbol,
                        fee_amount,
                        notes,
                    },
                ));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::MexcTradeHistory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "UID,Pairs,Time,Side,Filled Price,Executed Amount,Total,Fee,Role";

    #[test]
    fn buy_trade_with_stablecoin_quote_maps_to_swap_with_fee() {
        let csv = format!(
            "{}\n11111111,LTC_USDT,2025-12-18 22:13:41,Buy,78.04,0.315,24.58860,0.01229430USDT,Taker\n",
            HEADER
        );

        let parser = MexcTradeParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert!((tx.amount - 24.58860).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("LTC"));
        assert!((tx.swap_to_amount.unwrap() - 0.315).abs() < f64::EPSILON);
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("USDT"));
        assert!((tx.fee_amount.unwrap() - 0.01229430).abs() < f64::EPSILON);
    }

    #[test]
    fn sell_trade_with_stablecoin_quote_maps_to_swap() {
        let csv = format!(
            "{}\n11111111,DCR_USDT,2025-11-06 11:57:31,Sell,36.58,0.138,5.045,0.00252250USDT,Taker\n",
            HEADER
        );

        let parser = MexcTradeParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "DCR");
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert!((tx.amount - 0.138).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("USDT"));
        assert!((tx.swap_to_amount.unwrap() - 5.045).abs() < f64::EPSILON);
    }

    #[test]
    fn buy_trade_with_non_usd_fiat_quote_keeps_price_empty_and_marks_note() {
        let csv = format!(
            "{}\n11111111,BTC_EUR,2025-11-06 11:57:31,Buy,90000,0.01,900,0.10EUR,Taker\n",
            HEADER
        );

        let parser = MexcTradeParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert!((tx.amount - 0.01).abs() < f64::EPSILON);
        assert!(tx.price_per_coin.is_none());
        let note = tx.notes.as_deref().unwrap_or_default();
        assert!(note.contains("tax_reason=non_usd_quote:EUR"));
    }

    #[test]
    fn crypto_pair_maps_to_swap() {
        let csv = format!(
            "{}\n11111111,ETH_BTC,2025-01-10 09:00:00,Buy,0.05,2.0,0.1,0.0002BTC,Taker\n",
            HEADER
        );

        let parser = MexcTradeParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.errors.is_empty());
        let tx = &result.items[0].1;
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert_eq!(tx.symbol, "BTC");
        assert!((tx.amount - 0.1).abs() < f64::EPSILON);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("ETH"));
        assert!((tx.swap_to_amount.unwrap() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_side_produces_error() {
        let csv = format!(
            "{}\n11111111,LTC_USDT,2025-12-18 22:13:41,Hold,78.04,0.315,24.58860,0.01229430USDT,Taker\n",
            HEADER
        );

        let parser = MexcTradeParser;
        let result = parser.parse(&csv, "MEXC").unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("Invalid side value"));
    }

    #[test]
    fn parser_source_is_mexc_trade_history() {
        let parser = MexcTradeParser;
        assert_eq!(parser.source(), ExchangeSource::MexcTradeHistory);
    }
}
