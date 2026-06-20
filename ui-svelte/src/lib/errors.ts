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

// Typed errors crossing the Tauri boundary.
//
// Every command now rejects with an `AppError` payload `{ kind, message }`
// (see `src/error.rs`). `invoke` rejects with that object, so `String(e)`
// would render "[object Object]" — use `errorMessage(e)` for display and
// `errorKind(e)` to branch on the stable kind (e.g. `'session_expired'`).
// Both helpers fall back gracefully for non-AppError rejections (plain
// strings, JS Errors, etc.).

export type AppErrorKind =
  | 'validation'
  | 'not_found'
  | 'conflict'
  | 'no_vault_open'
  | 'session_expired'
  | 'rate_limited'
  | 'network'
  | 'parse'
  | 'unsupported_format'
  | 'file_too_large'
  | 'config'
  | 'internal'

export interface AppErrorShape {
  kind: AppErrorKind | string
  message: string
  // Reserved for the DTO/CQRS refactor (critique #4); optional today.
  field?: string
}

function isAppError(e: unknown): e is AppErrorShape {
  return (
    typeof e === 'object' &&
    e !== null &&
    'kind' in e &&
    'message' in e &&
    typeof (e as Record<string, unknown>).message === 'string'
  )
}

/** Human-readable message for any rejection, safe to show to the user. */
export function errorMessage(e: unknown): string {
  return isAppError(e) ? e.message : String(e)
}

/** Stable machine-readable kind, or `null` when not an AppError. */
export function errorKind(e: unknown): string | null {
  return isAppError(e) ? e.kind : null
}
