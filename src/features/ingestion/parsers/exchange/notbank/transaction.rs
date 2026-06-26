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

use csv::{ReaderBuilder, StringRecord, Trim};

use super::super::common::normalize_header;
use super::*;

pub struct NotBankTransactionParser;

fn resolve_transaction_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (idx, col) in headers.iter().enumerate() {
        match normalize_header(col).as_str() {
            "postingentryid" => {
                map.insert("posting_entry_id", idx);
            }
            "postingentrytype" => {
                map.insert("posting_entry_type", idx);
            }
            "postingdatetime" => {
                map.insert("posting_datetime", idx);
            }
            "product" => {
                map.insert("product", idx);
            }
            "cr" => {
                map.insert("cr", idx);
            }
            "dr" => {
                map.insert("dr", idx);
            }
            "referencetransactiontype" => {
                map.insert("reference_transaction_type", idx);
            }
            "referencetransactionid" => {
                map.insert("reference_transaction_id", idx);
            }
            _ => {}
        }
    }
    map
}

fn is_trade_reference(reference_type: &str) -> bool {
    let kind = reference_type.trim().to_lowercase();
    kind.contains("execution") || kind.contains("trade")
}

fn map_transaction_kind(
    signed_amount: f64,
    posting_entry_type_raw: &str,
    reference_type_raw: &str,
) -> (String, Option<String>) {
    let entry_type = posting_entry_type_raw.trim().to_lowercase();
    let reference_type = reference_type_raw.trim().to_lowercase();

    if entry_type.contains("fee") || reference_type.contains("fee") {
        return ("expense".to_string(), Some("fee".to_string()));
    }

    if signed_amount > 0.0 {
        if reference_type.contains("deposit")
            || reference_type.contains("transfer")
            || reference_type.contains("receive")
        {
            return ("transfer".to_string(), Some("deposit".to_string()));
        }
        return ("income".to_string(), Some("other".to_string()));
    }

    if reference_type.contains("withdraw")
        || reference_type.contains("transfer")
        || reference_type.contains("spend")
    {
        return ("transfer".to_string(), Some("withdrawal".to_string()));
    }

    ("expense".to_string(), Some("other".to_string()))
}

fn build_transaction_notes(
    posting_entry_id_raw: &str,
    reference_type_raw: &str,
    reference_id_raw: &str,
) -> Option<String> {
    let mut parts = vec!["NotBank transaction".to_string()];
    if !posting_entry_id_raw.trim().is_empty() {
        parts.push(format!("entry_id={}", posting_entry_id_raw.trim()));
    }
    if !reference_type_raw.trim().is_empty() {
        parts.push(format!("type={}", reference_type_raw.trim()));
    }
    if !reference_id_raw.trim().is_empty() {
        parts.push(format!("ref={}", reference_id_raw.trim()));
    }
    Some(parts.join(" | "))
}

impl ExchangeParser for NotBankTransactionParser {
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
        let cols = resolve_transaction_columns(&headers);

