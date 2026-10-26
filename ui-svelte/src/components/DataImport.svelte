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

<!-- Every file import: the generic CSV/JSON, exchange and wallet CSVs with
     format detection, and the manual column mapper. Shown in Settings, and
     on the Crypto page with only the crypto options. -->

<script lang="ts">
  import { onMount } from 'svelte'
  import { errorMessage } from '../lib/errors'
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as ingestionApi from '../lib/api/ingestion'
  import * as cryptoApi from '../lib/api/crypto'
  import * as financeApi from '../lib/api/finance'
  import type {
    ImportResultsResponse,
    ExchangeDetectionResult, CsvAnalysisResult, CustomCsvMapping
  } from '../lib/types'
  import CustomCsvMapper from './CustomCsvMapper.svelte'

  interface Props {
    /** Only the exchange/wallet and custom CSV options. */
    cryptoOnly?: boolean
    /** Called after an import wrote something. */
    onimported?: () => void
  }

  let { cryptoOnly = false, onimported }: Props = $props()

  let maxFileSize = $state(0)

  onMount(() => {
    ingestionApi.maxImportFileSize().then(size => { maxFileSize = size }).catch(() => {})
  })

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

  // Account types picked for accounts the file refers to but the vault lacks.
  let missingAccountTypes = $state<Record<string, string>>({})
  // Categories picked for crypto wallets the file refers to but the vault lacks.
  let missingWalletCategories = $state<Record<string, string>>({})
  let hasMissing = $derived(
    !!importPreview &&
      (importPreview.missing_accounts.length > 0 ||
        importPreview.missing_categories.length > 0 ||
        importPreview.missing_wallets.length > 0)
  )

  /** Creates what the preview found missing, then previews the same file again. */
  async function createMissingAndPreview() {
    if (!importPreview) return
    importLoading = true
    try {
      for (const acc of importPreview.missing_accounts) {
        await financeApi.createAccount(acc.name, missingAccountTypes[acc.name] ?? 'bank', acc.currency, '0')
      }
      for (const cat of importPreview.missing_categories) {
        await financeApi.addCategory(cat.name, cat.category_type)
      }
      for (const name of importPreview.missing_wallets) {
        await cryptoApi.addWallet(name, missingWalletCategories[name] ?? 'exchange')
      }
      missingAccountTypes = {}
      missingWalletCategories = {}
      importPreview = await ingestionApi.previewImport(importContent, importFilename)
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      importLoading = false
    }
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
      onimported?.()
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
      onimported?.()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      importLoading = false
    }
  }

