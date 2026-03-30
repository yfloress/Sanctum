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
  <div class="sidebar-header">
    {#if !app.sidebarCollapsed}
      <span class="logo-text">SANCTUM</span>
    {/if}
    <button class="collapse-btn" onclick={toggleCollapsed} aria-label="Toggle sidebar">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        {#if app.sidebarCollapsed}
          <path d="M13 5l7 7-7 7M5 5l7 7-7 7" />
        {:else}
          <path d="M11 19l-7-7 7-7M19 19l-7-7 7-7" />
        {/if}
      </svg>
    </button>
  </div>

  <nav class="nav-items">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={app.activePage === item.page}
        onclick={() => app.navigate(item.page)}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d={item.icon} />
        </svg>
        {#if !app.sidebarCollapsed}
          <span>{item.label}</span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="sidebar-footer">
    <button class="nav-item lock-btn" onclick={handleLock}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
      {#if !app.sidebarCollapsed}
        <span>Lock</span>
      {/if}
    </button>
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 220px;
    height: 100vh;
    background: linear-gradient(180deg, #111116 0%, #0d0d11 100%);
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    transition: width 0.25s ease;
    flex-shrink: 0;
  }

  .sidebar.collapsed {
    width: 60px;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 12px;
    border-bottom: 1px solid var(--glass-border);
    min-height: 56px;
  }

  .logo-text {
    font-size: 1.1rem;
    font-weight: 700;
    letter-spacing: 0.25em;
    color: var(--text-primary);
    text-shadow: 0 0 20px var(--accent-glow);
  }

  .collapse-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    transition: color 0.15s;
  }

  .collapse-btn:hover {
    color: var(--text-primary);
  }

  .collapse-btn svg {
    width: 18px;
    height: 18px;
  }

  .nav-items {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.9rem;
    text-align: left;
    transition: all 0.2s ease;
    position: relative;
  }

  .nav-item:hover {
    background: var(--glass-hover);
    border-color: var(--glass-border);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .nav-item.active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 4px;
    bottom: 4px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent-glow);
  }

  .nav-item svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .sidebar-footer {
    padding: 8px;
    border-top: 1px solid var(--glass-border);
  }

  .lock-btn:hover {
    color: var(--danger);
    background: rgba(248, 113, 113, 0.08);
    border-color: rgba(248, 113, 113, 0.15);
  }
</style>
