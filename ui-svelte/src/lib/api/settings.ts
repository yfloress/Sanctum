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

import { invoke } from '@tauri-apps/api/core'
import type { AppSettings, AppInfo } from '../types'

export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('load_settings')
}

export async function setDarkMode(enabled: boolean): Promise<void> {
  return invoke('set_dark_mode', { enabled })
}

export async function setAutoFetch(enabled: boolean): Promise<void> {
  return invoke('set_auto_fetch', { enabled })
}

export async function setProxyEnabled(enabled: boolean, current_url: string): Promise<void> {
  return invoke('set_proxy_enabled', { enabled, currentUrl: current_url })
}

export async function setProxyUrl(url: string): Promise<void> {
  return invoke('set_proxy_url', { url })
}

export async function setSessionTimeout(timeout_secs: number): Promise<void> {
  return invoke('set_session_timeout', { timeoutSecs: timeout_secs })
}

export async function setPreferredCurrency(currency: string): Promise<void> {
  return invoke('set_preferred_currency', { currency })
}

export async function setPreferredLanguage(language: string): Promise<void> {
  return invoke('set_preferred_language', { language })
}

export async function setSidebarCollapsed(collapsed: boolean): Promise<void> {
  return invoke('set_sidebar_collapsed', { collapsed })
}

export async function resetSettings(): Promise<void> {
  return invoke('reset_settings')
}

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>('get_app_info')
}

export async function getSessionRemaining(): Promise<number> {
  return invoke<number>('get_session_remaining')
}

export async function getTranslations(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('get_translations')
}

export async function setLoginWallpaper(path: string | null): Promise<void> {
  return invoke('set_login_wallpaper', { path })
}
