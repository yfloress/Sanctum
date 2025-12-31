//! Finance service
//!
//! Business logic for financial operations: accounts, transactions, analytics.
//! Split into focused submodules for maintainability.

use crate::db::{Database, DbError};
use crate::models::{Account, AccountBalance, BalanceSummary, Transaction, TransactionCategory};
use crate::security_log::{log_security_event, SecurityEvent};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::features::dashboard::DashboardCharts;
use super::repository::FinanceRepository;
use super::transactions::{CategoryOps, TransactionOps};
use super::validation::{
    sanitize_string, validate_color, validate_field_length, validate_uuid, EXCHANGE_RATE_TTL_SECS,
    MAX_ACCOUNT_NAME_LENGTH, MAX_CURRENCY_LENGTH, MAX_ICON_LENGTH,
};

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
    db: Arc<Mutex<Option<Database>>>,
}

impl FinanceService {
    pub fn new(db: Arc<Mutex<Option<Database>>>) -> Self {
        Self { db }
    }

    fn with_db<T, F>(&self, f: F) -> Result<T, FinanceError>
    where
        F: FnOnce(&Database) -> Result<T, FinanceError>,
    {
        let db_lock = self.db.lock().map_err(|_| FinanceError::Internal)?;
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

    pub fn create_account(
        &self,
        name: String,
        account_type: String,
        currency: String,
        initial_balance: i64,
        color: String,
        icon: Option<String>,
    ) -> Result<String, FinanceError> {
        self.with_db(|db| {
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
                if i.is_empty() {
                    None
                } else {
                    Some(i)
                }
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
                initial_balance,
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

    #[allow(clippy::too_many_arguments)]
    pub fn update_account(
        &self,
        id: String,
        name: String,
        account_type: String,
        currency: String,
        initial_balance: i64,
        color: String,
        icon: Option<String>,
    ) -> Result<(), FinanceError> {
        self.with_db(|db| {
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
                if i.is_empty() {
                    None
                } else {
                    Some(i)
                }
            } else {
                None
            };

            let existing = FinanceRepository::get_account(db, &validated_id)?;
            let account = Account {
                id: validated_id,
                name,
                account_type,
                currency,
                initial_balance,
                color,
                icon,
                is_archived: existing.is_archived,
                created_at: existing.created_at,
            };

            FinanceRepository::update_account(db, &account).map_err(FinanceError::Database)
        })
    }

    pub fn archive_account(&self, id: String) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            FinanceRepository::archive_account(db, &validated_id).map_err(FinanceError::Database)
        })
    }

    // ==================== Transaction Operations ====================

    pub fn add_transaction(
        &self,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        is_expense: bool,
    ) -> Result<String, FinanceError> {
        let db_arc = self.db.clone();
        TransactionOps::add_transaction(
            |f| {
                let db_lock = db_arc.lock().map_err(|_| FinanceError::Internal)?;
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
            amount,
            category,
            description,
            date,
            is_expense,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_transaction(
        &self,
        id: String,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        is_expense: bool,
    ) -> Result<(), FinanceError> {
        let db_arc = self.db.clone();
        TransactionOps::update_transaction(
            |f| {
                let db_lock = db_arc.lock().map_err(|_| FinanceError::Internal)?;
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
            amount,
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
                let db_lock = db_arc.lock().map_err(|_| FinanceError::Internal)?;
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

    pub fn transfer_funds(
        &self,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<String, FinanceError> {
        let db_arc = self.db.clone();
        TransactionOps::transfer_funds(
            |f| {
                let db_lock = db_arc.lock().map_err(|_| FinanceError::Internal)?;
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
            amount,
            description,
            date,
        )
    }

    pub fn update_transfer(
        &self,
        id: String,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<(), FinanceError> {
        let db_arc = self.db.clone();
        TransactionOps::update_transfer(
            |f| {
                let db_lock = db_arc.lock().map_err(|_| FinanceError::Internal)?;
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
            amount,
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

        let clp_rate = match self.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
            Ok(Some((r, _))) => r,
            _ => 1.0,
        };

        Ok(DashboardCharts::get_expenses_by_category(
            &transactions,
            &accounts,
            clp_rate,
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
    ) -> Result<DashboardData, FinanceError> {
        let balances = self.get_account_balances()?;
        let accounts = self.get_accounts()?;
        let transactions = self.get_transactions()?;

        let clp_rate = match self.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
            Ok(Some((r, _))) => r,
            _ => 1.0,
        };

        Ok(DashboardCharts::calculate_dashboard_data(
            &balances,
            &accounts,
            &transactions,
            crypto_total_usd,
            crypto_snapshots,
            clp_rate,
            &range,
        ))
    }
}
