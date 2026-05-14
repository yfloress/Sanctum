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

import type { AppSettings } from '../types'

export type Page = 'dashboard' | 'finances' | 'habits' | 'crypto' | 'settings'

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
