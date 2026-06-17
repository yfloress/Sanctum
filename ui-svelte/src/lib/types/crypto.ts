// Sanctum — a privacy-first personal finance and crypto vault.
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

export interface PortfolioResponse {
  total_value: string
  unrealized_pnl: string
  unrealized_pnl_negative: boolean
  realized_ytd: string
  realized_ytd_negative: boolean
  roi: string
  roi_negative: boolean
  assets: PortfolioAssetDto[]
  distribution: DistributionItem[]
  fx_rate: FxRateDto | null
  last_updated: string | null
}

export interface PortfolioAssetDto {
  coin_id: string
  symbol: string
  name: string
  icon_path: string | null
  price: string
  price_change_24h: string
  price_change_24h_negative: boolean
  amount: string
  value: string
  allocation_pct: number
}

export interface DistributionItem {
  coin_id: string
  symbol: string
  value: number
  percentage: number
}

export interface FxRateDto {
  pair: string
  rate: string
  is_live: boolean
}

export interface WalletDto {
  id: string
  name: string
  category: string
  icon_path: string | null
  total_value: string
  assets_count: number
}

export interface WalletsResponse {
  wallets: WalletDto[]
  simple_list: WalletSimpleDto[]
}

export interface WalletSimpleDto {
  id: string
  name: string
  category: string
}

export interface WalletDetailResponse {
  id: string
  name: string
  category: string
  icon_path: string | null
  total_value: string
  holdings: WalletHoldingDto[]
  transactions: CryptoTransactionDto[]
}

export interface WalletHoldingDto {
  coin_id: string
  symbol: string
  amount: string
  value: string
  price: string
}

export interface WalletInput {
  name: string
  category: string
  icon?: string
}

export interface CryptoTransactionDto {
  id: string
  wallet_id: string
  wallet_name: string
  coin_id: string
  symbol: string
  transaction_type: string
  subtype: string | null
  amount: string
  price: string
  fee: string
  fee_coin_id: string | null
  fee_amount: string | null
  value: string
  date: string
  notes: string | null
  has_related_tx: boolean
}

export interface CryptoTransactionInput {
  wallet_id: string
  coin_id: string
  symbol: string
  transaction_type: string
  amount: string
  price: string
  fee: string
  fee_coin_id?: string
  fee_coin_amount?: string
  date: string
  notes?: string
  subtype?: string
  override_proceeds?: string
  override_cost_basis?: string
}

export interface CryptoTransferInput {
  from_wallet_id: string
  to_wallet_id: string
  coin_id: string
  symbol: string
  from_amount: string
  to_amount: string
  fee: string
  fee_coin_id?: string
  fee_coin_amount?: string
  date: string
  notes?: string
}

export interface CryptoSwapInput {
  wallet_id: string
  from_coin_id: string
  from_symbol: string
  from_amount: string
  to_coin_id: string
  to_symbol: string
  to_amount: string
  fee: string
  fee_coin_id?: string
  fee_coin_amount?: string
  date: string
  notes?: string
}

export interface CryptoTransactionUpdateInput {
  id: string
  amount: string
  price: string
  fee: string
  fee_coin_id?: string
  fee_coin_amount?: string
  date: string
  notes?: string
  subtype?: string
  override_proceeds?: string
  override_cost_basis?: string
}

export interface CryptoTransactionEditData {
  id: string
  wallet_name: string
  coin_id: string
  symbol: string
  transaction_type: string
  subtype: string | null
  amount: string
  price: string
  fee: string
  fee_coin_id: string | null
  fee_coin_amount: string | null
  date: string
  notes: string | null
  override_proceeds: string | null
  override_cost_basis: string | null
  is_paired_swap: boolean
}

export interface AssetDetailResponse {
  coin_id: string
  symbol: string
  name: string
  price: string
  total_amount: string
  total_value: string
  unrealized_pnl: string
  unrealized_pnl_negative: boolean
  wallet_breakdown: AssetWalletBreakdown[]
  transactions: CryptoTransactionDto[]
}

export interface AssetWalletBreakdown {
  wallet_id: string
  wallet_name: string
  amount: string
  value: string
}

export interface TickerDto {
  coin_id: string
  symbol: string
  name: string
  icon_path: string | null
  price: string
  change_24h: string
  change_24h_negative: boolean
}

export interface TickerOptionDto {
  coin_id: string
  symbol: string
  name: string
  enabled: boolean
  is_custom: boolean
}

export interface CoinCatalogDto {
  id: string
  name: string
  symbol: string
  is_custom: boolean
  is_favorite: boolean
}

export interface TaxSettingsDto {
  period_id: string
  jurisdiction: string
  method: string
  include_swaps: boolean
  include_fee_crypto: boolean
  excluded_wallet_ids: string[]
}

export interface TaxReportDto {
  period_id: string
  jurisdiction: string
  method: string
  disposals_count: number
  total_proceeds: string
  total_cost: string
  total_gain: string
  total_gain_negative: boolean
  short_term_gain: string | null
  long_term_gain: string | null
  events: TaxEventDto[]
  warnings: TaxWarningDto[]
  readiness: TaxReadinessDto[]
}

export interface TaxEventDto {
  tx_id: string
  date: string
  coin_id: string
  symbol: string
  amount: string
  proceeds: string
  cost_basis: string
  gain: string
  gain_negative: boolean
  term: string | null
  disposal_type: string
}

export interface TaxWarningDto {
  code: string
  message: string
  tx_id: string | null
}

export interface TaxReadinessDto {
  code: string
  status: string
  detail: string
}

export interface TaxSummaryDto {
  report: TaxReportDto
  taxable_income_total: string
  taxable_income_count: number
  end_balance_value: string | null
  end_balance_missing: number
  transactions_in_period: number
  volume_processed: string
}

export interface IpcSummaryDto {
  records_count: number
  date_range: string | null
}

export interface PortfolioTrendData {
  dates: string[]
  values: number[]
}

export interface CryptoAssetPriceDto {
  id: string
  symbol: string
  name: string
  /** Raw price in USD — for calculations. */
  current_price: number
  /** Formatted price in preferred currency — use this for display. */
  current_price_display: string
  price_change_percentage_24h: number
  last_updated: string
}
