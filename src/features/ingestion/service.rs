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
use crate::features::crypto::tax::types::{derive_mechanical_type, normalize_subtype};
use crate::models::{CryptoTransaction, HabitLog, Transaction};
use crate::services::i18n::{t, t_args};
use chrono::{NaiveDate, NaiveDateTime};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::parsers::{
    CsvParser, ExchangeSource, ImportParser, JsonParser, TextParser, detect_exchange_source,
    detect_format, parser_for,
};
use super::repository::IngestionRepository;
use super::types::{
    CryptoDedupKey, ImportCryptoTransaction, ImportFormat, ImportHabitLog, ImportSummary,
    ImportTransaction, RowError, TransactionDedupKey,
};
use super::validation::{
    validate_amount, validate_file_size, validate_import_crypto_transaction,
    validate_import_habit_log, validate_import_transaction,
};

fn format_currency_simple(cents: i64, currency: &str) -> String {
    let amount = (cents.abs() as f64) / 100.0;
    format!("{:.2} {}", amount, currency)
}

fn normalize_import_symbol_key(raw: &str) -> String {
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

fn matches_swap_rollup_duplicate(existing_amounts: &[f64], import_amount: f64) -> bool {
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

fn uses_price_agnostic_dedup(format_name: &str) -> bool {
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

fn note_is_exchange_overlap_prone(note: Option<&str>) -> bool {
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

fn kraken_trade_ref_key(wallet_id: &str, note: Option<&str>) -> Option<(String, String)> {
    let reference = note.and_then(extract_kraken_trade_ref)?;
    Some((wallet_id.to_string(), reference))
}

fn has_mexc_transfer_overlap_duplicate(
    existing: &[CryptoTransaction],
    format_name: &str,
    wallet_id: &str,
    coin_id: &str,
    mechanical_type: &str,
    amount: f64,
    fee_amount: Option<f64>,
    date: &str,
) -> bool {
    if mechanical_type != "transfer_in" && mechanical_type != "transfer_out" {
        return false;
    }

    let incoming_is_statement = format_name.eq_ignore_ascii_case("MEXC Statement History");
    let incoming_is_deposit = format_name.eq_ignore_ascii_case("MEXC Deposit History");
    let incoming_is_withdrawal = format_name.eq_ignore_ascii_case("MEXC Withdrawal History");

    let required_existing_note_prefix = if incoming_is_statement {
        if mechanical_type == "transfer_in" {
            "mexc deposit"
        } else {
            "mexc withdrawal"
        }
    } else if incoming_is_deposit || incoming_is_withdrawal {
        "mexc statement"
    } else {
        return false;
    };

    let import_dt = match parse_ingestion_datetime(date) {
        Some(dt) => dt,
        None => return false,
    };

    const OVERLAP_WINDOW_SECONDS: i64 = 15 * 60;

    existing.iter().any(|tx| {
        if tx.wallet_id != wallet_id || tx.coin_id != coin_id {
            return false;
        }
        if tx.mechanical_type() != mechanical_type {
            return false;
        }
        if (tx.amount - amount).abs() > 1e-8 {
            return false;
        }

        if fee_amount.is_some() || tx.fee_amount.is_some() {
            match (fee_amount, tx.fee_amount) {
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
    /// path (with default wallet name) and the dedicated exchange import callback
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

    /// Process and insert transactions (with validation and deduplication)
    fn process_transactions(
        &self,
        transactions: Vec<(usize, ImportTransaction)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_transactions_internal(transactions, format_name, false)
    }

    /// Preview transactions (validation and deduplication without inserts)
    fn preview_transactions(
        &self,
        transactions: Vec<(usize, ImportTransaction)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_transactions_internal(transactions, format_name, true)
    }

    fn process_transactions_internal(
        &self,
        transactions: Vec<(usize, ImportTransaction)>,
        format_name: &str,
        dry_run: bool,
    ) -> Result<ImportSummary, IngestionError> {
        if dry_run {
            self.with_db_readonly(|db| {
                self.process_transactions_with_db(db, transactions, format_name, dry_run)
            })
        } else {
            self.with_db(|db| {
                self.process_transactions_with_db(db, transactions, format_name, dry_run)
            })
        }
    }

    fn process_transactions_with_db(
        &self,
        db: &Database,
        transactions: Vec<(usize, ImportTransaction)>,
        format_name: &str,
        dry_run: bool,
    ) -> Result<ImportSummary, IngestionError> {
        let mut summary = ImportSummary::new(format_name, "Transactions");
        let skipped_duplicate = t("import-skipped-duplicate-transaction");

        let account_lookup =
            IngestionRepository::build_account_lookup(db).map_err(IngestionError::Database)?;
        let category_lookup =
            IngestionRepository::build_category_lookup(db).map_err(IngestionError::Database)?;

        let existing =
            IngestionRepository::get_all_transactions(db).map_err(IngestionError::Database)?;
        let mut dedup_set: HashSet<TransactionDedupKey> = existing
            .iter()
            .filter_map(|tx| {
                let account = account_lookup.values().find(|a| a.id == tx.account_id)?;
                Some(TransactionDedupKey::new(
                    &tx.date,
                    &tx.account_id,
                    tx.transfer_account_id.as_deref(),
                    &account.currency,
                    tx.amount,
                    &tx.transaction_type,
                    &tx.description,
                ))
            })
            .collect();

        for (line_num, import_tx) in transactions {
            if let Err(mut error) = validate_import_transaction(&import_tx, line_num) {
                error.raw_data = Some(format!("{:?}", import_tx));
                summary.record_error(error);
                continue;
            }

            let account_key = import_tx.account.trim().to_lowercase();
            let account = match account_lookup.get(&account_key) {
                Some(a) => a,
                None => {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("account"),
                        t_args(
                            "import-error-account-not-found",
                            &[("name", import_tx.account.trim())],
                        ),
                    ));
                    continue;
                }
            };

            let import_currency = import_tx.currency.trim().to_uppercase();
            if account.currency.to_uppercase() != import_currency {
                summary.record_error(RowError::new(
                    line_num,
                    Some("currency"),
                    t_args(
                        "import-error-currency-mismatch-detail",
                        &[
                            ("account", account.name.as_str()),
                            ("import", import_currency.as_str()),
                            ("expected", account.currency.as_str()),
                        ],
                    ),
                ));
                continue;
            }

            let tx_type = import_tx.transaction_type.trim().to_lowercase();
            let category_type = if tx_type == "income" {
                "income"
            } else {
                "expense"
            };

            if tx_type != "transfer" {
                let category_key = (
                    import_tx.category.trim().to_lowercase(),
                    category_type.to_string(),
                );
                if !category_lookup.contains_key(&category_key) {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("category"),
                        t_args(
                            "import-error-category-not-found-detail",
                            &[("name", import_tx.category.trim()), ("type", category_type)],
                        ),
                    ));
                    continue;
                }
            }

            let amount_cents = match validate_amount(import_tx.amount) {
                Ok(c) => c,
                Err(e) => {
                    summary.record_error(RowError::new(line_num, Some("amount"), e));
                    continue;
                }
            };

            if tx_type == "transfer" {
                let Some(dest_name) = import_tx.transfer_to_account.as_ref() else {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("transfer_to_account"),
                        "Transfer transactions require a destination account",
                    ));
                    continue;
                };
                let dest_key = dest_name.trim().to_lowercase();
                let dest_account = match account_lookup.get(&dest_key) {
                    Some(a) => a,
                    None => {
                        summary.record_error(RowError::new(
                            line_num,
                            Some("transfer_to_account"),
                            t_args(
                                "import-error-destination-account-not-found",
                                &[("name", dest_name.trim())],
                            ),
                        ));
                        continue;
                    }
                };

                if dest_account.id == account.id {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("transfer_to_account"),
                        t("import-error-same-account-transfer"),
                    ));
                    continue;
                }

                let dedup_key = TransactionDedupKey::new(
                    &import_tx.date,
                    &account.id,
                    Some(dest_account.id.as_str()),
                    &account.currency,
                    amount_cents,
                    &tx_type,
                    &import_tx.description,
                );

                if dedup_set.contains(&dedup_key) {
                    summary.record_skipped(&skipped_duplicate);
                    continue;
                }

                if dry_run {
                    dedup_set.insert(dedup_key);
                    let amount_fmt = format!(
                        "{:.2} {}",
                        (amount_cents.abs() as f64) / 100.0,
                        account.currency
                    );
                    summary.record_preview_change(
                        &t("import-preview-change-transfer"),
                        if amount_cents < 0 {
                            format!("- {}", amount_fmt)
                        } else {
                            amount_fmt
                        },
                        format!(
                            "{} -> {} ({})",
                            account.name, dest_account.name, import_tx.description
                        ),
                    );
                    continue;
                }

                match IngestionRepository::create_transfer(
                    db,
                    &account.id,
                    &dest_account.id,
                    amount_cents,
                    import_tx.description.trim(),
                    import_tx.date.trim(),
                ) {
                    Ok(_) => {
                        dedup_set.insert(dedup_key);
                        summary.record_inserted();
                    }
                    Err(e) => {
                        summary.record_error(RowError::new(
                            line_num,
                            None,
                            format!("Database error: {}", e),
                        ));
                    }
                }
            } else {
                let dedup_key = TransactionDedupKey::new(
                    &import_tx.date,
                    &account.id,
                    None,
                    &account.currency,
                    amount_cents,
                    &tx_type,
                    &import_tx.description,
                );

                if dedup_set.contains(&dedup_key) {
                    summary.record_skipped(&skipped_duplicate);
                    continue;
                }

                if dry_run {
                    dedup_set.insert(dedup_key);
                    let amount_fmt = format_currency_simple(amount_cents, &account.currency);
                    let type_label = if tx_type == "income" {
                        t("import-preview-change-income")
                    } else {
                        t("import-preview-change-expense")
                    };

                    summary.record_preview_change(
                        &type_label,
                        if tx_type == "expense" {
                            format!("- {}", amount_fmt)
                        } else {
                            format!("+ {}", amount_fmt)
                        },
                        format!(
                            "{} - {} ({})",
                            account.name, import_tx.category, import_tx.description
                        ),
                    );
                    continue;
                }

                let transaction = Transaction::new(
                    Uuid::new_v4().to_string(),
                    account.id.clone(),
                    amount_cents,
                    import_tx.category.trim().to_string(),
                    import_tx.description.trim().to_string(),
                    import_tx.date.trim().to_string(),
                    tx_type.clone(),
                    None,
                );

                match IngestionRepository::create_transaction(db, &transaction) {
                    Ok(_) => {
                        dedup_set.insert(dedup_key);
                        summary.record_inserted();
                    }
                    Err(e) => {
                        summary.record_error(RowError::new(
                            line_num,
                            None,
                            format!("Database error: {}", e),
                        ));
                    }
                }
            }
        }

        Ok(summary)
    }

    /// Process and insert habit logs (with validation and deduplication)
    fn process_habit_logs(
        &self,
        logs: Vec<(usize, ImportHabitLog)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_habit_logs_internal(logs, format_name, false)
    }

    /// Preview habit logs (validation and deduplication without inserts)
    fn preview_habit_logs(
        &self,
        logs: Vec<(usize, ImportHabitLog)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_habit_logs_internal(logs, format_name, true)
    }

    fn process_habit_logs_internal(
        &self,
        logs: Vec<(usize, ImportHabitLog)>,
        format_name: &str,
        dry_run: bool,
    ) -> Result<ImportSummary, IngestionError> {
        if dry_run {
            self.with_db_readonly(|db| {
                self.process_habit_logs_with_db(db, logs, format_name, dry_run)
            })
        } else {
            self.with_db(|db| self.process_habit_logs_with_db(db, logs, format_name, dry_run))
        }
    }

    fn process_habit_logs_with_db(
        &self,
        db: &Database,
        logs: Vec<(usize, ImportHabitLog)>,
        format_name: &str,
        dry_run: bool,
    ) -> Result<ImportSummary, IngestionError> {
        let mut summary = ImportSummary::new(format_name, "Habit Logs");
        let skipped_not_completed = t("import-skipped-habit-not-completed");
        let skipped_already_logged = t("import-skipped-habit-already-logged");

        let habit_lookup =
            IngestionRepository::build_habit_lookup(db).map_err(IngestionError::Database)?;
        let mut seen_logs: HashSet<(String, String)> = HashSet::new();

        for (line_num, import_log) in logs {
            if !import_log.completed {
                summary.record_skipped(&skipped_not_completed);
                continue;
            }

            if let Err(mut error) = validate_import_habit_log(&import_log, line_num) {
                error.raw_data = Some(format!("{:?}", import_log));
                summary.record_error(error);
                continue;
            }

            let habit_key = import_log.habit.trim().to_lowercase();
            let habit = match habit_lookup.get(&habit_key) {
                Some(h) => h,
                None => {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("habit"),
                        t_args(
                            "import-error-habit-not-found",
                            &[("name", import_log.habit.trim())],
                        ),
                    ));
                    continue;
                }
            };

            let date = import_log.date.trim();
            let dedup_key = (habit.id.clone(), date.to_string());
            if seen_logs.contains(&dedup_key) {
                summary.record_skipped(&skipped_already_logged);
                continue;
            }

            match IngestionRepository::habit_log_exists(db, &habit.id, date) {
                Ok(true) => {
                    seen_logs.insert(dedup_key);
                    summary.record_skipped(&skipped_already_logged);
                    continue;
                }
                Ok(false) => {}
                Err(e) => {
                    summary.record_error(RowError::new(
                        line_num,
                        None,
                        format!("Database error: {}", e),
                    ));
                    continue;
                }
            }

            if dry_run {
                seen_logs.insert(dedup_key);
                summary.record_preview_change(
                    &t("import-preview-change-habit"),
                    habit.name.clone(),
                    date.to_string(),
                );
                continue;
            }

            let log = HabitLog::new(
                Uuid::new_v4().to_string(),
                habit.id.clone(),
                date.to_string(),
            );

            match IngestionRepository::create_habit_log(db, &log) {
                Ok(_) => {
                    seen_logs.insert(dedup_key);
                    summary.record_inserted();
                }
                Err(e) => {
                    summary.record_error(RowError::new(
                        line_num,
                        None,
                        format!("Database error: {}", e),
                    ));
                }
            }
        }

        Ok(summary)
    }

    /// Process and insert crypto transactions (with validation and deduplication)
    fn process_crypto_transactions(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(transactions, format_name, false, false)
    }

    /// Preview crypto transactions (validation and deduplication without inserts)
    fn preview_crypto_transactions(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(transactions, format_name, true, false)
    }

    /// Process crypto transactions, optionally skipping balance validation.
    /// Used by exchange imports where the wallet export is authoritative.
    fn process_crypto_transactions_ext(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        skip_balance_validation: bool,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(
            transactions,
            format_name,
            false,
            skip_balance_validation,
        )
    }

    /// Preview crypto transactions, optionally skipping balance validation.
    fn preview_crypto_transactions_ext(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        skip_balance_validation: bool,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(
            transactions,
            format_name,
            true,
            skip_balance_validation,
        )
    }

    fn process_crypto_transactions_internal(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        dry_run: bool,
        skip_balance_validation: bool,
    ) -> Result<ImportSummary, IngestionError> {
        if dry_run {
            self.with_db_readonly(|db| {
                self.process_crypto_transactions_with_db(
                    db,
                    transactions,
                    format_name,
                    dry_run,
                    skip_balance_validation,
                )
            })
        } else {
            self.with_db(|db| {
                self.process_crypto_transactions_with_db(
                    db,
                    transactions,
                    format_name,
                    dry_run,
                    skip_balance_validation,
                )
            })
        }
    }

    fn process_crypto_transactions_with_db(
        &self,
        db: &Database,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        dry_run: bool,
        skip_balance_validation: bool,
    ) -> Result<ImportSummary, IngestionError> {
        // Sort transactions by date to ensure chronological processing.
        // This prevents balance validation failures when the CSV has rows
        // out of order (e.g. a withdrawal before the deposit that funds it).
        let mut transactions = transactions;
        transactions.sort_by(|a, b| a.1.date.cmp(&b.1.date));

        let mut summary = ImportSummary::new(format_name, "Crypto");
        let skipped_duplicate = t("import-skipped-duplicate-crypto");
        let skipped_missing_coin = t("import-skipped-crypto-not-found");
        let is_mexc_spot_order_history_source =
            format_name.eq_ignore_ascii_case("MEXC Spot Trade History");
        let use_price_agnostic_dedup = uses_price_agnostic_dedup(format_name);

        let wallet_lookup =
            IngestionRepository::build_wallet_lookup(db).map_err(IngestionError::Database)?;
        let coin_lookup =
            IngestionRepository::build_coin_lookup(db).map_err(IngestionError::Database)?;

        let existing = IngestionRepository::get_all_crypto_transactions(db)
            .map_err(IngestionError::Database)?;
        let existing_map: HashMap<String, &CryptoTransaction> =
            existing.iter().map(|tx| (tx.id.clone(), tx)).collect();
        let mut existing_swap_amounts: HashMap<(String, String, String, String), Vec<f64>> =
            HashMap::new();
        if is_mexc_spot_order_history_source {
            for tx in &existing {
                if tx.mechanical_type() != "swap" {
                    continue;
                }
                let pair_coin_id = tx
                    .related_tx_id
                    .as_ref()
                    .and_then(|id| existing_map.get(id))
                    .map(|related| related.coin_id.as_str());
                if let Some(pair_coin_id) = pair_coin_id {
                    existing_swap_amounts
                        .entry((
                            tx.date.clone(),
                            tx.wallet_id.clone(),
                            tx.coin_id.clone(),
                            pair_coin_id.to_string(),
                        ))
                        .or_default()
                        .push(tx.amount);
                }
            }
        }

        let mut dedup_set: HashSet<CryptoDedupKey> = existing
            .iter()
            .map(|tx| {
                let pair_coin_id = if tx.mechanical_type() == "swap" {
                    tx.related_tx_id
                        .as_ref()
                        .and_then(|id| existing_map.get(id))
                        .map(|related| related.coin_id.as_str())
                } else {
                    None
                };
                CryptoDedupKey::new(
                    &tx.date,
                    &tx.wallet_id,
                    &tx.coin_id,
                    tx.mechanical_type(),
                    &tx.transaction_type,
                    tx.subtype.as_deref(),
                    tx.amount,
                    tx.price_per_coin,
                    pair_coin_id,
                )
            })
            .collect();
        let mut dedup_set_price_agnostic: HashSet<CryptoDedupKey> = existing
            .iter()
            .filter(|tx| note_is_exchange_overlap_prone(tx.notes.as_deref()))
            .map(|tx| {
                let pair_coin_id = if tx.mechanical_type() == "swap" {
                    tx.related_tx_id
                        .as_ref()
                        .and_then(|id| existing_map.get(id))
                        .map(|related| related.coin_id.as_str())
                } else {
                    None
                };
                CryptoDedupKey::new(
                    &tx.date,
                    &tx.wallet_id,
                    &tx.coin_id,
                    tx.mechanical_type(),
                    &tx.transaction_type,
                    tx.subtype.as_deref(),
                    tx.amount,
                    None,
                    pair_coin_id,
                )
            })
            .collect();
        let mut kraken_trade_ref_set: HashSet<(String, String)> = existing
            .iter()
            .filter_map(|tx| kraken_trade_ref_key(&tx.wallet_id, tx.notes.as_deref()))
            .collect();

        // Track pending balance changes for dry_run validation
        // Key: (wallet_id, coin_id), Value: pending balance delta
        let mut pending_balance_changes: std::collections::HashMap<(String, String), f64> =
            std::collections::HashMap::new();

        for (line_num, import_tx) in transactions {
            if let Err(mut error) = validate_import_crypto_transaction(&import_tx, line_num) {
                error.raw_data = Some(format!("{:?}", import_tx));
                summary.record_error(error);
                continue;
            }

            let category_type = import_tx.transaction_type.trim().to_lowercase();
            let normalized_subtype = import_tx
                .subtype
                .as_deref()
                .and_then(|s| normalize_subtype(&category_type, s));
            let mechanical_type =
                derive_mechanical_type(&category_type, normalized_subtype.as_deref());

            // Resolve wallet
            let wallet_key = import_tx.wallet.trim().to_lowercase();
            let wallet = match wallet_lookup.get(&wallet_key) {
                Some(w) => w,
                None => {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("wallet"),
                        t_args(
                            "import-error-wallet-not-found",
                            &[("name", import_tx.wallet.trim())],
                        ),
                    ));
                    continue;
                }
            };
            let kraken_ref_key = kraken_trade_ref_key(&wallet.id, import_tx.notes.as_deref());
            if let Some(ref_key) = kraken_ref_key.as_ref() {
                if kraken_trade_ref_set.contains(ref_key) {
                    summary.record_skipped(&skipped_duplicate);
                    continue;
                }
            }

            // Resolve coin (source)
            let symbol_key = normalize_import_symbol_key(&import_tx.symbol);
            let coin = match coin_lookup.get(&symbol_key) {
                Some(c) => c,
                None => {
                    summary.record_skipped(&skipped_missing_coin);
                    continue;
                }
            };

            let mut swap_to_coin = None;
            if mechanical_type == "swap" {
                let to_symbol = import_tx
                    .swap_to_symbol
                    .as_ref()
                    .map(|s| s.trim())
                    .unwrap_or("");
                let to_key = normalize_import_symbol_key(to_symbol);
                swap_to_coin = match coin_lookup.get(&to_key) {
                    Some(c) => Some(c),
                    None => {
                        summary.record_skipped(&skipped_missing_coin);
                        continue;
                    }
                };
            }

            // Resolve fee coin for ALL transaction types (not just swaps)
            let mut fee_coin = None;
            if let Some(symbol) = import_tx
                .fee_coin_symbol
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            {
                let fee_key = normalize_import_symbol_key(symbol);
                fee_coin = match coin_lookup.get(&fee_key) {
                    Some(c) => Some(c),
                    None => {
                        summary.record_skipped(&skipped_missing_coin);
                        continue;
                    }
                };
            }

            // Normalize fee_coin_id/fee_amount pair (require both or neither)
            let (resolved_fee_coin_id, resolved_fee_amount) = match (
                fee_coin.as_ref().map(|c| c.id.clone()),
                import_tx.fee_amount,
            ) {
                (Some(id), Some(amount)) if amount > 0.0 => (Some(id), Some(amount)),
                (None, Some(_)) => {
                    // fee_amount without fee_coin_symbol: already caught in validation
                    (None, None)
                }
                _ => (None, None),
            };

            if has_mexc_transfer_overlap_duplicate(
                &existing,
                format_name,
                &wallet.id,
                &coin.id,
                mechanical_type,
                import_tx.amount,
                resolved_fee_amount,
                &import_tx.date,
            ) {
                summary.record_skipped(&skipped_duplicate);
                continue;
            }

            // Validate balance for outflow operations
            // Skipped for exchange imports — the wallet export is authoritative.
            if !skip_balance_validation
                && (mechanical_type == "sell" || mechanical_type == "transfer_out")
            {
                let db_balance = match IngestionRepository::get_wallet_coin_balance(
                    db,
                    &wallet.id,
                    &coin.id,
                    import_tx.date.trim(),
                ) {
                    Ok(b) => b,
                    Err(e) => {
                        summary.record_error(RowError::new(
                            line_num,
                            None,
                            format!("Database error checking balance: {}", e),
                        ));
                        continue;
                    }
                };

                // Include pending changes from previous transactions in this import batch
                let balance_key = (wallet.id.clone(), coin.id.clone());
                let pending_delta = pending_balance_changes
                    .get(&balance_key)
                    .copied()
                    .unwrap_or(0.0);
                let available_balance = db_balance + pending_delta;

                if available_balance < import_tx.amount {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("amount"),
                        t_args(
                            "import-error-insufficient-crypto-balance",
                            &[
                                ("symbol", coin.symbol.as_str()),
                                ("wallet", wallet.name.as_str()),
                                ("available", &format!("{:.8}", available_balance)),
                                ("required", &format!("{:.8}", import_tx.amount)),
                            ],
                        ),
                    ));
                    continue;
                }

                // Validate fee balance for non-swap outflows with fee in same coin
                if let (Some(fee_coin_ref), Some(fee_amt)) =
                    (resolved_fee_coin_id.as_deref(), resolved_fee_amount)
                {
                    if fee_coin_ref == coin.id {
                        // Fee is in the same coin as the main outflow
                        let total_required = import_tx.amount + fee_amt;
                        if available_balance < total_required {
                            summary.record_error(RowError::new(
                                line_num,
                                Some("fee_amount"),
                                t_args(
                                    "import-error-insufficient-crypto-balance",
                                    &[
                                        ("symbol", coin.symbol.as_str()),
                                        ("wallet", wallet.name.as_str()),
                                        ("available", &format!("{:.8}", available_balance)),
                                        ("required", &format!("{:.8}", total_required)),
                                    ],
                                ),
                            ));
                            continue;
                        }
                    } else {
                        // Fee is in a different coin — check that coin's balance
                        let fee_db_balance = match IngestionRepository::get_wallet_coin_balance(
                            db,
                            &wallet.id,
                            fee_coin_ref,
                            import_tx.date.trim(),
                        ) {
                            Ok(b) => b,
                            Err(e) => {
                                summary.record_error(RowError::new(
                                    line_num,
                                    None,
                                    format!("Database error checking fee balance: {}", e),
                                ));
                                continue;
                            }
                        };
                        let fee_key = (wallet.id.clone(), fee_coin_ref.to_string());
                        let fee_pending = pending_balance_changes
                            .get(&fee_key)
                            .copied()
                            .unwrap_or(0.0);
                        let available_fee = fee_db_balance + fee_pending;
                        if fee_amt > available_fee {
                            summary.record_error(RowError::new(
                                line_num,
                                Some("fee_amount"),
                                t_args(
                                    "import-error-insufficient-crypto-balance",
                                    &[
                                        (
                                            "symbol",
                                            fee_coin
                                                .as_ref()
                                                .map(|c| c.symbol.as_str())
                                                .unwrap_or("?"),
                                        ),
                                        ("wallet", wallet.name.as_str()),
                                        ("available", &format!("{:.8}", available_fee)),
                                        ("required", &format!("{:.8}", fee_amt)),
                                    ],
                                ),
                            ));
                            continue;
                        }
                    }
                }
            } else if !skip_balance_validation
                && (mechanical_type == "buy" || mechanical_type == "transfer_in")
            {
                // For inflows, validate fee balance if fee is in a different coin
                if let (Some(fee_coin_ref), Some(fee_amt)) =
                    (resolved_fee_coin_id.as_deref(), resolved_fee_amount)
                {
                    if fee_coin_ref == coin.id {
                        // Fee is in same coin as inflow — after the buy we get `amount`,
                        // but we also need to pay `fee_amt` from that coin.
                        let db_balance = match IngestionRepository::get_wallet_coin_balance(
                            db,
                            &wallet.id,
                            &coin.id,
                            import_tx.date.trim(),
                        ) {
                            Ok(b) => b,
                            Err(e) => {
                                summary.record_error(RowError::new(
                                    line_num,
                                    None,
                                    format!("Database error checking balance: {}", e),
                                ));
                                continue;
                            }
                        };
                        let balance_key = (wallet.id.clone(), coin.id.clone());
                        let pending_delta = pending_balance_changes
                            .get(&balance_key)
                            .copied()
                            .unwrap_or(0.0);
                        let available = db_balance + pending_delta + import_tx.amount;
                        if fee_amt > available {
                            summary.record_error(RowError::new(
                                line_num,
                                Some("fee_amount"),
                                "Fee amount exceeds available balance after inflow".to_string(),
                            ));
                            continue;
                        }
                    } else {
                        let fee_db_balance = match IngestionRepository::get_wallet_coin_balance(
                            db,
                            &wallet.id,
                            fee_coin_ref,
                            import_tx.date.trim(),
                        ) {
                            Ok(b) => b,
                            Err(e) => {
                                summary.record_error(RowError::new(
                                    line_num,
                                    None,
                                    format!("Database error checking fee balance: {}", e),
                                ));
                                continue;
                            }
                        };
                        let fee_key = (wallet.id.clone(), fee_coin_ref.to_string());
                        let fee_pending = pending_balance_changes
                            .get(&fee_key)
                            .copied()
                            .unwrap_or(0.0);
                        let available_fee = fee_db_balance + fee_pending;
                        if fee_amt > available_fee {
                            summary.record_error(RowError::new(
                                line_num,
                                Some("fee_amount"),
                                t_args(
                                    "import-error-insufficient-crypto-balance",
                                    &[
                                        (
                                            "symbol",
                                            fee_coin
                                                .as_ref()
                                                .map(|c| c.symbol.as_str())
                                                .unwrap_or("?"),
                                        ),
                                        ("wallet", wallet.name.as_str()),
                                        ("available", &format!("{:.8}", available_fee)),
                                        ("required", &format!("{:.8}", fee_amt)),
                                    ],
                                ),
                            ));
                            continue;
                        }
                    }
                }
            }

            if mechanical_type == "swap" {
                let to_coin = match swap_to_coin {
                    Some(c) => c,
                    None => continue,
                };
                let to_amount = import_tx.swap_to_amount.unwrap_or(0.0);
                let fee_amount = import_tx.fee_amount.unwrap_or(0.0);

                if !skip_balance_validation {
                    let db_balance = match IngestionRepository::get_wallet_coin_balance(
                        db,
                        &wallet.id,
                        &coin.id,
                        import_tx.date.trim(),
                    ) {
                        Ok(b) => b,
                        Err(e) => {
                            summary.record_error(RowError::new(
                                line_num,
                                None,
                                format!("Database error checking balance: {}", e),
                            ));
                            continue;
                        }
                    };
                    let balance_key = (wallet.id.clone(), coin.id.clone());
                    let pending_delta = pending_balance_changes
                        .get(&balance_key)
                        .copied()
                        .unwrap_or(0.0);
                    let mut required_from = import_tx.amount;

                    if let Some(fee_coin) = fee_coin.as_ref()
                        && fee_coin.id == coin.id
                    {
                        required_from += fee_amount;
                    }

                    let available_from = db_balance + pending_delta;
                    if available_from < required_from {
                        summary.record_error(RowError::new(
                            line_num,
                            Some("amount"),
                            t_args(
                                "import-error-insufficient-crypto-balance",
                                &[
                                    ("symbol", coin.symbol.as_str()),
                                    ("wallet", wallet.name.as_str()),
                                    ("available", &format!("{:.8}", available_from)),
                                    ("required", &format!("{:.8}", required_from)),
                                ],
                            ),
                        ));
                        continue;
                    }

                    if let Some(fee_coin) = fee_coin.as_ref() {
                        if fee_coin.id == to_coin.id {
                            let to_balance = match IngestionRepository::get_wallet_coin_balance(
                                db,
                                &wallet.id,
                                &to_coin.id,
                                import_tx.date.trim(),
                            ) {
                                Ok(b) => b,
                                Err(e) => {
                                    summary.record_error(RowError::new(
                                        line_num,
                                        None,
                                        format!("Database error checking balance: {}", e),
                                    ));
                                    continue;
                                }
                            };
                            let to_key = (wallet.id.clone(), to_coin.id.clone());
                            let to_pending =
                                pending_balance_changes.get(&to_key).copied().unwrap_or(0.0);
                            let available_to = to_balance + to_pending + to_amount;
                            if fee_amount > available_to {
                                summary.record_error(RowError::new(
                                    line_num,
                                    Some("fee_amount"),
                                    "Fee amount exceeds available output balance".to_string(),
                                ));
                                continue;
                            }
                        } else if fee_coin.id != coin.id {
                            let fee_balance = match IngestionRepository::get_wallet_coin_balance(
                                db,
                                &wallet.id,
                                &fee_coin.id,
                                import_tx.date.trim(),
                            ) {
                                Ok(b) => b,
                                Err(e) => {
                                    summary.record_error(RowError::new(
                                        line_num,
                                        None,
                                        format!("Database error checking balance: {}", e),
                                    ));
                                    continue;
                                }
                            };
                            let fee_key = (wallet.id.clone(), fee_coin.id.clone());
                            let fee_pending = pending_balance_changes
                                .get(&fee_key)
                                .copied()
                                .unwrap_or(0.0);
                            let available_fee = fee_balance + fee_pending;
                            if fee_amount > available_fee {
                                summary.record_error(RowError::new(
                                    line_num,
                                    Some("fee_amount"),
                                    t_args(
                                        "import-error-insufficient-crypto-balance",
                                        &[
                                            ("symbol", fee_coin.symbol.as_str()),
                                            ("wallet", wallet.name.as_str()),
                                            ("available", &format!("{:.8}", available_fee)),
                                            ("required", &format!("{:.8}", fee_amount)),
                                        ],
                                    ),
                                ));
                                continue;
                            }
                        }
                    }
                } // end !skip_balance_validation for swap
            }

            let dedup_key = if mechanical_type == "swap" {
                let to_coin = match swap_to_coin {
                    Some(c) => c,
                    None => continue,
                };
                if is_mexc_spot_order_history_source {
                    let key = (
                        import_tx.date.clone(),
                        wallet.id.clone(),
                        coin.id.clone(),
                        to_coin.id.clone(),
                    );
                    if let Some(existing_amounts) = existing_swap_amounts.get(&key)
                        && matches_swap_rollup_duplicate(existing_amounts, import_tx.amount)
                    {
                        summary.record_skipped(&skipped_duplicate);
                        continue;
                    }
                }
                // Match the same source-side price normalisation used when persisting swaps.
                let source_price_for_dedup = import_tx.price_per_coin.or_else(|| {
                    let to_amount = import_tx.swap_to_amount.unwrap_or(0.0);
                    if import_tx.amount > 0.0 && to_amount > 0.0 {
                        Some(to_amount / import_tx.amount)
                    } else {
                        None
                    }
                });
                let from_key = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.amount,
                    source_price_for_dedup,
                    Some(&to_coin.id),
                );
                let from_key_price_agnostic = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.amount,
                    None,
                    Some(&to_coin.id),
                );
                let to_key = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &to_coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.swap_to_amount.unwrap_or(0.0),
                    None,
                    Some(&coin.id),
                );
                if dedup_set.contains(&from_key)
                    || dedup_set.contains(&to_key)
                    || (use_price_agnostic_dedup
                        && (dedup_set_price_agnostic.contains(&from_key_price_agnostic)
                            || dedup_set_price_agnostic.contains(&to_key)))
                {
                    summary.record_skipped(&skipped_duplicate);
                    continue;
                }
                // stash both keys later
                Some((from_key, to_key))
            } else {
                let key = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.amount,
                    import_tx.price_per_coin,
                    None,
                );
                let key_price_agnostic = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.amount,
                    None,
                    None,
                );
                if dedup_set.contains(&key)
                    || (use_price_agnostic_dedup
                        && dedup_set_price_agnostic.contains(&key_price_agnostic))
                {
                    summary.record_skipped(&skipped_duplicate);
                    continue;
                }
                None
            };

            // Update pending balance changes for subsequent validations
            if mechanical_type == "swap" {
                let to_coin = match swap_to_coin {
                    Some(c) => c,
                    None => continue,
                };
                let to_amount = import_tx.swap_to_amount.unwrap_or(0.0);
                *pending_balance_changes
                    .entry((wallet.id.clone(), coin.id.clone()))
                    .or_insert(0.0) -= import_tx.amount;
                *pending_balance_changes
                    .entry((wallet.id.clone(), to_coin.id.clone()))
                    .or_insert(0.0) += to_amount;

                if let (Some(fee_coin), Some(fee_amount)) =
                    (fee_coin.as_ref(), import_tx.fee_amount)
                {
                    *pending_balance_changes
                        .entry((wallet.id.clone(), fee_coin.id.clone()))
                        .or_insert(0.0) -= fee_amount;
                }
            } else {
                let balance_key = (wallet.id.clone(), coin.id.clone());
                let delta = match mechanical_type {
                    "buy" | "transfer_in" => import_tx.amount,
                    "sell" | "transfer_out" => -import_tx.amount,
                    _ => 0.0,
                };
                *pending_balance_changes.entry(balance_key).or_insert(0.0) += delta;

                // Track fee-coin balance changes for non-swap types
                if let (Some(fee_coin_id), Some(fee_amt)) =
                    (resolved_fee_coin_id.as_deref(), resolved_fee_amount)
                {
                    *pending_balance_changes
                        .entry((wallet.id.clone(), fee_coin_id.to_string()))
                        .or_insert(0.0) -= fee_amt;
                }
            }

            if dry_run {
                if let Some((from_key, to_key)) = dedup_key.clone() {
                    dedup_set.insert(from_key);
                    dedup_set.insert(to_key);
                    let to_coin = match swap_to_coin {
                        Some(c) => c,
                        None => continue,
                    };
                    if use_price_agnostic_dedup {
                        let from_key_price_agnostic = CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            import_tx.amount,
                            None,
                            Some(&to_coin.id),
                        );
                        dedup_set_price_agnostic.insert(from_key_price_agnostic);
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &to_coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            import_tx.swap_to_amount.unwrap_or(0.0),
                            None,
                            Some(&coin.id),
                        ));
                    }
                    summary.record_preview_change(
                        &t("import-preview-change-crypto"),
                        format!(
                            "{:.8} {} → {:.8} {}",
                            import_tx.amount,
                            coin.symbol,
                            import_tx.swap_to_amount.unwrap_or(0.0),
                            to_coin.symbol
                        ),
                        format!("{} - {}", wallet.name, import_tx.date),
                    );
                } else {
                    let key = CryptoDedupKey::new(
                        &import_tx.date,
                        &wallet.id,
                        &coin.id,
                        mechanical_type,
                        &category_type,
                        normalized_subtype.as_deref(),
                        import_tx.amount,
                        import_tx.price_per_coin,
                        None,
                    );
                    dedup_set.insert(key);
                    if use_price_agnostic_dedup {
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            import_tx.amount,
                            None,
                            None,
                        ));
                    }
                    summary.record_preview_change(
                        &t("import-preview-change-crypto"),
                        format!(
                            "{:.8} {} ({})",
                            import_tx.amount, coin.symbol, mechanical_type
                        ),
                        format!("{} - {}", wallet.name, import_tx.date),
                    );
                }
                if let Some(ref_key) = kraken_ref_key.clone() {
                    kraken_trade_ref_set.insert(ref_key);
                }
                continue;
            }

            if mechanical_type == "swap" {
                let to_coin = match swap_to_coin {
                    Some(c) => c,
                    None => continue,
                };
                let to_amount = import_tx.swap_to_amount.unwrap_or(0.0);
                let first_id = Uuid::new_v4().to_string();
                let second_id = Uuid::new_v4().to_string();
                let (source_id, target_id) = if first_id <= second_id {
                    (first_id, second_id)
                } else {
                    (second_id, first_id)
                };

                // Ensure source side always has price_per_coin so the portfolio
                // direction resolver can distinguish source from target (score +1).
                // If the parser didn't provide a price, derive it from the
                // exchange rate: to_amount / from_amount.
                let swap_price = import_tx.price_per_coin.or_else(|| {
                    if import_tx.amount > 0.0 && to_amount > 0.0 {
                        Some(to_amount / import_tx.amount)
                    } else {
                        None
                    }
                });

                let source = CryptoTransaction {
                    id: source_id.clone(),
                    wallet_id: wallet.id.clone(),
                    coin_id: coin.id.clone(),
                    symbol: coin.symbol.clone(),
                    transaction_type: category_type.clone(),
                    amount: import_tx.amount,
                    price_per_coin: swap_price,
                    fee: import_tx.fee,
                    fee_coin_id: resolved_fee_coin_id.clone(),
                    fee_amount: resolved_fee_amount,
                    subtype: normalized_subtype.clone(),
                    override_proceeds: import_tx.override_proceeds,
                    override_cost_basis: None,
                    date: import_tx.date.trim().to_string(),
                    notes: import_tx.notes.clone(),
                    related_tx_id: Some(target_id.clone()),
                };

                let target = CryptoTransaction {
                    id: target_id.clone(),
                    wallet_id: wallet.id.clone(),
                    coin_id: to_coin.id.clone(),
                    symbol: to_coin.symbol.clone(),
                    transaction_type: category_type.clone(),
                    amount: to_amount,
                    price_per_coin: None,
                    fee: None,
                    fee_coin_id: None,
                    fee_amount: None,
                    subtype: normalized_subtype.clone(),
                    override_proceeds: None,
                    override_cost_basis: import_tx.override_cost_basis,
                    date: import_tx.date.trim().to_string(),
                    notes: import_tx.notes.clone(),
                    related_tx_id: Some(source_id.clone()),
                };

                if let Err(e) = IngestionRepository::create_crypto_transaction(db, &source) {
                    summary.record_error(RowError::new(
                        line_num,
                        None,
                        format!("Database error: {}", e),
                    ));
                    continue;
                }

                if let Err(e) = IngestionRepository::create_crypto_transaction(db, &target) {
                    let _ = db.delete_crypto_transaction(&source_id);
                    summary.record_error(RowError::new(
                        line_num,
                        None,
                        format!("Database error: {}", e),
                    ));
                    continue;
                }

                if let Some((from_key, to_key)) = dedup_key {
                    dedup_set.insert(from_key);
                    dedup_set.insert(to_key);
                    if use_price_agnostic_dedup {
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            import_tx.amount,
                            None,
                            Some(&to_coin.id),
                        ));
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &to_coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            to_amount,
                            None,
                            Some(&coin.id),
                        ));
                    }
                }
                if let Some(ref_key) = kraken_ref_key.clone() {
                    kraken_trade_ref_set.insert(ref_key);
                }
                summary.record_inserted();
                continue;
            }

            let mut transaction = CryptoTransaction::new(
                Uuid::new_v4().to_string(),
                wallet.id.clone(),
                coin.id.clone(),
                coin.symbol.clone(),
                category_type.clone(),
                import_tx.amount,
                import_tx.price_per_coin,
                import_tx.fee,
                import_tx.date.trim().to_string(),
                import_tx.notes.clone(),
            );
            transaction.fee_coin_id = resolved_fee_coin_id.clone();
            transaction.fee_amount = resolved_fee_amount;
            transaction.subtype = normalized_subtype.clone();
            transaction.override_proceeds = import_tx.override_proceeds;
            transaction.override_cost_basis = import_tx.override_cost_basis;

            match IngestionRepository::create_crypto_transaction(db, &transaction) {
                Ok(_) => {
                    let key = CryptoDedupKey::new(
                        &import_tx.date,
                        &wallet.id,
                        &coin.id,
                        transaction.mechanical_type(),
                        &transaction.transaction_type,
                        transaction.subtype.as_deref(),
                        import_tx.amount,
                        transaction.price_per_coin,
                        None,
                    );
                    dedup_set.insert(key);
                    if use_price_agnostic_dedup {
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &coin.id,
                            transaction.mechanical_type(),
                            &transaction.transaction_type,
                            transaction.subtype.as_deref(),
                            import_tx.amount,
                            None,
                            None,
                        ));
                    }
                    if let Some(ref_key) = kraken_ref_key {
                        kraken_trade_ref_set.insert(ref_key);
                    }
                    summary.record_inserted();
                }
                Err(e) => {
                    summary.record_error(RowError::new(
                        line_num,
                        None,
                        format!("Database error: {}", e),
                    ));
                }
            }
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        extract_kraken_trade_ref, has_mexc_transfer_overlap_duplicate, kraken_trade_ref_key,
        matches_swap_rollup_duplicate, normalize_import_symbol_key,
        note_is_exchange_overlap_prone, uses_price_agnostic_dedup,
    };
    use crate::models::CryptoTransaction;

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
            "wallet-1",
            "tether",
            "transfer_in",
            100.0,
            None,
            "2026-01-12 10:07:00",
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
            "wallet-1",
            "tether",
            "transfer_in",
            100.0,
            None,
            "2026-01-12 10:30:01",
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
            "wallet-1",
            "litecoin",
            "transfer_out",
            0.5,
            Some(0.0001),
            "2026-01-12 12:12:30",
        ));
    }
}
