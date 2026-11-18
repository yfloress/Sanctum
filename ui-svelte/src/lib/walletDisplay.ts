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

// Icons for crypto wallets: exchange and wallet marks first, then generic
// glyphs. A stored icon_path is either one of these file names or a full path.

import type { AccountIcon } from './accountDisplay'

export const WALLET_ICONS: AccountIcon[] = [
  ...['binance', 'bisq', 'bitget', 'bitmart', 'bitso', 'buda', 'bybit', 'coinbase', 'feather', 'kraken', 'kucoin', 'mexc', 'notbank', 'okx', 'retoswap', 'uniswap']
    .map(n => ({ value: `${n}.svg`, src: `/assets/exchange-icons/${n}.svg`, generic: false })),
  ...['landmark', 'wallet', 'shield', 'shield-check', 'link', 'lock']
    .map(n => ({ value: `/assets/icons/${n}.svg`, src: `/assets/icons/${n}.svg`, generic: true })),
]

export function getDefaultWalletIconPath(category: string): string {
  const iconMap: Record<string, string> = {
    exchange: 'landmark',
    hardware: 'shield',
    software: 'wallet',
  }
  const icon = iconMap[category.toLowerCase()] || 'wallet'
  return `/assets/icons/${icon}.svg`
}

export function getWalletDisplayIcon(w: { category: string; icon_path: string | null }): string {
  if (w.icon_path) {
    if (w.icon_path.startsWith('/') || w.icon_path.startsWith('http')) return w.icon_path
    return `/assets/exchange-icons/${w.icon_path}`
  }
  return getDefaultWalletIconPath(w.category)
}

export function isGenericWalletIcon(iconPath: string | null): boolean {
  if (!iconPath) return true
  return iconPath.startsWith('/assets/icons/')
}
