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
  import { setPageActions, consumePendingCommand } from '../lib/shortcuts'
  import { app, type BackgroundFx } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as settingsApi from '../lib/api/settings'
  import * as vaultApi from '../lib/api/vault'
  import * as financeApi from '../lib/api/finance'
  import { save } from '@tauri-apps/plugin-dialog'
  import type { AppInfo } from '../lib/types'

  import ConfirmDialog from '../components/ConfirmDialog.svelte'
  import DataImport from '../components/DataImport.svelte'

  let info = $state<AppInfo | null>(null)
  let settingsLoading = $state(true)
  let showResetConfirm = $state(false)

  async function load() {
    try {
      info = await settingsApi.getAppInfo()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      settingsLoading = false
    }
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
    try {
      await settingsApi.setAutoFetch(next)
      app.settings.auto_fetch = next
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function toggleProxy() {
    if (!app.settings) return
    const next = !app.settings.proxy_enabled
    try {
      await settingsApi.setProxyEnabled(next)
      app.settings.proxy_enabled = next
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function updateProxyUrl() {
    if (!app.settings) return
    try {
      app.settings.proxy_url = await settingsApi.setProxyUrl(app.settings.proxy_url)
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

  let showPasswordChange = $state(false)
  let currentPassword = $state('')
  let newPassword = $state('')
  let confirmPassword = $state('')
  let changingPassword = $state(false)
  let passwordError = $state('')
  let rollbackPath = $state('')

  let passwordsMatch = $derived(newPassword.length > 0 && newPassword === confirmPassword)

  function resetPasswordForm() {
    showPasswordChange = false
    currentPassword = ''
    newPassword = ''
    confirmPassword = ''
    passwordError = ''
    rollbackPath = ''
  }

  async function changePassword() {
    passwordError = ''
    if (!passwordsMatch) {
      passwordError = i18n.t('settings-password-mismatch', 'The new passwords do not match')
      return
    }

    changingPassword = true
    try {
      rollbackPath = await vaultApi.changeVaultPassword(currentPassword, newPassword)
      currentPassword = ''
      newPassword = ''
      confirmPassword = ''
      app.showToast(i18n.t('settings-password-changed', 'Master password changed'))
    } catch (e) {
      passwordError = errorMessage(e)
    } finally {
      changingPassword = false
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

  // ── Palette targets ────────────────────────────────────────────────────────

  /** Which setting the palette asked for, until it has been scrolled to. */
  let revealed = $state<string | null>(null)
  let revealTimer = 0

  /**
   * Marks a setting as the one to go to. The scrolling itself waits for the
   * effect below, because the row may not exist yet: arriving from another page
   * the whole section is still behind the `app.settings` guard.
   */
  function reveal(target: string) {
    revealed = target
  }

  $effect(() => {
    const target = revealed
    // Read so the effect runs again once the page has something to scroll to.
    const ready = app.settings !== null
    if (!target || !ready) return

    const frame = requestAnimationFrame(() => {
      document
        .getElementById(`setting-${target}`)
        ?.scrollIntoView({ block: 'center', behavior: prefersReducedMotion() ? 'auto' : 'smooth' })
    })

    // Long enough to catch the eye after the scroll settles, short enough that
    // the page is not left permanently marked.
    clearTimeout(revealTimer)
    revealTimer = window.setTimeout(() => { revealed = null }, 2400)

    return () => {
      cancelAnimationFrame(frame)
      clearTimeout(revealTimer)
    }
  })

  function prefersReducedMotion(): boolean {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches
  }

  /** Bodies for the palette commands declared for this page in `lib/commands.ts`. */
  function commandHandlers(): Record<string, () => void> {
    return {
      'set-export-vault': () => void exportVault(),
      'set-export-csv': () => void exportTransactionsCsv(),
      'set-password': () => reveal('password'),
      'set-timeout': () => reveal('timeout'),
      'set-currency': () => reveal('currency'),
      'set-language': () => reveal('language'),
      'set-background': () => reveal('background'),
      'set-auto-fetch': () => reveal('auto-fetch'),
      'set-proxy': () => reveal('proxy'),
      'set-import': () => reveal('import'),
      'set-reset': () => reveal('reset'),
    }
  }

  $effect(() => setPageActions({ handlers: commandHandlers() }))

  $effect(() => consumePendingCommand(commandHandlers()))
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
        <button class="toggle-switch" class:on={app.settings.dark_mode} onclick={() => app.toggleDarkMode()} aria-label="Toggle dark mode">
          <span class="toggle-knob"></span>
        </button>
      </div>
      <div class="setting-row" id="setting-background" class:setting-highlight={revealed === 'background'}>
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
      <div class="setting-row" id="setting-currency" class:setting-highlight={revealed === 'currency'}>
        <span class="setting-label">{i18n.t('settings-preferred-currency', 'Preferred Currency')}</span>
        <select value={app.settings.preferred_currency} onchange={changeCurrency}>
          {#each ['USD', 'CLP', 'EUR', 'GBP', 'BRL', 'MXN', 'ARS', 'CAD', 'AUD', 'CHF', 'JPY'] as cur}
            <option value={cur}>{cur}</option>
          {/each}
        </select>
      </div>
      <div class="setting-row" id="setting-language" class:setting-highlight={revealed === 'language'}>
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
      <div class="setting-row" id="setting-timeout" class:setting-highlight={revealed === 'timeout'}>
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

      <div class="setting-row" id="setting-password" class:setting-highlight={revealed === 'password'}>
        <div>
          <span class="setting-label">{i18n.t('settings-change-password', 'Master Password')}</span>
          <span class="setting-desc">{i18n.t('settings-change-password-desc', 'Re-encrypts the whole vault with a new password')}</span>
        </div>
        {#if !showPasswordChange}
          <button class="glass-btn" onclick={() => (showPasswordChange = true)}>
            {i18n.t('settings-change-password-btn', 'Change')}
          </button>
        {/if}
      </div>

      {#if showPasswordChange}
        <div class="password-form">
          {#if rollbackPath}
            <p class="password-done">
              {i18n.t('settings-password-changed', 'Master password changed')}
            </p>
            <p class="password-warning">
              {i18n.t('settings-password-rollback-at', 'Rollback copy, still using the old password:')}
              <code>{rollbackPath}</code>
            </p>
            <div class="password-actions">
              <button class="primary-btn" onclick={resetPasswordForm}>
                {i18n.t('settings-import-done', 'Done')}
              </button>
            </div>
          {:else}
            <p class="password-warning">
              {i18n.t(
                'settings-password-backup-warning',
                'A rollback copy is written next to the vault first. It keeps the OLD password: restoring it later needs the password you are replacing now.'
              )}
            </p>
            <input
              type="password"
              autocomplete="current-password"
              placeholder={i18n.t('settings-password-current', 'Current password')}
              bind:value={currentPassword}
            />
            <input
              type="password"
              autocomplete="new-password"
              placeholder={i18n.t('settings-password-new', 'New password')}
              bind:value={newPassword}
            />
            <input
              type="password"
              autocomplete="new-password"
              placeholder={i18n.t('settings-password-confirm', 'Confirm new password')}
              bind:value={confirmPassword}
            />
            {#if passwordError}
              <p class="password-error">{passwordError}</p>
            {/if}
            <div class="password-actions">
              <button class="secondary-btn" onclick={resetPasswordForm} disabled={changingPassword}>
                {i18n.t('settings-cancel', 'Cancel')}
              </button>
              <button
                class="primary-btn"
                onclick={changePassword}
                disabled={changingPassword || !currentPassword || !passwordsMatch}
              >
                {changingPassword
                  ? i18n.t('settings-password-changing', 'Re-encrypting…')
                  : i18n.t('settings-change-password-btn', 'Change')}
              </button>
            </div>
          {/if}
        </div>
      {/if}
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
    <section class="section" id="setting-import" class:setting-highlight={revealed === 'import'}>
      <h3>{i18n.t('settings-data-import', 'Data Import')}</h3>

      <DataImport />
    </section>

    <!-- Data Sync -->
    <section class="section">
      <h3>{i18n.t('settings-data-sync', 'Data Sync')}</h3>
      <div class="setting-row" id="setting-auto-fetch" class:setting-highlight={revealed === 'auto-fetch'}>
        <div>
          <span class="setting-label">{i18n.t('settings-auto-fetch', 'Auto-fetch Prices')}</span>
          <span class="setting-desc">{i18n.t('settings-auto-fetch-desc', 'Off: Sanctum never goes online on its own. On: refreshes prices every 10 minutes while the window is visible.')}</span>
        </div>
        <button class="toggle-switch" class:on={app.settings.auto_fetch} onclick={toggleAutoFetch} aria-label="Toggle auto-fetch">
          <span class="toggle-knob"></span>
        </button>
      </div>
      <div class="setting-row" id="setting-proxy" class:setting-highlight={revealed === 'proxy'}>
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
            placeholder={i18n.t('settings-proxy-placeholder', 'socks5h://127.0.0.1:9050')}
          />
        </div>
        {#if !app.settings.proxy_url.trim()}
          <p class="section-note">{i18n.t('settings-proxy-no-url', 'Without a proxy URL, Sanctum makes no network requests.')}</p>
        {/if}
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
      <div class="setting-row" id="setting-reset" class:setting-highlight={revealed === 'reset'}>
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

  .password-form {
    display: flex; flex-direction: column; gap: 10px;
    padding: 14px; margin: 4px 0 8px;
    border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--glass);
  }
  .password-warning {
    margin: 0; font-size: 0.8rem; line-height: 1.5; color: var(--warning);
  }
  .password-form input {
    padding: 9px 12px;
    border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.85rem;
    transition: border-color 0.2s;
  }
  .password-form input:focus {
    border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .password-error { margin: 0; font-size: 0.8rem; color: var(--danger); }
  .password-done { margin: 0; font-size: 0.85rem; color: var(--success); }
  .password-warning code {
    display: block; margin-top: 4px; word-break: break-all;
    font-size: 0.75rem; color: var(--text-secondary);
  }
  .password-actions { display: flex; justify-content: flex-end; gap: 8px; }

  .setting-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: 10px 0; gap: 16px;
  }
  /* Where a palette command just landed. An outline rather than a border or a
     shadow: it is drawn outside the box and takes up no space, so nothing on
     the page shifts when it appears. It fades on its own, because a page left
     permanently marked stops meaning anything. */
  .setting-highlight {
    border-radius: var(--radius-sm);
    outline: 2px solid var(--accent);
    outline-offset: 4px;
    animation: setting-flash 2.4s ease-out forwards;
  }
  @keyframes setting-flash {
    0%, 60% { outline-color: var(--accent); }
    100% { outline-color: transparent; }
  }
  @media (prefers-reduced-motion: reduce) {
    .setting-highlight { animation: none; }
  }

  .setting-label { font-size: 0.9rem; color: var(--text-secondary); display: block; }
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

  .secondary-btn:disabled {
    opacity: 0.4; cursor: not-allowed;
  }
</style>
