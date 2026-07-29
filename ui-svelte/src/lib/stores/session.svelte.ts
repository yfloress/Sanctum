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

// Inactivity auto-lock. Two clocks can diverge: the frontend's, reset by user
// interaction, and the backend's, reset only when a command runs. The countdown
// takes whichever runs out first, and interaction refreshes both.

import { app } from './app.svelte'
import { errorKind } from '../errors'
import * as vaultApi from '../api/vault'
import * as settingsApi from '../api/settings'

/** Remaining seconds at which the lock warning appears. */
const WARNING_THRESHOLD_SECS = 60
/** Countdown refresh interval. */
const TICK_MS = 1_000
/** How often the backend clock is re-sampled. */
const POLL_MS = 5_000
/** Minimum gap between backend session refreshes driven by DOM activity. */
const TOUCH_THROTTLE_MS = 60_000

class SessionState {
  /** Seconds until auto-lock while the warning is up; `null` when hidden. */
  warningSecs = $state<number | null>(null)
}

export const session = new SessionState()

let lastActivity = Date.now()
let lastTouch = 0
let lastPoll = 0
let timer: ReturnType<typeof setInterval> | null = null

/** Last reading of the backend clock, extrapolated between polls. */
let backendSample: { secs: number; at: number } | null = null

function backendRemaining(): number | null {
  if (!backendSample) return null
  return backendSample.secs - Math.floor((Date.now() - backendSample.at) / 1000)
}

/** Refreshes the backend clock. `get_session_remaining` deliberately does not
 *  extend it, so this needs a real command; `load_settings` is the cheapest. */
async function keepBackendAlive(force = false) {
  if (!force && Date.now() - lastTouch < TOUCH_THROTTLE_MS) return
  lastTouch = Date.now()
  try {
    await settingsApi.loadSettings()
    // The backend clock just reset; the stale sample would under-report.
    backendSample = null
  } catch {
    // A dead session surfaces on the next tick, which locks properly.
  }
}

function resetActivity() {
  // Seconds from expiry: skip the throttle so this reaches the backend.
  const nearExpiry = session.warningSecs !== null
  lastActivity = Date.now()
  void keepBackendAlive(nearExpiry)
}

async function lock() {
  session.warningSecs = null
  try {
    await vaultApi.lockVault()
  } catch {
    // Already locked or no vault open — logging out is still correct.
  }
  app.logout()
}

async function tick() {
  if (!app.isLoggedIn || !app.settings) return

  const timeoutSecs = app.settings.session_timeout_secs
  if (timeoutSecs <= 0) {
    session.warningSecs = null
    return
  }

  if (Date.now() - lastPoll >= POLL_MS) {
    lastPoll = Date.now()
    try {
      backendSample = { secs: await settingsApi.getSessionRemaining(), at: Date.now() }
    } catch (e) {
      if (errorKind(e) === 'session_expired') {
        await lock()
        return
      }
      // Anything else (no vault open, transient failure): trust the local clock.
      backendSample = null
    }
  }

  const localRemaining = timeoutSecs - Math.floor((Date.now() - lastActivity) / 1000)
  const backend = backendRemaining()
  const remaining = backend === null ? localRemaining : Math.min(localRemaining, backend)

  if (remaining <= 0) {
    await lock()
    return
  }

  session.warningSecs = remaining <= WARNING_THRESHOLD_SECS ? remaining : null
}

/** Keeps the vault open: called from the warning's "stay unlocked" button. */
export function extendSession() {
  lastActivity = Date.now()
  session.warningSecs = null
  void keepBackendAlive(true)
}

/** Locks the vault immediately, from the warning's "lock now" button. */
export function lockNow() {
  void lock()
}

export function startSessionMonitor() {
  const events = ['mousedown', 'keydown', 'touchstart', 'scroll'] as const
  events.forEach(e => document.addEventListener(e, resetActivity, { passive: true }))

  lastActivity = Date.now()
  lastTouch = Date.now()
  lastPoll = 0
  backendSample = null
  timer = setInterval(() => void tick(), TICK_MS)

  return () => {
    events.forEach(e => document.removeEventListener(e, resetActivity))
    if (timer) {
      clearInterval(timer)
      timer = null
    }
    session.warningSecs = null
  }
}
