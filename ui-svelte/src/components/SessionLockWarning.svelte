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
  import { i18n } from '../lib/stores/i18n.svelte'
  import { session, extendSession, lockNow } from '../lib/stores/session.svelte'
</script>

{#if session.warningSecs !== null}
  <div class="session-warning" role="alert" aria-live="assertive">
    <div class="sw-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
    </div>
    <div class="sw-text">
      <strong>{i18n.t('session-warning-title', 'Vault about to lock')}</strong>
      <span>
        {i18n.tArgs(
          'session-warning-body',
          { seconds: session.warningSecs },
          'Locking in {$seconds}s due to inactivity.'
        )}
      </span>
    </div>
    <div class="sw-actions">
      <button class="sw-stay" onclick={extendSession}>
        {i18n.t('session-warning-stay', 'Stay unlocked')}
      </button>
      <button class="sw-lock" onclick={lockNow}>
        {i18n.t('session-warning-lock-now', 'Lock now')}
      </button>
    </div>
  </div>
{/if}

<style>
  .session-warning {
    position: fixed;
    top: 24px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 14px;
    max-width: calc(100vw - 32px);
    padding: 12px 16px;
    border-radius: var(--radius-md);
    background: rgba(58, 46, 20, 0.8);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    color: var(--warning);
    border: 1px solid rgba(251, 191, 36, 0.28);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35), 0 0 0 1px rgba(251, 191, 36, 0.06) inset;
    font-size: 0.875rem;
    /* Above modals (100) and toasts (1000). */
    z-index: 1100;
    animation: dropIn 0.25s ease;
  }

  :global(.light-mode) .session-warning {
    background: rgba(255, 251, 235, 0.92);
    color: #92400e;
    border-color: rgba(180, 83, 9, 0.28);
  }

  .sw-icon {
    display: flex;
    opacity: 0.9;
  }

  .sw-icon svg {
    width: 20px;
    height: 20px;
  }

  .sw-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    line-height: 1.35;
  }

  .sw-text strong {
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  .sw-text span {
    opacity: 0.85;
    font-variant-numeric: tabular-nums;
  }

  .sw-actions {
    display: flex;
    gap: 8px;
  }

  .sw-actions button {
    cursor: pointer;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font: inherit;
    font-weight: 600;
    letter-spacing: 0.02em;
    transition: background 0.15s, border-color 0.15s;
  }

  .sw-stay {
    background: rgba(251, 191, 36, 0.16);
    border: 1px solid rgba(251, 191, 36, 0.4);
    color: inherit;
  }

  .sw-stay:hover {
    background: rgba(251, 191, 36, 0.26);
    border-color: rgba(251, 191, 36, 0.6);
  }

  .sw-lock {
    background: none;
    border: 1px solid transparent;
    color: inherit;
    opacity: 0.75;
  }

  .sw-lock:hover {
    opacity: 1;
    border-color: rgba(251, 191, 36, 0.3);
  }

  @keyframes dropIn {
    from {
      transform: translate(-50%, -16px);
      opacity: 0;
    }
    to {
      transform: translate(-50%, 0);
      opacity: 1;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .session-warning {
      animation: none;
    }
  }

  @media (max-width: 640px) {
    .session-warning {
      flex-wrap: wrap;
      justify-content: flex-end;
    }
  }
</style>
