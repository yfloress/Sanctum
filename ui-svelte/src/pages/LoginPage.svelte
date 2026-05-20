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
  import * as vaultApi from '../lib/api/vault'
  import * as settingsApi from '../lib/api/settings'
  import { open } from '@tauri-apps/plugin-dialog'

  let vaultExists = $state<boolean | null>(null)
  let password = $state('')
  let showPassword = $state(false)
  let loading = $state(false)
  let error = $state('')
  let weakWarning = $state('')
  let confirmWeak = $state(false)

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
      app.login()
    } catch (e) {
      error = String(e)
      confirmWeak = false
    } finally {
      loading = false
    }
  }

  function handleKeydown(e: KeyboardEvent) {
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
      error = String(e)
    } finally {
      loading = false
    }
  }
</script>

<div class="login-page">
  <!-- Floating particles -->
  <div class="particles" aria-hidden="true">
    <div class="particle p1"></div>
    <div class="particle p2"></div>
    <div class="particle p3"></div>
    <div class="particle p4"></div>
    <div class="particle p5"></div>
    <div class="particle p6"></div>
  </div>

  <div class="login-card">
    <div class="card-shimmer"></div>

    <div class="logo-section">
      <div class="logo-ring">
        <img src="/assets/logo/sanctum_logo.svg" alt="Sanctum" class="login-logo" />
      </div>
      <h1 class="title">SANCTUM</h1>
      <p class="subtitle">{i18n.t('login-subtitle', 'Privacy-first personal vault')}</p>
    </div>

    {#if vaultExists === null}
      <div class="loading-state">
        <div class="loading-spinner"></div>
        <span>{i18n.t('login-initializing', 'Initializing...')}</span>
      </div>
    {:else}
      <div class="form-section">
        <div class="input-group">
          <svg class="input-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" />
          </svg>
          <input
            type={showPassword ? 'text' : 'password'}
            bind:value={password}
            onkeydown={handleKeydown}
            placeholder={vaultExists ? i18n.t('login-placeholder-unlock', 'Enter master password') : i18n.t('login-placeholder-create', 'Create master password')}
            disabled={loading}
            autocomplete="off"
            id="master-password"
          />
          <button
            class="toggle-vis"
            onclick={() => showPassword = !showPassword}
            aria-label={showPassword ? 'Hide password' : 'Show password'}
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

        {#if vaultExists}
          <button class="restore-link" onclick={restoreFromBackup} disabled={loading} id="restore-backup">
            {i18n.t('login-restore', 'Restore from backup')}
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <div class="version-tag">{i18n.t('login-version', 'Sanctum v0.1.0')}</div>
</div>

<style>
  /* ===== Page Layout ===== */
  .login-page {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: var(--bg-gradient);
    position: relative;
    overflow: hidden;
  }

  .login-page::before {
    content: '';
    position: absolute;
    top: -50%;
    left: -50%;
    width: 200%;
    height: 200%;
    background:
      radial-gradient(ellipse at 20% 20%, rgba(168, 85, 247, 0.08) 0%, transparent 50%),
      radial-gradient(ellipse at 80% 80%, rgba(74, 222, 128, 0.04) 0%, transparent 50%),
      radial-gradient(ellipse at 60% 10%, rgba(139, 92, 246, 0.05) 0%, transparent 40%);
    animation: auroraShift 20s ease-in-out infinite alternate;
    pointer-events: none;
  }

  @keyframes auroraShift {
    0% { transform: translate(0, 0) rotate(0deg); }
    50% { transform: translate(1%, -1%) rotate(1.5deg); }
    100% { transform: translate(2%, -2%) rotate(3deg); }
  }

  /* ===== Floating Particles ===== */
  .particles {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 0;
  }

  .particle {
    position: absolute;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(168, 85, 247, 0.25) 0%, transparent 70%);
    filter: blur(1px);
    animation: float linear infinite;
  }

  .p1 { width: 4px; height: 4px; left: 15%; top: 25%; animation-duration: 18s; animation-delay: 0s; }
  .p2 { width: 3px; height: 3px; left: 75%; top: 35%; animation-duration: 22s; animation-delay: -4s; }
  .p3 { width: 5px; height: 5px; left: 45%; top: 65%; animation-duration: 15s; animation-delay: -8s; }
  .p4 { width: 3px; height: 3px; left: 85%; top: 15%; animation-duration: 20s; animation-delay: -2s; }
  .p5 { width: 4px; height: 4px; left: 25%; top: 75%; animation-duration: 25s; animation-delay: -6s; }
  .p6 { width: 3px; height: 3px; left: 60%; top: 85%; animation-duration: 17s; animation-delay: -10s; }

  @keyframes float {
    0% { transform: translateY(0px) translateX(0px); opacity: 0; }
    10% { opacity: 0.8; }
    50% { transform: translateY(-120px) translateX(30px); opacity: 0.4; }
    90% { opacity: 0.6; }
    100% { transform: translateY(-240px) translateX(-20px); opacity: 0; }
  }

  /* ===== Card ===== */
  .login-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 36px;
    padding: 52px 44px;
    width: 420px;
    background: var(--glass);
    backdrop-filter: var(--glass-blur-heavy);
    -webkit-backdrop-filter: var(--glass-blur-heavy);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-xl);
    box-shadow:
      var(--glass-shadow-lg),
      var(--glass-glow),
      0 0 80px -20px rgba(168, 85, 247, 0.08);
    position: relative;
    z-index: 1;
    animation: cardEntrance 0.7s cubic-bezier(0.16, 1, 0.3, 1) both;
    overflow: hidden;
  }

  .card-shimmer {
    position: absolute;
    inset: 0;
    border-radius: inherit;
    overflow: hidden;
    pointer-events: none;
    z-index: 2;
  }

  .card-shimmer::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 60%;
    height: 100%;
    background: linear-gradient(
      105deg,
      transparent 0%,
      transparent 35%,
      rgba(168, 85, 247, 0.06) 45%,
      rgba(192, 132, 252, 0.1) 50%,
      rgba(168, 85, 247, 0.06) 55%,
      transparent 65%,
      transparent 100%
    );
    animation: shimmer 8s ease-in-out infinite;
  }

  @keyframes shimmer {
    0% { transform: translateX(-50%); }
    100% { transform: translateX(500%); }
  }

  @keyframes cardEntrance {
    from {
      opacity: 0;
      transform: translateY(24px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  /* ===== Logo Section ===== */
  .logo-section {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    animation: fadeIn 0.8s ease 0.2s both;
  }

  .logo-ring {
    position: relative;
    width: 80px;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo-ring::before {
    content: '';
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    background: conic-gradient(
      from 0deg,
      transparent,
      rgba(168, 85, 247, 0.3),
      transparent,
      rgba(139, 92, 246, 0.2),
      transparent
    );
    animation: logoSpin 8s linear infinite;
    filter: blur(3px);
  }

  .logo-ring::after {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: var(--bg-base);
    z-index: 0;
  }

  @keyframes logoSpin {
    to { transform: rotate(360deg); }
  }

  .login-logo {
    width: 56px;
    height: 56px;
    z-index: 1;
    filter: drop-shadow(0 0 20px rgba(168, 85, 247, 0.3));
    animation: logoPulse 3s ease-in-out infinite;
  }

  @keyframes logoPulse {
    0%, 100% { filter: drop-shadow(0 0 20px rgba(168, 85, 247, 0.3)); }
    50% { filter: drop-shadow(0 0 30px rgba(168, 85, 247, 0.5)); }
  }

  .title {
    font-size: 2rem;
    font-weight: 700;
    letter-spacing: 0.4em;
    color: var(--text-primary);
    margin: 0;
    text-shadow: 0 0 40px var(--accent-glow);
    background: linear-gradient(135deg, var(--text-primary) 0%, var(--accent) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .subtitle {
    color: var(--text-tertiary);
    font-size: 0.82rem;
    margin: 0;
    letter-spacing: 0.15em;
    text-transform: uppercase;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* ===== Loading State ===== */
  .loading-state {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text-tertiary);
    font-size: 0.85rem;
    animation: fadeIn 0.5s ease both;
  }

  .loading-spinner {
    width: 18px;
    height: 18px;
    border: 2px solid rgba(168, 85, 247, 0.2);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* ===== Form Section ===== */
  .form-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: 100%;
    animation: fadeIn 0.8s ease 0.35s both;
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
    transition: color 0.2s;
    z-index: 1;
  }

  .input-group:focus-within .input-icon {
    color: var(--accent);
  }

  input {
    width: 100%;
    padding: 14px 44px 14px 42px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    background: var(--select-bg);
    backdrop-filter: blur(8px);
    color: var(--text-primary);
    font-size: 0.9rem;
    letter-spacing: 0.02em;
    outline: none;
    transition: border-color 0.25s, box-shadow 0.25s, background 0.25s;
    box-sizing: border-box;
  }

  input:hover {
    border-color: var(--glass-border-hover);
    background: var(--select-bg-hover);
  }

  input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow), 0 0 20px -5px rgba(168, 85, 247, 0.15);
  }

  input::placeholder {
    color: var(--text-tertiary);
    letter-spacing: 0.03em;
  }

  .toggle-vis {
    position: absolute;
    right: 10px;
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 6px;
    display: flex;
    transition: color 0.2s, transform 0.15s;
    border-radius: 6px;
  }

  .toggle-vis:hover {
    color: var(--text-secondary);
    transform: scale(1.1);
  }

  .toggle-vis svg {
    width: 18px;
    height: 18px;
  }

  /* ===== Banners ===== */
  .warning-banner,
  .error-banner {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    backdrop-filter: blur(8px);
    font-size: 0.8rem;
    line-height: 1.4;
    animation: bannerSlide 0.3s ease both;
  }

  @keyframes bannerSlide {
    from { opacity: 0; transform: translateY(-6px); }
    to { opacity: 1; transform: translateY(0); }
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

  .warning-banner {
    background: rgba(58, 42, 10, 0.5);
    color: var(--warning);
    border: 1px solid rgba(251, 191, 36, 0.18);
  }

  .weak-hint {
    color: #a88520;
    font-size: 0.72rem;
    opacity: 0.85;
  }

  .error-banner {
    background: rgba(58, 26, 26, 0.5);
    color: var(--danger);
    border: 1px solid rgba(248, 113, 113, 0.18);
  }

  /* ===== Submit Button ===== */
  .submit-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    width: 100%;
    padding: 14px 24px;
    margin-top: 4px;
    border: none;
    border-radius: var(--radius-md);
    background: linear-gradient(135deg, #a855f7 0%, #8b5cf6 50%, #7c3aed 100%);
    color: #fff;
    font-size: 0.9rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    cursor: pointer;
    transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
    position: relative;
    overflow: hidden;
    box-shadow: 0 4px 20px -4px rgba(168, 85, 247, 0.35);
  }

  .submit-btn::before {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.15) 0%, transparent 50%);
    opacity: 0;
    transition: opacity 0.25s;
  }

  .submit-btn:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 6px 28px -4px rgba(168, 85, 247, 0.45);
  }

  .submit-btn:hover:not(:disabled)::before {
    opacity: 1;
  }

  .submit-btn:active:not(:disabled) {
    transform: translateY(0) scale(0.98);
    box-shadow: 0 2px 12px -2px rgba(168, 85, 247, 0.3);
  }

  .submit-btn:disabled {
    opacity: 0.4;
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
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  /* ===== Restore Link ===== */
  .restore-link {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 0.78rem;
    text-align: center;
    padding: 6px;
    transition: color 0.2s;
    text-decoration: none;
    letter-spacing: 0.02em;
    opacity: 0.8;
  }

  .restore-link:hover {
    color: var(--accent);
    opacity: 1;
  }

  .restore-link:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* ===== Version Tag ===== */
  .version-tag {
    position: fixed;
    bottom: 16px;
    right: 20px;
    font-size: 0.68rem;
    color: var(--text-tertiary);
    opacity: 0.35;
    letter-spacing: 0.05em;
    z-index: 0;
  }
</style>
