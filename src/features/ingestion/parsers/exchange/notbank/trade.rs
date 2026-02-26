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

use csv::{ReaderBuilder, StringRecord, Trim};

use super::*;

pub struct NotBankTradeParser;

fn resolve_trade_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (idx, col) in headers.iter().enumerate() {
        match col.trim().trim_matches('"').to_lowercase().as_str() {
            "transreportdatetime" => {
                map.insert("time", idx);
            }
            "side" => {
                map.insert("side", idx);
            }
            "quantity" => {
                map.insert("quantity", idx);
            }
            "instrument" => {
                map.insert("instrument", idx);
            }
            "price" => {
                map.insert("price", idx);
            }
            "notional" => {
                map.insert("notional", idx);
            }
            "fee" => {
                map.insert("fee", idx);
            }
            "feeproduct" => {
                map.insert("fee_product", idx);
            }
            "transreporttype" => {
                map.insert("report_type", idx);
            }
            "tradeid" => {
                map.insert("trade_id", idx);
            }
            "orderid" => {
                map.insert("order_id", idx);
            }
            "makertaker" => {
                map.insert("maker_taker", idx);
            }
            _ => {}
        }
    }
    map
}

fn parse_side(raw: &str) -> Option<bool> {
    match raw.trim().to_lowercase().as_str() {
        "buy" => Some(true),
        "sell" => Some(false),
        _ => None,
    }
}

