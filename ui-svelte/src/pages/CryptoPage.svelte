<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import * as cryptoApi from '../lib/api/crypto'
  import PortfolioTrendChart from '../components/charts/PortfolioTrendChart.svelte'
  import DistributionChart from '../components/charts/DistributionChart.svelte'
  import type {
    PortfolioResponse, PortfolioTrendData,
    WalletsResponse, WalletDetailResponse,
    CryptoTransactionDto, CoinCatalogDto,
    CryptoAssetPriceDto, IpcSummaryDto
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

  // Transaction form
  let showAddTransaction = $state(false)
  let txMode = $state<'buy' | 'sell' | 'income' | 'fee' | 'transfer' | 'swap'>('buy')
  let txWalletId = $state('')
  let txCoinId = $state('')
  let txSymbol = $state('')
  let txAmount = $state('')
  let txPrice = $state('')
  let txFee = $state('0')
  let txDate = $state(new Date().toISOString().slice(0, 10))
  let txNotes = $state('')
  // Transfer fields
  let txFromWalletId = $state('')
  let txToWalletId = $state('')
  let txFromAmount = $state('')
  let txToAmount = $state('')
  // Swap fields
  let txFromCoinId = $state('')
  let txFromSymbol = $state('')
  let txToCoinId = $state('')
  let txToSymbol = $state('')
  let txSwapFromAmount = $state('')
  let txSwapToAmount = $state('')

  // Ticker bar
  let tickerPrices = $state<CryptoAssetPriceDto[]>([])
  let usdClpRate = $state<number | null>(null)
  let tickerSyncing = $state(false)

  // Ticker config
  let showTickerConfig = $state(false)
  let tickerIds = $state<string[]>([])

  // Coin catalog modal
  let showCoinCatalog = $state(false)
  let catalogSearch = $state('')
  let customCoinId = $state('')
  let customCoinName = $state('')
  let customCoinSymbol = $state('')

  // Coin catalog (shared between transaction form and catalog modal)
  let coinCatalog = $state<CoinCatalogDto[]>([])

  let filteredCatalog = $derived(
    catalogSearch.length < 1 ? coinCatalog.slice(0, 100) :
    coinCatalog.filter(c =>
      c.symbol.toLowerCase().includes(catalogSearch.toLowerCase()) ||
      c.name.toLowerCase().includes(catalogSearch.toLowerCase())
    ).slice(0, 100)
  )

  async function openTickerConfig() {
    await loadCoinCatalog()
    try { tickerIds = await cryptoApi.getActiveTickerIds() } catch (e) { app.showToast(String(e), true) }
    showTickerConfig = true
  }

  function toggleTicker(coinId: string) {
    if (tickerIds.includes(coinId)) {
      tickerIds = tickerIds.filter(id => id !== coinId)
    } else {
      tickerIds = [...tickerIds, coinId]
    }
  }

  async function saveTickerConfig() {
    try {
      await cryptoApi.saveActiveTickerIds(tickerIds)
      showTickerConfig = false
      app.showToast('Ticker config saved')
      await loadTickerPrices()
    } catch (e) { app.showToast(String(e), true) }
  }

  async function loadTickerPrices() {
    try {
      const [ids, prices] = await Promise.all([
        cryptoApi.getActiveTickerIds(),
        cryptoApi.loadCryptoPrices(),
      ])
      tickerIds = ids
      tickerPrices = prices.filter(p => ids.includes(p.id))
    } catch (_) { /* silently fail on initial load */ }
    try {
      const result = await cryptoApi.loadExchangeRate('USD/CLP')
      if (result) {
        usdClpRate = result[0]
      }
    } catch (_) { /* ignore */ }
  }

  async function syncTickerPrices() {
    tickerSyncing = true
    try {
      const ids = await cryptoApi.getMonitoredCoinIds()
      if (ids.length === 0) {
        app.showToast('No coins to sync. Configure ticker first.', true)
        return
      }
      
      const msg = await cryptoApi.syncCryptoData()
      await loadTickerPrices()
      await load()
      app.showToast(msg)
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      tickerSyncing = false
    }
  }

  async function openCoinCatalog() {
    await loadCoinCatalog()
    catalogSearch = ''
    customCoinId = ''
    customCoinName = ''
    customCoinSymbol = ''
    showCoinCatalog = true
  }

  async function addCustomCoinSubmit() {
    if (!customCoinId.trim() || !customCoinName.trim() || !customCoinSymbol.trim()) return
    try {
      await cryptoApi.addCustomCoin(customCoinId, customCoinName, customCoinSymbol)
      coinCatalog = await cryptoApi.getCoinCatalog()
      customCoinId = ''
      customCoinName = ''
      customCoinSymbol = ''
      app.showToast('Custom coin added')
    } catch (e) { app.showToast(String(e), true) }
  }

  async function deleteCustomCoinAction(id: string) {
    try {
      await cryptoApi.deleteCustomCoin(id)
      coinCatalog = await cryptoApi.getCoinCatalog()
      app.showToast('Custom coin deleted')
    } catch (e) { app.showToast(String(e), true) }
  }

  async function toggleFavorite(id: string, current: boolean) {
    try {
      await cryptoApi.setFavoriteCoin(id, !current)
      coinCatalog = await cryptoApi.getCoinCatalog()
    } catch (e) { app.showToast(String(e), true) }
  }

  let coinSearch = $state('')
  let filteredCoins = $derived(
    coinSearch.length < 1 ? coinCatalog.slice(0, 50) :
    coinCatalog.filter(c =>
      c.symbol.toLowerCase().includes(coinSearch.toLowerCase()) ||
      c.name.toLowerCase().includes(coinSearch.toLowerCase())
    ).slice(0, 50)
  )

  async function loadCoinCatalog() {
    if (coinCatalog.length > 0) return
    try { coinCatalog = await cryptoApi.getCoinCatalog() } catch (e) { app.showToast(String(e), true) }
  }

  function openAddTransaction() {
    txMode = 'buy'
    txWalletId = walletsData?.simple_list[0]?.id ?? ''
    txCoinId = ''
    txSymbol = ''
    txAmount = ''
    txPrice = ''
    txFee = '0'
    txDate = new Date().toISOString().slice(0, 10)
    txNotes = ''
    txFromWalletId = walletsData?.simple_list[0]?.id ?? ''
    txToWalletId = walletsData?.simple_list[1]?.id ?? walletsData?.simple_list[0]?.id ?? ''
    txFromAmount = ''
    txToAmount = ''
    txFromCoinId = ''
    txFromSymbol = ''
    txToCoinId = ''
    txToSymbol = ''
    txSwapFromAmount = ''
    txSwapToAmount = ''
    coinSearch = ''
    loadCoinCatalog()
    showAddTransaction = true
  }

  function selectCoin(coin: CoinCatalogDto) {
    txCoinId = coin.id
    txSymbol = coin.symbol
    coinSearch = ''
  }

  function selectFromCoin(coin: CoinCatalogDto) {
    txFromCoinId = coin.id
    txFromSymbol = coin.symbol
    coinSearch = ''
  }

  function selectToCoin(coin: CoinCatalogDto) {
    txToCoinId = coin.id
    txToSymbol = coin.symbol
    coinSearch = ''
  }

  async function submitCryptoTransaction() {
    try {
      if (txMode === 'transfer') {
        await cryptoApi.addCryptoTransfer({
          from_wallet_id: txFromWalletId,
          to_wallet_id: txToWalletId,
          coin_id: txCoinId,
          symbol: txSymbol,
          from_amount: txFromAmount,
          to_amount: txToAmount || txFromAmount,
          fee: txFee,
          date: txDate,
          notes: txNotes || undefined,
        })
      } else if (txMode === 'swap') {
        await cryptoApi.addCryptoSwap({
          wallet_id: txWalletId,
          from_coin_id: txFromCoinId,
          from_symbol: txFromSymbol,
          from_amount: txSwapFromAmount,
          to_coin_id: txToCoinId,
          to_symbol: txToSymbol,
          to_amount: txSwapToAmount,
          fee: txFee,
          date: txDate,
          notes: txNotes || undefined,
        })
      } else {
        await cryptoApi.addCryptoTransaction({
          wallet_id: txWalletId,
          coin_id: txCoinId,
          symbol: txSymbol,
          transaction_type: txMode,
          amount: txAmount,
          price: txPrice,
          fee: txFee,
          date: txDate,
          notes: txNotes || undefined,
        })
      }
      showAddTransaction = false
      await load()
      if (activeTab === 'wallets') await loadWallets()
      app.showToast('Transaction added')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteCryptoTx(id: string) {
    try {
      await cryptoApi.deleteCryptoTransaction(id)
      if (showAssetDetail) {
        assetTransactions = await cryptoApi.getCryptoTransactionsByCoin(assetCoinId)
      }
      if (selectedWallet) {
        selectedWallet = await cryptoApi.fetchWalletDetail(selectedWallet.id)
      }
      await load()
      app.showToast('Transaction deleted')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

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

  // Wallet edit state
  let editingWalletName = $state('')
  let showEditWalletName = $state(false)

  async function startEditWalletName() {
    editingWalletName = selectedWallet?.name ?? ''
    showEditWalletName = true
  }

  async function submitWalletName() {
    if (!selectedWallet || !editingWalletName.trim()) return
    try {
      await cryptoApi.updateWalletName(selectedWallet.id, editingWalletName)
      selectedWallet = await cryptoApi.fetchWalletDetail(selectedWallet.id)
      await loadWallets()
      showEditWalletName = false
      app.showToast('Wallet renamed')
    } catch (e) { app.showToast(String(e), true) }
  }

  // Tax state
  let taxPeriodId = $state('')
  let taxReport = $state<any>(null)
  let taxSettings = $state<any>(null)
  let showTaxSettings = $state(false)
  let taxLoading = $state(false)
  let taxJurisdiction = $state('US')
  let taxMethod = $state('fifo')
  let taxIncludeSwaps = $state(true)
  let taxIncludeFeeCrypto = $state(false)
  let taxExcludedWalletIds = $state<string[]>([])
  let ipcSummary = $state<IpcSummaryDto | null>(null)
  let ipcFileInput = $state<HTMLInputElement>(null!)

  async function loadIpcSummary() {
    try { ipcSummary = await cryptoApi.getIpcSummary() } catch (_) { /* ignore */ }
  }

  async function handleIpcFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    try {
      const content = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader()
        reader.onload = () => resolve(reader.result as string)
        reader.onerror = () => reject(reader.error)
        reader.readAsText(file)
      })
      await cryptoApi.importIpcCsv(content)
      await loadIpcSummary()
      app.showToast('IPC data imported')
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      ipcFileInput.value = ''
    }
  }

  async function loadTaxSettings() {
    if (!taxPeriodId.trim()) {
      app.showToast('Please enter a period ID', true)
      return
    }
    taxLoading = true
    try {
      taxSettings = await cryptoApi.loadTaxSettings(taxPeriodId)
      taxJurisdiction = taxSettings.jurisdiction
      taxMethod = taxSettings.method
      taxIncludeSwaps = taxSettings.include_swaps
      taxIncludeFeeCrypto = taxSettings.include_fee_crypto
      taxExcludedWalletIds = taxSettings.excluded_wallet_ids ?? []
      if (!walletsData) await loadWallets()
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      taxLoading = false
    }
  }

  async function saveTaxSettings() {
    if (!taxPeriodId.trim()) {
      app.showToast('Please enter a period ID', true)
      return
    }
    taxLoading = true
    try {
      await cryptoApi.saveTaxSettings({
        period_id: taxPeriodId,
        jurisdiction: taxJurisdiction,
        method: taxMethod,
        include_swaps: taxIncludeSwaps,
        include_fee_crypto: taxIncludeFeeCrypto,
        excluded_wallet_ids: taxExcludedWalletIds
      })
      showTaxSettings = false
      app.showToast('Settings saved')
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      taxLoading = false
    }
  }

  async function generateTaxReport() {
    if (!taxPeriodId.trim()) {
      app.showToast('Please enter a period ID', true)
      return
    }
    taxLoading = true
    try {
      taxReport = await cryptoApi.generateTaxReport(taxPeriodId)
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      taxLoading = false
    }
  }

  async function exportTaxReport(format: 'csv' | 'history') {
    if (!taxPeriodId.trim()) {
      app.showToast('Please enter a period ID', true)
      return
    }
    try {
      const path = `tax_report_${taxPeriodId}_${new Date().getTime()}.csv`
      if (format === 'csv') {
        await cryptoApi.exportTaxReportCsv(taxPeriodId, path)
      } else {
        await cryptoApi.exportTaxHistoryCsv(taxPeriodId, path)
      }
      app.showToast(`Exported to ${path}`)
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function getCryptoIconPath(symbol: string): string {
    const normalized = symbol.toLowerCase().replace(/\s+/g, '')
    return `/src/assets/crypto-icons/${normalized}.svg`
  }

  $effect(() => { load(); loadTickerPrices() })
  $effect(() => { if (activeTab === 'wallets') loadWallets() })
  $effect(() => { if (activeTab === 'tax') loadIpcSummary() })
</script>

<div class="page" class:blurred={showAddWallet || showTaxSettings || selectedWallet || showAssetDetail || showAddTransaction || showTickerConfig || showCoinCatalog}>
  <!-- Ticker Bar -->
  <div class="ticker-bar">
    <div class="ticker-fx">
      <span class="ticker-fx-pair">USD/CLP</span>
      <span class="ticker-fx-rate">{usdClpRate != null ? `$${usdClpRate.toLocaleString()}` : '--'}</span>
    </div>
    <div class="ticker-prices">
      {#each tickerPrices as coin}
        <div class="ticker-coin">
          <span class="ticker-coin-sym">{coin.symbol}</span>
          <span class="ticker-coin-price">${coin.current_price.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: coin.current_price < 1 ? 6 : 2 })}</span>
          <span class="ticker-coin-change" class:negative={coin.price_change_percentage_24h < 0} class:positive={coin.price_change_percentage_24h >= 0}>
            {coin.price_change_percentage_24h >= 0 ? '+' : ''}{coin.price_change_percentage_24h.toFixed(1)}%
          </span>
        </div>
      {/each}
      {#if tickerPrices.length === 0}
        <span class="ticker-empty">No tickers configured</span>
      {/if}
    </div>
    <div class="ticker-actions">
      <button class="ticker-sync-btn" onclick={syncTickerPrices} disabled={tickerSyncing} aria-label="Sync prices" title="Sync prices">
        <svg class:spinning={tickerSyncing} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m0 0a9 9 0 019-9m-9 9a9 9 0 009 9"/></svg>
      </button>
      <button class="ticker-config-btn" onclick={openTickerConfig} aria-label="Configure ticker" title="Configure ticker">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12.22 2h-.44a2 2 0 00-2 2v.18a2 2 0 01-1 1.73l-.43.25a2 2 0 01-2 0l-.15-.08a2 2 0 00-2.73.73l-.22.38a2 2 0 00.73 2.73l.15.1a2 2 0 011 1.72v.51a2 2 0 01-1 1.74l-.15.09a2 2 0 00-.73 2.73l.22.38a2 2 0 002.73.73l.15-.08a2 2 0 012 0l.43.25a2 2 0 011 1.73V20a2 2 0 002 2h.44a2 2 0 002-2v-.18a2 2 0 011-1.73l.43-.25a2 2 0 012 0l.15.08a2 2 0 002.73-.73l.22-.39a2 2 0 00-.73-2.73l-.15-.08a2 2 0 01-1-1.74v-.5a2 2 0 011-1.74l.15-.09a2 2 0 00.73-2.73l-.22-.38a2 2 0 00-2.73-.73l-.15.08a2 2 0 01-2 0l-.43-.25a2 2 0 01-1-1.73V4a2 2 0 00-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
      </button>
    </div>
  </div>

  <!-- Hero -->
  <section class="hero">
    <h2 class="total">{portfolio?.total_value ?? '--'}</h2>
    <p class="label">Portfolio Value</p>
    {#if portfolio?.last_updated}
      <p class="last-updated">Last updated: {portfolio.last_updated}</p>
    {/if}
  </section>

  <!-- Tabs -->
  <div class="tab-row">
    <div class="tab-bar">
      <button class:active={activeTab === 'portfolio'} onclick={() => activeTab = 'portfolio'}>Portfolio</button>
      <button class:active={activeTab === 'wallets'} onclick={() => activeTab = 'wallets'}>Wallets</button>
      <button class:active={activeTab === 'tax'} onclick={() => activeTab = 'tax'}>Tax</button>
    </div>
  </div>

  {#if loading}
    <div class="loading">Loading...</div>

  <!-- PORTFOLIO TAB -->
  {:else if activeTab === 'portfolio' && portfolio}
    <div class="section-header">
      <span></span>
      <div class="header-actions">
        <button class="glass-btn" onclick={openCoinCatalog}>Coins</button>
        <button class="glass-btn" onclick={openTickerConfig}>Tickers</button>
        <button class="glass-btn" onclick={openAddTransaction}>New Transaction</button>
      </div>
    </div>
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
            <div class="asset-header">
              <img src={getCryptoIconPath(asset.symbol)} alt={asset.symbol} class="asset-icon" onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
              <div class="asset-top">
                <span class="asset-symbol">{asset.symbol}</span>
                <span class="asset-name">{asset.name}</span>
              </div>
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
      <div class="header-actions">
        <button class="glass-btn" onclick={openAddTransaction}>New Transaction</button>
        <button class="glass-btn" onclick={() => { showAddWallet = true; walletName = '' }}>Add Wallet</button>
      </div>
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
    <div class="tax-section">
      <!-- Period selector -->
      <div class="period-selector">
        <label>
          Tax Period ID
          <input type="text" bind:value={taxPeriodId} placeholder="e.g., 2024" />
        </label>
        <div class="period-actions">
          <button class="glass-btn" onclick={loadTaxSettings}>Load Settings</button>
          {#if taxSettings}
            <button class="glass-btn" onclick={() => showTaxSettings = true}>Configure</button>
          {/if}
        </div>
      </div>

      {#if taxSettings}
        <!-- Settings info -->
        <div class="settings-info">
          <div class="info-item">
            <span class="label">Jurisdiction</span>
            <span class="value">{taxSettings.jurisdiction}</span>
          </div>
          <div class="info-item">
            <span class="label">Method</span>
            <span class="value">{taxSettings.method}</span>
          </div>
          <div class="info-item">
            <span class="label">Include Swaps</span>
            <span class="value">{taxSettings.include_swaps ? 'Yes' : 'No'}</span>
          </div>
          <div class="info-item">
            <span class="label">Include Fee Crypto</span>
            <span class="value">{taxSettings.include_fee_crypto ? 'Yes' : 'No'}</span>
          </div>
        </div>

        <!-- Generate report button -->
        <div class="report-actions">
          <button class="glass-btn" onclick={generateTaxReport}>Generate Report</button>
        </div>

        <!-- IPC Import -->
        <div class="ipc-section">
          <div class="setting-row">
            <div>
              <span class="ipc-label">IPC Price History</span>
              {#if ipcSummary && ipcSummary.records_count > 0}
                <span class="ipc-info">{ipcSummary.records_count} records {ipcSummary.date_range ? `(${ipcSummary.date_range})` : ''}</span>
              {:else}
                <span class="ipc-info">No IPC data imported</span>
              {/if}
            </div>
            <div>
              <input type="file" accept=".csv" class="hidden-input" bind:this={ipcFileInput} onchange={handleIpcFile} />
              <button class="glass-btn" onclick={() => ipcFileInput.click()}>Import IPC CSV</button>
            </div>
          </div>
        </div>
      {/if}

      {#if taxReport}
        <!-- Report summary -->
        <div class="report-summary">
          <h3>Report Summary</h3>
          <div class="summary-grid">
            <div class="summary-item">
              <span class="label">Disposals</span>
              <span class="value">{taxReport.disposals_count}</span>
            </div>
            <div class="summary-item">
              <span class="label">Total Proceeds</span>
              <span class="value">{taxReport.total_proceeds}</span>
            </div>
            <div class="summary-item">
              <span class="label">Total Cost</span>
              <span class="value">{taxReport.total_cost}</span>
            </div>
            <div class="summary-item">
              <span class="label">Total Gain</span>
              <span class="value" class:negative={taxReport.total_gain_negative}>{taxReport.total_gain}</span>
            </div>
            {#if taxReport.short_term_gain}
              <div class="summary-item">
                <span class="label">Short-term Gain</span>
                <span class="value">{taxReport.short_term_gain}</span>
              </div>
            {/if}
            {#if taxReport.long_term_gain}
              <div class="summary-item">
                <span class="label">Long-term Gain</span>
                <span class="value">{taxReport.long_term_gain}</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Warnings -->
        {#if taxReport.warnings && taxReport.warnings.length > 0}
          <div class="warnings">
            <h4>Warnings</h4>
            {#each taxReport.warnings as w}
              <div class="warning-item">
                <span class="warning-code">{w.code}</span>
                <span class="warning-msg">{w.message}</span>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Readiness -->
        {#if taxReport.readiness && taxReport.readiness.length > 0}
          <div class="readiness">
            <h4>Readiness</h4>
            {#each taxReport.readiness as r}
              <div class="readiness-item" class:complete={r.status === 'complete'} class:incomplete={r.status === 'incomplete'}>
                <span class="status-badge" class:complete={r.status === 'complete'}>{r.status}</span>
                <span class="detail">{r.detail}</span>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Export -->
        <div class="export-actions">
          <button onclick={() => exportTaxReport('csv')} class="export-btn">Export Events CSV</button>
          <button onclick={() => exportTaxReport('history')} class="export-btn">Export History CSV</button>
        </div>

        <!-- Events table (first 50) -->
        {#if taxReport.events && taxReport.events.length > 0}
          <div class="events-table">
            <h4>Events (showing first 50)</h4>
            <div class="table-wrapper">
              <table>
                <thead>
                  <tr>
                    <th>Date</th>
                    <th>Coin</th>
                    <th>Amount</th>
                    <th>Proceeds</th>
                    <th>Cost Basis</th>
                    <th>Gain</th>
                    <th>Term</th>
                  </tr>
                </thead>
                <tbody>
                  {#each taxReport.events.slice(0, 50) as e}
                    <tr class:loss={e.gain_negative}>
                      <td>{e.date}</td>
                      <td>{e.symbol}</td>
                      <td>{e.amount}</td>
                      <td>{e.proceeds}</td>
                      <td>{e.cost_basis}</td>
                      <td class:negative={e.gain_negative}>{e.gain}</td>
                      <td>{e.term ?? '-'}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      {/if}

      {#if taxLoading}
        <div class="loading">Processing tax data...</div>
      {/if}
    </div>

    <!-- Tax Settings Modal -->
    {#if showTaxSettings}
      <div class="modal-backdrop" role="presentation" onclick={() => showTaxSettings = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showTaxSettings = false }}></div>
      <div class="modal-wrapper">
        <div class="modal">
          <h3>Tax Settings</h3>
          <div class="form-grid">
            <label>
              Jurisdiction
              <select bind:value={taxJurisdiction}>
                <option value="US">United States</option>
                <option value="CL">Chile</option>
                <option value="CA">Canada</option>
                <option value="UK">United Kingdom</option>
                <option value="AU">Australia</option>
                <option value="OTHER">Other</option>
              </select>
            </label>
            <label>
              Cost Basis Method
              <select bind:value={taxMethod}>
                <option value="fifo">FIFO</option>
                <option value="lifo">LIFO</option>
                <option value="hifo">HIFO</option>
                <option value="average">Average Cost</option>
              </select>
            </label>
            <label>
              <input type="checkbox" bind:checked={taxIncludeSwaps} />
              Include Swaps in Disposals
            </label>
            <label>
              <input type="checkbox" bind:checked={taxIncludeFeeCrypto} />
              Include Fee Crypto as Disposal
            </label>
            {#if walletsData && walletsData.wallets.length > 0}
              <div class="exclusion-section">
                <span class="exclusion-title">Exclude Wallets</span>
                {#each walletsData.wallets as w}
                  <label class="exclusion-row">
                    <input
                      type="checkbox"
                      checked={taxExcludedWalletIds.includes(w.id)}
                      onchange={() => {
                        if (taxExcludedWalletIds.includes(w.id)) {
                          taxExcludedWalletIds = taxExcludedWalletIds.filter(x => x !== w.id)
                        } else {
                          taxExcludedWalletIds = [...taxExcludedWalletIds, w.id]
                        }
                      }}
                    />
                    <span>{w.name}</span>
                  </label>
                {/each}
              </div>
            {/if}
          </div>
          <div class="modal-actions">
            <button class="secondary-btn" onclick={() => showTaxSettings = false}>Cancel</button>
            <button class="primary-btn" onclick={saveTaxSettings} disabled={taxLoading}>Save</button>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<!-- Wallet Detail Panel -->
{#if selectedWallet}
  <div class="overlay-backdrop" role="presentation" onclick={() => selectedWallet = null} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') selectedWallet = null }}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      {#if showEditWalletName}
        <div class="inline-edit">
          <input type="text" bind:value={editingWalletName} class="edit-name-input" />
          <button class="icon-btn-sm" onclick={submitWalletName}>Save</button>
          <button class="icon-btn-sm" onclick={() => showEditWalletName = false}>Cancel</button>
        </div>
      {:else}
        <button class="clickable-name" onclick={startEditWalletName} title="Click to rename">{selectedWallet.name}</button>
      {/if}
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
          <button class="delete-btn" onclick={() => deleteCryptoTx(tx.id)} aria-label="Delete">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
          </button>
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
          <button class="delete-btn" onclick={() => deleteCryptoTx(tx.id)} aria-label="Delete">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
          </button>
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

<!-- Ticker Config Modal -->
{#if showTickerConfig}
  <div class="modal-backdrop" role="presentation" onclick={() => showTickerConfig = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showTickerConfig = false }}></div>
  <div class="modal-wrapper">
    <div class="modal wide">
      <h3>Configure Tickers</h3>
      <p class="modal-desc">Select which coins appear in the ticker bar.</p>
      <div class="ticker-list">
        {#each coinCatalog.filter(c => c.is_favorite || tickerIds.includes(c.id)).concat(coinCatalog.filter(c => !c.is_favorite && !tickerIds.includes(c.id))).slice(0, 80) as coin}
          <label class="ticker-item">
            <input type="checkbox" checked={tickerIds.includes(coin.id)} onchange={() => toggleTicker(coin.id)} />
            <span class="ticker-sym">{coin.symbol}</span>
            <span class="ticker-name">{coin.name}</span>
          </label>
        {/each}
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showTickerConfig = false}>Cancel</button>
        <button class="primary-btn" onclick={saveTickerConfig}>Save</button>
      </div>
    </div>
  </div>
{/if}

<!-- Coin Catalog Modal -->
{#if showCoinCatalog}
  <div class="modal-backdrop" role="presentation" onclick={() => showCoinCatalog = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showCoinCatalog = false }}></div>
  <div class="modal-wrapper">
    <div class="modal wide">
      <h3>Coin Catalog</h3>
      <input type="text" class="catalog-search" bind:value={catalogSearch} placeholder="Search coins..." />

      <div class="catalog-list">
        {#each filteredCatalog as coin}
          <div class="catalog-item">
            <button class="fav-btn" class:active={coin.is_favorite} onclick={() => toggleFavorite(coin.id, coin.is_favorite)} aria-label="Toggle favorite">
              <svg viewBox="0 0 24 24" fill={coin.is_favorite ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="1.5"><path d="M11.48 3.499a.562.562 0 011.04 0l2.125 5.111a.563.563 0 00.475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 00-.182.557l1.285 5.385a.562.562 0 01-.84.61l-4.725-2.885a.563.563 0 00-.586 0L6.982 20.54a.562.562 0 01-.84-.61l1.285-5.386a.562.562 0 00-.182-.557l-4.204-3.602a.563.563 0 01.321-.988l5.518-.442a.563.563 0 00.475-.345L11.48 3.5z"/></svg>
            </button>
            <span class="catalog-sym">{coin.symbol}</span>
            <span class="catalog-name">{coin.name}</span>
            {#if coin.is_custom}
              <button class="delete-btn" onclick={() => deleteCustomCoinAction(coin.id)} aria-label="Delete">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6 18L18 6M6 6l12 12"/></svg>
              </button>
            {/if}
          </div>
        {/each}
      </div>

      <div class="custom-coin-form">
        <span class="form-label">Add Custom Coin</span>
        <div class="custom-coin-row">
          <input type="text" bind:value={customCoinId} placeholder="ID" />
          <input type="text" bind:value={customCoinName} placeholder="Name" />
          <input type="text" bind:value={customCoinSymbol} placeholder="Symbol" />
          <button class="primary-btn" onclick={addCustomCoinSubmit} disabled={!customCoinId.trim() || !customCoinSymbol.trim()}>Add</button>
        </div>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showCoinCatalog = false}>Close</button>
      </div>
    </div>
  </div>
{/if}

<!-- Add Crypto Transaction Modal -->
{#if showAddTransaction}
  <div class="modal-backdrop" role="presentation" onclick={() => showAddTransaction = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddTransaction = false }}></div>
  <div class="modal-wrapper">
    <div class="modal wide">
      <h3>New Transaction</h3>
      <!-- Transaction type selector -->
      <div class="tx-type-bar">
        {#each (['buy', 'sell', 'income', 'fee', 'transfer', 'swap'] as const) as t}
          <button class="tx-type-btn" class:active={txMode === t} onclick={() => txMode = t}>
            {t.charAt(0).toUpperCase() + t.slice(1)}
          </button>
        {/each}
      </div>

      <div class="form-grid">
        {#if txMode === 'transfer'}
          <!-- Transfer form -->
          <label>
            Coin
            {#if txCoinId}
              <div class="coin-selected">
                <span>{txSymbol}</span>
                <button class="clear-coin" onclick={() => { txCoinId = ''; txSymbol = '' }}>x</button>
              </div>
            {:else}
              <input type="text" bind:value={coinSearch} placeholder="Search coin..." />
              {#if coinSearch.length >= 1}
                <div class="coin-dropdown">
                  {#each filteredCoins as c}
                    <button class="coin-option" onclick={() => selectCoin(c)}>{c.symbol} - {c.name}</button>
                  {/each}
                </div>
              {/if}
            {/if}
          </label>
          <label>
            From Wallet
            <select bind:value={txFromWalletId}>
              {#each walletsData?.simple_list ?? [] as w}
                <option value={w.id}>{w.name}</option>
              {/each}
            </select>
          </label>
          <label>
            To Wallet
            <select bind:value={txToWalletId}>
              {#each walletsData?.simple_list ?? [] as w}
                <option value={w.id}>{w.name}</option>
              {/each}
            </select>
          </label>
          <label>
            Amount
            <input type="text" bind:value={txFromAmount} placeholder="0.00" />
          </label>
          <label>
            Received Amount (optional)
            <input type="text" bind:value={txToAmount} placeholder="Same as sent if empty" />
          </label>
        {:else if txMode === 'swap'}
          <!-- Swap form -->
          <label>
            Wallet
            <select bind:value={txWalletId}>
              {#each walletsData?.simple_list ?? [] as w}
                <option value={w.id}>{w.name}</option>
              {/each}
            </select>
          </label>
          <label>
            From Coin
            {#if txFromCoinId}
              <div class="coin-selected">
                <span>{txFromSymbol}</span>
                <button class="clear-coin" onclick={() => { txFromCoinId = ''; txFromSymbol = '' }}>x</button>
              </div>
            {:else}
              <input type="text" bind:value={coinSearch} placeholder="Search coin..." />
              {#if coinSearch.length >= 1}
                <div class="coin-dropdown">
                  {#each filteredCoins as c}
                    <button class="coin-option" onclick={() => selectFromCoin(c)}>{c.symbol} - {c.name}</button>
                  {/each}
                </div>
              {/if}
            {/if}
          </label>
          <label>
            From Amount
            <input type="text" bind:value={txSwapFromAmount} placeholder="0.00" />
          </label>
          <label>
            To Coin
            {#if txToCoinId}
              <div class="coin-selected">
                <span>{txToSymbol}</span>
                <button class="clear-coin" onclick={() => { txToCoinId = ''; txToSymbol = '' }}>x</button>
              </div>
            {:else}
              <input type="text" bind:value={coinSearch} placeholder="Search coin..." />
              {#if coinSearch.length >= 1}
                <div class="coin-dropdown">
                  {#each filteredCoins as c}
                    <button class="coin-option" onclick={() => selectToCoin(c)}>{c.symbol} - {c.name}</button>
                  {/each}
                </div>
              {/if}
            {/if}
          </label>
          <label>
            To Amount
            <input type="text" bind:value={txSwapToAmount} placeholder="0.00" />
          </label>
        {:else}
          <!-- Buy/Sell/Income/Fee form -->
          <label>
            Wallet
            <select bind:value={txWalletId}>
              {#each walletsData?.simple_list ?? [] as w}
                <option value={w.id}>{w.name}</option>
              {/each}
            </select>
          </label>
          <label>
            Coin
            {#if txCoinId}
              <div class="coin-selected">
                <span>{txSymbol}</span>
                <button class="clear-coin" onclick={() => { txCoinId = ''; txSymbol = '' }}>x</button>
              </div>
            {:else}
              <input type="text" bind:value={coinSearch} placeholder="Search coin..." />
              {#if coinSearch.length >= 1}
                <div class="coin-dropdown">
                  {#each filteredCoins as c}
                    <button class="coin-option" onclick={() => selectCoin(c)}>{c.symbol} - {c.name}</button>
                  {/each}
                </div>
              {/if}
            {/if}
          </label>
          <label>
            Amount
            <input type="text" bind:value={txAmount} placeholder="0.00" />
          </label>
          <label>
            Price (per coin)
            <input type="text" bind:value={txPrice} placeholder="0.00" />
          </label>
        {/if}

        <!-- Common fields -->
        <label>
          Fee
          <input type="text" bind:value={txFee} placeholder="0" />
        </label>
        <label>
          Date
          <input type="date" bind:value={txDate} />
        </label>
        <label>
          Notes (optional)
          <input type="text" bind:value={txNotes} placeholder="Notes..." />
        </label>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showAddTransaction = false}>Cancel</button>
        <button class="primary-btn" onclick={submitCryptoTransaction}>Add</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page { padding: 24px 32px; max-width: 960px; width: 100%; margin: 0 auto; }

  /* Ticker Bar */
  .ticker-bar {
    display: flex; align-items: center; gap: 0;
    background: var(--glass); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 0; margin-bottom: 16px; overflow: hidden;
    box-shadow: var(--glass-glow);
  }
  .ticker-fx {
    display: flex; align-items: center; gap: 8px;
    padding: 10px 16px; border-right: 1px solid var(--glass-border);
    flex-shrink: 0; white-space: nowrap;
  }
  .ticker-fx-pair { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.05em; }
  .ticker-fx-rate { font-size: 0.9rem; font-weight: 600; color: var(--text-primary); }
  .ticker-prices {
    display: flex; align-items: center; gap: 0;
    flex: 1; overflow-x: auto; scrollbar-width: none;
  }
  .ticker-prices::-webkit-scrollbar { display: none; }
  .ticker-coin {
    display: flex; align-items: center; gap: 6px;
    padding: 10px 16px; border-right: 1px solid var(--glass-border);
    white-space: nowrap; flex-shrink: 0;
  }
  .ticker-coin-sym { font-size: 0.75rem; font-weight: 700; color: var(--text-secondary); }
  .ticker-coin-price { font-size: 0.85rem; color: var(--text-primary); font-weight: 500; }
  .ticker-coin-change { font-size: 0.75rem; }
  .ticker-coin-change.positive { color: var(--success); }
  .ticker-coin-change.negative { color: var(--danger); }
  .ticker-empty { padding: 10px 16px; font-size: 0.8rem; color: var(--text-tertiary); }
  .ticker-actions {
    display: flex; align-items: center; gap: 2px;
    padding: 6px 8px; flex-shrink: 0; border-left: 1px solid var(--glass-border);
  }
  .ticker-sync-btn, .ticker-config-btn {
    background: none; border: none; cursor: pointer;
    color: var(--text-tertiary); padding: 5px; display: flex;
    border-radius: var(--radius-sm); transition: color 0.15s, background 0.15s;
  }
  .ticker-sync-btn:hover, .ticker-config-btn:hover { color: var(--accent); background: var(--accent-bg); }
  .ticker-sync-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .ticker-sync-btn svg, .ticker-config-btn svg { width: 16px; height: 16px; }
  .spinning { animation: spin 1s linear infinite; }
  @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

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
  .asset-header { display: flex; align-items: center; gap: 8px; margin-bottom: 4px; }
  .asset-icon { width: 24px; height: 24px; flex-shrink: 0; border-radius: 50%; }
  .asset-top { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; }
  .asset-symbol { font-weight: 700; color: var(--text-primary); font-size: 0.95rem; }
  .asset-name { font-size: 0.7rem; color: var(--text-tertiary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .asset-price { display: flex; align-items: baseline; gap: 6px; }
  .asset-price span:first-child { font-size: 0.85rem; color: var(--text-secondary); }
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



  /* Detail panel */
  .overlay-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.3); z-index: 50; }
  .detail-panel {
    position: fixed; top: 0; right: 0; bottom: 0; width: 400px;
    background: linear-gradient(180deg, rgba(22, 22, 28, 0.88) 0%, rgba(16, 16, 20, 0.85) 100%);
    border-left: 1px solid rgba(255, 255, 255, 0.08); z-index: 51;
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

  .panel-meta { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 16px; font-size: 0.9rem; color: var(--text-secondary); }
  .panel-total { font-weight: 600; font-size: 1.1rem; }

  .detail-panel h4 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin: 16px 0 8px; }

  .holding-row { display: grid; grid-template-columns: 60px 1fr auto; gap: 8px; padding: 6px 0; border-bottom: 1px solid var(--glass-border); font-size: 0.85rem; }
  .h-symbol { color: var(--text-secondary); font-weight: 500; }
  .h-amount { color: var(--text-secondary); }
  .h-value { color: var(--text-primary); text-align: right; }

  .panel-tx { display: grid; grid-template-columns: 70px auto 1fr; gap: 8px; font-size: 0.8rem; padding: 6px 0; border-bottom: 1px solid var(--glass-border); }
  .tx-date { color: var(--text-tertiary); }
  .tx-type { color: var(--text-secondary); text-transform: capitalize; }
  .tx-amount { color: var(--text-secondary); text-align: right; }

  .panel-actions { margin-top: 20px; }

  .asset-stats { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }
  .asset-stats div { display: flex; justify-content: space-between; font-size: 0.85rem; color: var(--text-secondary); }

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
    background: linear-gradient(145deg, rgba(26, 26, 31, 0.75) 0%, rgba(20, 20, 24, 0.72) 50%, rgba(17, 17, 21, 0.7) 100%);
    border: 1px solid rgba(255, 255, 255, 0.1); border-radius: var(--radius-lg);
    padding: 28px; width: 400px; z-index: 101;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    box-shadow: inset 0 0.125em 0.125em rgba(254, 254, 254, 0.05), inset 0 -0.125em 0.125em rgba(0, 0, 0, 0.5), 0 0.25em 0.125em -0.125em rgba(254, 254, 254, 0.2), 0 0 0.1em 0.25em inset rgba(0, 0, 0, 0.2);
  }
  .modal h3 { margin: 0 0 20px; color: var(--text-primary); position: relative; z-index: 10; }

  .form-grid { display: flex; flex-direction: column; gap: 14px; position: relative; z-index: 10; }
  .form-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: var(--text-secondary); }
  .form-grid input {
    padding: 10px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.9rem;
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
    border-color: var(--accent-border); color: var(--text-primary);
    background: var(--glass-active); box-shadow: 0 0 0 1px var(--accent-glow) inset;
  }

  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; position: relative; z-index: 10; }
  .primary-btn {
    padding: 8px 18px; border: 1px solid var(--accent-border); border-radius: var(--radius-sm);
    background: var(--accent-bg); backdrop-filter: blur(8px);
    color: var(--text-on-accent); cursor: pointer; font-size: 0.85rem; font-weight: 500;
    transition: all 0.2s;
  }
  .primary-btn:hover:not(:disabled) { background: var(--accent); box-shadow: 0 0 16px var(--accent-glow); }
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

  /* Tax Section */
  .tax-section { display: flex; flex-direction: column; gap: 24px; }

  .period-selector {
    display: flex; flex-direction: column; gap: 12px;
    padding: 16px; background: var(--glass);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--glass-glow);
  }
  .period-selector label {
    display: flex; flex-direction: column; gap: 6px;
    font-size: 0.8rem; color: var(--text-secondary);
  }
  .period-selector input {
    padding: 8px 12px; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: var(--select-bg);
    color: var(--text-primary); font-size: 0.9rem;
    transition: border-color 0.2s;
  }
  .period-selector input:focus {
    border-color: var(--accent); outline: none;
  }
  .period-actions { display: flex; gap: 8px; }

  .settings-info {
    display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px; padding: 16px;
    background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--glass-glow);
  }
  .info-item { display: flex; flex-direction: column; gap: 4px; }
  .info-item .label { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; }
  .info-item .value { font-size: 0.95rem; color: var(--text-primary); font-weight: 500; }

  .report-actions { display: flex; gap: 8px; }
  .report-summary {
    padding: 16px; background: var(--glass);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--glass-glow);
  }
  .report-summary h3 { margin: 0 0 16px; color: var(--text-primary); font-size: 0.9rem; }
  .summary-grid {
    display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 12px;
  }
  .summary-item { display: flex; flex-direction: column; gap: 4px; }
  .summary-item .label { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; }
  .summary-item .value { font-size: 1rem; font-weight: 600; color: var(--text-primary); }
  .summary-item .value.negative { color: var(--danger); }

  .warnings {
    padding: 16px; background: var(--glass);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid rgba(248, 113, 113, 0.2); border-radius: var(--radius-md);
    box-shadow: var(--glass-glow);
  }
  .warnings h4 { margin: 0 0 12px; color: var(--danger); font-size: 0.85rem; }
  .warning-item { display: flex; gap: 8px; padding: 6px 0; border-bottom: 1px solid rgba(248, 113, 113, 0.1); font-size: 0.85rem; }
  .warning-code { color: var(--text-secondary); font-weight: 500; min-width: 80px; }
  .warning-msg { color: var(--text-secondary); }

  .readiness {
    padding: 16px; background: var(--glass);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--glass-glow);
  }
  .readiness h4 { margin: 0 0 12px; color: var(--text-primary); font-size: 0.85rem; }
  .readiness-item {
    display: flex; align-items: center; gap: 8px; padding: 8px;
    border-radius: var(--radius-sm); margin-bottom: 6px;
  }
  .readiness-item.complete { background: rgba(74, 222, 128, 0.05); }
  .readiness-item.incomplete { background: rgba(248, 113, 113, 0.05); }
  .status-badge {
    font-size: 0.65rem; text-transform: uppercase; font-weight: 600;
    padding: 2px 6px; border-radius: 3px; color: #999;
  }
  .status-badge.complete { background: rgba(74, 222, 128, 0.2); color: var(--success); }
  .readiness-item.incomplete .status-badge { background: rgba(248, 113, 113, 0.2); color: var(--danger); }
  .readiness-item .detail { font-size: 0.85rem; color: var(--text-secondary); }

  .export-actions { display: flex; gap: 8px; }
  .export-btn {
    padding: 8px 14px; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: rgba(0, 0, 0, 0.2);
    color: var(--text-secondary); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .export-btn:hover { border-color: var(--glass-border-hover); background: rgba(0, 0, 0, 0.3); }

  .events-table {
    padding: 16px; background: var(--glass);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--glass-glow);
  }
  .events-table h4 { margin: 0 0 12px; color: var(--text-primary); font-size: 0.85rem; }
  .table-wrapper { overflow-x: auto; }
  .events-table table {
    width: 100%; border-collapse: collapse; font-size: 0.8rem;
  }
  .events-table thead {
    background: rgba(0, 0, 0, 0.1); border-bottom: 1px solid var(--glass-border);
  }
  .events-table th {
    padding: 8px; text-align: left; color: var(--text-tertiary);
    text-transform: uppercase; font-weight: 500;
  }
  .events-table td {
    padding: 8px; border-bottom: 1px solid var(--glass-border);
    color: var(--text-secondary);
  }
  .events-table tr:hover { background: rgba(0, 0, 0, 0.1); }
  .events-table td.negative { color: var(--danger); }

  /* Form */
  select {
    padding: 8px 12px; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: var(--select-bg);
    color: var(--text-primary); font-size: 0.9rem;
    transition: border-color 0.2s;
  }
  select:focus { border-color: var(--accent); outline: none; }
  select option { background: var(--option-bg); color: var(--text-primary); }

  .header-actions { display: flex; gap: 8px; }

  /* Crypto transaction modal */
  .modal.wide { width: 480px; }
  .tx-type-bar { display: flex; gap: 4px; margin-bottom: 16px; flex-wrap: wrap; }
  .tx-type-btn {
    flex: 1; min-width: 60px; padding: 8px 4px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.8rem; text-align: center;
    transition: all 0.15s;
  }
  .tx-type-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .tx-type-btn.active {
    background: var(--glass-active); color: var(--text-primary);
    border-color: var(--accent-border); box-shadow: 0 0 0 1px var(--accent-glow) inset;
  }
  .coin-selected {
    display: flex; align-items: center; gap: 8px; padding: 8px 12px;
    background: var(--select-bg); border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    color: var(--text-primary); font-size: 0.9rem;
  }
  .coin-selected span { font-weight: 500; }
  .clear-coin {
    background: none; border: none; color: var(--text-tertiary); cursor: pointer; font-size: 0.9rem;
    margin-left: auto; padding: 0 4px; transition: color 0.15s;
  }
  .clear-coin:hover { color: var(--text-primary); }
  .coin-dropdown {
    max-height: 150px; overflow-y: auto; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: rgba(20, 20, 24, 0.95); margin-top: 4px;
  }
  .coin-option {
    display: block; width: 100%; padding: 8px 12px; background: none; border: none;
    color: var(--text-secondary); cursor: pointer; font-size: 0.8rem; text-align: left;
    transition: background 0.1s;
  }
  .coin-option:hover { background: var(--glass-hover); color: var(--text-primary); }

  .delete-btn {
    background: none; border: none; color: var(--text-tertiary); cursor: pointer; padding: 2px;
    display: flex; align-items: center; transition: color 0.15s; flex-shrink: 0;
  }
  .delete-btn:hover { color: var(--danger); }
  .delete-btn svg { width: 14px; height: 14px; }

  .last-updated { font-size: 0.7rem; color: var(--text-tertiary); margin-top: 4px; }

  /* Wallet inline edit */
  .inline-edit { display: flex; align-items: center; gap: 6px; flex: 1; }
  .edit-name-input {
    padding: 6px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.9rem; flex: 1;
  }
  .edit-name-input:focus { border-color: var(--accent); outline: none; }
  .icon-btn-sm {
    padding: 4px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.75rem;
    transition: all 0.15s;
  }
  .icon-btn-sm:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .clickable-name {
    cursor: pointer; margin: 0; color: var(--text-primary); background: none; border: none;
    font-size: 1rem; font-weight: 600; text-align: left; padding: 0;
  }
  .clickable-name:hover { color: var(--accent); }

  /* Tax wallet exclusion */
  .exclusion-section { display: flex; flex-direction: column; gap: 6px; margin-top: 4px; }
  .exclusion-title { font-size: 0.8rem; color: var(--text-secondary); font-weight: 500; }
  .exclusion-row {
    display: flex; align-items: center; gap: 8px; font-size: 0.85rem; color: var(--text-secondary); cursor: pointer;
  }
  .exclusion-row input[type="checkbox"] { width: 16px; height: 16px; cursor: pointer; }

  /* IPC section */
  .ipc-section {
    padding: 16px; background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--glass-glow);
  }
  .setting-row { display: flex; justify-content: space-between; align-items: center; gap: 12px; }
  .ipc-label { font-size: 0.85rem; color: var(--text-primary); font-weight: 500; }
  .ipc-info { font-size: 0.75rem; color: var(--text-tertiary); display: block; margin-top: 2px; }
  .hidden-input { display: none; }

  /* Ticker config */
  .modal-desc { font-size: 0.8rem; color: var(--text-tertiary); margin: 0 0 12px; }
  .ticker-list { max-height: 300px; overflow-y: auto; display: flex; flex-direction: column; gap: 4px; }
  .ticker-item {
    display: flex; align-items: center; gap: 8px; padding: 6px 8px; cursor: pointer;
    border-radius: var(--radius-sm); transition: background 0.1s; font-size: 0.85rem;
  }
  .ticker-item:hover { background: var(--glass-hover); }
  .ticker-item input[type="checkbox"] { width: 16px; height: 16px; cursor: pointer; }
  .ticker-sym { font-weight: 600; color: var(--text-primary); min-width: 60px; }
  .ticker-name { color: var(--text-secondary); }

  /* Coin catalog */
  .catalog-search {
    width: 100%; padding: 10px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.9rem; margin-bottom: 12px;
    box-sizing: border-box;
  }
  .catalog-search:focus { border-color: var(--accent); outline: none; }
  .catalog-list { max-height: 250px; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; }
  .catalog-item {
    display: flex; align-items: center; gap: 8px; padding: 6px 8px;
    border-radius: var(--radius-sm); font-size: 0.85rem;
  }
  .catalog-item:hover { background: var(--glass-hover); }
  .catalog-sym { font-weight: 600; color: var(--text-primary); min-width: 60px; }
  .catalog-name { color: var(--text-secondary); flex: 1; }
  .fav-btn {
    background: none; border: none; cursor: pointer; padding: 2px; display: flex;
    color: var(--text-tertiary); transition: color 0.15s;
  }
  .fav-btn:hover, .fav-btn.active { color: #fbbf24; }
  .fav-btn svg { width: 16px; height: 16px; }
  .custom-coin-form { margin-top: 16px; border-top: 1px solid var(--glass-border); padding-top: 12px; }
  .form-label { font-size: 0.8rem; color: var(--text-secondary); display: block; margin-bottom: 8px; }
  .custom-coin-row { display: flex; gap: 6px; }
  .custom-coin-row input {
    flex: 1; padding: 8px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.8rem;
  }
  .custom-coin-row input:focus { border-color: var(--accent); outline: none; }
</style>
