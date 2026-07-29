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
  import { errorMessage } from '../../lib/errors'
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import * as cryptoApi from '../../lib/api/crypto'
  import { mask } from '../../lib/currency'
  import type { WalletDetailResponse } from '../../lib/types'

  const WALLET_ICONS: { value: string; src: string; generic: boolean }[] = [
    ...['binance', 'bisq', 'bitmart', 'buda', 'bybit', 'kraken', 'mexc', 'retoswap', 'uniswap']
      .map(n => ({ value: `${n}.svg`, src: `/assets/exchange-icons/${n}.svg`, generic: false })),
    ...['landmark', 'wallet', 'shield', 'shield-check', 'link', 'lock']
      .map(n => ({ value: `/assets/icons/${n}.svg`, src: `/assets/icons/${n}.svg`, generic: true })),
  ]

  function getDefaultWalletIconPath(category: string): string {
    const iconMap: { [key: string]: string } = {
      'exchange': 'landmark',
      'hardware': 'shield',
      'software': 'wallet',
    }
    const icon = iconMap[category.toLowerCase()] || 'wallet'
    return `/assets/icons/${icon}.svg`
  }

  interface Props {
    show: boolean
    wallet: WalletDetailResponse | null
    ondelete: (id: string) => Promise<void>
    onedit: (txId: string) => void
    ondeleteTx: (id: string) => Promise<void>
    onrefresh: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), wallet, ondelete, onedit, ondeleteTx, onrefresh, onclose }: Props = $props()

  let showEditWalletName = $state(false)
  let editingWalletName = $state('')
  let showWalletIconPicker = $state(false)

  function getWalletDisplayIcon(w: { category: string; icon_path: string | null }): string {
    if (w.icon_path) {
      if (w.icon_path.startsWith('/') || w.icon_path.startsWith('http')) return w.icon_path
      return `/assets/exchange-icons/${w.icon_path}`
    }
    return getDefaultWalletIconPath(w.category)
  }

  function isGenericWalletIcon(iconPath: string | null): boolean {
    if (!iconPath) return true
    return iconPath.startsWith('/assets/icons/')
  }

  function startEditWalletName() {
    if (!wallet) return
    editingWalletName = wallet.name
    showEditWalletName = true
  }

  async function submitWalletName() {
    if (!wallet || !editingWalletName.trim()) return
    try {
      await cryptoApi.updateWalletName(wallet.id, editingWalletName)
      showEditWalletName = false
      await onrefresh()
      app.showToast(i18n.t('crypto-toast-wallet-renamed', 'Wallet renamed'))
    } catch (e) { app.showToast(errorMessage(e), true) }
  }

  async function changeWalletIcon(icon: string) {
    if (!wallet) return
    try {
      await cryptoApi.updateWalletIcon(wallet.id, icon || null)
      showWalletIconPicker = false
      await onrefresh()
    } catch (e) { app.showToast(errorMessage(e), true) }
  }

  function close() {
    show = false
    onclose()
  }
</script>

