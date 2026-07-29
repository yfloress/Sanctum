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
  import * as financeApi from '../../lib/api/finance'
  import type { TransactionDto, AccountDto } from '../../lib/types'

  interface Props {
    show: boolean
    editing: TransactionDto | null
    accounts: AccountDto[]
    onsubmit: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), editing, accounts, onsubmit, onclose }: Props = $props()

  let fromId = $state('')
  let toId = $state('')
  let amount = $state('')
  let description = $state('')
  let date = $state(new Date().toISOString().slice(0, 10))

  $effect(() => {
    show
    editing
    if (!show) return
    if (editing) {
      fromId = editing.account_id
      toId = editing.transfer_account_id ?? ''
      amount = editing.amount_raw
      description = editing.description_raw
      date = editing.date
    } else {
      fromId = accounts[0]?.id ?? ''
      toId = accounts[1]?.id ?? ''
      amount = ''
      description = ''
      date = new Date().toISOString().slice(0, 10)
    }
  })

  async function submit() {
    const isEditing = !!editing
    try {
      if (isEditing) {
        await financeApi.updateTransfer({
          id: editing!.id,
          from_account_id: fromId,
          to_account_id: toId,
          amount,
          description,
          date,
        })
      } else {
        await financeApi.transferFunds({
          from_account_id: fromId,
          to_account_id: toId,
          amount,
          description,
          date,
        })
      }
      show = false
      await onsubmit()
      app.showToast(isEditing
        ? i18n.t('finances-tf-updated', 'Transfer updated')
        : i18n.t('finances-tf-completed', 'Transfer completed'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  function close() {
    show = false
    onclose()
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{editing ? i18n.t('finances-edit-transfer', 'Edit Transfer') : i18n.t('finances-transfer-funds', 'Transfer Funds')}</h3>
      <div class="form-grid">
        <label>
          {i18n.t('finances-from', 'From')}
          <select bind:value={fromId}>
            {#each accounts as acc}
              <option value={acc.id}>{acc.name}</option>
            {/each}
          </select>
        </label>
        <label>
          {i18n.t('finances-to', 'To')}
          <select bind:value={toId}>
            {#each accounts as acc}
              <option value={acc.id}>{acc.name}</option>
            {/each}
          </select>
        </label>
        <label>
          {i18n.t('finances-amount', 'Amount')}
          <input type="text" inputmode="decimal" bind:value={amount} placeholder="0.00" />
        </label>
        <label>
          {i18n.t('finances-description', 'Description')}
          <input type="text" bind:value={description} placeholder={i18n.t('finances-transfer-note', 'Transfer note')} />
        </label>
        <label>
          {i18n.t('finances-date', 'Date')}
          <input type="date" bind:value={date} />
        </label>
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('finances-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submit} disabled={!amount || fromId === toId}>
          {editing ? i18n.t('finances-update', 'Update') : i18n.t('finances-transfer-btn', 'Transfer')}
        </button>
      </div>
    </div>
  </div>
{/if}
