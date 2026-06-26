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

//! Ingestion service
//!
//! Orchestrates data import from various file formats.

use crate::db::{Database, DbError};
use crate::models::CryptoTransaction;
use chrono::{NaiveDate, NaiveDateTime};

use std::sync::{Arc, RwLock};

use super::parsers::{
    CsvParser, CsvStructure, CustomColumnMapping, CustomCsvParser, ExchangeSource, ImportParser,
    JsonParser, analyze_csv_structure, detect_exchange_source, detect_format, parser_for,
};
use super::types::{ImportFormat, ImportSummary};
use super::validation::validate_file_size;

mod crypto;
mod transactions;

pub(super) fn format_currency_simple(cents: i64, currency: &str) -> String {
    let amount = (cents.abs() as f64) / 100.0;
    format!("{:.2} {}", amount, currency)
}

pub(super) fn normalize_import_symbol_key(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    if lower.is_empty() {
        return lower;
    }

    let compact: String = lower
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    if compact.contains("usdt") || compact.contains("tether") {
        return "usdt".to_string();
    }
    if compact.contains("usdc") || compact.contains("usdcoin") {
        return "usdc".to_string();
    }
    if compact.contains("mxtoken") {
        return "mx".to_string();
    }

    match compact.as_str() {
        "xbt" => "btc".to_string(),
        "bcc" => "bch".to_string(),
        "matic" => "pol".to_string(),
        other => other.to_string(),
    }
}

pub(super) fn matches_swap_rollup_duplicate(existing_amounts: &[f64], import_amount: f64) -> bool {
    if existing_amounts.len() < 2 {
        return false;
    }
    let summed_amount: f64 = existing_amounts.iter().copied().sum();
    (summed_amount - import_amount).abs() <= 1e-8
}

fn parse_ingestion_datetime(raw: &str) -> Option<NaiveDateTime> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S")
        .ok()
        .or_else(|| {
            NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })
}

fn note_starts_with_lowercase(note: Option<&str>, expected_prefix: &str) -> bool {
    note.map(|n| n.trim().to_lowercase().starts_with(expected_prefix))
        .unwrap_or(false)
}

pub(super) fn uses_price_agnostic_dedup(format_name: &str) -> bool {
    matches!(
        format_name.trim().to_ascii_lowercase().as_str(),
        "kraken ledger"
            | "kraken trades"
            | "binance all statements"
            | "binance spot trade history"
            | "mexc spot trade history"
            | "mexc trade history"
            | "notbank trade activity report"
    )
}

pub(super) fn note_is_exchange_overlap_prone(note: Option<&str>) -> bool {
    note_starts_with_lowercase(note, "kraken")
        || note_starts_with_lowercase(note, "binance")
        || note_starts_with_lowercase(note, "mexc")
        || note_starts_with_lowercase(note, "notbank trade")
}

