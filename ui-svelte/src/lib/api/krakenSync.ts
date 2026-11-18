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

import { invoke } from './ipc'

export interface KrakenSource {
  wallet_id: string | null
  on_unlock: boolean
  synced_through: number | null
}

interface KrakenResult {
  inserted: number
  duplicates: number
  synced_through: number
}

export function krakenStatus() {
  return invoke<KrakenSource>('kraken_sync_status')
}

export function configureKraken(wallet_id: string, api_key: string, api_secret: string, on_unlock: boolean) {
  return invoke<void>('configure_kraken_sync', { wallet_id, api_key, api_secret, on_unlock })
}

export function disconnectKraken() {
  return invoke<void>('disconnect_kraken_sync')
}

export function syncKraken(automatic = false) {
  return invoke<KrakenResult>('run_kraken_sync', { automatic })
}

export async function autoSyncKraken() {
  const source = await krakenStatus()
  if (source.on_unlock) return syncKraken(true)
}
