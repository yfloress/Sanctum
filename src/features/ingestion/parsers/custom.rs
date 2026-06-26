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

//! Custom (user-mapped) CSV parser.
//!
//! Supports importing from *any* exchange or wallet whose CSV layout Sanctum
//! does not recognise. The flow is two-step:
//!
//! 1. [`analyze_csv_structure`] reads the header row and the first data row so
//!    the UI can show the user what each column contains.
//! 2. The user picks which header maps to each logical field
//!    ([`CustomColumnMapping`]); [`CustomCsvParser`] then resolves those
//!    columns by [`normalize_header`] (so casing/spacing/punctuation in the
//!    chosen header text does not matter) and turns every row into an
//!    [`ImportCryptoTransaction`] for the existing ingestion pipeline.
//!
//! Only `date`, `asset` and `amount` are mandatory. When no `type` column is
//! mapped, the transaction direction is inferred from the sign of the amount
//! (positive → deposit, negative → withdrawal). When a `type` column is mapped,
//! its value is classified into a Sanctum `(type, subtype)` pair via keyword
//! matching; unrecognised values produce a per-row error instead of a silent
//! mis-import.

use std::collections::HashMap;

use csv::{ReaderBuilder, StringRecord, Trim};

use super::ParseResult;
use super::exchange::common::{format_datetime, normalize_header, parse_decimal, parse_timestamp};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

/// The header row plus the first data row of an arbitrary CSV.
///
/// Returned by [`analyze_csv_structure`] to drive the column-mapping UI.
#[derive(Debug, Clone)]
pub struct CsvStructure {
    pub headers: Vec<String>,
    pub sample_row: Vec<String>,
}

/// Domain column mapping for a custom CSV import.
///
/// Each field holds the *header name* the user selected for that logical
/// column. `date_col`, `asset_col` and `amount_col` are mandatory; the rest
/// are left unmapped (`None`) when the source CSV has no matching column.
#[derive(Debug, Clone)]
pub struct CustomColumnMapping {
    pub date_col: String,
    pub asset_col: String,
    pub amount_col: String,
    pub type_col: Option<String>,
    pub fee_col: Option<String>,
    pub fee_currency_col: Option<String>,
    pub price_col: Option<String>,
    pub notes_col: Option<String>,
}

/// Reads the header row and the first data row of `content`.
///
/// The first line is assumed to be the header. Returns an error only when the
/// CSV has no readable header row.
pub fn analyze_csv_structure(content: &str) -> Result<CsvStructure, RowError> {
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| RowError::new(1, None, format!("Invalid CSV header: {e}")))?
        .iter()
        .map(clean_cell)
        .collect();

    if headers.iter().all(|h| h.is_empty()) {
        return Err(RowError::new(1, None, "CSV has no header row".to_string()));
    }

    let sample_row = reader
        .records()
        .flatten()
        .next()
        .map(|record| record.iter().map(clean_cell).collect())
        .unwrap_or_default();

    Ok(CsvStructure {
        headers,
        sample_row,
    })
}

/// Generic parser that turns mapped columns into [`ImportCryptoTransaction`]s.
pub struct CustomCsvParser {
    mapping: CustomColumnMapping,
}

impl CustomCsvParser {
    pub fn new(mapping: CustomColumnMapping) -> Self {
        Self { mapping }
    }

