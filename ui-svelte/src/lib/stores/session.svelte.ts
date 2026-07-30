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
/** Tick while the countdown is on screen and has to move every second. */
const TICK_ACTIVE_MS = 1_000
/** Tick the rest of the time. The local clock needs no better resolution. */
const TICK_IDLE_MS = 15_000
/**
 * How close to the warning the backend clock starts being sampled.
 *
 * Each sample is an IPC round trip, and away from expiry the local clock is
 * enough — so nothing is asked of the backend until the lock is near.
 */
const POLL_WINDOW_SECS = 150
/** Minimum gap between backend clock samples inside that window. */
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
let timer: ReturnType<typeof setTimeout> | null = null

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

  const localRemaining = timeoutSecs - Math.floor((Date.now() - lastActivity) / 1000)

  // Only bother the backend near the lock. Far from it the local clock decides,
  // so the steady state costs nothing but a subtraction.
  if (localRemaining <= POLL_WINDOW_SECS && Date.now() - lastPoll >= POLL_MS) {
    lastPoll = Date.now()
    try {
      backendSample = { secs: await settingsApi.getSessionRemaining(), at: Date.now() }
    } catch (e) {
      const kind = errorKind(e)
      // A closed vault behind a logged-in UI is a dead end: every command would
      // fail. Treat it like an expired session rather than leaving it stuck.
      if (kind === 'session_expired' || kind === 'no_vault_open') {
        await lock()
        return
      }
      // Transient failure: trust the local clock.
      backendSample = null
    }
  }

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

  // Self-scheduling instead of a fixed interval: one tick per second is only
  // needed while the countdown is on screen.
  let stopped = false
  const schedule = () => {
    if (stopped) return
    const delay = session.warningSecs === null ? TICK_IDLE_MS : TICK_ACTIVE_MS
    timer = setTimeout(async () => {
      await tick()
      schedule()
    }, delay)
  }
  schedule()

  return () => {
    stopped = true
    events.forEach(e => document.removeEventListener(e, resetActivity))
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
    session.warningSecs = null
  }
}