{#if show && wallet}
  <div class="overlay-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      {#if showEditWalletName}
        <div class="inline-edit">
          <input type="text" bind:value={editingWalletName} class="edit-name-input" />
          <button class="icon-btn-sm" onclick={submitWalletName}>{i18n.t('crypto-save', 'Save')}</button>
          <button class="icon-btn-sm" onclick={() => showEditWalletName = false}>{i18n.t('crypto-cancel', 'Cancel')}</button>
        </div>
      {:else}
        <button class="clickable-name" onclick={startEditWalletName} title={i18n.t('crypto-click-rename', 'Click to rename')}>{wallet.name}</button>
      {/if}
      <button class="close-panel" aria-label="Close panel" onclick={close}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
    <div class="panel-meta">
      <span>{wallet.category}</span>
      <span class="panel-total">{mask(wallet.total_value)}</span>
    </div>
    <div class="panel-icon-row">
      <img src={getWalletDisplayIcon(wallet)} alt="" class="panel-wallet-icon" class:themed-icon={isGenericWalletIcon(wallet.icon_path)} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
      <button class="change-icon-btn" onclick={() => showWalletIconPicker = !showWalletIconPicker}>
        {showWalletIconPicker ? i18n.t('crypto-close', 'Close') : i18n.t('crypto-change-icon', 'Change Icon')}
      </button>
    </div>
    {#if showWalletIconPicker}
      <div class="icon-picker">
        {#each WALLET_ICONS as icon}
          <button class="icon-option" onclick={() => changeWalletIcon(icon.value)} title={icon.value}>
            <img src={icon.src} alt={icon.value} class:themed-icon={icon.generic} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
          </button>
        {/each}
        <button class="icon-option icon-reset" onclick={() => changeWalletIcon('')} title="Default">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 12l9-9 9 9M5 10v9a1 1 0 001 1h4v-5h4v5h4a1 1 0 001-1v-9"/></svg>
        </button>
      </div>
    {/if}

    {#if wallet.holdings.length > 0}
      <h4>{i18n.t('crypto-holdings', 'Holdings')}</h4>
      {#each wallet.holdings as h}
        <div class="holding-row">
          <span class="h-symbol">{h.symbol}</span>
          <span class="h-amount">{mask(h.amount)}</span>
          <span class="h-value">{mask(h.value)}</span>
        </div>
      {/each}
    {/if}

    {#if wallet.transactions.length > 0}
      <h4>{i18n.t('crypto-transactions', 'Transactions')}</h4>
      {#each wallet.transactions.slice(0, 20) as tx}
        <div class="panel-tx">
          <span class="tx-date">{tx.date}</span>
          <span class="tx-type">{tx.transaction_type}</span>
          <span class="tx-amount">{mask(tx.amount)} {tx.symbol}</span>
          <div class="panel-tx-actions">
            <button class="icon-btn-mini" onclick={() => onedit(tx.id)} aria-label={i18n.t('crypto-edit', 'Edit')} title={i18n.t('crypto-edit', 'Edit')}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
            </button>
            <button class="delete-btn" onclick={() => ondeleteTx(tx.id)} aria-label={i18n.t('crypto-delete', 'Delete')} title={i18n.t('crypto-delete', 'Delete')}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
            </button>
          </div>
        </div>
      {/each}
    {/if}

    <div class="panel-actions">
      <button class="danger-btn" onclick={() => ondelete(wallet.id)}>{i18n.t('crypto-delete-wallet', 'Delete Wallet')}</button>
    </div>
  </aside>
{/if}

<style>
  .panel-icon-row { display: flex; align-items: center; gap: 10px; margin-bottom: 12px; }
  .panel-wallet-icon { width: 36px; height: 36px; border-radius: 6px; }

  :global(.light-mode) .themed-icon { filter: brightness(0); }

  .holding-row { display: grid; grid-template-columns: 60px 1fr auto; gap: 8px; padding: 6px 0; border-bottom: 1px solid var(--glass-border); font-size: 0.85rem; }
  .h-symbol { color: var(--text-secondary); font-weight: 500; }
  .h-amount { color: var(--text-secondary); }
  .h-value { color: var(--text-primary); text-align: right; }

  .inline-edit { display: flex; align-items: center; gap: 6px; flex: 1; }
  .edit-name-input {
    padding: 6px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.9rem; flex: 1;
  }
  .edit-name-input:focus { border-color: var(--accent); outline: none; }
  .icon-btn-sm {
    padding: 4px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.75rem;
    transition: all 0.15s;
  }
  .icon-btn-sm:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .clickable-name {
    cursor: pointer; margin: 0; color: var(--text-primary); background: none; border: none;
    font-size: 1rem; font-weight: 600; text-align: left; padding: 0;
  }
  .clickable-name:hover { color: var(--accent); }
</style>
