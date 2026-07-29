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

<script module lang="ts">
  import { errorMessage } from '../lib/errors'
  let savedRange = '1M'
</script>

<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as dashboardApi from '../lib/api/dashboard'
  import { mask } from '../lib/currency'
  import NetWorthChart from '../components/charts/NetWorthChart.svelte'
  import FinanceBarChart from '../components/charts/FinanceBarChart.svelte'
  import type { BalanceOverview, RecentTransaction, AnalyticsData } from '../lib/types'
  import { untrack } from 'svelte'

  let balance = $state<BalanceOverview | null>(null)
  let recent = $state<RecentTransaction[]>([])
  let analytics = $state<AnalyticsData | null>(null)
  let selectedRange = $state(savedRange)
  let rangeLoading = $state(false)
  let loading = $state(true)
  let error = $state('')

  async function load() {
    const range = untrack(() => selectedRange)
    loading = true
    error = ''
    try {
      const [b, r, a] = await Promise.all([
        dashboardApi.fetchBalance(),
        dashboardApi.fetchRecent(),
        dashboardApi.fetchAnalytics(range),
      ])
      balance = b
      recent = r
      analytics = a
    } catch (e) {
      error = errorMessage(e)
    } finally {
      loading = false
    }
  }

  async function changeRange(range: string) {
    savedRange = range
    selectedRange = range
    rangeLoading = true
    try {
      analytics = await dashboardApi.fetchAnalytics(range)
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      rangeLoading = false
    }
  }

  // % change from start of selected period to today
  let pctChange = $derived.by(() => {
    if (!analytics || analytics.chart.values.length < 2) return null
    const first = analytics.chart.values[0]
    const last = analytics.chart.values[analytics.chart.values.length - 1]
    if (first === 0) return null
    return ((last - first) / Math.abs(first)) * 100
  })

  let cashFlow = $derived({
    months:   analytics?.monthly_cash_flow.map(m => m.month)    ?? [],
    income:   analytics?.monthly_cash_flow.map(m => m.income)   ?? [],
    expenses: analytics?.monthly_cash_flow.map(m => m.expenses) ?? [],
  })

  $effect(() => {
    app.settings?.preferred_currency
    load()
  })
</script>

