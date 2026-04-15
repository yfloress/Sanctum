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

import * as settingsApi from '../api/settings'

class I18nStore {
  strings = $state<Record<string, string>>({})
  loaded = $state(false)
  /** Bumped on every load — forces Svelte to re-render all t() calls. */
  version = $state(0)

  async load() {
    try {
      this.strings = await settingsApi.getTranslations()
      this.version++
      this.loaded = true
    } catch (e) {
      console.error('Failed to load translations:', e)
    }
  }

  /** Simple key lookup with optional fallback.
   *  The Rust backend returns key-as-value for missing keys,
   *  so we treat value === key as "not found". */
  t(key: string, fallback?: string): string {
    // Read version to create reactive dependency
    void this.version
    const val = this.strings[key]
    if (val != null && val !== key) return val
    return fallback ?? key
  }

  /** Parameterized translation: replaces {$var} placeholders with values. */
  tArgs(key: string, args: Record<string, string | number>, fallback?: string): string {
    void this.version
    const raw = this.strings[key]
    let text = (raw != null && raw !== key) ? raw : (fallback ?? key)
    for (const [k, v] of Object.entries(args)) {
      text = text.replaceAll(`{$${k}}`, String(v))
    }
    return text
  }

  /** Plural-aware translation: resolves {key}-one or {key}-other variant. */
  tPlural(key: string, count: number, args?: Record<string, string | number>): string {
    const variant = count === 1 ? 'one' : 'other'
    const fullKey = `${key}-${variant}`
    const merged = { count, ...args }
    return this.tArgs(fullKey, merged, this.tArgs(key, merged))
  }
}

export const i18n = new I18nStore()
