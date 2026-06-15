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

//! Ingestion service — habit log processing.

use super::IngestionError;
use super::IngestionService;
use crate::db::Database;
use crate::features::ingestion::repository::IngestionRepository;
use crate::features::ingestion::types::{ImportHabitLog, ImportSummary, RowError};
use crate::features::ingestion::validation::validate_import_habit_log;
use crate::models::HabitLog;
use crate::services::i18n::{t, t_args};
use std::collections::HashSet;
use uuid::Uuid;

impl IngestionService {
    /// Process and insert habit logs (with validation and deduplication)
    pub(super) fn process_habit_logs(
        &self,
        logs: Vec<(usize, ImportHabitLog)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_habit_logs_internal(logs, format_name, false)
    }

    /// Preview habit logs (validation and deduplication without inserts)
    pub(super) fn preview_habit_logs(
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
}
