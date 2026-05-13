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

//! Finance domain DTOs.
//!
//! Covers: accounts, transactions, categories, transfers.

use serde::{Deserialize, Serialize};

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
