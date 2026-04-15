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

  async load() {
    try {
      this.strings = await settingsApi.getTranslations()
      this.loaded = true
    } catch (e) {
      console.error('Failed to load translations:', e)
    }
  }

  /** Simple key lookup with optional fallback. */
  t(key: string, fallback?: string): string {
    return this.strings[key] ?? fallback ?? key
  }

  /** Parameterized translation: replaces {$var} placeholders with values. */
  tArgs(key: string, args: Record<string, string | number>, fallback?: string): string {
    let text = this.strings[key] ?? fallback ?? key
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