    pub fn parse(
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

        // Normalised-header -> first matching column index of the actual file.
        let mut index_of: HashMap<String, usize> = HashMap::new();
        for (i, col) in headers.iter().enumerate() {
            index_of.entry(normalize_header(col)).or_insert(i);
        }

        let required = |name: &str| -> Result<usize, RowError> {
            resolve_index(&index_of, name).ok_or_else(|| missing_column(name))
        };
        let optional = |name: &Option<String>| -> Result<Option<usize>, RowError> {
            match name {
                Some(n) => resolve_index(&index_of, n)
                    .map(Some)
                    .ok_or_else(|| missing_column(n)),
                None => Ok(None),
            }
        };

        let date_idx = required(&self.mapping.date_col)?;
        let asset_idx = required(&self.mapping.asset_col)?;
        let amount_idx = required(&self.mapping.amount_col)?;
        let type_idx = optional(&self.mapping.type_col)?;
        let fee_idx = optional(&self.mapping.fee_col)?;
        let fee_currency_idx = optional(&self.mapping.fee_currency_col)?;
        let price_idx = optional(&self.mapping.price_col)?;
        let notes_idx = optional(&self.mapping.notes_col)?;

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

            let date_raw = field(&record, date_idx);
            let asset_raw = field(&record, asset_idx);
            let amount_raw = field(&record, amount_idx);

            // Skip fully empty rows.
            if date_raw.is_empty() && asset_raw.is_empty() && amount_raw.is_empty() {
                continue;
            }

            let date = match parse_timestamp(date_raw) {
                Some(dt) => format_datetime(dt),
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("date"),
                        format!("Unrecognized date format: '{date_raw}'"),
                    ));
                    continue;
                }
            };

            let symbol = asset_raw.to_uppercase();
            if symbol.is_empty() {
                result.errors.push(RowError::new(
                    line_number,
                    Some("asset"),
                    "Asset symbol is empty".to_string(),
                ));
                continue;
            }

            let amount_signed = match parse_decimal(amount_raw) {
                Some(v) if v != 0.0 => v,
                _ => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("amount"),
                        format!("Invalid or zero amount: '{amount_raw}'"),
                    ));
                    continue;
                }
            };

            let (transaction_type, subtype) = match type_idx {
                Some(i) => {
                    let raw = field(&record, i);
                    match classify_custom_type(raw) {
                        Some((category, sub)) => (category.to_string(), Some(sub.to_string())),
                        None => {
                            result.errors.push(RowError::new(
                                line_number,
                                Some("type"),
                                format!("Unrecognized transaction type: '{raw}'"),
                            ));
                            continue;
                        }
                    }
                }
                // No type column: infer direction from the sign of the amount.
                None if amount_signed < 0.0 => {
                    ("transfer".to_string(), Some("withdrawal".to_string()))
                }
                None => ("transfer".to_string(), Some("deposit".to_string())),
            };

            let (fee_coin_symbol, fee_amount) = match fee_idx {
                Some(i) => match parse_decimal(field(&record, i)) {
                    Some(v) if v != 0.0 => {
                        // Fee currency defaults to the row's asset when unmapped —
                        // network fees are most often paid in the same coin.
                        let coin = fee_currency_idx
                            .map(|ci| field(&record, ci))
                            .filter(|c| !c.is_empty())
                            .map(str::to_uppercase)
                            .unwrap_or_else(|| symbol.clone());
                        (Some(coin), Some(v.abs()))
                    }
                    _ => (None, None),
                },
                None => (None, None),
            };

            let price_per_coin = price_idx.and_then(|i| parse_decimal(field(&record, i)));

            let mut notes = String::from("Custom CSV");
            if let Some(i) = notes_idx {
                let note = field(&record, i);
                if !note.is_empty() {
                    notes.push_str(" | ");
                    notes.push_str(note);
                }
            }

            result.items.push((
                line_number,
                ImportCryptoTransaction {
                    date,
                    wallet: wallet_name.to_string(),
                    symbol,
                    transaction_type,
                    amount: amount_signed.abs(),
                    subtype,
                    price_per_coin,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol,
                    fee_amount,
                    notes: Some(notes),
                },
            ));
        }

        Ok(result)
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn clean_cell(cell: &str) -> String {
    cell.trim().trim_matches('"').trim().to_string()
}

fn field(record: &StringRecord, idx: usize) -> &str {
    record
        .get(idx)
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or("")
}

fn resolve_index(index_of: &HashMap<String, usize>, name: &str) -> Option<usize> {
    index_of.get(&normalize_header(name)).copied()
}

