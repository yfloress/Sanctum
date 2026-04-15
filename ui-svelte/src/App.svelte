<!-- Sanctum — a privacy-first personal finance, crypto, and habits vault.
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
  import { app } from './lib/stores/app.svelte'
  import { i18n } from './lib/stores/i18n.svelte'
  import { startSessionMonitor } from './lib/stores/session.svelte'
  import Sidebar from './components/Sidebar.svelte'
  import Toast from './components/Toast.svelte'
  import LoginPage from './pages/LoginPage.svelte'
  import DashboardPage from './pages/DashboardPage.svelte'
  import FinancesPage from './pages/FinancesPage.svelte'
  import HabitsPage from './pages/HabitsPage.svelte'
  import CryptoPage from './pages/CryptoPage.svelte'
  import SettingsPage from './pages/SettingsPage.svelte'

  $effect(() => {
    if (app.isLoggedIn) {
      i18n.load()
      return startSessionMonitor()
    }
  })

  $effect(() => {
    document.documentElement.classList.toggle('light-mode', !app.darkMode)
  })
</script>

{#if !app.isLoggedIn}
  <LoginPage />
{:else}
  <div class="shell">
    <Sidebar />
    <main class="content">
      {#if app.activePage === 'dashboard'}
        <DashboardPage />
      {:else if app.activePage === 'finances'}
        <FinancesPage />
      {:else if app.activePage === 'habits'}
        <HabitsPage />
      {:else if app.activePage === 'crypto'}
        <CryptoPage />
      {:else if app.activePage === 'settings'}
        <SettingsPage />
      {/if}
    </main>
  </div>
{/if}

<Toast />

<style>
  :global(:root) {
    --bg-base: #09090f;
    --bg-gradient: linear-gradient(135deg, #09090f 0%, #0e0b14 40%, #12101a 100%);
    --glass: rgba(255, 255, 255, 0.035);
    --glass-hover: rgba(255, 255, 255, 0.06);
    --glass-active: rgba(255, 255, 255, 0.08);
    --glass-elevated: rgba(255, 255, 255, 0.05);
    --glass-border: rgba(255, 255, 255, 0.08);
    --glass-border-hover: rgba(255, 255, 255, 0.14);
    --glass-blur: blur(16px);
    --glass-blur-heavy: blur(24px);
    --glass-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
    --glass-shadow-lg: 0 8px 40px rgba(0, 0, 0, 0.45);
    --glass-glow: 0 0 0 1px rgba(255, 255, 255, 0.04) inset;
    --accent: #a855f7;
    --accent-glow: rgba(168, 85, 247, 0.15);
    --accent-hover: #c084fc;
    --success: #4ade80;
    --danger: #f87171;
    --warning: #fbbf24;
    --text-primary: #e8eaed;
    --text-secondary: #9aa0a6;
    --text-tertiary: #5f6368;
    --radius-sm: 8px;
    --radius-md: 12px;
    --radius-lg: 16px;
    --radius-xl: 20px;
    --scrollbar-thumb: rgba(255, 255, 255, 0.1);
    --scrollbar-thumb-hover: rgba(255, 255, 255, 0.18);
    --select-bg: rgba(0, 0, 0, 0.25);
    --select-bg-hover: rgba(0, 0, 0, 0.35);
    --option-bg: #1a1a1a;
    --sidebar-bg: linear-gradient(180deg, #111116 0%, #0d0d11 100%);
    --sidebar-border: rgba(255, 255, 255, 0.06);
    --nav-active-bg: rgba(255, 255, 255, 0.08);
    --nav-active-border: rgba(255, 255, 255, 0.1);
    --danger-bg: rgba(248, 113, 113, 0.08);
    --danger-border: rgba(248, 113, 113, 0.15);
    --modal-bg: linear-gradient(145deg, rgba(26, 26, 31, 0.9) 0%, rgba(20, 20, 24, 0.85) 50%, rgba(17, 17, 21, 0.8) 100%);
    --modal-border: rgba(255, 255, 255, 0.1);
    --modal-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    --accent-bg: rgba(168, 85, 247, 0.2);
    --accent-border: rgba(168, 85, 247, 0.3);
    --text-on-accent: #ffffff;
  }

  :global(.light-mode) {
    --bg-base: #f8f7fd;
    --bg-gradient: linear-gradient(135deg, #f8f7fd 0%, #f0edff 40%, #e8e3fc 100%);
    --glass: rgba(255, 255, 255, 0.65);
    --glass-hover: rgba(147, 51, 234, 0.06);
    --glass-active: rgba(147, 51, 234, 0.1);
    --glass-elevated: rgba(255, 255, 255, 0.8);
    --glass-border: rgba(147, 51, 234, 0.1);
    --glass-border-hover: rgba(147, 51, 234, 0.18);
    --glass-shadow: 0 4px 20px rgba(100, 60, 180, 0.06);
    --glass-shadow-lg: 0 8px 32px rgba(100, 60, 180, 0.1);
    --glass-glow: 0 0 0 1px rgba(147, 51, 234, 0.06) inset;
    --accent: #8b5cf6;
    --accent-glow: rgba(139, 92, 246, 0.15);
    --accent-hover: #7c3aed;
    --success: #16a34a;
    --danger: #dc2626;
    --warning: #d97706;
    --text-primary: #1e1b2e;
    --text-secondary: #4c4665;
    --text-tertiary: #8882a0;
    --scrollbar-thumb: rgba(139, 92, 246, 0.12);
    --scrollbar-thumb-hover: rgba(139, 92, 246, 0.22);
    --select-bg: rgba(255, 255, 255, 0.7);
    --select-bg-hover: rgba(255, 255, 255, 0.9);
    --option-bg: #ffffff;
    --sidebar-bg: linear-gradient(180deg, #f5f3ff 0%, #ede9fe 100%);
    --sidebar-border: rgba(139, 92, 246, 0.1);
    --nav-active-bg: rgba(139, 92, 246, 0.1);
    --nav-active-border: rgba(139, 92, 246, 0.15);
    --danger-bg: rgba(220, 38, 38, 0.08);
    --danger-border: rgba(220, 38, 38, 0.15);
    --modal-bg: linear-gradient(145deg, rgba(255, 255, 255, 0.97) 0%, rgba(250, 248, 255, 0.95) 50%, rgba(245, 243, 255, 0.92) 100%);
    --modal-border: rgba(139, 92, 246, 0.1);
    --modal-shadow: 0 8px 32px rgba(100, 60, 180, 0.1);
    --accent-bg: rgba(139, 92, 246, 0.1);
    --accent-border: rgba(139, 92, 246, 0.2);
    --text-on-accent: #ffffff;
  }

  /* Make white logo visible in light mode */
  :global(.light-mode .logo-icon) {
    filter: brightness(0) saturate(100%) invert(25%) sepia(60%) saturate(4000%) hue-rotate(255deg) brightness(85%) contrast(95%);
  }

  :global(.light-mode .login-logo) {
    filter: brightness(0) saturate(100%) invert(25%) sepia(60%) saturate(4000%) hue-rotate(255deg) brightness(85%) contrast(95%) drop-shadow(0 0 20px rgba(139, 92, 246, 0.25)) !important;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    background: var(--bg-base);
    background-image: var(--bg-gradient);
    background-attachment: fixed;
    color: var(--text-primary);
    min-height: 100vh;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(::-webkit-scrollbar) {
    width: 6px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: var(--scrollbar-thumb);
    border-radius: 3px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: var(--scrollbar-thumb-hover);
  }

  :global(select) {
    padding: 8px 12px;
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    background: var(--select-bg);
    color: var(--text-primary);
    font-size: 0.9rem;
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2'%3e%3cpath d='M6 9l6 6 6-6'/%3e%3c/svg%3e");
    background-repeat: no-repeat;
    background-position: right 8px center;
    background-size: 20px;
    padding-right: 32px;
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  :global(select:hover) {
    border-color: var(--glass-border-hover);
    background-color: var(--select-bg-hover);
  }

  :global(select:focus) {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  :global(select option) {
    background: var(--option-bg);
    color: var(--text-primary);
  }

  :global(.blurred) {
    filter: blur(6px);
    pointer-events: none;
    transition: filter 0.2s ease;
  }

  :global(.glass-btn) {
    padding: 8px 16px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    background: var(--glass);
    color: var(--text-primary);
    font-size: 0.85rem;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s, box-shadow 0.2s;
  }

  :global(.glass-btn:hover) {
    background: var(--glass-hover);
    border-color: var(--glass-border-hover);
    box-shadow: var(--glass-shadow);
  }

  :global(.glass-btn:active) {
    background: var(--glass-active);
    transform: scale(0.98);
  }

  :global(.tab-bar) {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: 12px;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    width: fit-content;
  }

  :global(.tab-bar button) {
    padding: 8px 18px;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  :global(.tab-bar button:hover) {
    color: var(--text-primary);
    background: var(--glass-hover);
  }

  :global(.tab-bar button.active) {
    color: var(--text-primary);
    background: var(--nav-active-bg);
    border-color: var(--nav-active-border);
  }

  .shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
  }
</style>
