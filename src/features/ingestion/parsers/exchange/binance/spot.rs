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

use csv::{ReaderBuilder, Trim};

use super::*;

// ═══════════════════════════════════════════════════════════════════════════
//  Binance Spot Trade History Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct BinanceSpotParser;

impl ExchangeParser for BinanceSpotParser {
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

        let cols = resolve_spot_columns(&headers);

        for required in &["date", "side", "executed", "amount", "fee"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!(
                        "Missing required Binance Spot column: '{}'",
                        match *required {
                            "date" => "Date(UTC)",
                            "side" => "Side",
                            "executed" => "Executed",
                            "amount" => "Amount",
                            "fee" => "Fee",
                            other => other,
                        }
                    ),
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

            let date_raw = get_field(&record, &cols, "date");
            let side_raw = get_field(&record, &cols, "side");
            let executed_raw = get_field(&record, &cols, "executed");
            let amount_raw = get_field(&record, &cols, "amount");
            let fee_raw = get_field(&record, &cols, "fee");
            let pair_raw = get_field(&record, &cols, "pair");

            let timestamp = match parse_timestamp(date_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Date(UTC)"),
                        format!("Invalid timestamp: '{}'", date_raw),
                    ));
                    continue;
                }
            };

            let date = format_datetime(timestamp);
            let is_buy = side_raw.eq_ignore_ascii_case("BUY");

            // Parse Executed (base currency amount, e.g. "0.5BTC").
            // Normalise to absolute value — exchange exports covering a
            // partial window of an account's history can legitimately
            // contain negative figures.
            let (executed_qty, executed_unit) = match parse_amount_with_unit(executed_raw) {
                Some((q, u)) => (q.abs(), u),
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Executed"),
                        format!("Cannot parse executed amount: '{}'", executed_raw),
                    ));
                    continue;
                }
            };

            // Parse Amount (quote currency amount, e.g. "25000USDT").
            // Normalise to absolute value for the same reason.
            let (amount_qty, amount_unit) = match parse_amount_with_unit(amount_raw) {
                Some((q, u)) => (q.abs(), u),
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Amount"),
                        format!("Cannot parse amount: '{}'", amount_raw),
                    ));
                    continue;
                }
            };

            // Parse Fee (e.g. "0.001BNB"). Normalise to absolute value.
            let (fee_qty, fee_unit) = match parse_amount_with_unit(fee_raw) {
                Some((q, u)) => (q.abs(), u),
                None => (0.0, String::new()),
            };

            // Normalise currencies
            let base_symbol = normalise_coin(&executed_unit, timestamp);
            let quote_symbol = normalise_coin(&amount_unit, timestamp);
            let fee_symbol = if !fee_unit.is_empty() {
                normalise_coin(&fee_unit, timestamp)
            } else {
                String::new()
            };

            let base_fiat = is_fiat(&base_symbol);
            let quote_fiat = is_fiat(&quote_symbol);

            // Stablecoin-aware: treat stablecoins as pricing currencies
            let quote_is_pricing = is_quote_currency(&quote_symbol);
            let base_is_pricing = is_quote_currency(&base_symbol);

            let notes = if pair_raw.is_empty() {
                Some(format!(
                    "Binance Spot {} | {}/{}",
                    side_raw, base_symbol, quote_symbol
                ))
            } else {
                Some(format!("Binance Spot {} | {}", side_raw, pair_raw))
            };

            // Determine fee fields
            let (fee_usd, fee_coin_sym, fee_coin_amt) = if fee_qty.abs() > f64::EPSILON {
                if is_fiat(&fee_symbol) {
                    (Some(fee_qty), None, None)
                } else {
                    (None, Some(fee_symbol.clone()), Some(fee_qty))
                }
            } else {
                (None, None, None)
            };

            // Both fiat — skip
            if base_fiat && quote_fiat {
                continue;
            }

            if quote_is_pricing && !base_is_pricing {
                // Standard pair: BTC/USD, BTC/USDT, etc.
                let price = if executed_qty > 0.0 {
                    Some(amount_qty / executed_qty)
                } else {
                    None
                };

                let subtype = if is_buy { "buy" } else { "sell" };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: base_symbol,
                    transaction_type: "trade".to_string(),
                    amount: executed_qty,
                    subtype: Some(subtype.to_string()),
                    price_per_coin: price,
                    fee: fee_usd,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: fee_coin_sym,
                    fee_amount: fee_coin_amt,
                    notes,
                };

                result.items.push((line_number, tx));
            } else if base_is_pricing && !quote_is_pricing {
                // Inverted pair: USD/BTC, USDT/BTC (rare but possible)
                let subtype = if is_buy { "sell" } else { "buy" };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: quote_symbol,
                    transaction_type: "trade".to_string(),
                    amount: amount_qty,
                    subtype: Some(subtype.to_string()),
                    price_per_coin: None,
                    fee: fee_usd,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: fee_coin_sym,
                    fee_amount: fee_coin_amt,
                    notes,
                };

                result.items.push((line_number, tx));
            } else {
                // Crypto-to-crypto swap (includes stablecoin-to-stablecoin)

                // Guard: skip same-symbol pairs
                if base_symbol.eq_ignore_ascii_case(&quote_symbol) {
                    continue;
                }

                let (from_symbol, from_amount, to_symbol, to_amount) = if is_buy {
                    // Buying base with quote: out=quote, in=base
                    (quote_symbol, amount_qty, base_symbol, executed_qty)
                } else {
                    // Selling base for quote: out=base, in=quote
                    (base_symbol, executed_qty, quote_symbol, amount_qty)
                };

                let tx = ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol: from_symbol,
                    transaction_type: "trade".to_string(),
                    amount: from_amount,
                    subtype: Some("swap".to_string()),
                    price_per_coin: None,
                    fee: fee_usd,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: Some(to_symbol),
                    swap_to_amount: Some(to_amount),
                    fee_coin_symbol: fee_coin_sym,
                    fee_amount: fee_coin_amt,
                    notes,
                };

                result.items.push((line_number, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::BinanceSpotTradeHistory
    }
}
