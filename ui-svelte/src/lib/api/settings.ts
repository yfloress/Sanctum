import { invoke } from '@tauri-apps/api/core'
import type { AppSettings, AppInfo } from '../types'

export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('load_settings')
}

export async function setDarkMode(value: boolean): Promise<void> {
  return invoke('set_dark_mode', { value })
}

export async function setAutoFetch(value: boolean): Promise<void> {
  return invoke('set_auto_fetch', { value })
}

export async function setProxyEnabled(value: boolean): Promise<void> {
  return invoke('set_proxy_enabled', { value })
}

export async function setProxyUrl(value: string): Promise<void> {
  return invoke('set_proxy_url', { value })
}

export async function setSessionTimeout(timeoutSecs: number): Promise<void> {
  return invoke('set_session_timeout', { timeoutSecs })
}

export async function setPreferredCurrency(value: string): Promise<void> {
  return invoke('set_preferred_currency', { value })
}

export async function setPreferredLanguage(value: string): Promise<void> {
  return invoke('set_preferred_language', { value })
}

export async function setSidebarCollapsed(value: boolean): Promise<void> {
  return invoke('set_sidebar_collapsed', { value })
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
