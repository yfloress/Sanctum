<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import LiquidGlassButton from '../components/LiquidGlassButton.svelte'
  import * as settingsApi from '../lib/api/settings'
  import * as vaultApi from '../lib/api/vault'
  import * as ingestionApi from '../lib/api/ingestion'
  import { save } from '@tauri-apps/plugin-dialog'
  import type {
    AppInfo, ImportPreviewResponse, ImportResultsResponse,
    ExchangeDetectionResult
  } from '../lib/types'

  let info = $state<AppInfo | null>(null)
  let maxFileSize = $state(0)

  // Import state
  type ImportStep = 'idle' | 'preview' | 'results'
  let importStep = $state<ImportStep>('idle')
  let importMode = $state<'generic' | 'exchange'>('generic')
  let importFilename = $state('')
  let importContent = $state('')
  let importPreview = $state<ImportPreviewResponse | null>(null)
  let importResults = $state<ImportResultsResponse | null>(null)
  let importLoading = $state(false)

  // Exchange detection
  let exchangeDetection = $state<ExchangeDetectionResult | null>(null)
  let exchangeWalletName = $state('')

  async function load() {
    try {
      const [appInfo, maxSize] = await Promise.all([
        settingsApi.getAppInfo(),
        ingestionApi.maxImportFileSize(),
      ])
      info = appInfo
      maxFileSize = maxSize
    } catch (e) {
      app.showToast(String(e), true)
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
  }

  let genericFileInput = $state<HTMLInputElement>(null!)
  let exchangeFileInput = $state<HTMLInputElement>(null!)

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
      app.showToast(String(e), true)
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
        app.showToast('Could not detect exchange format', true)
        importLoading = false
        return
      }
      exchangeDetection = detection
      exchangeWalletName = detection.suggested_wallet
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      importLoading = false
      exchangeFileInput.value = ''
    }
  }

  async function previewExchange() {
    if (!exchangeWalletName.trim()) {
      app.showToast('Wallet name is required', true)
      return
    }
    try {
      importLoading = true
      importPreview = await ingestionApi.previewExchangeCsv(importContent, exchangeWalletName.trim())
      importStep = 'preview'
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      importLoading = false
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
      app.showToast(String(e), true)
    } finally {
      importLoading = false
    }
  }

  async function toggleDarkMode() {
    if (!app.settings) return
    const next = !app.settings.dark_mode
    app.settings.dark_mode = next
    await settingsApi.setDarkMode(next).catch(e => app.showToast(String(e), true))
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
    app.settings.preferred_language = val
    await settingsApi.setPreferredLanguage(val)
    await i18n.load()
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
    await settingsApi.setProxyUrl(app.settings.proxy_url)
  }

  async function exportVault() {
    try {
      const path = await save({
        title: 'Export Vault Backup',
        filters: [{ name: 'Sanctum Backup', extensions: ['db'] }],
      })
      if (!path) return
      await vaultApi.exportVault(path)
      app.showToast('Backup saved successfully')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function resetAllSettings() {
    try {
      await settingsApi.resetSettings()
      app.settings = await settingsApi.loadSettings()
      app.showToast('Settings reset to defaults')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  $effect(() => { load() })
</script>

<div class="page">
  <h2>Settings</h2>

  {#if app.settings}
    <!-- Appearance -->
    <section class="section">
      <h3>Appearance</h3>
      <div class="setting-row">
        <div>
          <span class="setting-label">Dark Mode</span>
          <span class="setting-desc">Toggle dark/light theme</span>
        </div>
        <button class="toggle-switch" class:on={app.settings.dark_mode} onclick={toggleDarkMode} aria-label="Toggle dark mode">
          <span class="toggle-knob"></span>
        </button>
      </div>
    </section>

    <!-- Regional -->
    <section class="section">
      <h3>Regional</h3>
      <div class="setting-row">
        <span class="setting-label">Preferred Currency</span>
        <select value={app.settings.preferred_currency} onchange={changeCurrency}>
          {#each ['USD', 'CLP', 'EUR', 'GBP', 'BRL', 'MXN', 'ARS', 'CAD', 'AUD', 'CHF', 'JPY'] as cur}
            <option value={cur}>{cur}</option>
          {/each}
        </select>
      </div>
      <div class="setting-row">
        <span class="setting-label">Language</span>
        <select value={app.settings.preferred_language} onchange={changeLanguage}>
          <option value="en">English</option>
          <option value="es">Espanol</option>
        </select>
      </div>
    </section>

    <!-- Security -->
    <section class="section">
      <h3>Security</h3>
      <div class="setting-row">
        <div>
          <span class="setting-label">Session Timeout</span>
          <span class="setting-desc">Auto-lock after inactivity</span>
        </div>
        <select value={app.settings.session_timeout_secs} onchange={changeTimeout}>
          <option value={300}>5 minutes</option>
          <option value={900}>15 minutes</option>
          <option value={1800}>30 minutes</option>
          <option value={3600}>1 hour</option>
          <option value={0}>Never</option>
        </select>
      </div>
    </section>

    <!-- Vault Backup -->
    <section class="section">
      <h3>Vault Backup</h3>
      <p class="section-note">Your vault is encrypted with SQLCipher (AES-256).</p>
      <div class="setting-row">
        <span class="setting-label">Export Vault</span>
        <LiquidGlassButton text="Export" contrast="dark" onclick={exportVault} />
      </div>
    </section>

    <!-- Data Import -->
    <section class="section">
      <h3>Data Import</h3>

      {#if importStep === 'idle'}
        {#if exchangeDetection}
          <!-- Exchange detected, ask for wallet name -->
          <div class="import-card">
            <p class="import-info">
              Detected: <strong>{exchangeDetection.exchange}</strong>
              ({exchangeDetection.total_records} records)
            </p>
            <div class="setting-row">
              <span class="setting-label">Target Wallet</span>
              <input
                type="text"
                bind:value={exchangeWalletName}
                placeholder="Wallet name"
              />
            </div>
            <div class="import-actions">
              <button class="secondary-btn" onclick={resetImport}>Cancel</button>
              <button class="primary-btn" onclick={previewExchange} disabled={importLoading}>
                {importLoading ? 'Loading...' : 'Preview'}
              </button>
            </div>
          </div>
        {:else}
          <input type="file" accept=".csv" class="hidden-input" bind:this={genericFileInput} onchange={handleGenericFile} />
          <input type="file" accept=".csv" class="hidden-input" bind:this={exchangeFileInput} onchange={handleExchangeFile} />
          <div class="setting-row">
            <div>
              <span class="setting-label">Generic CSV</span>
              <span class="setting-desc">Import transactions from a CSV file</span>
            </div>
            <button class="secondary-btn" onclick={() => genericFileInput.click()} disabled={importLoading}>
              {importLoading ? 'Loading...' : 'Select File'}
            </button>
          </div>
          <div class="setting-row">
            <div>
              <span class="setting-label">Exchange CSV</span>
              <span class="setting-desc">Import from Kraken, Binance, MEXC, and more</span>
            </div>
            <button class="secondary-btn" onclick={() => exchangeFileInput.click()} disabled={importLoading}>
              {importLoading ? 'Loading...' : 'Select File'}
            </button>
          </div>
        {/if}

      {:else if importStep === 'preview' && importPreview}
        <!-- Preview results -->
        <div class="import-card">
          <p class="import-info">
            Source: <strong>{importPreview.source}</strong> |
            {importPreview.total_records} records |
            {importPreview.to_add} to add |
            {importPreview.to_skip} to skip
          </p>
          {#if importPreview.changes.length > 0}
            <div class="import-changes">
              {#each importPreview.changes as change}
                <div class="change-row">
                  <span class="change-action">{change.action}</span>
                  <span class="change-desc">{change.description}</span>
                </div>
              {/each}
            </div>
          {/if}
          <div class="import-actions">
            <button class="secondary-btn" onclick={resetImport}>Cancel</button>
            <button class="primary-btn" onclick={confirmImport} disabled={importLoading}>
              {importLoading ? 'Importing...' : 'Confirm Import'}
            </button>
          </div>
        </div>

      {:else if importStep === 'results' && importResults}
        <!-- Import results -->
        <div class="import-card">
          <p class="import-info">
            Processed: {importResults.total_processed} |
            Inserted: {importResults.inserted} |
            Skipped: {importResults.skipped}
          </p>
          {#if importResults.errors.length > 0}
            <div class="import-errors">
              <p class="error-heading">Errors ({importResults.errors.length}):</p>
              {#each importResults.errors as err}
                <p class="error-line">
                  {#if err.line}Line {err.line}: {/if}{err.message}
                </p>
              {/each}
            </div>
          {/if}
          <div class="import-actions">
            <button class="secondary-btn" onclick={resetImport}>Done</button>
          </div>
        </div>
      {/if}
    </section>

    <!-- Data Sync -->
    <section class="section">
      <h3>Data Sync</h3>
      <div class="setting-row">
        <div>
          <span class="setting-label">Auto-fetch Prices</span>
          <span class="setting-desc">Automatically fetch crypto prices on sync</span>
        </div>
        <button class="toggle-switch" class:on={app.settings.auto_fetch} onclick={toggleAutoFetch} aria-label="Toggle auto-fetch">
          <span class="toggle-knob"></span>
        </button>
      </div>
      <div class="setting-row">
        <div>
          <span class="setting-label">Use Proxy</span>
          <span class="setting-desc">Route API calls through a proxy</span>
        </div>
        <button class="toggle-switch" class:on={app.settings.proxy_enabled} onclick={toggleProxy} aria-label="Toggle proxy">
          <span class="toggle-knob"></span>
        </button>
      </div>
      {#if app.settings.proxy_enabled}
        <div class="setting-row">
          <span class="setting-label">Proxy URL</span>
          <input
            type="text"
            bind:value={app.settings.proxy_url}
            onblur={updateProxyUrl}
            placeholder="https://proxy.example.com"
          />
        </div>
      {/if}
    </section>

    <!-- About -->
    {#if info}
      <section class="section">
        <h3>About</h3>
        <div class="about-grid">
          <span class="about-label">Version</span><span>{info.version}</span>
          <span class="about-label">Encryption</span><span>{info.encryption}</span>
          <span class="about-label">Storage</span><span>{info.storage}</span>
        </div>
      </section>
    {/if}

    <!-- Danger Zone -->
    <section class="section danger-section">
      <h3>Reset</h3>
      <div class="setting-row">
        <div>
          <span class="setting-label">Reset All Settings</span>
          <span class="setting-desc">Restore default values for all settings</span>
        </div>
        <button class="danger-btn" onclick={resetAllSettings}>Reset</button>
      </div>
    </section>
  {/if}
</div>

<style>
  .page { padding: 24px 32px; max-width: 640px; }
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
  .setting-label { font-size: 0.9rem; color: #ccc; display: block; }
  .setting-desc { font-size: 0.75rem; color: var(--text-tertiary); display: block; margin-top: 2px; }

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
    background: rgba(79, 156, 247, 0.25); border-color: rgba(79, 156, 247, 0.3);
    box-shadow: 0 0 12px var(--accent-glow);
  }
  .toggle-knob {
    position: absolute; top: 2px; left: 2px; width: 18px; height: 18px;
    border-radius: 50%; background: var(--text-primary); transition: transform 0.25s;
  }
  .toggle-switch.on .toggle-knob { transform: translateX(20px); }

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

  .danger-section { border-bottom: none; }

  .about-grid {
    display: grid; grid-template-columns: 100px 1fr; gap: 8px;
    font-size: 0.85rem; color: #ccc;
  }
  .about-label { color: var(--text-tertiary); }

  .hidden-input { display: none; }
  .import-card {
    background: var(--glass); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    padding: 16px; margin-top: 8px; box-shadow: var(--glass-glow);
  }
  .import-info { font-size: 0.85rem; color: #aaa; margin: 0 0 12px; }
  .import-info strong { color: var(--text-primary); }
  .import-actions {
    display: flex; gap: 10px; justify-content: flex-end; margin-top: 14px;
  }
  .primary-btn {
    padding: 8px 18px; border: 1px solid rgba(79, 156, 247, 0.3); border-radius: var(--radius-sm);
    background: rgba(79, 156, 247, 0.2); backdrop-filter: blur(8px);
    color: #fff; cursor: pointer; font-size: 0.85rem;
    transition: all 0.2s;
  }
  .primary-btn:hover:not(:disabled) { background: rgba(79, 156, 247, 0.3); box-shadow: 0 0 16px var(--accent-glow); }
  .primary-btn:disabled, .secondary-btn:disabled {
    opacity: 0.4; cursor: not-allowed;
  }
  .import-changes {
    max-height: 200px; overflow-y: auto; margin-bottom: 8px;
    border: 1px solid var(--glass-border); border-radius: 4px;
  }
  .change-row {
    display: flex; gap: 10px; padding: 6px 10px;
    font-size: 0.8rem; border-bottom: 1px solid var(--glass-border);
  }
  .change-row:last-child { border-bottom: none; }
  .change-action {
    color: var(--accent); font-weight: 600; min-width: 60px; text-transform: uppercase;
    font-size: 0.7rem;
  }
  .change-desc { color: #aaa; }
  .import-errors { margin-top: 8px; }
  .error-heading { color: var(--danger); font-size: 0.85rem; margin: 0 0 6px; font-weight: 600; }
  .error-line { color: #ccc; font-size: 0.8rem; margin: 2px 0; padding-left: 8px; }
</style>
