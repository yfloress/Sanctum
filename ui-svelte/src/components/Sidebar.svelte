<script lang="ts">
  import { app, type Page } from '../lib/stores/app.svelte'
  import * as settingsApi from '../lib/api/settings'
  import * as vaultApi from '../lib/api/vault'

  const navItems: { page: Page, label: string, icon: string }[] = [
    { page: 'dashboard', label: 'Dashboard', icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
    { page: 'finances', label: 'Finances', icon: 'M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z' },
    { page: 'crypto', label: 'Crypto', icon: 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6' },
    { page: 'habits', label: 'Habits', icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z' },
    { page: 'settings', label: 'Settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' },
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

  <div class="sidebar-header">
    <div class="logo-container">
      <img src="/src/assets/logo/sanctum_logo.svg" alt="Sanctum" class="logo-icon" />
      {#if !app.sidebarCollapsed}
        <span class="logo-text">SANCTUM</span>
      {/if}
    </div>
  </div>

  <nav class="nav-items">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={app.activePage === item.page}
        onclick={() => app.navigate(item.page)}
        id="nav-{item.page}"
      >
        <div class="nav-icon-wrap">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d={item.icon} />
          </svg>
        </div>
        {#if !app.sidebarCollapsed}
          <span class="nav-label">{item.label}</span>
        {/if}
        {#if app.activePage === item.page}
          <div class="active-indicator"></div>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="sidebar-footer">
    <button class="nav-item collapse-btn" onclick={toggleCollapsed} aria-label="Toggle sidebar" id="toggle-sidebar">
      <div class="nav-icon-wrap">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          {#if app.sidebarCollapsed}
            <path d="M13 5l7 7-7 7M5 5l7 7-7 7" />
          {:else}
            <path d="M11 19l-7-7 7-7M19 19l-7-7 7-7" />
          {/if}
        </svg>
      </div>
      {#if !app.sidebarCollapsed}
        <span class="nav-label">Collapse</span>
      {/if}
    </button>
    <button class="nav-item lock-btn" onclick={handleLock} id="lock-vault">
      <div class="nav-icon-wrap">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>
      {#if !app.sidebarCollapsed}
        <span class="nav-label">Lock</span>
      {/if}
    </button>
  </div>
</aside>

<style>
  /* ===== Sidebar Shell ===== */
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 220px;
    height: 100vh;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--sidebar-border);
    transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    flex-shrink: 0;
    position: relative;
    z-index: 10;
  }

  .sidebar.collapsed {
    width: 64px;
  }

  /* ===== Drag Region (borderless window) ===== */
  .drag-region {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 30px;
    z-index: 100;
    -webkit-app-region: drag;
  }

  /* ===== Header ===== */
  .sidebar-header {
    display: flex;
    align-items: center;
    padding: 20px 16px 16px;
    min-height: 60px;
    justify-content: center;
  }

  .collapsed .sidebar-header {
    padding: 20px 0 16px;
  }

  .logo-container {
    display: flex;
    align-items: center;
    gap: 11px;
  }

  .logo-icon {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    filter: drop-shadow(0 0 10px rgba(168, 85, 247, 0.25));
  }

  .logo-text {
    font-size: 1rem;
    font-weight: 700;
    letter-spacing: 0.28em;
    color: var(--text-primary);
    background: linear-gradient(135deg, var(--text-primary) 0%, var(--accent) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    white-space: nowrap;
    overflow: hidden;
  }

  /* ===== Navigation ===== */
  .nav-items {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    margin-top: 4px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0;
    height: 40px;
    padding-left: 12px;
    padding-right: 12px;
    border: none;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 450;
    text-align: left;
    transition: all 0.2s ease;
    position: relative;
    overflow: hidden;
    width: 100%;
  }

  .collapsed .nav-item {
    justify-content: center;
    padding-left: 0;
    padding-right: 0;
  }

  .nav-icon-wrap {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.2s ease;
  }

  .nav-icon-wrap svg {
    width: 20px;
    height: 20px;
  }

  .nav-label {
    white-space: nowrap;
    overflow: hidden;
    letter-spacing: 0.01em;
  }

  /* ===== Hover State ===== */
  .nav-item:hover {
    background: var(--glass-hover);
    color: var(--text-primary);
  }

  .nav-item:hover .nav-icon-wrap {
    transform: scale(1.08);
  }

  /* ===== Active State ===== */
  .nav-item.active {
    background: var(--nav-active-bg);
    color: var(--text-primary);
  }

  .active-indicator {
    position: absolute;
    left: 0;
    top: 6px;
    bottom: 6px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--accent);
    box-shadow: 0 0 12px rgba(168, 85, 247, 0.4), 0 0 4px rgba(168, 85, 247, 0.6);
  }

  .nav-item.active .nav-icon-wrap svg {
    filter: drop-shadow(0 0 6px rgba(168, 85, 247, 0.3));
  }

  /* ===== Footer ===== */
  .sidebar-footer {
    padding: 10px;
    border-top: 1px solid var(--glass-border);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .collapse-btn svg {
    width: 18px;
    height: 18px;
  }

  /* ===== Lock Button ===== */
  .lock-btn:hover {
    color: var(--danger);
    background: var(--danger-bg);
  }

  .lock-btn:hover .nav-icon-wrap svg {
    filter: drop-shadow(0 0 6px rgba(248, 113, 113, 0.3));
  }
</style>