fn missing_column(name: &str) -> RowError {
    RowError::new(
        1,
        None,
        format!("Custom CSV is missing the mapped column: '{name}'"),
    )
}

/// Classifies a raw type/operation cell into a Sanctum `(type, subtype)` pair.
///
/// Matching is keyword-based and case-insensitive; returns `None` when the
/// value cannot be confidently mapped (the caller turns that into a row error).
/// Swap/convert rows are intentionally not classified here — a single mapped
/// amount cannot describe both legs of a swap.
fn classify_custom_type(raw: &str) -> Option<(&'static str, &'static str)> {
    let t = raw.trim().to_lowercase();
    if t.is_empty() {
        return None;
    }

    // Trades.
    if t.contains("buy") || t.contains("bought") || t.contains("purchase") {
        return Some(("trade", "buy"));
    }
    if t.contains("sell") || t.contains("sold") || t.contains("sale") {
        return Some(("trade", "sell"));
    }
    // Income.
    if t.contains("airdrop") {
        return Some(("income", "airdrop"));
    }
    if t.contains("staking") || t.contains("stake") {
        return Some(("income", "staking"));
    }
    if t.contains("interest") {
        return Some(("income", "interest"));
    }
    if t.contains("mining") || t.contains("mined") {
        return Some(("income", "mining"));
    }
    if t.contains("reward") || t.contains("bonus") {
        return Some(("income", "reward"));
    }
    if t.contains("fork") {
        return Some(("income", "fork"));
    }
    // Expenses.
    if t.contains("donation") || t.contains("donate") {
        return Some(("expense", "donation"));
    }
    if t.contains("stolen") {
        return Some(("expense", "stolen"));
    }
    if t.contains("lost") {
        return Some(("expense", "lost"));
    }
    if t.contains("fee") {
        return Some(("expense", "fee"));
    }
    if t.contains("gift") {
        return Some(("income", "gift"));
    }
    // Transfers (checked last; the broadest verbs).
    if t.contains("withdraw")
        || t.contains("transfer out")
        || t.contains("send")
        || t.contains("sent")
        || t.contains("outgoing")
    {
        return Some(("transfer", "withdrawal"));
    }
    if t.contains("deposit")
        || t.contains("transfer in")
        || t.contains("receiv")
        || t.contains("incoming")
    {
        return Some(("transfer", "deposit"));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapping_min() -> CustomColumnMapping {
        CustomColumnMapping {
            date_col: "Date".to_string(),
            asset_col: "Asset".to_string(),
            amount_col: "Amount".to_string(),
            type_col: None,
            fee_col: None,
            fee_currency_col: None,
            price_col: None,
            notes_col: None,
        }
    }

    #[test]
    fn analyze_returns_headers_and_first_row() {
        let csv = "Date,Asset,Amount\n2024-01-15,BTC,1.5\n2024-01-16,ETH,2.0\n";
        let s = analyze_csv_structure(csv).expect("analyze");
        assert_eq!(s.headers, vec!["Date", "Asset", "Amount"]);
        assert_eq!(s.sample_row, vec!["2024-01-15", "BTC", "1.5"]);
    }

    #[test]
    fn analyze_handles_header_only_file() {
        let csv = "Fecha,Moneda,Cantidad\n";
        let s = analyze_csv_structure(csv).expect("analyze");
        assert_eq!(s.headers, vec!["Fecha", "Moneda", "Cantidad"]);
        assert!(s.sample_row.is_empty());
    }

    #[test]
    fn classify_covers_common_verbs() {
        assert_eq!(classify_custom_type("Buy"), Some(("trade", "buy")));
        assert_eq!(classify_custom_type("SELL ORDER"), Some(("trade", "sell")));
        assert_eq!(
            classify_custom_type("Deposit"),
            Some(("transfer", "deposit"))
        );
        assert_eq!(
            classify_custom_type("Withdrawal"),
            Some(("transfer", "withdrawal"))
        );
        assert_eq!(
            classify_custom_type("Staking Reward"),
            Some(("income", "staking"))
        );
        assert_eq!(
            classify_custom_type("Network Fee"),
            Some(("expense", "fee"))
        );
        assert_eq!(classify_custom_type("totally-unknown"), None);
        assert_eq!(classify_custom_type(""), None);
    }

    #[test]
    fn parse_infers_direction_from_amount_sign_without_type_column() {
        // Columns deliberately reordered and oddly spaced/cased.
        let csv = "Amount, ASSET ,Date\n1.5,btc,2024-01-15\n-0.5,btc,2024-01-16\n";
        let parsed = CustomCsvParser::new(mapping_min())
            .parse(csv, "MyWallet")
            .expect("parse");
        assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
        assert_eq!(parsed.items.len(), 2);

        let (_, deposit) = &parsed.items[0];
        assert_eq!(deposit.symbol, "BTC");
        assert_eq!(deposit.amount, 1.5);
        assert_eq!(deposit.transaction_type, "transfer");
        assert_eq!(deposit.subtype.as_deref(), Some("deposit"));
        assert_eq!(deposit.wallet, "MyWallet");

        let (_, withdrawal) = &parsed.items[1];
        assert_eq!(withdrawal.amount, 0.5);
        assert_eq!(withdrawal.subtype.as_deref(), Some("withdrawal"));
    }

    #[test]
    fn parse_uses_type_column_and_optional_fields() {
        let mut mapping = mapping_min();
        mapping.type_col = Some("Operation".to_string());
        mapping.fee_col = Some("Fee".to_string());
        mapping.fee_currency_col = Some("Fee Coin".to_string());
        mapping.price_col = Some("Unit Price".to_string());
        mapping.notes_col = Some("Memo".to_string());

        let csv = concat!(
            "Date,Asset,Amount,Operation,Fee,Fee Coin,Unit Price,Memo\n",
            "2024-01-15,BTC,1.0,Buy,0.001,BNB,50000,first buy\n",
        );
        let parsed = CustomCsvParser::new(mapping)
            .parse(csv, "Exchange")
            .expect("parse");
        assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
        assert_eq!(parsed.items.len(), 1);

        let (_, tx) = &parsed.items[0];
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
        assert_eq!(tx.amount, 1.0);
        assert_eq!(tx.price_per_coin, Some(50000.0));
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("BNB"));
        assert_eq!(tx.fee_amount, Some(0.001));
        assert_eq!(tx.notes.as_deref(), Some("Custom CSV | first buy"));
    }

    #[test]
    fn parse_fee_currency_defaults_to_asset_when_unmapped() {
        let mut mapping = mapping_min();
        mapping.fee_col = Some("Fee".to_string());

        let csv = "Date,Asset,Amount,Fee\n2024-01-15,XMR,1.0,0.0001\n";
        let parsed = CustomCsvParser::new(mapping)
            .parse(csv, "Wallet")
            .expect("parse");
        let (_, tx) = &parsed.items[0];
        assert_eq!(tx.fee_coin_symbol.as_deref(), Some("XMR"));
        assert_eq!(tx.fee_amount, Some(0.0001));
    }

    #[test]
    fn parse_errors_on_missing_required_column() {
        // No "Amount" column at all.
        let csv = "Date,Asset\n2024-01-15,BTC\n";
        let err = CustomCsvParser::new(mapping_min())
            .parse(csv, "Wallet")
            .expect_err("must fail");
        assert!(err.message.contains("Amount"), "msg: {}", err.message);
    }

    #[test]
    fn parse_records_row_error_for_unknown_type() {
        let mut mapping = mapping_min();
        mapping.type_col = Some("Type".to_string());
        let csv = "Date,Asset,Amount,Type\n2024-01-15,BTC,1.0,Frobnicate\n";
        let parsed = CustomCsvParser::new(mapping)
            .parse(csv, "Wallet")
            .expect("parse");
        assert!(parsed.items.is_empty());
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.errors[0].field.as_deref(), Some("type"));
    }
}
