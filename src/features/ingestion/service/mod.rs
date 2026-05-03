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

//! Ingestion service
//!
//! Orchestrates data import from various file formats.

use crate::db::{Database, DbError};
use crate::models::CryptoTransaction;
use chrono::{NaiveDate, NaiveDateTime};

use std::sync::{Arc, Mutex};

use super::parsers::{
    CsvParser, ExchangeSource, ImportParser, JsonParser, TextParser, detect_exchange_source,
    detect_format, parser_for,
};
use super::types::{ImportFormat, ImportSummary};
use super::validation::validate_file_size;

mod crypto;
mod habits;
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

pub(super) fn kraken_trade_ref_key(wallet_id: &str, note: Option<&str>) -> Option<(String, String)> {
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

pub(super) fn notbank_trade_ref_key(wallet_id: &str, note: Option<&str>) -> Option<(String, String)> {
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

pub(super) fn notbank_transaction_ref_key(wallet_id: &str, note: Option<&str>) -> Option<(String, String)> {
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
    db: Arc<Mutex<Option<Database>>>,
}

impl IngestionService {
    pub fn new(db: Arc<Mutex<Option<Database>>>) -> Self {
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
            .lock()
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
            ImportFormat::CsvHabitLogs => self.import_csv_habit_logs(content),
            ImportFormat::CsvCrypto => self.import_csv_crypto(content),
            ImportFormat::TextMixed => self.import_text_mixed(content),
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
            ImportFormat::CsvHabitLogs => self.preview_csv_habit_logs(content),
            ImportFormat::CsvCrypto => self.preview_csv_crypto(content),
            ImportFormat::TextMixed => self.preview_text_mixed(content),
            ImportFormat::ExchangeCsv(source) => {
                let wallet = source.default_wallet_name();
                self.preview_exchange_csv(content, wallet, source)
            }
        }
    }

    /// Import JSON format (can contain transactions, habit logs, and crypto)
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

        // Import habit logs
        if !file.habit_logs.items.is_empty() || !file.habit_logs.errors.is_empty() {
            let log_summary =
                self.process_habit_logs(file.habit_logs.items, parser.format_name())?;
            summary.merge(log_summary);
            for error in file.habit_logs.errors {
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

    /// Preview JSON format (can contain transactions, habit logs, and crypto)
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

        if !file.habit_logs.items.is_empty() || !file.habit_logs.errors.is_empty() {
            let log_summary =
                self.preview_habit_logs(file.habit_logs.items, parser.format_name())?;
            summary.merge(log_summary);
            for error in file.habit_logs.errors {
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

    /// Import CSV habit logs
    fn import_csv_habit_logs(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = CsvParser;
        let parsed = parser
            .parse_habit_logs(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.process_habit_logs(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Preview CSV habit logs
    fn preview_csv_habit_logs(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = CsvParser;
        let parsed = parser
            .parse_habit_logs(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.preview_habit_logs(parsed.items, parser.format_name())?;
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

    /// Import text mixed content (T;, H;, C; prefixes)
    fn import_text_mixed(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = TextParser;
        let mut summary = ImportSummary::new("Plain Text", "Mixed");
        let parsed = parser.parse_mixed(content);

        // Import transactions
        if !parsed.transactions.items.is_empty() || !parsed.transactions.errors.is_empty() {
            let tx_summary =
                self.process_transactions(parsed.transactions.items, parser.format_name())?;
            summary.merge(tx_summary);
            for error in parsed.transactions.errors {
                summary.record_error(error);
            }
        }

        // Import habit logs
        if !parsed.habit_logs.items.is_empty() || !parsed.habit_logs.errors.is_empty() {
            let log_summary =
                self.process_habit_logs(parsed.habit_logs.items, parser.format_name())?;
            summary.merge(log_summary);
            for error in parsed.habit_logs.errors {
                summary.record_error(error);
            }
        }

        // Import crypto transactions
        if !parsed.crypto_transactions.items.is_empty()
            || !parsed.crypto_transactions.errors.is_empty()
        {
            let crypto_summary = self.process_crypto_transactions(
                parsed.crypto_transactions.items,
                parser.format_name(),
            )?;
            summary.merge(crypto_summary);
            for error in parsed.crypto_transactions.errors {
                summary.record_error(error);
            }
        }

        Ok(summary)
    }

    /// Preview text mixed content
    fn preview_text_mixed(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = TextParser;
        let mut summary = ImportSummary::new("Plain Text", "Mixed");
        let parsed = parser.parse_mixed(content);

        if !parsed.transactions.items.is_empty() || !parsed.transactions.errors.is_empty() {
            let tx_summary =
                self.preview_transactions(parsed.transactions.items, parser.format_name())?;
            summary.merge(tx_summary);
            for error in parsed.transactions.errors {
                summary.record_error(error);
            }
        }

        if !parsed.habit_logs.items.is_empty() || !parsed.habit_logs.errors.is_empty() {
            let log_summary =
                self.preview_habit_logs(parsed.habit_logs.items, parser.format_name())?;
            summary.merge(log_summary);
            for error in parsed.habit_logs.errors {
                summary.record_error(error);
            }
        }

        if !parsed.crypto_transactions.items.is_empty()
            || !parsed.crypto_transactions.errors.is_empty()
        {
            let crypto_summary = self.preview_crypto_transactions(
                parsed.crypto_transactions.items,
                parser.format_name(),
            )?;
            summary.merge(crypto_summary);
            for error in parsed.crypto_transactions.errors {
                summary.record_error(error);
            }
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
            notbank_trade_ref_key("wallet-1", Some("NotBank trade | Buy BTCUSDT | trade_id=300001")),
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
        assert!(note_is_exchange_overlap_prone(Some("Kraken trade | Ref: TX-1")));
        assert!(note_is_exchange_overlap_prone(Some("Binance Spot BUY | BTCUSDT")));
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
