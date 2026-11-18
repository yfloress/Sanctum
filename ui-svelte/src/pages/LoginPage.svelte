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
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as vaultApi from '../lib/api/vault'
  import * as settingsApi from '../lib/api/settings'
  import * as financeApi from '../lib/api/finance'
  import { open } from '@tauri-apps/plugin-dialog'

  let vaultExists = $state<boolean | null>(null)
  let password = $state('')
  let showPassword = $state(false)
  let loading = $state(false)
  let error = $state('')
  let weakWarning = $state('')
  let confirmWeak = $state(false)
  let capsLock = $state(false)

  /** Focuses the password field as soon as it appears. */
  function focusOnMount(node: HTMLInputElement) {
    node.focus()
  }

  /** The weak-password confirmation is for the password it warned about. */
  function onPasswordInput() {
    if (confirmWeak || weakWarning) {
      confirmWeak = false
      weakWarning = ''
    }
  }

  function trackCapsLock(e: KeyboardEvent) {
    capsLock = e.getModifierState?.('CapsLock') ?? false
  }

  $effect(() => {
    vaultApi.checkVaultExists().then((exists) => {
      vaultExists = exists
    })
  })

  async function handleSubmit() {
    if (!password.trim()) return
    error = ''
    loading = true

    try {
      if (vaultExists) {
        await vaultApi.unlockVault(password)
      } else {
        if (!confirmWeak) {
          const strength = await vaultApi.checkPasswordStrength(password)
          if (strength.warning) {
            weakWarning = strength.warning
            confirmWeak = true
            loading = false
            return
          }
        }
        await vaultApi.createVault(password)
      }

      const settings = await settingsApi.loadSettings()
      app.settings = settings
      await i18n.load()

      // Fill in the recurring entries owed while the vault was closed, before
      // the dashboard reads any balance.
      try {
        const created = await financeApi.applyDueRecurring()
        if (created > 0) {
          app.showToast(
            i18n.tArgs(
              'finances-recurring-applied',
              { count: created },
              '{$count} recurring transactions added'
            )
          )
        }
      } catch {
        // Never block the unlock over this; the panel can retry later.
      }

      app.login()
    } catch (e) {
      error = errorMessage(e)
      confirmWeak = false
    } finally {
      loading = false
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    trackCapsLock(e)
    if (e.key === 'Enter') handleSubmit()
  }

  async function restoreFromBackup() {
    try {
      const path = await open({
        title: 'Select Vault Backup',
        filters: [{ name: 'Sanctum Backup', extensions: ['db'] }],
      })
      if (!path) return
      loading = true
      await vaultApi.restoreVault(path as string)
      vaultExists = await vaultApi.checkVaultExists()
      error = ''
      password = ''
    } catch (e) {
      error = errorMessage(e)
    } finally {
      loading = false
    }
  }
</script>

<div class="login-page">
  <div class="login-card">
    <div class="logo-section">
      <div class="logo-tile">
        <img src="/assets/logo/sanctum_logo.svg" alt="" class="login-logo" />
      </div>
      <h1 class="title">Sanctum</h1>
      <p class="subtitle">{i18n.t('login-subtitle', 'Privacy-first personal vault')}</p>
    </div>

    {#if vaultExists === null}
      <div class="loading-state">
        <div class="loading-spinner"></div>
        <span>{i18n.t('login-initializing', 'Initializing...')}</span>
      </div>
    {:else}
      <div class="form-section">
        <label class="field-label" for="master-password">
          {vaultExists ? i18n.t('login-label-unlock', 'Master password') : i18n.t('login-label-create', 'Choose a master password')}
        </label>
        <div class="input-group">
          <svg class="input-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" />
          </svg>
          <input
            type={showPassword ? 'text' : 'password'}
            bind:value={password}
            oninput={onPasswordInput}
            onkeydown={handleKeydown}
            onkeyup={trackCapsLock}
            onblur={() => capsLock = false}
            use:focusOnMount
            placeholder={vaultExists ? i18n.t('login-placeholder-unlock', 'Enter master password') : i18n.t('login-placeholder-create', 'Create master password')}
            disabled={loading}
            autocomplete="off"
            id="master-password"
          />
          <button
            class="toggle-vis"
            onclick={() => showPassword = !showPassword}
            aria-label={showPassword ? i18n.t('login-hide-password', 'Hide password') : i18n.t('login-show-password', 'Show password')}
            tabindex={-1}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              {#if showPassword}
                <path d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
              {:else}
                <path d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                <path d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
              {/if}
            </svg>
          </button>
        </div>

        {#if capsLock}
          <p class="field-hint warn">{i18n.t('login-caps-lock', 'Caps Lock is on')}</p>
        {/if}
        {#if vaultExists === false}
          <p class="field-hint">{i18n.t('login-create-hint', 'It encrypts everything and cannot be recovered: if you forget it, the data is lost. Write it down somewhere safe.')}</p>
        {/if}

        {#if weakWarning}
          <div class="warning-banner">
            <svg class="banner-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126z" />
              <path d="M12 15.75h.007v.008H12v-.008z" />
            </svg>
            <div class="banner-text">
              <span>{weakWarning}</span>
              <span class="weak-hint">{i18n.t('login-weak-hint', 'Press again to confirm with weak password')}</span>
            </div>
          </div>
        {/if}

        {#if error}
          <div class="error-banner">
            <svg class="banner-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
            </svg>
            <span>{error}</span>
          </div>
        {/if}

        <button
          class="submit-btn"
          onclick={handleSubmit}
          disabled={loading || !password.trim()}
          id="submit-vault"
        >
          {#if loading}
            <div class="btn-spinner"></div>
            <span>{i18n.t('login-authenticating', 'Authenticating...')}</span>
          {:else}
            <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              {#if vaultExists}
                <path d="M13.5 10.5V6.75a4.5 4.5 0 119 0v3.75M3.75 21.75h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H3.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" />
              {:else if confirmWeak}
                <path d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              {:else}
                <path d="M12 4.5v15m7.5-7.5h-15" />
              {/if}
            </svg>
            <span>{vaultExists ? i18n.t('login-unlock', 'Unlock Vault') : confirmWeak ? i18n.t('login-confirm-create', 'Confirm Create') : i18n.t('login-create', 'Create Vault')}</span>
          {/if}
        </button>

        <!-- Also on a fresh install: moving to a new computer starts by
             restoring the backup, not by creating an empty vault. -->
        <button class="restore-link" onclick={restoreFromBackup} disabled={loading} id="restore-backup">
          {i18n.t('login-restore', 'Restore from backup')}
        </button>
      </div>
    {/if}
  </div>

  <div class="version-tag">{i18n.t('login-version', 'Sanctum')}</div>
</div>

<style>
  /* One flat look for both themes, the same as the rest of the app: a solid
     card on the dotted page, a solid button, no glow and no motion beyond the
     card's entrance. */
  .login-page {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 24px;
    box-sizing: border-box;
    /* The login screen cannot read the user's chosen background yet, so it
       always shows the dots (themed via --bg-dots). */
    background-color: var(--bg-base);
    background-image: var(--bg-dots), var(--bg-gradient);
    background-size: 24px 24px, cover;
    background-position: center, center;
    background-repeat: repeat, no-repeat;
  }

  .login-card {
    display: flex;
    flex-direction: column;
    gap: 28px;
    width: 100%;
    max-width: 400px;
    padding: 40px 36px 28px;
    box-sizing: border-box;
    background: var(--modal-bg);
    border: 1px solid var(--modal-border);
    border-radius: var(--radius-xl);
    box-shadow: var(--modal-shadow);
    animation: cardEntrance 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  @keyframes cardEntrance {
    from { opacity: 0; transform: translateY(12px); }
    to { opacity: 1; transform: none; }
  }

  /* ===== Logo ===== */
  .logo-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    text-align: center;
  }

  .logo-tile {
    display: grid;
    place-items: center;
    width: 64px;
    height: 64px;
    margin-bottom: 10px;
    border-radius: 18px;
    background: var(--accent-bg);
    border: 1px solid var(--accent-border);
  }

  .login-logo { width: 40px; height: 40px; }

  .title {
    margin: 0;
    font-size: 1.6rem;
    font-weight: 650;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }

  .subtitle {
    margin: 0;
    font-size: 0.88rem;
    color: var(--text-secondary);
  }

  /* ===== Loading ===== */
  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-tertiary);
    font-size: 0.85rem;
  }

  .loading-spinner {
    width: 18px;
    height: 18px;
    border: 2px solid var(--accent-border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* ===== Form ===== */
  .form-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .field-label {
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .input-group {
    position: relative;
    display: flex;
    align-items: center;
  }

  .input-icon {
    position: absolute;
    left: 14px;
    width: 18px;
    height: 18px;
    color: var(--text-tertiary);
    pointer-events: none;
    transition: color 0.15s;
  }

  .input-group:focus-within .input-icon { color: var(--accent); }

  input {
    width: 100%;
    padding: 13px 44px 13px 42px;
    box-sizing: border-box;
    border: 1px solid var(--glass-border-hover);
    border-radius: var(--radius-md);
    background: var(--select-bg);
    color: var(--text-primary);
    font-size: 0.95rem;
    outline: none;
    transition: border-color 0.15s, box-shadow 0.15s;
  }

  input:hover { border-color: var(--accent-border); }

  input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  input::placeholder { color: var(--text-tertiary); }

  .toggle-vis {
    position: absolute;
    right: 8px;
    display: flex;
    padding: 6px;
    border: none;
    border-radius: 6px;
    background: none;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color 0.15s;
  }

  .toggle-vis:hover { color: var(--text-primary); }
  .toggle-vis svg { width: 18px; height: 18px; }

  .field-hint {
    margin: -4px 0 0;
    font-size: 0.78rem;
    line-height: 1.45;
    color: var(--text-tertiary);
  }

  .field-hint.warn { color: var(--warning); }

  /* ===== Banners ===== */
  .warning-banner,
  .error-banner {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 11px 13px;
    border: 1px solid;
    border-radius: var(--radius-sm);
    font-size: 0.82rem;
    line-height: 1.45;
    animation: bannerIn 0.2s ease both;
  }

  @keyframes bannerIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: none; }
  }

  .warning-banner {
    color: var(--warning);
    background: color-mix(in srgb, var(--warning) 10%, transparent);
    border-color: color-mix(in srgb, var(--warning) 28%, transparent);
  }

  .error-banner {
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 9%, transparent);
    border-color: color-mix(in srgb, var(--danger) 26%, transparent);
  }

  .banner-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .banner-text {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .weak-hint { font-size: 0.75rem; opacity: 0.85; }

  /* ===== Buttons ===== */
  .submit-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    width: 100%;
    margin-top: 4px;
    padding: 13px 24px;
    border: none;
    border-radius: var(--radius-md);
    background: var(--accent-solid);
    color: var(--text-on-accent);
    font-size: 0.92rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s, transform 0.1s;
  }

  .submit-btn:hover:not(:disabled) { background: var(--accent-solid-hover); }
  .submit-btn:active:not(:disabled) { transform: scale(0.99); }
  .submit-btn:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

  .submit-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .btn-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .btn-spinner {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  .restore-link {
    align-self: center;
    padding: 6px 8px;
    border: none;
    border-radius: 6px;
    background: none;
    color: var(--text-secondary);
    font-size: 0.82rem;
    cursor: pointer;
    transition: color 0.15s;
  }

  .restore-link:hover { color: var(--accent); }
  .restore-link:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

  .restore-link:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .version-tag {
    position: fixed;
    bottom: 16px;
    right: 20px;
    font-size: 0.7rem;
    color: var(--text-tertiary);
  }

  @media (prefers-reduced-motion: reduce) {
    .login-card, .warning-banner, .error-banner { animation: none; }
  }

  @media (max-width: 480px) {
    .login-card { padding: 32px 22px 22px; gap: 24px; }
    .title { font-size: 1.45rem; }
  }
</style>
