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

// Global keyboard shortcuts. Inert while typing or while a dialog is open, so
// they never fight a form. Page-specific ones are registered by the page.

import { app, type Page } from './stores/app.svelte'
import { lockNow } from './stores/session.svelte'
import { isDialogOpen } from './actions/dialog'
import * as settingsApi from './api/settings'

export interface PageActions {
  newEntry?: () => void
  focusSearch?: () => void
}

const PAGES: Page[] = ['dashboard', 'finances', 'crypto', 'settings']

let pageActions: PageActions = {}

/** Called by the active page; run the returned cleanup on unmount. */
export function setPageActions(actions: PageActions) {
  pageActions = actions
  return () => {
    if (pageActions === actions) pageActions = {}
  }
}

function isTyping(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null
  if (!el) return false
  return (
    el.tagName === 'INPUT' ||
    el.tagName === 'TEXTAREA' ||
    el.tagName === 'SELECT' ||
    el.isContentEditable
  )
}

async function toggleSidebar() {
  const next = !app.sidebarCollapsed
  if (app.settings) app.settings.sidebar_collapsed = next
  await settingsApi.setSidebarCollapsed(next)
}

function run(event: KeyboardEvent, action: (() => void) | undefined) {
  if (!action) return
  event.preventDefault()
  action()
}

function onKeydown(event: KeyboardEvent) {
  if (!app.isLoggedIn || event.altKey || isDialogOpen() || isTyping(event.target)) return

  if (event.ctrlKey || event.metaKey) {
    if (event.shiftKey) return
    switch (event.key.toLowerCase()) {
      case 'l':
        event.preventDefault()
        lockNow()
        return
      case 'b':
        event.preventDefault()
        void toggleSidebar()
        return
      case 'n':
        run(event, pageActions.newEntry)
        return
      case 'k':
        run(event, pageActions.focusSearch)
        return
    }
    return
  }

  // Checked before the Shift guard: on some layouts "/" is Shift+7.
  if (event.key === '/') {
    run(event, pageActions.focusSearch)
    return
  }

  if (event.shiftKey) return

  const digit = Number(event.key)
  if (Number.isInteger(digit) && digit >= 1 && digit <= PAGES.length) {
    event.preventDefault()
    app.navigate(PAGES[digit - 1])
  }
}

export function startShortcuts() {
  window.addEventListener('keydown', onKeydown)
  return () => window.removeEventListener('keydown', onKeydown)
}
