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

export interface ImportResultsResponse {
  total_processed: number
  inserted: number
  skipped: number
  errors: ImportErrorDto[]
  /** Accounts the file refers to that do not exist yet. */
  missing_accounts: MissingAccountDto[]
  /** Categories the file refers to that do not exist yet. */
  missing_categories: MissingCategoryDto[]
  /** Crypto wallets the file refers to that do not exist yet. */
  missing_wallets: string[]
}

export interface MissingAccountDto {
  name: string
  currency: string
}

export interface MissingCategoryDto {
  name: string
  category_type: 'income' | 'expense'
}

export interface ImportErrorDto {
  line: number | null
  message: string
}

export interface ExchangeDetectionResult {
  exchange_id: string
  exchange: string
  suggested_wallet: string
  file_count: number
  total_records: number
}

export interface CsvAnalysisResult {
  headers: string[]
  sample_row: string[]
}

export interface CustomCsvMapping {
  date_col: string
  asset_col: string
  amount_col: string
  type_col: string | null
  fee_col: string | null
  fee_currency_col: string | null
  price_col: string | null
  notes_col: string | null
}

// ==================== Bank statements ====================

export interface StatementFormatDto {
  id: string
  label: string
}

export interface StatementRowDto {
  date: string
  description: string
  /** Whole pesos; negative is money out. */
  amount: number
  category: string
  /** The category came from an earlier movement at the same merchant. */
  learned: boolean
  /** The account already has this movement. */
  duplicate: boolean
  /** The direction was guessed from the description. */
  sign_guessed: boolean
  installment: string | null
}

export interface StatementProblemDto {
  text: string
  reason: string
}

export interface StatementPreviewDto {
  format: string
  format_label: string
  credit_card: boolean
  period_start: string | null
  period_end: string | null
  rows: StatementRowDto[]
  problems: StatementProblemDto[]
  warnings: string[]
}

export interface StatementPreviewResponse {
  status: 'ready' | 'needs_password' | 'wrong_password'
  preview: StatementPreviewDto | null
}

export interface StatementEntryInput {
  date: string
  description: string
  /** Whole pesos; negative is money out. */
  amount: number
  category: string
  transfer_account_id: string | null
}
