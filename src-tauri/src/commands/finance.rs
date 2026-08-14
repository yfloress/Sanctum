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

//! Finance domain Tauri commands.
//!
//! Covers: accounts (CRUD, icon, detail), transactions (CRUD, filter),
//! transfers, and categories.

use sanctum::error::AppError;
use sanctum::features::finance::FinanceService;
use sanctum::features::settings::{SETTING_PREFERRED_CURRENCY, SettingsService};
use sanctum::services::search::normalize;
use sanctum::ui::dto::finance::{
    AccountDetailResponse, AccountDto, AccountInput, AccountsResponse, BudgetDto, BudgetInput,
    CategoriesResponse, CategoryDto, RecurringDto, RecurringInput, TransactionDto,
    TransactionFilter, TransactionInput, TransactionsResponse, TransferInput,
};
use sanctum::ui::{
    format_category_label, format_decimal_from_cents, format_money, format_money_signed,
    transaction_search_text,
};
use std::collections::HashMap;
use tauri::State;

// ==================== Accounts ====================

/// Fetch all accounts with balances.
#[tauri::command]
pub fn fetch_accounts(
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
) -> Result<AccountsResponse, AppError> {
    let preferred_currency = settings
        .get_app_setting(SETTING_PREFERRED_CURRENCY)
        .unwrap_or_else(|_| "USD".to_string());

    let state =
        sanctum::ui::load_accounts_state(&finance, &preferred_currency).map_err(AppError::from)?;

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
    finance: State<'_, FinanceService>,
    account_id: String,
) -> Result<AccountDetailResponse, AppError> {
    let accounts = finance.get_accounts().map_err(AppError::from)?;
    let account = accounts
        .iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| AppError::not_found("Account not found"))?;

    let balances = finance.get_account_balances().map_err(AppError::from)?;
    let balance_cents = balances
        .iter()
        .find(|b| b.account_id == account_id)
        .map(|b| b.current_balance)
        .unwrap_or(account.initial_balance);

    let transactions = finance.get_transactions().map_err(AppError::from)?;

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
    finance: State<'_, FinanceService>,
    input: AccountInput,
) -> Result<String, AppError> {
    Ok(finance.create_account(input.into_new_account()?)?)
}

/// Update an existing account.
#[tauri::command]
pub fn update_account(
    finance: State<'_, FinanceService>,
    input: AccountInput,
) -> Result<(), AppError> {
    let existing_icon = finance
        .get_accounts()?
        .into_iter()
        .find(|a| Some(&a.id) == input.id.as_ref())
        .and_then(|a| a.icon);

    Ok(finance.update_account(input.into_update_account(existing_icon)?)?)
}

/// Transfer funds between two accounts.
#[tauri::command]
pub fn transfer_funds(
    finance: State<'_, FinanceService>,
    input: TransferInput,
) -> Result<(), AppError> {
    finance.transfer_funds(input.into_new()?)?;
    Ok(())
}

/// Update an existing transfer.
#[tauri::command]
pub fn update_transfer(
    finance: State<'_, FinanceService>,
    input: TransferInput,
) -> Result<(), AppError> {
    Ok(finance.update_transfer(input.into_update()?)?)
}

/// Archive (soft-delete) an account.
#[tauri::command]
pub fn delete_account(finance: State<'_, FinanceService>, id: String) -> Result<(), AppError> {
    finance.archive_account(id).map_err(AppError::from)
}

