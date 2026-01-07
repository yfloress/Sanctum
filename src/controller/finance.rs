//! Finance-related controller methods
//!
//! FIAT accounts, transactions, and categories.

use super::{AppController, ControllerError};
use crate::models::{
    Account, AccountBalance, BalanceSummary, Transaction, TransactionCategory,
};

impl AppController {
    // ==================== FIAT Account Methods ====================

    /// Creates a new account
    pub fn create_account(
        &self,
        name: String,
        account_type: String,
        currency: String,
        initial_balance: i64,
        color: String,
        icon: Option<String>,
    ) -> Result<String, ControllerError> {
        self.finance_service
            .create_account(
                name,
                account_type,
                currency,
                initial_balance,
                color,
                icon,
            )
            .map_err(ControllerError::from)
    }

    /// Gets all accounts
    pub fn get_accounts(&self) -> Result<Vec<Account>, ControllerError> {
        self.finance_service
            .get_accounts()
            .map_err(ControllerError::from)
    }

    /// Gets all account balances
    pub fn get_account_balances(&self) -> Result<Vec<AccountBalance>, ControllerError> {
        self.finance_service
            .get_account_balances()
            .map_err(ControllerError::from)
    }

    /// Updates an account
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
    ) -> Result<(), ControllerError> {
        self.finance_service
            .update_account(
                id,
                name,
                account_type,
                currency,
                initial_balance,
                color,
                icon,
            )
            .map_err(ControllerError::from)
    }

    /// Updates an account icon
    pub fn update_account_icon(
        &self,
        id: String,
        icon: Option<String>,
    ) -> Result<(), ControllerError> {
        self.finance_service
            .update_account_icon(id, icon)
            .map_err(ControllerError::from)
    }

    /// Archives an account (soft delete)
    pub fn archive_account(&self, id: String) -> Result<(), ControllerError> {
        self.finance_service
            .archive_account(id)
            .map_err(ControllerError::from)
    }

    /// Transfers funds between accounts
    pub fn transfer_funds(
        &self,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<String, ControllerError> {
        self.finance_service
            .transfer_funds(from_account_id, to_account_id, amount, description, date)
            .map_err(ControllerError::from)
    }

    /// Updates a transfer transaction
    #[allow(clippy::too_many_arguments)]
    pub fn update_transfer(
        &self,
        id: String,
        from_account_id: String,
        to_account_id: String,
        amount: i64,
        description: String,
        date: String,
    ) -> Result<(), ControllerError> {
        self.finance_service
            .update_transfer(id, from_account_id, to_account_id, amount, description, date)
            .map_err(ControllerError::from)
    }

    // ==================== Financial Transaction Methods ====================

    /// Adds a new transaction
    #[allow(clippy::too_many_arguments)]
    pub fn add_transaction(
        &self,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        is_expense: bool,
    ) -> Result<String, ControllerError> {
        self.finance_service
            .add_transaction(account_id, amount, category, description, date, is_expense)
            .map_err(ControllerError::from)
    }

    /// Updates an existing transaction
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
    ) -> Result<(), ControllerError> {
        self.finance_service
            .update_transaction(
                id,
                account_id,
                amount,
                category,
                description,
                date,
                is_expense,
            )
            .map_err(ControllerError::from)
    }

    /// Gets all transactions
    pub fn get_transactions(&self) -> Result<Vec<Transaction>, ControllerError> {
        self.finance_service
            .get_transactions()
            .map_err(ControllerError::from)
    }

    /// Gets current balance summary
    pub fn get_balance(&self) -> Result<BalanceSummary, ControllerError> {
        self.finance_service
            .get_balance()
            .map_err(ControllerError::from)
    }

    /// Gets expenses by category for analytics
    pub fn get_expenses_by_category(&self) -> Result<Vec<(String, i64)>, ControllerError> {
        self.finance_service
            .get_expenses_by_category()
            .map_err(ControllerError::from)
    }

    /// Deletes a transaction
    pub fn delete_transaction(&self, id: String) -> Result<(), ControllerError> {
        self.finance_service
            .delete_transaction(id)
            .map_err(ControllerError::from)
    }

    // ==================== Transaction Category Methods ====================

    /// Gets transaction categories by type
    pub fn get_transaction_categories(
        &self,
        category_type: String,
    ) -> Result<Vec<TransactionCategory>, ControllerError> {
        self.finance_service
            .get_transaction_categories(category_type)
            .map_err(ControllerError::from)
    }

    /// Adds a new transaction category
    pub fn add_transaction_category(
        &self,
        name: String,
        category_type: String,
    ) -> Result<String, ControllerError> {
        self.finance_service
            .add_transaction_category(name, category_type)
            .map_err(ControllerError::from)
    }

    /// Updates a transaction category name
    pub fn update_transaction_category(
        &self,
        id: String,
        new_name: String,
    ) -> Result<(), ControllerError> {
        self.finance_service
            .update_transaction_category(id, new_name)
            .map_err(ControllerError::from)
    }

    /// Deletes a transaction category
    pub fn delete_transaction_category(&self, id: String) -> Result<(), ControllerError> {
        self.finance_service
            .delete_transaction_category(id)
            .map_err(ControllerError::from)
    }

    // ==================== Exchange Rate Methods ====================

    /// Saves exchange rate to cache
    pub fn save_exchange_rate(&self, pair: String, rate: f64) -> Result<(), ControllerError> {
        self.finance_service
            .save_exchange_rate(pair, rate)
            .map_err(ControllerError::from)
    }

    /// Loads cached exchange rate, even if stale
    pub fn load_exchange_rate_allow_stale(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, ControllerError> {
        self.finance_service
            .load_exchange_rate_allow_stale(pair)
            .map_err(ControllerError::from)
    }

    /// Loads cached exchange rate
    pub fn load_exchange_rate(
        &self,
        pair: String,
    ) -> Result<Option<(f64, String)>, ControllerError> {
        self.finance_service
            .load_exchange_rate(pair)
            .map_err(ControllerError::from)
    }

    // ==================== Dashboard Methods ====================

    /// Provides dashboard data with chart values (FIAT + Crypto combined).
    /// Use render_net_worth_chart() to render the chart image from the values.
    pub fn get_dashboard_data(
        &self,
        crypto_total_usd: f64,
        crypto_snapshots: &[(String, f64, f64)],
        range: String,
    ) -> Result<super::DashboardData, ControllerError> {
        self.finance_service
            .get_dashboard_data(crypto_total_usd, crypto_snapshots, range)
            .map_err(ControllerError::from)
    }
}
