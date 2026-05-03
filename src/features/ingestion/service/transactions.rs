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

//! Ingestion service — transaction processing.

use super::format_currency_simple;
use super::repository::IngestionRepository;
use super::types::{ImportSummary, ImportTransaction, RowError, TransactionDedupKey};
use super::validation::{validate_amount, validate_import_transaction};
use super::IngestionError;
use super::IngestionService;
use crate::db::Database;
use crate::models::Transaction;
use crate::services::i18n::{t, t_args};
use std::collections::HashSet;
use uuid::Uuid;

impl IngestionService {
    /// Process and insert transactions (with validation and deduplication)
    pub(super) fn process_transactions(
        &self,
        transactions: Vec<(usize, ImportTransaction)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_transactions_internal(transactions, format_name, false)
    }

    /// Preview transactions (validation and deduplication without inserts)
    pub(super) fn preview_transactions(
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
}
