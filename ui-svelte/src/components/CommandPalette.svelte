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

<script lang="ts">
  import { app, type Page } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import { lockNow } from '../lib/stores/session.svelte'
  import { dialog } from '../lib/actions/dialog'
  import { currentPageActions, shortcutPages, toggleSidebar, toggleDarkMode } from '../lib/shortcuts'
  import { PAGE_COMMANDS } from '../lib/commands'

  interface Props {
    show: boolean
    onclose: () => void
    /** The cheat sheet lives in the shell, so the palette can only ask for it. */
    onopenhelp: () => void
  }

  let { show = $bindable(false), onclose, onopenhelp }: Props = $props()

  interface Entry {
    id: string
    label: string
    group: string
    /** Printed on the right. Already resolved to this keyboard's modifier. */
    keys?: string[]
    run: () => void
  }

  let query = $state('')
  let activeIndex = $state(0)
  let listEl = $state<HTMLDivElement | undefined>()
  // Rebuilt on every open rather than derived: what the current page offers is
  // plain module state, so nothing would tell a derived value it went stale.
  let commands = $state<Entry[]>([])

  const mod = navigator.userAgent.includes('Mac') ? 'Cmd' : 'Ctrl'

  const PAGE_LABELS: Record<Page, [string, string]> = {
    dashboard: ['nav-dashboard', 'Dashboard'],
    finances: ['nav-finances', 'Finances'],
    crypto: ['nav-crypto', 'Crypto'],
    settings: ['nav-settings', 'Settings'],
  }

  function pageLabel(page: Page): string {
    const [key, fallback] = PAGE_LABELS[page]
    return i18n.t(key, fallback)
  }

  function buildCommands(): Entry[] {
    const navigation = i18n.t('shortcuts-group-navigation', 'Navigation')
    const actions = i18n.t('shortcuts-group-actions', 'Actions')
    const page = currentPageActions()

    const list: Entry[] = shortcutPages().map((target, index) => ({
      id: `nav-${target}`,
      label: pageLabel(target),
      group: navigation,
      keys: [String(index + 1)],
      run: () => app.navigate(target),
    }))

    // Every page's own commands, wherever we happen to be standing: the palette
    // navigates there and the page runs the body once it is on screen.
    list.push(
      ...PAGE_COMMANDS.map(spec => ({
        id: spec.id,
        label: i18n.t(spec.key, spec.fallback),
        group: pageLabel(spec.page),
        run: () => app.runPageCommand(spec.page, spec.id),
      })),
    )

    // These two are a capability of whatever is mounted, not a named command,
    // so they only appear where they mean something.
    if (page.newEntry) {
      list.push({
        id: 'new-entry',
        label: i18n.t('shortcuts-new-entry', 'New entry on the current page'),
        group: actions,
        keys: [mod, 'N'],
        run: page.newEntry,
      })
    }
    if (page.focusSearch) {
      list.push({
        id: 'focus-search',
        label: i18n.t('shortcuts-search', 'Jump to the search box'),
        group: actions,
        keys: [mod, 'K'],
        run: page.focusSearch,
      })
    }

    list.push(
      {
        id: 'toggle-sidebar',
        label: i18n.t('shortcuts-toggle-sidebar', 'Collapse or expand the sidebar'),
        group: actions,
        keys: [mod, 'B'],
        run: () => void toggleSidebar(),
      },
      {
        id: 'dark-mode',
        label: i18n.t('settings-dark-mode', 'Dark Mode'),
        group: actions,
        run: () => void toggleDarkMode(),
      },
      {
        id: 'hide-balances',
        label: app.hideBalances
          ? i18n.t('nav-show-balances', 'Show Balances')
          : i18n.t('nav-hide-balances', 'Hide Balances'),
        group: actions,
        run: () => app.toggleHideBalances(),
      },
      {
        id: 'help',
        label: i18n.t('shortcuts-title', 'Keyboard Shortcuts'),
        group: actions,
        keys: ['?'],
        run: onopenhelp,
      },
      {
        id: 'lock',
        label: i18n.t('shortcuts-lock', 'Lock the vault now'),
        group: actions,
        keys: [mod, 'L'],
        run: lockNow,
      },
    )

    return list
  }

  /**
   * Folds case and strips accents, so "credito" finds "Crédito". The app runs
   * in Spanish as readily as English and nobody reaches for the accent key
   * while typing into a search box.
   */
  function normalize(value: string): string {
    return value.toLowerCase().normalize('NFD').replace(/\p{Diacritic}/gu, '')
  }

  let matches = $derived.by(() => {
    const terms = normalize(query).split(/\s+/).filter(Boolean)
    if (terms.length === 0) return commands
    return commands.filter(cmd => {
      const haystack = normalize(`${cmd.label} ${cmd.group}`)
      return terms.every(term => haystack.includes(term))
    })
  })

  // A filtered list can be shorter than where the cursor was.
  $effect(() => {
    if (activeIndex >= matches.length) activeIndex = Math.max(0, matches.length - 1)
  })

  $effect(() => {
    if (show) {
      query = ''
      activeIndex = 0
      commands = buildCommands()
    }
  })

  $effect(() => {
    void activeIndex
    listEl?.querySelector('.palette-item.active')?.scrollIntoView({ block: 'nearest' })
  })

  function move(delta: number) {
    if (matches.length === 0) return
    activeIndex = (activeIndex + delta + matches.length) % matches.length
  }

  function runCommand(cmd: Entry) {
    close()
    // Deferred a frame: closing hands focus back to whatever held it before the
    // palette opened, and that has to land before the command opens anything of
    // its own, or the restore steals focus from the dialog it just opened.
    requestAnimationFrame(() => cmd.run())
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      move(1)
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      move(-1)
    } else if (event.key === 'Enter' && !event.isComposing) {
      event.preventDefault()
      const cmd = matches[activeIndex]
      if (cmd) runCommand(cmd)
    }
  }

  function close() {
    show = false
    onclose()
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close}></div>
  <div class="modal-wrapper palette-wrapper">
    <div class="modal palette" use:dialog={{ onclose: close }}>
      <input
        type="text"
        class="palette-input"
        bind:value={query}
        onkeydown={onKeydown}
        placeholder={i18n.t('palette-placeholder', 'Type a command...')}
        aria-label={i18n.t('palette-title', 'Command palette')}
      />
      <div class="palette-list" bind:this={listEl}>
        {#each matches as cmd, index (cmd.id)}
          <button
            class="palette-item"
            class:active={index === activeIndex}
            onclick={() => runCommand(cmd)}
            onmousemove={() => activeIndex = index}
          >
            <span class="palette-label">{cmd.label}</span>
            <span class="palette-group">{cmd.group}</span>
            {#if cmd.keys}
              <span class="palette-keys">
                {#each cmd.keys as key, position}
                  {#if position > 0}<span class="palette-plus">+</span>{/if}
                  <kbd>{key}</kbd>
                {/each}
              </span>
            {/if}
          </button>
        {/each}
        {#if matches.length === 0}
          <p class="palette-empty">{i18n.t('palette-no-results', 'No matching command')}</p>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  /* Sits high rather than centred: the list grows downwards and a centred box
     would jump on every keystroke. */
  .palette-wrapper { align-items: flex-start; padding-top: 12vh; }
  .palette { width: 520px; max-width: 100%; padding: 0; overflow: hidden; }

  .palette-input {
    width: 100%;
    padding: 16px 18px;
    border: none;
    border-bottom: 1px solid var(--glass-border);
    background: none;
    color: var(--text-primary);
    font-family: inherit;
    font-size: 0.95rem;
    box-sizing: border-box;
    position: relative;
    z-index: 10;
  }
  .palette-input:focus { outline: none; }

  .palette-list {
    max-height: 46vh;
    overflow-y: auto;
    padding: 6px;
    position: relative;
    z-index: 10;
  }
  .palette-item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 9px 12px;
    border: none;
    border-radius: var(--radius-sm);
    background: none;
    color: inherit;
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    font-size: 0.85rem;
  }
  .palette-item.active { background: var(--glass-active); }
  .palette-label { flex: 1; color: var(--text-primary); }
  .palette-group {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    white-space: nowrap;
  }
  .palette-keys {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-shrink: 0;
  }
  .palette-plus { color: var(--text-tertiary); font-size: 0.65rem; }
  .palette-keys kbd {
    display: inline-block;
    min-width: 18px;
    padding: 1px 5px;
    border: 1px solid var(--glass-border);
    border-radius: 4px;
    background: var(--glass-active);
    color: var(--text-tertiary);
    font-family: inherit;
    font-size: 0.68rem;
    text-align: center;
  }
  .palette-empty {
    margin: 0;
    padding: 20px;
    text-align: center;
    color: var(--text-tertiary);
    font-size: 0.85rem;
  }
</style>
