// Sanctum — a privacy-first personal finance, crypto, and habits vault.
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

import { app } from './app.svelte'
import * as vaultApi from '../api/vault'

let timer: ReturnType<typeof setInterval> | null = null
let lastActivity = Date.now()

function resetActivity() {
  lastActivity = Date.now()
}

async function checkTimeout() {
  if (!app.isLoggedIn || !app.settings) return

  const timeoutSecs = app.settings.session_timeout_secs
  if (timeoutSecs <= 0) return

  const elapsed = (Date.now() - lastActivity) / 1000
  if (elapsed >= timeoutSecs) {
    await vaultApi.lockVault()
    app.logout()
  }
}

export function startSessionMonitor() {
  const events = ['mousedown', 'keydown', 'touchstart', 'scroll'] as const
  events.forEach(e => document.addEventListener(e, resetActivity, { passive: true }))

  timer = setInterval(checkTimeout, 10_000)

  return () => {
    events.forEach(e => document.removeEventListener(e, resetActivity))
    if (timer) { clearInterval(timer); timer = null }
  }
}
