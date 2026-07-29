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

//! Finance domain DTOs.
//!
//! Covers: accounts, transactions, categories, transfers.
//!
//! Input DTOs carry raw, stringly values from the frontend. Their `into_*`
//! methods validate user input (tagging the offending field on [`AppError`])
//! and map into the domain command structs the service consumes — keeping the
//! domain layer free of these IPC-shaped types.

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::features::finance::{
    NewAccount, NewTransaction, NewTransfer, UpdateAccount, UpdateTransaction, UpdateTransfer,
};
use crate::ui::{normalize_account_type, parse_amount_input};

/// Default account accent color when the frontend does not supply one.
const DEFAULT_ACCOUNT_COLOR: &str = "#8b5cf6";

/// Parse a required, strictly-positive money amount, tagging `field` on failure.
fn parse_positive_amount(raw: &str, field: &str) -> Result<i64, AppError> {
    parse_amount_input(raw)
        .filter(|v| *v > 0)
        .ok_or_else(|| AppError::validation("Amount must be greater than zero").with_field(field))
}

// ==================== Accounts ====================

/// Account as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct AccountDto {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub account_type_key: String,
    pub icon_path: Option<String>,
    pub currency: String,
    pub balance: String,
    pub balance_negative: bool,
    pub initial_balance: String,
    pub is_archived: bool,
}

/// Accounts list with total balance.
#[derive(Debug, Clone, Serialize)]
pub struct AccountsResponse {
    pub accounts: Vec<AccountDto>,
    pub total_balance: String,
    pub total_balance_negative: bool,
}

/// Account detail with transaction history.
#[derive(Debug, Clone, Serialize)]
pub struct AccountDetailResponse {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub balance: String,
    pub balance_negative: bool,
    pub icon_path: Option<String>,
    pub transactions: Vec<TransactionDto>,
}

/// Input for creating or updating an account.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInput {
    pub id: Option<String>,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub initial_balance: String,
}

/// Input for updating an account icon.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountIconInput {
    pub id: String,
    pub icon: String,
}

/// Input for renaming an account.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountRenameInput {
    pub id: String,
    pub new_name: String,
}

// ==================== Transactions ====================

/// Transaction as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct TransactionDto {
    pub id: String,
    pub account_id: String,
    pub account_name: String,
    pub date: String,
    pub description: String,
    pub description_raw: String,
    pub category: String,
    pub category_raw: String,
    pub amount: String,
    pub amount_raw: String,
    pub is_expense: bool,
    pub is_transfer: bool,
    pub transfer_account_id: Option<String>,
    pub transfer_account_name: Option<String>,
}

/// Paginated transaction list.
#[derive(Debug, Clone, Serialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<TransactionDto>,
    pub has_more: bool,
}

/// Input for creating or updating a transaction.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionInput {
    pub id: Option<String>,
    pub account_id: String,
    pub amount: String,
    pub category: String,
    pub description: String,
    pub date: String,
    pub is_expense: bool,
}

/// Filter parameters for transaction queries.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionFilter {
    pub query: Option<String>,
    pub account_id: Option<String>,
    pub category: Option<String>,
    pub limit: Option<usize>,
}

// ==================== Transfers ====================

/// Input for creating or updating a fund transfer.
#[derive(Debug, Clone, Deserialize)]
pub struct TransferInput {
    pub id: Option<String>,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: String,
    pub description: String,
    pub date: String,
}

// ==================== Categories ====================

/// Transaction category as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Both expense and income categories grouped.
#[derive(Debug, Clone, Serialize)]
pub struct CategoriesResponse {
    pub expense: Vec<CategoryDto>,
    pub income: Vec<CategoryDto>,
}

/// Input for creating or updating a category.
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryInput {
    pub id: Option<String>,
    pub name: String,
    pub category_type: String,
}

// ==================== DTO -> domain command mapping ====================

