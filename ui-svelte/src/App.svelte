<script lang="ts">
  import { app } from './lib/stores/app.svelte'
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
      return startSessionMonitor()
    }
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
    --bg-base: #07080f;
    --bg-gradient: linear-gradient(135deg, #07080f 0%, #0d1117 40%, #101520 100%);
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
    --accent: #4f9cf7;
    --accent-glow: rgba(79, 156, 247, 0.15);
    --accent-hover: #6aafff;
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
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: rgba(255, 255, 255, 0.18);
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
