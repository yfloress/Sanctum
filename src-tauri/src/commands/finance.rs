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

//! Finance domain Tauri commands.
//!
//! Covers: accounts (CRUD, icon, detail), transactions (CRUD, filter),
//! transfers, and categories.

use sanctum::controller::{AppController, SETTING_PREFERRED_CURRENCY};
use sanctum::ui::dto::finance::{
    AccountDetailResponse, AccountDto, AccountsResponse, CategoriesResponse, CategoryDto,
    TransactionDto, TransactionsResponse,
};
use sanctum::ui::{
    format_category_label, format_decimal_from_cents, format_money, format_money_signed,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

// ==================== Accounts ====================

/// Fetch all accounts with balances.
#[tauri::command]
pub fn fetch_accounts(
    controller: State<'_, Arc<AppController>>,
) -> Result<AccountsResponse, String> {
    let preferred_currency = controller
        .get_app_setting(SETTING_PREFERRED_CURRENCY)
        .unwrap_or_else(|_| "USD".to_string());

    let state = sanctum::ui::load_accounts_state(&controller, &preferred_currency)
        .map_err(|e| e.to_string())?;

    let accounts: Vec<AccountDto> = state
        .accounts
        .into_iter()
        .map(|acc| AccountDto {
            id: acc.id,
            name: acc.name,
            account_type: acc.account_type,
            account_type_key: acc.account_type_key,
            icon_path: acc.icon_path,
            currency: acc.currency,
            balance: acc.balance,
            balance_negative: acc.balance_negative,
            initial_balance: acc.initial_balance,
            is_archived: acc.is_archived,
        })
        .collect();

    Ok(AccountsResponse {
        accounts,
        total_balance: state.total_balance,
        total_balance_negative: state.total_balance_negative,
    })
}

/// Fetch single account detail with transaction history.
#[tauri::command]
pub fn fetch_account_details(
    controller: State<'_, Arc<AppController>>,
    account_id: String,
) -> Result<AccountDetailResponse, String> {
    let accounts = controller.get_accounts().map_err(|e| e.to_string())?;
    let account = accounts
        .iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| "Account not found".to_string())?;

    let balances = controller
        .get_account_balances()
        .map_err(|e| e.to_string())?;
    let balance_cents = balances
        .iter()
        .find(|b| b.account_id == account_id)
        .map(|b| b.current_balance)
        .unwrap_or(account.initial_balance);

    let transactions = controller.get_transactions().map_err(|e| e.to_string())?;

    let account_lookup: HashMap<String, (String, String)> = accounts
        .iter()
        .map(|a| (a.id.clone(), (a.currency.clone(), a.name.clone())))
        .collect();

    let mapped: Vec<TransactionDto> = transactions
        .into_iter()
        .filter(|tx| {
            tx.account_id == account_id
                || tx.transfer_account_id.as_deref() == Some(account_id.as_str())
        })
        .map(|tx| build_transaction_dto(&tx, &account_lookup))
        .collect();

    Ok(AccountDetailResponse {
        id: account.id.clone(),
        name: account.name.clone(),
        account_type: account.account_type.clone(),
        currency: account.currency.clone(),
        balance: format_money_signed(balance_cents, &account.currency),
        balance_negative: balance_cents < 0,
        icon_path: account.icon.clone(),
        transactions: mapped,
    })
}

/// Create a new financial account.
#[tauri::command]
pub fn create_account(
    controller: State<'_, Arc<AppController>>,
    name: String,
    account_type: String,
    currency: String,
    initial_balance: String,
) -> Result<String, String> {
    let amount_cents = sanctum::ui::parse_amount_input(&initial_balance).unwrap_or(0);
    let account_type_key = sanctum::ui::normalize_account_type(&account_type);

    controller
        .create_account(
            name,
            account_type_key,
            currency.to_uppercase(),
            amount_cents,
            "#8b5cf6".to_string(),
            None,
        )
        .map_err(|e| e.to_string())
}

/// Update an existing account.
#[tauri::command]
pub fn update_account(
    controller: State<'_, Arc<AppController>>,
    id: String,
    name: String,
    account_type: String,
    currency: String,
    initial_balance: String,
) -> Result<(), String> {
    let amount_cents = sanctum::ui::parse_amount_input(&initial_balance).unwrap_or(0);
    let existing_icon = controller
        .get_accounts()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|a| a.id == id)
        .and_then(|a| a.icon.clone());

    controller
        .update_account(
            id,
            name,
            sanctum::ui::normalize_account_type(&account_type),
            currency.to_uppercase(),
            amount_cents,
            "#8b5cf6".to_string(),
            existing_icon,
        )
        .map_err(|e| e.to_string())
}