fn is_stablecoin_symbol(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_uppercase().as_str(),
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

fn is_usd_symbol(raw: &str) -> bool {
    raw.trim().eq_ignore_ascii_case("USD")
}

fn is_non_usd_fiat_symbol(raw: &str) -> bool {
    is_fiat(raw) && !is_usd_symbol(raw)
}

fn update_quote_to_usd_rate(
    quote_to_usd: &mut HashMap<String, f64>,
    spent_symbol: &str,
    spent_amount: f64,
    received_symbol: &str,
    received_amount: f64,
) {
    let (fiat_symbol, fiat_amount, stable_amount) =
        if is_non_usd_fiat_symbol(spent_symbol) && is_stablecoin_symbol(received_symbol) {
            (spent_symbol, spent_amount, received_amount)
        } else if is_non_usd_fiat_symbol(received_symbol) && is_stablecoin_symbol(spent_symbol) {
            (received_symbol, received_amount, spent_amount)
        } else {
            return;
        };

    if fiat_amount > f64::EPSILON && stable_amount > f64::EPSILON {
        quote_to_usd.insert(
            fiat_symbol.trim().to_ascii_uppercase(),
            stable_amount / fiat_amount,
        );
    }
}

fn convert_fiat_amount_to_usd(
    amount: f64,
    fiat_symbol: &str,
    quote_to_usd: &HashMap<String, f64>,
) -> Option<f64> {
    if amount <= f64::EPSILON {
        return None;
    }

    if is_usd_symbol(fiat_symbol) {
        return Some(amount);
    }

    let key = fiat_symbol.trim().to_ascii_uppercase();
    quote_to_usd.get(&key).map(|rate| amount * *rate)
}

fn normalize_fiat_trade_price_to_usd(
    crypto_symbol: &str,
    fiat_symbol: &str,
    fiat_per_coin: f64,
    quote_to_usd: &HashMap<String, f64>,
) -> Option<f64> {
    if fiat_per_coin <= f64::EPSILON {
        return None;
    }

    if is_usd_symbol(fiat_symbol) {
        return Some(fiat_per_coin);
    }

    // Stablecoins are treated as USD-pegged for tax valuation fallback.
    if is_stablecoin_symbol(crypto_symbol) {
        return Some(1.0);
    }

    let key = fiat_symbol.trim().to_ascii_uppercase();
    quote_to_usd.get(&key).map(|rate| fiat_per_coin * *rate)
}

fn annotate_missing_non_usd_quote(
    notes: Option<String>,
    quote_symbol: &str,
    price_per_coin: Option<f64>,
) -> Option<String> {
    if price_per_coin.is_some() || !is_non_usd_fiat_symbol(quote_symbol) {
        return notes;
    }
    append_tax_non_usd_quote_reason(notes, quote_symbol)
}

#[allow(clippy::too_many_arguments)]
fn build_trade_notes(
    instrument_raw: &str,
    side_raw: &str,
    report_type_raw: &str,
    trade_id_raw: &str,
    order_id_raw: &str,
    maker_taker_raw: &str,
    fee_raw: &str,
    fee_product_raw: &str,
) -> Option<String> {
    let mut parts = vec![format!(
        "NotBank trade | {} {}",
        side_raw.trim(),
        instrument_raw.trim()
    )];
    if !report_type_raw.trim().is_empty() {
        parts.push(format!("report_type={}", report_type_raw.trim()));
    }
    if !trade_id_raw.trim().is_empty() {
        parts.push(format!("trade_id={}", trade_id_raw.trim()));
    }
    if !order_id_raw.trim().is_empty() {
        parts.push(format!("order_id={}", order_id_raw.trim()));
    }
    if !maker_taker_raw.trim().is_empty() {
        parts.push(format!("liquidity={}", maker_taker_raw.trim()));
    }
    if !fee_raw.trim().is_empty() {
        if !fee_product_raw.trim().is_empty() {
            parts.push(format!("fee={} {}", fee_raw.trim(), fee_product_raw.trim()));
        } else {
            parts.push(format!("fee={}", fee_raw.trim()));
        }
    }
    Some(parts.join(" | "))
}

impl ExchangeParser for NotBankTradeParser {
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
        let cols = resolve_trade_columns(&headers);

        for (internal, display) in &[
            ("time", "TransReportDatetime"),
            ("side", "Side"),
            ("quantity", "Quantity"),
            ("instrument", "Instrument"),
            ("price", "Price"),
        ] {
            if !cols.contains_key(internal) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required NotBank Trade Activity column: '{display}'"),
                ));
            }
        }

        let mut result: ParseResult<ImportCryptoTransaction> = ParseResult::default();
        let mut quote_to_usd: HashMap<String, f64> = HashMap::new();

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

            let time_raw = get_field(&record, &cols, "time");
            let side_raw = get_field(&record, &cols, "side");
            let quantity_raw = get_field(&record, &cols, "quantity");
            let instrument_raw = get_field(&record, &cols, "instrument");
            let price_raw = get_field(&record, &cols, "price");
            let notional_raw = get_field(&record, &cols, "notional");
            let fee_raw = get_field(&record, &cols, "fee");
            let fee_product_raw = get_field(&record, &cols, "fee_product");
            let report_type_raw = get_field(&record, &cols, "report_type");
            let trade_id_raw = get_field(&record, &cols, "trade_id");
            let order_id_raw = get_field(&record, &cols, "order_id");
            let maker_taker_raw = get_field(&record, &cols, "maker_taker");

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("TransReportDatetime"),
                        format!("Invalid timestamp: '{time_raw}'"),
                    ));
                    continue;
                }
            };

            let is_buy = match parse_side(side_raw) {
                Some(v) => v,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Side"),
                        format!("Invalid side value: '{side_raw}'"),
                    ));
                    continue;
                }
            };

            let quantity = match parse_decimal(quantity_raw).map(f64::abs) {
                Some(v) if v > 0.0 => v,
                Some(_) => continue,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Quantity"),
                        format!("Invalid quantity: '{quantity_raw}'"),
                    ));
                    continue;
                }
            };

            let (base_symbol, quote_symbol) = match parse_compact_pair(instrument_raw) {
                Some(pair) => pair,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Instrument"),
                        format!("Cannot parse trading pair: '{instrument_raw}'"),
                    ));
                    continue;
                }
            };

            let price = parse_decimal(price_raw).map(f64::abs);
            let notional = parse_decimal(notional_raw).map(f64::abs).unwrap_or(0.0);
            let quote_amount = if notional > 0.0 {
                notional
            } else if let Some(p) = price {
                quantity * p
            } else {
                result.errors.push(RowError::new(
                    line_number,
                    Some("Notional"),
                    "Cannot derive quote amount from Notional or Price",
                ));
                continue;
            };

            let (spent_symbol, spent_amount, received_symbol, received_amount) = if is_buy {
                (
                    quote_symbol.clone(),
                    quote_amount,
                    base_symbol.clone(),
                    quantity,
                )
            } else {
                (
                    base_symbol.clone(),
                    quantity,
                    quote_symbol.clone(),
                    quote_amount,
                )
            };

            if spent_symbol.eq_ignore_ascii_case(&received_symbol) {
                continue;
            }
            if is_fiat(&spent_symbol) && is_fiat(&received_symbol) {
                continue;
            }

            update_quote_to_usd_rate(
                &mut quote_to_usd,
                &spent_symbol,
                spent_amount,
                &received_symbol,
                received_amount,
            );

            let parsed_fee_amount = parse_decimal(fee_raw)
                .map(f64::abs)
                .filter(|v| *v > f64::EPSILON);
            let parsed_fee_symbol = maybe_symbol(fee_product_raw);
            let (fee_symbol, fee_amount) = match (parsed_fee_symbol, parsed_fee_amount) {
                (Some(symbol), Some(amount)) => (Some(symbol), Some(amount)),
                // NotBank often exports numeric product identifiers in FeeProduct.
                // When that happens, infer fee coin from the received side so fees
                // are not silently dropped and balances stay coherent.
                (None, Some(amount)) => (Some(received_symbol.clone()), Some(amount)),
                _ => (None, None),
            };

            let base_notes = build_trade_notes(
                instrument_raw,
                side_raw,
                report_type_raw,
                trade_id_raw,
                order_id_raw,
                maker_taker_raw,
                fee_raw,
                fee_product_raw,
            );

            if is_fiat(&spent_symbol) && !is_fiat(&received_symbol) {
                let price_per_coin = normalize_fiat_trade_price_to_usd(
                    &received_symbol,
                    &spent_symbol,
                    spent_amount / received_amount,
                    &quote_to_usd,
                );
                let notes =
                    annotate_missing_non_usd_quote(base_notes.clone(), &spent_symbol, price_per_coin);
                let mut tx = ImportCryptoTransaction {
                    date: format_datetime(timestamp),
                    wallet: wallet_name.to_string(),
                    symbol: received_symbol.clone(),
                    transaction_type: "trade".to_string(),
                    amount: received_amount,
                    subtype: Some("buy".to_string()),
                    price_per_coin,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                if let (Some(fa), Some(fs)) = (fee_amount, fee_symbol.clone()) {
                    if is_fiat(&fs) {
                        tx.fee = convert_fiat_amount_to_usd(fa, &fs, &quote_to_usd);
                    } else {
                        tx.fee_coin_symbol = Some(fs);
                        tx.fee_amount = Some(fa);
                    }
                }

                result.items.push((line_number, tx));
                continue;
            }

            if !is_fiat(&spent_symbol) && is_fiat(&received_symbol) {
                let price_per_coin = normalize_fiat_trade_price_to_usd(
                    &spent_symbol,
                    &received_symbol,
                    received_amount / spent_amount,
                    &quote_to_usd,
                );
                let notes = annotate_missing_non_usd_quote(
                    base_notes.clone(),
                    &received_symbol,
                    price_per_coin,
                );
                let mut tx = ImportCryptoTransaction {
                    date: format_datetime(timestamp),
                    wallet: wallet_name.to_string(),
                    symbol: spent_symbol.clone(),
                    transaction_type: "trade".to_string(),
                    amount: spent_amount,
                    subtype: Some("sell".to_string()),
                    price_per_coin,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                };

                if let (Some(fa), Some(fs)) = (fee_amount, fee_symbol.clone()) {
                    if is_fiat(&fs) {
                        tx.fee = convert_fiat_amount_to_usd(fa, &fs, &quote_to_usd);
                    } else {
                        tx.fee_coin_symbol = Some(fs);
                        tx.fee_amount = Some(fa);
                    }
                }

                result.items.push((line_number, tx));
                continue;
            }

            result.items.push((
                line_number,
                ImportCryptoTransaction {
                    date: format_datetime(timestamp),
                    wallet: wallet_name.to_string(),
                    symbol: spent_symbol,
                    transaction_type: "trade".to_string(),
                    amount: spent_amount,
                    subtype: Some("swap".to_string()),
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: Some(received_symbol),
                    swap_to_amount: Some(received_amount),
                    fee_coin_symbol: fee_symbol.clone(),
                    fee_amount,
                    notes: base_notes,
                },
            ));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::NotBankTradeActivity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trade_parser_maps_fiat_pair_as_buy() {
        let csv = "\"RegisteredEntityId\",\"TransReportId\",\"TransReportRevision\",\"TransReportType\",\"OrderId\",\"ClientOrderId\",\"QuoteId\",\"ExtTradeReportId\",\"TradeId\",\"TransReportDatetime\",\"Side\",\"Quantity\",\"Instrument\",\"Price\",\"InsideBid\",\"InsideBidSize\",\"InsideOffer\",\"InsideOfferSize\",\"LeavesSize\",\"MakerTaker\",\"Trader\",\"AccountId\",\"AccountName\",\"Fee\",\"FeeProduct\",\"Notional\",\"BaseSettlementAmount\",\"CounterpartySettlementAmount\",\"OMSId\"\n\
\"\",\"1\",\"1\",\"QuoteExecution\",\"\",\"\",\"\",\"\",\"1001\",\"2025-10-30T19:53:11.325Z\",\"Buy\",\"100\",\"USDTCLP\",\"945\",\"0\",\"0\",\"0\",\"0\",\"0\",\"Maker\",\"1\",\"100\",\"Primary\",\"0.10\",\"CLP\",\"94500\",\"100\",\"-94500\",\"1\"\n";

        let parser = NotBankTradeParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.amount, 100.0);
        assert_eq!(tx.price_per_coin, Some(1.0));
    }

    #[test]
    fn trade_parser_maps_crypto_pair_as_swap() {
        let csv = "\"RegisteredEntityId\",\"TransReportId\",\"TransReportRevision\",\"TransReportType\",\"OrderId\",\"ClientOrderId\",\"QuoteId\",\"ExtTradeReportId\",\"TradeId\",\"TransReportDatetime\",\"Side\",\"Quantity\",\"Instrument\",\"Price\",\"InsideBid\",\"InsideBidSize\",\"InsideOffer\",\"InsideOfferSize\",\"LeavesSize\",\"MakerTaker\",\"Trader\",\"AccountId\",\"AccountName\",\"Fee\",\"FeeProduct\",\"Notional\",\"BaseSettlementAmount\",\"CounterpartySettlementAmount\",\"OMSId\"\n\
\"\",\"1\",\"1\",\"OrderExecution\",\"\",\"\",\"\",\"\",\"1001\",\"2025-10-30T20:19:39.450Z\",\"Buy\",\"0.5\",\"LTCUSDT\",\"91.2\",\"0\",\"0\",\"0\",\"0\",\"0\",\"Maker\",\"1\",\"100\",\"Primary\",\"0.00038\",\"31\",\"45.6\",\"0.5\",\"-45.6\",\"1\"\n";

        let parser = NotBankTradeParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.amount, 45.6);
        assert_eq!(tx.swap_to_symbol.as_deref(), Some("LTC"));
        assert_eq!(tx.swap_to_amount, Some(0.5));
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("LTC"));
        assert_eq!(tx.fee_amount, Some(0.00038));
    }

    #[test]
    fn trade_parser_swap_drops_orphan_fee_symbol_without_fee_amount() {
        let csv = "\"RegisteredEntityId\",\"TransReportId\",\"TransReportRevision\",\"TransReportType\",\"OrderId\",\"ClientOrderId\",\"QuoteId\",\"ExtTradeReportId\",\"TradeId\",\"TransReportDatetime\",\"Side\",\"Quantity\",\"Instrument\",\"Price\",\"InsideBid\",\"InsideBidSize\",\"InsideOffer\",\"InsideOfferSize\",\"LeavesSize\",\"MakerTaker\",\"Trader\",\"AccountId\",\"AccountName\",\"Fee\",\"FeeProduct\",\"Notional\",\"BaseSettlementAmount\",\"CounterpartySettlementAmount\",\"OMSId\"\n\
\"\",\"1\",\"1\",\"OrderExecution\",\"\",\"\",\"\",\"\",\"1001\",\"2025-10-30T20:19:39.450Z\",\"Buy\",\"0.5\",\"LTCUSDT\",\"91.2\",\"0\",\"0\",\"0\",\"0\",\"0\",\"Maker\",\"1\",\"100\",\"Primary\",\"0\",\"USDT\",\"45.6\",\"0.5\",\"-45.6\",\"1\"\n";

        let parser = NotBankTradeParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.subtype.as_deref(), Some("swap"));
        assert!(tx.fee_coin_symbol.is_none());
        assert!(tx.fee_amount.is_none());
    }

    #[test]
    fn trade_parser_infers_fee_coin_for_numeric_fee_product_on_fiat_buy() {
        let csv = "\"RegisteredEntityId\",\"TransReportId\",\"TransReportRevision\",\"TransReportType\",\"OrderId\",\"ClientOrderId\",\"QuoteId\",\"ExtTradeReportId\",\"TradeId\",\"TransReportDatetime\",\"Side\",\"Quantity\",\"Instrument\",\"Price\",\"InsideBid\",\"InsideBidSize\",\"InsideOffer\",\"InsideOfferSize\",\"LeavesSize\",\"MakerTaker\",\"Trader\",\"AccountId\",\"AccountName\",\"Fee\",\"FeeProduct\",\"Notional\",\"BaseSettlementAmount\",\"CounterpartySettlementAmount\",\"OMSId\"\n\
\"\",\"1\",\"1\",\"QuoteExecution\",\"\",\"\",\"\",\"\",\"1001\",\"2025-10-30T19:53:11.325Z\",\"Buy\",\"100\",\"USDTCLP\",\"945\",\"0\",\"0\",\"0\",\"0\",\"0\",\"Maker\",\"1\",\"100\",\"Primary\",\"0.095\",\"3\",\"94500\",\"100\",\"-94500\",\"1\"\n";

        let parser = NotBankTradeParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert_eq!(tx.symbol, "USDT");
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("USDT"));
        assert_eq!(tx.fee_amount, Some(0.095));
        assert!(tx.fee.is_none());
    }

    #[test]
    fn trade_parser_converts_non_usd_fiat_price_using_stablecoin_anchor() {
        let csv = "\"RegisteredEntityId\",\"TransReportId\",\"TransReportRevision\",\"TransReportType\",\"OrderId\",\"ClientOrderId\",\"QuoteId\",\"ExtTradeReportId\",\"TradeId\",\"TransReportDatetime\",\"Side\",\"Quantity\",\"Instrument\",\"Price\",\"InsideBid\",\"InsideBidSize\",\"InsideOffer\",\"InsideOfferSize\",\"LeavesSize\",\"MakerTaker\",\"Trader\",\"AccountId\",\"AccountName\",\"Fee\",\"FeeProduct\",\"Notional\",\"BaseSettlementAmount\",\"CounterpartySettlementAmount\",\"OMSId\"\n\
\"\",\"1\",\"1\",\"QuoteExecution\",\"\",\"\",\"\",\"\",\"1001\",\"2025-10-30T19:53:11.325Z\",\"Buy\",\"100\",\"USDTCLP\",\"945\",\"0\",\"0\",\"0\",\"0\",\"0\",\"Maker\",\"1\",\"100\",\"Primary\",\"0.10\",\"CLP\",\"94500\",\"100\",\"-94500\",\"1\"\n\
\"\",\"2\",\"1\",\"QuoteExecution\",\"\",\"\",\"\",\"\",\"1002\",\"2025-10-30T20:19:39.450Z\",\"Buy\",\"0.001\",\"BTCCLP\",\"63000000\",\"0\",\"0\",\"0\",\"0\",\"0\",\"Maker\",\"1\",\"100\",\"Primary\",\"0\",\"CLP\",\"63000\",\"0.001\",\"-63000\",\"1\"\n";

        let parser = NotBankTradeParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 2);

        let btc_buy = &result.items[1].1;
        assert_eq!(btc_buy.transaction_type, "trade");
        assert_eq!(btc_buy.subtype.as_deref(), Some("buy"));
        assert_eq!(btc_buy.symbol, "BTC");
        assert!((btc_buy.price_per_coin.unwrap_or_default() - 66666.6667).abs() < 0.01);
        let note = btc_buy.notes.as_deref().unwrap_or_default();
        assert!(!note.contains("tax_reason=non_usd_quote"));
    }

    #[test]
    fn trade_parser_leaves_non_usd_fiat_price_empty_without_anchor() {
        let csv = "\"RegisteredEntityId\",\"TransReportId\",\"TransReportRevision\",\"TransReportType\",\"OrderId\",\"ClientOrderId\",\"QuoteId\",\"ExtTradeReportId\",\"TradeId\",\"TransReportDatetime\",\"Side\",\"Quantity\",\"Instrument\",\"Price\",\"InsideBid\",\"InsideBidSize\",\"InsideOffer\",\"InsideOfferSize\",\"LeavesSize\",\"MakerTaker\",\"Trader\",\"AccountId\",\"AccountName\",\"Fee\",\"FeeProduct\",\"Notional\",\"BaseSettlementAmount\",\"CounterpartySettlementAmount\",\"OMSId\"\n\
\"\",\"1\",\"1\",\"QuoteExecution\",\"\",\"\",\"\",\"\",\"1001\",\"2025-10-30T20:19:39.450Z\",\"Buy\",\"0.001\",\"BTCCLP\",\"63000000\",\"0\",\"0\",\"0\",\"0\",\"0\",\"Maker\",\"1\",\"100\",\"Primary\",\"0\",\"CLP\",\"63000\",\"0.001\",\"-63000\",\"1\"\n";

        let parser = NotBankTradeParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert!(tx.price_per_coin.is_none());
        let note = tx.notes.as_deref().unwrap_or_default();
        assert!(note.contains("tax_reason=non_usd_quote:CLP"));
    }

    #[test]
    fn trade_parser_marks_non_usd_quote_when_sell_price_anchor_missing() {
        let csv = "\"RegisteredEntityId\",\"TransReportId\",\"TransReportRevision\",\"TransReportType\",\"OrderId\",\"ClientOrderId\",\"QuoteId\",\"ExtTradeReportId\",\"TradeId\",\"TransReportDatetime\",\"Side\",\"Quantity\",\"Instrument\",\"Price\",\"InsideBid\",\"InsideBidSize\",\"InsideOffer\",\"InsideOfferSize\",\"LeavesSize\",\"MakerTaker\",\"Trader\",\"AccountId\",\"AccountName\",\"Fee\",\"FeeProduct\",\"Notional\",\"BaseSettlementAmount\",\"CounterpartySettlementAmount\",\"OMSId\"\n\
\"\",\"1\",\"1\",\"QuoteExecution\",\"\",\"\",\"\",\"\",\"1001\",\"2025-10-30T20:19:39.450Z\",\"Sell\",\"0.001\",\"BTCCLP\",\"63000000\",\"0\",\"0\",\"0\",\"0\",\"0\",\"Maker\",\"1\",\"100\",\"Primary\",\"0\",\"CLP\",\"63000\",\"0.001\",\"-63000\",\"1\"\n";

        let parser = NotBankTradeParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 1);
        let tx = &result.items[0].1;
        assert_eq!(tx.symbol, "BTC");
        assert_eq!(tx.subtype.as_deref(), Some("sell"));
        assert!(tx.price_per_coin.is_none());
        let note = tx.notes.as_deref().unwrap_or_default();
        assert!(note.contains("tax_reason=non_usd_quote:CLP"));
    }
}
