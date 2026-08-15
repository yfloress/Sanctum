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
  reconciled: boolean
  tags: string[]
}

export interface ReconcileRowDto {
  id: string
  date: string
  description: string
  amount_cents: number
}

export interface ReconciliationResponse {
  account_id: string
  account_name: string
  currency: string
  confirmed_cents: number
  current_cents: number
  pending: ReconcileRowDto[]
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
  /** Omit to leave the existing tags alone. */
  tags?: string[]
}

/** Ordering keys the backend understands; anything else keeps newest first. */
export type TransactionSort = 'date-desc' | 'date-asc' | 'amount-desc' | 'amount-asc'

/**
 * Filtering, ordering and paging for a transaction query. Field names are
 * snake_case on purpose: Tauri only camel-maps a command's own arguments, not
 * the fields inside one.
 */
export interface TransactionFilter {
  query?: string
  account_id?: string
  category?: string
  tag?: string
  /** Inclusive ISO `YYYY-MM-DD` bounds. */
  date_from?: string
  date_to?: string
  limit?: number
  sort?: TransactionSort
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
  /** Stored name: what filters and new transactions must send back. */
  name: string
  /** Translated, display-ready version of `name`. */
  label: string
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

export interface RecurringDto {
  id: string
  account_id: string
  account_name: string
  amount: string
  amount_raw: string
  /** Stored category name, for sending back on edits. */
  category: string
  /** Translated, display-ready category. */
  category_label: string
  description: string
  frequency: 'weekly' | 'monthly' | 'yearly'
  next_date: string
  is_expense: boolean
  is_active: boolean
}

export interface RecurringInput {
  account_id: string
  amount: string
  category: string
  description: string
  frequency: string
  first_date: string
  is_expense: boolean
}

export type InstallmentKind = 'down_payment' | 'installment' | 'charge'

export interface CreditInstallmentDto {
  id: string
  kind: InstallmentKind
  /** 1-based position within its own kind. */
  number: number
  amount: string
  /** The same figure unformatted, for the correction form to start from. */
  amount_raw: string
  due_date: string
  /** When it was actually paid, which need not be the due date. */
  paid_date: string | null
  /** Why a charge was made. Only charges carry one. */
  note: string | null
  is_paid: boolean
  /** Unpaid and past its date. */
  is_overdue: boolean
}

export interface AmortizationRowDto {
  number: number
  due_date: string
  payment: string
  /** The part of the payment that only rents the money. */
  interest: string
  /** The part that actually reduces the debt. */
  principal: string
  /** What is still owed after this payment. */
  balance: string
}

export type CreditStatus = 'done' | 'overdue' | 'ahead' | 'on_track'

/**
 * How a credit was described to the borrower: either the payment is quoted and
 * the rate is buried inside it, or a rate is quoted and the payment follows.
 */
export type CreditKind = 'installments' | 'loan'

export interface CreditDto {
  id: string
  name: string
  account_id: string
  account_name: string
  /** Stored category name. */
  category: string
  /** Translated, display-ready category. */
  category_label: string
  kind: CreditKind
  /** Paid up front. Null when the credit had none. */
  down_payment: string | null
  installment_amount: string
  installment_count: number
  paid_count: number
  overdue_count: number
  total: string
  paid: string
  remaining: string
  /** Fees the lender added on top of the plan. Null when there are none. */
  charges: string | null
  /** Share of the plan paid, in money rather than in rows. */
  percentage: number
  next_due_date: string | null
  status: CreditStatus
  /** What the credit costs beyond the thing it bought. */
  interest: string | null
  cash_price: string | null
  /** Loans only: the amount financed. */
  principal: string | null
  /** Loans only: the monthly rate as a percentage, e.g. "1.79". */
  monthly_rate: string | null
  installments: CreditInstallmentDto[]
  /** Loans only: how each payment splits between interest and principal. */
  amortization: AmortizationRowDto[]
}

/**
 * Input for creating a credit. Field names are snake_case on purpose: Tauri
 * only camel-maps a command's own arguments, not the fields inside one.
 */
export interface CreditInput {
  account_id: string
  name: string
  category: string
  kind: CreditKind
  down_payment?: string
  down_payment_date?: string
  installment_amount: string
  installment_count: number
  first_due_date: string
  /** Installments mode: what it would have cost paid outright. */
  cash_price?: string
  /** Loan mode: the amount financed. */
  principal?: string
  /** Loan mode: the rate as typed, e.g. "1.79". */
  rate?: string
  /** Which period the rate is quoted in. Markets differ. */
  rate_period?: 'monthly' | 'annual'
}

export interface InstallmentUpdateInput {
  installment_id: string
  amount: string
  due_date: string
}

export interface ChargeInput {
  credit_id: string
  amount: string
  date: string
  note: string
}

export interface BudgetDto {
  /** Stored category name, for sending back on edits. */
  category: string
  /** Translated, display-ready category. */
  category_label: string
  limit: string
  limit_raw: string
  spent: string
  /** Spent as a share of the limit, capped at 100. */
  percentage: number
  over_budget: boolean
  /** Remaining amount, or the overspend when over_budget. */
  remaining: string
}
