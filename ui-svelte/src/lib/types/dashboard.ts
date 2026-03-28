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

export interface AnalyticsData {
  net_worth: string
  net_worth_min: string
  net_worth_max: string
  expense_breakdown: ExpenseBreakdownItem[]
  chart: NetWorthChartData
}

export interface NetWorthChartData {
  dates: string[]
  values: number[]
}
