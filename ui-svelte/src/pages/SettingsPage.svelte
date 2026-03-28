<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import * as settingsApi from '../lib/api/settings'
  import * as vaultApi from '../lib/api/vault'
  import type { AppInfo } from '../lib/types'

  let info = $state<AppInfo | null>(null)

  async function load() {
    try {
      info = await settingsApi.getAppInfo()
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function toggleDarkMode() {
    if (!app.settings) return
    const next = !app.settings.dark_mode
    app.settings.dark_mode = next
    await settingsApi.setDarkMode(next)
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
    await settingsApi.setProxyEnabled(next)
  }

  async function updateProxyUrl() {
    if (!app.settings) return
    await settingsApi.setProxyUrl(app.settings.proxy_url)
  }

  async function exportVault() {
    try {
      const result = await vaultApi.exportVault()
      app.showToast(`Backup saved to ${result.path}`)
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
        <button class="secondary-btn" onclick={exportVault}>Export</button>
      </div>
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
  h2 { font-size: 1.3rem; letter-spacing: 0.15em; color: #e0e0e0; margin-bottom: 28px; }

  .section {
    margin-bottom: 28px; padding-bottom: 24px; border-bottom: 1px solid #1a1a1a;
  }
  .section h3 {
    font-size: 0.8rem; color: #666; text-transform: uppercase;
    letter-spacing: 0.08em; margin-bottom: 14px;
  }
  .section-note { font-size: 0.8rem; color: #555; margin-bottom: 12px; }

  .setting-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: 10px 0; gap: 16px;
  }
  .setting-label { font-size: 0.9rem; color: #ccc; display: block; }
  .setting-desc { font-size: 0.75rem; color: #555; display: block; margin-top: 2px; }

  select, input[type="text"] {
    padding: 8px 12px; border: 1px solid #333; border-radius: 6px;
    background: #111; color: #e0e0e0; font-size: 0.85rem;
  }
  input[type="text"]:focus, select:focus { border-color: #4f9cf7; outline: none; }

  .toggle-switch {
    position: relative; width: 44px; height: 24px; border-radius: 12px;
    border: none; background: #333; cursor: pointer; padding: 0;
    transition: background 0.2s;
  }
  .toggle-switch.on { background: #4f9cf7; }
  .toggle-knob {
    position: absolute; top: 2px; left: 2px; width: 20px; height: 20px;
    border-radius: 50%; background: #e0e0e0; transition: transform 0.2s;
  }
  .toggle-switch.on .toggle-knob { transform: translateX(20px); }

  .secondary-btn {
    padding: 8px 18px; border: 1px solid #333; border-radius: 6px;
    background: none; color: #ccc; cursor: pointer; font-size: 0.85rem;
  }
  .secondary-btn:hover { border-color: #555; }

  .danger-btn {
    padding: 8px 18px; border: 1px solid #5a2d2d; border-radius: 6px;
    background: #3a1a1a; color: #f87171; cursor: pointer; font-size: 0.85rem;
  }
  .danger-btn:hover { background: #4a2020; }

  .danger-section { border-bottom: none; }

  .about-grid {
    display: grid; grid-template-columns: 100px 1fr; gap: 8px;
    font-size: 0.85rem; color: #ccc;
  }
  .about-label { color: #666; }
</style>
