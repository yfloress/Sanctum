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
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
</script>

{#if app.toast}
  <div
    class="toast"
    class:error={app.toast.isError}
    role="alert"
  >
    <span>{app.toast.message}</span>
    {#if app.toast.action}
      <button class="undo-btn" onclick={() => app.runToastAction()}>
        {app.toast.action.label}
      </button>
    {/if}
    <button class="close-btn" onclick={() => app.dismissToast()} aria-label={i18n.t('action-close', 'Close')}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    bottom: 24px;
    right: 24px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: var(--radius-md);
    background: rgba(26, 58, 42, 0.75);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    color: var(--success);
    border: 1px solid rgba(74, 222, 128, 0.2);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(74, 222, 128, 0.05) inset;
    font-size: 0.875rem;
    z-index: 1000;
    animation: slideIn 0.25s ease;
  }

  .toast.error {
    background: rgba(58, 26, 26, 0.75);
    color: var(--danger);
    border-color: rgba(248, 113, 113, 0.2);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(248, 113, 113, 0.05) inset;
  }

  .undo-btn {
    background: rgba(74, 222, 128, 0.12);
    border: 1px solid rgba(74, 222, 128, 0.3);
    color: inherit;
    cursor: pointer;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font: inherit;
    font-weight: 600;
    letter-spacing: 0.02em;
    transition: background 0.15s, border-color 0.15s;
  }

  .undo-btn:hover {
    background: rgba(74, 222, 128, 0.2);
    border-color: rgba(74, 222, 128, 0.5);
  }

  .toast.error .undo-btn {
    background: rgba(248, 113, 113, 0.12);
    border-color: rgba(248, 113, 113, 0.3);
  }

  .toast.error .undo-btn:hover {
    background: rgba(248, 113, 113, 0.2);
    border-color: rgba(248, 113, 113, 0.5);
  }

  .close-btn {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 2px;
    display: flex;
    opacity: 0.7;
    transition: opacity 0.15s;
  }

  .close-btn:hover {
    opacity: 1;
  }

  .close-btn svg {
    width: 16px;
    height: 16px;
  }

  @keyframes slideIn {
    from {
      transform: translateY(16px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
</style>
