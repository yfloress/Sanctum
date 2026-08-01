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

//! Finance transaction operations
//!
//! Transaction CRUD, transfers, and category management.

use crate::db::{Database, DbError};
use crate::models::{Transaction, TransactionCategory};
use crate::security_log::{SecurityEvent, log_security_event};
use rusqlite::Error as RusqliteError;
use uuid::Uuid;

use super::FinanceError;
use super::commands::{NewTransaction, UpdateTransaction};
use super::repository::FinanceRepository;
use super::validation::{
    MAX_BULK_TRANSACTIONS, MAX_CATEGORY_LENGTH, MAX_DESCRIPTION_LENGTH, sanitize_string,
    validate_category_id, validate_date, validate_field_length, validate_uuid,
};

/// Checks a bulk selection and hands back the validated ids.
///
/// Rejecting the whole batch on one bad id is deliberate: a bulk action is one
/// decision, and silently skipping part of it would leave the user believing
/// something happened that did not.
fn validate_bulk_ids(ids: &[String]) -> Result<Vec<String>, FinanceError> {
    if ids.is_empty() {
        return Err(FinanceError::Validation(
            "No transactions selected".to_string(),
        ));
    }
    if ids.len() > MAX_BULK_TRANSACTIONS {
        return Err(FinanceError::Validation(format!(
            "Cannot act on more than {MAX_BULK_TRANSACTIONS} transactions at once"
        )));
    }
    ids.iter()
        .map(|id| validate_uuid(id).map_err(FinanceError::from))
        .collect()
}

/// Transaction operations for FinanceService
pub struct TransactionOps;

impl TransactionOps {
    pub fn add_transaction<F>(with_db: F, cmd: NewTransaction) -> Result<String, FinanceError>
    where
        F: FnOnce(
            &dyn Fn(&Database) -> Result<String, FinanceError>,
        ) -> Result<String, FinanceError>,
    {
        with_db(&|db| {
            let account_id = validate_uuid(&cmd.account_id)?;
            let category = validate_field_length(&cmd.category, MAX_CATEGORY_LENGTH, "Category")?;
            let category = sanitize_string(&category);

            if category.is_empty() {
                return Err(FinanceError::Validation(
                    "Category cannot be empty".to_string(),
                ));
            }

            let description =
                validate_field_length(&cmd.description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&cmd.date)?;

            if cmd.amount_cents <= 0 {
                return Err(FinanceError::Validation(
                    "Amount must be greater than zero".to_string(),
                ));
            }

            let id = Uuid::new_v4().to_string();
            let transaction_type = if cmd.is_expense { "expense" } else { "income" };

            let transaction = Transaction::new(
                id.clone(),
                account_id,
                cmd.amount_cents,
                category,
                description,
                date,
                transaction_type.to_string(),
                None,
            );

            FinanceRepository::create_transaction(db, &transaction)?;
            log_security_event(SecurityEvent::TransactionCreated, Some(transaction_type));
            Ok(id)
        })
    }

    pub fn update_transaction<F>(with_db: F, cmd: UpdateTransaction) -> Result<(), FinanceError>
    where
        F: FnOnce(&dyn Fn(&Database) -> Result<(), FinanceError>) -> Result<(), FinanceError>,
    {
        with_db(&|db| {
            let id = validate_uuid(&cmd.id)?;
            let account_id = validate_uuid(&cmd.account_id)?;
            let category = validate_field_length(&cmd.category, MAX_CATEGORY_LENGTH, "Category")?;
            let category = sanitize_string(&category);

            if category.is_empty() {
                return Err(FinanceError::Validation(
                    "Category cannot be empty".to_string(),
                ));
            }

            let description =
                validate_field_length(&cmd.description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&cmd.date)?;

            if cmd.amount_cents <= 0 {
                return Err(FinanceError::Validation(
                    "Amount must be greater than zero".to_string(),
                ));
            }

            let transaction_type = if cmd.is_expense { "expense" } else { "income" };

            let transaction = Transaction::new(
                id,
                account_id,
                cmd.amount_cents,
                category,
                description,
                date,
                transaction_type.to_string(),
                None,
            );

            FinanceRepository::update_transaction(db, &transaction)?;
            Ok(())
        })
    }

    pub fn delete_transaction<F>(with_db: F, id: String) -> Result<(), FinanceError>
    where
        F: FnOnce(&dyn Fn(&Database) -> Result<(), FinanceError>) -> Result<(), FinanceError>,
    {
        with_db(&|db| {
            let validated_id = validate_uuid(&id)?;
            FinanceRepository::delete_transaction(db, &validated_id)?;
            log_security_event(SecurityEvent::TransactionDeleted, None);
            Ok(())
        })
    }

    /// Deletes every transaction in `ids`. Returns how many were removed.
    pub fn delete_transactions(db: &Database, ids: &[String]) -> Result<usize, FinanceError> {
        let ids = validate_bulk_ids(ids)?;
        let deleted = FinanceRepository::delete_transactions(db, &ids)?;
        log_security_event(
            SecurityEvent::TransactionDeleted,
            Some(&format!("bulk:{deleted}")),
        );
        Ok(deleted)
    }

