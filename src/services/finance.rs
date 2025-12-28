use crate::db::{Database, DbError};
use crate::models::{Account, AccountBalance, BalanceSummary, Transaction, TransactionCategory};
use crate::security_log::{SecurityEvent, log_security_event};
use chrono::{Datelike, Local, NaiveDate, Utc};
use rusqlite::Error as RusqliteError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct ExpenseSlice {
    pub category: String,
    pub amount: i64,
    pub percentage: f32,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct AnalyticsSummary {
    pub chart_path: String,
    pub net_worth: String,
    pub max_value: String,
    pub min_value: String,
    pub expense_slices: Vec<ExpenseSlice>,
}

const MAX_CATEGORY_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 512;
const MAX_ACCOUNT_NAME_LENGTH: usize = 64;
const MAX_CURRENCY_LENGTH: usize = 8;
const MAX_ICON_LENGTH: usize = 32;
const EXCHANGE_RATE_TTL_SECS: i64 = 6 * 60 * 60; // 6 hours

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
                if i.is_empty() { None } else { Some(i) }
            } else {
                None
            };

            let id = Uuid::new_v4().to_string();
            let created_at = chrono::Utc::now().to_rfc3339();

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

            db.create_account(&account)?;
            log_security_event(SecurityEvent::TransactionCreated, Some("account_created"));
            Ok(id)
        })
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>, FinanceError> {
        self.with_db(|db| db.get_accounts().map_err(FinanceError::Database))
    }

    pub fn get_account_balances(&self) -> Result<Vec<AccountBalance>, FinanceError> {
        self.with_db(|db| {
            db.get_all_account_balances()
                .map_err(FinanceError::Database)
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
                if i.is_empty() { None } else { Some(i) }
            } else {
                None
            };

            let existing = db.get_account(&validated_id)?;
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

            db.update_account(&account)
                .map_err(FinanceError::Database)
        })
    }

    pub fn archive_account(&self, id: String) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.archive_account(&validated_id)
                .map_err(FinanceError::Database)
        })
    }

    pub fn transfer_funds(
        &self,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<String, FinanceError> {
        self.with_db(|db| {
            let from_id = validate_uuid(&from_account_id)?;
            let to_id = validate_uuid(&to_account_id)?;

            if amount <= 0 {
                return Err(FinanceError::Validation(
                    "Transfer amount must be greater than zero".to_string(),
                ));
            }

            let from_account = db.get_account(&from_id)?;
            let to_account = db.get_account(&to_id)?;
            if from_account.currency.to_uppercase() != to_account.currency.to_uppercase() {
                return Err(FinanceError::Validation(
                    "Transfers require both accounts to use the same currency".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            let tx_id = db.create_transfer(&from_id, &to_id, amount, &description, &date)?;
            log_security_event(SecurityEvent::TransactionCreated, Some("transfer"));
            Ok(tx_id)
        })
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
        self.with_db(|db| {
            let id = validate_uuid(&id)?;
            let from_id = validate_uuid(&from_account_id)?;
            let to_id = validate_uuid(&to_account_id)?;

            if amount <= 0 {
                return Err(FinanceError::Validation(
                    "Transfer amount must be greater than zero".to_string(),
                ));
            }

            let from_account = db.get_account(&from_id)?;
            let to_account = db.get_account(&to_id)?;
            if from_account.currency.to_uppercase() != to_account.currency.to_uppercase() {
                return Err(FinanceError::Validation(
                    "Transfers require both accounts to use the same currency".to_string(),
                ));
            }

            let description =
                validate_field_length(&description, MAX_DESCRIPTION_LENGTH, "Description")?;
            let description = sanitize_string(&description);
            let date = validate_date(&date)?;

            db.update_transfer(&id, &from_id, &to_id, amount, &description, &date)?;
            Ok(())
        })
    }

    pub fn add_transaction(
        &self,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        is_expense: bool,
    ) -> Result<String, FinanceError> {
        self.with_db(|db| {
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

            db.create_transaction(&transaction)?;
            log_security_event(SecurityEvent::TransactionCreated, Some(transaction_type));
            Ok(id)
        })
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
        self.with_db(|db| {
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

            db.update_transaction(&transaction)?;
            Ok(())
        })
    }

    pub fn get_transactions(&self) -> Result<Vec<Transaction>, FinanceError> {
        self.with_db(|db| db.get_transactions().map_err(FinanceError::Database))
    }

    pub fn get_balance(&self) -> Result<BalanceSummary, FinanceError> {
        self.with_db(|db| db.get_balance_summary().map_err(FinanceError::Database))
    }

    pub fn get_expenses_by_category(&self) -> Result<Vec<(String, i64)>, FinanceError> {
        let transactions = self.get_transactions()?;
        let accounts = self.get_accounts()?;

        let currency_map: HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let clp_rate = match self.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
            Ok(Some((r, _))) => r,
            _ => 1.0,
        };
        let rate = if clp_rate > 0.0 { clp_rate } else { 1.0 };

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
        };

        let mut map: HashMap<String, i64> = HashMap::new();

        for tx in transactions {
            if tx.transaction_type == "expense" {
                let amount = normalize(tx.amount, &tx.account_id);
                *map.entry(tx.category).or_default() += amount;
            }
        }

        let mut result: Vec<(String, i64)> = map.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(result)
    }

    pub fn delete_transaction(&self, id: String) -> Result<(), FinanceError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.delete_transaction(&validated_id)?;
            log_security_event(SecurityEvent::TransactionDeleted, None);
            Ok(())
        })
    }

    pub fn get_transaction_categories(
        &self,
        category_type: String,
    ) -> Result<Vec<TransactionCategory>, FinanceError> {
        if category_type != "expense" && category_type != "income" {
            return Err(FinanceError::Validation(
                "Category type must be 'expense' or 'income'".to_string(),
            ));
        }

        self.with_db(|db| {
            db.get_transaction_categories(&category_type)
                .map_err(FinanceError::Database)
        })
    }

    pub fn add_transaction_category(
        &self,
        name: String,
        category_type: String,
    ) -> Result<String, FinanceError> {
        let validated_name = validate_field_length(&name, MAX_CATEGORY_LENGTH, "Category name")?;

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

        self.with_db(|db| {
            db.add_transaction_category(&validated_name, &category_type)
                .map_err(|e| match e {
                    DbError::Sqlite(RusqliteError::ExecuteReturnedResults) => {
                        FinanceError::Validation(
                            "Category with this name already exists".to_string(),
                        )
                    }
                    _ => FinanceError::Database(e),
                })
        })
    }

    pub fn update_transaction_category(
        &self,
        id: String,
        new_name: String,
    ) -> Result<(), FinanceError> {
        let validated_id = validate_uuid(&id)?;
        let validated_name =
            validate_field_length(&new_name, MAX_CATEGORY_LENGTH, "Category name")?;

        if validated_name.is_empty() {
            return Err(FinanceError::Validation(
                "Category name cannot be empty".to_string(),
            ));
        }

        self.with_db(|db| {
            db.update_transaction_category(&validated_id, &validated_name)
                .map_err(|e| match e {
                    DbError::Sqlite(RusqliteError::ExecuteReturnedResults) => {
                        FinanceError::Validation(
                            "Category with this name already exists".to_string(),
                        )
                    }
                    _ => FinanceError::Database(e),
                })
        })
    }

    pub fn delete_transaction_category(&self, id: String) -> Result<(), FinanceError> {
        let validated_id = validate_uuid(&id)?;

        self.with_db(|db| {
            db.delete_transaction_category(&validated_id)
                .map_err(FinanceError::Database)
        })
    }

    pub fn save_exchange_rate(&self, pair: String, rate: f64) -> Result<(), FinanceError> {
        self.with_db(|db| {
            db.save_exchange_rate(&pair, rate)
                .map_err(FinanceError::Database)
        })
    }

    pub fn load_exchange_rate_allow_stale(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, FinanceError> {
        self.with_db(|db| {
            db.load_exchange_rate(&pair)
                .map_err(FinanceError::Database)
        })
    }

    pub fn load_exchange_rate(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, FinanceError> {
        self.with_db(|db| {
            let cached = db
                .load_exchange_rate(&pair)
                .map_err(FinanceError::Database)?;

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

    pub fn get_analytics_summary(&self, range: String) -> Result<AnalyticsSummary, FinanceError> {
        let balances = self.get_account_balances()?;
        let accounts = self.get_accounts()?;
        let transactions = self.get_transactions()?;

        let currency_map: HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let clp_rate = match self.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
            Ok(Some((r, _))) => r,
            _ => 1.0,
        };
        let rate = if clp_rate > 0.0 { clp_rate } else { 1.0 };

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
        };

        let current_balance: i64 = balances
            .iter()
            .map(|b| normalize(b.current_balance, &b.account_id))
            .sum();

        let today = Local::now().date_naive();
        let start_date = match range.as_str() {
            "1M" => today
                .checked_sub_signed(chrono::Duration::days(30))
                .unwrap_or(today),
            "3M" => today
                .checked_sub_signed(chrono::Duration::days(90))
                .unwrap_or(today),
            "6M" => today
                .checked_sub_signed(chrono::Duration::days(180))
                .unwrap_or(today),
            "1Y" => today
                .checked_sub_signed(chrono::Duration::days(365))
                .unwrap_or(today),
            _ => today,
        };

        let mut delta_by_day: HashMap<NaiveDate, i64> = HashMap::new();
        let mut earliest_tx: Option<NaiveDate> = None;
        for tx in &transactions {
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d") {
                let raw_delta = match tx.transaction_type.as_str() {
                    "income" => tx.amount,
                    "expense" => -tx.amount,
                    _ => 0,
                };
                let delta = normalize(raw_delta, &tx.account_id);

                *delta_by_day.entry(date).or_insert(0) += delta;
                earliest_tx = Some(earliest_tx.map_or(date, |d| d.min(date)));
            }
        }

        let effective_start = if range == "ALL" {
            earliest_tx.unwrap_or(today)
        } else {
            start_date.min(today)
        };

        let mut cursor = today;
        let mut points_rev: Vec<(NaiveDate, i64)> = Vec::new();
        let mut balance = current_balance;

        loop {
            points_rev.push((cursor, balance));
            let delta = *delta_by_day.get(&cursor).unwrap_or(&0);
            balance -= delta;

            if cursor <= effective_start {
                break;
            }
            if let Some(prev) = cursor.pred_opt() {
                cursor = prev;
            } else {
                break;
            }
        }

        points_rev.reverse();
        if points_rev.is_empty() {
            points_rev.push((today, current_balance));
        }

        let values: Vec<i64> = points_rev.iter().map(|(_, v)| *v).collect();
        let min_val = *values.iter().min().unwrap_or(&0);
        let max_val = *values.iter().max().unwrap_or(&0);
        let safe_range = ((max_val - min_val) as f64).max(1.0);

        let points: Vec<(f64, f64)> = values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = (i as f64 / (values.len().max(2) - 1) as f64) * 100.0;
                let y_ratio = (v - min_val) as f64 / safe_range;
                let y = 100.0 - (5.0 + (y_ratio * 90.0));
                (x, y)
            })
            .collect();

        let mut path_cmd = String::new();
        if !points.is_empty() {
            path_cmd.push_str(&format!("M {:.2} {:.2}", points[0].0, points[0].1));

            for i in 0..points.len() - 1 {
                let p0 = if i == 0 { points[0] } else { points[i - 1] };
                let p1 = points[i];
                let p2 = points[i + 1];
                let p3 = if i + 2 < points.len() { points[i + 2] } else { p2 };

                let cp1x = p1.0 + (p2.0 - p0.0) / 6.0;
                let cp1y = p1.1 + (p2.1 - p0.1) / 6.0;

                let cp2x = p2.0 - (p3.0 - p1.0) / 6.0;
                let cp2y = p2.1 - (p3.1 - p1.1) / 6.0;

                path_cmd.push_str(&format!(
                    " C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                    cp1x, cp1y, cp2x, cp2y, p2.0, p2.1
                ));
            }
        } else {
            path_cmd = "M 0 50 L 100 50".to_string();
        }

        let mut expenses: HashMap<String, i64> = HashMap::new();
        let current_month = today.month();
        let current_year = today.year();
        for tx in &transactions {
            if tx.transaction_type != "expense" {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d")
                && date.year() == current_year
                && date.month() == current_month
            {
                let amount = normalize(tx.amount, &tx.account_id);
                *expenses.entry(tx.category.to_uppercase()).or_insert(0) += amount;
            }
        }

        let total_expense: i64 = expenses.values().sum();
        let mut expense_slices: Vec<ExpenseSlice> = Vec::new();
        if total_expense > 0 {
            let mut by_amount: Vec<(String, i64)> = expenses.into_iter().collect();
            by_amount.sort_by(|a, b| b.1.cmp(&a.1));

            let colors = [
                "#8b5cf6", "#ec4899", "#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#6366f1",
                "#14b8a6",
            ];

            for (idx, (category, amount)) in by_amount.iter().enumerate() {
                if *amount <= 0 {
                    continue;
                }
                let percentage = *amount as f32 / total_expense as f32;
                let color = colors[idx % colors.len()].to_string();

                expense_slices.push(ExpenseSlice {
                    category: category.clone(),
                    amount: *amount,
                    percentage,
                    color,
                });
            }
        }

        Ok(AnalyticsSummary {
            chart_path: path_cmd,
            net_worth: format_money_display(current_balance),
            max_value: format_money_display(max_val),
            min_value: format_money_display(min_val),
            expense_slices,
        })
    }

    pub fn get_net_worth_history(
        &self,
        range: &str,
    ) -> Result<(String, String, String, String), FinanceError> {
        let accounts = self.get_accounts()?;
        let transactions = self.get_transactions()?;

        let currency_map: HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.to_uppercase()))
            .collect();

        let clp_rate = match self.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
            Ok(Some((r, _))) => r,
            _ => 1.0,
        };
        let rate = if clp_rate > 0.0 { clp_rate } else { 1.0 };

        let normalize = |amount: i64, account_id: &str| -> i64 {
            let currency = currency_map
                .get(account_id)
                .map(|s| s.as_str())
                .unwrap_or("USD");
            if currency == "CLP" {
                ((amount as f64) / rate) as i64
            } else {
                amount
            }
        };

        struct FinancialEvent {
            date: NaiveDate,
            amount_delta: i64,
        }

        let mut events: Vec<FinancialEvent> = Vec::new();

        for acc in &accounts {
            if acc.initial_balance != 0 {
                let date = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&acc.created_at) {
                    dt.date_naive()
                } else if let Ok(d) = NaiveDate::parse_from_str(&acc.created_at, "%Y-%m-%d") {
                    d
                } else {
                    Local::now().date_naive()
                };

                let amount_delta = normalize(acc.initial_balance, &acc.id);
                events.push(FinancialEvent { date, amount_delta });
            }
        }

        for tx in &transactions {
            let raw_delta = match tx.transaction_type.as_str() {
                "income" => tx.amount,
                "expense" => -tx.amount,
                _ => 0,
            };

            if raw_delta == 0 {
                continue;
            }

            if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d") {
                let amount_delta = normalize(raw_delta, &tx.account_id);
                events.push(FinancialEvent { date, amount_delta });
            }
        }

        events.sort_by(|a, b| a.date.cmp(&b.date));

        let mut full_history: Vec<(NaiveDate, i64)> = Vec::new();
        let mut current_balance = 0;

        if let Some(first) = events.first() {
            full_history.push((first.date.pred_opt().unwrap_or(first.date), 0));
        } else {
            full_history.push((Local::now().date_naive(), 0));
        }

        for event in events {
            current_balance += event.amount_delta;
            full_history.push((event.date, current_balance));
        }

        let today = Local::now().date_naive();
        if full_history.last().is_some_and(|last| last.0 < today) {
            full_history.push((today, current_balance));
        }

        let net_worth_formatted = format_money_display(current_balance);

        let start_date = match range {
            "1M" => Some(today - chrono::Duration::days(30)),
            "3M" => Some(today - chrono::Duration::days(90)),
            "6M" => Some(today - chrono::Duration::days(180)),
            "1Y" => Some(today - chrono::Duration::days(365)),
            _ => None,
        };

        let filtered_history: Vec<(NaiveDate, i64)> = if let Some(start) = start_date {
            let start_balance = full_history
                .iter()
                .rfind(|(d, _)| *d <= start)
                .map(|(_, b)| *b)
                .unwrap_or(0);

            let mut range_points: Vec<(NaiveDate, i64)> = Vec::new();
            range_points.push((start, start_balance));
            range_points.extend(full_history.into_iter().filter(|(d, _)| *d >= start));
            range_points
        } else {
            full_history
        };

        if filtered_history.is_empty() {
            return Ok((
                "M 0 50 L 100 50".to_string(),
                net_worth_formatted,
                "$ 0.00".to_string(),
                "$ 0.00".to_string(),
            ));
        }

        let balances: Vec<i64> = filtered_history.iter().map(|(_, b)| *b).collect();
        let min_val = *balances.iter().min().unwrap_or(&0);
        let max_val = *balances.iter().max().unwrap_or(&0);

        let min_formatted = format_money_display(min_val);
        let max_formatted = format_money_display(max_val);

        let len = balances.len() as f32;
        let mut path_cmd = String::new();

        let range = (max_val - min_val) as f32;
        let safe_range = if range == 0.0 { 1.0 } else { range };

        for (idx, val) in balances.iter().enumerate() {
            let x = if len > 1.0 {
                (idx as f32) * (100.0 / (len - 1.0))
            } else {
                0.0
            };

            let y_norm = if max_val == min_val {
                50.0
            } else {
                let ratio = (*val - min_val) as f32 / safe_range;
                100.0 - (5.0 + (ratio * 90.0))
            };

            if idx == 0 {
                path_cmd.push_str(&format!("M {:.2} {:.2}", x, y_norm));
            } else {
                path_cmd.push_str(&format!(" L {:.2} {:.2}", x, y_norm));
            }
        }

        if path_cmd.is_empty() {
            path_cmd = "M 0 50 L 100 50".to_string();
        }

        Ok((path_cmd, net_worth_formatted, min_formatted, max_formatted))
    }
}

