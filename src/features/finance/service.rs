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

//! Finance service
//!
//! Business logic for financial operations: accounts, transactions, analytics.
//! Split into focused submodules for maintainability.

use crate::db::{Database, DbError};
use crate::models::{
    Account, AccountBalance, BalanceSummary, BudgetStatus, CategoryBudget, Credit,
    CreditInstallment, RecurrenceFrequency, RecurringTransaction, Transaction, TransactionCategory,
};
use crate::security_log::{SecurityEvent, log_security_event};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use super::commands::{
    NewAccount, NewCharge, NewCredit, NewRecurring, NewTransaction, NewTransfer, UpdateAccount,
    UpdateInstallment, UpdateTransaction, UpdateTransfer,
};
use super::credits::CreditOps;
use super::export;
use super::repository::FinanceRepository;
use super::transactions::{CategoryOps, TransactionOps};
use super::validation::{
    EXCHANGE_RATE_TTL_SECS, MAX_ACCOUNT_NAME_LENGTH, MAX_CURRENCY_LENGTH, MAX_ICON_LENGTH,
    sanitize_string, validate_color, validate_field_length, validate_uuid,
};
use crate::features::dashboard::DashboardCharts;

#[derive(thiserror::Error, Debug)]
pub enum FinanceError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error")]
    Internal,

    #[error("No vault is currently open")]
    NoVaultOpen,

    #[error("Session expired due to inactivity. Please unlock the vault again.")]
    SessionExpired,
}

impl From<String> for FinanceError {
    fn from(s: String) -> Self {
        FinanceError::Validation(s)
    }
}

// Re-export types from submodules
pub use crate::features::dashboard::{DashboardData, ExpenseSlice};

pub struct FinanceService {
    db: Arc<RwLock<Option<Database>>>,
}

impl FinanceService {
    pub fn new(db: Arc<RwLock<Option<Database>>>) -> Self {
        Self { db }
    }

    fn with_db<T, F>(&self, f: F) -> Result<T, FinanceError>
    where
        F: FnOnce(&Database) -> Result<T, FinanceError>,
    {
        let db_lock = self.db.read().map_err(|_| FinanceError::Internal)?;
        let db = db_lock.as_ref().ok_or(FinanceError::NoVaultOpen)?;

        db.check_session_timeout().map_err(|e| match e {
            DbError::SessionExpired => FinanceError::SessionExpired,
            _ => FinanceError::Database(e),
        })?;

        let result = f(db)?;
        db.touch_session().map_err(FinanceError::Database)?;
        Ok(result)
    }

    // ==================== Account Operations ====================