#[tauri::command]
pub fn fetch_archived_accounts(
    finance: State<'_, FinanceService>,
) -> Result<Vec<AccountDto>, AppError> {
    let accounts = finance.get_archived_accounts().map_err(AppError::from)?;
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
pub fn unarchive_account(finance: State<'_, FinanceService>, id: String) -> Result<(), AppError> {
    finance.unarchive_account(id).map_err(AppError::from)
}

/// Update an account's bank icon.
#[tauri::command]
pub fn update_account_icon(
    finance: State<'_, FinanceService>,
    id: String,
    icon: String,
) -> Result<(), AppError> {
    let icon_path =
        sanctum::ui::normalize_bank_icon_path(if icon.is_empty() { None } else { Some(icon) });
    finance
        .update_account_icon(id, icon_path)
        .map_err(AppError::from)
}

/// Rename an account.
#[tauri::command]
pub fn update_account_name(
    finance: State<'_, FinanceService>,
    id: String,
    new_name: String,
) -> Result<(), AppError> {
    finance
        .update_account_name(id, new_name)
        .map_err(AppError::from)
}

// ==================== Transactions ====================

/// Fetch transactions with optional filters.
#[tauri::command]
pub fn fetch_transactions(
    finance: State<'_, FinanceService>,
    filter: TransactionFilter,
) -> Result<TransactionsResponse, AppError> {
    let TransactionFilter {
        query,
        account_id,
        category,
        date_from,
        date_to,
        limit,
        sort,
    } = filter;

    let accounts = finance.get_accounts().map_err(AppError::from)?;
    let account_lookup: HashMap<String, (String, String)> = accounts
        .iter()
        .map(|a| (a.id.clone(), (a.currency.clone(), a.name.clone())))
        .collect();

    let transactions = finance.get_transactions().map_err(AppError::from)?;

    // Folded the same way the haystack is, so "credito" finds "Crédito".
    let query_lower = normalize(&query.unwrap_or_default());
    let account_filter = account_id.unwrap_or_default();
    let category_filter = category.unwrap_or_default();
    let category_filter_upper = category_filter.to_uppercase();
    let date_from_filter = date_from.unwrap_or_default();
    let date_to_filter = date_to.unwrap_or_default();
    let display_limit = limit.unwrap_or(100);

    let mut matched: Vec<sanctum::models::Transaction> = transactions
        .into_iter()
        .filter(|tx| {
            let is_transfer = tx.transaction_type == "transfer";

            // Account filter
            if !account_filter.is_empty()
                && tx.account_id != account_filter
                && tx.transfer_account_id.as_deref() != Some(account_filter.as_str())
            {
                return false;
            }

            // Category filter
            if !category_filter.is_empty()
                && ((is_transfer && category_filter_upper != "TRANSFER")
                    || (!is_transfer
                        && (category_filter_upper == "TRANSFER"
                            || !tx.category.eq_ignore_ascii_case(&category_filter))))
            {
                return false;
            }

            // Date range filter (ISO YYYY-MM-DD compares lexicographically)
            if !date_from_filter.is_empty() && tx.date.as_str() < date_from_filter.as_str() {
                return false;
            }
            if !date_to_filter.is_empty() && tx.date.as_str() > date_to_filter.as_str() {
                return false;
            }

            // Text search
            if !query_lower.is_empty() {
                let (_, from_name) = account_lookup
                    .get(&tx.account_id)
                    .cloned()
                    .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));
                let transfer_name = if is_transfer {
                    tx.transfer_account_id
                        .as_ref()
                        .and_then(|tid| account_lookup.get(tid))
                        .map(|(_, name)| name.as_str())
                } else {
                    None
                };
                let haystack = transaction_search_text(
                    &tx.description,
                    &tx.category,
                    &tx.date,
                    &from_name,
                    transfer_name,
                );
                if !haystack.contains(&query_lower) {
                    return false;
                }
            }

            true
        })
        .collect();

    // Sorted over everything that matched, not over the page: ordering after
    // truncation would rank the newest hundred rather than the whole ledger.
    sort_transactions(&mut matched, sort.as_deref());

    let matched_count = matched.len();
    let mapped: Vec<TransactionDto> = matched
        .iter()
        .take(display_limit)
        .map(|tx| build_transaction_dto(tx, &account_lookup))
        .collect();

    Ok(TransactionsResponse {
        transactions: mapped,
        has_more: matched_count > display_limit,
    })
}

/// Add a new transaction.
#[tauri::command]
pub fn add_transaction(
    finance: State<'_, FinanceService>,
    input: TransactionInput,
) -> Result<(), AppError> {
    finance.add_transaction(input.into_new()?)?;
    Ok(())
}

