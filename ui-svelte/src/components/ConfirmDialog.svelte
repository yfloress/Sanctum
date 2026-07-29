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

  interface Props {
    show: boolean
    title?: string
    message: string
    detail?: string
    confirmLabel?: string
    cancelLabel?: string
    danger?: boolean
    onconfirm: () => void | Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), title, message, detail, confirmLabel, cancelLabel, danger = false, onconfirm, onclose }: Props = $props()

  let busy = $state(false)

  async function confirm() {
    if (busy) return
    busy = true
    try {
      await onconfirm()
      show = false
    } finally {
      busy = false
    }
  }

  function close() {
    if (busy) return
    show = false
    onclose()
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <div class="modal-wrapper">
    <div class="modal confirm-modal">
      <h3>{title ?? i18n.t('confirm-delete-title', 'Confirm')}</h3>
      <p class="confirm-message">{message}</p>
      {#if detail}
        <p class="confirm-detail">{detail}</p>
      {/if}
      <div class="modal-actions">
        <button class="secondary-btn" onclick={close} disabled={busy}>
          {cancelLabel ?? i18n.t('finances-cancel', 'Cancel')}
        </button>
        {#if danger}
          <button class="danger-btn" onclick={confirm} disabled={busy}>
            {confirmLabel ?? i18n.t('confirm-delete-button', 'Delete')}
          </button>
        {:else}
          <button class="primary-btn" onclick={confirm} disabled={busy}>
            {confirmLabel ?? i18n.t('finances-update', 'Confirm')}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .confirm-modal { width: 380px; }
  .confirm-message {
    margin: 0 0 8px;
    color: var(--text-primary);
    font-size: 0.92rem;
    line-height: 1.5;
    position: relative;
    z-index: 10;
  }
  .confirm-detail {
    margin: 0 0 4px;
    color: var(--text-tertiary);
    font-size: 0.82rem;
    line-height: 1.5;
    position: relative;
    z-index: 10;
  }
</style>