fn extract_kraken_trade_ref(note: &str) -> Option<String> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("kraken") || !lower.contains("trade") {
        return None;
    }

    for marker in ["| ref:", "| tx:"] {
        if let Some(pos) = lower.find(marker) {
            let start = pos + marker.len();
            if let Some(tail) = trimmed.get(start..) {
                let value = tail.split('|').next().unwrap_or("").trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

pub(super) fn kraken_trade_ref_key(
    wallet_id: &str,
    note: Option<&str>,
) -> Option<(String, String)> {
    let reference = note.and_then(extract_kraken_trade_ref)?;
    Some((wallet_id.to_string(), reference))
}

fn extract_notbank_trade_ref(note: &str) -> Option<String> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("notbank trade") {
        return None;
    }

    for marker in ["| trade_id=", "| trans_report_id="] {
        if let Some(pos) = lower.find(marker) {
            let start = pos + marker.len();
            if let Some(tail) = trimmed.get(start..) {
                let value = tail.split('|').next().unwrap_or("").trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

pub(super) fn notbank_trade_ref_key(
    wallet_id: &str,
    note: Option<&str>,
) -> Option<(String, String)> {
    let reference = note.and_then(extract_notbank_trade_ref)?;
    Some((wallet_id.to_string(), reference))
}

fn extract_notbank_transaction_entry_id(note: &str) -> Option<String> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("notbank transaction") {
        return None;
    }

    let marker = "| entry_id=";
    if let Some(pos) = lower.find(marker) {
        let start = pos + marker.len();
        if let Some(tail) = trimmed.get(start..) {
            let value = tail.split('|').next().unwrap_or("").trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

pub(super) fn notbank_transaction_ref_key(
    wallet_id: &str,
    note: Option<&str>,
) -> Option<(String, String)> {
    let reference = note.and_then(extract_notbank_transaction_entry_id)?;
    Some((wallet_id.to_string(), reference))
}

pub(super) struct MexcTransferOverlapProbe<'a> {
    wallet_id: &'a str,
    coin_id: &'a str,
    mechanical_type: &'a str,
    amount: f64,
    fee_amount: Option<f64>,
    date: &'a str,
}

pub(super) fn has_mexc_transfer_overlap_duplicate(
    existing: &[CryptoTransaction],
    format_name: &str,
    probe: &MexcTransferOverlapProbe<'_>,
) -> bool {
    if probe.mechanical_type != "transfer_in" && probe.mechanical_type != "transfer_out" {
        return false;
    }

    let incoming_is_statement = format_name.eq_ignore_ascii_case("MEXC Statement History");
    let incoming_is_deposit = format_name.eq_ignore_ascii_case("MEXC Deposit History");
    let incoming_is_withdrawal = format_name.eq_ignore_ascii_case("MEXC Withdrawal History");

    let required_existing_note_prefix = if incoming_is_statement {
        if probe.mechanical_type == "transfer_in" {
            "mexc deposit"
        } else {
            "mexc withdrawal"
        }
    } else if incoming_is_deposit || incoming_is_withdrawal {
        "mexc statement"
    } else {
        return false;
    };

    let import_dt = match parse_ingestion_datetime(probe.date) {
        Some(dt) => dt,
        None => return false,
    };

    const OVERLAP_WINDOW_SECONDS: i64 = 15 * 60;

    existing.iter().any(|tx| {
        if tx.wallet_id != probe.wallet_id || tx.coin_id != probe.coin_id {
            return false;
        }
        if tx.mechanical_type() != probe.mechanical_type {
            return false;
        }
        if (tx.amount - probe.amount).abs() > 1e-8 {
            return false;
        }

        if probe.fee_amount.is_some() || tx.fee_amount.is_some() {
            match (probe.fee_amount, tx.fee_amount) {
                (Some(incoming_fee), Some(existing_fee))
                    if (incoming_fee - existing_fee).abs() <= 1e-8 => {}
                _ => return false,
            }
        }

        if !note_starts_with_lowercase(tx.notes.as_deref(), required_existing_note_prefix) {
            return false;
        }

        let existing_dt = match parse_ingestion_datetime(&tx.date) {
            Some(dt) => dt,
            None => return false,
        };

        let delta = (existing_dt - import_dt).num_seconds().abs();
        delta <= OVERLAP_WINDOW_SECONDS
    })
}

#[derive(thiserror::Error, Debug)]
pub enum IngestionError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("No vault is currently open")]
    NoVaultOpen,

    #[error("Session expired due to inactivity")]
    SessionExpired,

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("File too large: {0}")]
    FileTooLarge(String),
}

pub struct IngestionService {
    db: Arc<RwLock<Option<Database>>>,
}

impl IngestionService {
    pub fn new(db: Arc<RwLock<Option<Database>>>) -> Self {
        Self { db }
    }

    fn with_db<T, F>(&self, f: F) -> Result<T, IngestionError>
    where
        F: FnOnce(&Database) -> Result<T, IngestionError>,
    {
        self.with_db_session(f, true)
    }

    fn with_db_readonly<T, F>(&self, f: F) -> Result<T, IngestionError>
    where
        F: FnOnce(&Database) -> Result<T, IngestionError>,
    {
        self.with_db_session(f, false)
    }

    fn with_db_session<T, F>(&self, f: F, touch_session: bool) -> Result<T, IngestionError>
    where
        F: FnOnce(&Database) -> Result<T, IngestionError>,
    {
        let db_lock = self
            .db
            .read()
            .map_err(|_| IngestionError::Parse("Lock error".to_string()))?;
        let db = db_lock.as_ref().ok_or(IngestionError::NoVaultOpen)?;

        let session_result = if touch_session {
            db.check_session_timeout()
        } else {
            db.check_session_timeout_readonly()
        };
        session_result.map_err(|e| match e {
            DbError::SessionExpired => IngestionError::SessionExpired,
            _ => IngestionError::Database(e),
        })?;

        let result = f(db)?;
        if touch_session {
            db.touch_session().map_err(IngestionError::Database)?;
        }
        Ok(result)
    }

    /// Main entry point for importing data from file content
    pub fn import_from_content(
        &self,
        content: &str,
        filename: &str,
    ) -> Result<ImportSummary, IngestionError> {
        // Validate file size
        validate_file_size(content.len()).map_err(IngestionError::FileTooLarge)?;

        // Detect format
        let format = detect_format(content, filename).ok_or_else(|| {
            IngestionError::UnsupportedFormat(
                "Could not detect file format. Supported: JSON (.json), CSV (.csv), Text (.txt)"
                    .to_string(),
            )
        })?;

        match format {
            ImportFormat::Json => self.import_json(content),
            ImportFormat::CsvTransactions => self.import_csv_transactions(content),
            ImportFormat::CsvCrypto => self.import_csv_crypto(content),
            ImportFormat::ExchangeCsv(source) => {
                let wallet = source.default_wallet_name();
                self.import_exchange_csv(content, wallet, source)
            }
        }
    }

    /// Preview import results without writing to the database
    pub fn preview_from_content(
        &self,
        content: &str,
        filename: &str,
    ) -> Result<ImportSummary, IngestionError> {
        validate_file_size(content.len()).map_err(IngestionError::FileTooLarge)?;

        let format = detect_format(content, filename).ok_or_else(|| {
            IngestionError::UnsupportedFormat(
                "Could not detect file format. Supported: JSON (.json), CSV (.csv), Text (.txt)"
                    .to_string(),
            )
        })?;

        match format {
            ImportFormat::Json => self.preview_json(content),
            ImportFormat::CsvTransactions => self.preview_csv_transactions(content),
            ImportFormat::CsvCrypto => self.preview_csv_crypto(content),
            ImportFormat::ExchangeCsv(source) => {
                let wallet = source.default_wallet_name();
                self.preview_exchange_csv(content, wallet, source)
            }
        }
    }

    /// Import JSON format (can contain transactions and crypto)
    fn import_json(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = JsonParser;
        let mut summary = ImportSummary::new("JSON", "Mixed");

        let file = parser
            .parse_full(content)
            .map_err(|e| IngestionError::Parse(e.message))?;

        // Import transactions
        if !file.transactions.items.is_empty() || !file.transactions.errors.is_empty() {
            let tx_summary =
                self.process_transactions(file.transactions.items, parser.format_name())?;
            summary.merge(tx_summary);
            for error in file.transactions.errors {
                summary.record_error(error);
            }
        }

        // Import crypto transactions
        if !file.crypto_transactions.items.is_empty() || !file.crypto_transactions.errors.is_empty()
        {
            let crypto_summary = self.process_crypto_transactions(
                file.crypto_transactions.items,
                parser.format_name(),
            )?;
            summary.merge(crypto_summary);
            for error in file.crypto_transactions.errors {
                summary.record_error(error);
            }
        }

        Ok(summary)
    }

    /// Preview JSON format (can contain transactions and crypto)
    fn preview_json(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = JsonParser;
        let mut summary = ImportSummary::new("JSON", "Mixed");

        let file = parser
            .parse_full(content)
            .map_err(|e| IngestionError::Parse(e.message))?;

        if !file.transactions.items.is_empty() || !file.transactions.errors.is_empty() {
            let tx_summary =
                self.preview_transactions(file.transactions.items, parser.format_name())?;
            summary.merge(tx_summary);
            for error in file.transactions.errors {
                summary.record_error(error);
            }
        }

        if !file.crypto_transactions.items.is_empty() || !file.crypto_transactions.errors.is_empty()
        {
            let crypto_summary = self.preview_crypto_transactions(
                file.crypto_transactions.items,
                parser.format_name(),
            )?;
            summary.merge(crypto_summary);
            for error in file.crypto_transactions.errors {
                summary.record_error(error);
            }
        }

        Ok(summary)
    }

    /// Import CSV transactions
    fn import_csv_transactions(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = CsvParser;
        let parsed = parser
            .parse_transactions(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.process_transactions(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Preview CSV transactions
    fn preview_csv_transactions(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = CsvParser;
        let parsed = parser
            .parse_transactions(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.preview_transactions(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Import CSV crypto transactions
    fn import_csv_crypto(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = CsvParser;
        let parsed = parser
            .parse_crypto_transactions(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.process_crypto_transactions(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Preview CSV crypto transactions
    fn preview_csv_crypto(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = CsvParser;
        let parsed = parser
            .parse_crypto_transactions(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.preview_crypto_transactions(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    // ── Exchange CSV import/preview ──────────────────────────────────────────

    /// Import exchange CSV with explicit wallet name and source.
    ///
    /// This is the main entry point used by both the generic `import_from_content`
    /// path (with default wallet name) and the dedicated exchange import command
    /// (with user-provided wallet name).
    ///
    /// Balance validation is skipped for exchange imports because the wallet
    /// export is the authoritative source of truth — if it records a withdrawal,
    /// it happened regardless of what Sanctum currently knows about the balance.
    pub fn import_exchange_csv(
        &self,
        content: &str,
        wallet_name: &str,
        source: ExchangeSource,
    ) -> Result<ImportSummary, IngestionError> {
        let parser = parser_for(source);
        let parsed = parser
            .parse(content, wallet_name)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary =
            self.process_crypto_transactions_ext(parsed.items, source.label(), true)?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Preview exchange CSV import without writing to the database.
    pub fn preview_exchange_csv(
        &self,
        content: &str,
        wallet_name: &str,
        source: ExchangeSource,
    ) -> Result<ImportSummary, IngestionError> {
        let parser = parser_for(source);
        let parsed = parser
            .parse(content, wallet_name)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary =
            self.preview_crypto_transactions_ext(parsed.items, source.label(), true)?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Import exchange CSV with auto-detection of exchange source.
    ///
    /// Validates file size, detects the exchange from headers, and delegates
    /// to `import_exchange_csv`. The caller provides the wallet name.
    pub fn import_exchange_csv_auto(
        &self,
        content: &str,
        wallet_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        validate_file_size(content.len()).map_err(IngestionError::FileTooLarge)?;

        let source = detect_exchange_source(content).ok_or_else(|| {
                IngestionError::UnsupportedFormat(
                    "Could not detect exchange format. Supported: Kraken, Binance, MEXC, NotBank, Feather Wallet, Monero GUI Wallet"
                        .to_string(),
                )
            })?;

        self.import_exchange_csv(content, wallet_name, source)
    }

    /// Preview exchange CSV with auto-detection of exchange source.
    pub fn preview_exchange_csv_auto(
        &self,
        content: &str,
        wallet_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        validate_file_size(content.len()).map_err(IngestionError::FileTooLarge)?;

        let source = detect_exchange_source(content).ok_or_else(|| {
                IngestionError::UnsupportedFormat(
                    "Could not detect exchange format. Supported: Kraken, Binance, MEXC, NotBank, Feather Wallet, Monero GUI Wallet"
                        .to_string(),
                )
            })?;

        self.preview_exchange_csv(content, wallet_name, source)
    }

    // ── Custom (user-mapped) CSV import ──────────────────────────────────────

    /// Inspects an arbitrary CSV, returning its header row and first data row
    /// so the UI can let the user map each column.
    ///
    /// Used for exchanges/wallets whose layout Sanctum does not auto-detect.
    pub fn analyze_custom_csv(&self, content: &str) -> Result<CsvStructure, IngestionError> {
        validate_file_size(content.len()).map_err(IngestionError::FileTooLarge)?;
        analyze_csv_structure(content).map_err(|e| IngestionError::Parse(e.message))
    }

    /// Imports an arbitrary CSV using a user-provided column mapping.
    ///
    /// Builds a generic parser from `mapping`, converts each row into a crypto
    /// transaction, and feeds them through the shared processing pipeline.
    /// Balance validation is skipped — like the dedicated exchange imports, the
    /// imported file is treated as the authoritative record.
    pub fn import_custom_csv(
        &self,
        content: &str,
        mapping: CustomColumnMapping,
        wallet_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        validate_file_size(content.len()).map_err(IngestionError::FileTooLarge)?;
        let parsed = CustomCsvParser::new(mapping)
            .parse(content, wallet_name)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.process_crypto_transactions_ext(parsed.items, "Custom CSV", true)?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn sample_crypto_tx(
    date: &str,
    wallet_id: &str,
    coin_id: &str,
    tx_type: &str,
    subtype: Option<&str>,
    amount: f64,
    fee_amount: Option<f64>,
    notes: Option<&str>,
) -> CryptoTransaction {
    CryptoTransaction {
        id: "tx-1".to_string(),
        wallet_id: wallet_id.to_string(),
        coin_id: coin_id.to_string(),
        symbol: "USDT".to_string(),
        transaction_type: tx_type.to_string(),
        amount,
        price_per_coin: None,
        fee: None,
        fee_coin_id: fee_amount.map(|_| coin_id.to_string()),
        fee_amount,
        subtype: subtype.map(|s| s.to_string()),
        override_proceeds: None,
        override_cost_basis: None,
        date: date.to_string(),
        notes: notes.map(|n| n.to_string()),
        related_tx_id: None,
    }
}

#[test]
fn normalize_import_symbol_key_maps_common_aliases() {
    assert_eq!(normalize_import_symbol_key("TETHER"), "usdt");
    assert_eq!(normalize_import_symbol_key("USDT(TRC20)"), "usdt");
    assert_eq!(normalize_import_symbol_key("Tether USDt"), "usdt");
    assert_eq!(normalize_import_symbol_key("USD Coin"), "usdc");
    assert_eq!(normalize_import_symbol_key("MXTOKEN"), "mx");
    assert_eq!(normalize_import_symbol_key("xbt"), "btc");
    assert_eq!(normalize_import_symbol_key("BCC"), "bch");
    assert_eq!(normalize_import_symbol_key("MATIC"), "pol");
}

#[test]
fn normalize_import_symbol_key_keeps_regular_symbols() {
    assert_eq!(normalize_import_symbol_key("USDT"), "usdt");
    assert_eq!(normalize_import_symbol_key("MX"), "mx");
    assert_eq!(normalize_import_symbol_key(" ICP "), "icp");
}

#[test]
fn swap_rollup_duplicate_requires_multiple_existing_rows() {
    assert!(!matches_swap_rollup_duplicate(&[12.0], 12.0));
}

#[test]
fn swap_rollup_duplicate_matches_summed_split_fills() {
    assert!(matches_swap_rollup_duplicate(&[1.2, 3.8], 5.0));
    assert!(!matches_swap_rollup_duplicate(&[1.2, 3.8], 4.9));
}

#[test]
fn extract_kraken_trade_ref_accepts_ref_and_tx_markers() {
    assert_eq!(
        extract_kraken_trade_ref("Kraken trade | BTC/USD | Ref: TX-123"),
        Some("TX-123".to_string())
    );
    assert_eq!(
        extract_kraken_trade_ref("Kraken trade | ETH/USDT | Tx: TX-456 | Order: ORD-1"),
        Some("TX-456".to_string())
    );
}

#[test]
fn extract_kraken_trade_ref_ignores_non_trade_notes() {
    assert_eq!(
        extract_kraken_trade_ref("Kraken deposit | Ref: TX-123"),
        None
    );
    assert_eq!(extract_kraken_trade_ref("MEXC trade | Ref: TX-123"), None);
}

#[test]
fn kraken_trade_ref_key_binds_reference_to_wallet() {
    assert_eq!(
        kraken_trade_ref_key("wallet-1", Some("Kraken trade | BTC/USD | Ref: TX-777")),
        Some(("wallet-1".to_string(), "TX-777".to_string()))
    );
    assert_eq!(
        kraken_trade_ref_key("wallet-1", Some("Kraken deposit | Ref: TX-777")),
        None
    );
}

#[test]
fn extract_notbank_trade_ref_accepts_trade_id_marker() {
    assert_eq!(
        extract_notbank_trade_ref("NotBank trade | Buy BTCUSDT | trade_id=300001 | order_id=1"),
        Some("300001".to_string())
    );
}

#[test]
fn extract_notbank_trade_ref_falls_back_to_trans_report_id_marker() {
    assert_eq!(
        extract_notbank_trade_ref(
            "NotBank trade | Buy BTCUSDT | trans_report_id=200001 | order_id=1",
        ),
        Some("200001".to_string())
    );
}

#[test]
fn extract_notbank_trade_ref_ignores_non_trade_notes() {
    assert_eq!(
        extract_notbank_trade_ref("NotBank transaction | type=Deposit | ref=A001"),
        None
    );
    assert_eq!(
        extract_notbank_trade_ref("Kraken trade | BTC/USD | Ref: TX-1"),
        None
    );
}

#[test]
fn notbank_trade_ref_key_binds_reference_to_wallet() {
    assert_eq!(
        notbank_trade_ref_key(
            "wallet-1",
            Some("NotBank trade | Buy BTCUSDT | trade_id=300001")
        ),
        Some(("wallet-1".to_string(), "300001".to_string()))
    );
    assert_eq!(
        notbank_trade_ref_key(
            "wallet-1",
            Some("NotBank transaction | type=Deposit | ref=A001"),
        ),
        None
    );
}

#[test]
fn extract_notbank_transaction_entry_id_accepts_entry_id_marker() {
    assert_eq!(
        extract_notbank_transaction_entry_id(
            "NotBank transaction | entry_id=10000123 | type=Deposit | ref=A001",
        ),
        Some("10000123".to_string())
    );
}

#[test]
fn extract_notbank_transaction_entry_id_ignores_non_transaction_notes() {
    assert_eq!(
        extract_notbank_transaction_entry_id("NotBank trade | trade_id=300001"),
        None
    );
    assert_eq!(
        extract_notbank_transaction_entry_id("Kraken ledger | Ref: TX-1"),
        None
    );
}

#[test]
fn notbank_transaction_ref_key_binds_entry_id_to_wallet() {
    assert_eq!(
        notbank_transaction_ref_key(
            "wallet-1",
            Some("NotBank transaction | entry_id=10000123 | type=Deposit"),
        ),
        Some(("wallet-1".to_string(), "10000123".to_string()))
    );
    assert_eq!(
        notbank_transaction_ref_key("wallet-1", Some("NotBank trade | trade_id=300001")),
        None
    );
}

#[test]
fn uses_price_agnostic_dedup_for_overlap_prone_exchange_sources() {
    assert!(uses_price_agnostic_dedup("Kraken Ledger"));
    assert!(uses_price_agnostic_dedup("Kraken Trades"));
    assert!(uses_price_agnostic_dedup("Binance All Statements"));
    assert!(uses_price_agnostic_dedup("Binance Spot Trade History"));
    assert!(uses_price_agnostic_dedup("MEXC Spot Trade History"));
    assert!(uses_price_agnostic_dedup("MEXC Trade History"));
    assert!(uses_price_agnostic_dedup("NotBank Trade Activity Report"));
    assert!(!uses_price_agnostic_dedup("NotBank Transaction Report"));
}

#[test]
fn note_is_exchange_overlap_prone_matches_supported_prefixes() {
    assert!(note_is_exchange_overlap_prone(Some(
        "Kraken trade | Ref: TX-1"
    )));
    assert!(note_is_exchange_overlap_prone(Some(
        "Binance Spot BUY | BTCUSDT"
    )));
    assert!(note_is_exchange_overlap_prone(Some(
        "MEXC trade | BTC_USDT | Ref=123"
    )));
    assert!(note_is_exchange_overlap_prone(Some("NotBank trade | id=1")));
    assert!(!note_is_exchange_overlap_prone(Some(
        "NotBank transaction | type=Deposit"
    )));
    assert!(!note_is_exchange_overlap_prone(None));
}

#[test]
fn mexc_overlap_detects_statement_deposit_vs_dedicated_deposit() {
    let existing = vec![sample_crypto_tx(
        "2026-01-12 10:00:00",
        "wallet-1",
        "tether",
        "transfer",
        Some("deposit"),
        100.0,
        None,
        Some("MEXC deposit | network=Polygon"),
    )];

    assert!(has_mexc_transfer_overlap_duplicate(
        &existing,
        "MEXC Statement History",
        &MexcTransferOverlapProbe {
            wallet_id: "wallet-1",
            coin_id: "tether",
            mechanical_type: "transfer_in",
            amount: 100.0,
            fee_amount: None,
            date: "2026-01-12 10:07:00",
        },
    ));
}

#[test]
fn mexc_overlap_rejects_rows_outside_time_window() {
    let existing = vec![sample_crypto_tx(
        "2026-01-12 10:00:00",
        "wallet-1",
        "tether",
        "transfer",
        Some("deposit"),
        100.0,
        None,
        Some("MEXC deposit | network=Polygon"),
    )];

    assert!(!has_mexc_transfer_overlap_duplicate(
        &existing,
        "MEXC Statement History",
        &MexcTransferOverlapProbe {
            wallet_id: "wallet-1",
            coin_id: "tether",
            mechanical_type: "transfer_in",
            amount: 100.0,
            fee_amount: None,
            date: "2026-01-12 10:30:01",
        },
    ));
}

#[test]
fn mexc_overlap_detects_dedicated_withdrawal_vs_statement_with_fee() {
    let existing = vec![sample_crypto_tx(
        "2026-01-12 12:20:00",
        "wallet-1",
        "litecoin",
        "transfer",
        Some("withdrawal"),
        0.5,
        Some(0.0001),
        Some("MEXC statement | type=Withdraw | direction=Outflow"),
    )];

    assert!(has_mexc_transfer_overlap_duplicate(
        &existing,
        "MEXC Withdrawal History",
        &MexcTransferOverlapProbe {
            wallet_id: "wallet-1",
            coin_id: "litecoin",
            mechanical_type: "transfer_out",
            amount: 0.5,
            fee_amount: Some(0.0001),
            date: "2026-01-12 12:12:30",
        },
    ));
}

// ── Integration tests for process_transactions ────────────────────────────────

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::db::Database as IngestionTestDb;
    use crate::features::ingestion::types::{ImportCryptoTransaction, ImportTransaction};
    use crate::models::{Account, CryptoWallet, TransactionCategory};
    use secrecy::SecretString;
    use std::path::PathBuf;
    use uuid::Uuid;

    struct IngestionTestHarness {
        service: IngestionService,
        db: Arc<RwLock<Option<IngestionTestDb>>>,
        test_dir: PathBuf,
    }

    #[cfg(test)]
    impl Drop for IngestionTestHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.test_dir);
        }
    }

    #[cfg(test)]
    fn new_ingestion_harness() -> IngestionTestHarness {
        let base_dir =
            std::env::temp_dir().join(format!("sanctum-ingestion-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&base_dir).expect("create test dir");
        let db_path = base_dir.join("vault.db");
        let password = SecretString::from("test-password-123".to_string());
        let db = IngestionTestDb::init(db_path, &password).expect("init test database");
        let db_arc = Arc::new(RwLock::new(Some(db)));
        let service = IngestionService::new(db_arc.clone());
        IngestionTestHarness {
            service,
            db: db_arc,
            test_dir: base_dir,
        }
    }

    fn seed_account(db: &IngestionTestDb) -> String {
        let id = Uuid::new_v4().to_string();
        let account = Account {
            id: id.clone(),
            name: "Checking".to_string(),
            account_type: "bank".to_string(),
            currency: "USD".to_string(),
            initial_balance: 0,
            color: "#8b5cf6".to_string(),
            icon: None,
            is_archived: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        db.create_account(&account).expect("create account");
        id
    }

    fn seed_account_named(db: &IngestionTestDb, name: &str, currency: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let account = Account {
            id: id.clone(),
            name: name.to_string(),
            account_type: "bank".to_string(),
            currency: currency.to_string(),
            initial_balance: 0,
            color: "#8b5cf6".to_string(),
            icon: None,
            is_archived: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        db.create_account(&account).expect("create account");
        id
    }

    fn seed_category(db: &IngestionTestDb, name: &str, category_type: &str) {
        let cat = TransactionCategory {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            category_type: category_type.to_string(),
            sort_order: 0,
            is_default: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        db.add_transaction_category(&cat.name, &cat.category_type)
            .expect("create category");
    }

    // ==================== Transaction Ingestion Tests ====================

    #[test]
    fn test_process_transactions_imports_income() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account(db);
            seed_category(db, "Salary", "income");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Checking".to_string(),
            transaction_type: "income".to_string(),
            amount: 5000.00,
            currency: "USD".to_string(),
            category: "Salary".to_string(),
            description: "June salary".to_string(),
            transfer_to_account: None,
        };
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 1);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn test_process_transactions_imports_expense() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account(db);
            seed_category(db, "Food", "expense");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Checking".to_string(),
            transaction_type: "expense".to_string(),
            amount: 25.50,
            currency: "USD".to_string(),
            category: "Food".to_string(),
            description: "Lunch".to_string(),
            transfer_to_account: None,
        };
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 1);
    }

    #[test]
    fn test_process_transactions_imports_transfer() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account_named(db, "Source", "USD");
            seed_account_named(db, "Dest", "USD");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Source".to_string(),
            transaction_type: "transfer".to_string(),
            amount: 1000.00,
            currency: "USD".to_string(),
            category: "transfer".to_string(),
            description: "Monthly transfer".to_string(),
            transfer_to_account: Some("Dest".to_string()),
        };
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 1);
    }

    #[test]
    fn test_process_transactions_rejects_unknown_account() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_category(db, "Test", "income");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Nonexistent".to_string(),
            transaction_type: "income".to_string(),
            amount: 100.0,
            currency: "USD".to_string(),
            category: "Test".to_string(),
            description: "test".to_string(),
            transfer_to_account: None,
        };
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_process_transactions_rejects_unknown_category() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account(db);
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Checking".to_string(),
            transaction_type: "income".to_string(),
            amount: 100.0,
            currency: "USD".to_string(),
            category: "Nonexistent".to_string(),
            description: "test".to_string(),
            transfer_to_account: None,
        };
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_process_transactions_rejects_currency_mismatch() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account_named(db, "EUR Acc", "EUR");
            seed_category(db, "Test", "income");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "EUR Acc".to_string(),
            transaction_type: "income".to_string(),
            amount: 100.0,
            currency: "USD".to_string(),
            category: "Test".to_string(),
            description: "test".to_string(),
            transfer_to_account: None,
        };
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_process_transactions_rejects_self_transfer() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account_named(db, "Acc", "USD");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Acc".to_string(),
            transaction_type: "transfer".to_string(),
            amount: 100.0,
            currency: "USD".to_string(),
            category: "transfer".to_string(),
            description: "self".to_string(),
            transfer_to_account: Some("Acc".to_string()),
        };
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_process_transactions_deduplicates_exact_match() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account(db);
            seed_category(db, "Salary", "income");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Checking".to_string(),
            transaction_type: "income".to_string(),
            amount: 5000.00,
            currency: "USD".to_string(),
            category: "Salary".to_string(),
            description: "June salary".to_string(),
            transfer_to_account: None,
        };
        // First import inserts
        h.service
            .process_transactions(vec![(1, tx.clone())], "Test Format")
            .expect("first");
        // Second import deduplicates
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("second");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.skipped, 1);
    }

    #[test]
    fn test_preview_transactions_does_not_insert() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account(db);
            seed_category(db, "Salary", "income");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Checking".to_string(),
            transaction_type: "income".to_string(),
            amount: 5000.00,
            currency: "USD".to_string(),
            category: "Salary".to_string(),
            description: "June salary".to_string(),
            transfer_to_account: None,
        };
        let result = h
            .service
            .preview_transactions(vec![(1, tx)], "Test Format")
            .expect("preview");
        assert_eq!(
            result.preview_changes.len(),
            1,
            "preview should show 1 change"
        );
        assert_eq!(result.errors, 0, "preview should have no errors");
        // Verify nothing was actually persisted
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            let txs = db.get_transactions().expect("get transactions");
            assert!(txs.is_empty(), "preview should not persist data");
        }
    }

    #[test]
    fn test_process_transactions_rejects_transfer_without_destination() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account_named(db, "Acc", "USD");
        }
        let tx = ImportTransaction {
            date: "2024-06-15".to_string(),
            account: "Acc".to_string(),
            transaction_type: "transfer".to_string(),
            amount: 100.0,
            currency: "USD".to_string(),
            category: "transfer".to_string(),
            description: "missing dest".to_string(),
            transfer_to_account: None,
        };
        let result = h
            .service
            .process_transactions(vec![(1, tx)], "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_process_transactions_handles_mixed_income_and_expense() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_account(db);
            seed_category(db, "Salary", "income");
            seed_category(db, "Food", "expense");
        }
        let txs = vec![
            (
                1,
                ImportTransaction {
                    date: "2024-06-15".to_string(),
                    account: "Checking".to_string(),
                    transaction_type: "income".to_string(),
                    amount: 5000.00,
                    currency: "USD".to_string(),
                    category: "Salary".to_string(),
                    description: "Salary".to_string(),
                    transfer_to_account: None,
                },
            ),
            (
                2,
                ImportTransaction {
                    date: "2024-06-15".to_string(),
                    account: "Checking".to_string(),
                    transaction_type: "expense".to_string(),
                    amount: 50.00,
                    currency: "USD".to_string(),
                    category: "Food".to_string(),
                    description: "Dinner".to_string(),
                    transfer_to_account: None,
                },
            ),
        ];
        let result = h
            .service
            .process_transactions(txs, "Test Format")
            .expect("process");
        assert_eq!(result.inserted, 2);
        assert_eq!(result.errors, 0);
    }

    // ==================== Crypto Transaction Ingestion Tests ====================

    fn seed_wallet(db: &IngestionTestDb, name: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let wallet = CryptoWallet::new(id.clone(), name.to_string(), "exchange".to_string(), None);
        db.create_wallet(&wallet).expect("create wallet");
        id
    }

    // Test fixture builder: cohesive args mirror the struct fields it constructs.
    #[allow(clippy::too_many_arguments)]
    fn make_crypto_tx(
        date: &str,
        wallet: &str,
        symbol: &str,
        tx_type: &str,
        subtype: Option<&str>,
        amount: f64,
        price: Option<f64>,
        fee: Option<f64>,
        swap_to_symbol: Option<&str>,
        swap_to_amount: Option<f64>,
    ) -> ImportCryptoTransaction {
        ImportCryptoTransaction {
            date: date.to_string(),
            wallet: wallet.to_string(),
            symbol: symbol.to_string(),
            transaction_type: tx_type.to_string(),
            amount,
            subtype: subtype.map(String::from),
            price_per_coin: price,
            fee,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: swap_to_symbol.map(String::from),
            swap_to_amount,
            fee_coin_symbol: None,
            fee_amount: None,
            notes: None,
        }
    }

    #[test]
    fn test_process_crypto_buy() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "Exchange",
            "BTC",
            "trade",
            Some("buy"),
            1.5,
            Some(50000.0),
            Some(0.10),
            None,
            None,
        );
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        assert_eq!(result.inserted, 1);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn test_process_crypto_buy_with_fee_coin() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let mut tx = make_crypto_tx(
            "2024-06-15",
            "Exchange",
            "BTC",
            "trade",
            Some("buy"),
            1.5,
            Some(50000.0),
            Some(0.10),
            None,
            None,
        );
        tx.fee_coin_symbol = Some("BTC".to_string());
        tx.fee_amount = Some(0.001);
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        assert_eq!(result.inserted, 1);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn test_process_crypto_sell() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "Exchange",
            "BTC",
            "trade",
            Some("sell"),
            0.5,
            Some(51000.0),
            Some(0.10),
            None,
            None,
        );
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        assert_eq!(result.inserted, 1);
    }

    #[test]
    fn test_process_crypto_transfer_in() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "MyWallet");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "MyWallet",
            "ETH",
            "transfer",
            None,
            2.0,
            None,
            None,
            None,
            None,
        );
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        assert_eq!(result.inserted, 1);
    }

    #[test]
    fn test_process_crypto_transfer_out() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "MyWallet");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "MyWallet",
            "ETH",
            "transfer",
            Some("withdrawal"),
            1.0,
            None,
            None,
            None,
            None,
        );
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        assert_eq!(result.inserted, 1);
    }

    #[test]
    fn test_process_crypto_swap_creates_two_transactions() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "Exchange",
            "BTC",
            "trade",
            Some("swap"),
            1.0,
            None,
            Some(0.05),
            Some("ETH"),
            Some(100.0),
        );
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        assert_eq!(
            result.inserted, 1,
            "swap counts as one insertion in summary"
        );

        // Verify two transactions were actually created in DB
        let guard = h.db.read().expect("lock");
        let db = guard.as_ref().expect("db");
        let all = db
            .get_all_crypto_transactions(0, i64::MAX)
            .expect("get all");
        assert_eq!(all.len(), 2, "swap should create source + target tx");
    }

    #[test]
    fn test_process_crypto_rejects_unknown_wallet() {
        let h = new_ingestion_harness();
        let tx = make_crypto_tx(
            "2024-06-15",
            "Nonexistent",
            "BTC",
            "trade",
            Some("buy"),
            1.0,
            None,
            None,
            None,
            None,
        );
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_process_crypto_rejects_unknown_coin() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "Exchange",
            "UNKNOWNCOIN123",
            "trade",
            Some("buy"),
            1.0,
            None,
            None,
            None,
            None,
        );
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        // UNKNOWNCOIN123 is not in default_coin_catalog, should be skipped
        assert!(
            result.skipped > 0 || result.errors > 0,
            "expected unknown coin to be rejected, got inserted={}, skipped={}, errors={}",
            result.inserted,
            result.skipped,
            result.errors,
        );
        assert_eq!(result.inserted, 0);
    }

    #[test]
    fn test_process_crypto_rejects_swap_with_unknown_target() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "Exchange",
            "BTC",
            "trade",
            Some("swap"),
            1.0,
            None,
            None,
            Some("UNKNOWN"),
            Some(100.0),
        );
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("process");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.skipped, 1);
    }

    #[test]
    fn test_process_crypto_deduplicates_exact_match() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "Exchange",
            "BTC",
            "trade",
            Some("buy"),
            1.0,
            Some(50000.0),
            None,
            None,
            None,
        );
        h.service
            .process_crypto_transactions_ext(vec![(1, tx.clone())], "Test Format", true)
            .expect("first");
        let result = h
            .service
            .process_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("second");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.skipped, 1);
    }

    #[test]
    fn test_preview_crypto_does_not_insert() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let tx = make_crypto_tx(
            "2024-06-15",
            "Exchange",
            "BTC",
            "trade",
            Some("buy"),
            1.0,
            Some(50000.0),
            None,
            None,
            None,
        );
        let result = h
            .service
            .preview_crypto_transactions_ext(vec![(1, tx)], "Test Format", true)
            .expect("preview");
        assert!(
            !result.preview_changes.is_empty(),
            "preview should show changes"
        );
        // Verify nothing persisted
        let guard = h.db.read().expect("lock");
        let db = guard.as_ref().expect("db");
        let all = db
            .get_all_crypto_transactions(0, i64::MAX)
            .expect("get all");
        assert!(all.is_empty(), "preview should not persist");
    }

    #[test]
    fn test_process_crypto_handles_multiple_types() {
        let h = new_ingestion_harness();
        {
            let guard = h.db.read().expect("lock");
            let db = guard.as_ref().expect("db");
            seed_wallet(db, "Exchange");
        }
        let txs = vec![
            (
                1,
                make_crypto_tx(
                    "2024-06-15",
                    "Exchange",
                    "BTC",
                    "trade",
                    Some("buy"),
                    1.0,
                    Some(50000.0),
                    Some(0.10),
                    None,
                    None,
                ),
            ),
            (
                2,
                make_crypto_tx(
                    "2024-06-16",
                    "Exchange",
                    "ETH",
                    "trade",
                    Some("buy"),
                    10.0,
                    Some(3000.0),
                    Some(0.50),
                    None,
                    None,
                ),
            ),
            (
                3,
                make_crypto_tx(
                    "2024-06-17",
                    "Exchange",
                    "BTC",
                    "trade",
                    Some("sell"),
                    0.5,
                    Some(51000.0),
                    Some(0.10),
                    None,
                    None,
                ),
            ),
        ];
        let result = h
            .service
            .process_crypto_transactions_ext(txs, "Test Format", true)
            .expect("process");
        assert_eq!(result.inserted, 3);
        assert_eq!(result.errors, 0);
    }
}
