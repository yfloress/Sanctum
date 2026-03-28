<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import * as cryptoApi from '../lib/api/crypto'
  import PortfolioTrendChart from '../components/charts/PortfolioTrendChart.svelte'
  import DistributionChart from '../components/charts/DistributionChart.svelte'
  import type {
    PortfolioResponse, PortfolioTrendData,
    WalletsResponse, WalletDetailResponse,
    CryptoTransactionDto
  } from '../lib/types'

  type Tab = 'portfolio' | 'wallets' | 'tax'
  let activeTab = $state<Tab>('portfolio')
  let loading = $state(true)

  // Portfolio state
  let portfolio = $state<PortfolioResponse | null>(null)
  let trend = $state<PortfolioTrendData | null>(null)

  // Wallets state
  let walletsData = $state<WalletsResponse | null>(null)
  let selectedWallet = $state<WalletDetailResponse | null>(null)

  // Wallet form
  let showAddWallet = $state(false)
  let walletName = $state('')
  let walletCategory = $state('exchange')

  // Asset detail overlay
  let showAssetDetail = $state(false)
  let assetCoinId = $state('')
  let assetTransactions = $state<CryptoTransactionDto[]>([])

  async function load() {
    loading = true
    try {
      const [p, t] = await Promise.all([
        cryptoApi.fetchPortfolio(),
        cryptoApi.fetchPortfolioTrend(),
      ])
      portfolio = p
      trend = t
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      loading = false
    }
  }

  async function loadWallets() {
    try {
      walletsData = await cryptoApi.fetchWallets()
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function openWalletDetail(id: string) {
    try {
      selectedWallet = await cryptoApi.fetchWalletDetail(id)
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function openAssetDetail(coinId: string) {
    assetCoinId = coinId
    try {
      assetTransactions = await cryptoApi.getCryptoTransactionsByCoin(coinId)
      showAssetDetail = true
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function submitWallet() {
    try {
      await cryptoApi.addWallet({ name: walletName, category: walletCategory })
      showAddWallet = false
      walletName = ''
      await loadWallets()
      app.showToast('Wallet created')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteWallet(id: string) {
    try {
      await cryptoApi.deleteWallet(id, false)
      selectedWallet = null
      await loadWallets()
      app.showToast('Wallet deleted')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  let assetInView = $derived(
    portfolio?.assets.find(a => a.coin_id === assetCoinId)
  )

  $effect(() => { load() })
  $effect(() => { if (activeTab === 'wallets') loadWallets() })
</script>

<div class="page">
  <!-- FX Rate Badge -->
  {#if portfolio?.fx_rate}
    <div class="fx-badge">
      <span class="fx-pair">{portfolio.fx_rate.pair}</span>
      <span class="fx-rate">{portfolio.fx_rate.rate}</span>
      {#if portfolio.fx_rate.is_live}
        <span class="fx-live"></span>
      {/if}
    </div>
  {/if}

  <!-- Hero -->
  <section class="hero">
    <h2 class="total">{portfolio?.total_value ?? '--'}</h2>
    <p class="label">Portfolio Value</p>
  </section>

  <!-- Tabs -->
  <div class="tab-row">
    <div class="tabs">
      {#each [['portfolio', 'Portfolio'], ['wallets', 'Wallets'], ['tax', 'Tax']] as [key, label]}
        <button class="tab-btn" class:active={activeTab === key} onclick={() => activeTab = key as Tab}>{label}</button>
      {/each}
    </div>
  </div>

  {#if loading}
    <div class="loading">Loading...</div>

  <!-- PORTFOLIO TAB -->
  {:else if activeTab === 'portfolio' && portfolio}
    <!-- Stats bar -->
    <div class="stats-bar">
      <div class="stat">
        <span class="stat-lbl">Unrealized P&L</span>
        <span class="stat-val" class:negative={portfolio.unrealized_pnl_negative} class:positive={!portfolio.unrealized_pnl_negative}>
          {portfolio.unrealized_pnl}
        </span>
      </div>
      <div class="stat">
        <span class="stat-lbl">Realized YTD</span>
        <span class="stat-val" class:negative={portfolio.realized_ytd_negative} class:positive={!portfolio.realized_ytd_negative}>
          {portfolio.realized_ytd}
        </span>
      </div>
      <div class="stat">
        <span class="stat-lbl">ROI</span>
        <span class="stat-val" class:negative={portfolio.roi_negative} class:positive={!portfolio.roi_negative}>
          {portfolio.roi}
        </span>
      </div>
    </div>

    <!-- Holdings -->
    {#if portfolio.assets.length === 0}
      <p class="empty">No assets yet. Create a wallet and add transactions to get started.</p>
    {:else}
      <div class="holdings-grid">
        {#each portfolio.assets as asset}
          <button class="asset-card" onclick={() => openAssetDetail(asset.coin_id)}>
            <div class="asset-top">
              <span class="asset-symbol">{asset.symbol}</span>
              <span class="asset-name">{asset.name}</span>
            </div>
            <div class="asset-price">
              <span>{asset.price}</span>
              <span class="change" class:negative={asset.price_change_24h_negative}>
                {asset.price_change_24h}
              </span>
            </div>
            <div class="asset-bottom">
              <span class="asset-amount">{asset.amount}</span>
              <span class="asset-value">{asset.value}</span>
            </div>
          </button>
        {/each}
      </div>

      <!-- Portfolio Trend Chart -->
      {#if trend && trend.dates.length > 0}
        <div class="chart-section">
          <h3>Portfolio Trend</h3>
          <PortfolioTrendChart data={trend} />
        </div>
      {/if}

      <!-- Distribution Chart -->
      {#if portfolio.distribution.length > 0}
        <div class="chart-section">
          <h3>Distribution</h3>
          <DistributionChart data={portfolio.distribution} />
        </div>
      {/if}
    {/if}

  <!-- WALLETS TAB -->
  {:else if activeTab === 'wallets'}
    <div class="section-header">
      <h3>Wallets</h3>
      <button class="primary-btn" onclick={() => { showAddWallet = true; walletName = '' }}>Add Wallet</button>
    </div>

    {#if (walletsData?.wallets ?? []).length === 0}
      <p class="empty">No wallets yet.</p>
    {:else}
      <div class="wallet-grid">
        {#each walletsData?.wallets ?? [] as w}
          <button class="wallet-card" onclick={() => openWalletDetail(w.id)}>
            <div class="wallet-name">{w.name}</div>
            <div class="wallet-cat">{w.category}</div>
            <div class="wallet-val">{w.total_value}</div>
            <div class="wallet-count">{w.assets_count} asset{w.assets_count !== 1 ? 's' : ''}</div>
          </button>
        {/each}
      </div>
    {/if}

  <!-- TAX TAB -->
  {:else if activeTab === 'tax'}
    <div class="tax-placeholder">
      <h3>Tax Tools</h3>
      <p class="empty">Tax settings, reports, and IPC import coming soon.</p>
    </div>
  {/if}
</div>

<!-- Wallet Detail Panel -->
{#if selectedWallet}
  <div class="overlay-backdrop" onclick={() => selectedWallet = null}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      <h3>{selectedWallet.name}</h3>
      <button class="close-panel" onclick={() => selectedWallet = null}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
    <div class="panel-meta">
      <span>{selectedWallet.category}</span>
      <span class="panel-total">{selectedWallet.total_value}</span>
    </div>

    {#if selectedWallet.holdings.length > 0}
      <h4>Holdings</h4>
      {#each selectedWallet.holdings as h}
        <div class="holding-row">
          <span class="h-symbol">{h.symbol}</span>
          <span class="h-amount">{h.amount}</span>
          <span class="h-value">{h.value}</span>
        </div>
      {/each}
    {/if}

    {#if selectedWallet.transactions.length > 0}
      <h4>Transactions</h4>
      {#each selectedWallet.transactions.slice(0, 20) as tx}
        <div class="panel-tx">
          <span class="tx-date">{tx.date}</span>
          <span class="tx-type">{tx.transaction_type}</span>
          <span class="tx-amount">{tx.amount} {tx.symbol}</span>
        </div>
      {/each}
    {/if}

    <div class="panel-actions">
      <button class="danger-btn" onclick={() => deleteWallet(selectedWallet!.id)}>Delete Wallet</button>
    </div>
  </aside>
{/if}

<!-- Asset Detail Overlay -->
{#if showAssetDetail && assetInView}
  <div class="overlay-backdrop" onclick={() => showAssetDetail = false}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      <h3>{assetInView.symbol} - {assetInView.name}</h3>
      <button class="close-panel" onclick={() => showAssetDetail = false}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
    <div class="panel-meta">
      <span>{assetInView.price}</span>
      <span class="change" class:negative={assetInView.price_change_24h_negative}>{assetInView.price_change_24h}</span>
    </div>
    <div class="asset-stats">
      <div><span class="stat-lbl">Amount</span><span>{assetInView.amount}</span></div>
      <div><span class="stat-lbl">Value</span><span>{assetInView.value}</span></div>
      <div><span class="stat-lbl">Allocation</span><span>{assetInView.allocation_pct.toFixed(1)}%</span></div>
    </div>

    {#if assetTransactions.length > 0}
      <h4>Transactions</h4>
      {#each assetTransactions as tx}
        <div class="panel-tx">
          <span class="tx-date">{tx.date}</span>
          <span class="tx-type">{tx.transaction_type}</span>
          <span class="tx-amount">{tx.amount} {tx.symbol}</span>
        </div>
      {/each}
    {/if}
  </aside>
{/if}

<!-- Add Wallet Modal -->
{#if showAddWallet}
  <div class="modal-backdrop" onclick={() => showAddWallet = false}></div>
  <div class="modal">
    <h3>New Wallet</h3>
    <div class="form-grid">
      <label>
        Name
        <input type="text" bind:value={walletName} placeholder="Wallet name" />
      </label>
      <label>
        Category
        <div class="category-cards">
          {#each ['exchange', 'hardware', 'software'] as cat}
            <button class="cat-card" class:selected={walletCategory === cat} onclick={() => walletCategory = cat}>
              {cat}
            </button>
          {/each}
        </div>
      </label>
    </div>
    <div class="modal-actions">
      <button class="secondary-btn" onclick={() => showAddWallet = false}>Cancel</button>
      <button class="primary-btn" onclick={submitWallet} disabled={!walletName.trim()}>Create</button>
    </div>
  </div>
{/if}

<style>
  .page { padding: 24px 32px; max-width: 960px; }

  .fx-badge {
    display: inline-flex; align-items: center; gap: 8px;
    padding: 6px 14px; background: #111; border: 1px solid #222;
    border-radius: 20px; font-size: 0.8rem; margin-bottom: 12px;
  }
  .fx-pair { color: #888; }
  .fx-rate { color: #e0e0e0; font-weight: 500; }
  .fx-live { width: 6px; height: 6px; border-radius: 50%; background: #4ade80; }

  .hero { text-align: center; padding: 8px 0 20px; }
  .total { font-size: 2.2rem; font-weight: 700; color: #e0e0e0; margin: 0; }
  .label { color: #666; font-size: 0.8rem; margin-top: 4px; }

  .tab-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
  .tabs {
    display: flex; gap: 4px; background: #111; border-radius: 8px;
    padding: 3px; border: 1px solid #222;
  }
  .tab-btn {
    padding: 8px 20px; border: none; border-radius: 6px; background: none;
    color: #888; cursor: pointer; font-size: 0.85rem; font-weight: 500;
  }
  .tab-btn.active { background: #1a1a1a; color: #e0e0e0; }

  .loading { text-align: center; padding: 48px; color: #666; }
  .empty { text-align: center; padding: 48px; color: #555; }

  /* Stats bar */
  .stats-bar { display: flex; gap: 24px; margin-bottom: 24px; }
  .stats-bar .stat { display: flex; flex-direction: column; }
  .stat-lbl { font-size: 0.7rem; color: #666; text-transform: uppercase; }
  .stat-val { font-size: 1rem; font-weight: 600; color: #888; }
  .stat-val.positive { color: #4ade80; }
  .stat-val.negative { color: #f87171; }

  /* Holdings */
  .holdings-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 12px; margin-bottom: 24px;
  }
  .asset-card {
    display: flex; flex-direction: column; gap: 6px; padding: 14px;
    background: #111; border: 1px solid #222; border-radius: 10px;
    cursor: pointer; text-align: left; color: inherit;
    transition: border-color 0.15s;
  }
  .asset-card:hover { border-color: #444; }
  .asset-top { display: flex; align-items: baseline; gap: 6px; }
  .asset-symbol { font-weight: 700; color: #e0e0e0; font-size: 0.95rem; }
  .asset-name { font-size: 0.7rem; color: #666; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .asset-price { display: flex; align-items: baseline; gap: 6px; }
  .asset-price span:first-child { font-size: 0.85rem; color: #ccc; }
  .change { font-size: 0.75rem; color: #4ade80; }
  .change.negative { color: #f87171; }
  .asset-bottom { display: flex; justify-content: space-between; }
  .asset-amount { font-size: 0.8rem; color: #888; }
  .asset-value { font-size: 0.85rem; font-weight: 500; color: #e0e0e0; }

  /* Charts */
  .chart-section {
    background: #111; border: 1px solid #222; border-radius: 10px;
    padding: 16px; margin-bottom: 24px;
  }
  .chart-section h3 { font-size: 0.8rem; color: #666; text-transform: uppercase; margin: 0 0 8px; }

  /* Wallets */
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .section-header h3 { font-size: 0.85rem; color: #888; text-transform: uppercase; margin: 0; }

  .wallet-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px;
  }
  .wallet-card {
    display: flex; flex-direction: column; gap: 4px; padding: 16px;
    background: #111; border: 1px solid #222; border-radius: 10px;
    cursor: pointer; text-align: left; color: inherit;
    transition: border-color 0.15s;
  }
  .wallet-card:hover { border-color: #444; }
  .wallet-name { font-weight: 600; color: #e0e0e0; }
  .wallet-cat { font-size: 0.75rem; color: #666; text-transform: capitalize; }
  .wallet-val { font-size: 1.1rem; font-weight: 600; color: #e0e0e0; margin-top: 6px; }
  .wallet-count { font-size: 0.75rem; color: #555; }

  .tax-placeholder { text-align: center; }
  .tax-placeholder h3 { font-size: 0.9rem; color: #888; }

  /* Detail panel */
  .overlay-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 50; }
  .detail-panel {
    position: fixed; top: 0; right: 0; bottom: 0; width: 400px;
    background: #111; border-left: 1px solid #222; z-index: 51;
    padding: 24px; overflow-y: auto; animation: slideInRight 0.2s ease;
  }
  @keyframes slideInRight { from { transform: translateX(100%); } to { transform: translateX(0); } }

  .panel-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .panel-header h3 { margin: 0; color: #e0e0e0; font-size: 1rem; }
  .close-panel { background: none; border: none; color: #666; cursor: pointer; padding: 4px; display: flex; }
  .close-panel:hover { color: #e0e0e0; }
  .close-panel svg { width: 20px; height: 20px; }

  .panel-meta { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 16px; font-size: 0.9rem; color: #ccc; }
  .panel-total { font-weight: 600; font-size: 1.1rem; }

  .detail-panel h4 { font-size: 0.8rem; color: #666; text-transform: uppercase; margin: 16px 0 8px; }

  .holding-row { display: grid; grid-template-columns: 60px 1fr auto; gap: 8px; padding: 6px 0; border-bottom: 1px solid #1a1a1a; font-size: 0.85rem; }
  .h-symbol { color: #ccc; font-weight: 500; }
  .h-amount { color: #888; }
  .h-value { color: #e0e0e0; text-align: right; }

  .panel-tx { display: grid; grid-template-columns: 70px auto 1fr; gap: 8px; font-size: 0.8rem; padding: 6px 0; border-bottom: 1px solid #1a1a1a; }
  .tx-date { color: #666; }
  .tx-type { color: #888; text-transform: capitalize; }
  .tx-amount { color: #ccc; text-align: right; }

  .panel-actions { margin-top: 20px; }

  .asset-stats { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }
  .asset-stats div { display: flex; justify-content: space-between; font-size: 0.85rem; color: #ccc; }

  /* Modal */
  .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.6); z-index: 100; }
  .modal {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    background: #111; border: 1px solid #222; border-radius: 12px;
    padding: 28px; width: 400px; z-index: 101;
  }
  .modal h3 { margin: 0 0 20px; color: #e0e0e0; }

  .form-grid { display: flex; flex-direction: column; gap: 14px; }
  .form-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: #888; }
  .form-grid input {
    padding: 10px 12px; border: 1px solid #333; border-radius: 6px;
    background: #0a0a0a; color: #e0e0e0; font-size: 0.9rem;
  }
  .form-grid input:focus { border-color: #4f9cf7; outline: none; }

  .category-cards { display: flex; gap: 8px; }
  .cat-card {
    flex: 1; padding: 10px; border: 1px solid #333; border-radius: 8px;
    background: none; color: #888; cursor: pointer; text-transform: capitalize;
    font-size: 0.85rem; text-align: center;
  }
  .cat-card.selected { border-color: #4f9cf7; color: #e0e0e0; background: #1a1a1a; }

  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }
  .primary-btn {
    padding: 8px 18px; border: none; border-radius: 6px;
    background: #4f9cf7; color: #fff; cursor: pointer; font-size: 0.85rem; font-weight: 500;
  }
  .primary-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .secondary-btn {
    padding: 8px 18px; border: 1px solid #333; border-radius: 6px;
    background: none; color: #ccc; cursor: pointer; font-size: 0.85rem;
  }
  .danger-btn {
    padding: 8px 18px; border: 1px solid #5a2d2d; border-radius: 6px;
    background: #3a1a1a; color: #f87171; cursor: pointer; font-size: 0.85rem;
  }
</style>
