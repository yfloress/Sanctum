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

<script module lang="ts">
  /** One past entry, used to suggest descriptions already typed for a category. */
  export interface DescriptionHistoryEntry {
    category: string
    description: string
  }

  /** Longest datalist offered; beyond this the dropdown stops being a shortcut. */
  const MAX_SUGGESTIONS = 40

  // What the last saved entry used, so a run of similar transactions does not
  // mean picking the same account over and over. Deliberately module state and
  // not persisted: an account id written outside the encrypted vault would be
  // spending data sitting in the clear.
  const recalled = {
    accountId: '',
    category: { expense: '', income: '' },
  }
</script>

<script lang="ts">
  import { errorMessage } from '../../lib/errors'
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import { dialog } from '../../lib/actions/dialog'
  import * as financeApi from '../../lib/api/finance'
  import type { TransactionDto, CategoryDto } from '../../lib/types'

  interface Props {
    show: boolean
    editing: TransactionDto | null
    /** Copies a transaction's fields into a new entry, dated today. */
    prefill?: TransactionDto | null
    accounts: { id: string; name: string }[]
    categories: { expense: CategoryDto[]; income: CategoryDto[] }
    /** Past entries, newest first, used for the description suggestions. */
    descriptionHistory?: DescriptionHistoryEntry[]
    onsubmit: () => Promise<void>
    onclose: () => void
  }

  let {
    show = $bindable(false), editing, prefill = null, accounts, categories,
    descriptionHistory = [], onsubmit, onclose,
  }: Props = $props()

  /**
   * `offset` days back from today, in the YYYY-MM-DD the date input expects.
   * Built from the local date parts, not `toISOString`, which is UTC and so
   * lands on tomorrow for anyone west of Greenwich late in the evening.
   */
  function dayOffset(offset: number): string {
    const d = new Date()
    d.setDate(d.getDate() - offset)
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
  }

  const today = () => dayOffset(0)

  let txAccountId = $state('')
  let txAmount = $state('')
  let txCategory = $state('')
  let txDescription = $state('')
  let txDate = $state(today())
  let txIsExpense = $state(true)

  let txCategoryOptions = $derived<CategoryDto[]>(
    txIsExpense ? (categories.expense ?? []) : (categories.income ?? [])
  )

  // Narrowed to the chosen category so the dropdown offers what fits, and
  // deduplicated keeping the newest first because the caller sorts by date.
  let descriptionSuggestions = $derived(
    Array.from(
      new Set(
        descriptionHistory
          .filter(entry => !txCategory || entry.category === txCategory)
          .map(entry => entry.description)
          .filter(description => description.length > 0)
      )
    ).slice(0, MAX_SUGGESTIONS)
  )

  let canSubmit = $derived(!!txAmount && !!txAccountId)

  /** The remembered category, but only while it is still an option. */
  function recalledCategory(isExpense: boolean): string {
    const name = recalled.category[isExpense ? 'expense' : 'income']
    const options = (isExpense ? categories.expense : categories.income) ?? []
    return options.some(cat => cat.name === name) ? name : ''
  }

  function setType(isExpense: boolean) {
    if (txIsExpense === isExpense) return
    txIsExpense = isExpense
    // The category list is swapped out with the type, so a name from the old
    // list would leave the select showing nothing at all.
    const options = (isExpense ? categories.expense : categories.income) ?? []
    if (!options.some(cat => cat.name === txCategory)) {
      txCategory = recalledCategory(isExpense)
    }
  }

  $effect(() => {
    show
    editing
    prefill
    if (show) {
      const source = editing ?? prefill
      if (source) {
        txAccountId = source.account_id
        txAmount = source.amount_raw
        txCategory = source.category_raw
        txDescription = source.description
        txDate = editing ? source.date : today()
        txIsExpense = source.is_expense
      } else {
        // Falls back to the first account when the remembered one is gone.
        txAccountId = accounts.some(acc => acc.id === recalled.accountId)
          ? recalled.accountId
          : (accounts[0]?.id ?? '')
        txAmount = ''
        txCategory = recalledCategory(true)
        txDescription = ''
        txDate = today()
        txIsExpense = true
      }
    }
  })

  async function submitTransaction() {
    if (!canSubmit) return
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
      recalled.accountId = txAccountId
      recalled.category[txIsExpense ? 'expense' : 'income'] = txCategory
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
  <div class="modal-backdrop" role="presentation" onclick={close}></div>
  <div class="modal-wrapper">
    <div class="modal" use:dialog={{ onclose: close, onsubmit: submitTransaction }}>
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
          <button class="toggle-btn" class:active={txIsExpense} onclick={() => setType(true)}>{i18n.t('finances-expense', 'Expense')}</button>
          <button class="toggle-btn" class:active={!txIsExpense} onclick={() => setType(false)}>{i18n.t('finances-income', 'Income')}</button>
        </div>
      </label>
      <label>
        {i18n.t('finances-category', 'Category')}
        <select bind:value={txCategory}>
          <option value="">{i18n.t('finances-select', 'Select...')}</option>
          {#each txCategoryOptions as cat}
            <option value={cat.name}>{cat.label}</option>
          {/each}
        </select>
      </label>
      <label>
        {i18n.t('finances-description', 'Description')}
        <input
          type="text"
          list="tx-description-history"
          bind:value={txDescription}
          placeholder={i18n.t('finances-description', 'Description')}
        />
        <!-- Suggestions only: any other text is still accepted. -->
        <datalist id="tx-description-history">
          {#each descriptionSuggestions as suggestion}
            <option value={suggestion}></option>
          {/each}
        </datalist>
      </label>
      <label>
        {i18n.t('finances-date', 'Date')}
        <div class="date-row">
          <input type="date" bind:value={txDate} />
          <button class="date-shortcut" class:active={txDate === dayOffset(0)} onclick={() => txDate = dayOffset(0)}>
            {i18n.t('finances-date-today', 'Today')}
          </button>
          <button class="date-shortcut" class:active={txDate === dayOffset(1)} onclick={() => txDate = dayOffset(1)}>
            {i18n.t('finances-date-yesterday', 'Yesterday')}
          </button>
        </div>
      </label>
    </div>
    <div class="modal-actions">
      <button class="secondary-btn" onclick={close}>{i18n.t('finances-cancel', 'Cancel')}</button>
      <button class="primary-btn" onclick={submitTransaction} disabled={!canSubmit}>
        {editing ? i18n.t('finances-update', 'Update') : i18n.t('finances-add-btn', 'Add')}
      </button>
    </div>
    </div>
  </div>
{/if}

<style>
</style>
