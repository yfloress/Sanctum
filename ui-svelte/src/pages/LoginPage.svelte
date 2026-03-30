<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
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
</script>

<div class="login-page">
  <div class="login-card">
    <div class="logo-section">
      <img src="/src/assets/logo/sanctum_logo.svg" alt="Sanctum" class="login-logo" />
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

        <div class="button-container">
          <button class="glass-btn" onclick={handleSubmit}>
            {loading ? 'Authenticating...' : vaultExists ? 'Unlock' : confirmWeak ? 'Confirm Create' : 'Create Vault'}
          </button>
        </div>
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
    background: radial-gradient(ellipse at 30% 20%, rgba(168, 85, 247, 0.06) 0%, transparent 50%),
                radial-gradient(ellipse at 70% 80%, rgba(74, 222, 128, 0.04) 0%, transparent 50%);
    animation: auroraShift 20s ease-in-out infinite alternate;
    pointer-events: none;
  }

  @keyframes auroraShift {
    0% { transform: translate(0, 0) rotate(0deg); }
    100% { transform: translate(2%, -2%) rotate(3deg); }
  }

  .button-container {
    display: flex;
    justify-content: center;
    padding-top: 16px;
    margin-top: 8px;
  }

  .login-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 32px;
    padding: 48px 40px;
    width: 400px;
    background: var(--glass);
    backdrop-filter: var(--glass-blur-heavy);
    -webkit-backdrop-filter: var(--glass-blur-heavy);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-xl);
    box-shadow: var(--glass-shadow-lg), var(--glass-glow);
    position: relative;
    z-index: 1;
  }

  .logo-section {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .login-logo {
    width: 64px;
    height: 64px;
  }

  .title {
    font-size: 2.2rem;
    font-weight: 700;
    letter-spacing: 0.35em;
    color: var(--text-primary);
    margin: 0;
    text-shadow: 0 0 30px var(--accent-glow);
  }

  .subtitle {
    color: var(--text-tertiary);
    font-size: 0.85rem;
    margin-top: 8px;
    letter-spacing: 0.1em;
  }

  .loading-state {
    color: var(--text-tertiary);
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
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.25);
    backdrop-filter: blur(8px);
    color: var(--text-primary);
    font-size: 0.95rem;
    outline: none;
    transition: border-color 0.2s, box-shadow 0.2s;
    box-sizing: border-box;
  }

  input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  input::placeholder {
    color: var(--text-tertiary);
  }

  .toggle-vis {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 6px;
    display: flex;
    transition: color 0.15s;
  }

  .toggle-vis:hover {
    color: var(--text-secondary);
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
    border-radius: var(--radius-sm);
    background: rgba(58, 42, 10, 0.6);
    backdrop-filter: blur(8px);
    color: var(--warning);
    border: 1px solid rgba(251, 191, 36, 0.2);
    font-size: 0.825rem;
  }

  .weak-hint {
    color: #a88520;
    font-size: 0.75rem;
  }

  .error-banner {
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    background: rgba(58, 26, 26, 0.6);
    backdrop-filter: blur(8px);
    color: var(--danger);
    border: 1px solid rgba(248, 113, 113, 0.2);
    font-size: 0.825rem;
  }

</style>
