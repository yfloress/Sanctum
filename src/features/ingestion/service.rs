//! Ingestion service
//!
//! Orchestrates data import from various file formats.

use crate::db::{Database, DbError};
use crate::models::{CryptoTransaction, HabitLog, Transaction};
use crate::services::i18n::{t, t_args};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::parsers::{CsvParser, ImportParser, JsonV1Parser, TextParser, detect_format};
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
            ImportFormat::JsonV1 => self.import_json_v1(content),
            ImportFormat::CsvTransactions => self.import_csv_transactions(content),
            ImportFormat::CsvHabitLogs => self.import_csv_habit_logs(content),
            ImportFormat::CsvCrypto => self.import_csv_crypto(content),
            ImportFormat::TextMixed => self.import_text_mixed(content),
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
            ImportFormat::JsonV1 => self.preview_json_v1(content),
            ImportFormat::CsvTransactions => self.preview_csv_transactions(content),
            ImportFormat::CsvHabitLogs => self.preview_csv_habit_logs(content),
            ImportFormat::CsvCrypto => self.preview_csv_crypto(content),
            ImportFormat::TextMixed => self.preview_text_mixed(content),
        }
    }

    /// Import JSON v1 format (can contain transactions, habit logs, and crypto)
    fn import_json_v1(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = JsonV1Parser;
        let mut summary = ImportSummary::new("JSON v1", "Mixed");

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

    /// Preview JSON v1 format (can contain transactions, habit logs, and crypto)
    fn preview_json_v1(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = JsonV1Parser;
        let mut summary = ImportSummary::new("JSON v1", "Mixed");

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
                let dest_name = import_tx.transfer_to_account.as_ref().unwrap();
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
        self.process_crypto_transactions_internal(transactions, format_name, false)
    }

    /// Preview crypto transactions (validation and deduplication without inserts)
    fn preview_crypto_transactions(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(transactions, format_name, true)
    }

    fn process_crypto_transactions_internal(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        dry_run: bool,
    ) -> Result<ImportSummary, IngestionError> {
        if dry_run {
            self.with_db_readonly(|db| {
                self.process_crypto_transactions_with_db(db, transactions, format_name, dry_run)
            })
        } else {
            self.with_db(|db| {
                self.process_crypto_transactions_with_db(db, transactions, format_name, dry_run)
            })
        }
    }

    fn process_crypto_transactions_with_db(
        &self,
        db: &Database,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        dry_run: bool,
    ) -> Result<ImportSummary, IngestionError> {
        let mut summary = ImportSummary::new(format_name, "Crypto");
        let skipped_duplicate = t("import-skipped-duplicate-crypto");

        let wallet_lookup =
            IngestionRepository::build_wallet_lookup(db).map_err(IngestionError::Database)?;
        let coin_lookup =
            IngestionRepository::build_coin_lookup(db).map_err(IngestionError::Database)?;

        let existing = IngestionRepository::get_all_crypto_transactions(db)
            .map_err(IngestionError::Database)?;

        let mut dedup_set: HashSet<CryptoDedupKey> = existing
            .iter()
            .map(|tx| {
                CryptoDedupKey::new(
                    &tx.date,
                    &tx.wallet_id,
                    &tx.coin_id,
                    &tx.transaction_type,
                    tx.amount,
                )
            })
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

            // Resolve coin
            let symbol_key = import_tx.symbol.trim().to_lowercase();
            let coin = match coin_lookup.get(&symbol_key) {
                Some(c) => c,
                None => {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("symbol"),
                        t_args(
                            "import-error-crypto-not-found",
                            &[("symbol", import_tx.symbol.trim())],
                        ),
                    ));
                    continue;
                }
            };

            let tx_type = import_tx.transaction_type.trim().to_lowercase();

            // Validate balance for sell and transfer_out operations
            if tx_type == "sell" || tx_type == "transfer_out" {
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
            }

            let dedup_key = CryptoDedupKey::new(
                &import_tx.date,
                &wallet.id,
                &coin.id,
                &tx_type,
                import_tx.amount,
            );

            if dedup_set.contains(&dedup_key) {
                summary.record_skipped(&skipped_duplicate);
                continue;
            }

            // Update pending balance changes for subsequent validations
            let balance_key = (wallet.id.clone(), coin.id.clone());
            let delta = match tx_type.as_str() {
                "buy" | "transfer_in" => import_tx.amount,
                "sell" | "transfer_out" => -import_tx.amount,
                _ => 0.0,
            };
            *pending_balance_changes.entry(balance_key).or_insert(0.0) += delta;

            if dry_run {
                dedup_set.insert(dedup_key);
                summary.record_preview_change(
                    &t("import-preview-change-crypto"),
                    format!("{:.8} {} ({})", import_tx.amount, coin.symbol, tx_type),
                    format!("{} - {}", wallet.name, import_tx.date),
                );
                continue;
            }

            let transaction = CryptoTransaction::new(
                Uuid::new_v4().to_string(),
                wallet.id.clone(),
                coin.id.clone(),
                coin.symbol.clone(),
                tx_type,
                import_tx.amount,
                import_tx.price_per_coin,
                import_tx.fee,
                import_tx.date.trim().to_string(),
                import_tx.notes.clone(),
            );

            match IngestionRepository::create_crypto_transaction(db, &transaction) {
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

        Ok(summary)
    }
}
