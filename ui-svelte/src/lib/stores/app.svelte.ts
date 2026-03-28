import type { AppSettings } from '../types'

export type Page = 'dashboard' | 'finances' | 'habits' | 'crypto' | 'settings'

interface Toast {
  message: string
  isError: boolean
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

  showToast(message: string, isError = false, durationMs = 3000) {
    if (this.toastTimeout) clearTimeout(this.toastTimeout)
    this.toast = { message, isError }
    this.toastTimeout = setTimeout(() => {
      this.toast = null
      this.toastTimeout = null
    }, durationMs)
  }

  dismissToast() {
    if (this.toastTimeout) clearTimeout(this.toastTimeout)
    this.toast = null
    this.toastTimeout = null
  }
}

export const app = new AppState()