    /// Moves every transaction in `ids` to `category`. Returns how many changed.
    ///
    /// The count can be lower than `ids.len()`: transfers carry no user category
    /// and are left as they are.
    pub fn recategorize_transactions(
        db: &Database,
        ids: &[String],
        category: &str,
    ) -> Result<usize, FinanceError> {
        let ids = validate_bulk_ids(ids)?;
        let category = validate_field_length(category, MAX_CATEGORY_LENGTH, "Category")?;
        let category = sanitize_string(&category);

        if category.is_empty() {
            return Err(FinanceError::Validation(
                "Category cannot be empty".to_string(),
            ));
        }

        Ok(FinanceRepository::recategorize_transactions(
            db, &ids, &category,
        )?)
    }

    pub fn transfer_funds<F>(
        with_db: F,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<String, FinanceError>
    where
        F: FnOnce(
            &dyn Fn(&Database) -> Result<String, FinanceError>,
        ) -> Result<String, FinanceError>,
    {
        with_db(&|db| {
            let from_id = validate_uuid(&from_account_id)?;
            let to_id = validate_uuid(&to_account_id)?;

            if amount <= 0 {
                return Err(FinanceError::Validation(
                    "Transfer amount must be greater than zero".to_string(),
                ));
            }

            let from_account = FinanceRepository::get_account(db, &from_id)?;
            let to_account = FinanceRepository::get_account(db, &to_id)?;
            if from_account.currency.to_uppercase() != to_account.currency.to_uppercase() {
                return Err(FinanceError::Validation(
                    "Transfers require both accounts to use the same currency".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            let tx_id = FinanceRepository::create_transfer(
                db,
                &from_id,
                &to_id,
                amount,
                &description,
                &date,
            )?;
            log_security_event(SecurityEvent::TransactionCreated, Some("transfer"));
            Ok(tx_id)
        })
    }

    pub fn update_transfer<F>(
        with_db: F,
        id: String,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<(), FinanceError>
    where
        F: FnOnce(&dyn Fn(&Database) -> Result<(), FinanceError>) -> Result<(), FinanceError>,
    {
        with_db(&|db| {
            let id = validate_uuid(&id)?;
            let from_id = validate_uuid(&from_account_id)?;
            let to_id = validate_uuid(&to_account_id)?;

            if amount <= 0 {
                return Err(FinanceError::Validation(
                    "Transfer amount must be greater than zero".to_string(),
                ));
            }

            let from_account = FinanceRepository::get_account(db, &from_id)?;
            let to_account = FinanceRepository::get_account(db, &to_id)?;
            if from_account.currency.to_uppercase() != to_account.currency.to_uppercase() {
                return Err(FinanceError::Validation(
                    "Transfers require both accounts to use the same currency".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            FinanceRepository::update_transfer(
                db,
                &id,
                &from_id,
                &to_id,
                amount,
                &description,
                &date,
            )?;
            Ok(())
        })
    }
}

/// Category operations for FinanceService
pub struct CategoryOps;

impl CategoryOps {
    pub fn get_transaction_categories(
        db: &Database,
        category_type: &str,
    ) -> Result<Vec<TransactionCategory>, FinanceError> {
        if category_type != "expense" && category_type != "income" {
            return Err(FinanceError::Validation(
                "Category type must be 'expense' or 'income'".to_string(),
            ));
        }

        FinanceRepository::get_transaction_categories(db, category_type)
            .map_err(FinanceError::Database)
    }

    pub fn add_transaction_category(
        db: &Database,
        name: &str,
        category_type: &str,
    ) -> Result<String, FinanceError> {
        let validated_name = validate_field_length(name, MAX_CATEGORY_LENGTH, "Category name")?;

        if validated_name.is_empty() {
            return Err(FinanceError::Validation(
                "Category name cannot be empty".to_string(),
            ));
        }

        if category_type != "expense" && category_type != "income" {
            return Err(FinanceError::Validation(
                "Category type must be 'expense' or 'income'".to_string(),
            ));
        }

        FinanceRepository::add_transaction_category(db, &validated_name, category_type).map_err(
            |e| match e {
                DbError::Sqlite(RusqliteError::ExecuteReturnedResults) => {
                    FinanceError::Validation("Category with this name already exists".to_string())
                }
                _ => FinanceError::Database(e),
            },
        )
    }

    pub fn update_transaction_category(
        db: &Database,
        id: &str,
        new_name: &str,
    ) -> Result<(), FinanceError> {
        let validated_id = validate_category_id(id)?;
        let validated_name = validate_field_length(new_name, MAX_CATEGORY_LENGTH, "Category name")?;

        if validated_name.is_empty() {
            return Err(FinanceError::Validation(
                "Category name cannot be empty".to_string(),
            ));
        }

        FinanceRepository::update_transaction_category(db, &validated_id, &validated_name).map_err(
            |e| match e {
                DbError::Sqlite(RusqliteError::ExecuteReturnedResults) => {
                    FinanceError::Validation("Category with this name already exists".to_string())
                }
                _ => FinanceError::Database(e),
            },
        )
    }

    pub fn delete_transaction_category(db: &Database, id: &str) -> Result<(), FinanceError> {
        let validated_id = validate_category_id(id)?;
        FinanceRepository::delete_transaction_category(db, &validated_id).map_err(|e| match e {
            // A guard the user can act on, not an internal failure.
            DbError::CategoryInUse => FinanceError::Validation(
                "This category still has transactions. Move or delete them first.".to_string(),
            ),
            other => FinanceError::Database(other),
        })
    }
}