    pub fn create_account(&self, cmd: NewAccount) -> Result<String, FinanceError> {
        self.with_db(|db| {
            let NewAccount {
                name,
                account_type,
                currency,
                initial_balance_cents,
                color,
                icon,
            } = cmd;

            let name = validate_field_length(&name, MAX_ACCOUNT_NAME_LENGTH, "Account name")?;
            let name = sanitize_string(&name);

            if name.is_empty() {
                return Err(FinanceError::Validation(
                    "Account name cannot be empty".to_string(),
                ));
            }

            let currency = validate_field_length(&currency, MAX_CURRENCY_LENGTH, "Currency")?;
            let currency = sanitize_string(&currency).to_uppercase();

            if currency.is_empty() {
                return Err(FinanceError::Validation(
                    "Currency cannot be empty".to_string(),
                ));
            }

            let color = validate_color(&color)?;

            let icon = if let Some(i) = icon {
                let i = validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?;
                if i.is_empty() { None } else { Some(i) }
            } else {
                None
            };

            let id = Uuid::new_v4().to_string();
            let created_at = Utc::now().to_rfc3339();

            let account = Account::new(
                id.clone(),
                name,
                account_type,
                currency,
                initial_balance_cents,
                color,
                icon,
                created_at,
            );

            FinanceRepository::create_account(db, &account)?;
            log_security_event(SecurityEvent::TransactionCreated, Some("account_created"));
            Ok(id)
        })
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>, FinanceError> {
        self.with_db(|db| FinanceRepository::get_accounts(db).map_err(FinanceError::Database))
    }

    pub fn get_account_balances(&self) -> Result<Vec<AccountBalance>, FinanceError> {
        self.with_db(|db| {
            FinanceRepository::get_all_account_balances(db).map_err(FinanceError::Database)
        })
    }

    pub fn update_account(&self, cmd: UpdateAccount) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let UpdateAccount {
                id,
                name,
                account_type,
                currency,
                initial_balance_cents,
                color,
                icon,
            } = cmd;

            let validated_id = validate_uuid(&id)?;
            let name = validate_field_length(&name, MAX_ACCOUNT_NAME_LENGTH, "Account name")?;
            let name = sanitize_string(&name);

            if name.is_empty() {
                return Err(FinanceError::Validation(
                    "Account name cannot be empty".to_string(),
                ));
            }

            let currency = validate_field_length(&currency, MAX_CURRENCY_LENGTH, "Currency")?;
            let currency = sanitize_string(&currency).to_uppercase();
            let color = validate_color(&color)?;

            let icon = if let Some(i) = icon {
                let i = validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?;
                if i.is_empty() { None } else { Some(i) }
            } else {
                None
            };

            let existing = FinanceRepository::get_account(db, &validated_id)?;
            let account = Account {
                id: validated_id,
                name,
                account_type,
                currency,
                initial_balance: initial_balance_cents,
                color,
                icon,
                is_archived: existing.is_archived,
                created_at: existing.created_at,
            };

            FinanceRepository::update_account(db, &account).map_err(FinanceError::Database)
        })
    }

    pub fn update_account_icon(
        &self,
        id: String,
        icon: Option<String>,
    ) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            let mut account = FinanceRepository::get_account(db, &validated_id)?;

            let icon = if let Some(i) = icon {
                let i = validate_field_length(&i, MAX_ICON_LENGTH, "Icon")?;
                if i.is_empty() { None } else { Some(i) }
            } else {
                None
            };

            account.icon = icon;
            FinanceRepository::update_account(db, &account).map_err(FinanceError::Database)
        })
    }

    pub fn update_account_name(&self, id: String, new_name: String) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            let validated_name =
                validate_field_length(&new_name, MAX_ACCOUNT_NAME_LENGTH, "Account name")?;
            let sanitized_name = sanitize_string(&validated_name);

            if sanitized_name.is_empty() {
                return Err(FinanceError::Validation(
                    "Account name cannot be empty".to_string(),
                ));
            }

            let mut account = FinanceRepository::get_account(db, &validated_id)?;
            account.name = sanitized_name;
            FinanceRepository::update_account(db, &account).map_err(FinanceError::Database)
        })
    }

    pub fn archive_account(&self, id: String) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            FinanceRepository::archive_account(db, &validated_id).map_err(FinanceError::Database)
        })
    }

    pub fn get_archived_accounts(&self) -> Result<Vec<Account>, FinanceError> {
        self.with_db(|db| {
            FinanceRepository::get_archived_accounts(db).map_err(FinanceError::Database)
        })
    }

    pub fn unarchive_account(&self, id: String) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            FinanceRepository::unarchive_account(db, &validated_id).map_err(FinanceError::Database)
        })
    }

    // ==================== Transaction Operations ====================

    pub fn add_transaction(&self, cmd: NewTransaction) -> Result<String, FinanceError> {
        let db_arc = self.db.clone();
        TransactionOps::add_transaction(
            |f| {
                let db_lock = db_arc.read().map_err(|_| FinanceError::Internal)?;
                let db = db_lock.as_ref().ok_or(FinanceError::NoVaultOpen)?;
                db.check_session_timeout().map_err(|e| match e {
                    DbError::SessionExpired => FinanceError::SessionExpired,
                    _ => FinanceError::Database(e),
                })?;
                let result = f(db)?;
                db.touch_session().map_err(FinanceError::Database)?;
                Ok(result)
            },
            cmd,
        )
    }

    pub fn update_transaction(&self, cmd: UpdateTransaction) -> Result<(), FinanceError> {
        let db_arc = self.db.clone();
        TransactionOps::update_transaction(
            |f| {
                let db_lock = db_arc.read().map_err(|_| FinanceError::Internal)?;
                let db = db_lock.as_ref().ok_or(FinanceError::NoVaultOpen)?;
                db.check_session_timeout().map_err(|e| match e {
                    DbError::SessionExpired => FinanceError::SessionExpired,
                    _ => FinanceError::Database(e),
                })?;
                f(db)?;
                db.touch_session().map_err(FinanceError::Database)?;
                Ok(())
            },
            cmd,
        )
    }

    /// Deletes a whole selection at once. Returns how many rows went.
    pub fn delete_transactions(&self, ids: Vec<String>) -> Result<usize, FinanceError> {
        self.with_db(|db| TransactionOps::delete_transactions(db, &ids))
    }

    /// Moves a whole selection to one category. Returns how many rows changed,
    /// which is fewer than requested when the selection included transfers.
    pub fn recategorize_transactions(
        &self,
        ids: Vec<String>,
        category: String,
    ) -> Result<usize, FinanceError> {
        self.with_db(|db| TransactionOps::recategorize_transactions(db, &ids, &category))
    }

    // ==================== Tags ====================

    /// Replaces the tags on one transaction.
    pub fn set_transaction_tags(&self, id: String, tags: Vec<String>) -> Result<(), FinanceError> {
        self.with_db(|db| TransactionOps::set_transaction_tags(db, &id, &tags))
    }

    /// Tags of every transaction, keyed by transaction id.
    pub fn get_all_transaction_tags(
        &self,
    ) -> Result<std::collections::HashMap<String, Vec<String>>, FinanceError> {
        self.with_db(TransactionOps::get_all_transaction_tags)
    }

    /// Tags in use, most used first.
    pub fn get_tag_catalog(&self) -> Result<Vec<String>, FinanceError> {
        self.with_db(TransactionOps::get_tag_catalog)
    }

    /// Puts one tag on a whole selection. Returns how many rows gained it.
    pub fn tag_transactions(&self, ids: Vec<String>, tag: String) -> Result<usize, FinanceError> {
        self.with_db(|db| TransactionOps::tag_transactions(db, &ids, &tag))
    }

    // ==================== Reconciliation ====================

    /// The account's balance counting only rows confirmed against a statement.
    pub fn reconciled_balance(&self, account_id: String) -> Result<i64, FinanceError> {
        self.with_db(|db| TransactionOps::reconciled_balance(db, &account_id))
    }

    /// Rows of the account still waiting to be checked off, oldest first.
    pub fn unreconciled_transactions(
        &self,
        account_id: String,
    ) -> Result<Vec<Transaction>, FinanceError> {
        self.with_db(|db| TransactionOps::unreconciled_transactions(db, &account_id))
    }

    /// Confirms a whole selection against one account's statement.
    pub fn confirm_reconciliation(
        &self,
        account_id: String,
        ids: Vec<String>,
    ) -> Result<usize, FinanceError> {
        self.with_db(|db| TransactionOps::confirm_reconciliation(db, &account_id, &ids))
    }

    // ==================== Credits ====================

    /// Creates a credit and its whole installment schedule.
    pub fn create_credit(&self, cmd: NewCredit) -> Result<String, FinanceError> {
        self.with_db(|db| CreditOps::create_credit(db, cmd.clone()))
    }

    pub fn get_credits(&self) -> Result<Vec<Credit>, FinanceError> {
        self.with_db(CreditOps::get_credits)
    }

    /// Installments of every credit, keyed by credit id and in order.
    pub fn get_credit_installments(
        &self,
    ) -> Result<HashMap<String, Vec<CreditInstallment>>, FinanceError> {
        self.with_db(CreditOps::get_credit_installments)
    }

    /// Deletes a credit. Payments already made stay in the ledger.
    pub fn delete_credit(&self, id: String) -> Result<(), FinanceError> {
        self.with_db(|db| CreditOps::delete_credit(db, &id))
    }

    /// Pays one installment on `date`, writing the expense it stands for.
    pub fn pay_installment(
        &self,
        installment_id: String,
        date: Option<String>,
    ) -> Result<String, FinanceError> {
        let date = match date {
            Some(value) if !value.trim().is_empty() => value,
            _ => Utc::now().format("%Y-%m-%d").to_string(),
        };
        self.with_db(|db| CreditOps::pay_installment(db, &installment_id, &date))
    }

    /// Undoes a payment, deleting the expense it wrote.
    pub fn unpay_installment(&self, installment_id: String) -> Result<(), FinanceError> {
        self.with_db(|db| CreditOps::unpay_installment(db, &installment_id))
    }

    /// Corrects one unpaid row of a schedule.
    pub fn update_installment(&self, cmd: UpdateInstallment) -> Result<(), FinanceError> {
        self.with_db(|db| CreditOps::update_installment(db, cmd.clone()))
    }

    /// Records a fee the lender charged on top of a credit's plan.
    pub fn add_charge(&self, cmd: NewCharge) -> Result<String, FinanceError> {
        self.with_db(|db| CreditOps::add_charge(db, cmd.clone()))
    }

    /// Removes an unpaid charge.
    pub fn delete_charge(&self, installment_id: String) -> Result<(), FinanceError> {
        self.with_db(|db| CreditOps::delete_charge(db, &installment_id))
    }

    // ==================== Category Budgets ====================

    /// Sets (or replaces) the monthly limit for a category.
    pub fn set_category_budget(
        &self,
        category: String,
        amount_cents: i64,
    ) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let category = sanitize_string(&validate_field_length(
                &category,
                MAX_ACCOUNT_NAME_LENGTH,
                "Category",
            )?);
            if category.is_empty() {
                return Err(FinanceError::Validation(
                    "Category cannot be empty".to_string(),
                ));
            }
            if amount_cents <= 0 {
                return Err(FinanceError::Validation(
                    "Budget must be greater than zero".to_string(),
                ));
            }

            let budget = CategoryBudget {
                id: Uuid::new_v4().to_string(),
                category,
                amount: amount_cents,
                created_at: Utc::now().to_rfc3339(),
            };
            db.upsert_category_budget(&budget)
                .map_err(FinanceError::Database)
        })
    }

    pub fn delete_category_budget(&self, category: String) -> Result<(), FinanceError> {
        self.with_db(|db| {
            db.delete_category_budget(category.trim())
                .map_err(FinanceError::Database)
        })
    }

    /// Budgets with the spending measured against them. Defaults to this month.
    pub fn get_budget_status(
        &self,
        month: Option<String>,
    ) -> Result<Vec<BudgetStatus>, FinanceError> {
        let month = match month {
            Some(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => Utc::now().format("%Y-%m").to_string(),
        };
        self.with_db(|db| db.get_budget_status(&month).map_err(FinanceError::Database))
    }

    // ==================== Recurring Transactions ====================

    /// Creates a recurring rule. `first_date` is when it fires for the first time.
    pub fn create_recurring(&self, cmd: NewRecurring) -> Result<String, FinanceError> {
        let NewRecurring {
            account_id,
            amount_cents,
            category,
            description,
            frequency,
            first_date,
            is_expense,
        } = cmd;

        self.with_db(|db| {
            let account_id = validate_uuid(&account_id)?;
            let category = sanitize_string(&validate_field_length(
                &category,
                MAX_ACCOUNT_NAME_LENGTH,
                "Category",
            )?);
            if category.is_empty() {
                return Err(FinanceError::Validation(
                    "Category cannot be empty".to_string(),
                ));
            }
            if amount_cents <= 0 {
                return Err(FinanceError::Validation(
                    "Amount must be greater than zero".to_string(),
                ));
            }
            let frequency = RecurrenceFrequency::parse(&frequency).ok_or_else(|| {
                FinanceError::Validation("Frequency must be weekly, monthly or yearly".to_string())
            })?;
            let first_date = crate::core::validate_date(&first_date)?;
            let description = sanitize_string(&description);

            // Fails early rather than at the first occurrence.
            FinanceRepository::get_account(db, &account_id)?;

            let rule = RecurringTransaction {
                id: Uuid::new_v4().to_string(),
                account_id,
                amount: amount_cents,
                category,
                description,
                transaction_type: if is_expense { "expense" } else { "income" }.to_string(),
                frequency: frequency.as_str().to_string(),
                next_date: first_date,
                last_created_date: None,
                is_active: true,
                created_at: Utc::now().to_rfc3339(),
            };

            db.create_recurring_transaction(&rule)
                .map_err(FinanceError::Database)?;
            Ok(rule.id)
        })
    }

    pub fn get_recurring(&self) -> Result<Vec<RecurringTransaction>, FinanceError> {
        self.with_db(|db| {
            db.get_recurring_transactions()
                .map_err(FinanceError::Database)
        })
    }

    /// Pauses or resumes a rule, keeping it and its history.
    pub fn set_recurring_active(&self, id: String, active: bool) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let id = validate_uuid(&id)?;
            db.set_recurring_active(&id, active)
                .map_err(FinanceError::Database)
        })
    }

    pub fn delete_recurring(&self, id: String) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let id = validate_uuid(&id)?;
            db.delete_recurring_transaction(&id)
                .map_err(FinanceError::Database)
        })
    }

    /// Materialises every occurrence owed up to today. Returns how many landed.
    ///
    /// Called on unlock, so a vault left closed for a month catches up in one
    /// pass instead of needing the app open on the exact day.
    pub fn apply_due_recurring(&self) -> Result<usize, FinanceError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.with_db(|db| {
            db.apply_due_recurring(&today)
                .map_err(FinanceError::Database)
        })
    }

    /// Writes the whole ledger to `path` as CSV and returns the row count.
    pub fn export_transactions_csv(&self, path: &str) -> Result<usize, FinanceError> {
        let transactions = self.get_transactions()?;
        let accounts = self.get_accounts()?;
        let csv = export::transactions_to_csv(&transactions, &accounts);

        std::fs::write(path, csv)
            .map_err(|e| FinanceError::Validation(format!("Failed to write export: {e}")))?;

        Ok(transactions.len())
    }

    pub fn get_transactions(&self) -> Result<Vec<Transaction>, FinanceError> {
        self.with_db(|db| FinanceRepository::get_transactions(db).map_err(FinanceError::Database))
    }

    pub fn get_balance(&self) -> Result<BalanceSummary, FinanceError> {
        self.with_db(|db| {
            FinanceRepository::get_balance_summary(db).map_err(FinanceError::Database)
        })
    }

    pub fn delete_transaction(&self, id: String) -> Result<(), FinanceError> {
        let db_arc = self.db.clone();
        TransactionOps::delete_transaction(
            |f| {
                let db_lock = db_arc.read().map_err(|_| FinanceError::Internal)?;
                let db = db_lock.as_ref().ok_or(FinanceError::NoVaultOpen)?;
                db.check_session_timeout().map_err(|e| match e {
                    DbError::SessionExpired => FinanceError::SessionExpired,
                    _ => FinanceError::Database(e),
                })?;
                f(db)?;
                db.touch_session().map_err(FinanceError::Database)?;
                Ok(())
            },
            id,
        )
    }

    // ==================== Transfer Operations ====================

    pub fn transfer_funds(&self, cmd: NewTransfer) -> Result<String, FinanceError> {
        let NewTransfer {
            from_account_id,
            to_account_id,
            amount_cents,
            description,
            date,
        } = cmd;
        let db_arc = self.db.clone();
        TransactionOps::transfer_funds(
            |f| {
                let db_lock = db_arc.read().map_err(|_| FinanceError::Internal)?;
                let db = db_lock.as_ref().ok_or(FinanceError::NoVaultOpen)?;
                db.check_session_timeout().map_err(|e| match e {
                    DbError::SessionExpired => FinanceError::SessionExpired,
                    _ => FinanceError::Database(e),
                })?;
                let result = f(db)?;
                db.touch_session().map_err(FinanceError::Database)?;
                Ok(result)
            },
            from_account_id,
            to_account_id,
            amount_cents,
            description,
            date,
        )
    }

    pub fn update_transfer(&self, cmd: UpdateTransfer) -> Result<(), FinanceError> {
        let UpdateTransfer {
            id,
            from_account_id,
            to_account_id,
            amount_cents,
            description,
            date,
        } = cmd;
        let db_arc = self.db.clone();
        TransactionOps::update_transfer(
            |f| {
                let db_lock = db_arc.read().map_err(|_| FinanceError::Internal)?;
                let db = db_lock.as_ref().ok_or(FinanceError::NoVaultOpen)?;
                db.check_session_timeout().map_err(|e| match e {
                    DbError::SessionExpired => FinanceError::SessionExpired,
                    _ => FinanceError::Database(e),
                })?;
                f(db)?;
                db.touch_session().map_err(FinanceError::Database)?;
                Ok(())
            },
            id,
            from_account_id,
            to_account_id,
            amount_cents,
            description,
            date,
        )
    }

    // ==================== Category Operations ====================

    pub fn get_transaction_categories(
        &self,
        category_type: String,
    ) -> Result<Vec<TransactionCategory>, FinanceError> {
        self.with_db(|db| CategoryOps::get_transaction_categories(db, &category_type))
    }

    pub fn add_transaction_category(
        &self,
        name: String,
        category_type: String,
    ) -> Result<String, FinanceError> {
        self.with_db(|db| CategoryOps::add_transaction_category(db, &name, &category_type))
    }

    pub fn update_transaction_category(
        &self,
        id: String,
        new_name: String,
    ) -> Result<(), FinanceError> {
        self.with_db(|db| CategoryOps::update_transaction_category(db, &id, &new_name))
    }

    pub fn delete_transaction_category(&self, id: String) -> Result<(), FinanceError> {
        self.with_db(|db| CategoryOps::delete_transaction_category(db, &id))
    }

    // ==================== Exchange Rate Operations ====================

    pub fn save_exchange_rate(&self, pair: String, rate: f64) -> Result<(), FinanceError> {
        self.with_db(|db| {
            FinanceRepository::save_exchange_rate(db, &pair, rate).map_err(FinanceError::Database)
        })
    }

    /// Cached rate for `pair`, paired with whether it is still within its TTL.
    ///
    /// The freshness rule lives here alone so a caller that shows a rate and one
    /// that converts with it cannot drift into disagreeing on what counts as
    /// current. An unparseable timestamp counts as stale.
    pub fn load_exchange_rate_checked(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String, bool)>, FinanceError> {
        self.with_db(|db| {
            let cached =
                FinanceRepository::load_exchange_rate(db, &pair).map_err(FinanceError::Database)?;

            let Some((rate, updated_at)) = cached else {
                return Ok(None);
            };

            let is_fresh = chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| {
                    let age = Utc::now().signed_duration_since(dt.with_timezone(&Utc));
                    age.num_seconds() <= EXCHANGE_RATE_TTL_SECS
                })
                .unwrap_or(false);

            Ok(Some((rate, updated_at, is_fresh)))
        })
    }

    pub fn load_exchange_rate_allow_stale(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, FinanceError> {
        Ok(self
            .load_exchange_rate_checked(pair)?
            .map(|(rate, updated_at, _)| (rate, updated_at)))
    }

    pub fn load_exchange_rate(&self, pair: String) -> Result<Option<(f64, String)>, FinanceError> {
        Ok(self
            .load_exchange_rate_checked(pair)?
            .filter(|(_, _, is_fresh)| *is_fresh)
            .map(|(rate, updated_at, _)| (rate, updated_at)))
    }

    // ==================== Analytics Operations ====================

    pub fn get_expenses_by_category(&self) -> Result<Vec<(String, i64)>, FinanceError> {
        let transactions = self.get_transactions()?;
        let accounts = self.get_accounts()?;
        let usd_rates = self.load_account_usd_rates(&accounts);

        Ok(DashboardCharts::get_expenses_by_category(
            &transactions,
            &accounts,
            &usd_rates,
        ))
    }

    /// Returns dashboard data with chart values for rendering (FIAT + Crypto combined).
    /// The crypto_total_usd parameter should be calculated by the caller.
    /// The crypto_snapshots contain historical portfolio values for accurate chart rendering.
    /// Use controller.render_net_worth_chart() to render the chart image.
    pub fn get_dashboard_data(
        &self,
        crypto_total_usd: f64,
        crypto_snapshots: &[(String, f64, f64)],
        range: String,
        preferred_currency: String,
    ) -> Result<DashboardData, FinanceError> {
        let balances = self.get_account_balances()?;
        let accounts = self.get_accounts()?;
        let transactions = self.get_transactions()?;
        let usd_rates = self.load_account_usd_rates(&accounts);

        Ok(DashboardCharts::calculate_dashboard_data(
            &balances,
            &accounts,
            &transactions,
            crypto_total_usd,
            crypto_snapshots,
            &usd_rates,
            &range,
            &preferred_currency,
        ))
    }

    fn load_account_usd_rates(&self, accounts: &[Account]) -> HashMap<String, f64> {
        let mut rates = HashMap::from([("USD".to_string(), 1.0)]);
        let currencies: HashSet<String> = accounts
            .iter()
            .map(|account| account.currency.trim().to_uppercase())
            .filter(|currency| !currency.is_empty())
            .collect();

        for currency in currencies {
            if currency == "USD" {
                continue;
            }

            let pair = format!("{}_USD", currency);
            let rate = self
                .load_exchange_rate_allow_stale(pair)
                .ok()
                .and_then(|entry| entry.map(|(value, _)| value))
                .filter(|value| *value > 0.0)
                .unwrap_or(1.0);
            rates.insert(currency, rate);
        }

        rates
    }
}

#[cfg(test)]
mod tests;
