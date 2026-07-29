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

<script lang="ts">
  import { errorMessage } from '../lib/errors'
  import { app, type BackgroundFx } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as settingsApi from '../lib/api/settings'
  import * as vaultApi from '../lib/api/vault'
  import * as ingestionApi from '../lib/api/ingestion'
  import * as cryptoApi from '../lib/api/crypto'
  import * as financeApi from '../lib/api/finance'
  import { save } from '@tauri-apps/plugin-dialog'
  import type {
    AppInfo, ImportResultsResponse,
    ExchangeDetectionResult, CsvAnalysisResult, CustomCsvMapping
  } from '../lib/types'

  import ConfirmDialog from '../components/ConfirmDialog.svelte'
  import CustomCsvMapper from '../components/CustomCsvMapper.svelte'

  let info = $state<AppInfo | null>(null)
  let maxFileSize = $state(0)
  let settingsLoading = $state(true)
  let showResetConfirm = $state(false)

  // Import state
  type ImportStep = 'idle' | 'preview' | 'results'
  let importStep = $state<ImportStep>('idle')
  let importMode = $state<'generic' | 'exchange' | 'custom'>('generic')
  let importFilename = $state('')
  let importContent = $state('')
  let importPreview = $state<ImportResultsResponse | null>(null)
  let importResults = $state<ImportResultsResponse | null>(null)
  let importLoading = $state(false)

  // Exchange detection
  let exchangeDetection = $state<ExchangeDetectionResult | null>(null)
  let exchangeWalletName = $state('')
  // Inline "create target wallet" step when the entered wallet doesn't exist yet.
  let walletNamesLower = $state<string[]>([])
  let showWalletCreate = $state(false)
  let newWalletCategory = $state('exchange')
  let creatingWallet = $state(false)

  // Custom CSV column mapping (unknown-exchange import)
  let customAnalysis = $state<CsvAnalysisResult | null>(null)
  let customWallets = $state<string[]>([])

  async function load() {
    try {
      const [appInfo, maxSize] = await Promise.all([
        settingsApi.getAppInfo(),
        ingestionApi.maxImportFileSize(),
      ])
      info = appInfo
      maxFileSize = maxSize
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      settingsLoading = false
    }
  }

  function resetImport() {
    importStep = 'idle'
    importMode = 'generic'
    importFilename = ''
    importContent = ''
    importPreview = null
    importResults = null
    exchangeDetection = null
    exchangeWalletName = ''
    showWalletCreate = false
    customAnalysis = null
    customWallets = []
  }

  let genericFileInput = $state<HTMLInputElement>(null!)
  let exchangeFileInput = $state<HTMLInputElement>(null!)
  let customFileInput = $state<HTMLInputElement>(null!)

  function readFile(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => resolve(reader.result as string)
      reader.onerror = () => reject(reader.error)
      reader.readAsText(file)
    })
  }

  async function handleGenericFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    try {
      const content = await readFile(file)
      if (maxFileSize > 0 && content.length > maxFileSize) {
        app.showToast(`File too large (max ${Math.round(maxFileSize / 1024)}KB)`, true)
        return
      }
      importContent = content
      importFilename = file.name
      importMode = 'generic'
      importLoading = true
      importPreview = await ingestionApi.previewImport(content, file.name)
      importStep = 'preview'
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      importLoading = false
      genericFileInput.value = ''
    }
  }

  async function handleExchangeFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    try {
      const content = await readFile(file)
      if (maxFileSize > 0 && content.length > maxFileSize) {
        app.showToast(`File too large (max ${Math.round(maxFileSize / 1024)}KB)`, true)
        return
      }
      importContent = content
      importMode = 'exchange'
      importLoading = true
      const detection = await ingestionApi.detectExchangeSource(content)
      if (!detection) {
        app.showToast(i18n.t('settings-import-no-detection', 'Could not detect exchange format'), true)
        importLoading = false
        return
      }
      exchangeDetection = detection
      exchangeWalletName = detection.suggested_wallet
      showWalletCreate = false
      // Load existing wallet names so we can offer to create a missing target.
      try {
        const w = await cryptoApi.fetchWallets()
        walletNamesLower = w.wallets.map((x) => x.name.trim().toLowerCase())
      } catch {
        walletNamesLower = []
      }
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      importLoading = false
      exchangeFileInput.value = ''
    }
  }

  async function handleCustomFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    try {
      const content = await readFile(file)
      if (maxFileSize > 0 && content.length > maxFileSize) {
        app.showToast(`File too large (max ${Math.round(maxFileSize / 1024)}KB)`, true)
        return
      }
      importContent = content
      importMode = 'custom'
      importLoading = true
      const analysis = await ingestionApi.analyzeCustomCsv(content)
      // Load existing wallet names so the user can pick the import target.
      try {
        const w = await cryptoApi.fetchWallets()
        customWallets = w.wallets.map((x) => x.name)
      } catch {
        customWallets = []
      }
      customAnalysis = analysis
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      importLoading = false
      customFileInput.value = ''
    }
  }

  async function importCustomMapped(mapping: CustomCsvMapping, walletName: string) {
    try {
      importLoading = true
      importResults = await ingestionApi.importCustomCsv(importContent, mapping, walletName.trim())
      importStep = 'results'
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      importLoading = false
    }
  }

  async function previewExchange() {
    if (!exchangeWalletName.trim()) {
      app.showToast(i18n.t('settings-import-wallet-required', 'Wallet name is required'), true)
      return
    }
    try {
      importLoading = true
      importPreview = await ingestionApi.previewExchangeCsv(importContent, exchangeWalletName.trim())
      importStep = 'preview'
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      importLoading = false
    }
  }

  // Preview, but if the target wallet doesn't exist yet, surface the inline
  // create step first instead of failing with "wallet not found" on every row.
  async function previewOrCreate() {
    if (!exchangeWalletName.trim()) {
      app.showToast(i18n.t('settings-import-wallet-required', 'Wallet name is required'), true)
      return
    }
    if (walletNamesLower.includes(exchangeWalletName.trim().toLowerCase())) {
      showWalletCreate = false
      await previewExchange()
    } else {
      showWalletCreate = true
    }
  }

  async function createWalletAndPreview() {
    if (!exchangeWalletName.trim()) return
    creatingWallet = true
    try {
      await cryptoApi.addWallet(exchangeWalletName.trim(), newWalletCategory)
      walletNamesLower = [...walletNamesLower, exchangeWalletName.trim().toLowerCase()]
      showWalletCreate = false
      await previewExchange()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      creatingWallet = false
    }
  }

  async function confirmImport() {
    try {
      importLoading = true
      if (importMode === 'generic') {
        importResults = await ingestionApi.importData(importContent, importFilename)
      } else {
        importResults = await ingestionApi.importExchangeCsv(importContent, exchangeWalletName.trim())
      }
      importStep = 'results'
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      importLoading = false
    }
  }

  async function toggleDarkMode() {
    if (!app.settings) return
    const next = !app.settings.dark_mode
    app.settings.dark_mode = next
    await settingsApi.setDarkMode(next).catch(e => app.showToast(errorMessage(e), true))
  }

  function changeBackground(e: Event) {
    app.setBackgroundFx((e.target as HTMLSelectElement).value as BackgroundFx)
  }

  async function changeCurrency(e: Event) {
    const val = (e.target as HTMLSelectElement).value
    if (!app.settings) return
    app.settings.preferred_currency = val
    await settingsApi.setPreferredCurrency(val)
  }

  async function changeLanguage(e: Event) {
    const val = (e.target as HTMLSelectElement).value
    if (!app.settings) return
    try {
      app.settings.preferred_language = val
      await settingsApi.setPreferredLanguage(val)
      await i18n.load()
      app.showToast(`Language → ${val} (${Object.keys(i18n.strings).length} keys)`)
    } catch (err) {
      app.showToast(`Language change failed: ${err}`, true)
    }
  }

  async function changeTimeout(e: Event) {
    const val = parseInt((e.target as HTMLSelectElement).value)
    if (!app.settings) return
    app.settings.session_timeout_secs = val
    await settingsApi.setSessionTimeout(val)
  }

  async function toggleAutoFetch() {
    if (!app.settings) return
    const next = !app.settings.auto_fetch
    app.settings.auto_fetch = next
    await settingsApi.setAutoFetch(next)
  }

  async function toggleProxy() {
    if (!app.settings) return
    const next = !app.settings.proxy_enabled
    app.settings.proxy_enabled = next
    await settingsApi.setProxyEnabled(next, app.settings.proxy_url)
  }

  async function updateProxyUrl() {
    if (!app.settings) return
    try {
      await settingsApi.setProxyUrl(app.settings.proxy_url)
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function exportVault() {
    try {
      const path = await save({
        title: 'Export Vault Backup',
        filters: [{ name: 'Sanctum Backup', extensions: ['db'] }],
      })
      if (!path) return
      await vaultApi.exportVault(path)
      app.settings = await settingsApi.loadSettings()
      app.showToast(i18n.t('settings-export-success', 'Backup saved successfully'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function exportTransactionsCsv() {
    try {
      const path = await save({
        title: i18n.t('settings-export-transactions', 'Export Transactions'),
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      })
      if (!path) return
      const rows = await financeApi.exportTransactionsCsv(path)
      app.showToast(
        i18n.tArgs('settings-export-csv-done', { count: rows }, '{$count} transactions exported')
      )
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  /** Days since the last vault export, or null when never exported. */
  let daysSinceBackup = $derived.by(() => {
    const stamp = app.settings?.last_backup_at
    if (!stamp) return null
    const then = new Date(stamp).getTime()
    if (Number.isNaN(then)) return null
    return Math.max(0, Math.floor((Date.now() - then) / 86_400_000))
  })

  async function resetAllSettings() {
    try {
      await settingsApi.resetSettings()
      app.settings = await settingsApi.loadSettings()
      app.showToast(i18n.t('settings-reset-success', 'Settings reset to defaults'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  $effect(() => { load() })
</script>

<div class="page">
  <h2>{i18n.t('settings-title', 'Settings')}</h2>

  {#if app.settings}
    <!-- Appearance -->
    <section class="section">
      <h3>{i18n.t('settings-appearance', 'Appearance')}</h3>
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-dark-mode', 'Dark Mode')}</span>
          <span class="setting-desc">{i18n.t('settings-dark-mode-desc', 'Toggle dark/light theme')}</span>
        </div>
        <button class="toggle-switch" class:on={app.settings.dark_mode} onclick={toggleDarkMode} aria-label="Toggle dark mode">
          <span class="toggle-knob"></span>
        </button>
      </div>
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-background', 'Background')}</span>
          <span class="setting-desc">{i18n.t('settings-background-desc', 'Choose the backdrop design')}</span>
        </div>
        <select value={app.backgroundFx} onchange={changeBackground}>
          <option value="dots">{i18n.t('settings-bg-dots', 'Dots')}</option>
          <option value="stars">{i18n.t('settings-bg-stars', 'Starfield')}</option>
          <option value="aurora">{i18n.t('settings-bg-aurora', 'Aurora')}</option>
          <option value="diamonds">{i18n.t('settings-bg-diamonds', 'Diamonds')}</option>
          <option value="dragon">{i18n.t('settings-bg-dragon', 'Dragon')}</option>
        </select>
      </div>
    </section>

    <!-- Regional -->
    <section class="section">
      <h3>{i18n.t('settings-regional', 'Regional')}</h3>
      <div class="setting-row">
        <span class="setting-label">{i18n.t('settings-preferred-currency', 'Preferred Currency')}</span>
        <select value={app.settings.preferred_currency} onchange={changeCurrency}>
          {#each ['USD', 'CLP', 'EUR', 'GBP', 'BRL', 'MXN', 'ARS', 'CAD', 'AUD', 'CHF', 'JPY'] as cur}
            <option value={cur}>{cur}</option>
          {/each}
        </select>
      </div>
      <div class="setting-row">
        <span class="setting-label">{i18n.t('settings-language', 'Language')}</span>
        <select value={app.settings.preferred_language} onchange={changeLanguage}>
          <option value="en">English</option>
          <option value="es">Español</option>
        </select>
      </div>
    </section>

    <!-- Security -->
    <section class="section">
      <h3>{i18n.t('settings-security', 'Security')}</h3>
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-session-timeout', 'Session Timeout')}</span>
          <span class="setting-desc">{i18n.t('settings-session-timeout-desc', 'Auto-lock after inactivity')}</span>
        </div>
        <select value={app.settings.session_timeout_secs} onchange={changeTimeout}>
          <option value={300}>{i18n.t('settings-timeout-5min', '5 minutes')}</option>
          <option value={900}>{i18n.t('settings-timeout-15min', '15 minutes')}</option>
          <option value={1800}>{i18n.t('settings-timeout-30min', '30 minutes')}</option>
          <option value={3600}>{i18n.t('settings-timeout-1hour', '1 hour')}</option>
        </select>
      </div>
    </section>

    <!-- Vault Backup -->
    <section class="section">
      <h3>{i18n.t('settings-vault-backup', 'Vault Backup')}</h3>
      <p class="section-note">{i18n.t('settings-vault-note', 'Your vault is encrypted with SQLCipher (AES-256).')}</p>
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-export-vault', 'Export Vault')}</span>
          <span class="setting-desc">
            {i18n.t('settings-last-backup', 'Last backup')}:
            {#if daysSinceBackup === null}
              {i18n.t('settings-last-backup-never', 'never')}
            {:else if daysSinceBackup === 0}
              {i18n.t('time-today', 'Today')}
            {:else}
              {i18n.tArgs('settings-last-backup-days', { count: daysSinceBackup }, '{$count} days ago')}
            {/if}
          </span>
        </div>
        <button class="glass-btn" onclick={exportVault}>{i18n.t('settings-export-btn', 'Export')}</button>
      </div>
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-export-transactions', 'Export Transactions')}</span>
          <span class="setting-desc">{i18n.t('settings-export-transactions-desc', 'Plain CSV of your whole ledger, unencrypted')}</span>
        </div>
        <button class="glass-btn" onclick={exportTransactionsCsv}>{i18n.t('settings-export-btn', 'Export')}</button>
      </div>
    </section>

    <!-- Data Import -->
    <section class="section">
      <h3>{i18n.t('settings-data-import', 'Data Import')}</h3>

      {#if importStep === 'idle'}
        {#if exchangeDetection}
          <!-- Exchange detected, ask for wallet name -->
          <div class="import-card">
            <p class="import-info">
              {i18n.t('settings-import-detected', 'Detected:')} <strong>{exchangeDetection.exchange}</strong>
              ({exchangeDetection.total_records} {i18n.t('settings-import-records', 'records')})
            </p>
            <div class="setting-row">
              <span class="setting-label">{i18n.t('settings-import-target-wallet', 'Target Wallet')}</span>
              <input
                type="text"
                bind:value={exchangeWalletName}
                oninput={() => (showWalletCreate = false)}
                placeholder={i18n.t('settings-import-wallet-placeholder', 'Wallet name')}
              />
            </div>
            {#if showWalletCreate}
              <div class="wallet-create">
                <p class="wallet-create-note">{i18n.tArgs('settings-import-wallet-missing', { name: exchangeWalletName.trim() }, `Wallet "${exchangeWalletName.trim()}" doesn't exist yet. Create it?`)}</p>
                <div class="setting-row">
                  <span class="setting-label">{i18n.t('crypto-wallet-category', 'Category')}</span>
                  <select bind:value={newWalletCategory}>
                    <option value="exchange">Exchange</option>
                    <option value="hardware">Hardware</option>
                    <option value="software">Software</option>
                  </select>
                </div>
              </div>
            {/if}
            <div class="import-actions">
              <button class="secondary-btn" onclick={resetImport}>{i18n.t('settings-cancel', 'Cancel')}</button>
              {#if showWalletCreate}
                <button class="primary-btn" onclick={createWalletAndPreview} disabled={creatingWallet || !exchangeWalletName.trim()}>
                  {creatingWallet ? i18n.t('settings-import-loading', 'Loading...') : i18n.t('settings-import-create-wallet-preview', 'Create wallet & preview')}
                </button>
              {:else}
                <button class="primary-btn" onclick={previewOrCreate} disabled={importLoading || !exchangeWalletName.trim()}>
                  {importLoading ? i18n.t('settings-import-loading', 'Loading...') : i18n.t('settings-import-preview-btn', 'Preview')}
                </button>
              {/if}
            </div>
          </div>
        {:else if customAnalysis}
          <!-- Custom CSV: user maps each column to a Sanctum field -->
          <div class="import-card">
            <CustomCsvMapper
              headers={customAnalysis.headers}
              sampleRow={customAnalysis.sample_row}
              wallets={customWallets}
              loading={importLoading}
              onimport={importCustomMapped}
              oncancel={resetImport}
            />
          </div>
        {:else}
          <input type="file" accept=".csv" class="hidden-input" bind:this={genericFileInput} onchange={handleGenericFile} />
          <input type="file" accept=".csv" class="hidden-input" bind:this={exchangeFileInput} onchange={handleExchangeFile} />
          <input type="file" accept=".csv" class="hidden-input" bind:this={customFileInput} onchange={handleCustomFile} />
          <div class="setting-row">
            <div>
              <span class="setting-label">{i18n.t('settings-import-generic', 'Generic CSV')}</span>
              <span class="setting-desc">{i18n.t('settings-import-generic-desc', 'Import transactions from a CSV file')}</span>
            </div>
            <button class="secondary-btn" onclick={() => genericFileInput.click()} disabled={importLoading}>
              {importLoading ? i18n.t('settings-import-loading', 'Loading...') : i18n.t('settings-import-select-file', 'Select File')}
            </button>
          </div>
          <div class="setting-row">
            <div>
              <span class="setting-label">{i18n.t('settings-import-exchange', 'Exchange / Wallet CSV')}</span>
              <span class="setting-desc">{i18n.t('settings-import-exchange-desc', 'Import from Kraken, Binance, MEXC, Feather, Monero…')}</span>
            </div>
            <button class="secondary-btn" onclick={() => exchangeFileInput.click()} disabled={importLoading}>
              {importLoading ? i18n.t('settings-import-loading', 'Loading...') : i18n.t('settings-import-select-file', 'Select File')}
            </button>
          </div>
          <div class="setting-row">
            <div>
              <span class="setting-label">{i18n.t('settings-import-custom', 'Custom CSV (manual mapping)')}</span>
              <span class="setting-desc">{i18n.t('settings-import-custom-desc', 'Import from any other exchange by mapping its columns')}</span>
            </div>
            <button class="secondary-btn" onclick={() => customFileInput.click()} disabled={importLoading}>
              {importLoading ? i18n.t('settings-import-loading', 'Loading...') : i18n.t('settings-import-select-file', 'Select File')}
            </button>
          </div>
          <details class="exchange-help">
            <summary>{i18n.t('settings-import-exchange-help', 'How to export from each exchange or wallet')}</summary>
            <ul class="exchange-help-list">
              <li>{i18n.t('import-exchange-hint-kraken', 'Kraken: Documents > export Ledgers and Trades CSV files (you can upload both together).')}</li>
              <li>{i18n.t('import-exchange-hint-kraken-pro', 'Kraken Pro: History > Statements > export Ledgers and Trades CSV files (you can upload both together).')}</li>
              <li>{i18n.t('import-exchange-hint-binance', 'Export from Binance: Orders > Transaction History > Generate All Statements')}</li>
              <li>{i18n.t('import-exchange-hint-mexc', 'MEXC: Help Center > Account Data Export > select required reports > convert to CSV. Supports 17 report CSV types; you can upload multiple files at once.')}</li>
              <li>{i18n.t('import-exchange-hint-notbank', 'NotBank (CryptoMarket): Exchange Pro > Reports > Single Report > Transaction and Trade Activity (you can upload both together).')}</li>
              <li>{i18n.t('import-exchange-hint-feather', 'Export from Feather Wallet: History > Export CSV')}</li>
              <li>{i18n.t('import-exchange-hint-monero-gui', 'Export from Monero GUI: Wallet > History > Export CSV')}</li>
            </ul>
          </details>
        {/if}

      {:else if importStep === 'preview' && importPreview}
        <!-- Preview results (dry run) -->
        <div class="import-card">
          <p class="import-info">
            {importPreview.total_processed} {i18n.t('settings-import-records', 'records')} |
            {importPreview.inserted} {i18n.t('settings-import-to-add', 'to add')} |
            {importPreview.skipped} {i18n.t('settings-import-to-skip', 'to skip')}
          </p>
          {#if importPreview.errors.length > 0}
            <div class="import-errors">
              <p class="error-heading">{i18n.t('settings-import-errors', 'Errors')} ({importPreview.errors.length}):</p>
              {#each importPreview.errors as err}
                <p class="error-line">
                  {#if err.line}{i18n.t('settings-import-line', 'Line')} {err.line}: {/if}{err.message}
                </p>
              {/each}
            </div>
          {/if}
          <div class="import-actions">
            <button class="secondary-btn" onclick={resetImport}>{i18n.t('settings-cancel', 'Cancel')}</button>
            <button class="primary-btn" onclick={confirmImport} disabled={importLoading}>
              {importLoading ? i18n.t('settings-import-importing', 'Importing...') : i18n.t('settings-import-confirm', 'Confirm Import')}
            </button>
          </div>
        </div>

      {:else if importStep === 'results' && importResults}
        <!-- Import results -->
        <div class="import-card">
          <p class="import-info">
            {i18n.t('settings-import-processed', 'Processed:')} {importResults.total_processed} |
            {i18n.t('settings-import-inserted', 'Inserted:')} {importResults.inserted} |
            {i18n.t('settings-import-skipped', 'Skipped:')} {importResults.skipped}
          </p>
          {#if importResults.errors.length > 0}
            <div class="import-errors">
              <p class="error-heading">{i18n.t('settings-import-errors', 'Errors')} ({importResults.errors.length}):</p>
              {#each importResults.errors as err}
                <p class="error-line">
                  {#if err.line}{i18n.t('settings-import-line', 'Line')} {err.line}: {/if}{err.message}
                </p>
              {/each}
            </div>
          {/if}
          <div class="import-actions">
            <button class="secondary-btn" onclick={resetImport}>{i18n.t('settings-import-done', 'Done')}</button>
          </div>
        </div>
      {/if}
    </section>

    <!-- Data Sync -->
    <section class="section">
      <h3>{i18n.t('settings-data-sync', 'Data Sync')}</h3>
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-auto-fetch', 'Auto-fetch Prices')}</span>
          <span class="setting-desc">{i18n.t('settings-auto-fetch-desc', 'Automatically fetch crypto prices on sync')}</span>
        </div>
        <button class="toggle-switch" class:on={app.settings.auto_fetch} onclick={toggleAutoFetch} aria-label="Toggle auto-fetch">
          <span class="toggle-knob"></span>
        </button>
      </div>
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-use-proxy', 'Use Proxy')}</span>
          <span class="setting-desc">{i18n.t('settings-use-proxy-desc', 'Route API calls through a proxy')}</span>
        </div>
        <button class="toggle-switch" class:on={app.settings.proxy_enabled} onclick={toggleProxy} aria-label="Toggle proxy">
          <span class="toggle-knob"></span>
        </button>
      </div>
      {#if app.settings.proxy_enabled}
        <div class="setting-row">
          <span class="setting-label">{i18n.t('settings-proxy-url', 'Proxy URL')}</span>
          <input
            type="text"
            bind:value={app.settings.proxy_url}
            onblur={updateProxyUrl}
            placeholder={i18n.t('settings-proxy-placeholder', 'socks5://127.0.0.1:9050')}
          />
        </div>
      {/if}
    </section>

    <!-- About -->
    {#if info}
      <section class="section">
        <h3>{i18n.t('settings-about', 'About')}</h3>
        <div class="about-grid">
          <span class="about-label">{i18n.t('settings-about-version', 'Version')}</span><span>{info.version}</span>
          <span class="about-label">{i18n.t('settings-about-encryption', 'Encryption')}</span><span>{info.encryption}</span>
          <span class="about-label">{i18n.t('settings-about-storage', 'Storage')}</span><span>{info.storage}</span>
        </div>
      </section>
    {:else if settingsLoading}
      <section class="section">
        <h3>{i18n.t('settings-about', 'About')}</h3>
        <div class="skeleton" style="width:140px;height:14px;margin-bottom:8px"></div>
        <div class="skeleton" style="width:100px;height:14px;margin-bottom:8px"></div>
        <div class="skeleton" style="width:160px;height:14px"></div>
      </section>
    {/if}

    <!-- Danger Zone -->
    <section class="section danger-section">
      <h3>{i18n.t('settings-reset-section', 'Reset')}</h3>
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-reset-all', 'Reset All Settings')}</span>
          <span class="setting-desc">{i18n.t('settings-reset-all-desc', 'Restore default values for all settings')}</span>
        </div>
        <button class="danger-btn" onclick={() => showResetConfirm = true}>{i18n.t('settings-reset-btn', 'Reset')}</button>
      </div>
    </section>
  {/if}
</div>

<ConfirmDialog
  show={showResetConfirm}
  message={i18n.t('confirm-reset-settings', 'Reset all settings to their default values? This will not affect your data.')}
  danger
  onconfirm={async () => {
    await resetAllSettings()
    showResetConfirm = false
  }}
  onclose={() => showResetConfirm = false}
/>

<style>
  .page { padding: 24px 32px; max-width: 640px; width: 100%; margin: 0 auto; }
  h2 { font-size: 1.3rem; letter-spacing: 0.15em; color: var(--text-primary); margin-bottom: 28px; }

  .section {
    margin-bottom: 28px; padding-bottom: 24px; border-bottom: 1px solid var(--glass-border);
  }
  .section h3 {
    font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase;
    letter-spacing: 0.08em; margin-bottom: 14px;
  }
  .section-note { font-size: 0.8rem; color: var(--text-tertiary); margin-bottom: 12px; }

  .setting-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: 10px 0; gap: 16px;
  }
  .setting-label { font-size: 0.9rem; color: var(--text-secondary); display: block; }
  .setting-desc { font-size: 0.75rem; color: var(--text-tertiary); display: block; margin-top: 2px; }
  .exchange-help { margin-top: 4px; font-size: 0.8rem; }
  .exchange-help summary { cursor: pointer; color: var(--text-secondary); user-select: none; padding: 4px 0; }
  .exchange-help summary:hover { color: var(--text-primary); }
  .exchange-help-list { margin: 6px 0 0; padding-left: 18px; display: flex; flex-direction: column; gap: 6px; color: var(--text-secondary); line-height: 1.45; }

  select, input[type="text"] {
    padding: 8px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--glass); backdrop-filter: var(--glass-blur);
    color: var(--text-primary); font-size: 0.85rem;
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  input[type="text"]:focus, select:focus {
    border-color: var(--accent); outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .toggle-switch {
    position: relative; width: 44px; height: 24px; border-radius: 12px;
    border: 1px solid var(--glass-border); background: var(--glass); cursor: pointer; padding: 0;
    transition: all 0.25s;
  }
  .toggle-switch.on {
    background: var(--accent-bg); border-color: var(--accent-border);
    box-shadow: 0 0 12px var(--accent-glow);
  }
  .toggle-knob {
    position: absolute; top: 2px; left: 2px; width: 18px; height: 18px;
    border-radius: 50%; background: var(--text-primary); transition: transform 0.25s;
  }
  .toggle-switch.on .toggle-knob { transform: translateX(20px); }

  .secondary-btn {
    padding: 8px 18px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .secondary-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }

  .danger-btn {
    padding: 8px 18px; border: 1px solid var(--danger-border); border-radius: var(--radius-sm);
    background: var(--danger-bg); color: var(--danger); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .danger-btn:hover { background: var(--danger-border); }

  .danger-section { border-bottom: none; }

  .about-grid {
    display: grid; grid-template-columns: 100px 1fr; gap: 8px;
    font-size: 0.85rem; color: var(--text-secondary);
  }

  /* ── Mobile ───────────────────────────────────────────────── */
  @media (max-width: 640px) {
    .page { padding: 20px 16px; }
  }
  .about-label { color: var(--text-tertiary); }

  .hidden-input { display: none; }
  .import-card {
    background: var(--card-bg); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 16px; margin-top: 8px; box-shadow: var(--card-shadow);
  }
  .import-info { font-size: 0.85rem; color: var(--text-secondary); margin: 0 0 12px; }
  .import-info strong { color: var(--text-primary); }
  .import-actions {
    display: flex; gap: 10px; justify-content: flex-end; margin-top: 14px;
  }
  .primary-btn {
    padding: 8px 18px; border: 1px solid var(--accent-border); border-radius: var(--radius-sm);
    background: var(--accent-bg); backdrop-filter: blur(8px);
    color: var(--text-on-accent); cursor: pointer; font-size: 0.85rem;
    transition: all 0.2s;
  }
  .primary-btn:hover:not(:disabled) { background: var(--accent-border); box-shadow: 0 0 16px var(--accent-glow); }  .primary-btn:disabled, .secondary-btn:disabled {
    opacity: 0.4; cursor: not-allowed;
  }
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
  .import-errors { margin-top: 8px; }
  .error-heading { color: var(--danger); font-size: 0.85rem; margin: 0 0 6px; font-weight: 600; }
  .error-line { color: var(--text-secondary); font-size: 0.8rem; margin: 2px 0; padding-left: 8px; }
  .wallet-create { margin: 8px 0; padding: 10px 12px; border-radius: 8px; background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.3); }
  .wallet-create-note { margin: 0 0 8px; font-size: 0.82rem; color: var(--text-secondary); }
  .wallet-create select { padding: 6px 8px; border-radius: 6px; background: var(--input-bg, rgba(255, 255, 255, 0.04)); border: 1px solid var(--glass-border); color: var(--text-primary); }
</style>
