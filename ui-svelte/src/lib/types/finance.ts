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
