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
  import CryptoMovementRow from './CryptoMovementRow.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import { mask } from '../../lib/currency'
  import { dialog } from '../../lib/actions/dialog'
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
  <div class="overlay-backdrop" role="presentation" onclick={close}></div>
  <aside class="detail-panel" use:dialog={{ onclose: close, autofocus: false }}>
    <div class="panel-header">
      <h3>{asset.symbol} - {asset.name}</h3>
      <button class="close-panel" aria-label={i18n.t('finances-close', 'Close')} onclick={close}>
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
        <CryptoMovementRow {tx} {onedit} {ondeleteTx} />
      {/each}
    {/if}
  </aside>
{/if}

<style>
  .detail-panel { container-type: inline-size; }
  .change { font-size: 0.75rem; color: var(--success); }
  .change.negative { color: var(--danger); }

  .asset-stats { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }
  .asset-stats div { display: flex; justify-content: space-between; font-size: 0.85rem; color: var(--text-secondary); }
  .stat-lbl { font-size: 0.78rem; color: var(--text-tertiary); font-weight: 500; }
</style>