/// Update an existing transaction.
#[tauri::command]
pub fn update_transaction(
    finance: State<'_, FinanceService>,
    input: TransactionInput,
) -> Result<(), AppError> {
    Ok(finance.update_transaction(input.into_update()?)?)
}

/// Delete a transaction.
#[tauri::command]
pub fn delete_transaction(finance: State<'_, FinanceService>, id: String) -> Result<(), AppError> {
    finance.delete_transaction(id).map_err(AppError::from)
}

/// Delete a whole selection. Returns how many rows went.
#[tauri::command]
pub fn delete_transactions(
    finance: State<'_, FinanceService>,
    ids: Vec<String>,
) -> Result<usize, AppError> {
    finance.delete_transactions(ids).map_err(AppError::from)
}

/// Move a whole selection to one category. Returns how many rows changed:
/// transfers in the selection keep their structural category and are not counted.
#[tauri::command]
pub fn recategorize_transactions(
    finance: State<'_, FinanceService>,
    ids: Vec<String>,
    category: String,
) -> Result<usize, AppError> {
    finance
        .recategorize_transactions(ids, category)
        .map_err(AppError::from)
}

// ==================== Categories ====================

/// Load all expense and income categories.
#[tauri::command]
pub fn load_categories(finance: State<'_, FinanceService>) -> Result<CategoriesResponse, AppError> {
    let expense = finance
        .get_transaction_categories("expense".to_string())
        .map_err(AppError::from)?
        .into_iter()
        .map(|c| CategoryDto {
            label: format_category_label(&c.name),
            name: c.name,
            id: c.id,
            is_default: c.is_default,
        })
        .collect();

    let income = finance
        .get_transaction_categories("income".to_string())
        .map_err(AppError::from)?
        .into_iter()
        .map(|c| CategoryDto {
            label: format_category_label(&c.name),
            name: c.name,
            id: c.id,
            is_default: c.is_default,
        })
        .collect();

    Ok(CategoriesResponse { expense, income })
}

/// Add a new category.
#[tauri::command]
pub fn add_category(
    finance: State<'_, FinanceService>,
    name: String,
    category_type: String,
) -> Result<(), AppError> {
    finance
        .add_transaction_category(name, category_type)
        .map(|_| ())
        .map_err(AppError::from)
}

/// Update a category name.
#[tauri::command]
pub fn update_category(
    finance: State<'_, FinanceService>,
    id: String,
    new_name: String,
) -> Result<(), AppError> {
    finance
        .update_transaction_category(id, new_name)
        .map_err(AppError::from)
}

/// Delete a category.
#[tauri::command]
pub fn delete_category(finance: State<'_, FinanceService>, id: String) -> Result<(), AppError> {
    finance
        .delete_transaction_category(id)
        .map_err(AppError::from)
}

// ==================== Helpers ====================

/// Build a TransactionDto from a raw transaction model.
/// Reorders the matched transactions in place.
///
/// Amount ordering compares magnitudes: the sign carries expense versus income,
/// which every row already shows on its own, so the useful question is which
/// movements were the largest. The sort is stable, so ties keep the newest
/// first, and an unrecognised key leaves the query's own order untouched.
fn sort_transactions(transactions: &mut [sanctum::models::Transaction], sort: Option<&str>) {
    match sort {
        // The query returns `date DESC, rowid DESC`, so this is `date ASC`.
        Some("date-asc") => transactions.reverse(),
        Some("amount-desc") => transactions.sort_by_key(|tx| std::cmp::Reverse(tx.amount.abs())),
        Some("amount-asc") => transactions.sort_by_key(|tx| tx.amount.abs()),
        _ => {}
    }
}

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

// ==================== Export ====================

/// Write the whole transaction ledger to `path` as CSV.
///
/// Returns the number of rows written so the frontend can report it.
#[tauri::command]
pub fn export_transactions_csv(
    finance: State<'_, FinanceService>,
    path: String,
) -> Result<usize, AppError> {
    finance
        .export_transactions_csv(&path)
        .map_err(AppError::from)
}

// ==================== Recurring Transactions ====================

