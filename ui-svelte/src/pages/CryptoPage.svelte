<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import LiquidGlassButton from '../components/LiquidGlassButton.svelte'
  import LiquidGlassTab from '../components/LiquidGlassTab.svelte'
  import LiquidGlassBackground from '../components/LiquidGlassBackground.svelte'
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
      await cryptoApi.addWallet(walletName, walletCategory)
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
    <LiquidGlassTab
      options={[
        { label: 'Portfolio', value: 'portfolio' },
        { label: 'Wallets', value: 'wallets' },
        { label: 'Tax', value: 'tax' }
      ]}
      active={activeTab}
      onchange={(value) => activeTab = value as Tab}
    />
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
      <LiquidGlassButton text="Add Wallet" contrast="dark" onclick={() => { showAddWallet = true; walletName = '' }} />
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
  <div class="overlay-backdrop" role="presentation" onclick={() => selectedWallet = null} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') selectedWallet = null }}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      <h3>{selectedWallet.name}</h3>
      <button class="close-panel" aria-label="Close panel" onclick={() => selectedWallet = null}>
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
  <div class="overlay-backdrop" role="presentation" onclick={() => showAssetDetail = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAssetDetail = false }}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      <h3>{assetInView.symbol} - {assetInView.name}</h3>
      <button class="close-panel" aria-label="Close panel" onclick={() => showAssetDetail = false}>
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
  <div class="modal-backdrop" role="presentation" onclick={() => showAddWallet = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddWallet = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <LiquidGlassBackground />
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
  </div>
{/if}

<style>
  .page { padding: 24px 32px; max-width: 960px; }

  .fx-badge {
    display: inline-flex; align-items: center; gap: 8px;
    padding: 6px 14px;
    background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: 20px; font-size: 0.8rem; margin-bottom: 12px;
    box-shadow: var(--glass-glow);
  }
  .fx-pair { color: var(--text-secondary); }
  .fx-rate { color: var(--text-primary); font-weight: 500; }
  .fx-live { width: 6px; height: 6px; border-radius: 50%; background: var(--success); box-shadow: 0 0 6px rgba(74, 222, 128, 0.4); }

  .hero { text-align: center; padding: 8px 0 20px; }
  .total { font-size: 2.2rem; font-weight: 700; color: var(--text-primary); margin: 0; }
  .label { color: var(--text-tertiary); font-size: 0.8rem; margin-top: 4px; }

  .tab-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }

  .loading { text-align: center; padding: 48px; color: var(--text-tertiary); }
  .empty { text-align: center; padding: 48px; color: var(--text-tertiary); }

  /* Stats bar */
  .stats-bar { display: flex; gap: 24px; margin-bottom: 24px; }
  .stats-bar .stat { display: flex; flex-direction: column; }
  .stat-lbl { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; }
  .stat-val { font-size: 1rem; font-weight: 600; color: var(--text-secondary); }
  .stat-val.positive { color: var(--success); }
  .stat-val.negative { color: var(--danger); }

  /* Holdings */
  .holdings-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 12px; margin-bottom: 24px;
  }
  .asset-card {
    display: flex; flex-direction: column; gap: 6px; padding: 14px;
    background: var(--glass); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    cursor: pointer; text-align: left; color: inherit;
    transition: all 0.2s; box-shadow: var(--glass-glow);
  }
  .asset-card:hover { border-color: var(--glass-border-hover); background: var(--glass-hover); box-shadow: var(--glass-shadow); }
  .asset-top { display: flex; align-items: baseline; gap: 6px; }
  .asset-symbol { font-weight: 700; color: var(--text-primary); font-size: 0.95rem; }
  .asset-name { font-size: 0.7rem; color: var(--text-tertiary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .asset-price { display: flex; align-items: baseline; gap: 6px; }
  .asset-price span:first-child { font-size: 0.85rem; color: #ccc; }
  .change { font-size: 0.75rem; color: var(--success); }
  .change.negative { color: var(--danger); }
  .asset-bottom { display: flex; justify-content: space-between; }
  .asset-amount { font-size: 0.8rem; color: var(--text-secondary); }
  .asset-value { font-size: 0.85rem; font-weight: 500; color: var(--text-primary); }

  /* Charts */
  .chart-section {
    background: var(--glass); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 16px; margin-bottom: 24px;
    box-shadow: var(--glass-shadow);
  }
  .chart-section h3 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin: 0 0 8px; }

  /* Wallets */
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .section-header h3 { font-size: 0.85rem; color: var(--text-secondary); text-transform: uppercase; margin: 0; }

  .wallet-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px;
  }
  .wallet-card {
    display: flex; flex-direction: column; gap: 4px; padding: 16px;
    background: var(--glass); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    cursor: pointer; text-align: left; color: inherit;
    transition: all 0.2s; box-shadow: var(--glass-glow);
  }
  .wallet-card:hover { border-color: var(--glass-border-hover); background: var(--glass-hover); box-shadow: var(--glass-shadow); }
  .wallet-name { font-weight: 600; color: var(--text-primary); }
  .wallet-cat { font-size: 0.75rem; color: var(--text-tertiary); text-transform: capitalize; }
  .wallet-val { font-size: 1.1rem; font-weight: 600; color: var(--text-primary); margin-top: 6px; }
  .wallet-count { font-size: 0.75rem; color: var(--text-tertiary); }

  .tax-placeholder { text-align: center; }
  .tax-placeholder h3 { font-size: 0.9rem; color: var(--text-secondary); }

  /* Detail panel */
  .overlay-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.4); backdrop-filter: blur(4px); z-index: 50; }
  .detail-panel {
    position: fixed; top: 0; right: 0; bottom: 0; width: 400px;
    background: var(--glass-elevated); backdrop-filter: var(--glass-blur-heavy);
    -webkit-backdrop-filter: var(--glass-blur-heavy);
    border-left: 1px solid var(--glass-border); z-index: 51;
    padding: 24px; overflow-y: auto;
    box-shadow: var(--glass-shadow-lg);
    animation: slideInRight 0.25s ease;
  }
  @keyframes slideInRight { from { transform: translateX(100%); } to { transform: translateX(0); } }

  .panel-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .panel-header h3 { margin: 0; color: var(--text-primary); font-size: 1rem; }
  .close-panel { background: none; border: none; color: var(--text-tertiary); cursor: pointer; padding: 4px; display: flex; transition: color 0.15s; }
  .close-panel:hover { color: var(--text-primary); }
  .close-panel svg { width: 20px; height: 20px; }

  .panel-meta { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 16px; font-size: 0.9rem; color: #ccc; }
  .panel-total { font-weight: 600; font-size: 1.1rem; }

  .detail-panel h4 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin: 16px 0 8px; }

  .holding-row { display: grid; grid-template-columns: 60px 1fr auto; gap: 8px; padding: 6px 0; border-bottom: 1px solid var(--glass-border); font-size: 0.85rem; }
  .h-symbol { color: #ccc; font-weight: 500; }
  .h-amount { color: var(--text-secondary); }
  .h-value { color: var(--text-primary); text-align: right; }

  .panel-tx { display: grid; grid-template-columns: 70px auto 1fr; gap: 8px; font-size: 0.8rem; padding: 6px 0; border-bottom: 1px solid var(--glass-border); }
  .tx-date { color: var(--text-tertiary); }
  .tx-type { color: var(--text-secondary); text-transform: capitalize; }
  .tx-amount { color: #ccc; text-align: right; }

  .panel-actions { margin-top: 20px; }

  .asset-stats { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }
  .asset-stats div { display: flex; justify-content: space-between; font-size: 0.85rem; color: #ccc; }

  /* Modal */
  .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); backdrop-filter: blur(4px); z-index: 100; }
  .modal-wrapper {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    z-index: 101; pointer-events: none;
  }
  .modal-wrapper .modal {
    pointer-events: auto;
  }
  .modal {
    position: relative;
    background: linear-gradient(-75deg, rgba(0, 0, 0, 0.05), rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.05));
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 28px; width: 400px; z-index: 101;
    box-shadow: inset 0 0.125em 0.125em rgba(254, 254, 254, 0.05), inset 0 -0.125em 0.125em rgba(0, 0, 0, 0.5), 0 0.25em 0.125em -0.125em rgba(254, 254, 254, 0.2), 0 0 0.1em 0.25em inset rgba(0, 0, 0, 0.2);
  }
  .modal h3 { margin: 0 0 20px; color: var(--text-primary); position: relative; z-index: 10; }

  .form-grid { display: flex; flex-direction: column; gap: 14px; position: relative; z-index: 10; }
  .form-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: var(--text-secondary); }
  .form-grid input {
    padding: 10px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.25); color: var(--text-primary); font-size: 0.9rem;
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .form-grid input:focus { border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow); }

  .category-cards { display: flex; gap: 8px; }
  .cat-card {
    flex: 1; padding: 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; text-transform: capitalize;
    font-size: 0.85rem; text-align: center; transition: all 0.15s;
  }
  .cat-card:hover { border-color: var(--glass-border-hover); }
  .cat-card.selected {
    border-color: rgba(79, 156, 247, 0.3); color: var(--text-primary);
    background: var(--glass-active); box-shadow: 0 0 0 1px var(--accent-glow) inset;
  }

  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; position: relative; z-index: 10; }
  .primary-btn {
    padding: 8px 18px; border: 1px solid rgba(79, 156, 247, 0.3); border-radius: var(--radius-sm);
    background: rgba(79, 156, 247, 0.2); backdrop-filter: blur(8px);
    color: #fff; cursor: pointer; font-size: 0.85rem; font-weight: 500;
    transition: all 0.2s;
  }
  .primary-btn:hover:not(:disabled) { background: rgba(79, 156, 247, 0.3); box-shadow: 0 0 16px var(--accent-glow); }
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
</style>
