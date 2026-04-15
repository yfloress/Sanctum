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
  import { app, type Page } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as settingsApi from '../lib/api/settings'
  import * as vaultApi from '../lib/api/vault'

  const navItems: { page: Page, key: string, icon: string }[] = [
    { page: 'dashboard', key: 'nav-dashboard', icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
    { page: 'finances', key: 'nav-finances', icon: 'M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z' },
    { page: 'crypto', key: 'nav-crypto', icon: 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6' },
    { page: 'habits', key: 'nav-habits', icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z' },
    { page: 'settings', key: 'nav-settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' },
  ]

  async function toggleCollapsed() {
    const next = !app.sidebarCollapsed
    if (app.settings) app.settings.sidebar_collapsed = next
    await settingsApi.setSidebarCollapsed(next)
  }

  async function handleLock() {
    await vaultApi.lockVault()
    app.logout()
  }
</script>

<aside class="sidebar" class:collapsed={app.sidebarCollapsed}>
  <!-- Drag region for borderless window -->
  <div class="drag-region" data-tauri-drag-region></div>

  <!-- ── Header ── -->
  <div class="sidebar-header">
    <div class="logo-container">
      <div class="logo-icon-wrap">
        <img src="/src/assets/logo/sanctum_logo.svg" alt="Sanctum" class="logo-icon" />
      </div>
      <span class="logo-text">SANCTUM</span>
    </div>
  </div>

  <!-- ── Nav ── -->
  <nav class="nav-items">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={app.activePage === item.page}
        onclick={() => app.navigate(item.page)}
        id="nav-{item.page}"
        title={app.sidebarCollapsed ? i18n.t(item.key) : undefined}
      >
        {#if app.activePage === item.page}
          <div class="active-indicator"></div>
        {/if}
        <div class="nav-icon-wrap">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d={item.icon} />
          </svg>
        </div>
        <span class="nav-label">{i18n.t(item.key)}</span>
      </button>
    {/each}
  </nav>

  <!-- ── Footer ── -->
  <div class="sidebar-footer">
    <button
      class="nav-item collapse-btn"
      onclick={toggleCollapsed}
      aria-label="Toggle sidebar"
      id="toggle-sidebar"
      title={app.sidebarCollapsed ? i18n.t('nav-expand', 'Expand') : undefined}
    >
      <div class="nav-icon-wrap collapse-icon" class:rotated={app.sidebarCollapsed}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M11 19l-7-7 7-7M19 19l-7-7 7-7" />
        </svg>
      </div>
      <span class="nav-label">{i18n.t('nav-collapse', 'Collapse')}</span>
    </button>

    <button
      class="nav-item lock-btn"
      onclick={handleLock}
      id="lock-vault"
      title={app.sidebarCollapsed ? i18n.t('nav-lock', 'Lock') : undefined}
    >
      <div class="nav-icon-wrap">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>
      <span class="nav-label">{i18n.t('nav-lock', 'Lock')}</span>
    </button>
  </div>
</aside>

<style>
  /* ── Shell ─────────────────────────────────────────── */
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 220px;
    height: 100vh;
    background: linear-gradient(170deg, #13111d 0%, #0f0d18 45%, #0c0b14 100%);
    border-right: 1px solid rgba(168, 85, 247, 0.08);
    box-shadow: inset -1px 0 0 rgba(168, 85, 247, 0.04), 4px 0 24px rgba(0, 0, 0, 0.3);
    transition: width 0.35s cubic-bezier(0.16, 1, 0.3, 1);
    flex-shrink: 0;
    position: relative;
    z-index: 10;
    overflow: hidden;
  }

  .sidebar.collapsed {
    width: 64px;
  }

  /* ── Drag region ────────────────────────────────────── */
  .drag-region {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 30px;
    z-index: 100;
    -webkit-app-region: drag;
  }

  /* ── Header ─────────────────────────────────────────── */
  .sidebar-header {
    padding: 22px 14px 16px;
    min-height: 62px;
    display: flex;
    align-items: center;
  }

  .logo-container {
    display: flex;
    align-items: center;
    gap: 10px;
    overflow: hidden;
  }

  .logo-icon-wrap {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo-icon {
    width: 28px;
    height: 28px;
    filter: drop-shadow(0 0 10px rgba(168, 85, 247, 0.3));
  }

  .logo-text {
    font-size: 0.95rem;
    font-weight: 700;
    letter-spacing: 0.3em;
    background: linear-gradient(135deg, #e8eaed 0%, #a855f7 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    white-space: nowrap;
    /* animated via label mixin below */
    opacity: 1;
    max-width: 160px;
    transform: translateX(0);
    transition: opacity 0.2s ease, max-width 0.35s cubic-bezier(0.16, 1, 0.3, 1), transform 0.2s ease;
  }

  .collapsed .logo-text {
    opacity: 0;
    max-width: 0;
    transform: translateX(-6px);
    pointer-events: none;
  }

  /* ── Nav ─────────────────────────────────────────────── */
  .nav-items {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 8px;
    overflow: hidden;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 40px;
    padding: 0 10px;
    border: none;
    border-radius: 10px;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 450;
    text-align: left;
    transition: background 0.18s ease, color 0.18s ease;
    position: relative;
    width: 100%;
    overflow: hidden;
    white-space: nowrap;
  }

  /* ── Nav label animation ─────────────────────────────── */
  .nav-label {
    opacity: 1;
    max-width: 160px;
    transform: translateX(0);
    overflow: hidden;
    white-space: nowrap;
    letter-spacing: 0.01em;
    transition: opacity 0.2s ease, max-width 0.35s cubic-bezier(0.16, 1, 0.3, 1), transform 0.2s ease;
    flex-shrink: 0;
  }

  .collapsed .nav-label {
    opacity: 0;
    max-width: 0;
    transform: translateX(-6px);
    pointer-events: none;
  }

  .collapsed .nav-item {
    justify-content: center;
    padding: 0;
  }

  /* ── Icon ────────────────────────────────────────────── */
  .nav-icon-wrap {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.2s ease, color 0.18s ease;
  }

  .nav-icon-wrap svg {
    width: 20px;
    height: 20px;
  }

  /* ── Hover ───────────────────────────────────────────── */
  .nav-item:hover {
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-primary);
  }

  .nav-item:hover .nav-icon-wrap {
    transform: scale(1.08);
    color: rgba(196, 181, 253, 0.8);
  }

  /* ── Active ──────────────────────────────────────────── */
  .nav-item.active {
    background: linear-gradient(90deg, rgba(168, 85, 247, 0.16) 0%, rgba(168, 85, 247, 0.04) 100%);
    color: var(--text-primary);
  }

  .nav-item.active .nav-icon-wrap {
    color: #c084fc;
  }

  .nav-item.active .nav-label {
    color: #e2d9f3;
    font-weight: 500;
  }

  .active-indicator {
    position: absolute;
    left: 0;
    top: 8px;
    bottom: 8px;
    width: 2px;
    border-radius: 0 2px 2px 0;
    background: linear-gradient(180deg, #d8b4fe, #a855f7);
    box-shadow: 0 0 8px rgba(168, 85, 247, 0.5);
  }

  /* ── Footer ──────────────────────────────────────────── */
  .sidebar-footer {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    position: relative;
    overflow: hidden;
  }

  .sidebar-footer::before {
    content: '';
    display: block;
    height: 1px;
    margin: 0 4px 8px;
    background: linear-gradient(
      90deg,
      transparent 0%,
      rgba(168, 85, 247, 0.15) 30%,
      rgba(168, 85, 247, 0.2) 50%,
      rgba(168, 85, 247, 0.15) 70%,
      transparent 100%
    );
  }

  /* ── Collapse button ─────────────────────────────────── */
  .collapse-icon {
    transition: transform 0.35s cubic-bezier(0.16, 1, 0.3, 1), color 0.18s ease;
  }

  .collapse-icon.rotated {
    transform: rotate(180deg);
  }

  /* ── Lock button ─────────────────────────────────────── */
  .lock-btn:hover {
    background: rgba(248, 113, 113, 0.1);
    color: var(--danger);
  }

  .lock-btn:hover .nav-icon-wrap {
    color: var(--danger);
    filter: drop-shadow(0 0 5px rgba(248, 113, 113, 0.3));
  }
</style>
