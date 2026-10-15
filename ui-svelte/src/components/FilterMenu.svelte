<!-- Sanctum — a privacy-first personal finance and crypto vault.
     Copyright (C) 2026  yfloress

     This program is free software: you can redistribute it and/or modify
     it under the terms of the GNU Affero General Public License as
     published by the Free Software Foundation, either version 3 of the
     License, or (at your option) any later version.

     This program is distributed in the hope that it will be useful,
     but WITHOUT ANY WARRANTY; without even the implied warranty of
     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
     GNU Affero General Public License for more details.

     You should have received a copy of the GNU Affero General Public License
     along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>. -->

<!-- A "Filters" button that opens a panel with the page's filter fields, so the
     toolbar keeps only the search box. The button shows how many are active. -->
<script lang="ts">
  import type { Snippet } from 'svelte'
  import { i18n } from '../lib/stores/i18n.svelte'

  interface Props {
    /** Active filters, not counting the search box. */
    count: number
    onclear: () => void
    children: Snippet
  }

  let { count, onclear, children }: Props = $props()

  let open = $state(false)
  let root: HTMLDivElement | undefined = $state()
  let trigger: HTMLButtonElement | undefined = $state()
  let panelStyle = $state('')

  /**
   * Places the panel under the button, right-aligned and kept on screen.
   *
   * Fixed rather than absolute, like the modals: inside the scrolling content
   * WebKitGTK left the panel with nothing behind it to blur, so the glass
   * showed as plain transparency.
   */
  function place() {
    if (!trigger) return
    const rect = trigger.getBoundingClientRect()
    const width = Math.min(320, window.innerWidth - 16)
    const left = Math.max(8, Math.min(rect.right - width, window.innerWidth - width - 8))
    panelStyle = `top: ${rect.bottom + 6}px; left: ${left}px; width: ${width}px;`
  }

  function toggle() {
    open = !open
    if (open) place()
  }

  // Close on a click outside the menu or on Escape, while it is open.
  $effect(() => {
    if (!open) return
    const onPointer = (e: PointerEvent) => {
      if (root && !root.contains(e.target as Node)) open = false
    }
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') open = false
    }
    document.addEventListener('pointerdown', onPointer)
    document.addEventListener('keydown', onKey)
    // The page scrolls inside its own container, so listen in the capture phase.
    window.addEventListener('scroll', place, true)
    window.addEventListener('resize', place)
    return () => {
      document.removeEventListener('pointerdown', onPointer)
      document.removeEventListener('keydown', onKey)
      window.removeEventListener('scroll', place, true)
      window.removeEventListener('resize', place)
    }
  })
</script>

<div class="filter-menu" bind:this={root}>
  <button class="glass-btn filter-trigger" class:active={count > 0} aria-expanded={open} bind:this={trigger} onclick={toggle}>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M4 6h16M7 12h10M10 18h4"/></svg>
    {i18n.t('filters-button', 'Filters')}
    {#if count > 0}<span class="filter-count">{count}</span>{/if}
  </button>

  {#if open}
    <div class="filter-panel" role="dialog" aria-label={i18n.t('filters-button', 'Filters')} style={panelStyle}>
      {@render children()}
      <div class="filter-panel-actions">
        {#if count > 0}
          <button class="secondary-btn" onclick={onclear}>{i18n.t('filters-clear', 'Clear filters')}</button>
        {/if}
        <button class="primary-btn" onclick={() => (open = false)}>{i18n.t('filters-done', 'Done')}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .filter-menu { position: relative; flex-shrink: 0; }
  .filter-trigger { display: flex; align-items: center; gap: 6px; height: 100%; }
  .filter-trigger svg { width: 14px; height: 14px; }
  .filter-trigger.active { border-color: var(--accent-border); }
  .filter-count {
    min-width: 18px; height: 18px; padding: 0 5px; border-radius: 9px;
    display: inline-grid; place-items: center;
    font-size: 0.7rem; font-weight: 600;
    background: var(--accent-solid); color: var(--text-on-accent);
  }
  .filter-panel {
    position: fixed; z-index: 50;
    display: flex; flex-direction: column; gap: 12px;
    padding: 14px;
    /* Nearly opaque: the blur below does not render here in WebKitGTK, and
       without it a translucent panel lets the list behind compete with the
       fields. */
    background: color-mix(in srgb, var(--card-bg-solid) 94%, transparent);
    backdrop-filter: blur(20px) saturate(140%);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--glass-shadow-lg);
  }
  .filter-panel-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 2px; }
</style>