<div class="page">
  {#if loading}
    <div class="skeleton-page">
      <div class="skeleton" style="width:160px;height:14px;margin-bottom:10px"></div>
      <div class="skeleton" style="width:220px;height:42px;margin-bottom:6px"></div>
      <div class="skeleton" style="width:80px;height:20px;margin-bottom:32px"></div>
      <div class="skeleton-row">
        <div class="skeleton" style="flex:1;height:88px;border-radius:var(--radius-md)"></div>
        <div class="skeleton" style="flex:1;height:88px;border-radius:var(--radius-md)"></div>
        <div class="skeleton" style="flex:1;height:88px;border-radius:var(--radius-md)"></div>
      </div>
      <div class="skeleton" style="width:100%;height:260px;border-radius:var(--radius-lg);margin-bottom:16px"></div>
      <div class="skeleton-row">
        <div class="skeleton" style="flex:1;height:180px;border-radius:var(--radius-lg)"></div>
        <div class="skeleton" style="flex:1;height:180px;border-radius:var(--radius-lg)"></div>
      </div>
    </div>

  {:else if error}
    <div class="error-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="width:32px;height:32px;color:var(--danger)">
        <circle cx="12" cy="12" r="10"/><path d="M12 8v4M12 16h.01"/>
      </svg>
      <p>{error}</p>
      <button onclick={load}>{i18n.t('dashboard-retry', 'Retry')}</button>
    </div>

  {:else if balance}

    <!-- ── Hero ─────────────────────────────────────────────────── -->
    <section class="hero">
      <p class="hero-label">{i18n.t('dashboard-net-worth', 'Net Worth')} · {balance.currency}</p>
      <div class="hero-value-row">
        <h1 class="net-worth" class:negative={balance.total_negative}>{mask(balance.total)}</h1>
        {#if pctChange !== null}
          <span class="trend-badge" class:positive={pctChange >= 0} class:negative={pctChange < 0}>
            {#if pctChange >= 0}
              <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 12V4M4 8l4-4 4 4"/></svg>
            {:else}
              <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 4v8M4 8l4 4 4-4"/></svg>
            {/if}
            {Math.abs(pctChange).toFixed(1)}%
            <span class="trend-period">{selectedRange}</span>
          </span>
        {/if}
      </div>

      {#if analytics && (analytics.net_worth_min || analytics.net_worth_max)}
        <p class="net-worth-range">
          {mask(analytics.net_worth_min)}
          <span class="range-sep">——</span>
          {mask(analytics.net_worth_max)}
          <span class="range-period">({selectedRange})</span>
        </p>
      {/if}

      <div class="balance-strip">
        <div class="balance-cell">
          <span class="balance-cell-label">{i18n.t('dashboard-fiat', 'Fiat')}</span>
          <span class="balance-cell-value" class:negative={balance.fiat_negative}>{mask(balance.fiat_total)}</span>
        </div>
        <div class="balance-divider"></div>
        <div class="balance-cell">
          <span class="balance-cell-label">{i18n.t('dashboard-crypto', 'Crypto')}</span>
          <span class="balance-cell-value" class:negative={balance.crypto_negative}>{mask(balance.crypto_total)}</span>
        </div>
      </div>
    </section>

    {#if analytics}
      <!-- ── Stats row ────────────────────────────────────────── -->
      <div class="stats-row">
        <div class="stat-card income">
          <span class="stat-label">{i18n.t('dashboard-income', 'Income')}</span>
          <span class="stat-value">{mask(analytics.total_income)}</span>
          <span class="stat-period">{i18n.t('dashboard-last', 'last')} {selectedRange}</span>
        </div>
        <div class="stat-card expenses">
          <span class="stat-label">{i18n.t('dashboard-expenses', 'Expenses')}</span>
          <span class="stat-value">{mask(analytics.total_expenses)}</span>
          <span class="stat-period">{i18n.t('dashboard-last', 'last')} {selectedRange}</span>
        </div>
        <div class="stat-card net" class:negative={analytics.total_net_negative}>
          <span class="stat-label">{i18n.t('dashboard-net', 'Net')}</span>
          <span class="stat-value">{mask(`${analytics.total_net_negative ? '−' : '+'}${analytics.total_net}`)}</span>
          <span class="stat-period">{i18n.t('dashboard-last', 'last')} {selectedRange}</span>
        </div>
      </div>

      <!-- ── Net Worth Chart ───────────────────────────────────── -->
      <div class="chart-card">
        <div class="chart-card-header">
          <h4>{i18n.t('dashboard-net-worth-trend', 'Net Worth Trend')}</h4>
          <div class="range-picker">
            {#each ['1M','3M','6M','1Y','ALL'] as r}
              <button class:active={selectedRange === r} onclick={() => changeRange(r)}>{r}</button>
            {/each}
          </div>
        </div>
        {#if analytics.chart.dates.length > 0}
          <NetWorthChart data={analytics.chart} range={selectedRange} />
          {#if rangeLoading}
            <div class="chart-loading-overlay"><div class="mini-spinner"></div></div>
          {/if}
        {:else}
          <p class="chart-empty">{i18n.t('dashboard-no-data-range', 'No data for this range')}</p>
        {/if}
      </div>

      <!-- ── Cash Flow Chart ──────────────────────────────────── -->
      {#if cashFlow.months.length > 0}
        <div class="chart-card">
          <div class="chart-card-header">
            <h4>{i18n.t('dashboard-monthly-cash-flow', 'Monthly Cash Flow')}</h4>
            <span class="chart-subtitle">{i18n.t('dashboard-last-6-months', 'Last 6 months')}</span>
          </div>
          <FinanceBarChart
            months={cashFlow.months}
            income={cashFlow.income}
            expenses={cashFlow.expenses}
          />
        </div>
      {/if}
    {/if}

    <!-- ── Bottom grid ─────────────────────────────────────────── -->
    {#if (analytics && analytics.expense_breakdown.length > 0) || recent.length > 0}
      <div class="bottom-grid">

        {#if analytics && analytics.expense_breakdown.length > 0}
          <div class="panel">
            <h4 class="panel-title">{i18n.t('dashboard-spending-breakdown', 'Spending Breakdown')} <span class="panel-title-period">({selectedRange})</span></h4>
            <div class="breakdown-list">
              {#each analytics.expense_breakdown as item}
                <div class="breakdown-row">
                  <div class="breakdown-row-top">
                    <span class="breakdown-cat">{item.category}</span>
                    <div class="breakdown-right">
                      <span class="breakdown-amount">{mask(item.amount)}</span>
                      <span class="breakdown-pct">{item.percentage.toFixed(1)}%</span>
                    </div>
                  </div>
                  <div class="breakdown-track">
                    <div class="breakdown-fill" style="width:{item.percentage}%;background:{item.color}"></div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        {#if recent.length > 0}
          <div class="panel">
            <h4 class="panel-title">{i18n.t('dashboard-recent-activity', 'Recent Activity')}</h4>
            <div class="tx-list">
              {#each recent as tx}
                <div class="tx-row">
                  <span class="tx-type-dot" class:expense={tx.is_expense}></span>
                  <div class="tx-main">
                    <span class="tx-desc">{tx.description || tx.category}</span>
                    <div class="tx-meta">
                      <span class="tx-cat-badge">{tx.category}</span>
                      <span class="tx-date">{tx.date}</span>
                    </div>
                  </div>
                  <span class="tx-amount" class:expense={tx.is_expense}>{mask(tx.amount)}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}

      </div>
    {/if}

  {:else}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="width:48px;height:48px;color:var(--accent);margin-bottom:16px">
        <path d="M12 20V10"/>
        <path d="M18 20V4"/>
        <path d="M6 20v-4"/>
      </svg>
      <h3>{i18n.t('dashboard-welcome', 'Welcome to Sanctum')}</h3>
      <p>{i18n.t('dashboard-welcome-desc', 'Add accounts and transactions in the Finances page to see your overview here.')}</p>
    </div>

  {/if}
</div>

<style>
  .page {
    padding: 32px;
    max-width: 960px;
    width: 100%;
    margin: 0 auto;
  }

  /* ── Loading / error ─────────────────────────────────────── */
  .skeleton-page { padding: 8px 0; }
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 64px 0;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }
  .error-state button {
    padding: 8px 20px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.2s;
  }
  .error-state button:hover {
    background: var(--glass-hover);
    border-color: var(--glass-border-hover);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 0;
    text-align: center;
  }
  .empty-state h3 {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 8px;
  }
  .empty-state p {
    font-size: 0.9rem;
    color: var(--text-secondary);
    max-width: 320px;
    margin: 0;
    line-height: 1.5;
  }

  /* ── Hero ─────────────────────────────────────────────────── */
  .hero {
    text-align: center;
    padding: 20px 0 32px;
  }
  .hero-label {
    font-size: 0.68rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    margin: 0 0 10px;
  }
  .hero-value-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-bottom: 6px;
  }
  .net-worth {
    font-size: 3.4rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.035em;
    margin: 0;
    line-height: 1;
  }
  .net-worth.negative { color: var(--danger); }

  .trend-badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 4px 10px 4px 7px;
    border-radius: 20px;
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.01em;
    margin-top: 6px;
  }
  .trend-badge svg { width: 14px; height: 14px; flex-shrink: 0; }
  .trend-badge.positive {
    background: rgba(74, 222, 128, 0.12);
    color: var(--success);
    border: 1px solid rgba(74, 222, 128, 0.2);
  }
  .trend-badge.negative {
    background: rgba(248, 113, 113, 0.12);
    color: var(--danger);
    border: 1px solid rgba(248, 113, 113, 0.2);
  }
  .trend-period {
    font-size: 0.68rem;
    font-weight: 400;
    opacity: 0.65;
    margin-left: 2px;
  }

  .net-worth-range {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    margin: 0 0 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }
  .range-sep { opacity: 0.35; }
  .range-period { opacity: 0.55; }

  .balance-strip {
    display: inline-flex;
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    box-shadow: var(--glass-shadow);
  }
  .balance-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 14px 44px;
    gap: 5px;
  }
  .balance-divider {
    width: 1px;
    background: var(--glass-border);
    margin: 10px 0;
  }
  .balance-cell-label {
    font-size: 0.68rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .balance-cell-value {
    font-size: 1.2rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }
  .balance-cell-value.negative { color: var(--danger); }

  /* ── Stats row ────────────────────────────────────────────── */
  .stats-row {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 12px;
    margin-bottom: 16px;
  }
  .stat-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    box-shadow: var(--card-shadow);
    border-left-width: 3px;
    overflow: hidden;
  }
  .stat-card.income  { border-left-color: var(--success); }
  .stat-card.expenses { border-left-color: var(--danger); }
  .stat-card.net     { border-left-color: var(--accent); }
  .stat-card.net.negative { border-left-color: var(--danger); }

  .stat-label {
    font-size: 0.68rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .stat-value {
    font-size: 1.15rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }
  .stat-card.net .stat-value     { color: var(--success); }
  .stat-card.net.negative .stat-value { color: var(--danger); }
  .stat-period {
    font-size: 0.67rem;
    color: var(--text-tertiary);
    opacity: 0.7;
  }

  /* ── Chart card ───────────────────────────────────────────── */
  .chart-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 20px 20px 6px;
    margin-bottom: 16px;
    box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .chart-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
  }
  .chart-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }
  .chart-card-header h4 {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin: 0;
  }
  .chart-subtitle {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    opacity: 0.6;
  }
  .range-picker {
    display: flex;
    gap: 2px;
    background: var(--glass-active);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    padding: 2px;
  }
  .range-picker button {
    padding: 4px 12px;
    border: none;
    border-radius: 6px;
    background: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 0.72rem;
    font-weight: 500;
    transition: all 0.15s;
  }
  .range-picker button:hover { color: var(--text-primary); }
  .range-picker button.active {
    background: var(--glass-elevated);
    color: var(--text-primary);
    box-shadow: 0 1px 4px rgba(0,0,0,0.25);
  }
  .chart-empty {
    padding: 48px;
    text-align: center;
    color: var(--text-tertiary);
    font-size: 0.85rem;
    margin: 0;
  }

  .chart-card { position: relative; }
  .chart-loading-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    pointer-events: none;
    z-index: 2;
  }

  /* cash-flow */

  /* ── Bottom grid ──────────────────────────────────────────── */
  .bottom-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 8px;
  }
  .bottom-grid .panel:only-child {
    grid-column: 1 / -1;
  }
  .panel {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 20px;
    box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .panel::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
  }
  .panel-title {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin: 0 0 16px;
  }
  .panel-title-period {
    font-size: 0.65rem;
    opacity: 0.6;
    text-transform: none;
    letter-spacing: 0;
  }

  /* ── Spending breakdown ───────────────────────────────────── */
  .breakdown-list { display: flex; flex-direction: column; gap: 13px; }
  .breakdown-row { display: flex; flex-direction: column; gap: 5px; }
  .breakdown-row-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .breakdown-cat { font-size: 0.82rem; color: var(--text-secondary); }
  .breakdown-right { display: flex; align-items: center; gap: 10px; }
  .breakdown-amount { font-size: 0.82rem; font-weight: 500; color: var(--text-primary); }
  .breakdown-pct { font-size: 0.7rem; color: var(--text-tertiary); min-width: 34px; text-align: right; }
  .breakdown-track {
    height: 4px;
    background: var(--glass-active);
    border-radius: 2px;
    overflow: hidden;
  }
  .breakdown-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 0.5s ease;
  }

  /* ── Recent transactions ──────────────────────────────────── */
  .tx-list { display: flex; flex-direction: column; }
  .tx-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 0;
    border-bottom: 1px solid var(--glass-border);
  }
  .tx-row:last-child { border-bottom: none; }
  .tx-type-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--success);
    flex-shrink: 0;
  }
  .tx-type-dot.expense { background: var(--danger); }
  .tx-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .tx-desc {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tx-meta { display: flex; align-items: center; gap: 8px; }
  .tx-cat-badge {
    font-size: 0.67rem;
    padding: 1px 6px;
    border-radius: 20px;
    background: var(--glass-active);
    border: 1px solid var(--glass-border);
    color: var(--text-tertiary);
    white-space: nowrap;
  }
  .tx-date { font-size: 0.71rem; color: var(--text-tertiary); }
  .tx-amount {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--success);
    white-space: nowrap;
    text-align: right;
  }
  .tx-amount.expense { color: var(--danger); }

  /* ── Mobile ───────────────────────────────────────────────── */
  @media (max-width: 640px) {
    .page { padding: 20px 16px; }
    .hero { padding: 12px 0 24px; }
    .net-worth { font-size: 2.3rem; }
    .hero-value-row { gap: 8px; flex-wrap: wrap; }
    .balance-cell { padding: 12px 22px; }
    .stats-row { grid-template-columns: 1fr; }
    .bottom-grid { grid-template-columns: 1fr; }
  }
</style>
