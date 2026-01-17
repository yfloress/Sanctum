//! Ingestion service
//!
//! Orchestrates data import from various file formats.

use crate::db::{Database, DbError};
use crate::models::{HabitLog, Transaction};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::parsers::{detect_format, CsvParser, ImportParser, JsonV1Parser, TextParser};
use super::repository::IngestionRepository;
use super::types::{
    ImportFormat, ImportHabitLog, ImportSummary, ImportTransaction, RowError, TransactionDedupKey,
};
use super::validation::{validate_amount, validate_file_size, validate_import_habit_log, validate_import_transaction};

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
        let db_lock = self
            .db
            .lock()
            .map_err(|_| IngestionError::Parse("Lock error".to_string()))?;
        let db = db_lock.as_ref().ok_or(IngestionError::NoVaultOpen)?;

        db.check_session_timeout().map_err(|e| match e {
            DbError::SessionExpired => IngestionError::SessionExpired,
            _ => IngestionError::Database(e),
        })?;

        let result = f(db)?;
        db.touch_session().map_err(IngestionError::Database)?;
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
            ImportFormat::TextTransactions => self.import_text_transactions(content),
            ImportFormat::TextHabitLogs => self.import_text_habit_logs(content),
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
            ImportFormat::TextTransactions => self.preview_text_transactions(content),
            ImportFormat::TextHabitLogs => self.preview_text_habit_logs(content),
        }
    }

    /// Import JSON v1 format (can contain both transactions and habit logs)
    fn import_json_v1(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = JsonV1Parser;
        let mut summary = ImportSummary::new("JSON v1", "Mixed");

        // Parse full JSON to get both transactions and habit logs
        let file = parser
            .parse_full(content)
            .map_err(|e| IngestionError::Parse(e.message))?;

        // Import transactions
        if !file.transactions.items.is_empty() || !file.transactions.errors.is_empty() {
            let tx_summary = self.process_transactions(file.transactions.items, parser.format_name())?;
            summary.merge(tx_summary);
            for error in file.transactions.errors {
                summary.record_error(error);
            }
        }

        // Import habit logs
        if !file.habit_logs.items.is_empty() || !file.habit_logs.errors.is_empty() {
            let log_summary = self.process_habit_logs(file.habit_logs.items, parser.format_name())?;
            summary.merge(log_summary);
            for error in file.habit_logs.errors {
                summary.record_error(error);
            }
        }

        Ok(summary)
    }

    /// Preview JSON v1 format (can contain both transactions and habit logs)
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

    /// Import text transactions
    fn import_text_transactions(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = TextParser;
        let parsed = parser
            .parse_transactions(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.process_transactions(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Preview text transactions
    fn preview_text_transactions(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = TextParser;
        let parsed = parser
            .parse_transactions(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.preview_transactions(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Import text habit logs
    fn import_text_habit_logs(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = TextParser;
        let parsed = parser
            .parse_habit_logs(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.process_habit_logs(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
        }
        Ok(summary)
    }

    /// Preview text habit logs
    fn preview_text_habit_logs(&self, content: &str) -> Result<ImportSummary, IngestionError> {
        let parser = TextParser;
        let parsed = parser
            .parse_habit_logs(content)
            .map_err(|e| IngestionError::Parse(e.message))?;
        let mut summary = self.preview_habit_logs(parsed.items, parser.format_name())?;
        for error in parsed.errors {
            summary.record_error(error);
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
        self.with_db(|db| {
            let mut summary = ImportSummary::new(format_name, "Transactions");

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
                            format!("Account not found: '{}'", import_tx.account),
                        ));
                        continue;
                    }
                };

                let import_currency = import_tx.currency.trim().to_uppercase();
                if account.currency.to_uppercase() != import_currency {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("currency"),
                        format!(
                            "Currency mismatch: import has '{}' but account '{}' uses '{}'",
                            import_currency, account.name, account.currency
                        ),
                    ));
                    continue;
                }

                let tx_type = import_tx.transaction_type.trim().to_lowercase();
                let category_type = if tx_type == "income" { "income" } else { "expense" };

                if tx_type != "transfer" {
                    let category_key = (
                        import_tx.category.trim().to_lowercase(),
                        category_type.to_string(),
                    );
                    if !category_lookup.contains_key(&category_key) {
                        summary.record_error(RowError::new(
                            line_num,
                            Some("category"),
                            format!(
                                "Category not found: '{}' (type: {})",
                                import_tx.category, category_type
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
                                format!("Destination account not found: '{}'", dest_name),
                            ));
                            continue;
                        }
                    };

                    if dest_account.id == account.id {
                        summary.record_error(RowError::new(
                            line_num,
                            Some("transfer_to_account"),
                            "Cannot transfer to the same account".to_string(),
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
                        summary.record_skipped(
                            "Duplicate transaction (same date/account/amount/type/description)",
                        );
                        continue;
                    }

                    if dry_run {
                        dedup_set.insert(dedup_key);
                        summary.record_inserted();
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
                        summary.record_skipped(
                            "Duplicate transaction (same date/account/amount/type/description)",
                        );
                        continue;
                    }

                    if dry_run {
                        dedup_set.insert(dedup_key);
                        summary.record_inserted();
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
        })
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
        self.with_db(|db| {
            let mut summary = ImportSummary::new(format_name, "Habit Logs");

            let habit_lookup =
                IngestionRepository::build_habit_lookup(db).map_err(IngestionError::Database)?;
            let mut seen_logs: HashSet<(String, String)> = HashSet::new();

            for (line_num, import_log) in logs {
                if !import_log.completed {
                    summary.record_skipped("Habit log not completed (completed=false)");
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
                            format!("Habit not found: '{}'", import_log.habit),
                        ));
                        continue;
                    }
                };

                let date = import_log.date.trim();
                let dedup_key = (habit.id.clone(), date.to_string());
                if seen_logs.contains(&dedup_key) {
                    summary.record_skipped("Habit already logged for this date");
                    continue;
                }

                match IngestionRepository::habit_log_exists(db, &habit.id, date) {
                    Ok(true) => {
                        seen_logs.insert(dedup_key);
                        summary.record_skipped("Habit already logged for this date");
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
                    summary.record_inserted();
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
        })
    }
}