fn validate_field_length(
    value: &str,
    max_length: usize,
    field_name: &str,
) -> Result<String, FinanceError> {
    let trimmed = value.trim();
    if trimmed.len() > max_length {
        return Err(FinanceError::Validation(format!(
            "{} exceeds maximum length of {} characters",
            field_name, max_length
        )));
    }
    Ok(trimmed.to_string())
}

fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric()
                || c.is_whitespace()
                || matches!(
                    c,
                    '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '-' | '_' | '+'
                        | '=' | '{' | '}' | '[' | ']' | '|' | '\\' | ':' | '\'' | '"' | ',' | '.'
                        | '<' | '>' | '?' | '/' | '`' | '~'
                )
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn validate_uuid(id: &str) -> Result<String, FinanceError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(FinanceError::Validation("ID cannot be empty".to_string()));
    }

    if Uuid::parse_str(trimmed).is_ok() {
        return Ok(trimmed.to_string());
    }

    Err(FinanceError::Validation("Invalid ID format".to_string()))
}

fn validate_date(date: &str) -> Result<String, FinanceError> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return Err(FinanceError::Validation("Date cannot be empty".to_string()));
    }

    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%d-%m-%Y") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    if let Ok(parsed) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    Err(FinanceError::Validation(
        "Invalid date format. Use DD-MM-YYYY or YYYY-MM-DD".to_string(),
    ))
}

fn validate_color(color: &str) -> Result<String, FinanceError> {
    let trimmed = color.trim();

    if trimmed.is_empty() {
        return Err(FinanceError::Validation("Color cannot be empty".to_string()));
    }

    if trimmed.len() != 7 {
        return Err(FinanceError::Validation(
            "Color must be in #RRGGBB format".to_string(),
        ));
    }

    if !trimmed.starts_with('#') {
        return Err(FinanceError::Validation("Color must start with #".to_string()));
    }

    if !trimmed[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(FinanceError::Validation(
            "Color must contain valid hex characters".to_string(),
        ));
    }

    Ok(trimmed.to_lowercase())
}

fn format_money_display(value: i64) -> String {
    let abs = value.abs();
    let units = abs / 100;
    let cents = abs % 100;
    let sign = if value < 0 { "-" } else { "" };
    format!("{sign}$ {units}.{cents:02}")
}
