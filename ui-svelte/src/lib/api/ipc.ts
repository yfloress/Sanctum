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

// The single door to the backend.
//
// Every api module goes through here so that a session which died on the
// backend cannot leave a logged-in shell on screen. The backend does not close
// the vault when the session expires; it starts refusing commands. Nothing else
// tells the frontend, so the refusal itself is the signal, and it has to be
// noticed wherever it happens rather than only in the places that thought to
// check.

import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { errorKind } from '../errors'

/** Backend answers that mean the vault is no longer usable. */
const SESSION_LOST = new Set(['session_expired', 'no_vault_open'])

let onSessionLost: (() => void) | null = null

/**
 * Registers what to do when the backend reports a dead session.
 *
 * Set by the session monitor while logged in, and cleared on the way out: the
 * login screen legitimately talks to a backend with no vault open, and must not
 * be read as a session that just died.
 */
export function setSessionLostHandler(handler: (() => void) | null) {
  onSessionLost = handler
}

type TimingSink = (command: string, milliseconds: number) => void
let timingSink: TimingSink | undefined

/** Samples a foreground dev window. Only frame intervals and aggregate command timings are retained. */
export async function profilePerformance() {
  if (!import.meta.env.DEV || document.hidden || timingSink) {
    throw new Error('Profiling requires a visible development window and a single active capture.')
  }
  const commands = new Map<string, { calls: number; slowestMs: number }>()
  const intervals: number[] = []
  let lastFrame: number | undefined
  let frameId = 0
  let interrupted = false
  const onVisibility = () => { if (document.hidden) interrupted = true }
  function frame(now: number) {
    if (lastFrame !== undefined) intervals.push(now - lastFrame)
    lastFrame = now
    frameId = requestAnimationFrame(frame)
  }
  timingSink = (command, elapsed) => {
    const entry = commands.get(command) ?? { calls: 0, slowestMs: 0 }
    entry.calls++
    entry.slowestMs = Math.max(entry.slowestMs, Math.round(elapsed))
    commands.set(command, entry)
  }
  try {
    document.addEventListener('visibilitychange', onVisibility)
    frameId = requestAnimationFrame(frame)
    await new Promise(resolve => setTimeout(resolve, 10000))
    if (interrupted || !intervals.length) throw new Error('Keep the app visible throughout the capture.')
    const total = intervals.reduce((a, b) => a + b, 0)
    intervals.sort((a, b) => a - b)
    const report = {
      averageFps: Math.round(1000 * intervals.length / total),
      medianFrameMs: Math.round(intervals[Math.floor(intervals.length / 2)] * 10) / 10,
      longestFrameMs: Math.round(intervals[intervals.length - 1]),
      p95FrameMs: Math.round(intervals[Math.ceil(intervals.length * 0.95) - 1]),
      framesOver50Ms: intervals.filter(ms => ms > 50).length,
      commands: Object.fromEntries(commands),
    }
    console.info('[Sanctum] Performance report', JSON.stringify(report))
    return report
  } finally {
    timingSink = undefined
    cancelAnimationFrame(frameId)
    document.removeEventListener('visibilitychange', onVisibility)
  }
}

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const sink = import.meta.env.DEV ? timingSink : undefined
  const start = sink ? performance.now() : 0
  try {
    return await tauriInvoke<T>(command, args)
  } catch (e) {
    if (SESSION_LOST.has(errorKind(e) ?? '')) onSessionLost?.()
    // Rethrown untouched: the caller still gets to report what it was doing.
    throw e
  } finally {
    if (sink && sink === timingSink) sink(command, performance.now() - start)
  }
}
