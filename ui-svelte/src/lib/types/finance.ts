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

export interface AccountDto {
  id: string
  name: string
  account_type: string
  account_type_key: string
  icon_path: string | null
  currency: string
  balance: string
  balance_negative: boolean
  initial_balance: string
  is_archived: boolean
}

export interface AccountsResponse {
  accounts: AccountDto[]
  total_balance: string
  total_balance_negative: boolean
}

export interface AccountDetailResponse {
  id: string
  name: string
  account_type: string
  currency: string
  balance: string
  balance_negative: boolean
  icon_path: string | null
  transactions: TransactionDto[]
}

export interface TransactionDto {
  id: string
  account_id: string
  account_name: string
  date: string
  description: string
  description_raw: string
  category: string
  category_raw: string
  amount: string
  amount_raw: string
  is_expense: boolean
  is_transfer: boolean
  transfer_account_id: string | null
  transfer_account_name: string | null
}

export interface TransactionsResponse {
  transactions: TransactionDto[]
  has_more: boolean
}

export interface TransactionInput {
  id?: string
  account_id: string
  amount: string
  category: string
  description: string
  date: string
  is_expense: boolean
}

export interface TransactionFilter {
  query?: string
  account_id?: string
  category?: string
  limit?: number
}

export interface TransferInput {
  id?: string
  from_account_id: string
  to_account_id: string
  amount: string
  description: string
  date: string
}

export interface CategoryDto {
  id: string
  name: string
  is_default: boolean
}

export interface CategoriesResponse {
  expense: CategoryDto[]
  income: CategoryDto[]
}

export interface AccountInput {
  id?: string
  name: string
  account_type: string
  currency: string
  initial_balance: string
}