/// Transfer funds between two accounts.
#[tauri::command]
pub fn transfer_funds(
    controller: State<'_, Arc<AppController>>,
    from_account_id: String,
    to_account_id: String,
    amount: String,
    description: String,
    date: String,
) -> Result<(), String> {
    let amount_cents = sanctum::ui::parse_amount_input(&amount)
        .filter(|v| *v > 0)
        .ok_or_else(|| "Amount must be greater than zero".to_string())?;

    controller
        .transfer_funds(
            from_account_id,
            to_account_id,
            amount_cents,
            description,
            date,
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Update an existing transfer.
#[tauri::command]
pub fn update_transfer(
    controller: State<'_, Arc<AppController>>,
    id: String,
    from_account_id: String,
    to_account_id: String,
    amount: String,
    description: String,
    date: String,
) -> Result<(), String> {
    let amount_cents = sanctum::ui::parse_amount_input(&amount)
        .filter(|v| *v > 0)
        .ok_or_else(|| "Amount must be greater than zero".to_string())?;

    controller
        .update_transfer(
            id,
            from_account_id,
            to_account_id,
            amount_cents,
            description,
            date,
        )
        .map_err(|e| e.to_string())
}

/// Archive (soft-delete) an account.
#[tauri::command]
pub fn delete_account(controller: State<'_, Arc<AppController>>, id: String) -> Result<(), String> {
    controller.archive_account(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn fetch_archived_accounts(
    controller: State<'_, Arc<AppController>>,
) -> Result<Vec<AccountDto>, String> {
    let accounts = controller
        .get_archived_accounts()
        .map_err(|e| e.to_string())?;
    Ok(accounts
        .into_iter()
        .map(|acc| AccountDto {
            id: acc.id,
            name: acc.name.clone(),
            account_type: match acc.account_type.as_str() {
                "bank" | "Bank" => "Bank".to_string(),
                "cash" | "Cash" => "Cash".to_string(),
                "savings" | "Savings" => "Savings".to_string(),
                "credit_card" | "CreditCard" => "Credit Card".to_string(),
                _ => acc.account_type.clone(),
            },
            account_type_key: acc.account_type.clone(),
            icon_path: sanctum::ui::normalize_bank_icon_path(acc.icon),
            currency: acc.currency,
            balance: String::new(),
            balance_negative: false,
            initial_balance: format_decimal_from_cents(acc.initial_balance),
            is_archived: true,
        })
        .collect())
}

#[tauri::command]
pub fn unarchive_account(
    controller: State<'_, Arc<AppController>>,
    id: String,
) -> Result<(), String> {
    controller.unarchive_account(id).map_err(|e| e.to_string())
}

/// Update an account's bank icon.
#[tauri::command]
pub fn update_account_icon(
    controller: State<'_, Arc<AppController>>,
    id: String,
    icon: String,
) -> Result<(), String> {
    let icon_path =
        sanctum::ui::normalize_bank_icon_path(if icon.is_empty() { None } else { Some(icon) });
    controller
        .update_account_icon(id, icon_path)
        .map_err(|e| e.to_string())
}

/// Rename an account.
#[tauri::command]
pub fn update_account_name(
    controller: State<'_, Arc<AppController>>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    controller
        .update_account_name(id, new_name)
        .map_err(|e| e.to_string())
}

// ==================== Transactions ====================

/// Fetch transactions with optional filters.
#[tauri::command]
pub fn fetch_transactions(
    controller: State<'_, Arc<AppController>>,
    query: Option<String>,
    account_id: Option<String>,
    category: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    limit: Option<usize>,
) -> Result<TransactionsResponse, String> {
    let accounts = controller.get_accounts().map_err(|e| e.to_string())?;
    let account_lookup: HashMap<String, (String, String)> = accounts
        .iter()
        .map(|a| (a.id.clone(), (a.currency.clone(), a.name.clone())))
        .collect();

    let transactions = controller.get_transactions().map_err(|e| e.to_string())?;

    let query_lower = query.unwrap_or_default().trim().to_lowercase();
    let account_filter = account_id.unwrap_or_default();
    let category_filter = category.unwrap_or_default();
    let category_filter_upper = category_filter.to_uppercase();
    let date_from_filter = date_from.unwrap_or_default();
    let date_to_filter = date_to.unwrap_or_default();
    let display_limit = limit.unwrap_or(100);

    let mut matched_count: usize = 0;

    let mapped: Vec<TransactionDto> = transactions
        .into_iter()
        .filter_map(|tx| {
            let is_transfer = tx.transaction_type == "transfer";

            // Account filter
            if !account_filter.is_empty()
                && tx.account_id != account_filter
                && tx.transfer_account_id.as_deref() != Some(account_filter.as_str())
            {
                return None;
            }

            // Category filter
            if !category_filter.is_empty()
                && ((is_transfer && category_filter_upper != "TRANSFER")
                    || (!is_transfer
                        && (category_filter_upper == "TRANSFER"
                            || !tx.category.eq_ignore_ascii_case(&category_filter))))
            {
                return None;
            }

            // Date range filter (ISO YYYY-MM-DD compares lexicographically)
            if !date_from_filter.is_empty() && tx.date.as_str() < date_from_filter.as_str() {
                return None;
            }
            if !date_to_filter.is_empty() && tx.date.as_str() > date_to_filter.as_str() {
                return None;
            }

            // Text search
            if !query_lower.is_empty() {
                let (_, from_name) = account_lookup
                    .get(&tx.account_id)
                    .cloned()
                    .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));
                let mut haystack = format!(
                    "{} {} {} {}",
                    tx.description, tx.category, tx.date, from_name
                )
                .to_lowercase();
                if is_transfer
                    && let Some(ref tid) = tx.transfer_account_id
                    && let Some((_, tname)) = account_lookup.get(tid)
                {
                    haystack.push(' ');
                    haystack.push_str(&tname.to_lowercase());
                }
                if !haystack.contains(&query_lower) {
                    return None;
                }
            }

            matched_count += 1;
            if matched_count > display_limit {
                return None;
            }

            Some(build_transaction_dto(&tx, &account_lookup))
        })
        .collect();

    Ok(TransactionsResponse {
        transactions: mapped,
        has_more: matched_count > display_limit,
    })
}

/// Add a new transaction.
#[tauri::command]
pub fn add_transaction(
    controller: State<'_, Arc<AppController>>,
    account_id: String,
    amount: String,
    category: String,
    description: String,
    date: String,
    is_expense: bool,
) -> Result<(), String> {
    let amount_cents = sanctum::ui::parse_amount_input(&amount)
        .filter(|v| *v > 0)
        .ok_or_else(|| "Amount must be greater than zero".to_string())?;

    controller
        .add_transaction(
            account_id,
            amount_cents,
            category,
            description,
            date,
            is_expense,
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Update an existing transaction.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_transaction(
    controller: State<'_, Arc<AppController>>,
    id: String,
    account_id: String,
    amount: String,
    category: String,
    description: String,
    date: String,
    is_expense: bool,
) -> Result<(), String> {
    let amount_cents = sanctum::ui::parse_amount_input(&amount)
        .filter(|v| *v > 0)
        .ok_or_else(|| "Amount must be greater than zero".to_string())?;

    controller
        .update_transaction(
            id,
            account_id,
            amount_cents,
            category,
            description,
            date,
            is_expense,
        )
        .map_err(|e| e.to_string())
}

/// Delete a transaction.
#[tauri::command]
pub fn delete_transaction(
    controller: State<'_, Arc<AppController>>,
    id: String,
) -> Result<(), String> {
    controller.delete_transaction(id).map_err(|e| e.to_string())
}

// ==================== Categories ====================

/// Load all expense and income categories.
#[tauri::command]
pub fn load_categories(
    controller: State<'_, Arc<AppController>>,
) -> Result<CategoriesResponse, String> {
    let expense = controller
        .get_transaction_categories("expense".to_string())
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| CategoryDto {
            id: c.id,
            name: c.name,
            is_default: c.is_default,
        })
        .collect();

    let income = controller
        .get_transaction_categories("income".to_string())
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| CategoryDto {
            id: c.id,
            name: c.name,
            is_default: c.is_default,
        })
        .collect();

    Ok(CategoriesResponse { expense, income })
}

/// Add a new category.
#[tauri::command]
pub fn add_category(
    controller: State<'_, Arc<AppController>>,
    name: String,
    category_type: String,
) -> Result<(), String> {
    controller
        .add_transaction_category(name, category_type)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Update a category name.
#[tauri::command]
pub fn update_category(
    controller: State<'_, Arc<AppController>>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    controller
        .update_transaction_category(id, new_name)
        .map_err(|e| e.to_string())
}

/// Delete a category.
#[tauri::command]
pub fn delete_category(
    controller: State<'_, Arc<AppController>>,
    id: String,
) -> Result<(), String> {
    controller
        .delete_transaction_category(id)
        .map_err(|e| e.to_string())
}

// ==================== Helpers ====================

/// Build a TransactionDto from a raw transaction model.
fn build_transaction_dto(
    tx: &sanctum::models::Transaction,
    account_lookup: &HashMap<String, (String, String)>,
) -> TransactionDto {
    let (currency, from_name) = account_lookup
        .get(&tx.account_id)
        .cloned()
        .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));

    let is_transfer = tx.transaction_type == "transfer";
    let is_expense = tx.transaction_type == "expense";

    let transfer_label = tx
        .transfer_account_id
        .as_ref()
        .and_then(|id| account_lookup.get(id))
        .map(|(_, name)| name.clone());

    let description = if is_transfer {
        let label = transfer_label.as_deref().unwrap_or("Account");
        if tx.description.is_empty() {
            format!("{from_name} -> {label}")
        } else {
            format!("{} ({from_name} -> {label})", tx.description)
        }
    } else {
        tx.description.clone()
    };

    let category_raw = if is_transfer {
        "TRANSFER".to_string()
    } else {
        tx.category.to_uppercase()
    };

    TransactionDto {
        id: tx.id.clone(),
        account_id: tx.account_id.clone(),
        account_name: from_name,
        date: tx.date.clone(),
        description,
        description_raw: tx.description.clone(),
        category: format_category_label(&category_raw),
        category_raw,
        amount: format_money(tx.amount.abs(), &currency),
        amount_raw: format_decimal_from_cents(tx.amount),
        is_expense,
        is_transfer,
        transfer_account_id: tx.transfer_account_id.clone(),
        transfer_account_name: transfer_label,
    }
}
