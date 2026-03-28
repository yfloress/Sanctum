import { invoke } from '@tauri-apps/api/core'
import type {
  AccountsResponse, AccountDetailResponse, AccountInput,
  TransactionsResponse, TransactionInput, TransactionFilter,
  TransferInput, CategoriesResponse
} from '../types'

export async function fetchAccounts(): Promise<AccountsResponse> {
  return invoke<AccountsResponse>('fetch_accounts')
}

export async function fetchAccountDetails(id: string): Promise<AccountDetailResponse> {
  return invoke<AccountDetailResponse>('fetch_account_details', { id })
}

export async function createAccount(input: AccountInput): Promise<void> {
  return invoke('create_account', { ...input })
}

export async function updateAccount(input: AccountInput): Promise<void> {
  return invoke('update_account', { ...input })
}

export async function deleteAccount(id: string): Promise<void> {
  return invoke('delete_account', { id })
}

export async function updateAccountIcon(id: string, icon: string): Promise<void> {
  return invoke('update_account_icon', { id, icon })
}

export async function updateAccountName(id: string, newName: string): Promise<void> {
  return invoke('update_account_name', { id, newName })
}

export async function transferFunds(input: TransferInput): Promise<void> {
  return invoke('transfer_funds', { ...input })
}

export async function updateTransfer(input: TransferInput): Promise<void> {
  return invoke('update_transfer', { ...input })
}

export async function fetchTransactions(filter: TransactionFilter): Promise<TransactionsResponse> {
  return invoke<TransactionsResponse>('fetch_transactions', { ...filter })
}

export async function addTransaction(input: TransactionInput): Promise<void> {
  return invoke('add_transaction', { ...input })
}

export async function updateTransaction(input: TransactionInput): Promise<void> {
  return invoke('update_transaction', { ...input })
}

export async function deleteTransaction(id: string): Promise<void> {
  return invoke('delete_transaction', { id })
}

export async function loadCategories(): Promise<CategoriesResponse> {
  return invoke<CategoriesResponse>('load_categories')
}

export async function addCategory(name: string, categoryType: string): Promise<void> {
  return invoke('add_category', { name, categoryType })
}

export async function updateCategory(id: string, name: string, categoryType: string): Promise<void> {
  return invoke('update_category', { id, name, categoryType })
}

export async function deleteCategory(id: string): Promise<void> {
  return invoke('delete_category', { id })
}
