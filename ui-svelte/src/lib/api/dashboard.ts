import { invoke } from '@tauri-apps/api/core'
import type { BalanceOverview, RecentTransaction, AnalyticsData } from '../types'

export async function fetchBalance(): Promise<BalanceOverview> {
  return invoke<BalanceOverview>('fetch_balance')
}

export async function fetchRecent(): Promise<RecentTransaction[]> {
  return invoke<RecentTransaction[]>('fetch_recent')
}

export async function fetchAnalytics(range: string): Promise<AnalyticsData> {
  return invoke<AnalyticsData>('fetch_analytics', { range })
}
