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

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args)
  } catch (e) {
    if (SESSION_LOST.has(errorKind(e) ?? '')) onSessionLost?.()
    // Rethrown untouched: the caller still gets to report what it was doing.
    throw e
  }
}
