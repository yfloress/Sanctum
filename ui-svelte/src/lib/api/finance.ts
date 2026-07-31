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

import { invoke } from '@tauri-apps/api/core'
import type {
  AccountsResponse, AccountDetailResponse,
  TransactionsResponse, TransactionFilter, TransferInput, CategoriesResponse,
  RecurringDto, RecurringInput, BudgetDto
} from '../types'

export async function fetchAccounts(): Promise<AccountsResponse> {
  return invoke<AccountsResponse>('fetch_accounts')
}

export async function fetchAccountDetails(account_id: string): Promise<AccountDetailResponse> {
  return invoke<AccountDetailResponse>('fetch_account_details', { accountId: account_id })
}

export async function createAccount(
  name: string, account_type: string, currency: string, initial_balance: string
): Promise<void> {
  return invoke('create_account', {
    input: { name, account_type, currency, initial_balance },
  })
}

export async function updateAccount(
  id: string, name: string, account_type: string, currency: string, initial_balance: string
): Promise<void> {
  return invoke('update_account', {
    input: { id, name, account_type, currency, initial_balance },
  })
}

export async function deleteAccount(id: string): Promise<void> {
  return invoke('delete_account', { id })
}

export async function fetchArchivedAccounts(): Promise<import('../types').AccountDto[]> {
  return invoke('fetch_archived_accounts')
}

export async function unarchiveAccount(id: string): Promise<void> {
  return invoke('unarchive_account', { id })
}

export async function updateAccountIcon(id: string, icon: string): Promise<void> {
  return invoke('update_account_icon', { id, icon })
}

export async function updateAccountName(id: string, new_name: string): Promise<void> {
  return invoke('update_account_name', { id, newName: new_name })
}

export async function transferFunds(input: TransferInput): Promise<void> {
  return invoke('transfer_funds', { input })
}

export async function updateTransfer(input: TransferInput): Promise<void> {
  return invoke('update_transfer', { input })
}

export async function fetchTransactions(
  filter: TransactionFilter = {}
): Promise<TransactionsResponse> {
  return invoke<TransactionsResponse>('fetch_transactions', { filter })
}

export async function addTransaction(
  account_id: string, amount: string, category: string,
  description: string, date: string, is_expense: boolean
): Promise<void> {
  return invoke('add_transaction', {
    input: { account_id, amount, category, description, date, is_expense },
  })
}

export async function updateTransaction(
  id: string, account_id: string, amount: string, category: string,
  description: string, date: string, is_expense: boolean
): Promise<void> {
  return invoke('update_transaction', {
    input: { id, account_id, amount, category, description, date, is_expense },
  })
}

export async function deleteTransaction(id: string): Promise<void> {
  return invoke('delete_transaction', { id })
}

export async function loadCategories(): Promise<CategoriesResponse> {
  return invoke<CategoriesResponse>('load_categories')
}

export async function addCategory(name: string, category_type: string): Promise<void> {
  return invoke('add_category', { name, categoryType: category_type })
}

export async function updateCategory(id: string, new_name: string): Promise<void> {
  return invoke('update_category', { id, newName: new_name })
}

export async function deleteCategory(id: string): Promise<void> {
  return invoke('delete_category', { id })
}

/** Writes the whole ledger to `path` as CSV and returns the row count. */
export async function exportTransactionsCsv(path: string): Promise<number> {
  return invoke<number>('export_transactions_csv', { path })
}

export async function fetchRecurring(): Promise<RecurringDto[]> {
  return invoke<RecurringDto[]>('fetch_recurring')
}

export async function addRecurring(input: RecurringInput): Promise<void> {
  return invoke('add_recurring', { input })
}

export async function setRecurringActive(id: string, active: boolean): Promise<void> {
  return invoke('set_recurring_active', { id, active })
}

export async function deleteRecurring(id: string): Promise<void> {
  return invoke('delete_recurring', { id })
}

/** Creates every occurrence owed up to today. Returns how many landed. */
export async function applyDueRecurring(): Promise<number> {
  return invoke<number>('apply_due_recurring')
}

export async function fetchBudgets(month?: string): Promise<BudgetDto[]> {
  return invoke<BudgetDto[]>('fetch_budgets', { month })
}

export async function setBudget(category: string, amount: string): Promise<void> {
  return invoke('set_budget', { input: { category, amount } })
}

export async function deleteBudget(category: string): Promise<void> {
  return invoke('delete_budget', { category })
}