/// List every recurring rule, soonest first.
#[tauri::command]
pub fn fetch_recurring(
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
) -> Result<Vec<RecurringDto>, AppError> {
    let rules = finance.get_recurring().map_err(AppError::from)?;
    let accounts = finance.get_accounts().map_err(AppError::from)?;
    let names: HashMap<String, (String, String)> = accounts
        .iter()
        .map(|a| (a.id.clone(), (a.name.clone(), a.currency.clone())))
        .collect();
    let preferred = settings
        .get_app_setting(SETTING_PREFERRED_CURRENCY)
        .unwrap_or_else(|_| "USD".to_string());

    Ok(rules
        .into_iter()
        .map(|rule| {
            let (account_name, currency) = names
                .get(&rule.account_id)
                .cloned()
                .unwrap_or_else(|| (String::new(), preferred.clone()));
            RecurringDto {
                amount: format_money(rule.amount, &currency),
                amount_raw: format_decimal_from_cents(rule.amount),
                category_label: format_category_label(&rule.category),
                is_expense: rule.transaction_type == "expense",
                id: rule.id,
                account_id: rule.account_id,
                account_name,
                category: rule.category,
                description: rule.description,
                frequency: rule.frequency,
                next_date: rule.next_date,
                is_active: rule.is_active,
            }
        })
        .collect())
}

/// Create a recurring rule.
#[tauri::command]
pub fn add_recurring(
    finance: State<'_, FinanceService>,
    input: RecurringInput,
) -> Result<(), AppError> {
    finance.create_recurring(input.into_new()?)?;
    Ok(())
}

/// Pause or resume a rule without losing it.
#[tauri::command]
pub fn set_recurring_active(
    finance: State<'_, FinanceService>,
    id: String,
    active: bool,
) -> Result<(), AppError> {
    finance
        .set_recurring_active(id, active)
        .map_err(AppError::from)
}

/// Delete a rule. Transactions it already created stay.
#[tauri::command]
pub fn delete_recurring(finance: State<'_, FinanceService>, id: String) -> Result<(), AppError> {
    finance.delete_recurring(id).map_err(AppError::from)
}

/// Create every occurrence owed up to today. Returns how many landed.
#[tauri::command]
pub fn apply_due_recurring(finance: State<'_, FinanceService>) -> Result<usize, AppError> {
    finance.apply_due_recurring().map_err(AppError::from)
}

// ==================== Category Budgets ====================

/// List the budgets with this month's progress against them.
#[tauri::command]
pub fn fetch_budgets(
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
    month: Option<String>,
) -> Result<Vec<BudgetDto>, AppError> {
    let currency = settings
        .get_app_setting(SETTING_PREFERRED_CURRENCY)
        .unwrap_or_else(|_| "USD".to_string());

    Ok(finance
        .get_budget_status(month)
        .map_err(AppError::from)?
        .into_iter()
        .map(|status| {
            let over_budget = status.spent > status.limit;
            let percentage = if status.limit > 0 {
                ((status.spent as f64 / status.limit as f64) * 100.0).min(100.0) as f32
            } else {
                0.0
            };
            BudgetDto {
                category_label: format_category_label(&status.category),
                category: status.category,
                limit: format_money(status.limit, &currency),
                limit_raw: format_decimal_from_cents(status.limit),
                spent: format_money(status.spent, &currency),
                percentage,
                over_budget,
                remaining: format_money((status.limit - status.spent).abs(), &currency),
            }
        })
        .collect())
}

/// Set or replace a category's monthly limit.
#[tauri::command]
pub fn set_budget(finance: State<'_, FinanceService>, input: BudgetInput) -> Result<(), AppError> {
    let amount = sanctum::ui::parse_amount_input(&input.amount)
        .filter(|v| *v > 0)
        .ok_or_else(|| {
            AppError::validation("Budget must be greater than zero").with_field("amount")
        })?;
    finance
        .set_category_budget(input.category, amount)
        .map_err(AppError::from)
}

/// Remove a category's budget.
#[tauri::command]
pub fn delete_budget(finance: State<'_, FinanceService>, category: String) -> Result<(), AppError> {
    finance
        .delete_category_budget(category)
        .map_err(AppError::from)
}
