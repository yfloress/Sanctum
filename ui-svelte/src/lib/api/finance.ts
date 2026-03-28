import { invoke } from '@tauri-apps/api/core'
import type {
  AccountsResponse, AccountDetailResponse,
  TransactionsResponse, TransferInput, CategoriesResponse
} from '../types'

export async function fetchAccounts(): Promise<AccountsResponse> {
  return invoke<AccountsResponse>('fetch_accounts')
}

export async function fetchAccountDetails(account_id: string): Promise<AccountDetailResponse> {
  return invoke<AccountDetailResponse>('fetch_account_details', { account_id })
}

export async function createAccount(
  name: string, account_type: string, currency: string, initial_balance: string
): Promise<void> {
  return invoke('create_account', { name, account_type, currency, initial_balance })
}

export async function updateAccount(
  id: string, name: string, account_type: string, currency: string, initial_balance: string
): Promise<void> {
  return invoke('update_account', { id, name, account_type, currency, initial_balance })
}

export async function deleteAccount(id: string): Promise<void> {
  return invoke('delete_account', { id })
}

export async function updateAccountIcon(id: string, icon: string): Promise<void> {
  return invoke('update_account_icon', { id, icon })
}

export async function updateAccountName(id: string, new_name: string): Promise<void> {
  return invoke('update_account_name', { id, new_name })
}

export async function transferFunds(input: TransferInput): Promise<void> {
  return invoke('transfer_funds', {
    from_account_id: input.from_account_id,
    to_account_id: input.to_account_id,
    amount: input.amount,
    description: input.description,
    date: input.date,
  })
}

export async function updateTransfer(input: TransferInput): Promise<void> {
  return invoke('update_transfer', {
    id: input.id,
    from_account_id: input.from_account_id,
    to_account_id: input.to_account_id,
    amount: input.amount,
    description: input.description,
    date: input.date,
  })
}

export async function fetchTransactions(
  query?: string, account_id?: string, category?: string, limit?: number
): Promise<TransactionsResponse> {
  return invoke<TransactionsResponse>('fetch_transactions', {
    query: query ?? null,
    account_id: account_id ?? null,
    category: category ?? null,
    limit: limit ?? null,
  })
}

export async function addTransaction(
  account_id: string, amount: string, category: string,
  description: string, date: string, is_expense: boolean
): Promise<void> {
  return invoke('add_transaction', { account_id, amount, category, description, date, is_expense })
}

export async function updateTransaction(
  id: string, account_id: string, amount: string, category: string,
  description: string, date: string, is_expense: boolean
): Promise<void> {
  return invoke('update_transaction', { id, account_id, amount, category, description, date, is_expense })
}

export async function deleteTransaction(id: string): Promise<void> {
  return invoke('delete_transaction', { id })
}

export async function loadCategories(): Promise<CategoriesResponse> {
  return invoke<CategoriesResponse>('load_categories')
}

export async function addCategory(name: string, category_type: string): Promise<void> {
  return invoke('add_category', { name, category_type })
}

export async function updateCategory(id: string, new_name: string): Promise<void> {
  return invoke('update_category', { id, new_name })
}

export async function deleteCategory(id: string): Promise<void> {
  return invoke('delete_category', { id })
}
