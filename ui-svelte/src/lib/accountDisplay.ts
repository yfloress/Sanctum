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

import { i18n } from './stores/i18n.svelte'

export interface AccountIcon {
  value: string
  src: string
  generic: boolean
}

export const ACCOUNT_ICONS: AccountIcon[] = [
  ...['banco-chile', 'banco-estado', 'bank-of-america', 'bci', 'citibank', 'jpmorgan', 'mercado_pago', 'santander', 'wf']
    .map(n => ({ value: `${n}.svg`, src: `/src/assets/bank-icons/${n}.svg`, generic: false })),
  ...['landmark', 'wallet', 'credit-card', 'piggy-bank', 'briefcase', 'coins', 'banknote', 'building-2']
    .map(n => ({ value: `/src/assets/icons/${n}.svg`, src: `/src/assets/icons/${n}.svg`, generic: true })),
]

export function getDefaultIconPath(accountType: string): string {
  const iconMap: Record<string, string> = {
    savings: 'piggy-bank',
    credit: 'credit-card',
    credit_card: 'credit-card',
    cash: 'wallet',
    bank: 'landmark',
    other: 'coins',
  }
  const icon = iconMap[accountType.toLowerCase()] || 'landmark'
  return `/src/assets/icons/${icon}.svg`
}

export function isGenericIcon(iconPath: string | null): boolean {
  if (!iconPath) return true
  return iconPath.startsWith('/src/assets/icons/')
}

export function getAccountDisplayIcon(acc: { account_type: string; account_type_key?: string; icon_path: string | null }): string {
  if (acc.icon_path) {
    if (acc.icon_path.startsWith('/') || acc.icon_path.startsWith('http')) return acc.icon_path
    return `/src/assets/bank-icons/${acc.icon_path}`
  }
  return getDefaultIconPath(acc.account_type_key ?? acc.account_type)
}

export function normalizeAccountTypeKey(value: string | null | undefined): string {
  if (!value) return ''
  const lower = value.toLowerCase().trim()
  return lower === 'credit_card' ? 'credit' : lower
}

export function accountTypeLabel(rawType: string): string {
  const key = normalizeAccountTypeKey(rawType)
  if (!key) return rawType
  return i18n.t(`finances-account-type-${key}`, rawType)
}