</script>

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
      <input type="file" accept=".csv,.json" class="hidden-input" bind:this={genericFileInput} onchange={handleGenericFile} />
      <input type="file" accept=".csv" class="hidden-input" bind:this={exchangeFileInput} onchange={handleExchangeFile} />
      <input type="file" accept=".csv" class="hidden-input" bind:this={customFileInput} onchange={handleCustomFile} />
      {#if !cryptoOnly}
      <div class="setting-row">
        <div>
          <span class="setting-label">{i18n.t('settings-import-generic', 'Generic CSV')}</span>
          <span class="setting-desc">{i18n.t('settings-import-generic-desc', 'Import transactions from a CSV file')}</span>
        </div>
        <button class="secondary-btn" onclick={() => genericFileInput.click()} disabled={importLoading}>
          {importLoading ? i18n.t('settings-import-loading', 'Loading...') : i18n.t('settings-import-select-file', 'Select File')}
        </button>
      </div>
      {/if}
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
      <details class="exchange-help" open={cryptoOnly}>
        <summary>{i18n.t('settings-import-exchange-help', 'How to export from each exchange or wallet')}</summary>
        <ul class="exchange-help-list">
          <li>{i18n.t('import-exchange-hint-kraken', 'Kraken: Documents > export Ledgers and Trades CSV files (you can upload both together).')}</li>
          <li>{i18n.t('import-exchange-hint-kraken-pro', 'Kraken Pro: History > Statements > export Ledgers and Trades CSV files (you can upload both together).')}</li>
          <li>{i18n.t('import-exchange-hint-binance', 'Export from Binance: Orders > Transaction History > Generate All Statements')}</li>
          <li>{i18n.t('import-exchange-hint-mexc', 'MEXC: Help Center > Account Data Export > select required reports > convert to CSV. Supports 17 report CSV types; you can upload multiple files at once.')}</li>
          <li>{i18n.t('import-exchange-hint-notbank', 'NotBank (CryptoMarket): Exchange Pro > Reports > Single Report > Transaction and Trade Activity (you can upload both together).')}</li>
          <li>{i18n.t('import-exchange-hint-feather', 'Export from Feather Wallet: History > Export CSV')}</li>
          <li>{i18n.t('import-exchange-hint-monero-gui', 'Export from Monero GUI: Wallet > History > Export CSV')}</li>
          <li>{i18n.t('import-exchange-hint-monero-view-only', 'Monero without the spend key: in Feather, Wallet > View-Only creates a wallet from your address and private view key. Import its key images (needs your own node or a trusted one) so outgoing payments count, then export the History CSV.')}</li>
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
      {#if hasMissing}
        <div class="wallet-create">
          <p class="wallet-create-note">{i18n.t('settings-import-missing-note', 'The file uses accounts, wallets or categories that do not exist yet. Create them to import its rows.')}</p>
          {#each importPreview.missing_accounts as acc}
            <div class="setting-row">
              <span class="setting-label">{acc.name} · {acc.currency}</span>
              <select
                value={missingAccountTypes[acc.name] ?? 'bank'}
                onchange={(e) => missingAccountTypes[acc.name] = (e.target as HTMLSelectElement).value}
                aria-label={i18n.t('finances-type', 'Type')}
              >
                <option value="bank">{i18n.t('finances-account-type-bank', 'Bank')}</option>
                <option value="savings">{i18n.t('finances-account-type-savings', 'Savings')}</option>
                <option value="credit">{i18n.t('finances-account-type-credit', 'Credit Card')}</option>
                <option value="cash">{i18n.t('finances-account-type-cash', 'Cash')}</option>
                <option value="other">{i18n.t('finances-account-type-other', 'Other')}</option>
              </select>
            </div>
          {/each}
          {#each importPreview.missing_wallets as wallet}
            <div class="setting-row">
              <span class="setting-label">{wallet}</span>
              <select
                value={missingWalletCategories[wallet] ?? 'exchange'}
                onchange={(e) => missingWalletCategories[wallet] = (e.target as HTMLSelectElement).value}
                aria-label={i18n.t('crypto-wallet-category', 'Category')}
              >
                <option value="exchange">Exchange</option>
                <option value="hardware">Hardware</option>
                <option value="software">Software</option>
              </select>
            </div>
          {/each}
          {#each importPreview.missing_categories as cat}
            <div class="setting-row">
              <span class="setting-label">{cat.name}</span>
              <span class="setting-desc">{cat.category_type === 'income' ? i18n.t('finances-income', 'Income') : i18n.t('finances-expense', 'Expense')}</span>
            </div>
          {/each}
        </div>
      {/if}
      {#if importPreview.errors.length > 0}
        <details class="import-errors" open={!hasMissing}>
          <summary class="error-heading">{i18n.t('settings-import-errors', 'Errors')} ({importPreview.errors.length})</summary>
          {#each importPreview.errors as err}
            <p class="error-line">
              {#if err.line}{i18n.t('settings-import-line', 'Line')} {err.line}: {/if}{err.message}
            </p>
          {/each}
        </details>
      {/if}
      <div class="import-actions">
        <button class="secondary-btn" onclick={resetImport}>{i18n.t('settings-cancel', 'Cancel')}</button>
        {#if hasMissing}
          <button class="primary-btn" onclick={createMissingAndPreview} disabled={importLoading}>
            {i18n.t('settings-import-create-missing', 'Create and preview again')}
          </button>
        {:else}
          <button class="primary-btn" onclick={confirmImport} disabled={importLoading}>
            {importLoading ? i18n.t('settings-import-importing', 'Importing...') : i18n.t('settings-import-confirm', 'Confirm Import')}
          </button>
        {/if}
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

<style>
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
  .hidden-input { display: none; }
  .import-card {
    background: var(--card-bg); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 16px; margin-top: 8px; box-shadow: var(--card-shadow);
  }
  .import-info { font-size: 0.85rem; color: var(--text-secondary); margin: 0 0 12px; }
  .import-info strong { color: var(--text-primary); }
  .import-actions {
    display: flex; gap: 10px; justify-content: flex-end; margin-top: 14px;
  }
  .secondary-btn:disabled {
    opacity: 0.4; cursor: not-allowed;
  }
  .import-errors { margin-top: 8px; }
  .error-heading { color: var(--danger); font-size: 0.85rem; margin: 0 0 6px; font-weight: 600; }
  summary.error-heading { cursor: pointer; }
  .error-line { color: var(--text-secondary); font-size: 0.8rem; margin: 2px 0; padding-left: 8px; }
  .wallet-create { margin: 8px 0; padding: 10px 12px; border-radius: 8px; background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.3); }
  .wallet-create-note { margin: 0 0 8px; font-size: 0.82rem; color: var(--text-secondary); }
  .wallet-create select { padding: 6px 8px; border-radius: 6px; background: var(--input-bg, rgba(255, 255, 255, 0.04)); border: 1px solid var(--glass-border); color: var(--text-primary); }
</style>
