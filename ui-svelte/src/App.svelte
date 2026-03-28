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
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: system-ui, -apple-system, sans-serif;
    background: #0a0a0a;
    color: #e0e0e0;
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
    background: #333;
    border-radius: 3px;
  }

  .shell {
    display: flex;
    min-height: 100vh;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    max-height: 100vh;
  }
</style>
