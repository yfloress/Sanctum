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
  import { i18n } from '../lib/stores/i18n.svelte'
  import { dialog } from '../lib/actions/dialog'

  interface Props {
    show: boolean
    onclose: () => void
  }

  let { show = $bindable(false), onclose }: Props = $props()

  // Label only. Both modifiers are already accepted, this just names the one
  // printed on the user's keyboard.
  const mod = navigator.userAgent.includes('Mac') ? 'Cmd' : 'Ctrl'

  let groups = $derived([
    {
      title: i18n.t('shortcuts-group-navigation', 'Navigation'),
      rows: [
        { keys: ['1'], label: i18n.t('nav-dashboard', 'Dashboard') },
        { keys: ['2'], label: i18n.t('nav-finances', 'Finances') },
        { keys: ['3'], label: i18n.t('nav-crypto', 'Crypto') },
        { keys: ['4'], label: i18n.t('nav-settings', 'Settings') },
        { keys: [mod, 'B'], label: i18n.t('shortcuts-toggle-sidebar', 'Collapse or expand the sidebar') },
      ],
    },
    {
      title: i18n.t('shortcuts-group-actions', 'Actions'),
      rows: [
        { keys: [mod, 'N'], label: i18n.t('shortcuts-new-entry', 'New entry on the current page') },
        { keys: [mod, 'K'], label: i18n.t('shortcuts-search', 'Jump to the search box') },
        { keys: ['/'], label: i18n.t('shortcuts-search', 'Jump to the search box') },
        { keys: [mod, 'L'], label: i18n.t('shortcuts-lock', 'Lock the vault now') },
      ],
    },
    {
      title: i18n.t('shortcuts-group-dialogs', 'Dialogs'),
      rows: [
        { keys: ['Enter'], label: i18n.t('shortcuts-confirm', 'Confirm the open form') },
        { keys: ['Esc'], label: i18n.t('shortcuts-close', 'Close without saving') },
        { keys: ['?'], label: i18n.t('shortcuts-help', 'Show this list') },
      ],
    },
  ])

  function close() {
    show = false
    onclose()
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close}></div>
  <div class="modal-wrapper">
    <!-- Nothing here to focus, and no destructive default to arm. -->
    <div class="modal shortcuts-modal" use:dialog={{ onclose: close, autofocus: false }}>
      <h3>{i18n.t('shortcuts-title', 'Keyboard Shortcuts')}</h3>
      <div class="shortcuts-body">
        {#each groups as group}
          <section class="shortcuts-group">
            <h4>{group.title}</h4>
            {#each group.rows as row}
              <div class="shortcuts-row">
                <span class="shortcuts-keys">
                  {#each row.keys as key, index}
                    {#if index > 0}<span class="shortcuts-plus">+</span>{/if}
                    <kbd>{key}</kbd>
                  {/each}
                </span>
                <span class="shortcuts-label">{row.label}</span>
              </div>
            {/each}
          </section>
        {/each}
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('finances-close', 'Close')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .shortcuts-modal { width: 460px; max-width: 100%; }
  .shortcuts-body {
    display: flex; flex-direction: column; gap: 18px;
    position: relative; z-index: 10;
  }
  .shortcuts-group h4 {
    margin: 0 0 8px;
    font-size: 0.72rem; font-weight: 600; letter-spacing: 0.08em;
    text-transform: uppercase; color: var(--text-tertiary);
  }
  .shortcuts-row {
    display: flex; align-items: center; gap: 12px;
    padding: 5px 0; font-size: 0.85rem;
  }
  .shortcuts-keys {
    display: flex; align-items: center; gap: 3px;
    flex: 0 0 108px; justify-content: flex-end;
  }
  .shortcuts-plus { color: var(--text-tertiary); font-size: 0.7rem; }
  .shortcuts-label { color: var(--text-secondary); }
  kbd {
    display: inline-block; min-width: 22px; padding: 2px 6px;
    border: 1px solid var(--glass-border); border-radius: 4px;
    background: var(--glass-active); color: var(--text-primary);
    font-family: inherit; font-size: 0.75rem; text-align: center;
  }
</style>
