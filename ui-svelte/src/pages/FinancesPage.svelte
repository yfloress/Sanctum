<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import LiquidGlassButton from '../components/LiquidGlassButton.svelte'
  import LiquidGlassTab from '../components/LiquidGlassTab.svelte'
  import LiquidGlassBackground from '../components/LiquidGlassBackground.svelte'
  import * as financeApi from '../lib/api/finance'
  import type {
    TransactionDto, CategoriesResponse, CategoryDto,
    AccountsResponse, AccountDetailResponse
  } from '../lib/types'

  type Tab = 'activity' | 'accounts' | 'settings'

  let activeTab = $state<Tab>('activity')
  let loading = $state(true)

  // Activity state
  let transactions = $state<TransactionDto[]>([])
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

  // Transaction form
  let txAccountId = $state('')
  let txAmount = $state('')
  let txCategory = $state('')
  let txDescription = $state('')
  let txDate = $state(new Date().toISOString().slice(0, 10))
  let txIsExpense = $state(true)

  // Account form
  let accName = $state('')
  let accType = $state('checking')
  let accCurrency = $state('USD')
  let accInitialBalance = $state('0')

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
      await loadTransactions()
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      loading = false
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
    editingTransaction = tx
    txAccountId = tx.account_id
    txAmount = tx.amount_raw
    txCategory = tx.category_raw
    txDescription = tx.description
    txDate = tx.date
    txIsExpense = tx.is_expense
    showAddTransaction = true
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
      await Promise.all([loadTransactions(), refreshAccounts()])
      app.showToast(editingTransaction ? 'Transaction updated' : 'Transaction added')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteTransaction(id: string) {
    try {
      await financeApi.deleteTransaction(id)
      await Promise.all([loadTransactions(), refreshAccounts()])
      app.showToast('Transaction deleted')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function openAddAccount() {
    accName = ''
    accType = 'checking'
    accCurrency = 'USD'
    accInitialBalance = '0'
    showAddAccount = true
  }

  async function submitAccount() {
    try {
      await financeApi.createAccount(accName, accType, accCurrency, accInitialBalance)
      showAddAccount = false
      await refreshAccounts()
      app.showToast('Account created')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteAccount(id: string) {
    try {
      await financeApi.deleteAccount(id)
      selectedAccount = null
      await refreshAccounts()
      app.showToast('Account deleted')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function openTransfer() {
    tfFromId = accountsData?.accounts[0]?.id ?? ''
    tfToId = accountsData?.accounts[1]?.id ?? ''
    tfAmount = ''
    tfDescription = ''
    tfDate = new Date().toISOString().slice(0, 10)
    showTransfer = true
  }

  async function submitTransfer() {
    try {
      await financeApi.transferFunds({
        from_account_id: tfFromId,
        to_account_id: tfToId,
        amount: tfAmount,
        description: tfDescription,
        date: tfDate,
      })
      showTransfer = false
      await Promise.all([loadTransactions(), refreshAccounts()])
      app.showToast('Transfer completed')
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
      app.showToast('Category added')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteCat(id: string) {
    try {
      await financeApi.deleteCategory(id)
      categories = await financeApi.loadCategories()
      app.showToast('Category deleted')
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

  function getBankIconPath(accountType: string): string {
    const iconMap: { [key: string]: string } = {
      'checking': 'banco-chile.svg',
      'savings': 'banco-estado.svg',
      'credit': 'citibank.svg',
      'cash': 'wf.svg',
      'investment': 'jpmorgan.svg',
    }
    const icon = iconMap[accountType.toLowerCase()] || 'bank-of-america.svg'
    return `/src/assets/bank-icons/${icon}`
  }

  $effect(() => { loadAll() })
</script>

<div class="page">
  <!-- Hero -->
  <section class="hero">
    <h2 class="balance" class:negative={accountsData?.total_balance_negative}>
      {accountsData?.total_balance ?? '--'}
    </h2>
    <p class="label">Total Balance</p>
  </section>

  <!-- Tab selector -->
  <LiquidGlassTab
    options={[
      { label: 'Activity', value: 'activity' },
      { label: 'Accounts', value: 'accounts' },
      { label: 'Settings', value: 'settings' }
    ]}
    active={activeTab}
    onchange={(value) => activeTab = value as Tab}
  />

  {#if loading}
    <div class="loading">Loading...</div>

  <!-- ACTIVITY TAB -->
  {:else if activeTab === 'activity'}
    <section class="tab-content">
      <div class="section-header">
        <h3>Transactions</h3>
        <LiquidGlassButton text="New Entry" contrast="dark" onclick={openAddTransaction} />
      </div>

      <div class="filters">
        <input
          type="text"
          placeholder="Search..."
          bind:value={filterQuery}
          oninput={() => loadTransactions()}
        />
        <select bind:value={filterAccountId} onchange={() => loadTransactions()}>
          <option value="">All Accounts</option>
          {#each accountsData?.accounts ?? [] as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
        <select bind:value={filterCategory} onchange={() => loadTransactions()}>
          <option value="">All Categories</option>
          {#each allCategories as cat}
            <option value={cat.name}>{cat.name}</option>
          {/each}
        </select>
        {#if filterQuery || filterAccountId || filterCategory}
          <button class="clear-btn" onclick={clearFilters}>Clear</button>
        {/if}
      </div>

      {#if transactions.length === 0}
        <p class="empty">{filterQuery || filterAccountId || filterCategory ? 'No matching transactions' : 'No transactions yet'}</p>
      {:else}
        <div class="tx-list">
          {#each transactions as tx}
            <div class="tx-row" role="button" tabindex="0" onclick={() => openEditTransaction(tx)} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') openEditTransaction(tx) }}>
              <span class="tx-date">{tx.date}</span>
              <span class="tx-desc">{tx.description}</span>
              <span class="tx-cat">{tx.category}</span>
              <span class="tx-acc">{tx.account_name}</span>
              <span class="tx-amount" class:expense={tx.is_expense} class:transfer={tx.is_transfer}>
                {tx.amount}
              </span>
              <button class="delete-btn" onclick={(e: MouseEvent) => { e.stopPropagation(); deleteTransaction(tx.id) }} aria-label="Delete">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
              </button>
            </div>
          {/each}
        </div>
        {#if hasMore}
          <button class="load-more-btn" onclick={loadMoreTransactions}>Load More</button>
        {/if}
      {/if}
    </section>

  <!-- ACCOUNTS TAB -->
  {:else if activeTab === 'accounts'}
    <section class="tab-content">
      <div class="section-header">
        <h3>My Accounts</h3>
        <div class="header-actions">
          <LiquidGlassButton text="Transfer" contrast="dark" onclick={openTransfer} />
          <LiquidGlassButton text="New Account" contrast="dark" onclick={openAddAccount} />
        </div>
      </div>

      {#if (accountsData?.accounts ?? []).length === 0}
        <p class="empty">No accounts yet. Create your first account.</p>
      {:else}
        <div class="account-grid">
          {#each accountsData?.accounts ?? [] as acc}
            <button class="account-card" onclick={() => openAccountDetail(acc.id)}>
              <img src={getBankIconPath(acc.account_type)} alt={acc.account_type} class="acc-icon" onerror={(e) => e.target.style.display='none'} />
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
      <h3>Transaction Categories</h3>

      <div class="cat-add-form">
        <input type="text" placeholder="Category name" bind:value={newCatName} />
        <select bind:value={newCatType}>
          <option value="expense">Expense</option>
          <option value="income">Income</option>
        </select>
        <button class="primary-btn" onclick={addCategory} disabled={!newCatName.trim()}>Add</button>
      </div>

      {#if categories}
        <div class="cat-section">
          <h4>Expense</h4>
          {#each categories.expense as cat}
            <div class="cat-row">
              <span>{cat.name}</span>
              {#if !cat.is_default}
                <button class="delete-btn-sm" onclick={() => deleteCat(cat.id)}>Delete</button>
              {:else}
                <span class="default-badge">Default</span>
              {/if}
            </div>
          {/each}
        </div>
        <div class="cat-section">
          <h4>Income</h4>
          {#each categories.income as cat}
            <div class="cat-row">
              <span>{cat.name}</span>
              {#if !cat.is_default}
                <button class="delete-btn-sm" onclick={() => deleteCat(cat.id)}>Delete</button>
              {:else}
                <span class="default-badge">Default</span>
              {/if}
            </div>
          {/each}
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
      <div class="info-row"><span>Type</span><span>{selectedAccount.account_type}</span></div>
      <div class="info-row"><span>Currency</span><span>{selectedAccount.currency}</span></div>
      <div class="info-row">
        <span>Balance</span>
        <span class:negative={selectedAccount.balance_negative}>{selectedAccount.balance}</span>
      </div>
    </div>
    {#if selectedAccount.transactions.length > 0}
      <h4>Recent Transactions</h4>
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
      <button class="danger-btn" onclick={() => deleteAccount(selectedAccount!.id)}>Delete Account</button>
    </div>
  </aside>
{/if}

<!-- Add/Edit Transaction Modal -->
{#if showAddTransaction}
  <div class="modal-backdrop" role="presentation" onclick={() => showAddTransaction = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddTransaction = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
    <LiquidGlassBackground />
    <h3>{editingTransaction ? 'Edit Transaction' : 'Add Transaction'}</h3>
    <div class="form-grid">
      <label>
        Account
        <select bind:value={txAccountId}>
          {#each accountsData?.accounts ?? [] as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
      </label>
      <label>
        Amount
        <input type="text" bind:value={txAmount} placeholder="0.00" />
      </label>
      <label>
        Type
        <div class="toggle-row">
          <button class="toggle-btn" class:active={txIsExpense} onclick={() => txIsExpense = true}>Expense</button>
          <button class="toggle-btn" class:active={!txIsExpense} onclick={() => txIsExpense = false}>Income</button>
        </div>
      </label>
      <label>
        Category
        <select bind:value={txCategory}>
          <option value="">Select...</option>
          {#each txCategoryOptions as cat}
            <option value={cat.name}>{cat.name}</option>
          {/each}
        </select>
      </label>
      <label>
        Description
        <input type="text" bind:value={txDescription} placeholder="Description" />
      </label>
      <label>
        Date
        <input type="date" bind:value={txDate} />
      </label>
    </div>
    <div class="modal-actions">
      <button class="secondary-btn" onclick={() => showAddTransaction = false}>Cancel</button>
      <button class="primary-btn" onclick={submitTransaction} disabled={!txAmount || !txAccountId}>
        {editingTransaction ? 'Update' : 'Add'}
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
      <LiquidGlassBackground />
      <h3>New Account</h3>
    <div class="form-grid">
      <label>
        Name
        <input type="text" bind:value={accName} placeholder="Account name" />
      </label>
      <label>
        Type
        <select bind:value={accType}>
          <option value="checking">Checking</option>
          <option value="savings">Savings</option>
          <option value="credit">Credit Card</option>
          <option value="cash">Cash</option>
          <option value="investment">Investment</option>
        </select>
      </label>
      <label>
        Currency
        <select bind:value={accCurrency}>
          {#each ['USD', 'CLP', 'EUR', 'GBP', 'BRL', 'MXN', 'ARS', 'CAD', 'AUD', 'CHF', 'JPY'] as cur}
            <option value={cur}>{cur}</option>
          {/each}
        </select>
      </label>
      <label>
        Initial Balance
        <input type="text" bind:value={accInitialBalance} placeholder="0.00" />
      </label>
    </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showAddAccount = false}>Cancel</button>
        <button class="primary-btn" onclick={submitAccount} disabled={!accName.trim()}>Create</button>
      </div>
    </div>
  </div>
{/if}

<!-- Transfer Modal -->
{#if showTransfer}
  <div class="modal-backdrop" role="presentation" onclick={() => showTransfer = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showTransfer = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <LiquidGlassBackground />
      <h3>Transfer Funds</h3>
    <div class="form-grid">
      <label>
        From
        <select bind:value={tfFromId}>
          {#each accountsData?.accounts ?? [] as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
      </label>
      <label>
        To
        <select bind:value={tfToId}>
          {#each accountsData?.accounts ?? [] as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
      </label>
      <label>
        Amount
        <input type="text" bind:value={tfAmount} placeholder="0.00" />
      </label>
      <label>
        Description
        <input type="text" bind:value={tfDescription} placeholder="Transfer note" />
      </label>
      <label>
        Date
        <input type="date" bind:value={tfDate} />
      </label>
    </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showTransfer = false}>Cancel</button>
        <button class="primary-btn" onclick={submitTransfer} disabled={!tfAmount || tfFromId === tfToId}>Transfer</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page { padding: 24px 32px; max-width: 960px; width: 100%; margin: 0 auto; }

  .hero { text-align: center; padding: 16px 0 24px; }
  .balance { font-size: 2rem; font-weight: 700; color: var(--text-primary); margin: 0; }
  .balance.negative { color: var(--danger); }
  .label { color: var(--text-tertiary); font-size: 0.8rem; margin-top: 4px; }

  .loading { text-align: center; padding: 48px; color: var(--text-tertiary); }
  .empty { text-align: center; padding: 48px; color: var(--text-tertiary); }

  .section-header {
    display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;
  }
  .section-header h3 { font-size: 0.9rem; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.08em; margin: 0; }
  .header-actions { display: flex; gap: 8px; }

  .filters {
    display: flex; gap: 8px; margin-bottom: 16px; flex-wrap: wrap;
  }
  .filters input, .filters select {
    padding: 8px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--glass); backdrop-filter: var(--glass-blur);
    color: var(--text-primary); font-size: 0.85rem;
    transition: border-color 0.2s;
  }
  .filters input:focus, .filters select:focus {
    border-color: var(--accent); outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .filters input { flex: 1; min-width: 150px; }
  .clear-btn {
    padding: 8px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.8rem;
    transition: all 0.15s;
  }
  .clear-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }

  .tx-list { display: flex; flex-direction: column; }
  .tx-row {
    display: grid; grid-template-columns: 80px 1fr auto auto auto 32px;
    gap: 12px; padding: 10px 8px; border-bottom: 1px solid var(--glass-border);
    align-items: center; cursor: pointer; border-radius: var(--radius-sm);
    transition: background 0.15s;
  }
  .tx-row:hover { background: var(--glass-hover); }
  .tx-date { color: var(--text-tertiary); font-size: 0.8rem; }
  .tx-desc { color: #ccc; font-size: 0.85rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tx-cat { color: var(--text-secondary); font-size: 0.8rem; }
  .tx-acc { color: var(--text-tertiary); font-size: 0.75rem; }
  .tx-amount { font-size: 0.85rem; font-weight: 500; text-align: right; color: var(--success); }
  .tx-amount.expense { color: var(--danger); }
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
    display: flex; flex-direction: column; gap: 4px; padding: 16px;
    background: var(--glass); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    cursor: pointer; text-align: left; color: inherit;
    transition: all 0.2s; box-shadow: var(--glass-glow);
  }
  .account-card:hover { border-color: var(--glass-border-hover); background: var(--glass-hover); box-shadow: var(--glass-shadow); }
  .acc-name { font-weight: 600; color: var(--text-primary); font-size: 0.9rem; }
  .acc-type { font-size: 0.75rem; color: var(--text-tertiary); text-transform: capitalize; }
  .acc-balance { font-size: 1.1rem; font-weight: 600; color: var(--text-primary); margin-top: 8px; }
  .acc-balance.negative { color: var(--danger); }
  .acc-currency { font-size: 0.7rem; color: var(--text-tertiary); }

  /* Categories */
  .cat-add-form {
    display: flex; gap: 8px; margin-bottom: 20px;
  }
  .cat-add-form input, .cat-add-form select {
    padding: 8px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--glass); color: var(--text-primary); font-size: 0.85rem;
  }
  .cat-add-form input:focus, .cat-add-form select:focus { border-color: var(--accent); outline: none; }
  .cat-add-form input { flex: 1; }

  .cat-section { margin-bottom: 20px; }
  .cat-section h4 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 8px; }
  .cat-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: 8px 12px; border-bottom: 1px solid var(--glass-border); font-size: 0.85rem; color: #ccc;
  }
  .delete-btn-sm {
    background: none; border: none; color: var(--text-tertiary); cursor: pointer; font-size: 0.75rem;
    transition: color 0.15s;
  }
  .delete-btn-sm:hover { color: var(--danger); }
  .default-badge { font-size: 0.7rem; color: var(--text-tertiary); }

  /* Overlay & detail panel */
  .overlay-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.4);
    backdrop-filter: blur(4px); z-index: 50;
  }
  .detail-panel {
    position: fixed; top: 0; right: 0; bottom: 0; width: 380px;
    background: var(--glass-elevated); backdrop-filter: var(--glass-blur-heavy);
    -webkit-backdrop-filter: var(--glass-blur-heavy);
    border-left: 1px solid var(--glass-border); z-index: 51;
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
  .info-row { display: flex; justify-content: space-between; font-size: 0.85rem; color: #ccc; }
  .info-row .negative { color: var(--danger); }

  .detail-panel h4 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 8px; }
  .panel-tx-list { display: flex; flex-direction: column; gap: 4px; }
  .panel-tx { display: grid; grid-template-columns: 70px 1fr auto; gap: 8px; font-size: 0.8rem; padding: 6px 0; border-bottom: 1px solid var(--glass-border); }
  .panel-actions { margin-top: 24px; }

  /* Modals */
  .modal-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.5);
    backdrop-filter: blur(4px); z-index: 100;
  }
  .modal {
    position: relative;
    background: linear-gradient(-75deg, rgba(0, 0, 0, 0.05), rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.05));
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 28px; width: 420px; max-height: 85vh; overflow-y: auto; z-index: 101;
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
    background: rgba(0, 0, 0, 0.25); color: var(--text-primary); font-size: 0.9rem;
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
    border-color: rgba(79, 156, 247, 0.3); box-shadow: 0 0 0 1px var(--accent-glow) inset;
  }

  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; position: relative; z-index: 10; }

  .primary-btn {
    padding: 8px 18px; border: 1px solid rgba(79, 156, 247, 0.3); border-radius: var(--radius-sm);
    background: rgba(79, 156, 247, 0.2); backdrop-filter: blur(8px);
    color: #fff; cursor: pointer; font-size: 0.85rem; font-weight: 500;
    transition: all 0.2s;
  }
  .primary-btn:hover:not(:disabled) {
    background: rgba(79, 156, 247, 0.3); border-color: rgba(79, 156, 247, 0.5);
    box-shadow: 0 0 16px var(--accent-glow);
  }
  .primary-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  .secondary-btn {
    padding: 8px 18px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: #ccc; cursor: pointer; font-size: 0.85rem;
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
</style>
