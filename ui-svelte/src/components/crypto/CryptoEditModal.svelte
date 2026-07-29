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
  import { dialog } from '../../lib/actions/dialog'
  import * as cryptoApi from '../../lib/api/crypto'
  import type { CryptoTransactionEditData } from '../../lib/types'

  const SUBTYPES_BY_TYPE: Record<string, string[]> = {
    trade: ['buy', 'sell', 'swap', 'other'],
    income: ['interest', 'reward', 'airdrop', 'gift', 'staking', 'mining', 'fork', 'payment', 'rebate', 'other'],
    expense: ['payment', 'gift', 'fee', 'lost', 'stolen', 'donation', 'sell', 'other'],
    transfer: ['deposit', 'withdrawal'],
  }

  interface Props {
    show: boolean
    txId: string
    onsubmit: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), txId, onsubmit, onclose }: Props = $props()

  let loading = $state(false)
  let editTxData = $state<CryptoTransactionEditData | null>(null)
  let editTxAmount = $state('')
  let editTxPrice = $state('')
  let editTxFee = $state('')
  let editTxFeeCoinId = $state('')
  let editTxFeeCoinAmount = $state('')
  let editTxDate = $state('')
  let editTxNotes = $state('')
  let editTxSubtype = $state('')
  let editTxOverrideProceeds = $state('')
  let editTxOverrideCostBasis = $state('')

  $effect(() => {
    if (show && txId) {
      loadTransaction()
    }
  })

  async function loadTransaction() {
    try {
      const data = await cryptoApi.getCryptoTransaction(txId)
      editTxData = data
      editTxAmount = data.amount
      editTxPrice = data.price
      editTxFee = data.fee
      editTxFeeCoinId = data.fee_coin_id ?? ''
      editTxFeeCoinAmount = data.fee_coin_amount ?? ''
      editTxDate = data.date
      editTxNotes = data.notes ?? ''
      editTxSubtype = data.subtype ?? ''
      editTxOverrideProceeds = data.override_proceeds ?? ''
      editTxOverrideCostBasis = data.override_cost_basis ?? ''
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function submitEditTransaction() {
    if (!editTxData) return
    loading = true
    try {
      await cryptoApi.updateCryptoTransaction({
        id: editTxData.id,
        amount: editTxAmount,
        price: editTxPrice,
        fee: editTxFee || '0',
        fee_coin_id: editTxFeeCoinId || undefined,
        fee_coin_amount: editTxFeeCoinAmount || undefined,
        date: editTxDate,
        notes: editTxNotes || undefined,
        subtype: editTxSubtype || undefined,
        override_proceeds: editTxOverrideProceeds || undefined,
        override_cost_basis: editTxOverrideCostBasis || undefined,
      })
      show = false
      editTxData = null
      await onsubmit()
      app.showToast(i18n.t('crypto-toast-tx-updated', 'Transaction updated'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      loading = false
    }
  }

  function close() {
    show = false
    onclose()
  }
</script>

{#if show && editTxData}
  <div class="modal-backdrop" role="presentation" onclick={close}></div>
  <div class="modal-wrapper">
    <div class="modal wide" use:dialog={{ onclose: close }}>
      <h3>{i18n.t('crypto-tx-edit-title', 'Edit Transaction')}</h3>
      <div class="edit-tx-meta">
        <span class="etm-wallet">{editTxData.wallet_name}</span>
        <span class="etm-coin">{editTxData.symbol}</span>
        <span class="etm-type" class:etm-negative={/^(sell|expense|fee)$/.test(editTxData.transaction_type)}>{editTxData.transaction_type}</span>
      </div>

      <div class="form-grid">
        <label>
          {i18n.t('crypto-tx-subtype', 'Subtype')}
          <select bind:value={editTxSubtype}>
            <option value="">--</option>
            {#each SUBTYPES_BY_TYPE[editTxData.transaction_type] ?? [] as st}
              <option value={st}>{st}</option>
            {/each}
          </select>
        </label>
        <label>
          {i18n.t('crypto-tx-amount', 'Amount')}
          <input type="text" bind:value={editTxAmount} placeholder="0.00" />
        </label>
        <label>
          {i18n.t('crypto-tx-price', 'Price (per coin)')}
          <input type="text" bind:value={editTxPrice} placeholder="0.00" />
        </label>
        <label>
          {i18n.t('crypto-tx-fee-label', 'Fee')}
          <input type="text" bind:value={editTxFee} placeholder="0" />
        </label>
        <label>
          {i18n.t('crypto-tx-fee-coin-id', 'Fee Coin (optional)')}
          <input type="text" bind:value={editTxFeeCoinId} placeholder="" />
        </label>
        <label>
          {i18n.t('crypto-tx-fee-coin-amount', 'Fee Coin Amount (optional)')}
          <input type="text" bind:value={editTxFeeCoinAmount} placeholder="0.00" />
        </label>
        <label>
          {i18n.t('crypto-tx-date', 'Date')}
          <input type="date" bind:value={editTxDate} />
        </label>
        <label>
          {i18n.t('crypto-tx-notes', 'Notes (optional)')}
          <input type="text" bind:value={editTxNotes} placeholder={i18n.t('crypto-tx-notes-placeholder', 'Notes...')} />
        </label>
        <label>
          {i18n.t('crypto-tx-override-proceeds', 'Override Proceeds (optional)')}
          <input type="text" bind:value={editTxOverrideProceeds} placeholder="0.00" />
        </label>
        <label>
          {i18n.t('crypto-tx-override-cost-basis', 'Override Cost Basis (optional)')}
          <input type="text" bind:value={editTxOverrideCostBasis} placeholder="0.00" />
        </label>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('crypto-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitEditTransaction} disabled={loading}>
          {loading ? i18n.t('crypto-saving', 'Saving...') : i18n.t('crypto-save', 'Save')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .edit-tx-meta {
    display: flex; gap: 10px; align-items: center; margin-bottom: 16px; flex-wrap: wrap;
  }
  .etm-wallet {
    padding: 4px 10px; background: var(--glass); border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); color: var(--text-secondary); font-size: 0.8rem;
  }
  .etm-coin {
    padding: 4px 10px; background: rgba(0, 0, 0, 0.2); border-radius: var(--radius-sm);
    color: var(--accent); font-size: 0.8rem; font-weight: 600;
  }
  .etm-type {
    padding: 4px 10px; border-radius: var(--radius-sm);
    font-size: 0.75rem; font-weight: 600; text-transform: uppercase;
    background: rgba(255, 255, 255, 0.05); color: var(--text-secondary);
  }
  .etm-negative { background: rgba(255, 69, 58, 0.12); color: var(--danger); }
</style>
