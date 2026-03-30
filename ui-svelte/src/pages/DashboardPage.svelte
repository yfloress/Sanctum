<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import * as dashboardApi from '../lib/api/dashboard'
  import NetWorthChart from '../components/charts/NetWorthChart.svelte'
  import type { BalanceOverview, RecentTransaction, AnalyticsData } from '../lib/types'

  let balance = $state<BalanceOverview | null>(null)
  let recent = $state<RecentTransaction[]>([])
  let analytics = $state<AnalyticsData | null>(null)
  let selectedRange = $state('1M')
  let loading = $state(true)
  let error = $state('')

  async function load() {
    loading = true
    error = ''
    try {
      const [b, r, a] = await Promise.all([
        dashboardApi.fetchBalance(),
        dashboardApi.fetchRecent(),
        dashboardApi.fetchAnalytics(selectedRange),
      ])
      balance = b
      recent = r
      analytics = a
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function changeRange(range: string) {
    selectedRange = range
    try {
      analytics = await dashboardApi.fetchAnalytics(range)
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  $effect(() => { load() })
</script>

<div class="page">
  {#if loading}
    <div class="loading">Loading dashboard...</div>
  {:else if error}
    <div class="error-state">
      <p>{error}</p>
      <button onclick={load}>Retry</button>
    </div>
  {:else if balance}
    <section class="hero">
      <h2 class="net-worth" class:negative={balance.total_negative}>{balance.total}</h2>
      <p class="label">Net Worth ({balance.currency})</p>
      <div class="stat-cards">
        <div class="stat-card">
          <span class="stat-label">Fiat</span>
          <span class="stat-value" class:negative={balance.fiat_negative}>{balance.fiat_total}</span>
        </div>
        <div class="stat-card">
          <span class="stat-label">Crypto</span>
          <span class="stat-value" class:negative={balance.crypto_negative}>{balance.crypto_total}</span>
        </div>
      </div>
    </section>

    <section class="controls">
      <div class="tab-bar">
        <button class:active={selectedRange === '1M'} onclick={() => changeRange('1M')}>1M</button>
        <button class:active={selectedRange === '6M'} onclick={() => changeRange('6M')}>6M</button>
        <button class:active={selectedRange === '1Y'} onclick={() => changeRange('1Y')}>1Y</button>
        <button class:active={selectedRange === 'ALL'} onclick={() => changeRange('ALL')}>ALL</button>
      </div>
    </section>

    {#if analytics}
      {#if analytics.chart.dates.length > 0}
        <section class="chart-section">
          <NetWorthChart data={analytics.chart} />
        </section>
      {:else}
        <section class="chart-placeholder">
          <p class="placeholder-text">No chart data available for this range</p>
        </section>
      {/if}

      {#if analytics.expense_breakdown.length > 0}
        <section class="breakdown">
          <h3>Spending Breakdown</h3>
          {#each analytics.expense_breakdown as item}
            <div class="breakdown-row">
              <div class="breakdown-bar" style="width: {item.percentage}%; background: {item.color}"></div>
              <span class="breakdown-cat">{item.category}</span>
              <span class="breakdown-amount">{item.amount}</span>
              <span class="breakdown-pct">{item.percentage.toFixed(1)}%</span>
            </div>
          {/each}
        </section>
      {/if}
    {/if}

    {#if recent.length > 0}
      <section class="recent">
        <h3>Recent Activity</h3>
        {#each recent as tx}
          <div class="tx-row">
            <span class="tx-date">{tx.date}</span>
            <span class="tx-desc">{tx.description}</span>
            <span class="tx-cat">{tx.category}</span>
            <span class="tx-amount" class:expense={tx.is_expense}>{tx.amount}</span>
          </div>
        {/each}
      </section>
    {/if}
  {/if}
</div>

<style>
  .page {
    padding: 24px 32px;
    max-width: 960px;
    width: 100%;
    margin: 0 auto;
  }

  .loading, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 64px 0;
    color: var(--text-secondary);
  }

  .error-state button {
    margin-top: 12px;
    padding: 8px 16px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.2s;
  }

  .error-state button:hover {
    background: var(--glass-hover);
    border-color: var(--glass-border-hover);
  }

  .hero {
    text-align: center;
    padding: 24px 0 32px;
  }

  .net-worth {
    font-size: 2.4rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .net-worth.negative { color: var(--danger); }

  .label {
    color: var(--text-tertiary);
    font-size: 0.85rem;
    margin-top: 4px;
  }

  .stat-cards {
    display: flex;
    gap: 16px;
    justify-content: center;
    margin-top: 20px;
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 14px 28px;
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    min-width: 140px;
    box-shadow: var(--glass-glow);
    transition: all 0.2s;
  }

  .stat-card:hover {
    background: var(--glass-hover);
    border-color: var(--glass-border-hover);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .stat-value {
    font-size: 1.2rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-top: 4px;
  }

  .stat-value.negative { color: var(--danger); }

  .controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .chart-section {
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    padding: 16px;
    margin-bottom: 24px;
    box-shadow: var(--glass-shadow);
  }

  .chart-placeholder {
    background: var(--glass);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    padding: 48px;
    text-align: center;
    margin-bottom: 24px;
  }

  .placeholder-text { color: var(--text-tertiary); font-size: 0.85rem; }

  .breakdown {
    margin-bottom: 24px;
  }

  .breakdown h3, .recent h3 {
    font-size: 0.9rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: 12px;
  }

  .breakdown-row {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 12px;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--glass-border);
    position: relative;
  }

  .breakdown-bar {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    opacity: 0.1;
    border-radius: 4px;
  }

  .breakdown-cat { color: var(--text-secondary); font-size: 0.85rem; }
  .breakdown-amount { color: var(--text-primary); font-size: 0.85rem; font-weight: 500; }
  .breakdown-pct { color: var(--text-tertiary); font-size: 0.8rem; min-width: 48px; text-align: right; }

  .recent { margin-bottom: 24px; }

  .tx-row {
    display: grid;
    grid-template-columns: 80px 1fr auto auto;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--glass-border);
    align-items: center;
  }

  .tx-date { color: var(--text-tertiary); font-size: 0.8rem; }
  .tx-desc { color: var(--text-secondary); font-size: 0.85rem; }
  .tx-cat { color: var(--text-secondary); font-size: 0.8rem; }
  .tx-amount { color: var(--success); font-size: 0.85rem; font-weight: 500; text-align: right; }
  .tx-amount.expense { color: var(--danger); }
</style>
