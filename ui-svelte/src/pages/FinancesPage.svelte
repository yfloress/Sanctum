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
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as financeApi from '../lib/api/finance'
  import type {
    TransactionDto, CategoriesResponse, CategoryDto,
    AccountsResponse, AccountDetailResponse, AccountDto
  } from '../lib/types'
  import { accountTypeLabel, getAccountDisplayIcon, isGenericIcon } from '../lib/accountDisplay'
  import FinanceBarChart from '../components/charts/FinanceBarChart.svelte'
  import FinanceDonutChart from '../components/charts/FinanceDonutChart.svelte'
  import FinanceCategoryChart from '../components/charts/FinanceCategoryChart.svelte'
  import FinanceTransactionModal from '../components/finance/FinanceTransactionModal.svelte'
  import FinanceAccountModal from '../components/finance/FinanceAccountModal.svelte'
  import FinanceTransferModal from '../components/finance/FinanceTransferModal.svelte'
  import FinanceAccountPanel from '../components/finance/FinanceAccountPanel.svelte'
  import FinanceCategoryPanel from '../components/finance/FinanceCategoryPanel.svelte'
  import ConfirmDialog from '../components/ConfirmDialog.svelte'

  type Tab = 'overview' | 'activity' | 'accounts' | 'settings'

  let activeTab = $state<Tab>('overview')
  let loading = $state(true)

  // Activity state
  let transactions = $state<TransactionDto[]>([])
  // Unfiltered transactions used for Overview charts
  let chartTransactions = $state<TransactionDto[]>([])
  let hasMore = $state(false)
  let filterQuery = $state('')
  let filterAccountId = $state('')
  let filterCategory = $state('')
  type DateRange = 'all' | 'this-month' | 'last-30' | 'last-90' | 'this-year' | 'custom'
  let filterDateRange = $state<DateRange>('all')
  let filterDateFrom = $state('')
  let filterDateTo = $state('')

  function computeDateBounds(): { from: string | undefined; to: string | undefined } {
    const today = new Date()
    const iso = (d: Date) => d.toISOString().slice(0, 10)
    switch (filterDateRange) {
      case 'this-month': {
        const start = new Date(today.getFullYear(), today.getMonth(), 1)
        return { from: iso(start), to: iso(today) }
      }
      case 'last-30': {
        const start = new Date(today)
        start.setDate(today.getDate() - 30)
        return { from: iso(start), to: iso(today) }
      }
      case 'last-90': {
        const start = new Date(today)
        start.setDate(today.getDate() - 90)
        return { from: iso(start), to: iso(today) }
      }
      case 'this-year': {
        const start = new Date(today.getFullYear(), 0, 1)
        return { from: iso(start), to: iso(today) }
      }
      case 'custom':
        return { from: filterDateFrom || undefined, to: filterDateTo || undefined }
      case 'all':
      default:
        return { from: undefined, to: undefined }
    }
  }

  let hasActiveFilters = $derived(
    !!filterQuery || !!filterAccountId || !!filterCategory ||
    filterDateRange !== 'all'
  )

  // Accounts state
  let accountsData = $state<AccountsResponse | null>(null)
  let selectedAccount = $state<AccountDetailResponse | null>(null)

  // Categories state
  let categories = $state<CategoriesResponse | null>(null)

  // Archived accounts state
  let archivedAccounts = $state<AccountDto[]>([])

  // Modal state
  let showAddTransaction = $state(false)
  let showAddAccount = $state(false)
  let showTransfer = $state(false)
  let editingTransaction = $state<TransactionDto | null>(null)
  let editingAccount = $state<AccountDetailResponse | null>(null)
  let editingTransfer = $state<TransactionDto | null>(null)
  let pendingDeleteTx = $state<TransactionDto | null>(null)

  async function loadAll() {
    loading = true
    try {
      const [acc, cats] = await Promise.all([
        financeApi.fetchAccounts(),
        financeApi.loadCategories(),
      ])
      accountsData = acc
      categories = cats
      await Promise.all([loadTransactions(), loadChartTransactions()])
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      loading = false
    }
  }

  async function loadChartTransactions() {
    try {
      const res = await financeApi.fetchTransactions(undefined, undefined, undefined, undefined, undefined, 1000)
      chartTransactions = res.transactions
    } catch (_) {
      // charts degrade gracefully
    }
  }

  async function loadTransactions() {
    try {
      const { from, to } = computeDateBounds()
      const res = await financeApi.fetchTransactions(
        filterQuery || undefined, filterAccountId || undefined,
        filterCategory || undefined, from, to, 100
      )
      transactions = res.transactions
      hasMore = res.has_more
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  let searchDebounce: ReturnType<typeof setTimeout> | undefined
  function onSearchInput() {
    if (searchDebounce) clearTimeout(searchDebounce)
    searchDebounce = setTimeout(() => loadTransactions(), 200)
  }

  async function loadMoreTransactions() {
    try {
      const { from, to } = computeDateBounds()
      const res = await financeApi.fetchTransactions(
        filterQuery || undefined, filterAccountId || undefined,
        filterCategory || undefined, from, to, transactions.length + 100
      )
      transactions = res.transactions
      hasMore = res.has_more
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function clearFilters() {
    filterQuery = ''
    filterAccountId = ''
    filterCategory = ''
    filterDateRange = 'all'
    filterDateFrom = ''
    filterDateTo = ''
    loadTransactions()
  }

  async function openAccountDetail(id: string) {
    try {
      selectedAccount = await financeApi.fetchAccountDetails(id)
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function openAddTransaction() {
    editingTransaction = null
    showAddTransaction = true
  }

  function openEditTransaction(tx: TransactionDto) {
    if (tx.is_transfer) {
      openEditTransfer(tx)
      return
    }
    editingTransaction = tx
    showAddTransaction = true
  }

  function openEditTransfer(tx: TransactionDto) {
    editingTransfer = tx
    showTransfer = true
  }

  async function deleteTransaction(id: string) {
    try {
      await financeApi.deleteTransaction(id)
      await Promise.all([loadTransactions(), refreshAccounts(), loadChartTransactions()])
      app.showToast(i18n.t('finances-tx-deleted', 'Transaction deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function openAddAccount() {
    editingAccount = null
    showAddAccount = true
  }

  function openEditAccount(detail: AccountDetailResponse) {
    editingAccount = detail
    showAddAccount = true
  }

  async function deleteAccount(id: string) {
    try {
      await financeApi.deleteAccount(id)
      selectedAccount = null
      await refreshAccounts()
      app.showToast(i18n.t('finances-acc-deleted', 'Account deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function loadArchivedAccounts() {
    try {
      archivedAccounts = await financeApi.fetchArchivedAccounts()
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function unarchiveAccount(id: string) {
    try {
      await financeApi.unarchiveAccount(id)
      await Promise.all([refreshAccounts(), loadArchivedAccounts()])
      app.showToast(i18n.t('finances-acc-restored', 'Account restored'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function openTransfer() {
    editingTransfer = null
    showTransfer = true
  }

  async function refreshAccounts() {
    try {
      accountsData = await financeApi.fetchAccounts()
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function handleAddCategory(name: string, type: 'expense' | 'income') {
    try {
      await financeApi.addCategory(name, type)
      categories = await financeApi.loadCategories()
      app.showToast(i18n.t('finances-cat-added', 'Category added'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function handleDeleteCat(id: string) {
    try {
      await financeApi.deleteCategory(id)
      categories = await financeApi.loadCategories()
      app.showToast(i18n.t('finances-cat-deleted', 'Category deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function handleAccountIconChange(icon: string) {
    if (!selectedAccount) return
    await financeApi.updateAccountIcon(selectedAccount.id, icon)
  }

  let allCategories = $derived<CategoryDto[]>([
    ...(categories?.expense ?? []),
    ...(categories?.income ?? []),
  ])

  // ── Overview chart data ────────────────────────────────────────────────────

  const currentMonthKey = new Date().toISOString().slice(0, 7) // "YYYY-MM"

  let barChartData = $derived.by(() => {
    const now = new Date()
    const locale = app.settings?.preferred_language || 'en'
    const months: string[] = []
    const income: number[] = []
    const expenses: number[] = []
    for (let i = 5; i >= 0; i--) {
      const d = new Date(now.getFullYear(), now.getMonth() - i, 1)
      const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
      const label = d.toLocaleDateString(locale, { month: 'short' })
      const relevant = chartTransactions.filter(tx => !tx.is_transfer && tx.date.startsWith(key))
      const inc = relevant.filter(tx => !tx.is_expense).reduce((s, tx) => s + (parseFloat(tx.amount_raw) || 0), 0)
      const exp = relevant.filter(tx => tx.is_expense).reduce((s, tx) => s + (parseFloat(tx.amount_raw) || 0), 0)
      months.push(label)
      income.push(parseFloat(inc.toFixed(2)))
      expenses.push(parseFloat(exp.toFixed(2)))
    }
    return { months, income, expenses }
  })

  function parseBalanceNum(s: string, currency: string): number {
    // Strip currency code, spaces, and any sign
    const stripped = s.replace(/[^0-9.,]/g, '')
    if (currency.toUpperCase() === 'CLP') {
      // CLP: dot is thousand separator, no decimals
      return parseFloat(stripped.replace(/\./g, '')) || 0
    }
    // Default: comma is thousand separator, dot is decimal
    return parseFloat(stripped.replace(/,/g, '')) || 0
  }

  let donutData = $derived(
    (accountsData?.accounts ?? [])
      .filter(a => !a.balance_negative && !a.is_archived)
      .map(a => ({ name: a.name, value: parseBalanceNum(a.balance, a.currency) }))
      .filter(a => a.value > 0)
  )

  let currentMonthTx = $derived(
    chartTransactions.filter(tx => !tx.is_transfer && tx.date.startsWith(currentMonthKey))
  )
  let monthIncome = $derived(
    currentMonthTx.filter(tx => !tx.is_expense).reduce((s, tx) => s + (parseFloat(tx.amount_raw) || 0), 0)
  )
  let monthExpense = $derived(
    currentMonthTx.filter(tx => tx.is_expense).reduce((s, tx) => s + (parseFloat(tx.amount_raw) || 0), 0)
  )
  let monthNet = $derived(monthIncome - monthExpense)
  let netIsNegative = $derived(monthNet < 0)

  let expenseByCategoryData = $derived.by(() => {
    const map: Record<string, number> = {}
    for (const tx of chartTransactions) {
      if (tx.is_expense && !tx.is_transfer) {
        map[tx.category] = (map[tx.category] ?? 0) + (parseFloat(tx.amount_raw) || 0)
      }
    }
    return Object.entries(map)
      .map(([name, value]) => ({ name, value: parseFloat(value.toFixed(2)) }))
      .sort((a, b) => a.value - b.value)
      .slice(-8)
  })

  function fmtStat(n: number): string {
    return n === 0 ? '—' : n.toFixed(2)
  }

  // ──────────────────────────────────────────────────────────────────────────

  $effect(() => {
    app.settings?.preferred_currency
    loadAll()
  })

  $effect(() => {
    if (activeTab === 'settings') loadArchivedAccounts()
  })
</script>

<div class="page" class:blurred={showAddTransaction || showAddAccount || showTransfer || !!selectedAccount}>
  <!-- Hero -->
  <section class="hero">
    <h2 class="balance" class:negative={accountsData?.total_balance_negative}>
      {accountsData?.total_balance ?? '--'}
    </h2>
    <p class="label">{i18n.t('finances-total-balance', 'Total Balance')}</p>
  </section>

  <!-- Tab selector -->
  <div class="tab-bar">
    <button class:active={activeTab === 'overview'} onclick={() => activeTab = 'overview'}>{i18n.t('finances-tab-overview', 'Overview')}</button>
    <button class:active={activeTab === 'activity'} onclick={() => activeTab = 'activity'}>{i18n.t('finances-tab-activity', 'Activity')}</button>
    <button class:active={activeTab === 'accounts'} onclick={() => activeTab = 'accounts'}>{i18n.t('finances-tab-accounts', 'Accounts')}</button>
    <button class:active={activeTab === 'settings'} onclick={() => activeTab = 'settings'}>{i18n.t('finances-tab-settings', 'Settings')}</button>
  </div>

  {#if loading}
    <div class="skeleton-page">
      <div class="skeleton-row">
        <div class="skeleton" style="flex:1;height:80px;border-radius:var(--radius-md)"></div>
        <div class="skeleton" style="flex:1;height:80px;border-radius:var(--radius-md)"></div>
        <div class="skeleton" style="flex:1;height:80px;border-radius:var(--radius-md)"></div>
      </div>
      <div class="skeleton" style="width:100%;height:220px;border-radius:var(--radius-lg);margin-bottom:14px"></div>
      <div class="skeleton-row">
        <div class="skeleton" style="flex:3;height:160px;border-radius:var(--radius-lg)"></div>
        <div class="skeleton" style="flex:2;height:160px;border-radius:var(--radius-lg)"></div>
      </div>
      <div class="skeleton" style="width:100%;height:40px;border-radius:var(--radius-sm);margin-bottom:8px"></div>
      <div class="skeleton" style="width:100%;height:40px;border-radius:var(--radius-sm);margin-bottom:8px"></div>
      <div class="skeleton" style="width:100%;height:40px;border-radius:var(--radius-sm)"></div>
    </div>

  <!-- OVERVIEW TAB -->
  {:else if activeTab === 'overview'}
    <section class="tab-content">

      <!-- Monthly stat pills -->
      <div class="overview-stats">
        <div class="stat-pill income">
          <span class="pill-label">{i18n.t('finances-income-this-month', 'Income this month')}</span>
          <span class="pill-value">{fmtStat(monthIncome)}</span>
        </div>
        <div class="stat-pill expense">
          <span class="pill-label">{i18n.t('finances-expenses-this-month', 'Expenses this month')}</span>
          <span class="pill-value">{fmtStat(monthExpense)}</span>
        </div>
        <div class="stat-pill net" class:negative-net={netIsNegative}>
          <span class="pill-label">{i18n.t('finances-net-this-month', 'Net this month')}</span>
          <span class="pill-value">{monthNet >= 0 ? '+' : ''}{fmtStat(monthNet)}</span>
        </div>
      </div>

      <!-- Charts row -->
      <div class="charts-row">
        <div class="chart-card">
          <h4>{i18n.t('finances-monthly-overview', 'Monthly Overview')}</h4>
          <FinanceBarChart
            months={barChartData.months}
            income={barChartData.income}
            expenses={barChartData.expenses}
          />
        </div>
        <div class="chart-card">
          <h4>{i18n.t('finances-balance-distribution', 'Balance Distribution')}</h4>
          {#if donutData.length > 0}
            <FinanceDonutChart data={donutData} />
          {:else}
            <p class="empty-chart">{i18n.t('finances-no-positive-balances', 'No positive balances to display')}</p>
          {/if}
        </div>
      </div>

      <!-- Expense by category chart -->
      {#if expenseByCategoryData.length > 0}
        <div class="chart-card chart-card--wide">
          <h4>{i18n.t('finances-expenses-by-category', 'Expenses by Category')}</h4>
          <FinanceCategoryChart data={expenseByCategoryData} />
        </div>
      {/if}

      <!-- Accounts summary -->
      <div class="overview-section">
        <div class="section-header">
          <h3>{i18n.t('finances-accounts', 'Accounts')}</h3>
          <div class="header-actions">
            <button class="glass-btn" onclick={openTransfer}>{i18n.t('finances-transfer', 'Transfer')}</button>
            <button class="glass-btn" onclick={openAddAccount}>{i18n.t('finances-new-account', 'New Account')}</button>
          </div>
        </div>
        {#if (accountsData?.accounts ?? []).length === 0}
          <p class="empty">{i18n.t('finances-no-accounts', 'No accounts yet.')}</p>
        {:else}
          <div class="account-strips">
            {#each accountsData?.accounts ?? [] as acc}
              <button class="account-strip" onclick={() => openAccountDetail(acc.id)}>
                <img
                  src={getAccountDisplayIcon(acc)}
                  alt=""
                  class="strip-icon"
                  class:themed-icon={isGenericIcon(acc.icon_path)}
                  onerror={(e) => (e.target as HTMLImageElement).style.display='none'}
                />
                <div class="strip-info">
                  <span class="strip-name">{acc.name}</span>
                  <span class="strip-type">{accountTypeLabel(acc.account_type_key ?? acc.account_type)}</span>
                </div>
                <div class="strip-balance">
                  <span class:negative={acc.balance_negative}>{acc.balance}</span>
                  <span class="strip-currency">{acc.currency}</span>
                </div>
                <svg class="strip-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M9 18l6-6-6-6"/>
                </svg>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Recent transactions -->
      <div class="overview-section">
        <div class="section-header">
          <h3>{i18n.t('finances-recent-transactions', 'Recent Transactions')}</h3>
          <button class="glass-btn" onclick={() => activeTab = 'activity'}>{i18n.t('finances-view-all', 'View All')}</button>
        </div>
        {#if chartTransactions.length === 0}
          <p class="empty">{i18n.t('finances-no-transactions', 'No transactions yet.')}</p>
        {:else}
          <div class="tx-list">
            {#each chartTransactions.slice(0, 6) as tx}
              <div class="tx-row" role="button" tabindex="0"
                onclick={() => openEditTransaction(tx)}
                onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') openEditTransaction(tx) }}>
                <span class="tx-type-dot" class:expense={tx.is_expense} class:transfer={tx.is_transfer}></span>
                <div class="tx-main">
                  <span class="tx-desc">{tx.description || tx.category}</span>
                  <div class="tx-meta">
                    <span class="tx-cat-badge">{tx.category}</span>
                    <span class="tx-acc">{tx.account_name}</span>
                    <span class="tx-date">{tx.date}</span>
                  </div>
                </div>
                <span class="tx-amount" class:expense={tx.is_expense} class:transfer={tx.is_transfer}>{tx.amount}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    </section>

  <!-- ACTIVITY TAB -->
  {:else if activeTab === 'activity'}
    <section class="tab-content">
      <div class="activity-toolbar">
        <div class="activity-filters">
          <div class="filter-search">
            <svg class="filter-search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
            </svg>
            <input
              type="text"
              placeholder={i18n.t('finances-search-placeholder', 'Search transactions...')}
              bind:value={filterQuery}
              oninput={onSearchInput}
            />
          </div>
          <select bind:value={filterAccountId} onchange={() => loadTransactions()}>
            <option value="">{i18n.t('finances-all-accounts', 'All Accounts')}</option>
            {#each accountsData?.accounts ?? [] as acc}
              <option value={acc.id}>{acc.name}</option>
            {/each}
          </select>
          <select bind:value={filterCategory} onchange={() => loadTransactions()}>
            <option value="">{i18n.t('finances-all-categories', 'All Categories')}</option>
            {#each allCategories as cat}
              <option value={cat.name}>{cat.name}</option>
            {/each}
          </select>
          <select
            bind:value={filterDateRange}
            aria-label={i18n.t('finances-date-range', 'Date range')}
            onchange={() => loadTransactions()}
          >
            <option value="all">{i18n.t('finances-date-all', 'All time')}</option>
            <option value="this-month">{i18n.t('finances-date-this-month', 'This month')}</option>
            <option value="last-30">{i18n.t('finances-date-last-30', 'Last 30 days')}</option>
            <option value="last-90">{i18n.t('finances-date-last-90', 'Last 90 days')}</option>
            <option value="this-year">{i18n.t('finances-date-this-year', 'This year')}</option>
            <option value="custom">{i18n.t('finances-date-custom', 'Custom range')}</option>
          </select>
          {#if filterDateRange === 'custom'}
            <input
              type="date"
              class="date-input"
              aria-label={i18n.t('finances-date-from', 'From')}
              bind:value={filterDateFrom}
              onchange={() => loadTransactions()}
            />
            <input
              type="date"
              class="date-input"
              aria-label={i18n.t('finances-date-to', 'To')}
              bind:value={filterDateTo}
              onchange={() => loadTransactions()}
            />
          {/if}
          {#if hasActiveFilters}
            <button class="clear-btn" onclick={clearFilters}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:12px;height:12px">
                <path d="M18 6L6 18M6 6l12 12"/>
              </svg>
              {i18n.t('finances-clear', 'Clear')}
            </button>
          {/if}
        </div>
        <button class="primary-btn activity-add-btn" onclick={openAddTransaction}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:14px;height:14px">
            <path d="M12 5v14M5 12h14"/>
          </svg>
          {i18n.t('finances-new-entry', 'New Entry')}
        </button>
      </div>

      {#if transactions.length === 0}
        <p class="empty">{hasActiveFilters ? i18n.t('finances-no-matching', 'No matching transactions') : i18n.t('finances-no-transactions-yet', 'No transactions yet')}</p>
      {:else}
        <div class="tx-list">
          {#each transactions as tx}
            <div class="tx-row" role="button" tabindex="0"
              onclick={() => openEditTransaction(tx)}
              onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') openEditTransaction(tx) }}>
              <span class="tx-type-dot" class:expense={tx.is_expense} class:transfer={tx.is_transfer}></span>
              <div class="tx-main">
                <span class="tx-desc">{tx.description || tx.category}</span>
                <div class="tx-meta">
                  <span class="tx-cat-badge">{tx.category}</span>
                  <span class="tx-acc">{tx.account_name}</span>
                  <span class="tx-date">{tx.date}</span>
                </div>
              </div>
              <span class="tx-amount" class:expense={tx.is_expense} class:transfer={tx.is_transfer}>{tx.amount}</span>
              <button class="delete-btn" onclick={(e: MouseEvent) => { e.stopPropagation(); pendingDeleteTx = tx }} aria-label={i18n.t('confirm-delete-button', 'Delete')}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
              </button>
            </div>
          {/each}
        </div>
        {#if hasMore}
          <button class="load-more-btn" onclick={loadMoreTransactions}>{i18n.t('finances-load-more', 'Load More')}</button>
        {/if}
      {/if}
    </section>

  <!-- ACCOUNTS TAB -->
  {:else if activeTab === 'accounts'}
    <section class="tab-content">
      <div class="section-header">
        <h3>{i18n.t('finances-my-accounts', 'My Accounts')}</h3>
        <div class="header-actions">
          <button class="glass-btn" onclick={openTransfer}>{i18n.t('finances-transfer', 'Transfer')}</button>
          <button class="glass-btn" onclick={openAddAccount}>{i18n.t('finances-new-account', 'New Account')}</button>
        </div>
      </div>

      {#if (accountsData?.accounts ?? []).length === 0}
        <p class="empty">{i18n.t('finances-no-accounts-create', 'No accounts yet. Create your first account.')}</p>
      {:else}
        <div class="account-grid">
          {#each accountsData?.accounts ?? [] as acc}
            <button class="account-card" onclick={() => openAccountDetail(acc.id)}>
              <img src={getAccountDisplayIcon(acc)} alt={acc.account_type} class="acc-icon" class:themed-icon={isGenericIcon(acc.icon_path)} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
              <div class="acc-info">
                <div class="acc-name">{acc.name}</div>
                <div class="acc-type">{accountTypeLabel(acc.account_type_key ?? acc.account_type)}</div>
              </div>
              <div class="acc-footer">
                <div class="acc-balance" class:negative={acc.balance_negative}>{acc.balance}</div>
                <div class="acc-currency">{acc.currency}</div>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </section>

  <!-- SETTINGS TAB -->
  {:else if activeTab === 'settings'}
    <FinanceCategoryPanel
      categories={categories}
      onadd={handleAddCategory}
      ondelete={handleDeleteCat}
    />

    {#if archivedAccounts.length > 0}
      <div class="settings-card archived-section">
        <h3 class="settings-card-title">{i18n.t('finances-archived-accounts', 'Archived Accounts')}</h3>
        {#each archivedAccounts as acc}
          <div class="archived-row">
            {#if acc.icon_path}
              <img src={acc.icon_path} alt="" class="archived-icon" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
            {/if}
            <div class="archived-info">
              <span class="archived-name">{acc.name}</span>
              <span class="archived-meta">{accountTypeLabel(acc.account_type_key ?? acc.account_type)} · {acc.currency}</span>
            </div>
            <button class="glass-btn" onclick={() => unarchiveAccount(acc.id)}>
              {i18n.t('finances-restore', 'Restore')}
            </button>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<!-- Account detail panel -->
<FinanceAccountPanel
  show={selectedAccount !== null}
  account={selectedAccount}
  ondelete={deleteAccount}
  onedit={openEditAccount}
  onrefresh={async () => { if (selectedAccount) selectedAccount = await financeApi.fetchAccountDetails(selectedAccount.id); await refreshAccounts() }}
  oniconchange={handleAccountIconChange}
  onclose={() => selectedAccount = null}
/>

<!-- Add/Edit Transaction Modal -->
<FinanceTransactionModal
  bind:show={showAddTransaction}
  editing={editingTransaction}
  accounts={accountsData?.accounts ?? []}
  categories={categories ?? { expense: [], income: [] }}
  onsubmit={async () => { await Promise.all([loadTransactions(), refreshAccounts(), loadChartTransactions()]) }}
  onclose={() => showAddTransaction = false}
/>

<!-- Add/Edit Account Modal -->
<FinanceAccountModal
  bind:show={showAddAccount}
  editing={editingAccount}
  accountsData={accountsData}
  onsubmit={async () => {
    await refreshAccounts()
    if (selectedAccount && editingAccount && selectedAccount.id === editingAccount.id) {
      selectedAccount = await financeApi.fetchAccountDetails(editingAccount.id)
    }
  }}
  onclose={() => { showAddAccount = false; editingAccount = null }}
/>

<!-- Transfer Modal -->
<FinanceTransferModal
  bind:show={showTransfer}
  editing={editingTransfer}
  accounts={accountsData?.accounts ?? []}
  onsubmit={async () => { await Promise.all([loadTransactions(), refreshAccounts(), loadChartTransactions()]) }}
  onclose={() => { showTransfer = false; editingTransfer = null }}
/>

<!-- Delete Transaction Confirm -->
<ConfirmDialog
  show={pendingDeleteTx !== null}
  message={i18n.t('confirm-delete-transaction', 'Are you sure you want to delete this transaction?')}
  detail={pendingDeleteTx ? `${pendingDeleteTx.amount} · ${pendingDeleteTx.description || pendingDeleteTx.category}` : ''}
  danger
  onconfirm={async () => {
    if (pendingDeleteTx) await deleteTransaction(pendingDeleteTx.id)
    pendingDeleteTx = null
  }}
  onclose={() => pendingDeleteTx = null}
/>

<style>
  .page { padding: 24px 32px; max-width: 960px; width: 100%; margin: 0 auto; }

  /* Give every tab's content breathing room from the tab bar */
  .tab-content { padding-top: 20px; }

  .hero { text-align: center; padding: 20px 0 28px; }
  .balance { font-size: 2.4rem; font-weight: 700; color: var(--text-primary); margin: 0; letter-spacing: -0.02em; }
  .balance.negative { color: var(--danger); }
  .label { color: var(--text-tertiary); font-size: 0.78rem; margin-top: 6px; text-transform: uppercase; letter-spacing: 0.08em; }

  .skeleton-page { padding: 8px 0; }
  .empty { text-align: center; padding: 48px; color: var(--text-tertiary); }

  .section-header {
    display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;
  }
  .section-header h3 { font-size: 0.9rem; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.08em; margin: 0; }
  .header-actions { display: flex; gap: 8px; }

  /* Activity toolbar */
  .activity-toolbar {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    margin-bottom: 16px;
    flex-wrap: wrap;
  }
  .activity-filters {
    display: flex;
    gap: 8px;
    flex: 1;
    flex-wrap: wrap;
    align-items: center;
  }
  .filter-search {
    position: relative;
    flex: 1;
    min-width: 160px;
  }
  .filter-search-icon {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    width: 14px;
    height: 14px;
    color: var(--text-tertiary);
    pointer-events: none;
  }
  .filter-search input {
    width: 100%;
    padding: 8px 12px 8px 32px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    color: var(--text-primary);
    font-size: 0.85rem;
    box-sizing: border-box;
    transition: border-color 0.2s;
  }
  .filter-search input:focus {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .activity-filters select {
    padding: 8px 12px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass);
    color: var(--text-primary);
    font-size: 0.82rem;
    transition: border-color 0.2s;
  }
  .activity-filters select:focus {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .activity-filters .date-input {
    padding: 7px 10px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass);
    color: var(--text-primary);
    font-size: 0.82rem;
    transition: border-color 0.2s;
  }
  .activity-filters .date-input:focus {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .clear-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 8px 12px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 0.78rem;
    transition: all 0.15s;
    white-space: nowrap;
  }
  .clear-btn:hover { border-color: var(--danger); color: var(--danger); }
  .activity-add-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* Transaction list */
  .tx-list { display: flex; flex-direction: column; }
  .tx-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 11px 8px;
    border-bottom: 1px solid var(--glass-border);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: background 0.15s;
  }
  .tx-row:last-child { border-bottom: none; }
  .tx-row:hover { background: var(--glass-hover); }

  .tx-type-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--success);
    flex-shrink: 0;
  }
  .tx-type-dot.expense  { background: var(--danger); }
  .tx-type-dot.transfer { background: #60a5fa; }

  .tx-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .tx-desc {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tx-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .tx-cat-badge {
    font-size: 0.67rem;
    padding: 1px 7px;
    border-radius: 20px;
    background: var(--glass-active);
    border: 1px solid var(--glass-border);
    color: var(--text-tertiary);
    white-space: nowrap;
  }
  .tx-acc  { font-size: 0.71rem; color: var(--text-tertiary); }
  .tx-date { font-size: 0.71rem; color: var(--text-tertiary); }
  .tx-amount {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--success);
    white-space: nowrap;
    text-align: right;
    min-width: 80px;
  }
  .tx-amount.expense  { color: var(--danger); }
  .tx-amount.transfer { color: #60a5fa; }

  .delete-btn {
    background: none; border: none; color: var(--text-tertiary); cursor: pointer; padding: 4px;
    display: flex; align-items: center; transition: color 0.15s;
  }
  .delete-btn:hover { color: var(--danger); }
  .delete-btn svg { width: 16px; height: 16px; }

  .load-more-btn {
    display: block; margin: 16px auto; padding: 8px 24px;
    border: 1px solid var(--glass-border); border-radius: var(--radius-sm); background: none;
    color: var(--text-secondary); cursor: pointer; transition: all 0.15s;
  }
  .load-more-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }

  .account-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px;
  }
  .account-card {
    position: relative;
    display: flex; flex-direction: column; gap: 4px; padding: 16px;
    background: var(--card-bg); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    cursor: pointer; text-align: left; color: inherit;
    transition: all 0.2s; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .account-card:hover { border-color: var(--glass-border-hover); box-shadow: var(--glass-shadow-lg); }
  .acc-name { font-weight: 600; color: var(--text-primary); font-size: 0.9rem; }
  .acc-type { font-size: 0.75rem; color: var(--text-tertiary); text-transform: capitalize; }
  .acc-balance { font-size: 1.1rem; font-weight: 600; color: var(--text-primary); margin-top: 8px; }
  .acc-balance.negative { color: var(--danger); }
  .acc-currency { font-size: 0.7rem; color: var(--text-tertiary); }
  /* Accounts */
  .account-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px; margin-bottom: 20px;
  }
  .account-card {
    display: flex; flex-direction: column; gap: 12px; padding: 16px;
    background: var(--glass); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    cursor: pointer; text-align: left; color: inherit; transition: all 0.2s;
    box-shadow: var(--glass-glow);
  }
  .account-card:hover { border-color: var(--glass-border-hover); background: var(--glass-hover); box-shadow: var(--glass-shadow); }
  .acc-icon { width: 32px; height: 32px; border-radius: 4px; }
  .acc-info { display: flex; flex-direction: column; gap: 4px; }
  .acc-name { font-weight: 600; color: var(--text-primary); font-size: 0.95rem; }
  .acc-type { font-size: 0.75rem; color: var(--text-tertiary); text-transform: capitalize; }
  .acc-footer { display: flex; justify-content: space-between; align-items: center; }
  .acc-balance { font-size: 1rem; font-weight: 600; color: var(--text-primary); }
  .acc-balance.negative { color: var(--danger); }
  .acc-currency { font-size: 0.75rem; color: var(--text-tertiary); }

  /* Overview tab */
  .overview-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    margin-bottom: 24px;
  }
  .stat-pill {
    position: relative;
    padding: 18px 20px;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-left-width: 3px;
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 6px;
    box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .stat-pill.income  { border-left-color: #4ade80; }
  .stat-pill.expense { border-left-color: #f87171; }
  .stat-pill.net     { border-left-color: var(--accent); }
  .stat-pill.net.negative-net { border-left-color: #f87171; }
  .pill-label {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .pill-value {
    font-size: 1.25rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }
  .stat-pill.income  .pill-value { color: #4ade80; }
  .stat-pill.expense .pill-value { color: #f87171; }
  .stat-pill.net     .pill-value { color: var(--accent); }
  .stat-pill.net.negative-net .pill-value { color: #f87171; }

  .charts-row {
    display: grid;
    grid-template-columns: 3fr 2fr;
    gap: 14px;
    margin-bottom: 14px;
  }
  .chart-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 16px 16px 6px;
    box-shadow: var(--card-shadow);
    margin-bottom: 14px;
    overflow: hidden;
  }
  .chart-card::before {
    content: ''; position: absolute;
    top: 0; left: 0; right: 0; height: 1px;
    background: var(--card-accent-line); opacity: 0.5;
  }
  .chart-card--wide { grid-column: unset; }
  .chart-card h4 {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin: 0 0 2px;
  }
  .empty-chart {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 220px;
    color: var(--text-tertiary);
    font-size: 0.85rem;
    margin: 0;
  }

  .overview-section { margin-bottom: 28px; }

  .account-strips {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .account-strip {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--glass);
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    width: 100%;
    text-align: left;
    color: inherit;
    transition: all 0.15s;
  }
  .account-strip:hover {
    background: var(--glass-hover);
    border-color: var(--glass-border-hover);
  }
  .strip-icon {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    border-radius: 4px;
  }
  .strip-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .strip-name {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .strip-type {
    font-size: 0.72rem;
    color: var(--text-tertiary);
    text-transform: capitalize;
  }
  .strip-balance {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }
  .strip-balance span {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .strip-balance span.negative { color: var(--danger); }
  .strip-currency {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    font-weight: 400 !important;
  }
  .strip-chevron {
    width: 16px;
    height: 16px;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  /* Generic Lucide icons: invert to white in dark mode, keep dark in light mode */
  .themed-icon { filter: brightness(0) invert(1); }
  :global(.light-mode) .themed-icon { filter: brightness(0); }

  .archived-section { margin-top: 20px; }
  .archived-row {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 0; border-bottom: 1px solid var(--glass-border);
  }
  .archived-row:last-child { border-bottom: none; }
  .archived-icon { width: 22px; height: 22px; object-fit: contain; flex-shrink: 0; }
  .archived-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .archived-name { font-size: 0.9rem; color: var(--text-primary); }
  .archived-meta { font-size: 0.75rem; color: var(--text-tertiary); }
</style>
