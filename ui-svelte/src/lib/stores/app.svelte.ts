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

import type { AppSettings } from '../types'

export type Page = 'dashboard' | 'finances' | 'crypto' | 'settings'

export type BackgroundFx = 'dots' | 'stars' | 'aurora' | 'diamonds' | 'dragon'

const HIDE_BALANCES_KEY = 'sanctum:hideBalances'
const BACKGROUND_FX_KEY = 'sanctum:backgroundFx'

function readHideBalances(): boolean {
  try {
    return localStorage.getItem(HIDE_BALANCES_KEY) === '1'
  } catch {
    return false
  }
}

function readBackgroundFx(): BackgroundFx {
  try {
    const v = localStorage.getItem(BACKGROUND_FX_KEY)
    if (v === 'dots' || v === 'stars' || v === 'aurora' || v === 'diamonds' || v === 'dragon')
      return v
  } catch {
    // localStorage unavailable — fall through to the default
  }
  return 'dots'
}

interface ToastAction {
  label: string
  handler: () => void | Promise<void>
}

interface Toast {
  message: string
  isError: boolean
  action: ToastAction | null
}

class AppState {
  isLoggedIn = $state(false)
  activePage = $state<Page>('dashboard')
  isLoading = $state(false)
  settings = $state<AppSettings | null>(null)
  toast = $state<Toast | null>(null)
  hideBalances = $state(readHideBalances())
  backgroundFx = $state<BackgroundFx>(readBackgroundFx())

  private toastTimeout: ReturnType<typeof setTimeout> | null = null

  get darkMode(): boolean {
    return this.settings?.dark_mode ?? true
  }

  get sidebarCollapsed(): boolean {
    return this.settings?.sidebar_collapsed ?? false
  }

  login() {
    this.isLoggedIn = true
  }

  logout() {
    this.isLoggedIn = false
    this.activePage = 'dashboard'
  }

  navigate(page: Page) {
    this.activePage = page
  }

  toggleHideBalances() {
    this.hideBalances = !this.hideBalances
    try {
      localStorage.setItem(HIDE_BALANCES_KEY, this.hideBalances ? '1' : '0')
    } catch {
      // localStorage unavailable — keep the in-memory toggle working anyway
    }
  }

  setBackgroundFx(fx: BackgroundFx) {
    this.backgroundFx = fx
    try {
      localStorage.setItem(BACKGROUND_FX_KEY, fx)
    } catch {
      // localStorage unavailable — keep the in-memory choice working anyway
    }
  }

  showToast(message: string, isError = false, durationMs = 3000, action: ToastAction | null = null) {
    if (this.toastTimeout) clearTimeout(this.toastTimeout)
    this.toast = { message, isError, action }
    this.toastTimeout = setTimeout(() => {
      this.toast = null
      this.toastTimeout = null
    }, durationMs)
  }

  async runToastAction() {
    const action = this.toast?.action
    if (!action) return
    this.dismissToast()
    await action.handler()
  }

  dismissToast() {
    if (this.toastTimeout) clearTimeout(this.toastTimeout)
    this.toast = null
    this.toastTimeout = null
  }
}

export const app = new AppState()
