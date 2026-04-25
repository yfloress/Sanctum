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
    AccountsResponse, AccountDetailResponse
  } from '../lib/types'
  import FinanceBarChart from '../components/charts/FinanceBarChart.svelte'
  import FinanceDonutChart from '../components/charts/FinanceDonutChart.svelte'
  import FinanceCategoryChart from '../components/charts/FinanceCategoryChart.svelte'

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

  // Accounts state
  let accountsData = $state<AccountsResponse | null>(null)
  let selectedAccount = $state<AccountDetailResponse | null>(null)

  // Categories state
  let categories = $state<CategoriesResponse | null>(null)

  // Modal state
  let showAddTransaction = $state(false)
  let showAddAccount = $state(false)
  let showTransfer = $state(false)
  let editingTransaction = $state<TransactionDto | null>(null)
  let editingAccount = $state<AccountDetailResponse | null>(null)
  let editingTransfer = $state<string | null>(null)
  let showIconPicker = $state(false)

  const ACCOUNT_ICONS: { value: string; src: string; generic: boolean }[] = [
    ...['banco-chile', 'banco-estado', 'bank-of-america', 'bci', 'citibank', 'jpmorgan', 'santander', 'wf']
      .map(n => ({ value: `${n}.svg`, src: `/src/assets/bank-icons/${n}.svg`, generic: false })),
    ...['landmark', 'wallet', 'credit-card', 'piggy-bank', 'briefcase', 'coins', 'banknote', 'building-2']
      .map(n => ({ value: `/src/assets/icons/${n}.svg`, src: `/src/assets/icons/${n}.svg`, generic: true })),
  ]

  function isGenericIcon(iconPath: string | null): boolean {
    if (!iconPath) return true  // fallback is always a generic icon now
    return iconPath.startsWith('/src/assets/icons/')
  }

  // Transaction form
  let txAccountId = $state('')
  let txAmount = $state('')
  let txCategory = $state('')
  let txDescription = $state('')
  let txDate = $state(new Date().toISOString().slice(0, 10))
  let txIsExpense = $state(true)

  // Account form
  let accName = $state('')
  let accType = $state('bank')
  let accCurrency = $state('USD')
  let accInitialBalance = $state('0')
  let accIcon = $state('')
  let showAccIconPicker = $state(false)
  let pickedIconSrc = $state('')
  let pickedIconGeneric = $state(true)
  $effect(() => {
    const found = accIcon ? ACCOUNT_ICONS.find(i => i.value === accIcon) : null
    pickedIconSrc = found ? found.src : getDefaultIconPath(accType)
    pickedIconGeneric = found ? found.generic : true
  })

  // Transfer form
  let tfFromId = $state('')
  let tfToId = $state('')
  let tfAmount = $state('')
  let tfDescription = $state('')
  let tfDate = $state(new Date().toISOString().slice(0, 10))

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
      const res = await financeApi.fetchTransactions(undefined, undefined, undefined, 1000)
      chartTransactions = res.transactions
    } catch (_) {
      // charts degrade gracefully
    }
  }

  async function loadTransactions() {
    try {
      const res = await financeApi.fetchTransactions(
        filterQuery || undefined, filterAccountId || undefined,
        filterCategory || undefined, 100
      )
      transactions = res.transactions
      hasMore = res.has_more
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function loadMoreTransactions() {
    try {
      const res = await financeApi.fetchTransactions(
        filterQuery || undefined, filterAccountId || undefined,
        filterCategory || undefined, transactions.length + 100
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
    txAccountId = accountsData?.accounts[0]?.id ?? ''
    txAmount = ''
    txCategory = ''
    txDescription = ''
    txDate = new Date().toISOString().slice(0, 10)
    txIsExpense = true
    showAddTransaction = true
  }

  function openEditTransaction(tx: TransactionDto) {
    if (tx.is_transfer) {
      openEditTransfer(tx)
      return
    }
    editingTransaction = tx
    txAccountId = tx.account_id
    txAmount = tx.amount_raw
    txCategory = tx.category_raw
    txDescription = tx.description
    txDate = tx.date
    txIsExpense = tx.is_expense
    showAddTransaction = true
  }

  function openEditTransfer(tx: TransactionDto) {
    editingTransfer = tx.id
    tfFromId = tx.account_id
    tfToId = tx.transfer_account_id ?? ''
    tfAmount = tx.amount_raw
    tfDescription = ''
    tfDate = tx.date
    showTransfer = true
  }

  async function submitTransaction() {
    try {
      if (editingTransaction) {
        await financeApi.updateTransaction(
          editingTransaction.id, txAccountId, txAmount, txCategory, txDescription, txDate, txIsExpense
        )
      } else {
        await financeApi.addTransaction(
          txAccountId, txAmount, txCategory, txDescription, txDate, txIsExpense
        )
      }
      showAddTransaction = false
      await Promise.all([loadTransactions(), refreshAccounts(), loadChartTransactions()])
      app.showToast(editingTransaction ? i18n.t('finances-tx-updated', 'Transaction updated') : i18n.t('finances-tx-added', 'Transaction added'))
    } catch (e) {
      app.showToast(String(e), true)
    }
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
    accName = ''
    accType = 'bank'
    accCurrency = 'USD'
    accInitialBalance = '0'
    accIcon = ''
    showAccIconPicker = false
    showAddAccount = true
  }

  function openEditAccount(detail: AccountDetailResponse) {
    editingAccount = detail
    accName = detail.name
    accType = detail.account_type
    accCurrency = detail.currency
    const fullAcc = accountsData?.accounts.find(a => a.id === detail.id)
    accInitialBalance = fullAcc?.initial_balance ?? '0'
    showAddAccount = true
  }

  async function submitAccount() {
    try {
      const isEditing = !!editingAccount
      if (isEditing) {
        await financeApi.updateAccount(editingAccount!.id, accName, accType, accCurrency, accInitialBalance)
        await refreshAccounts()
        if (selectedAccount?.id === editingAccount!.id) {
          selectedAccount = await financeApi.fetchAccountDetails(editingAccount!.id)
        }
      } else {
        const before = new Set(accountsData?.accounts.map(a => a.id) ?? [])
        await financeApi.createAccount(accName, accType, accCurrency, accInitialBalance)
        await refreshAccounts()
        if (accIcon) {
          const newAcc = accountsData?.accounts.find(a => !before.has(a.id))
          if (newAcc) {
            await financeApi.updateAccountIcon(newAcc.id, accIcon)
            await refreshAccounts()
          }
        }
      }
      showAddAccount = false
      editingAccount = null
      app.showToast(isEditing ? i18n.t('finances-acc-updated', 'Account updated') : i18n.t('finances-acc-created', 'Account created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
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

  function openTransfer() {
    editingTransfer = null
    tfFromId = accountsData?.accounts[0]?.id ?? ''
    tfToId = accountsData?.accounts[1]?.id ?? ''
    tfAmount = ''
    tfDescription = ''
    tfDate = new Date().toISOString().slice(0, 10)
    showTransfer = true
  }

  async function submitTransfer() {
    const isEditing = !!editingTransfer
    try {
      if (isEditing) {
        await financeApi.updateTransfer({
          id: editingTransfer!,
          from_account_id: tfFromId,
          to_account_id: tfToId,
          amount: tfAmount,
          description: tfDescription,
          date: tfDate,
        })
      } else {
        await financeApi.transferFunds({
          from_account_id: tfFromId,
          to_account_id: tfToId,
          amount: tfAmount,
          description: tfDescription,
          date: tfDate,
        })
      }
      showTransfer = false
      editingTransfer = null
      await Promise.all([loadTransactions(), refreshAccounts(), loadChartTransactions()])
      app.showToast(isEditing ? i18n.t('finances-tf-updated', 'Transfer updated') : i18n.t('finances-tf-completed', 'Transfer completed'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function changeAccountIcon(icon: string) {
    if (!selectedAccount) return
    try {
      await financeApi.updateAccountIcon(selectedAccount.id, icon)
      selectedAccount = await financeApi.fetchAccountDetails(selectedAccount.id)
      await refreshAccounts()
      showIconPicker = false
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function refreshAccounts() {
    try {
      accountsData = await financeApi.fetchAccounts()
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  // Category management
  let newCatName = $state('')
  let newCatType = $state<'expense' | 'income'>('expense')

  async function addCategory() {
    if (!newCatName.trim()) return
    try {
      await financeApi.addCategory(newCatName, newCatType)
      newCatName = ''
      categories = await financeApi.loadCategories()
      app.showToast(i18n.t('finances-cat-added', 'Category added'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteCat(id: string) {
    try {
      await financeApi.deleteCategory(id)
      categories = await financeApi.loadCategories()
      app.showToast(i18n.t('finances-cat-deleted', 'Category deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  let allCategories = $derived<CategoryDto[]>([
    ...(categories?.expense ?? []),
    ...(categories?.income ?? []),
  ])

  let txCategoryOptions = $derived<CategoryDto[]>(
    txIsExpense ? (categories?.expense ?? []) : (categories?.income ?? [])
  )

  // ── Overview chart data ────────────────────────────────────────────────────

  const currentMonthKey = new Date().toISOString().slice(0, 7) // "YYYY-MM"

  let barChartData = $derived.by(() => {
    const now = new Date()
    const months: string[] = []
    const income: number[] = []
    const expenses: number[] = []
    for (let i = 5; i >= 0; i--) {
      const d = new Date(now.getFullYear(), now.getMonth() - i, 1)
      const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
      const label = d.toLocaleDateString('en', { month: 'short' })
      const relevant = chartTransactions.filter(tx => !tx.is_transfer && tx.date.startsWith(key))
      const inc = relevant.filter(tx => !tx.is_expense).reduce((s, tx) => s + (parseFloat(tx.amount_raw) || 0), 0)
      const exp = relevant.filter(tx => tx.is_expense).reduce((s, tx) => s + (parseFloat(tx.amount_raw) || 0), 0)
      months.push(label)
      income.push(parseFloat(inc.toFixed(2)))
      expenses.push(parseFloat(exp.toFixed(2)))
    }
    return { months, income, expenses }
  })

  function parseBalanceNum(s: string): number {
    return parseFloat(s.replace(/[^0-9.]/g, '')) || 0
  }

  let donutData = $derived(
    (accountsData?.accounts ?? [])
      .filter(a => !a.balance_negative && !a.is_archived)
      .map(a => ({ name: a.name, value: parseBalanceNum(a.balance) }))
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

  function getDefaultIconPath(accountType: string): string {
    const iconMap: { [key: string]: string } = {
      'savings': 'piggy-bank',
      'credit': 'credit-card',
      'credit_card': 'credit-card',
      'cash': 'wallet',
      'bank': 'landmark',
      'other': 'coins',
    }
    const icon = iconMap[accountType.toLowerCase()] || 'landmark'
    return `/src/assets/icons/${icon}.svg`
  }

  function getAccountDisplayIcon(acc: { account_type: string; account_type_key?: string; icon_path: string | null }): string {
    if (acc.icon_path) {
      if (acc.icon_path.startsWith('/') || acc.icon_path.startsWith('http')) return acc.icon_path
      return `/src/assets/bank-icons/${acc.icon_path}`
    }
    return getDefaultIconPath(acc.account_type_key ?? acc.account_type)
  }

  $effect(() => {
    app.settings?.preferred_currency
    loadAll()
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
                  <span class="strip-type">{acc.account_type}</span>
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
              oninput={() => loadTransactions()}
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
          {#if filterQuery || filterAccountId || filterCategory}
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
        <p class="empty">{filterQuery || filterAccountId || filterCategory ? i18n.t('finances-no-matching', 'No matching transactions') : i18n.t('finances-no-transactions-yet', 'No transactions yet')}</p>
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
              <button class="delete-btn" onclick={(e: MouseEvent) => { e.stopPropagation(); deleteTransaction(tx.id) }} aria-label="Delete">
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
                <div class="acc-type">{acc.account_type}</div>
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
    <section class="tab-content">

      <!-- Add category card -->
      <div class="settings-card">
        <span class="settings-card-label">{i18n.t('finances-new-category', 'New Category')}</span>
        <div class="cat-add-row">
          <input
            class="cat-name-input"
            type="text"
            placeholder={i18n.t('finances-category-placeholder', 'Category name...')}
            bind:value={newCatName}
            onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter' && newCatName.trim()) addCategory() }}
          />
          <div class="toggle-row cat-type-toggle">
            <button class="toggle-btn" class:active={newCatType === 'expense'} onclick={() => newCatType = 'expense'}>{i18n.t('finances-expense', 'Expense')}</button>
            <button class="toggle-btn" class:active={newCatType === 'income'} onclick={() => newCatType = 'income'}>{i18n.t('finances-income', 'Income')}</button>
          </div>
          <button class="primary-btn" onclick={addCategory} disabled={!newCatName.trim()}>{i18n.t('finances-add', 'Add')}</button>
        </div>
      </div>

      {#if categories}
        <div class="cat-columns">

          <!-- Expense column -->
          <div class="cat-col">
            <div class="cat-col-header">
              <span class="cat-col-dot cat-col-dot--expense"></span>
              <h4>{i18n.t('finances-expense', 'Expense')}</h4>
              <span class="cat-count">{categories.expense.length}</span>
            </div>
            <div class="cat-chips">
              {#each categories.expense as cat}
                <div class="cat-chip" class:cat-chip--default={cat.is_default}>
                  <span class="cat-chip-name">{cat.name}</span>
                  {#if cat.is_default}
                    <svg class="cat-chip-lock" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/>
                    </svg>
                  {:else}
                    <button class="cat-chip-del" onclick={() => deleteCat(cat.id)} aria-label="Delete {cat.name}">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                        <path d="M18 6L6 18M6 6l12 12"/>
                      </svg>
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
          </div>

          <!-- Income column -->
          <div class="cat-col">
            <div class="cat-col-header">
              <span class="cat-col-dot cat-col-dot--income"></span>
              <h4>{i18n.t('finances-income', 'Income')}</h4>
              <span class="cat-count">{categories.income.length}</span>
            </div>
            <div class="cat-chips">
              {#each categories.income as cat}
                <div class="cat-chip" class:cat-chip--default={cat.is_default}>
                  <span class="cat-chip-name">{cat.name}</span>
                  {#if cat.is_default}
                    <svg class="cat-chip-lock" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/>
                    </svg>
                  {:else}
                    <button class="cat-chip-del" onclick={() => deleteCat(cat.id)} aria-label="Delete {cat.name}">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                        <path d="M18 6L6 18M6 6l12 12"/>
                      </svg>
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
          </div>

        </div>
      {/if}
    </section>
  {/if}
</div>

<!-- Account detail panel -->
{#if selectedAccount}
  <div class="overlay-backdrop" role="presentation" onclick={() => selectedAccount = null} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') selectedAccount = null }}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      <h3>{selectedAccount.name}</h3>
      <button class="close-panel" aria-label="Close panel" onclick={() => selectedAccount = null}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
    <div class="panel-info">
      <div class="info-row panel-icon-row">
        <img src={getAccountDisplayIcon(selectedAccount)} alt="" class="panel-acc-icon" class:themed-icon={isGenericIcon(selectedAccount.icon_path)} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
        <button class="change-icon-btn" onclick={() => showIconPicker = !showIconPicker}>
          {showIconPicker ? i18n.t('finances-close', 'Close') : i18n.t('finances-change-icon', 'Change Icon')}
        </button>
      </div>
      {#if showIconPicker}
        <div class="icon-picker">
          {#each ACCOUNT_ICONS as icon}
            <button class="icon-option" onclick={() => changeAccountIcon(icon.value)} title={icon.value}>
              <img src={icon.src} alt={icon.value} class:themed-icon={icon.generic} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
            </button>
          {/each}
          <button class="icon-option icon-reset" onclick={() => changeAccountIcon('')} title="Default">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 12l9-9 9 9M5 10v9a1 1 0 001 1h4v-5h4v5h4a1 1 0 001-1v-9"/></svg>
          </button>
        </div>
      {/if}
      <div class="info-row"><span>{i18n.t('finances-type', 'Type')}</span><span>{selectedAccount.account_type}</span></div>
      <div class="info-row"><span>{i18n.t('finances-currency', 'Currency')}</span><span>{selectedAccount.currency}</span></div>
      <div class="info-row">
        <span>{i18n.t('finances-balance', 'Balance')}</span>
        <span class:negative={selectedAccount.balance_negative}>{selectedAccount.balance}</span>
      </div>
    </div>
    {#if selectedAccount.transactions.length > 0}
      <h4>{i18n.t('finances-recent-transactions', 'Recent Transactions')}</h4>
      <div class="panel-tx-list">
        {#each selectedAccount.transactions as tx}
          <div class="panel-tx">
            <span class="tx-date">{tx.date}</span>
            <span class="tx-desc">{tx.description}</span>
            <span class="tx-amount" class:expense={tx.is_expense}>{tx.amount}</span>
          </div>
        {/each}
      </div>
    {/if}
    <div class="panel-actions">
      <button class="primary-btn" onclick={() => openEditAccount(selectedAccount!)}>{i18n.t('finances-edit-account', 'Edit Account')}</button>
      <button class="danger-btn" onclick={() => deleteAccount(selectedAccount!.id)}>{i18n.t('finances-delete-account', 'Delete Account')}</button>
    </div>
  </aside>
{/if}

<!-- Add/Edit Transaction Modal -->
{#if showAddTransaction}
  <div class="modal-backdrop" role="presentation" onclick={() => showAddTransaction = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddTransaction = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
    <h3>{editingTransaction ? i18n.t('finances-edit-transaction', 'Edit Transaction') : i18n.t('finances-add-transaction', 'Add Transaction')}</h3>
    <div class="form-grid">
      <label>
        {i18n.t('finances-account', 'Account')}
        <select bind:value={txAccountId}>
          {#each accountsData?.accounts ?? [] as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
      </label>
      <label>
        {i18n.t('finances-amount', 'Amount')}
        <input type="text" bind:value={txAmount} placeholder="0.00" />
      </label>
      <label>
        {i18n.t('finances-type', 'Type')}
        <div class="toggle-row">
          <button class="toggle-btn" class:active={txIsExpense} onclick={() => txIsExpense = true}>{i18n.t('finances-expense', 'Expense')}</button>
          <button class="toggle-btn" class:active={!txIsExpense} onclick={() => txIsExpense = false}>{i18n.t('finances-income', 'Income')}</button>
        </div>
      </label>
      <label>
        {i18n.t('finances-category', 'Category')}
        <select bind:value={txCategory}>
          <option value="">{i18n.t('finances-select', 'Select...')}</option>
          {#each txCategoryOptions as cat}
            <option value={cat.name}>{cat.name}</option>
          {/each}
        </select>
      </label>
      <label>
        {i18n.t('finances-description', 'Description')}
        <input type="text" bind:value={txDescription} placeholder={i18n.t('finances-description', 'Description')} />
      </label>
      <label>
        {i18n.t('finances-date', 'Date')}
        <input type="date" bind:value={txDate} />
      </label>
    </div>
    <div class="modal-actions">
      <button class="secondary-btn" onclick={() => showAddTransaction = false}>{i18n.t('finances-cancel', 'Cancel')}</button>
      <button class="primary-btn" onclick={submitTransaction} disabled={!txAmount || !txAccountId}>
        {editingTransaction ? i18n.t('finances-update', 'Update') : i18n.t('finances-add-btn', 'Add')}
      </button>
    </div>
    </div>
  </div>
{/if}

<!-- Add Account Modal -->
{#if showAddAccount}
  <div class="modal-backdrop" role="presentation" onclick={() => showAddAccount = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddAccount = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
        <h3>{editingAccount ? i18n.t('finances-edit-account-modal', 'Edit Account') : i18n.t('finances-new-account-modal', 'New Account')}</h3>
    <div class="form-grid">
      <label>
        {i18n.t('finances-name', 'Name')}
        <input type="text" bind:value={accName} placeholder={i18n.t('finances-account-name-placeholder', 'Account name')} />
      </label>
      <label>
        {i18n.t('finances-type', 'Type')}
        <select bind:value={accType}>
          <option value="bank">{i18n.t('finances-account-type-bank', 'Bank')}</option>
          <option value="savings">{i18n.t('finances-account-type-savings', 'Savings')}</option>
          <option value="credit">{i18n.t('finances-account-type-credit', 'Credit Card')}</option>
          <option value="cash">{i18n.t('finances-account-type-cash', 'Cash')}</option>
          <option value="other">{i18n.t('finances-account-type-other', 'Other')}</option>
        </select>
      </label>
      <label>
        {i18n.t('finances-currency', 'Currency')}
        <select bind:value={accCurrency}>
          {#each ['USD', 'CLP', 'EUR', 'GBP', 'BRL', 'MXN', 'ARS', 'CAD', 'AUD', 'CHF', 'JPY'] as cur}
            <option value={cur}>{cur}</option>
          {/each}
        </select>
      </label>
      <label>
        {i18n.t('finances-initial-balance', 'Initial Balance')}
        <input type="text" bind:value={accInitialBalance} placeholder="0.00" />
      </label>
      {#if !editingAccount}
        <div class="icon-select-label">
          <span>{i18n.t('finances-icon', 'Icon')}</span>
          <button class="change-icon-btn" onclick={() => showAccIconPicker = !showAccIconPicker}>
            <img
              src={pickedIconSrc || getDefaultIconPath(accType)}
              alt=""
              class="selected-icon-preview"
              class:themed-icon={pickedIconGeneric}
            />
            {showAccIconPicker ? i18n.t('finances-close', 'Close') : i18n.t('finances-change', 'Change')}
          </button>
        </div>
        {#if showAccIconPicker}
          <div class="icon-picker">
            {#each ACCOUNT_ICONS as icon}
              <button class="icon-option" class:selected={accIcon === icon.value} onclick={() => { accIcon = icon.value; showAccIconPicker = false }} title={icon.value}>
                <img src={icon.src} alt={icon.value} class:themed-icon={icon.generic} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
              </button>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showAddAccount = false}>{i18n.t('finances-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitAccount} disabled={!accName.trim()}>
          {editingAccount ? i18n.t('finances-update', 'Update') : i18n.t('finances-create', 'Create')}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Transfer Modal -->
{#if showTransfer}
  <div class="modal-backdrop" role="presentation" onclick={() => { showTransfer = false; editingTransfer = null }} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') { showTransfer = false; editingTransfer = null } }}></div>
  <div class="modal-wrapper">
    <div class="modal">
        <h3>{editingTransfer ? i18n.t('finances-edit-transfer', 'Edit Transfer') : i18n.t('finances-transfer-funds', 'Transfer Funds')}</h3>
    <div class="form-grid">
      <label>
        {i18n.t('finances-from', 'From')}
        <select bind:value={tfFromId}>
          {#each accountsData?.accounts ?? [] as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
      </label>
      <label>
        {i18n.t('finances-to', 'To')}
        <select bind:value={tfToId}>
          {#each accountsData?.accounts ?? [] as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
      </label>
      <label>
        {i18n.t('finances-amount', 'Amount')}
        <input type="text" bind:value={tfAmount} placeholder="0.00" />
      </label>
      <label>
        {i18n.t('finances-description', 'Description')}
        <input type="text" bind:value={tfDescription} placeholder={i18n.t('finances-transfer-note', 'Transfer note')} />
      </label>
      <label>
        {i18n.t('finances-date', 'Date')}
        <input type="date" bind:value={tfDate} />
      </label>
    </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => { showTransfer = false; editingTransfer = null }}>{i18n.t('finances-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitTransfer} disabled={!tfAmount || tfFromId === tfToId}>
          {editingTransfer ? i18n.t('finances-update', 'Update') : i18n.t('finances-transfer-btn', 'Transfer')}
        </button>
      </div>
    </div>
  </div>
{/if}

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

  /* Settings tab */
  .settings-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 22px;
    margin-bottom: 20px;
    box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .settings-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.6;
  }
  .settings-card-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.68rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    font-weight: 600;
    margin-bottom: 14px;
  }
  .settings-card-label::before {
    content: '';
    width: 3px;
    height: 12px;
    border-radius: 2px;
    background: linear-gradient(180deg, var(--accent) 0%, var(--accent-hover) 100%);
    box-shadow: 0 0 6px var(--accent-glow);
  }
  .cat-add-row {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
  }
  .cat-name-input {
    flex: 1;
    min-width: 140px;
    padding: 9px 12px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass-active);
    color: var(--text-primary);
    font-size: 0.875rem;
    transition: border-color 0.2s;
  }
  .cat-name-input:focus {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .cat-type-toggle { flex-shrink: 0; }

  .cat-columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .cat-col {
    position: relative;
    background: var(--card-bg);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    padding: 18px;
    box-shadow: var(--card-shadow);
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .cat-col:hover {
    border-color: var(--glass-border-hover);
    box-shadow: var(--glass-shadow-lg), inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }
  .cat-col-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
  }
  .cat-col-header h4 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    flex: 1;
  }
  .cat-col-dot {
    width: 8px; height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .cat-col-dot--expense { background: var(--danger); }
  .cat-col-dot--income  { background: var(--success); }
  .cat-count {
    font-size: 0.68rem;
    color: var(--text-tertiary);
    background: var(--glass-active);
    border: 1px solid var(--glass-border);
    border-radius: 20px;
    padding: 1px 8px;
  }
  .cat-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .cat-chip {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 8px 5px 12px;
    background: var(--glass-active);
    border: 1px solid var(--glass-border);
    border-radius: 20px;
    transition: border-color 0.15s;
  }
  .cat-chip--default {
    border-color: rgba(168, 85, 247, 0.2);
    background: rgba(168, 85, 247, 0.06);
    padding-right: 12px;
  }
  .cat-chip-name {
    font-size: 0.8rem;
    color: var(--text-primary);
  }
  .cat-chip-del {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    display: flex;
    color: var(--text-tertiary);
    transition: color 0.15s;
    line-height: 0;
  }
  .cat-chip-del:hover { color: var(--danger); }
  .cat-chip-del svg { width: 11px; height: 11px; }
  .cat-chip-lock { width: 11px; height: 11px; color: var(--accent); flex-shrink: 0; }

  /* Overlay & detail panel */
  .overlay-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.3); z-index: 50;
  }
  .detail-panel {
    position: fixed; top: 0; right: 0; bottom: 0; width: 380px;
    background: linear-gradient(180deg, rgba(22, 22, 28, 0.88) 0%, rgba(16, 16, 20, 0.85) 100%);
    border-left: 1px solid rgba(255, 255, 255, 0.08); z-index: 51;
    padding: 24px; overflow-y: auto;
    box-shadow: var(--glass-shadow-lg);
    animation: slideInRight 0.25s ease;
  }
  @keyframes slideInRight { from { transform: translateX(100%); } to { transform: translateX(0); } }

  .panel-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
  .panel-header h3 { margin: 0; color: var(--text-primary); }
  .close-panel { background: none; border: none; color: var(--text-tertiary); cursor: pointer; padding: 4px; display: flex; transition: color 0.15s; }
  .close-panel:hover { color: var(--text-primary); }
  .close-panel svg { width: 20px; height: 20px; }

  .panel-info { display: flex; flex-direction: column; gap: 8px; margin-bottom: 20px; }
  .info-row { display: flex; justify-content: space-between; font-size: 0.85rem; color: var(--text-secondary); }
  .info-row .negative { color: var(--danger); }

  .detail-panel h4 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 8px; }
  .panel-tx-list { display: flex; flex-direction: column; gap: 4px; }
  .panel-tx { display: grid; grid-template-columns: 70px 1fr auto; gap: 8px; font-size: 0.8rem; padding: 6px 0; border-bottom: 1px solid var(--glass-border); }
  .panel-actions { margin-top: 24px; display: flex; gap: 8px; }

  /* Modals */
  .modal-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.5);
    backdrop-filter: blur(4px); z-index: 100;
  }
  .modal {
    position: relative;
    background: linear-gradient(145deg, rgba(26, 26, 31, 0.75) 0%, rgba(20, 20, 24, 0.72) 50%, rgba(17, 17, 21, 0.7) 100%);
    border: 1px solid rgba(255, 255, 255, 0.1); border-radius: var(--radius-lg);
    padding: 28px; width: 420px; max-height: 85vh; overflow-y: auto; z-index: 101;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    box-shadow: inset 0 0.125em 0.125em rgba(254, 254, 254, 0.05), inset 0 -0.125em 0.125em rgba(0, 0, 0, 0.5), 0 0.25em 0.125em -0.125em rgba(254, 254, 254, 0.2), 0 0 0.1em 0.25em inset rgba(0, 0, 0, 0.2);
  }
  .modal-wrapper {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    z-index: 101; pointer-events: none;
  }
  .modal-wrapper .modal {
    pointer-events: auto;
  }
  .modal h3 { margin: 0 0 20px; color: var(--text-primary); font-size: 1.1rem; position: relative; z-index: 10; }

  .form-grid { display: flex; flex-direction: column; gap: 14px; position: relative; z-index: 10; }
  .form-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: var(--text-secondary); }
  .form-grid input, .form-grid select {
    padding: 10px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.9rem;
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .form-grid input:focus, .form-grid select:focus {
    border-color: var(--accent); outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .toggle-row { display: flex; gap: 4px; }
  .toggle-btn {
    flex: 1; padding: 8px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .toggle-btn.active {
    background: var(--glass-active); color: var(--text-primary);
    border-color: var(--accent-border); box-shadow: 0 0 0 1px var(--accent-glow) inset;
  }

  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; position: relative; z-index: 10; }

  .primary-btn {
    padding: 8px 18px; border: 1px solid var(--accent-border); border-radius: var(--radius-sm);
    background: var(--accent-bg); backdrop-filter: blur(8px);
    color: var(--text-on-accent); cursor: pointer; font-size: 0.85rem; font-weight: 500;
    transition: all 0.2s;
  }
  .primary-btn:hover:not(:disabled) {
    background: var(--accent); border-color: var(--accent);
    box-shadow: 0 0 16px var(--accent-glow);
  }
  .primary-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  .secondary-btn {
    padding: 8px 18px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .secondary-btn:hover { border-color: var(--glass-border-hover); }

  .danger-btn {
    padding: 8px 18px; border: 1px solid rgba(248, 113, 113, 0.2); border-radius: var(--radius-sm);
    background: rgba(248, 113, 113, 0.08); color: var(--danger); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .danger-btn:hover { background: rgba(248, 113, 113, 0.15); border-color: rgba(248, 113, 113, 0.3); }

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

  /* Icon picker */
  .panel-icon-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
  .panel-acc-icon { width: 36px; height: 36px; border-radius: 6px; }
  .change-icon-btn {
    background: none; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    color: var(--text-secondary); cursor: pointer; font-size: 0.75rem; padding: 4px 10px;
    transition: all 0.15s;
  }
  .change-icon-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .icon-picker {
    display: flex; flex-wrap: wrap; gap: 6px; padding: 10px;
    background: var(--glass); border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    margin-bottom: 8px;
  }
  .icon-option {
    width: 36px; height: 36px; border-radius: 6px; border: 1px solid var(--glass-border);
    background: var(--glass-hover); cursor: pointer; display: flex; align-items: center; justify-content: center;
    padding: 4px; transition: all 0.15s;
  }
  .icon-option:hover { border-color: var(--accent-border); background: var(--glass-active); }
  .icon-option img { width: 100%; height: 100%; object-fit: contain; border-radius: 3px; }
  .icon-option.icon-reset svg { width: 18px; height: 18px; color: var(--text-tertiary); }

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

  /* Icon selector in New Account modal */
  .icon-select-label {
    display: flex; justify-content: space-between; align-items: center;
    font-size: 0.8rem; color: var(--text-secondary);
  }
  .selected-icon-preview { width: 20px; height: 20px; margin-right: 6px; vertical-align: middle; }
  .icon-option.selected { border-color: var(--accent-border); background: var(--glass-active); }
</style>
