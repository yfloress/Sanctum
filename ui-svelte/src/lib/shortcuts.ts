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
  /**
   * Bodies for the page's own palette commands, keyed by the id declared in
   * `lib/commands.ts`. Only exist while the page is mounted, which is why the
   * names live there and not here.
   */
  handlers?: Record<string, () => void>
}

export interface GlobalActions {
  /** Show the cheat sheet. Without it the shortcuts below are undiscoverable. */
  openHelp?: () => void
  /** Show the command palette, the searchable form of everything here. */
  openPalette?: () => void
}

const PAGES: Page[] = ['dashboard', 'finances', 'crypto', 'settings']

let pageActions: PageActions = {}
let globalActions: GlobalActions = {}

/** Called by the active page; run the returned cleanup on unmount. */
export function setPageActions(actions: PageActions) {
  pageActions = actions
  return () => {
    if (pageActions === actions) pageActions = {}
  }
}

/** What the mounted page currently offers, for the command palette to list. */
export function currentPageActions(): PageActions {
  return pageActions
}

/**
 * Runs the palette command left waiting for this page, if the id is one of
 * its own. Call it from an `$effect` so the read of `app.pendingCommand` is
 * tracked; ids belonging to another page are left alone for it to claim.
 */
export function consumePendingCommand(handlers: Record<string, () => void>) {
  const id = app.pendingCommand
  if (!id) return
  const handler = handlers[id]
  if (!handler) return
  app.pendingCommand = null
  handler()
}

/** The pages the number keys walk through, in that order. */
export function shortcutPages(): Page[] {
  return PAGES
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

export async function toggleSidebar() {
  const next = !app.sidebarCollapsed
  if (app.settings) app.settings.sidebar_collapsed = next
  await settingsApi.setSidebarCollapsed(next)
}

export async function toggleDarkMode() {
  if (!app.settings) return
  const next = !app.settings.dark_mode
  app.settings.dark_mode = next
  await settingsApi.setDarkMode(next)
}

function run(event: KeyboardEvent, action: (() => void) | undefined) {
  if (!action) return
  event.preventDefault()
  action()
}

function onKeydown(event: KeyboardEvent) {
  if (!app.isLoggedIn || event.altKey || isDialogOpen()) return

  // Modifier combos fire from anywhere, a field included: locking the vault or
  // reaching for the palette should not mean leaving the form first. Bare keys
  // below belong to whatever is being typed into.
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
      case 'p':
        run(event, globalActions.openPalette)
        return
    }
    return
  }

  if (isTyping(event.target)) return

  // Both checked before the Shift guard: "?" is Shift+something everywhere, and
  // on some layouts "/" is Shift+7.
  if (event.key === '?') {
    run(event, globalActions.openHelp)
    return
  }

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

export function startShortcuts(actions: GlobalActions = {}) {
  globalActions = actions
  window.addEventListener('keydown', onKeydown)
  return () => {
    globalActions = {}
    window.removeEventListener('keydown', onKeydown)
  }
}
