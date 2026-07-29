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
  import { i18n } from '../../lib/stores/i18n.svelte'
  import { mask } from '../../lib/currency'
  import type { CryptoTransactionDto } from '../../lib/types'

  interface HoldingView {
    symbol: string
    name: string
    price: string
    amount: string
    value: string
    price_change_24h: string
    price_change_24h_negative: boolean
    allocation_pct: number
  }

  interface Props {
    show: boolean
    asset: HoldingView | null
    transactions: CryptoTransactionDto[]
    onedit: (txId: string) => void
    ondeleteTx: (id: string) => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), asset, transactions, onedit, ondeleteTx, onclose }: Props = $props()

  function close() {
    show = false
    onclose()
  }
</script>

{#if show && asset}
  <div class="overlay-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      <h3>{asset.symbol} - {asset.name}</h3>
      <button class="close-panel" aria-label="Close panel" onclick={close}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
    <div class="panel-meta">
      <span>{asset.price}</span>
      <span class="change" class:negative={asset.price_change_24h_negative}>{asset.price_change_24h}</span>
    </div>
    <div class="asset-stats">
      <div><span class="stat-lbl">{i18n.t('crypto-amount', 'Amount')}</span><span>{mask(asset.amount)}</span></div>
      <div><span class="stat-lbl">{i18n.t('crypto-value', 'Value')}</span><span>{mask(asset.value)}</span></div>
      <div><span class="stat-lbl">{i18n.t('crypto-allocation', 'Allocation')}</span><span>{asset.allocation_pct.toFixed(1)}%</span></div>
    </div>

    {#if transactions.length > 0}
      <h4>{i18n.t('crypto-transactions', 'Transactions')}</h4>
      {#each transactions as tx}
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
  </aside>
{/if}

<style>
  .change { font-size: 0.75rem; color: var(--success); }
  .change.negative { color: var(--danger); }

  .asset-stats { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }
  .asset-stats div { display: flex; justify-content: space-between; font-size: 0.85rem; color: var(--text-secondary); }
  .stat-lbl { font-size: 0.7rem; color: var(--text-tertiary); text-transform: uppercase; }
</style>
