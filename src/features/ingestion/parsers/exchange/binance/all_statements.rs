// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
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

use csv::{ReaderBuilder, Trim};

use super::*;

// ═══════════════════════════════════════════════════════════════════════════
//  Binance All Statements Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct BinanceAllStatementsParser;

impl ExchangeParser for BinanceAllStatementsParser {
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

        let cols = resolve_all_statements_columns(&headers);

        for required in &["utc_time", "operation", "coin", "change"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!(
                        "Missing required Binance column: '{}'",
                        match *required {
                            "utc_time" => "UTC_Time",
                            "operation" => "Operation",
                            "coin" => "Coin",
                            "change" => "Change",
                            other => other,
                        }
                    ),
                ));
            }
        }

        let mut result: ParseResult<ImportCryptoTransaction> = ParseResult::default();

        // Accumulate all rows first so we can pair Convert/SmallAssets rows.
        let mut rows: Vec<BinanceRow> = Vec::new();

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

            let time_raw = get_field(&record, &cols, "utc_time");
            let operation_raw = get_field(&record, &cols, "operation").to_string();
            let coin_raw = get_field(&record, &cols, "coin");
            let change_raw = get_field(&record, &cols, "change");
            let remark = get_field(&record, &cols, "remark").to_string();

            let timestamp = match parse_timestamp(time_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("UTC_Time"),
                        format!("Invalid timestamp: '{}'", time_raw),
                    ));
                    continue;
                }
            };

            let change = match parse_decimal(change_raw) {
                Some(v) => v,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("Change"),
                        format!("Invalid change value: '{}'", change_raw),
                    ));
                    continue;
                }
            };

            // Skip zero-change rows
            if change.abs() < f64::EPSILON {
                continue;
            }

            let symbol = normalise_coin(coin_raw, timestamp);
            let operation = BinanceOperation::parse(&operation_raw);

            rows.push(BinanceRow {
                timestamp,
                operation,
                operation_raw,
                symbol,
                change,
                remark,
                line_number,
            });
        }

        // ── Phase 1: pair Convert and SmallAssetsExchange rows ──

        // We pair by (timestamp, operation_type). For SmallAssetsExchange there
        // may be many outgoing rows with the same timestamp.
        //
        // Key: (timestamp_string, is_convert_or_small_assets)
        // We use the formatted timestamp as key to handle minor rounding.
        let mut pending_converts: HashMap<String, PendingConvert> = HashMap::new();
        let mut standalone_rows: Vec<BinanceRow> = Vec::new();

        // Also handle Transaction Spend/Revenue pairing
        let mut pending_spend_revenue: HashMap<String, PendingConvert> = HashMap::new();

        for row in rows {
            if row.operation.should_skip() {
                continue;
            }

            if row.operation.needs_pairing() {
                let time_key = row.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                let remark_key = row.remark.trim().to_ascii_lowercase();
                let key = if remark_key.is_empty() {
                    format!("{}_{:?}", time_key, row.operation)
                } else {
                    format!("{}_{:?}_{}", time_key, row.operation, remark_key)
                };
                let entry = pending_converts.entry(key).or_default();
                entry.insert(row);
            } else if matches!(
                row.operation,
                BinanceOperation::TransactionSpend | BinanceOperation::TransactionRevenue
            ) {
                // Pair TransactionSpend + TransactionRevenue by timestamp
                let time_key = row.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                let remark_key = row.remark.trim().to_ascii_lowercase();
                let key = if remark_key.is_empty() {
                    time_key
                } else {
                    format!("{}_{}", time_key, remark_key)
                };
                let entry = pending_spend_revenue.entry(key).or_default();
                // Spend is outgoing (negative), Revenue is incoming (positive)
                entry.insert(row);
            } else {
                standalone_rows.push(row);
            }
        }

        // Resolve Convert pairs
        for (_key, pending) in pending_converts {
            if pending.is_complete() {
                let txs = pending.resolve(wallet_name);
                for (line, tx) in txs {
                    result.items.push((line, tx));
                }
            } else {
                // Incomplete pair: emit individual rows as standalone
                for row in pending.outgoing.iter().chain(pending.incoming.iter()) {
                    if let Some(tx) = unpaired_row_to_transaction(row, wallet_name) {
                        result.items.push((row.line_number, tx));
                    }
                }
            }
        }

        // Resolve Transaction Spend/Revenue pairs (treated like converts)
        for (_key, pending) in pending_spend_revenue {
            if pending.is_complete() {
                let txs = pending.resolve(wallet_name);
                for (line, tx) in txs {
                    result.items.push((line, tx));
                }
            } else {
                // Emit unpaired spend/revenue as standalone
                for row in pending.outgoing.iter().chain(pending.incoming.iter()) {
                    if let Some(tx) = unpaired_row_to_transaction(row, wallet_name) {
                        result.items.push((row.line_number, tx));
                    }
                }
            }
        }

        // ── Phase 2: process standalone rows ──

        for row in &standalone_rows {
            if let Some(tx) = single_row_to_transaction(row, wallet_name) {
                result.items.push((row.line_number, tx));
            }
        }

        // HashMap iteration order is non-deterministic; keep output stable by source line.
        result.items.sort_by_key(|(line, _)| *line);

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::BinanceAllStatements
    }
}
