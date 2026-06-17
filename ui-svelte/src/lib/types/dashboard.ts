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

export interface BalanceOverview {
  total: string
  total_negative: boolean
  fiat_total: string
  fiat_negative: boolean
  crypto_total: string
  crypto_negative: boolean
  currency: string
}

export interface RecentTransaction {
  id: string
  date: string
  description: string
  category: string
  amount: string
  is_expense: boolean
  is_transfer: boolean
  account_name: string
}

export interface ExpenseBreakdownItem {
  category: string
  amount: string
  percentage: number
  color: string
}

export interface MonthlyCashFlowItem {
  month: string
  income: number
  expenses: number
}

export interface AnalyticsData {
  net_worth: string
  net_worth_min: string
  net_worth_max: string
  total_income: string
  total_expenses: string
  total_net: string
  total_net_negative: boolean
  expense_breakdown: ExpenseBreakdownItem[]
  chart: NetWorthChartData
  monthly_cash_flow: MonthlyCashFlowItem[]
}

export interface NetWorthChartData {
  dates: string[]
  values: number[]
}
