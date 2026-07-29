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

// Escape, autofocus and Tab trapping for modals and detail panels. Applied to
// the dialog root, not the backdrop: a backdrop `<div>` is not focusable, so it
// never receives `keydown`. Dialogs stack; only the topmost one reacts.

export interface DialogOptions {
  onclose: () => void
  /** Focus the first field on open. Pass `false` for read-only panels. */
  autofocus?: boolean
}

interface Entry {
  node: HTMLElement
  onclose: () => void
}

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

/** Autofocus targets. Buttons excluded: never arm a destructive default. */
const FIELDS =
  'input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled])'

const stack: Entry[] = []

/** Rendered, focusable descendants in DOM order. */
function focusables(node: HTMLElement): HTMLElement[] {
  return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
    el => el.getClientRects().length > 0
  )
}

function trapTab(event: KeyboardEvent, node: HTMLElement) {
  const items = focusables(node)
  if (items.length === 0) {
    event.preventDefault()
    return
  }

  const first = items[0]
  const last = items[items.length - 1]
  const active = document.activeElement as HTMLElement | null

  // Focus escaped the dialog (or never entered it) — pull it back in.
  if (!active || !node.contains(active)) {
    event.preventDefault()
    ;(event.shiftKey ? last : first).focus()
    return
  }

  if (event.shiftKey && active === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && active === last) {
    event.preventDefault()
    first.focus()
  }
}

function onKeydown(event: KeyboardEvent) {
  const top = stack[stack.length - 1]
  if (!top) return

  if (event.key === 'Escape') {
    top.onclose()
  } else if (event.key === 'Tab') {
    trapTab(event, top.node)
  }
}

export function dialog(node: HTMLElement, options: DialogOptions) {
  const entry: Entry = { node, onclose: options.onclose }
  const previouslyFocused = document.activeElement as HTMLElement | null

  if (stack.length === 0) window.addEventListener('keydown', onKeydown)
  stack.push(entry)

  if (options.autofocus ?? true) {
    const field = node.querySelector<HTMLElement>(FIELDS)
    if (field) {
      field.focus()
    } else {
      // No field: focus the dialog so Tab starts inside it.
      if (!node.hasAttribute('tabindex')) node.setAttribute('tabindex', '-1')
      node.focus()
    }
  }

  return {
    update(next: DialogOptions) {
      entry.onclose = next.onclose
    },
    destroy() {
      const index = stack.indexOf(entry)
      if (index !== -1) stack.splice(index, 1)
      if (stack.length === 0) window.removeEventListener('keydown', onKeydown)
      previouslyFocused?.focus?.()
    },
  }
}
