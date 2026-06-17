<!-- Sanctum — a privacy-first personal finance and crypto vault.
     Copyright (C) 2026  Kyronix

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
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as vaultApi from '../lib/api/vault'

  let menuOpen = $state(false)

  function goSettings() {
    menuOpen = false
    app.navigate('settings')
  }

  async function handleLock() {
    menuOpen = false
    await vaultApi.lockVault()
    app.logout()
  }
</script>

<header class="mtopbar">
  <button
    class="mtb-btn"
    onclick={() => (menuOpen = !menuOpen)}
    aria-label={i18n.t('nav-menu', 'Menu')}
    aria-expanded={menuOpen}
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
      <path d="M3 12h18M3 6h18M3 18h18" />
    </svg>
  </button>

  <img src="/assets/logo/sanctum_logo.svg" alt="Sanctum" class="mtb-logo" />

  <button
    class="mtb-btn"
    class:active={app.hideBalances}
    onclick={() => app.toggleHideBalances()}
    aria-pressed={app.hideBalances}
    aria-label={app.hideBalances
      ? i18n.t('nav-show-balances', 'Show balances')
      : i18n.t('nav-hide-balances', 'Hide balances')}
  >
    {#if app.hideBalances}
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
        <path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49" />
        <path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" />
        <path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143" />
        <path d="m2 2 20 20" />
      </svg>
    {:else}
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
        <path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0" />
        <circle cx="12" cy="12" r="3" />
      </svg>
    {/if}
  </button>

  {#if menuOpen}
    <button class="mtb-scrim" aria-label={i18n.t('nav-close', 'Close')} onclick={() => (menuOpen = false)}></button>
    <div class="mtb-menu" role="menu">
      <button class="mtb-menu-item" role="menuitem" onclick={goSettings}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        <span>{i18n.t('nav-settings', 'Settings')}</span>
      </button>
      <button class="mtb-menu-item lock" role="menuitem" onclick={handleLock}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
        <span>{i18n.t('nav-lock', 'Lock')}</span>
      </button>
    </div>
  {/if}
</header>

<style>
  /* Hidden on desktop; the sidebar handles navigation there. */
  .mtopbar {
    display: none;
  }

  @media (max-width: 720px) {
    .mtopbar {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      height: calc(52px + env(safe-area-inset-top, 0px));
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 8px;
      padding-top: env(safe-area-inset-top, 0px);
      background: rgba(14, 12, 20, 0.9);
      backdrop-filter: blur(18px) saturate(1.3);
      -webkit-backdrop-filter: blur(18px) saturate(1.3);
      border-bottom: 1px solid rgba(168, 85, 247, 0.12);
      z-index: 100;
    }
  }

  :global(.light-mode) .mtopbar {
    background: rgba(250, 248, 255, 0.92);
    border-bottom: 1px solid rgba(139, 92, 246, 0.12);
  }

  .mtb-btn {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    border-radius: 10px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }
  .mtb-btn:hover {
    background: var(--glass-hover);
    color: var(--text-primary);
  }
  .mtb-btn.active {
    color: #c084fc;
  }
  .mtb-btn svg {
    width: 22px;
    height: 22px;
  }

  .mtb-logo {
    width: 24px;
    height: 24px;
    filter: drop-shadow(0 0 8px rgba(168, 85, 247, 0.3));
  }

  .mtb-scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    border: none;
    z-index: 101;
    cursor: default;
  }

  .mtb-menu {
    position: fixed;
    top: calc(52px + env(safe-area-inset-top, 0px));
    left: 8px;
    min-width: 184px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: var(--modal-bg);
    border: 1px solid var(--modal-border);
    border-radius: var(--radius-md);
    box-shadow: var(--glass-shadow-lg);
    backdrop-filter: var(--glass-blur-heavy);
    -webkit-backdrop-filter: var(--glass-blur-heavy);
    z-index: 102;
    animation: menuIn 0.16s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes menuIn {
    from { opacity: 0; transform: translateY(-6px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .mtb-menu-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border: none;
    border-radius: 8px;
    background: none;
    color: var(--text-primary);
    font-size: 0.9rem;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }
  .mtb-menu-item:hover {
    background: var(--glass-hover);
  }
  .mtb-menu-item svg {
    width: 18px;
    height: 18px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }
  .mtb-menu-item.lock:hover {
    background: var(--danger-bg);
    color: var(--danger);
  }
  .mtb-menu-item.lock:hover svg {
    color: var(--danger);
  }
</style>
