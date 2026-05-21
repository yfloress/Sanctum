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
  import { formatCurrency, mask } from '../lib/currency'
  import * as cryptoApi from '../lib/api/crypto'
  import PortfolioTrendChart from '../components/charts/PortfolioTrendChart.svelte'
  import DistributionChart from '../components/charts/DistributionChart.svelte'
  import CryptoTransactionModal from '../components/crypto/CryptoTransactionModal.svelte'
  import CryptoEditModal from '../components/crypto/CryptoEditModal.svelte'
  import CryptoWalletPanel from '../components/crypto/CryptoWalletPanel.svelte'
  import CryptoAssetPanel from '../components/crypto/CryptoAssetPanel.svelte'
  import CryptoTaxPanel from '../components/crypto/CryptoTaxPanel.svelte'
  import type {
    PortfolioResponse, PortfolioTrendData,
    WalletsResponse, WalletDetailResponse,
    CryptoTransactionDto, CoinCatalogDto,
    CryptoAssetPriceDto, IpcSummaryDto,
    TaxReportDto, TaxSettingsDto, TaxSummaryDto
  } from '../lib/types'

  type Tab = 'portfolio' | 'wallets' | 'activity' | 'tax'
  let activeTab = $state<Tab>('portfolio')
  let loading = $state(true)

  // Wallet icons catalog (mirrors FinancesPage ACCOUNT_ICONS)
  const WALLET_ICONS: { value: string; src: string; generic: boolean }[] = [
    ...['binance', 'bisq', 'bitmart', 'buda', 'bybit', 'kraken', 'mexc', 'retoswap', 'uniswap']
      .map(n => ({ value: `${n}.svg`, src: `/assets/exchange-icons/${n}.svg`, generic: false })),
    ...['landmark', 'wallet', 'shield', 'shield-check', 'link', 'lock']
      .map(n => ({ value: `/assets/icons/${n}.svg`, src: `/assets/icons/${n}.svg`, generic: true })),
  ]

  function getDefaultWalletIconPath(category: string): string {
    const iconMap: { [key: string]: string } = {
      'exchange': 'landmark',
      'hardware': 'shield',
      'software': 'wallet',
    }
    const icon = iconMap[category.toLowerCase()] || 'wallet'
    return `/assets/icons/${icon}.svg`
  }

  function getWalletDisplayIcon(w: { category: string; icon_path: string | null }): string {
    if (w.icon_path) {
      if (w.icon_path.startsWith('/') || w.icon_path.startsWith('http')) return w.icon_path
      return `/assets/exchange-icons/${w.icon_path}`
    }
    return getDefaultWalletIconPath(w.category)
  }

  function isGenericWalletIcon(iconPath: string | null): boolean {
    if (!iconPath) return true
    return iconPath.startsWith('/assets/icons/')
  }

  // Portfolio state
  let portfolio = $state<PortfolioResponse | null>(null)
  let trend = $state<PortfolioTrendData | null>(null)
  let trendDays = $state(30)
  let recentTxs = $state<CryptoTransactionDto[]>([])

  // Transactions tab state
  let txList = $state<CryptoTransactionDto[]>([])
  let txListHasMore = $state(false)
  let txListFilter = $state('')

  async function changeTrendDays(days: number) {
    trendDays = days
    try {
      trend = await cryptoApi.fetchPortfolioTrend(days)
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function getCryptoTxClass(tx: CryptoTransactionDto): string {
    const t = tx.transaction_type
    const s = tx.subtype
    if (t === 'income') return 'buy'
    if (t === 'expense') return 'sell'
    if (t === 'transfer') return s === 'withdrawal' ? 'sell' : s === 'deposit' ? 'buy' : 'transfer'
    if (t === 'trade') {
      if (s === 'sell') return 'sell'
      if (s === 'swap') return 'transfer'
      return 'buy'
    }
    return 'buy'
  }

  function getCryptoTxLabel(tx: CryptoTransactionDto): string {
    const t = tx.transaction_type
    const s = tx.subtype
    const detail = `${tx.amount} ${tx.symbol}`
    const args = { detail }
    if (t === 'income' || (t === 'transfer' && s === 'deposit'))
      return i18n.tArgs('crypto-tx-received', args, `Received ${detail}`)
    if (t === 'expense' || (t === 'transfer' && s === 'withdrawal'))
      return i18n.tArgs('crypto-tx-sent', args, `Sent ${detail}`)
    if (t === 'transfer')
      return i18n.tArgs('crypto-tx-transferred', args, `Transferred ${detail}`)
    if (s === 'sell')
      return i18n.tArgs('crypto-tx-sold', args, `Sold ${detail}`)
    if (s === 'swap')
      return i18n.tArgs('crypto-tx-swapped', args, `Swapped ${detail}`)
    return i18n.tArgs('crypto-tx-bought', args, `Bought ${detail}`)
  }

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

  // Edit transaction modal
  let showEditTransaction = $state(false)
  let editTxId = $state('')

  // Transaction form
  let showAddTransaction = $state(false)

  // Ticker bar
  let tickerPrices = $state<CryptoAssetPriceDto[]>([])
  let usdClpRate = $state<number | null>(null)
  let tickerSyncing = $state(false)

  // Ticker config (with Coins tab)
  let showTickerConfig = $state(false)
  let tickerConfigTab = $state<'ticker' | 'coins'>('ticker')
  let tickerIds = $state<string[]>([])
  let tickerConfigSearch = $state('')
  let catalogSearch = $state('')
  let customCoinId = $state('')
  let customCoinName = $state('')
  let customCoinSymbol = $state('')

  // Coin catalog (shared between transaction form, catalog modal and ticker config)
  let coinCatalog = $state<CoinCatalogDto[]>([])

  let tickerConfigActive = $derived(
    tickerIds.map(id => coinCatalog.find(c => c.id === id)).filter(Boolean) as typeof coinCatalog
  )

  let filteredCatalog = $derived(
    catalogSearch.length < 1 ? coinCatalog.slice(0, 100) :
    coinCatalog.filter(c =>
      c.symbol.toLowerCase().includes(catalogSearch.toLowerCase()) ||
      c.name.toLowerCase().includes(catalogSearch.toLowerCase())
    ).slice(0, 100)
  )

  let tickerConfigAvailable = $derived(
    coinCatalog.filter(c => !tickerIds.includes(c.id) && (
      tickerConfigSearch.length < 1 ||
      c.symbol.toLowerCase().includes(tickerConfigSearch.toLowerCase()) ||
      c.name.toLowerCase().includes(tickerConfigSearch.toLowerCase())
    )).slice(0, 60)
  )

  async function openTickerConfig(tab: 'ticker' | 'coins' = 'ticker') {
    await loadCoinCatalog()
    try { tickerIds = await cryptoApi.getActiveTickerIds() } catch (e) { app.showToast(String(e), true) }
    tickerConfigSearch = ''
    catalogSearch = ''
    tickerConfigTab = tab
    showTickerConfig = true
  }

  function addTicker(coinId: string) {
    if (!tickerIds.includes(coinId)) tickerIds = [...tickerIds, coinId]
    tickerConfigSearch = ''
  }

  function removeTicker(coinId: string) {
    tickerIds = tickerIds.filter(id => id !== coinId)
  }

  function moveTickerUp(i: number) {
    if (i === 0) return
    const arr = [...tickerIds];
    [arr[i - 1], arr[i]] = [arr[i], arr[i - 1]]
    tickerIds = arr
  }

  function moveTickerDown(i: number) {
    if (i === tickerIds.length - 1) return
    const arr = [...tickerIds];
    [arr[i], arr[i + 1]] = [arr[i + 1], arr[i]]
    tickerIds = arr
  }

  async function saveTickerConfig() {
    try {
      await cryptoApi.saveActiveTickerIds(tickerIds)
      showTickerConfig = false
      app.showToast(i18n.t('crypto-toast-ticker-saved', 'Ticker config saved'))
      // Update bar immediately from current tickerIds without re-fetching them from DB
      await refreshTickerBar()
    } catch (e) { app.showToast(String(e), true) }
  }

  // Rebuild tickerPrices in the order of current tickerIds.
  // Uses DB prices when available; falls back to coinCatalog entry (no price) for new coins.
  async function refreshTickerBar() {
    try {
      const prices = await cryptoApi.loadCryptoPrices()
      const priceMap = new Map(prices.map(p => [p.id, p]))
      tickerPrices = tickerIds
        .map(id => {
          const p = priceMap.get(id)
          if (p) return p
          const coin = coinCatalog.find(c => c.id === id)
          if (!coin) return null
          return {
            id: coin.id,
            symbol: coin.symbol,
            name: coin.name,
            current_price: 0,
            current_price_display: '',
            price_change_percentage_24h: 0,
            last_updated: '',
          } satisfies CryptoAssetPriceDto
        })
        .filter((p): p is CryptoAssetPriceDto => p !== null)
    } catch (_) { /* ignore */ }
  }

  async function loadTickerPrices() {
    try {
      const [ids, prices] = await Promise.all([
        cryptoApi.getActiveTickerIds(),
        cryptoApi.loadCryptoPrices(),
      ])
      tickerIds = ids
      const priceMap = new Map(prices.map(p => [p.id, p]))
      // Preserve user-configured order
      tickerPrices = ids
        .map(id => priceMap.get(id))
        .filter((p): p is CryptoAssetPriceDto => p !== undefined)
    } catch (_) { /* silently fail on initial load */ }
    try {
      const result = await cryptoApi.loadExchangeRate('USD/CLP')
      if (result) usdClpRate = result[0]
    } catch (_) { /* ignore */ }
  }

  async function syncTickerPrices() {
    tickerSyncing = true
    try {
      const ids = await cryptoApi.getMonitoredCoinIds()
      if (ids.length === 0) {
        app.showToast(i18n.t('crypto-toast-no-coins-sync', 'No coins to sync. Configure ticker first.'), true)
        return
      }
      
      const msg = await cryptoApi.syncCryptoData()
      await Promise.all([refreshTickerBar(), load()])
      app.showToast(msg)
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      tickerSyncing = false
    }
  }

  async function addCustomCoinSubmit() {
    if (!customCoinId.trim() || !customCoinName.trim() || !customCoinSymbol.trim()) return
    try {
      await cryptoApi.addCustomCoin(customCoinId, customCoinName, customCoinSymbol)
      coinCatalog = await cryptoApi.getCoinCatalog()
      customCoinId = ''
      customCoinName = ''
      customCoinSymbol = ''
      app.showToast(i18n.t('crypto-toast-custom-added', 'Custom coin added'))
    } catch (e) { app.showToast(String(e), true) }
  }

  async function deleteCustomCoinAction(id: string) {
    try {
      await cryptoApi.deleteCustomCoin(id)
      coinCatalog = await cryptoApi.getCoinCatalog()
      app.showToast(i18n.t('crypto-toast-custom-deleted', 'Custom coin deleted'))
    } catch (e) { app.showToast(String(e), true) }
  }

  async function loadCoinCatalog() {
    if (coinCatalog.length > 0) return
    try { coinCatalog = await cryptoApi.getCoinCatalog() } catch (e) { app.showToast(String(e), true) }
  }

  function openAddTransaction() {
    loadCoinCatalog()
    loadWallets()
    showAddTransaction = true
  }

  function openEditTransaction(id: string) {
    editTxId = id
    showEditTransaction = true
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
      if (activeTab === 'activity') await loadTransactionsList()
      app.showToast(i18n.t('crypto-toast-tx-deleted', 'Transaction deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function load() {
    loading = true
    try {
      const p = await cryptoApi.fetchPortfolio()
      const t = await cryptoApi.fetchPortfolioTrend(trendDays)
      portfolio = p
      trend = t
      recentTxs = (await cryptoApi.fetchAllCryptoTransactions(0, 6)).transactions
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      loading = false
    }
  }

  async function loadTransactionsList() {
    try {
      const res = await cryptoApi.fetchAllCryptoTransactions(0, 50)
      txList = res.transactions
      txListHasMore = res.has_more
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function loadMoreTransactionsList() {
    try {
      const res = await cryptoApi.fetchAllCryptoTransactions(txList.length, 50)
      txList = res.transactions
      txListHasMore = res.has_more
    } catch (e) {
      app.showToast(String(e), true)
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
      await cryptoApi.addWallet(walletName, walletCategory, walletIcon || undefined)
      showAddWallet = false
      walletName = ''
      walletIcon = ''
      showCreateWalletIconPicker = false
      await loadWallets()
      app.showToast(i18n.t('crypto-toast-wallet-created', 'Wallet created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteWallet(id: string) {
    try {
      await cryptoApi.deleteWallet(id, false)
      selectedWallet = null
      await loadWallets()
      app.showToast(i18n.t('crypto-toast-wallet-deleted', 'Wallet deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  // Create-wallet icon state
  let walletIcon = $state('')
  let showCreateWalletIconPicker = $state(false)
  let pickedWalletIconSrc = $state('')
  let pickedWalletIconGeneric = $state(true)
  $effect(() => {
    const found = walletIcon ? WALLET_ICONS.find(i => i.value === walletIcon) : null
    pickedWalletIconSrc = found ? found.src : getDefaultWalletIconPath(walletCategory)
    pickedWalletIconGeneric = found ? found.generic : true
  })

  let assetInView = $derived(
    portfolio?.assets.find(a => a.coin_id === assetCoinId)
  )

  let taxPeriodId = $state(new Date().getFullYear().toString())
  let taxReport = $state<TaxReportDto | null>(null)
  let taxSummary = $state<TaxSummaryDto | null>(null)
  let taxFillingTxId = $state<string | null>(null)
  let taxSettings = $state<TaxSettingsDto | null>(null)
  let taxReportLoading = $state(false)

  let taxCurrency = $derived(taxReport?.jurisdiction === 'chile' ? 'CLP' : app.settings?.preferred_currency ?? 'USD')

  const RESOLVABLE_PRICE_CODES = new Set([
    'missing_price', 'fee_missing_price', 'swap_missing_price', 'income_missing_price',
  ])
  let showTaxSettings = $state(false)
  let taxLoading = $state(false)
  let taxJurisdiction = $state('usa')
  let taxMethod = $state('fifo')

  const JURISDICTION_LABELS: Record<string, string> = {
    chile: 'Chile', usa: 'United States', other: 'Other',
  }
  const METHOD_LABELS: Record<string, string> = {
    fifo: 'FIFO', lifo: 'LIFO', hifo: 'HIFO', cpp: 'Average Cost',
  }
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
      app.showToast(i18n.t('crypto-toast-ipc-imported', 'IPC data imported'))
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      ipcFileInput.value = ''
    }
  }

  async function loadTaxSettings() {
    if (!taxPeriodId.trim()) {
      app.showToast(i18n.t('crypto-toast-enter-period', 'Please enter a period ID'), true)
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
      app.showToast(i18n.t('crypto-toast-enter-period', 'Please enter a period ID'), true)
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
      app.showToast(i18n.t('crypto-toast-settings-saved', 'Settings saved'))
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      taxLoading = false
    }
  }

  async function generateTaxReport() {
    if (!taxPeriodId.trim()) {
      app.showToast(i18n.t('crypto-toast-enter-period', 'Please enter a period ID'), true)
      return
    }
    taxReportLoading = true
    try {
      taxSummary = await cryptoApi.generateTaxSummary(taxPeriodId)
      taxReport = taxSummary.report
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      taxReportLoading = false
    }
  }

  // Fetches a historical USD price for the warning's transaction and persists
  // it via fill_missing_tax_prices, then regenerates the report so the warning
  // disappears in place. Strips the ":fee" suffix some warnings carry.
  async function fetchAndFillPrice(txId: string | null) {
    if (!txId) return
    const id = txId.endsWith(':fee') ? txId.slice(0, -4) : txId
    taxFillingTxId = id
    try {
      const tx = await cryptoApi.getCryptoTransaction(id)
      const price = await cryptoApi.getCryptoHistoricalPriceUsd(tx.coin_id, tx.date)
      await cryptoApi.fillMissingTaxPrices(id, price)
      await generateTaxReport()
      app.showToast(i18n.tArgs(
        'crypto-tax-toast-price-filled',
        { price: price.toFixed(4) },
        `Price filled: $${price.toFixed(4)}`
      ))
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      taxFillingTxId = null
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
      app.showToast(i18n.tArgs('crypto-toast-exported', { path }, `Exported to ${path}`))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function getCryptoIconPath(symbol: string): string {
    const normalized = symbol.toLowerCase().replace(/\s+/g, '')
    return `/assets/crypto-icons/${normalized}.svg`
  }

  $effect(() => {
    app.settings?.preferred_currency
    load()
    loadTickerPrices()
    loadWallets()
  })
  $effect(() => {
    app.settings?.preferred_currency
    if (activeTab === 'wallets') loadWallets()
  })
  $effect(() => { if (activeTab === 'tax') loadIpcSummary() })
  $effect(() => { if (activeTab === 'activity') loadTransactionsList() })
</script>

<div class="page" class:blurred={showAddWallet || showTaxSettings || selectedWallet || showAssetDetail || showAddTransaction || showEditTransaction || showTickerConfig}>
  <!-- Ticker Bar -->
  <div class="ticker-bar">
    <div class="ticker-fx">
      <span class="ticker-fx-pair">USD/CLP</span>
      <span class="ticker-fx-rate">{usdClpRate != null ? formatCurrency(usdClpRate, 'CLP') : '--'}</span>
    </div>
    <div class="ticker-prices">
      {#each tickerPrices as coin}
        <div class="ticker-coin">
          <img src={getCryptoIconPath(coin.symbol)} alt={coin.symbol} class="ticker-coin-icon" onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
          <span class="ticker-coin-sym">{coin.symbol}</span>
          {#if coin.last_updated}
            <span class="ticker-coin-price">{coin.current_price_display}</span>
            <span class="ticker-coin-change" class:negative={coin.price_change_percentage_24h < 0} class:positive={coin.price_change_percentage_24h >= 0}>
              {coin.price_change_percentage_24h >= 0 ? '+' : ''}{coin.price_change_percentage_24h.toFixed(1)}%
            </span>
          {:else}
            <span class="ticker-coin-price ticker-no-price">--</span>
          {/if}
        </div>
      {/each}
      {#if tickerPrices.length === 0}
        <span class="ticker-empty">{i18n.t('crypto-no-tickers', 'No tickers configured')}</span>
      {/if}
    </div>
    <div class="ticker-actions">
      {#if tickerPrices.length > 0 && tickerPrices[0].last_updated}
        <span class="ticker-updated" title={tickerPrices[0].last_updated}>{new Date(tickerPrices[0].last_updated).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
      {/if}
      <button class="ticker-sync-btn" onclick={syncTickerPrices} disabled={tickerSyncing} aria-label={i18n.t('crypto-sync-prices', 'Sync prices')} title={i18n.t('crypto-sync-prices', 'Sync prices')}>
        <svg class:spinning={tickerSyncing} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m0 0a9 9 0 019-9m-9 9a9 9 0 009 9"/></svg>
      </button>
      <button class="ticker-config-btn" onclick={() => openTickerConfig()} aria-label={i18n.t('crypto-configure-ticker', 'Configure ticker')} title={i18n.t('crypto-configure-ticker', 'Configure ticker')}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12.22 2h-.44a2 2 0 00-2 2v.18a2 2 0 01-1 1.73l-.43.25a2 2 0 01-2 0l-.15-.08a2 2 0 00-2.73.73l-.22.38a2 2 0 00.73 2.73l.15.1a2 2 0 011 1.72v.51a2 2 0 01-1 1.74l-.15.09a2 2 0 00-.73 2.73l.22.38a2 2 0 002.73.73l.15-.08a2 2 0 012 0l.43.25a2 2 0 011 1.73V20a2 2 0 002 2h.44a2 2 0 002-2v-.18a2 2 0 011-1.73l.43-.25a2 2 0 012 0l.15.08a2 2 0 002.73-.73l.22-.39a2 2 0 00-.73-2.73l-.15-.08a2 2 0 01-1-1.74v-.5a2 2 0 011-1.74l.15-.09a2 2 0 00.73-2.73l-.22-.38a2 2 0 00-2.73-.73l-.15.08a2 2 0 01-2 0l-.43-.25a2 2 0 01-1-1.73V4a2 2 0 00-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
      </button>
    </div>
  </div>

  <!-- Hero -->
  <section class="hero">
    <h2 class="total">{mask(portfolio?.total_value ?? '--')}</h2>
    <p class="label">{i18n.t('crypto-portfolio-value', 'Portfolio Value')}</p>
    {#if portfolio?.last_updated}
      <p class="last-updated">{i18n.tArgs('crypto-last-updated-label', { value: portfolio.last_updated }, `Last updated: ${portfolio.last_updated}`)}</p>
    {/if}
  </section>

  <!-- Tabs -->
  <div class="tab-row">
    <div class="tab-bar">
      <button class:active={activeTab === 'portfolio'} onclick={() => activeTab = 'portfolio'}>{i18n.t('crypto-tab-portfolio', 'Portfolio')}</button>
      <button class:active={activeTab === 'wallets'} onclick={() => activeTab = 'wallets'}>{i18n.t('crypto-tab-wallets', 'Wallets')}</button>
      <button class:active={activeTab === 'activity'} onclick={() => activeTab = 'activity'}>{i18n.t('crypto-tab-activity', 'Activity')}</button>
      <button class:active={activeTab === 'tax'} onclick={() => activeTab = 'tax'}>{i18n.t('crypto-tab-tax', 'Tax')}</button>
    </div>
  </div>

  {#if loading}
    <div class="skeleton-page">
      <div style="text-align:center;margin-bottom:24px">
        <div class="skeleton" style="width:180px;height:42px;margin:0 auto 8px"></div>
        <div class="skeleton" style="width:120px;height:18px;margin:0 auto"></div>
      </div>
      <div class="skeleton-row" style="justify-content:center;gap:24px;margin-bottom:24px">
        <div class="skeleton" style="width:80px;height:48px;border-radius:var(--radius-sm)"></div>
        <div class="skeleton" style="width:80px;height:48px;border-radius:var(--radius-sm)"></div>
        <div class="skeleton" style="width:80px;height:48px;border-radius:var(--radius-sm)"></div>
      </div>
      <div class="skeleton-grid" style="grid-template-columns:repeat(auto-fill,minmax(180px,1fr))">
        {#each Array(6) as _}
          <div class="skeleton" style="height:96px;border-radius:var(--radius-md)"></div>
        {/each}
      </div>
      <div class="skeleton" style="width:100%;height:200px;border-radius:var(--radius-lg)"></div>
    </div>

  <!-- PORTFOLIO TAB -->
  {:else if activeTab === 'portfolio' && portfolio}
    <div class="section-header">
      <span></span>
      <div class="header-actions">
        <button class="glass-btn" onclick={openAddTransaction}>{i18n.t('crypto-new-transaction', 'New Transaction')}</button>
      </div>
    </div>
    <!-- Stats bar -->
    <div class="stats-bar">
      <div class="stat">
        <span class="stat-lbl">{i18n.t('crypto-unrealized-pnl', 'Unrealized P&L')}</span>
        <span class="stat-val" class:negative={portfolio.unrealized_pnl_negative} class:positive={!portfolio.unrealized_pnl_negative}>
          {mask(portfolio.unrealized_pnl)}
        </span>
      </div>
      <div class="stat">
        <span class="stat-lbl">{i18n.t('crypto-realized-ytd', 'Realized YTD')}</span>
        <span class="stat-val" class:negative={portfolio.realized_ytd_negative} class:positive={!portfolio.realized_ytd_negative}>
          {mask(portfolio.realized_ytd)}
        </span>
      </div>
      <div class="stat">
        <span class="stat-lbl">{i18n.t('crypto-roi', 'ROI')}</span>
        <span class="stat-val" class:negative={portfolio.roi_negative} class:positive={!portfolio.roi_negative}>
          {portfolio.roi}
        </span>
      </div>
    </div>

    <!-- Holdings -->
    {#if portfolio.assets.length === 0}
      <p class="empty">{i18n.t('crypto-no-assets-empty', 'No assets yet. Create a wallet and add transactions to get started.')}</p>
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
              <span class="asset-amount">{mask(asset.amount)}</span>
              <span class="asset-value">{mask(asset.value)}</span>
            </div>
          </button>
        {/each}
      </div>

      <!-- Portfolio Trend Chart -->
      {#if trend && trend.dates.length > 0}
        <div class="chart-section">
          <div class="chart-card-header">
            <h3>{i18n.t('crypto-portfolio-trend', 'Portfolio Trend')}</h3>
            <div class="range-picker">
              {#each [30, 90, 180, 365] as d}
                <button class:active={trendDays === d} onclick={() => changeTrendDays(d)}>{d}d</button>
              {/each}
            </div>
          </div>
          <PortfolioTrendChart data={trend} />
        </div>
      {/if}

      <!-- Distribution Chart -->
      {#if portfolio.distribution.length > 0}
        <div class="chart-section">
          <h3>{i18n.t('crypto-distribution', 'Distribution')}</h3>
          <DistributionChart data={portfolio.distribution} />
        </div>
      {/if}
    {/if}

    <!-- Recent Transactions -->
    <div class="chart-section">
      <div class="chart-card-header">
        <h3>{i18n.t('crypto-recent-transactions', 'Recent Transactions')}</h3>
      </div>
      {#if recentTxs.length === 0}
        <p class="empty">{i18n.t('crypto-no-transactions', 'No transactions yet.')}</p>
      {:else}
        <div class="tx-list">
          {#each recentTxs as tx}
            <div class="tx-row" role="button" tabindex="0"
              onclick={() => openEditTransaction(tx.id)}
              onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') openEditTransaction(tx.id) }}>
              <span class="tx-type-dot" class:buy={getCryptoTxClass(tx) === 'buy'} class:sell={getCryptoTxClass(tx) === 'sell'} class:transfer={getCryptoTxClass(tx) === 'transfer'}></span>
              <img src={getCryptoIconPath(tx.symbol)} alt={tx.symbol} class="tx-crypto-icon" onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
              <div class="tx-main">
                <span class="tx-desc">{getCryptoTxLabel(tx)}</span>
                <div class="tx-meta">
                  <span class="tx-acc">{tx.wallet_name || '?'}</span>
                  <span class="tx-date">{tx.date}</span>
                </div>
              </div>
              <span class="tx-amount" class:buy={getCryptoTxClass(tx) === 'buy'} class:sell={getCryptoTxClass(tx) === 'sell'} class:transfer={getCryptoTxClass(tx) === 'transfer'}>{mask(tx.value)}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>

  <!-- WALLETS TAB -->
  {:else if activeTab === 'wallets'}
    <div class="section-header">
      <h3>{i18n.t('crypto-wallets-title', 'Wallets')}</h3>
      <div class="header-actions">
        <button class="glass-btn" onclick={openAddTransaction}>{i18n.t('crypto-new-transaction', 'New Transaction')}</button>
        <button class="glass-btn" onclick={() => { showAddWallet = true; walletName = '' }}>{i18n.t('crypto-add-wallet', 'Add Wallet')}</button>
      </div>
    </div>

    {#if (walletsData?.wallets ?? []).length === 0}
      <p class="empty">{i18n.t('crypto-no-wallets', 'No wallets yet.')}</p>
    {:else}
      <div class="wallet-grid">
        {#each walletsData?.wallets ?? [] as w}
          <button class="wallet-card" onclick={() => openWalletDetail(w.id)}>
            <img src={getWalletDisplayIcon(w)} alt="" class="wallet-icon" class:themed-icon={isGenericWalletIcon(w.icon_path)} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
            <div class="wallet-name">{w.name}</div>
            <div class="wallet-cat">{w.category}</div>
            <div class="wallet-val">{mask(w.total_value)}</div>
            <div class="wallet-count">{w.assets_count} {w.assets_count !== 1 ? i18n.t('crypto-wallet-assets-other', 'assets') : i18n.t('crypto-wallet-assets-one', 'asset')}</div>
          </button>
        {/each}
      </div>
    {/if}

  <!-- TRANSACTIONS TAB -->
  {:else if activeTab === 'activity'}
    <section class="tab-content">
      <div class="activity-toolbar">
        <div class="filter-search">
          <svg class="filter-search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
          </svg>
          <input
            type="text"
            placeholder={i18n.t('crypto-search-transactions', 'Search transactions...')}
            bind:value={txListFilter}
          />
        </div>
        <button class="glass-btn" onclick={openAddTransaction}>{i18n.t('crypto-new-transaction', 'New Transaction')}</button>
      </div>

      {#if txList.length === 0}
        <p class="empty">{txListFilter ? i18n.t('crypto-no-matching', 'No matching transactions') : i18n.t('crypto-no-transactions', 'No transactions yet.')}</p>
      {:else}
        <div class="tx-list">
          {#each txList.filter(tx => !txListFilter || tx.symbol.toLowerCase().includes(txListFilter.toLowerCase()) || tx.wallet_name?.toLowerCase().includes(txListFilter.toLowerCase()) || tx.transaction_type.toLowerCase().includes(txListFilter.toLowerCase())) as tx (tx.id)}
            <div class="tx-row" role="button" tabindex="0"
              onclick={() => openEditTransaction(tx.id)}
              onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') openEditTransaction(tx.id) }}>
              <span class="tx-type-dot" class:buy={getCryptoTxClass(tx) === 'buy'} class:sell={getCryptoTxClass(tx) === 'sell'} class:transfer={getCryptoTxClass(tx) === 'transfer'}></span>
              <img src={getCryptoIconPath(tx.symbol)} alt={tx.symbol} class="tx-crypto-icon" onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
              <div class="tx-main">
                <span class="tx-desc">{getCryptoTxLabel(tx)}</span>
                <div class="tx-meta">
                  <span class="tx-acc">{tx.wallet_name || '?'}</span>
                  <span class="tx-date">{tx.date}</span>
                </div>
              </div>
              <span class="tx-amount" class:buy={getCryptoTxClass(tx) === 'buy'} class:sell={getCryptoTxClass(tx) === 'sell'} class:transfer={getCryptoTxClass(tx) === 'transfer'}>{mask(tx.value)}</span>
              <button class="delete-btn" onclick={(e: MouseEvent) => { e.stopPropagation(); deleteCryptoTx(tx.id) }} aria-label={i18n.t('crypto-delete', 'Delete')}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
              </button>
            </div>
          {/each}
        </div>
        {#if txListHasMore}
          <button class="load-more-btn" onclick={loadMoreTransactionsList}>{i18n.t('crypto-load-more', 'Load More')}</button>
        {/if}
      {/if}
    </section>

  <!-- TAX TAB -->
  {:else if activeTab === 'tax'}
    <div class="tax-section">
      {#if !taxSettings}
        <!-- Empty state / onboarding -->
        <div class="tax-onboarding">
          <div class="tax-onboarding-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
          </div>
          <h3>{i18n.t('crypto-tax-onboarding-title', 'Tax Reporting')}</h3>
          <p class="tax-onboarding-desc">{i18n.t('crypto-tax-onboarding-desc', 'Generate a tax report for your crypto transactions. Follow the steps below to get started.')}</p>
          <div class="tax-steps">
            <div class="tax-step">
              <span class="tax-step-num">1</span>
              <div>
                <strong>{i18n.t('crypto-tax-step1-title', 'Enter tax year')}</strong>
                <p>{i18n.t('crypto-tax-step1-desc', 'Type the year to report (e.g. 2024) and load your settings.')}</p>
              </div>
            </div>
            <div class="tax-step">
              <span class="tax-step-num">2</span>
              <div>
                <strong>{i18n.t('crypto-tax-step2-title', 'Configure jurisdiction & method')}</strong>
                <p>{i18n.t('crypto-tax-step2-desc', 'Select your tax jurisdiction, cost basis method, and optional settings. For Chile, import IPC data.')}</p>
              </div>
            </div>
            <div class="tax-step">
              <span class="tax-step-num">3</span>
              <div>
                <strong>{i18n.t('crypto-tax-step3-title', 'Generate & export')}</strong>
                <p>{i18n.t('crypto-tax-step3-desc', 'Generate the report, review warnings, fix missing prices, and export CSV for your filing.')}</p>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <!-- Step 1: Period selector -->
      <div class="period-selector">
        <div class="period-input-group">
          <label>
            <span class="period-label">{i18n.t('crypto-tax-period-id', 'Tax Period')}</span>
            <input type="text" bind:value={taxPeriodId} placeholder={i18n.t('crypto-tax-period-placeholder', 'e.g., 2024')} />
          </label>
          <div class="period-actions">
            <button class="glass-btn" onclick={loadTaxSettings} disabled={taxLoading}>
              {taxLoading ? i18n.t('crypto-tax-loading-settings', 'Loading…') : i18n.t('crypto-tax-load-settings', 'Load Settings')}
            </button>
            {#if taxSettings}
              <button class="glass-btn" onclick={() => showTaxSettings = true}>{i18n.t('crypto-tax-configure', 'Configure')}</button>
            {/if}
          </div>
        </div>
      </div>

      {#if taxSettings}
        <!-- Configuration cards -->
        <div class="tax-config-row">
          <!-- Settings card -->
          <div class="settings-info">
            <h4 class="tax-card-title">{i18n.t('crypto-tax-settings-title', 'Tax Settings')}</h4>
            <div class="info-grid">
              <div class="info-item">
                <span class="label">{i18n.t('crypto-tax-jurisdiction', 'Jurisdiction')}</span>
                <span class="value">{JURISDICTION_LABELS[taxSettings.jurisdiction] ?? taxSettings.jurisdiction}</span>
              </div>
              <div class="info-item">
                <span class="label">{i18n.t('crypto-tax-method', 'Method')}</span>
                <span class="value">{METHOD_LABELS[taxSettings.method] ?? taxSettings.method}</span>
              </div>
              <div class="info-item">
                <span class="label">{i18n.t('crypto-tax-include-swaps', 'Include Swaps')}</span>
                <span class="value">{taxSettings.include_swaps ? i18n.t('crypto-tax-yes', 'Yes') : i18n.t('crypto-tax-no', 'No')}</span>
              </div>
              <div class="info-item">
                <span class="label">{i18n.t('crypto-tax-include-fee-crypto', 'Include Fee Crypto')}</span>
                <span class="value">{taxSettings.include_fee_crypto ? i18n.t('crypto-tax-yes', 'Yes') : i18n.t('crypto-tax-no', 'No')}</span>
              </div>
              {#if taxSettings.excluded_wallet_ids.length > 0}
                <div class="info-item wide">
                  <span class="label">{i18n.t('crypto-tax-exclude-wallets', 'Excluded Wallets')}</span>
                  <span class="value">{taxSettings.excluded_wallet_ids.length} {taxSettings.excluded_wallet_ids.length === 1 ? 'wallet' : 'wallets'}</span>
                </div>
              {/if}
            </div>
          </div>

          <!-- IPC card (Chile only visually prominent, but always shown) -->
          <div class="ipc-section">
            <h4 class="tax-card-title">{i18n.t('crypto-ipc-label', 'IPC Price History')}</h4>
            <p class="ipc-desc">{i18n.t('crypto-ipc-desc', 'Chile requires monthly IPC data for inflation adjustment. Import a CSV from INE.')}</p>
            <div class="ipc-status-row">
              {#if ipcSummary && ipcSummary.records_count > 0}
                <span class="ipc-status ok">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="ipc-status-icon"><path d="M20 6L9 17l-5-5"/></svg>
                  {ipcSummary.records_count} records {ipcSummary.date_range ? `(${ipcSummary.date_range})` : ''}
                </span>
              {:else}
                <span class="ipc-status warn">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="ipc-status-icon"><path d="M12 9v4M12 17h.01"/><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/></svg>
                  {i18n.t('crypto-ipc-no-data', 'No IPC data imported')}
                </span>
              {/if}
              <input type="file" accept=".csv" class="hidden-input" bind:this={ipcFileInput} onchange={handleIpcFile} />
              <button class="glass-btn" onclick={() => ipcFileInput.click()}>{i18n.t('crypto-ipc-import', 'Import IPC CSV')}</button>
            </div>
          </div>
        </div>

        <!-- Chile-specific info -->
        {#if taxSettings.jurisdiction === 'chile'}
          <div class="chile-info">
            <div class="chile-info-header">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="chile-info-icon"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
              <span>{i18n.t('crypto-tax-chile-info-title', 'Chile Tax Notes')}</span>
            </div>
            <ul class="chile-info-list">
              <li>{i18n.t('crypto-tax-chile-ipc', 'IPC adjustments (correccion monetaria) are applied to cost basis and gains automatically.')}</li>
              <li>{i18n.t('crypto-tax-chile-clp', 'All values in this report are shown in Chilean Pesos (CLP). For filing, use the Dolar Observado published by SII.')}</li>
              <li>{i18n.t('crypto-tax-chile-f22', 'File under Formulario 22, Linea 7 (Mayor Valor). Verify current casilla codes with the SII suplemento tributario.')}</li>
              <li>{i18n.t('crypto-tax-chile-exemption', 'Net annual income under 13.5 UTA (~$11.3M CLP in 2026) is exempt from IGC.')}</li>
              <li>{i18n.t('crypto-tax-chile-fees', 'Fees and commissions treatment may vary. Consult a Chilean tax professional for your specific situation.')}</li>
            </ul>
          </div>
        {/if}

        <!-- Generate report -->
        {#if !taxReportLoading}
          <div class="report-actions">
            <button class="primary-btn generate-report-btn" onclick={generateTaxReport}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:14px;height:14px"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><path d="M14 2v6h6M16 13H8M16 17H8M10 9H8"/></svg>
              {i18n.t('crypto-tax-generate-report', 'Generate Report')}
            </button>
            {#if taxReport}
              <button class="glass-btn" onclick={generateTaxReport}>{i18n.t('crypto-tax-regenerate', 'Regenerate')}</button>
            {/if}
          </div>
        {/if}
      {/if}

      <!-- Report loading skeleton -->
      {#if taxReportLoading}
        <div class="report-skeleton">
          <div class="skeleton" style="width:60%;height:24px;margin-bottom:16px"></div>
          <div class="skeleton" style="width:100%;height:120px;margin-bottom:16px;border-radius:var(--radius-md)"></div>
          <div class="skeleton" style="width:100%;height:80px;margin-bottom:16px;border-radius:var(--radius-md)"></div>
          <div class="skeleton" style="width:100%;height:200px;border-radius:var(--radius-md)"></div>
        </div>
      {/if}

      {#if taxReport && !taxReportLoading}
        <!-- Report summary -->
        <div class="report-summary">
          <div class="tax-card-header">
            <h3>{i18n.t('crypto-tax-report-summary', 'Report Summary')}</h3>
            <span class="tax-currency-badge">({taxCurrency})</span>
          </div>
          <div class="summary-grid">
            <div class="summary-item">
              <span class="label">{i18n.t('crypto-tax-disposals', 'Disposals')}</span>
              <span class="value">{taxReport.disposals_count}</span>
            </div>
            <div class="summary-item">
              <span class="label">{i18n.t('crypto-tax-total-proceeds', 'Total Proceeds')}</span>
              <span class="value">{mask(taxReport.total_proceeds)}</span>
            </div>
            <div class="summary-item">
              <span class="label">{i18n.t('crypto-tax-total-cost', 'Total Cost')}</span>
              <span class="value">{mask(taxReport.total_cost)}</span>
            </div>
            <div class="summary-item highlight">
              <span class="label">{i18n.t('crypto-tax-total-gain', 'Total Gain')}</span>
              <span class="value" class:negative={taxReport.total_gain_negative}>{mask(taxReport.total_gain)}</span>
            </div>
            {#if taxReport.jurisdiction !== 'chile'}
              {#if taxReport.short_term_gain}
                <div class="summary-item">
                  <span class="label">{i18n.t('crypto-tax-short-term', 'Short-term Gain')}</span>
                  <span class="value">{mask(taxReport.short_term_gain)}</span>
                </div>
              {/if}
              {#if taxReport.long_term_gain}
                <div class="summary-item">
                  <span class="label">{i18n.t('crypto-tax-long-term', 'Long-term Gain')}</span>
                  <span class="value">{mask(taxReport.long_term_gain)}</span>
                </div>
              {/if}
            {/if}
            {#if taxSummary}
              <div class="summary-item">
                <span class="label">{i18n.t('crypto-tax-taxable-income', 'Taxable Income')}</span>
                <span class="value">{mask(taxSummary.taxable_income_total)}</span>
              </div>
              <div class="summary-item">
                <span class="label">{i18n.t('crypto-tax-end-balance', 'End-of-period Balance')}</span>
                <span class="value">{mask(taxSummary.end_balance_value ?? '—')}</span>
              </div>
              <div class="summary-item">
                <span class="label">{i18n.t('crypto-tax-tx-in-period', 'Transactions in Period')}</span>
                <span class="value">{taxSummary.transactions_in_period}</span>
              </div>
              <div class="summary-item">
                <span class="label">{i18n.t('crypto-tax-volume', 'Volume Processed')}</span>
                <span class="value">{taxSummary.volume_processed}</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Readiness checklist -->
        {#if taxReport.readiness && taxReport.readiness.length > 0}
          <div class="readiness">
            <h4>{i18n.t('crypto-tax-readiness', 'Readiness')}</h4>
            {#each taxReport.readiness as r}
              <div class="readiness-item" class:ok={r.status === 'ok'} class:warn={r.status === 'warn'} class:error={r.status === 'error'} class:info={r.status === 'info'}>
                <span class="status-badge" class:ok={r.status === 'ok'} class:warn={r.status === 'warn'} class:error={r.status === 'error'} class:info={r.status === 'info'}>{r.status}</span>
                <span class="code">{r.code}</span>
                <span class="detail">{r.detail}</span>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Warnings -->
        {#if taxReport.warnings && taxReport.warnings.length > 0}
          <div class="warnings">
            <h4>{i18n.t('crypto-tax-warnings', 'Warnings')} ({taxReport.warnings.length})</h4>
            {#each taxReport.warnings as w}
              <div class="warning-item">
                <span class="warning-code">{w.code}</span>
                <span class="warning-msg">{w.message}</span>
                {#if w.tx_id && RESOLVABLE_PRICE_CODES.has(w.code)}
                  <button class="warning-action" onclick={() => fetchAndFillPrice(w.tx_id)} disabled={taxFillingTxId === w.tx_id}>
                    {taxFillingTxId === w.tx_id
                      ? i18n.t('crypto-tax-fetching', 'Fetching…')
                      : i18n.t('crypto-tax-fetch-price', 'Fetch price')}
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        {/if}

        <!-- Export -->
        <div class="export-actions">
          <button onclick={() => exportTaxReport('csv')} class="export-btn">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:14px;height:14px"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>
            {i18n.t('crypto-tax-export-events', 'Export Events CSV')}
          </button>
          <button onclick={() => exportTaxReport('history')} class="export-btn">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:14px;height:14px"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>
            {i18n.t('crypto-tax-export-history', 'Export History CSV')}
          </button>
        </div>

        <!-- Events table -->
        {#if taxReport.events && taxReport.events.length > 0}
          <div class="events-table">
            <h4>{i18n.t('crypto-tax-events', 'Events')} <span class="events-count">({Math.min(taxReport.events.length, 50)}{taxReport.events.length > 50 ? '/ ' + taxReport.events.length : ''})</span></h4>
            <div class="table-wrapper">
              <table>
                <thead>
                  <tr>
                    <th>{i18n.t('crypto-tax-col-date', 'Date')}</th>
                    <th>{i18n.t('crypto-tax-col-coin', 'Coin')}</th>
                    <th>{i18n.t('crypto-tax-col-amount', 'Amount')}</th>
                    <th>{i18n.t('crypto-tax-col-proceeds', 'Proceeds')}</th>
                    <th>{i18n.t('crypto-tax-col-cost-basis', 'Cost Basis')}</th>
                    <th>{i18n.t('crypto-tax-col-gain', 'Gain')}</th>
                    <th>{i18n.t('crypto-tax-col-term', 'Term')}</th>
                  </tr>
                </thead>
                <tbody>
                  {#each taxReport.events.slice(0, 50) as e}
                    <tr class:loss={e.gain_negative}>
                      <td>{e.date}</td>
                      <td>{e.symbol}</td>
                      <td>{mask(e.amount)}</td>
                      <td>{mask(e.proceeds)}</td>
                      <td>{mask(e.cost_basis)}</td>
                      <td class:negative={e.gain_negative}>{mask(e.gain)}</td>
                      <td>{e.term ?? '-'}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      {/if}
    </div>

  {/if}
</div>

<!-- Tax Settings Modal -->
<CryptoTaxPanel
  bind:show={showTaxSettings}
  bind:taxJurisdiction={taxJurisdiction}
  bind:taxMethod={taxMethod}
  bind:taxIncludeSwaps={taxIncludeSwaps}
  bind:taxIncludeFeeCrypto={taxIncludeFeeCrypto}
  bind:taxExcludedWalletIds={taxExcludedWalletIds}
  taxLoading={taxLoading}
  wallets={walletsData?.wallets ?? []}
  onsave={saveTaxSettings}
  onclose={() => showTaxSettings = false}
/>

<!-- Wallet Detail Panel -->
<CryptoWalletPanel
  show={selectedWallet !== null}
  wallet={selectedWallet}
  ondelete={deleteWallet}
  onedit={(id) => openEditTransaction(id)}
  ondeleteTx={deleteCryptoTx}
  onrefresh={async () => { const id = selectedWallet?.id; if (id) selectedWallet = await cryptoApi.fetchWalletDetail(id); await loadWallets() }}
  onclose={() => selectedWallet = null}
/>

<!-- Asset Detail Overlay -->
<CryptoAssetPanel
  show={showAssetDetail && assetInView !== undefined}
  asset={assetInView ?? null}
  transactions={assetTransactions}
  onedit={(id) => openEditTransaction(id)}
  ondeleteTx={deleteCryptoTx}
  onclose={() => showAssetDetail = false}
/>

<!-- Add Wallet Modal -->
{#if showAddWallet}
  <div class="modal-backdrop" role="presentation" onclick={() => showAddWallet = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddWallet = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{i18n.t('crypto-new-wallet', 'New Wallet')}</h3>
    <div class="form-grid">
      <label>
        {i18n.t('crypto-wallet-name', 'Name')}
        <input type="text" bind:value={walletName} placeholder={i18n.t('crypto-wallet-name-placeholder', 'Wallet name')} />
      </label>
      <label>
        {i18n.t('crypto-wallet-category', 'Category')}
        <div class="category-cards">
          {#each ['exchange', 'hardware', 'software'] as cat}
            <button class="cat-card" class:selected={walletCategory === cat} onclick={() => walletCategory = cat}>
              {cat}
            </button>
          {/each}
        </div>
      </label>
      <div class="icon-select-label">
        <span>{i18n.t('crypto-wallet-icon', 'Icon')}</span>
        <button class="change-icon-btn" onclick={() => showCreateWalletIconPicker = !showCreateWalletIconPicker}>
          <img src={pickedWalletIconSrc || getDefaultWalletIconPath(walletCategory)} alt="" class="selected-icon-preview" class:themed-icon={pickedWalletIconGeneric} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
          {showCreateWalletIconPicker ? i18n.t('crypto-close', 'Close') : i18n.t('crypto-change', 'Change')}
        </button>
      </div>
      {#if showCreateWalletIconPicker}
        <div class="icon-picker">
          {#each WALLET_ICONS as icon}
            <button class="icon-option" class:selected={walletIcon === icon.value} onclick={() => { walletIcon = icon.value; showCreateWalletIconPicker = false }} title={icon.value}>
              <img src={icon.src} alt={icon.value} class:themed-icon={icon.generic} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
            </button>
          {/each}
        </div>
      {/if}
    </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showAddWallet = false}>{i18n.t('crypto-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitWallet} disabled={!walletName.trim()}>{i18n.t('crypto-wallet-create', 'Create')}</button>
      </div>
    </div>
  </div>
{/if}

<!-- Ticker Config / Coins Modal -->
{#if showTickerConfig}
  <div class="modal-backdrop" role="presentation" onclick={() => showTickerConfig = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showTickerConfig = false }}></div>
  <div class="modal-wrapper">
    <div class="modal wide">
      <!-- Tab bar -->
      <div class="cfg-tabs">
        <button class="cfg-tab" class:active={tickerConfigTab === 'ticker'} onclick={() => tickerConfigTab = 'ticker'}>{i18n.t('crypto-ticker-tab', 'Ticker')}</button>
        <button class="cfg-tab" class:active={tickerConfigTab === 'coins'} onclick={() => tickerConfigTab = 'coins'}>{i18n.t('crypto-coins-tab', 'Coins')}</button>
      </div>

      {#if tickerConfigTab === 'ticker'}
        <!-- Active tickers (ordered) -->
        <p class="tc-section-label">{i18n.t('crypto-ticker-active', 'Active — use arrows to reorder')}</p>
        {#if tickerConfigActive.length === 0}
          <p class="tc-empty">{i18n.t('crypto-ticker-no-selected', 'No tickers selected yet.')}</p>
        {:else}
          <div class="tc-active-list">
            {#each tickerConfigActive as coin, i}
              <div class="tc-active-item">
                <div class="tc-order-btns">
                  <button class="tc-arrow" onclick={() => moveTickerUp(i)} disabled={i === 0} aria-label="Move up">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M5 15l7-7 7 7"/></svg>
                  </button>
                  <button class="tc-arrow" onclick={() => moveTickerDown(i)} disabled={i === tickerIds.length - 1} aria-label="Move down">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M19 9l-7 7-7-7"/></svg>
                  </button>
                </div>
                <img src={getCryptoIconPath(coin.symbol)} alt={coin.symbol} class="tc-icon" onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
                <span class="tc-sym">{coin.symbol}</span>
                <span class="tc-name">{coin.name}</span>
                <button class="tc-remove" onclick={() => removeTicker(coin.id)} aria-label="Remove">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
                </button>
              </div>
            {/each}
          </div>
        {/if}

        <p class="tc-section-label" style="margin-top: 16px;">{i18n.t('crypto-ticker-add-coins', 'Add coins')}</p>
        <input type="text" class="catalog-search" bind:value={tickerConfigSearch} placeholder={i18n.t('crypto-ticker-search', 'Search coins...')} />
        <div class="tc-available-list">
          {#each tickerConfigAvailable as coin}
            <button class="tc-available-item" onclick={() => addTicker(coin.id)}>
              <img src={getCryptoIconPath(coin.symbol)} alt={coin.symbol} class="tc-icon" onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
              <span class="tc-sym">{coin.symbol}</span>
              <span class="tc-name">{coin.name}</span>
              <svg class="tc-add-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
            </button>
          {/each}
        </div>

        <div class="modal-actions">
          <button class="secondary-btn" onclick={() => showTickerConfig = false}>{i18n.t('crypto-cancel', 'Cancel')}</button>
          <button class="primary-btn" onclick={saveTickerConfig}>{i18n.t('crypto-ticker-save', 'Save')}</button>
        </div>

      {:else}
        <!-- Coins tab -->
        <input type="text" class="catalog-search" bind:value={catalogSearch} placeholder={i18n.t('crypto-ticker-search', 'Search coins...')} />
        <div class="catalog-list">
          {#each filteredCatalog as coin}
            <div class="catalog-item">
              <img src={getCryptoIconPath(coin.symbol)} alt={coin.symbol} class="catalog-icon" onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
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
          <span class="form-label">{i18n.t('crypto-custom-coin', 'Add Custom Coin')}</span>
          <div class="custom-coin-row">
            <input type="text" bind:value={customCoinId} placeholder={i18n.t('crypto-custom-id', 'ID')} />
            <input type="text" bind:value={customCoinName} placeholder={i18n.t('crypto-custom-name', 'Name')} />
          </div>
          <div class="custom-coin-row-bottom">
            <input type="text" bind:value={customCoinSymbol} placeholder={i18n.t('crypto-custom-symbol', 'Symbol')} />
            <button class="primary-btn" onclick={addCustomCoinSubmit} disabled={!customCoinId.trim() || !customCoinSymbol.trim()}>{i18n.t('crypto-custom-add', 'Add')}</button>
          </div>
        </div>

        <div class="modal-actions">
          <button class="secondary-btn" onclick={() => showTickerConfig = false}>Close</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<!-- Add Crypto Transaction Modal -->
<CryptoTransactionModal
  bind:show={showAddTransaction}
  wallets={walletsData?.simple_list ?? []}
  coinCatalog={coinCatalog}
  onsubmit={async () => { await load(); if (activeTab === 'wallets') await loadWallets(); if (activeTab === 'activity') await loadTransactionsList() }}
  onclose={() => showAddTransaction = false}
/>

<!-- Edit Crypto Transaction Modal -->
<CryptoEditModal
  bind:show={showEditTransaction}
  txId={editTxId}
  onsubmit={async () => {
    if (showAssetDetail) assetTransactions = await cryptoApi.getCryptoTransactionsByCoin(assetCoinId)
    if (selectedWallet) selectedWallet = await cryptoApi.fetchWalletDetail(selectedWallet.id)
    await load()
    if (activeTab === 'activity') await loadTransactionsList()
  }}
  onclose={() => showEditTransaction = false}
/>

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
  .ticker-coin-icon { width: 14px; height: 14px; border-radius: 50%; flex-shrink: 0; }
  .ticker-coin-sym { font-size: 0.75rem; font-weight: 700; color: var(--text-secondary); }
  .ticker-coin-price { font-size: 0.85rem; color: var(--text-primary); font-weight: 500; }
  .ticker-no-price { color: var(--text-tertiary); }
  .ticker-coin-change { font-size: 0.75rem; }
  .ticker-coin-change.positive { color: var(--success); }
  .ticker-coin-change.negative { color: var(--danger); }
  .ticker-empty { padding: 10px 16px; font-size: 0.8rem; color: var(--text-tertiary); }
  .ticker-updated { font-size: 0.65rem; color: var(--text-tertiary); white-space: nowrap; }
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

  .skeleton-page { padding: 8px 0; }
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
    position: relative;
    display: flex; flex-direction: column; gap: 6px; padding: 14px;
    background: var(--card-bg); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    cursor: pointer; text-align: left; color: inherit;
    transition: all 0.2s; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .asset-card:hover { border-color: var(--glass-border-hover); box-shadow: var(--glass-shadow-lg); }
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
    position: relative;
    background: var(--card-bg); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 16px; margin-bottom: 24px;
    box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .chart-section::before {
    content: ''; position: absolute;
    top: 0; left: 0; right: 0; height: 1px;
    background: var(--card-accent-line); opacity: 0.5;
  }
  .chart-section h3 { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.08em; margin: 0; }
  .chart-card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
  .range-picker {
    display: flex; gap: 2px;
    background: var(--glass-active); border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); padding: 2px;
  }
  .range-picker button {
    padding: 4px 12px; border: none; border-radius: 6px;
    background: none; color: var(--text-tertiary); cursor: pointer;
    font-size: 0.72rem; font-weight: 500; transition: all 0.15s;
  }
  .range-picker button:hover { color: var(--text-primary); }
  .range-picker button.active {
    background: var(--glass-elevated); color: var(--text-primary);
    box-shadow: 0 1px 4px rgba(0,0,0,0.25);
  }

  /* Wallets */
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .section-header h3 { font-size: 0.85rem; color: var(--text-secondary); text-transform: uppercase; margin: 0; }

  .wallet-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px;
  }
  .wallet-card {
    position: relative;
    display: flex; flex-direction: column; gap: 4px; padding: 16px;
    background: var(--card-bg); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    cursor: pointer; text-align: left; color: inherit;
    transition: all 0.2s; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .wallet-card:hover { border-color: var(--glass-border-hover); box-shadow: var(--glass-shadow-lg); }
  .wallet-icon { width: 32px; height: 32px; border-radius: 4px; margin-bottom: 6px; }
  .wallet-name { font-weight: 600; color: var(--text-primary); }
  .wallet-cat { font-size: 0.75rem; color: var(--text-tertiary); text-transform: capitalize; }
  .wallet-val { font-size: 1.1rem; font-weight: 600; color: var(--text-primary); margin-top: 6px; }
  .wallet-count { font-size: 0.75rem; color: var(--text-tertiary); }


  .icon-picker {
    display: grid; grid-template-columns: repeat(6, 1fr); gap: 6px;
    padding: 10px; background: var(--glass); border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); margin-bottom: 12px;
  }
  .icon-option {
    width: 36px; height: 36px; padding: 4px; border: 1px solid transparent; border-radius: var(--radius-sm);
    background: none; cursor: pointer; display: flex; align-items: center; justify-content: center;
    transition: border-color 0.15s, background 0.15s;
  }
  .icon-option:hover { border-color: var(--accent-border); background: var(--glass-active); }
  .icon-option.selected { border-color: var(--accent-border); background: var(--glass-active); }
  .icon-option img { width: 100%; height: 100%; object-fit: contain; border-radius: 3px; }
  .change-icon-btn {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 6px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.8rem;
    transition: all 0.15s;
  }
  .change-icon-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .selected-icon-preview { width: 20px; height: 20px; margin-right: 6px; vertical-align: middle; }
  .icon-select-label {
    display: flex; align-items: center; justify-content: space-between;
    font-size: 0.8rem; color: var(--text-secondary);
  }

  .themed-icon { filter: brightness(0) invert(1); }
  :global(.light-mode) .themed-icon { filter: brightness(0); }

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
  .modal.wide { width: 480px; }

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
  /* Light mode: dark text on a soft tonal button (white text would vanish). */
  :global(.light-mode) .primary-btn {
    background: rgba(139, 92, 246, 0.18);
    border-color: rgba(139, 92, 246, 0.38);
    color: var(--text-primary);
  }
  :global(.light-mode) .primary-btn:hover:not(:disabled) {
    background: rgba(139, 92, 246, 0.3);
    color: var(--text-primary);
  }
  .secondary-btn {
    padding: 8px 18px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .secondary-btn:hover { border-color: var(--glass-border-hover); }

  /* Tax Section */
  .tax-section { display: flex; flex-direction: column; gap: 24px; }

  /* Tax onboarding */
  .tax-onboarding {
    padding: 32px 24px; background: var(--card-bg);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--card-shadow); text-align: center;
  }
  .tax-onboarding-icon { width: 48px; height: 48px; margin: 0 auto 16px; color: var(--accent); }
  .tax-onboarding h3 { margin: 0 0 8px; color: var(--text-primary); font-size: 1.1rem; }
  .tax-onboarding-desc { margin: 0 0 24px; color: var(--text-secondary); font-size: 0.85rem; max-width: 420px; margin-left: auto; margin-right: auto; }
  .tax-steps { text-align: left; display: flex; flex-direction: column; gap: 12px; max-width: 440px; margin: 0 auto; }
  .tax-step { display: flex; gap: 12px; align-items: flex-start; }
  .tax-step-num {
    width: 24px; height: 24px; border-radius: 50%; background: var(--accent);
    color: #fff; font-size: 0.75rem; font-weight: 700; display: flex;
    align-items: center; justify-content: center; flex-shrink: 0; margin-top: 1px;
  }
  .tax-step strong { font-size: 0.9rem; color: var(--text-primary); display: block; }
  .tax-step p { margin: 4px 0 0; font-size: 0.8rem; color: var(--text-secondary); }

  /* Period selector */
  .period-selector {
    display: flex; flex-direction: column; gap: 12px;
    padding: 16px; background: var(--card-bg);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--card-shadow);
  }
  .period-input-group { display: flex; align-items: flex-end; gap: 12px; flex-wrap: wrap; }
  .period-input-group label { flex: 1; min-width: 140px; display: flex; flex-direction: column; gap: 6px; }
  .period-label { font-size: 0.8rem; color: var(--text-secondary); font-weight: 500; }
  .period-input-group input {
    padding: 8px 12px; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: var(--select-bg);
    color: var(--text-primary); font-size: 0.9rem; transition: border-color 0.2s;
  }
  .period-input-group input:focus { border-color: var(--accent); outline: none; }
  .period-actions { display: flex; gap: 8px; align-items: flex-end; }

  /* Tax config grid */
  .tax-config-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  @media (max-width: 640px) { .tax-config-row { grid-template-columns: 1fr; } }

  .settings-info {
    padding: 16px; background: var(--card-bg); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur); border: 1px solid var(--glass-border);
    border-radius: var(--radius-md); box-shadow: var(--card-shadow);
  }
  .info-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 12px; }
  .info-item { display: flex; flex-direction: column; gap: 4px; }
  .info-item.wide { grid-column: 1 / -1; }
  .info-item .label { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; }
  .info-item .value { font-size: 0.95rem; color: var(--text-primary); font-weight: 500; }

  .tax-card-title { margin: 0 0 12px; color: var(--text-primary); font-size: 0.85rem; font-weight: 600; }
  .tax-card-header { display: flex; align-items: baseline; gap: 6px; margin-bottom: 16px; }
  .tax-card-header h3 { margin: 0; color: var(--text-primary); font-size: 0.9rem; }
  .tax-currency-badge { font-size: 0.72rem; color: var(--text-tertiary); }

  /* IPC section */
  .ipc-section {
    padding: 16px; background: var(--card-bg);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--card-shadow);
  }
  .ipc-desc { margin: 0 0 12px; font-size: 0.8rem; color: var(--text-secondary); }
  .ipc-status-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
  .ipc-status { display: flex; align-items: center; gap: 4px; font-size: 0.8rem; flex: 1; }
  .ipc-status.ok { color: var(--success); }
  .ipc-status.warn { color: var(--warning); }
  .ipc-status-icon { width: 14px; height: 14px; flex-shrink: 0; }
  .hidden-input { display: none; }

  /* Chile info */
  .chile-info {
    padding: 14px 16px; background: var(--card-bg);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid rgba(96, 165, 250, 0.2); border-radius: var(--radius-md);
    box-shadow: var(--card-shadow);
  }
  .chile-info-header { display: flex; align-items: center; gap: 8px; margin-bottom: 10px; font-size: 0.85rem; font-weight: 600; color: #60a5fa; }
  .chile-info-icon { width: 16px; height: 16px; flex-shrink: 0; }
  .chile-info-list { margin: 0; padding-left: 20px; display: flex; flex-direction: column; gap: 4px; }
  .chile-info-list li { font-size: 0.8rem; color: var(--text-secondary); }

  /* Generate report */
  .report-actions { display: flex; gap: 8px; align-items: center; }
  .generate-report-btn { display: flex; align-items: center; gap: 6px; padding: 10px 20px; }

  /* Report skeleton */
  .report-skeleton {
    padding: 16px; background: var(--card-bg); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur); border: 1px solid var(--glass-border);
    border-radius: var(--radius-md); box-shadow: var(--card-shadow);
  }

  /* Summary */
  .report-summary {
    padding: 16px; background: var(--card-bg);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--card-shadow);
  }
  .summary-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 12px; }
  .summary-item { display: flex; flex-direction: column; gap: 4px; }
  .summary-item .label { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; }
  .summary-item .value { font-size: 1rem; font-weight: 600; color: var(--text-primary); }
  .summary-item .value.negative { color: var(--danger); }
  .summary-item.highlight { background: rgba(255, 255, 255, 0.04); border-radius: var(--radius-sm); padding: 8px; }

  /* Warnings */
  .warnings {
    padding: 16px; background: var(--card-bg);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid rgba(248, 113, 113, 0.2); border-radius: var(--radius-md);
    box-shadow: var(--card-shadow);
  }
  .warnings h4 { margin: 0 0 12px; color: var(--danger); font-size: 0.85rem; }
  .warning-item { display: flex; align-items: center; gap: 8px; padding: 6px 0; border-bottom: 1px solid rgba(248, 113, 113, 0.1); font-size: 0.85rem; }
  .warning-code { color: var(--text-secondary); font-weight: 500; min-width: 80px; }
  .warning-msg { color: var(--text-secondary); flex: 1; }
  .warning-action {
    padding: 4px 10px; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: rgba(0, 0, 0, 0.2);
    color: var(--text-primary); cursor: pointer; font-size: 0.75rem;
    transition: all 0.15s;
  }
  .warning-action:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
  .warning-action:disabled { opacity: 0.5; cursor: not-allowed; }

  /* Readiness */
  .readiness {
    padding: 16px; background: var(--card-bg);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--card-shadow);
  }
  .readiness h4 { margin: 0 0 12px; color: var(--text-primary); font-size: 0.85rem; }
  .readiness-item {
    display: flex; align-items: center; gap: 8px; padding: 8px;
    border-radius: var(--radius-sm); margin-bottom: 6px;
  }
  .readiness-item.ok { background: rgba(74, 222, 128, 0.05); }
  .readiness-item.warn { background: rgba(251, 191, 36, 0.05); }
  .readiness-item.error { background: rgba(248, 113, 113, 0.05); }
  .readiness-item.info { background: rgba(96, 165, 250, 0.05); }
  .status-badge {
    font-size: 0.65rem; text-transform: uppercase; font-weight: 600;
    padding: 2px 6px; border-radius: 3px; color: #999;
  }
  .status-badge.ok { background: rgba(74, 222, 128, 0.2); color: var(--success); }
  .status-badge.warn { background: rgba(251, 191, 36, 0.2); color: var(--warning); }
  .status-badge.error { background: rgba(248, 113, 113, 0.2); color: var(--danger); }
  .status-badge.info { background: rgba(96, 165, 250, 0.2); color: #60a5fa; }
  .readiness-item .code { font-size: 0.8rem; color: var(--text-secondary); font-weight: 500; min-width: 90px; }
  .readiness-item .detail { font-size: 0.85rem; color: var(--text-secondary); }

  /* Export */
  .export-actions { display: flex; gap: 8px; }
  .header-actions { display: flex; gap: 8px; }
  .last-updated { font-size: 0.7rem; color: var(--text-tertiary); margin-top: 4px; }
  .export-btn {
    padding: 8px 14px; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: rgba(0, 0, 0, 0.2);
    color: var(--text-secondary); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s; display: flex; align-items: center; gap: 6px;
  }
  .export-btn:hover { border-color: var(--glass-border-hover); background: rgba(0, 0, 0, 0.3); }

  /* Events table */
  .events-table {
    padding: 16px; background: var(--card-bg);
    backdrop-filter: var(--glass-blur); -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    box-shadow: var(--card-shadow);
  }
  .events-table h4 { margin: 0 0 12px; color: var(--text-primary); font-size: 0.85rem; }
  .table-wrapper { overflow-x: auto; }
  .events-table table { width: 100%; border-collapse: collapse; font-size: 0.8rem; }
  .events-table thead { background: rgba(0, 0, 0, 0.1); border-bottom: 1px solid var(--glass-border); }
  .events-table th {
    padding: 8px; text-align: left; color: var(--text-tertiary);
    text-transform: uppercase; font-weight: 500;
  }
  .events-table td { padding: 8px; border-bottom: 1px solid var(--glass-border); color: var(--text-secondary); }
  .events-table tr:hover { background: rgba(0, 0, 0, 0.1); }
  .events-table td.negative { color: var(--danger); }
  .events-table tr.loss { background: rgba(248, 113, 113, 0.04); }
  .events-table tr.loss:hover { background: rgba(248, 113, 113, 0.08); }
  .events-count { font-size: 0.75rem; color: var(--text-tertiary); font-weight: 400; }

  /* Ticker config tabs */
  .cfg-tabs { display: flex; gap: 2px; margin-bottom: 20px; border-bottom: 1px solid var(--glass-border); }
  .cfg-tab {
    padding: 8px 18px; background: none; border: none; border-bottom: 2px solid transparent;
    color: var(--text-tertiary); cursor: pointer; font-size: 0.85rem; font-weight: 500;
    transition: color 0.15s, border-color 0.15s; margin-bottom: -1px;
  }
  .cfg-tab:hover { color: var(--text-primary); }
  .cfg-tab.active { color: var(--accent); border-bottom-color: var(--accent); }

  /* Ticker config modal */
  .tc-section-label { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.05em; margin: 0 0 8px; }
  .tc-empty { font-size: 0.85rem; color: var(--text-tertiary); padding: 8px 0; margin: 0; }

  .tc-active-list { display: flex; flex-direction: column; gap: 4px; margin-bottom: 4px; }
  .tc-active-item {
    display: flex; align-items: center; gap: 8px; padding: 6px 8px;
    background: var(--glass); border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); font-size: 0.85rem;
  }
  .tc-order-btns { display: flex; flex-direction: column; gap: 1px; flex-shrink: 0; }
  .tc-arrow {
    background: none; border: none; cursor: pointer; padding: 1px 3px;
    color: var(--text-tertiary); display: flex; border-radius: 3px;
    transition: color 0.15s, background 0.15s;
  }
  .tc-arrow:hover:not(:disabled) { color: var(--accent); background: var(--accent-bg); }
  .tc-arrow:disabled { opacity: 0.2; cursor: not-allowed; }
  .tc-arrow svg { width: 12px; height: 12px; }
  .tc-icon { width: 16px; height: 16px; border-radius: 50%; flex-shrink: 0; }
  .tc-sym { font-weight: 600; color: var(--text-primary); min-width: 52px; }
  .tc-name { color: var(--text-secondary); flex: 1; font-size: 0.8rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tc-remove {
    background: none; border: none; cursor: pointer; padding: 2px;
    color: var(--text-tertiary); display: flex; flex-shrink: 0;
    border-radius: 3px; transition: color 0.15s;
  }
  .tc-remove:hover { color: var(--danger); }
  .tc-remove svg { width: 14px; height: 14px; }

  .tc-available-list { max-height: 180px; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; margin-bottom: 4px; }
  .tc-available-item {
    display: flex; align-items: center; gap: 8px; padding: 6px 8px; width: 100%;
    background: none; border: none; cursor: pointer; text-align: left;
    border-radius: var(--radius-sm); color: inherit; font-size: 0.85rem;
    transition: background 0.1s;
  }
  .tc-available-item:hover { background: var(--glass-hover); }
  .tc-add-icon { width: 14px; height: 14px; color: var(--accent); flex-shrink: 0; margin-left: auto; opacity: 0; }
  .tc-available-item:hover .tc-add-icon { opacity: 1; }

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
  .catalog-icon { width: 18px; height: 18px; border-radius: 50%; flex-shrink: 0; }
  .catalog-sym { font-weight: 600; color: var(--text-primary); min-width: 56px; }
  .catalog-name { color: var(--text-secondary); flex: 1; }
  .custom-coin-form { margin-top: 16px; border-top: 1px solid var(--glass-border); padding-top: 12px; display: flex; flex-direction: column; gap: 6px; }
  .form-label { font-size: 0.8rem; color: var(--text-secondary); display: block; margin-bottom: 2px; }
  .custom-coin-row { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
  .custom-coin-row-bottom { display: grid; grid-template-columns: 1fr auto; gap: 6px; }
  .custom-coin-row input, .custom-coin-row-bottom input {
    width: 100%; padding: 8px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.8rem; box-sizing: border-box;
  }
  .custom-coin-row input:focus, .custom-coin-row-bottom input:focus { border-color: var(--accent); outline: none; }

  /* Transaction list (Recent) */
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
    width: 8px; height: 8px; border-radius: 50%;
    background: var(--text-tertiary); flex-shrink: 0;
  }
  .tx-type-dot.buy { background: var(--success); }
  .tx-type-dot.sell { background: var(--danger); }
  .tx-type-dot.transfer { background: #60a5fa; }
  .tx-crypto-icon { width: 16px; height: 16px; border-radius: 50%; flex-shrink: 0; }

  .tx-main {
    flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 3px;
  }
  .tx-desc {
    font-size: 0.875rem; font-weight: 500; color: var(--text-primary);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .tx-meta {
    display: flex; align-items: center; gap: 8px; flex-wrap: wrap;
  }
  .tx-acc  { font-size: 0.71rem; color: var(--text-tertiary); }
  .tx-date { font-size: 0.71rem; color: var(--text-tertiary); }
  .tx-amount {
    font-size: 0.9rem; font-weight: 600; color: var(--text-secondary);
    white-space: nowrap; text-align: right; min-width: 80px;
  }
  .tx-amount.buy { color: var(--success); }
  .tx-amount.sell { color: var(--danger); }
  .tx-amount.transfer { color: #60a5fa; }

  /* Transactions tab */
  .tab-content { padding-top: 20px; }
  .activity-toolbar {
    display: flex; gap: 12px; align-items: flex-start; margin-bottom: 16px; flex-wrap: wrap;
  }
  .filter-search {
    position: relative; flex: 1; min-width: 160px;
  }
  .filter-search-icon {
    position: absolute; left: 10px; top: 50%; transform: translateY(-50%);
    width: 14px; height: 14px; color: var(--text-tertiary); pointer-events: none;
  }
  .filter-search input {
    width: 100%; padding: 8px 12px 8px 32px;
    border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--glass); backdrop-filter: var(--glass-blur);
    color: var(--text-primary); font-size: 0.85rem; box-sizing: border-box;
    transition: border-color 0.2s;
  }
  .filter-search input:focus { border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow); }
  .load-more-btn {
    display: block; margin: 16px auto; padding: 8px 24px;
    border: 1px solid var(--glass-border); border-radius: var(--radius-sm); background: none;
    color: var(--text-secondary); cursor: pointer; transition: all 0.15s;
  }
  .load-more-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
</style>
