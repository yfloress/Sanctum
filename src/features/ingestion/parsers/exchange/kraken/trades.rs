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

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::*;

// ═══════════════════════════════════════════════════════════════════════════
//  Kraken Trades Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct KrakenTradesParser;

/// Resolves column indices for the Kraken trades CSV.
fn resolve_trades_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"').to_lowercase();
        match key.as_str() {
            "txid" => {
                map.insert("txid", i);
            }
            "ordertxid" => {
                map.insert("ordertxid", i);
            }
            "pair" => {
                map.insert("pair", i);
            }
            "time" => {
                map.insert("time", i);
            }
            "type" => {
                map.insert("type", i);
            }
            "ordertype" => {
                map.insert("ordertype", i);
            }
            "price" => {
                map.insert("price", i);
            }
            "cost" => {
                map.insert("cost", i);
            }
            "fee" => {
                map.insert("fee", i);
            }
            "vol" => {
                map.insert("vol", i);
            }
            "margin" => {
                map.insert("margin", i);
            }
            _ => {}
        }
    }
    map
}

impl ExchangeParser for KrakenTradesParser {
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

        let cols = resolve_trades_columns(&headers);

        for required in &["pair", "time", "type", "cost", "fee", "vol"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required Kraken trades column: '{}'", required),
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

            let pair_raw = get_field(&record, &cols, "pair");
            let time_raw = get_field(&record, &cols, "time");
            let type_raw = get_field(&record, &cols, "type");
            let cost_raw = get_field(&record, &cols, "cost");
            let fee_raw = get_field(&record, &cols, "fee");
            let vol_raw = get_field(&record, &cols, "vol");
            let txid = get_field(&record, &cols, "txid").to_string();
            let ordertxid = get_field(&record, &cols, "ordertxid").to_string();

            let time = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("time"),
                        format!("Invalid timestamp: '{}'", time_raw),
                    ));
                    continue;
                }
            };

            let (base, quote) = match parse_kraken_pair(pair_raw) {
                Some(pair) => pair,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("pair"),
                        format!("Cannot parse pair: '{}'", pair_raw),
                    ));
                    continue;
                }
            };

            // Parse volume (base asset amount).
            // Negative values are accepted and normalised — exchange exports
            // covering a partial window of an account's history can
            // legitimately contain negative figures.
            let volume = match parse_decimal(vol_raw) {
                Some(v) if v.abs() > 0.0 => v.abs(),
                Some(_) => {
                    // Zero after abs — skip silently
                    continue;
                }
                _ => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("vol"),
                        format!("Invalid volume: '{}'", vol_raw),
                    ));
                    continue;
                }
            };

            // Normalise cost and fee to absolute values for the same reason.
            let cost = parse_decimal(cost_raw).map(|v| v.abs()).unwrap_or(0.0);
            let fee = parse_decimal(fee_raw).map(|v| v.abs()).unwrap_or(0.0);

            let is_buy = type_raw.eq_ignore_ascii_case("buy");
            let side = if is_buy { "buy" } else { "sell" };
            let date = format_datetime(time);

            let base_fiat = is_fiat(&base);
            let quote_fiat = is_fiat(&quote);

            // Only true fiat should be treated as the pricing side for
            // buy/sell. Stablecoins remain crypto so balances are tracked
            // through swaps.
            let quote_is_pricing = quote_fiat;
            let base_is_pricing = base_fiat;

            let notes = match (txid.is_empty(), ordertxid.is_empty()) {
                (false, false) => Some(format!(
                    "Kraken trade | {} | Ref: {} | Order: {}",
                    pair_raw, txid, ordertxid
                )),
                (false, true) => Some(format!("Kraken trade | {} | Ref: {}", pair_raw, txid)),
                (true, false) => Some(format!(
                    "Kraken trade | {} | Order: {}",
                    pair_raw, ordertxid
                )),
                (true, true) => Some(format!("Kraken trade | {}", pair_raw)),
            };

            // Determine the crypto asset, subtype, and amounts
            if base_fiat && quote_fiat {
                // Fiat-to-fiat trade — skip
                continue;
            }

            if quote_is_pricing && !base_is_pricing {
                // Standard fiat pair: BTC/USD, ETH/EUR, etc.
                let price = if volume > 0.0 {
                    Some(cost / volume)
                } else {
                    None
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: base.clone(),
                    transaction_type: "trade".to_string(),
                    amount: volume,
                    subtype: Some(side.to_string()),
                    price_per_coin: price,
                    fee: if fee.abs() > f64::EPSILON {
                        Some(fee)
                    } else {
                        None
                    },
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
                // Inverted fiat pair: USD/BTC (rare but possible)
                let subtype_str = if is_buy { "sell" } else { "buy" };
                let price = if cost > 0.0 {
                    Some(volume / cost)
                } else {
                    None
                };
                let (fee_coin_symbol, fee_amount) = if fee.abs() > f64::EPSILON {
                    (Some(quote.clone()), Some(fee))
                } else {
                    (None, None)
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: quote.clone(),
                    transaction_type: "trade".to_string(),
                    amount: cost,
                    subtype: Some(subtype_str.to_string()),
                    price_per_coin: price,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol,
                    fee_amount,
                    notes,
                };

                result.items.push((line_number, tx));
            } else {
                // Crypto-to-crypto pair — swap (includes stablecoin-to-stablecoin)

                // Guard: skip same-symbol pairs (shouldn't happen with valid
                // Kraken data, but prevents invalid swap X→X if it does).
                if base == quote {
                    continue;
                }

                let (from_symbol, from_amount, to_symbol, to_amount) = if is_buy {
                    // Buying base with quote: outgoing=quote, incoming=base
                    (quote.clone(), cost, base.clone(), volume)
                } else {
                    // Selling base for quote: outgoing=base, incoming=quote
                    (base.clone(), volume, quote.clone(), cost)
                };

                let fee_coin_symbol = if fee.abs() > f64::EPSILON {
                    Some(quote.clone())
                } else {
                    None
                };
                let fee_amount = if fee.abs() > f64::EPSILON {
                    Some(fee)
                } else {
                    None
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
                    fee_coin_symbol,
                    fee_amount,
                    notes,
                };

                result.items.push((line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::KrakenTrades
    }
}
