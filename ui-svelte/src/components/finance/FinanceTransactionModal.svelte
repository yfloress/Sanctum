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
  import type { TransactionDto, CategoryDto } from '../../lib/types'

  interface Props {
    show: boolean
    editing: TransactionDto | null
    accounts: { id: string; name: string }[]
    categories: { expense: CategoryDto[]; income: CategoryDto[] }
    onsubmit: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), editing, accounts, categories, onsubmit, onclose }: Props = $props()

  let txAccountId = $state('')
  let txAmount = $state('')
  let txCategory = $state('')
  let txDescription = $state('')
  let txDate = $state(new Date().toISOString().slice(0, 10))
  let txIsExpense = $state(true)

  let txCategoryOptions = $derived<CategoryDto[]>(
    txIsExpense ? (categories.expense ?? []) : (categories.income ?? [])
  )

  $effect(() => {
    show
    editing
    if (show) {
      if (editing) {
        txAccountId = editing.account_id
        txAmount = editing.amount_raw
        txCategory = editing.category_raw
        txDescription = editing.description
        txDate = editing.date
        txIsExpense = editing.is_expense
      } else {
        txAccountId = accounts[0]?.id ?? ''
        txAmount = ''
        txCategory = ''
        txDescription = ''
        txDate = new Date().toISOString().slice(0, 10)
        txIsExpense = true
      }
    }
  })

  async function submitTransaction() {
    try {
      if (editing) {
        await financeApi.updateTransaction(
          editing.id, txAccountId, txAmount, txCategory, txDescription, txDate, txIsExpense
        )
      } else {
        await financeApi.addTransaction(
          txAccountId, txAmount, txCategory, txDescription, txDate, txIsExpense
        )
      }
      show = false
      await onsubmit()
      app.showToast(editing ? i18n.t('finances-tx-updated', 'Transaction updated') : i18n.t('finances-tx-added', 'Transaction added'))
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
    <h3>{editing ? i18n.t('finances-edit-transaction', 'Edit Transaction') : i18n.t('finances-add-transaction', 'Add Transaction')}</h3>
    <div class="form-grid">
      <label>
        {i18n.t('finances-account', 'Account')}
        <select bind:value={txAccountId}>
          {#each accounts as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
      </label>
      <label>
        {i18n.t('finances-amount', 'Amount')}
        <input type="text" inputmode="decimal" bind:value={txAmount} placeholder="0.00" />
      </label>
      <label>
        {i18n.t('finances-type', 'Type')}
        <div class="toggle-row">
          <button class="toggle-btn" class:active={txIsExpense} onclick={() => txIsExpense = true}>{i18n.t('finances-expense', 'Expense')}</button>
          <button class="toggle-btn" class:active={!txIsExpense} onclick={() => txIsExpense = false}>{i18n.t('finances-income', 'Income')}</button>
        </div>
      </label>
      <label>
        {i18n.t('finances-category', 'Category')}
        <select bind:value={txCategory}>
          <option value="">{i18n.t('finances-select', 'Select...')}</option>
          {#each txCategoryOptions as cat}
            <option value={cat.name}>{cat.name}</option>
          {/each}
        </select>
      </label>
      <label>
        {i18n.t('finances-description', 'Description')}
        <input type="text" bind:value={txDescription} placeholder={i18n.t('finances-description', 'Description')} />
      </label>
      <label>
        {i18n.t('finances-date', 'Date')}
        <input type="date" bind:value={txDate} />
      </label>
    </div>
    <div class="modal-actions">
      <button class="secondary-btn" onclick={close}>{i18n.t('finances-cancel', 'Cancel')}</button>
      <button class="primary-btn" onclick={submitTransaction} disabled={!txAmount || !txAccountId}>
        {editing ? i18n.t('finances-update', 'Update') : i18n.t('finances-add-btn', 'Add')}
      </button>
    </div>
    </div>
  </div>
{/if}

<style>
</style>