        for (internal, display) in &[
            ("posting_datetime", "PostingDatetime"),
            ("product", "Product"),
            ("cr", "CR"),
            ("dr", "DR"),
            ("reference_transaction_type", "ReferenceTransactionType"),
        ] {
            if !cols.contains_key(internal) {
                return Err(RowError::new(
                    1,
                    None,
                    format!("Missing required NotBank Transaction column: '{display}'"),
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

            let posting_datetime_raw = get_field(&record, &cols, "posting_datetime");
            let posting_entry_id_raw = get_field(&record, &cols, "posting_entry_id");
            let product_raw = get_field(&record, &cols, "product");
            let cr_raw = get_field(&record, &cols, "cr");
            let dr_raw = get_field(&record, &cols, "dr");
            let reference_type_raw = get_field(&record, &cols, "reference_transaction_type");
            let reference_id_raw = get_field(&record, &cols, "reference_transaction_id");
            let posting_entry_type_raw = get_field(&record, &cols, "posting_entry_type");

            let symbol = normalize_symbol(product_raw);
            if symbol.is_empty() {
                continue;
            }
            if is_fiat(&symbol) {
                continue;
            }
            if is_trade_reference(reference_type_raw) {
                continue;
            }

            let timestamp = match parse_timestamp(posting_datetime_raw) {
                Some(dt) => dt,
                None => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("PostingDatetime"),
                        format!("Invalid timestamp: '{posting_datetime_raw}'"),
                    ));
                    continue;
                }
            };

            let credit = match parse_non_negative_decimal(cr_raw) {
                Ok(v) => v,
                Err(()) => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("CR"),
                        format!("Invalid credit amount: '{cr_raw}'"),
                    ));
                    continue;
                }
            };
            let debit = match parse_non_negative_decimal(dr_raw) {
                Ok(v) => v,
                Err(()) => {
                    result.errors.push(RowError::new(
                        line_number,
                        Some("DR"),
                        format!("Invalid debit amount: '{dr_raw}'"),
                    ));
                    continue;
                }
            };

            let signed_amount = credit - debit;
            if signed_amount.abs() <= f64::EPSILON {
                continue;
            }

            let (transaction_type, subtype) =
                map_transaction_kind(signed_amount, posting_entry_type_raw, reference_type_raw);
            let notes =
                build_transaction_notes(posting_entry_id_raw, reference_type_raw, reference_id_raw);

            result.items.push((
                line_number,
                ImportCryptoTransaction {
                    date: format_datetime(timestamp),
                    wallet: wallet_name.to_string(),
                    symbol,
                    transaction_type,
                    amount: signed_amount.abs(),
                    subtype,
                    price_per_coin: None,
                    fee: None,
                    override_proceeds: None,
                    override_cost_basis: None,
                    swap_to_symbol: None,
                    swap_to_amount: None,
                    fee_coin_symbol: None,
                    fee_amount: None,
                    notes,
                },
            ));
        }

        Ok(result)
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::NotBankTransactions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_parser_maps_deposit_and_withdrawal() {
        let csv = "\"RegisteredEntityId\",\"PostingEntryId\",\"PostingEntryType\",\"PostingDatetime\",\"AccountId\",\"AccountName\",\"Product\",\"CR\",\"DR\",\"ReferenceTransactionType\",\"ReferenceTransactionId\",\"SystemRecordReference\",\"OMSId\",\"Balance\"\n\
\"\",\"1\",\"Other\",\"2025-08-14T03:23:15.264Z\",\"100\",\"Primary\",\"LTC\",\"0.1200000000\",\"0\",\"Deposit\",\"REF-1\",\"\",\"1\",\"0.120000000\"\n\
\"\",\"2\",\"Other\",\"2025-08-14T04:42:35.795Z\",\"100\",\"Primary\",\"LTC\",\"0\",\"0.0300000000\",\"Withdraw\",\"REF-2\",\"\",\"1\",\"0.090000000\"\n";

        let parser = NotBankTransactionParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].1.transaction_type, "transfer");
        assert_eq!(result.items[0].1.subtype.as_deref(), Some("deposit"));
        let first_note = result.items[0].1.notes.as_deref().unwrap_or_default();
        assert!(first_note.contains("entry_id=1"));
        assert_eq!(result.items[1].1.transaction_type, "transfer");
        assert_eq!(result.items[1].1.subtype.as_deref(), Some("withdrawal"));
        let second_note = result.items[1].1.notes.as_deref().unwrap_or_default();
        assert!(second_note.contains("entry_id=2"));
    }

    #[test]
    fn transaction_parser_skips_trade_execution_rows() {
        let csv = "\"RegisteredEntityId\",\"PostingEntryId\",\"PostingEntryType\",\"PostingDatetime\",\"AccountId\",\"AccountName\",\"Product\",\"CR\",\"DR\",\"ReferenceTransactionType\",\"ReferenceTransactionId\",\"SystemRecordReference\",\"OMSId\",\"Balance\"\n\
\"\",\"1\",\"Other\",\"2025-08-14T03:23:15.264Z\",\"100\",\"Primary\",\"POL\",\"10\",\"0\",\"OrderExecution\",\"TR-1\",\"\",\"1\",\"10\"\n";

        let parser = NotBankTransactionParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 0);
    }
}