impl AccountInput {
    /// Map into a create command. Initial balance defaults to 0 when unparseable
    /// (matches the previous command behavior).
    pub fn into_new_account(self) -> Result<NewAccount, AppError> {
        Ok(NewAccount {
            name: self.name,
            account_type: normalize_account_type(&self.account_type),
            currency: self.currency,
            initial_balance_cents: parse_amount_input(&self.initial_balance).unwrap_or(0),
            color: DEFAULT_ACCOUNT_COLOR.to_string(),
            icon: None,
        })
    }

    /// Map into an update command. `existing_icon` is preserved (the frontend
    /// edits accounts without re-sending the icon).
    pub fn into_update_account(
        self,
        existing_icon: Option<String>,
    ) -> Result<UpdateAccount, AppError> {
        let id = self
            .id
            .ok_or_else(|| AppError::validation("Account id is required").with_field("id"))?;
        Ok(UpdateAccount {
            id,
            name: self.name,
            account_type: normalize_account_type(&self.account_type),
            currency: self.currency,
            initial_balance_cents: parse_amount_input(&self.initial_balance).unwrap_or(0),
            color: DEFAULT_ACCOUNT_COLOR.to_string(),
            icon: existing_icon,
        })
    }
}

impl TransactionInput {
    /// Map into an add command, validating the amount.
    pub fn into_new(self) -> Result<NewTransaction, AppError> {
        Ok(NewTransaction {
            account_id: self.account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            category: self.category,
            description: self.description,
            date: self.date,
            is_expense: self.is_expense,
        })
    }

    /// Map into an update command, requiring an id and validating the amount.
    pub fn into_update(self) -> Result<UpdateTransaction, AppError> {
        let id = self
            .id
            .ok_or_else(|| AppError::validation("Transaction id is required").with_field("id"))?;
        Ok(UpdateTransaction {
            id,
            account_id: self.account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            category: self.category,
            description: self.description,
            date: self.date,
            is_expense: self.is_expense,
        })
    }
}

impl TransferInput {
    /// Map into a transfer command, validating the amount.
    pub fn into_new(self) -> Result<NewTransfer, AppError> {
        Ok(NewTransfer {
            from_account_id: self.from_account_id,
            to_account_id: self.to_account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            description: self.description,
            date: self.date,
        })
    }

    /// Map into a transfer-update command, requiring an id and validating the amount.
    pub fn into_update(self) -> Result<UpdateTransfer, AppError> {
        let id = self
            .id
            .ok_or_else(|| AppError::validation("Transfer id is required").with_field("id"))?;
        Ok(UpdateTransfer {
            id,
            from_account_id: self.from_account_id,
            to_account_id: self.to_account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            description: self.description,
            date: self.date,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn transaction_invalid_amount_tags_field() {
        let input = TransactionInput {
            id: None,
            account_id: "acc".to_string(),
            amount: "0".to_string(),
            category: "food".to_string(),
            description: String::new(),
            date: "2026-01-01".to_string(),
            is_expense: true,
        };
        let err = input.into_new().unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(err.field.as_deref(), Some("amount"));
    }

    #[test]
    fn transfer_update_requires_id() {
        let input = TransferInput {
            id: None,
            from_account_id: "a".to_string(),
            to_account_id: "b".to_string(),
            amount: "100".to_string(),
            description: String::new(),
            date: "2026-01-01".to_string(),
        };
        let err = input.into_update().unwrap_err();
        assert_eq!(err.field.as_deref(), Some("id"));
    }

    #[test]
    fn account_create_maps_defaults() {
        let input = AccountInput {
            id: None,
            name: "Checking".to_string(),
            account_type: "bank".to_string(),
            currency: "usd".to_string(),
            initial_balance: "10.00".to_string(),
            // no color/icon in the DTO
        };
        let cmd = input.into_new_account().unwrap();
        assert_eq!(cmd.color, DEFAULT_ACCOUNT_COLOR);
        assert!(cmd.icon.is_none());
        assert_eq!(cmd.initial_balance_cents, 1000);
    }
}
