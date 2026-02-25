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

//! Finance transaction operations
//!
//! Transaction CRUD, transfers, and category management.

use crate::db::{Database, DbError};
use crate::models::{Transaction, TransactionCategory};
use crate::security_log::{SecurityEvent, log_security_event};
use rusqlite::Error as RusqliteError;
use uuid::Uuid;

use super::FinanceError;
use super::repository::FinanceRepository;
use super::validation::{
    MAX_CATEGORY_LENGTH, MAX_DESCRIPTION_LENGTH, sanitize_string, validate_category_id,
    validate_date, validate_field_length, validate_uuid,
};

/// Transaction operations for FinanceService
pub struct TransactionOps;

impl TransactionOps {
    pub fn add_transaction<F>(
        with_db: F,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        is_expense: bool,
    ) -> Result<String, FinanceError>
    where
        F: FnOnce(
            &dyn Fn(&Database) -> Result<String, FinanceError>,
        ) -> Result<String, FinanceError>,
    {
        with_db(&|db| {
            let account_id = validate_uuid(&account_id)?;
            let category = validate_field_length(&category, MAX_CATEGORY_LENGTH, "Category")?;
            let category = sanitize_string(&category);

            if category.is_empty() {
                return Err(FinanceError::Validation(
                    "Category cannot be empty".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            if amount <= 0 {
                return Err(FinanceError::Validation(
                    "Amount must be greater than zero".to_string(),
                ));
            }

            let id = Uuid::new_v4().to_string();
            let transaction_type = if is_expense { "expense" } else { "income" };

            let transaction = Transaction::new(
                id.clone(),
                account_id,
                amount,
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

    #[allow(clippy::too_many_arguments)]
    pub fn update_transaction<F>(
        with_db: F,
        id: String,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        is_expense: bool,
    ) -> Result<(), FinanceError>
    where
        F: FnOnce(&dyn Fn(&Database) -> Result<(), FinanceError>) -> Result<(), FinanceError>,
    {
        with_db(&|db| {
            let id = validate_uuid(&id)?;
            let account_id = validate_uuid(&account_id)?;
            let category = validate_field_length(&category, MAX_CATEGORY_LENGTH, "Category")?;
            let category = sanitize_string(&category);

            if category.is_empty() {
                return Err(FinanceError::Validation(
                    "Category cannot be empty".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            if amount <= 0 {
                return Err(FinanceError::Validation(
                    "Amount must be greater than zero".to_string(),
                ));
            }

            let transaction_type = if is_expense { "expense" } else { "income" };

            let transaction = Transaction::new(
                id,
                account_id,
                amount,
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
        FinanceRepository::delete_transaction_category(db, &validated_id)
            .map_err(FinanceError::Database)
    }
}
