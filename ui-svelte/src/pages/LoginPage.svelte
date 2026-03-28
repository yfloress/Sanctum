<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import * as vaultApi from '../lib/api/vault'
  import * as settingsApi from '../lib/api/settings'

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
</script>

<div class="login-page">
  <div class="login-card">
    <div class="logo-section">
      <h1 class="title">SANCTUM</h1>
      <p class="subtitle">Privacy-first personal vault</p>
    </div>

    {#if vaultExists === null}
      <div class="loading-state">Loading...</div>
    {:else}
      <div class="form-section">
        <div class="input-group">
          <input
            type={showPassword ? 'text' : 'password'}
            bind:value={password}
            onkeydown={handleKeydown}
            placeholder="Master password"
            disabled={loading}
            autocomplete="off"
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
            <span>{weakWarning}</span>
            <span class="weak-hint">Press again to confirm with weak password</span>
          </div>
        {/if}

        {#if error}
          <div class="error-banner">{error}</div>
        {/if}

        <button
          class="submit-btn"
          onclick={handleSubmit}
          disabled={loading || !password.trim()}
        >
          {#if loading}
            Authenticating...
          {:else if vaultExists}
            Unlock
          {:else if confirmWeak}
            Confirm Create
          {:else}
            Create Vault
          {/if}
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .login-page {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: linear-gradient(135deg, #0a0a0a 0%, #111827 100%);
  }

  .login-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 32px;
    padding: 48px 40px;
    width: 380px;
  }

  .logo-section {
    text-align: center;
  }

  .title {
    font-size: 2.2rem;
    font-weight: 700;
    letter-spacing: 0.35em;
    color: #e0e0e0;
    margin: 0;
  }

  .subtitle {
    color: #666;
    font-size: 0.85rem;
    margin-top: 8px;
    letter-spacing: 0.1em;
  }

  .loading-state {
    color: #666;
    font-size: 0.9rem;
  }

  .form-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: 100%;
  }

  .input-group {
    position: relative;
    display: flex;
    align-items: center;
  }

  input {
    width: 100%;
    padding: 12px 44px 12px 16px;
    border: 1px solid #333;
    border-radius: 8px;
    background: #111;
    color: #e0e0e0;
    font-size: 0.95rem;
    outline: none;
    transition: border-color 0.15s;
    box-sizing: border-box;
  }

  input:focus {
    border-color: #4f9cf7;
  }

  input::placeholder {
    color: #555;
  }

  .toggle-vis {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    padding: 6px;
    display: flex;
  }

  .toggle-vis:hover {
    color: #999;
  }

  .toggle-vis svg {
    width: 20px;
    height: 20px;
  }

  .warning-banner {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 14px;
    border-radius: 6px;
    background: #3a2a0a;
    color: #fbbf24;
    border: 1px solid #5a3a0d;
    font-size: 0.825rem;
  }

  .weak-hint {
    color: #a88520;
    font-size: 0.75rem;
  }

  .error-banner {
    padding: 10px 14px;
    border-radius: 6px;
    background: #3a1a1a;
    color: #f87171;
    border: 1px solid #5a2d2d;
    font-size: 0.825rem;
  }

  .submit-btn {
    width: 100%;
    padding: 12px;
    border: none;
    border-radius: 8px;
    background: #4f9cf7;
    color: #fff;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .submit-btn:hover:not(:disabled) {
    background: #3b82f6;
  }

  .submit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
