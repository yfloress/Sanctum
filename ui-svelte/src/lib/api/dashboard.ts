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
