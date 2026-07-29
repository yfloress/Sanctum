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
//  Kraken Ledger Parser
// ═══════════════════════════════════════════════════════════════════════════

pub struct KrakenLedgerParser;

impl ExchangeParser for KrakenLedgerParser {
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

        let cols = resolve_ledger_columns(&headers);

        // Validate required columns
        for required in &["txid", "refid", "time", "type", "asset", "amount", "fee"] {
            if !cols.contains_key(required) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required Kraken ledger column: '{}'", required),
                ));
            }
        }

        let mut result: ParseResult<ImportCryptoTransaction> = ParseResult::default();
        let mut pending: HashMap<String, PendingTrade> = HashMap::new();

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

            // Parse fields
            let refid = get_field(&record, &cols, "refid").to_string();
            let time_raw = get_field(&record, &cols, "time");
            let type_raw = get_field(&record, &cols, "type");
            let subtype_raw = get_field(&record, &cols, "subtype");
            let asset_raw = get_field(&record, &cols, "asset");
            let amount_raw = get_field(&record, &cols, "amount");
            let fee_raw = get_field(&record, &cols, "fee");

            // Parse timestamp
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

            let ledger_type = LedgerType::parse(type_raw);

            // Skip internal staking/futures transfers
            if LedgerSubtype::parse(subtype_raw)
                .as_ref()
                .is_some_and(|st| st.is_internal_transfer())
            {
                continue;
            }

            let symbol = normalize_kraken_currency(asset_raw).to_string();

            let amount = match parse_decimal(amount_raw) {
                Some(v) => v,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("amount"),
                        format!("Invalid amount: '{}'", amount_raw),
                    ));
                    continue;
                }
            };

            let fee = parse_decimal(fee_raw).unwrap_or(0.0);

            let row = LedgerRow {
                time,
                ledger_type,
                symbol,
                amount,
                fee,
                line_number,
            };

            // Determine whether this row participates in a paired trade
            let is_pairable = matches!(
                ledger_type,
                LedgerType::Trade
                    | LedgerType::MarginTrade
                    | LedgerType::Spend
                    | LedgerType::Receive
            );

            let has_refid = !refid.is_empty();

            if is_pairable && has_refid {
                let entry = pending.remove(&refid).unwrap_or_default();
                let mut entry = entry;
                let entry_line = row.line_number;
                entry.insert(row);

                if entry.is_complete() {
                    let txs = entry.resolve(&refid, wallet_name);
                    for tx in txs {
                        result.items.push((entry_line, tx));
                    }
                } else {
                    pending.insert(refid, entry);
                }
            } else {
                // Non-pairable row: emit directly as a single transaction
                if let Some(tx) = single_row_to_transaction(&row, wallet_name, &refid) {
                    result.items.push((line_number, tx));
                }
            }
        }

        // Drain remaining pending trades (incomplete pairs)
        for (refid, entry) in pending {
            let txs = entry.resolve(&refid, wallet_name);
            for tx in txs {
                // Use line 0 since we don't have a single authoritative line
                result.items.push((0, tx));
            }
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::KrakenLedger
    }
}
