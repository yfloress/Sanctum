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
use crate::models::{Account, AccountBalance, BalanceSummary, Transaction, TransactionCategory};
use crate::security_log::{SecurityEvent, log_security_event};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use super::commands::{
    NewAccount, NewTransaction, NewTransfer, UpdateAccount, UpdateTransaction, UpdateTransfer,
};
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
        let NewTransaction {
            account_id,
            amount_cents,
            category,
            description,
            date,
            is_expense,
        } = cmd;
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
            account_id,
            amount_cents,
            category,
            description,
            date,
            is_expense,
        )
    }

    pub fn update_transaction(&self, cmd: UpdateTransaction) -> Result<(), FinanceError> {
        let UpdateTransaction {
            id,
            account_id,
            amount_cents,
            category,
            description,
            date,
            is_expense,
        } = cmd;
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
            id,
            account_id,
            amount_cents,
            category,
            description,
            date,
            is_expense,
        )
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

    pub fn load_exchange_rate_allow_stale(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, FinanceError> {
        self.with_db(|db| {
            FinanceRepository::load_exchange_rate(db, &pair).map_err(FinanceError::Database)
        })
    }

    pub fn load_exchange_rate(&self, pair: String) -> Result<Option<(f64, String)>, FinanceError> {
        self.with_db(|db| {
            let cached =
                FinanceRepository::load_exchange_rate(db, &pair).map_err(FinanceError::Database)?;

            if let Some((rate, updated_at)) = cached {
                let is_fresh = chrono::DateTime::parse_from_rfc3339(&updated_at)
                    .map(|dt| {
                        let age = Utc::now().signed_duration_since(dt.with_timezone(&Utc));
                        age.num_seconds() <= EXCHANGE_RATE_TTL_SECS
                    })
                    .unwrap_or(false);

                if !is_fresh {
                    return Ok(None);
                }

                return Ok(Some((rate, updated_at)));
            }

            Ok(None)
        })
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
