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

// Optional background price sync. It only runs once the user turns it on in
// Settings: by default Sanctum makes no network request nobody asked for.
// While on, it syncs when the vault opens and then every SYNC_INTERVAL_MS, only
// while the window is visible, and waits longer after each failure, since a
// shared Tor exit is often rate limited.
import { syncCryptoData } from './api/crypto'

const SYNC_INTERVAL_MS = 10 * 60_000
const MAX_BACKOFF_MS = 60 * 60_000
const CHECK_EVERY_MS = 60_000

/**
 * Starts the loop and returns the function that stops it. `lastSynced` is read
 * on every check so a manual sync also pushes the next automatic one back.
 */
export function startAutoSync(
  lastSynced: () => string | null,
  onSynced: (at: string) => void,
): () => void {
  let lastAttempt = 0
  let failures = 0
  let running = false
  let stopped = false

  async function tick() {
    if (stopped || running || document.visibilityState !== 'visible') return
    const since = Math.max(lastAttempt, Date.parse(lastSynced() ?? '') || 0)
    const wait = Math.min(MAX_BACKOFF_MS, SYNC_INTERVAL_MS * 2 ** failures)
    if (Date.now() - since < wait) return

    running = true
    lastAttempt = Date.now()
    try {
      const result = await syncCryptoData()
      failures = 0
      if (!stopped) onSynced(result.updated_at)
    } catch {
      failures++
    } finally {
      running = false
    }
  }

  const onVisibility = () => void tick()
  const timer = window.setInterval(() => void tick(), CHECK_EVERY_MS)
  document.addEventListener('visibilitychange', onVisibility)
  void tick()

  return () => {
    stopped = true
    window.clearInterval(timer)
    document.removeEventListener('visibilitychange', onVisibility)
  }
}
