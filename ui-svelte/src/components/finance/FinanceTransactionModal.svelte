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

  // Every past description, not just those of the chosen category: the
  // description is what now decides the category, so narrowing it by the
  // category would hide the entry that was going to set it.
  // Deduplicated keeping the newest first, because the caller sorts by date.
  let descriptionSuggestions = $derived(
    Array.from(
      new Set(
        descriptionHistory
          .map(entry => entry.description)
          .filter(description => description.length > 0)
      )
    ).slice(0, MAX_SUGGESTIONS)
  )

  /**
   * The category each description was last filed under.
   *
   * History arrives newest first, so the first sighting of a description is the
   * most recent decision the user made about it.
   */
  let categoryByDescription = $derived.by(() => {
    const map = new Map<string, string>()
    for (const entry of descriptionHistory) {
      const key = entry.description.trim().toLowerCase()
      if (key && !map.has(key)) map.set(key, entry.category)
    }
    return map
  })

  /** Set once the user picks a category by hand; their choice then wins. */
  let categoryTouched = $state(false)

  /**
   * Fills the category from what this description was filed under last time.
   *
   * Silent when the user has already chosen: deciding once per thing bought is
   * the point, overruling a deliberate choice is not.
   */
  function recallCategoryFromDescription() {
    if (categoryTouched) return
    const remembered = categoryByDescription.get(txDescription.trim().toLowerCase())
    if (!remembered) return
    // Resolved against the current list, or the select would go blank on a name
    // that belongs to the other type.
    const match = txCategoryOptions.find(
      cat => cat.name.toUpperCase() === remembered.toUpperCase()
    )
    if (match) txCategory = match.name
  }

  let canSubmit = $derived(!!txAmount && !!txAccountId)

  // ── Tags ──────────────────────────────────────────────────────────────────

  let txTags = $state<string[]>([])
  let tagDraft = $state('')
  let tagCatalog = $state<string[]>([])

  function addTag(raw: string) {
    const tag = raw.trim().toLowerCase()
    if (!tag || txTags.includes(tag)) {
      tagDraft = ''
      return
    }
    txTags = [...txTags, tag]
    tagDraft = ''
  }

  function removeTag(tag: string) {
    txTags = txTags.filter(t => t !== tag)
  }

  function onTagKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ',') {
      // Stopped as well as prevented: the dialog action treats Enter as submit,
      // and adding a tag must not also save the transaction.
      event.preventDefault()
      event.stopPropagation()
      addTag(tagDraft)
    } else if (event.key === 'Backspace' && !tagDraft && txTags.length > 0) {
      txTags = txTags.slice(0, -1)
    }
  }

  /** Tags already in use that are not on this transaction yet. */
  let tagSuggestions = $derived(tagCatalog.filter(tag => !txTags.includes(tag)))

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
      categoryTouched = false
      tagDraft = ''
      void financeApi.fetchTags().then(tags => { tagCatalog = tags }).catch(() => { tagCatalog = [] })
      const source = editing ?? prefill
      if (source) {
        txAccountId = source.account_id
        txAmount = source.amount_raw
        txCategory = source.category_raw
        txDescription = source.description
        txDate = editing ? source.date : today()
        txIsExpense = source.is_expense
        txTags = [...source.tags]
        // An entry that arrives with a category was already filed; the
        // description must not quietly refile it.
        categoryTouched = true
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
        txTags = []
      }
    }
  })

  async function submitTransaction() {
    if (!canSubmit) return
    try {
      // A draft left in the box is a tag the user meant to add and did not
      // press Enter on; dropping it silently would lose their work.
      const tags = tagDraft.trim() ? [...txTags, tagDraft.trim().toLowerCase()] : txTags
      if (editing) {
        await financeApi.updateTransaction(
          editing.id, txAccountId, txAmount, txCategory, txDescription, txDate, txIsExpense, tags
        )
      } else {
        await financeApi.addTransaction(
          txAccountId, txAmount, txCategory, txDescription, txDate, txIsExpense, tags
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
        {i18n.t('finances-description', 'Description')}
        <input
          type="text"
          list="tx-description-history"
          bind:value={txDescription}
          oninput={recallCategoryFromDescription}
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
        {i18n.t('finances-category', 'Category')}
        <select bind:value={txCategory} onchange={() => categoryTouched = true}>
          <option value="">{i18n.t('finances-select', 'Select...')}</option>
          {#each txCategoryOptions as cat}
            <option value={cat.name}>{cat.label}</option>
          {/each}
        </select>
      </label>
      <!-- A div and not a label: a label forwards any click inside it to its
           control, which fights with the remove buttons sitting in the chips. -->
      <div class="field">
        <span class="field-label">{i18n.t('finances-tags', 'Tags')}</span>
        <div class="tag-box">
          {#each txTags as tag (tag)}
            <span class="tag-chip">
              <span class="tag-text">{tag}</span>
              <button type="button" class="tag-remove" onclick={() => removeTag(tag)}
                aria-label={i18n.tArgs('finances-tag-remove', { tag }, `Remove ${tag}`)}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4">
                  <path d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </span>
          {/each}
          <input
            type="text"
            class="tag-input"
            list="tx-tag-catalog"
            aria-label={i18n.t('finances-tags', 'Tags')}
            bind:value={tagDraft}
            onkeydown={onTagKeydown}
            onblur={() => addTag(tagDraft)}
            placeholder={txTags.length === 0 ? i18n.t('finances-tags-placeholder', 'snack, work...') : ''}
          />
        </div>
        <datalist id="tx-tag-catalog">
          {#each tagSuggestions as suggestion}
            <option value={suggestion}></option>
          {/each}
        </datalist>
      </div>
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
  /* Looks like one field even though it is a row of chips plus an input, so
     the tags read as the value of "Tags" rather than as a widget of their own. */
  .tag-box {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 5px;
    padding: 6px 8px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass-active);
  }
  .tag-box:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-glow); }

  /* The label is a plain span here, so it needs the shape `.form-grid label`
     gives the other fields. */
  .field { display: flex; flex-direction: column; gap: 4px; }
  .field-label { font-size: 0.8rem; color: var(--text-secondary); }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    padding: 2px 2px 2px 9px;
    border-radius: 20px;
    background: var(--accent-glow);
    color: var(--text-primary);
    font-size: 0.72rem;
  }
  /* Non-interactive on purpose: only the X removes, so brushing the word does
     nothing. */
  .tag-text { pointer-events: none; }

  .tag-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    /* A square just big enough to hit deliberately and small enough not to be
       hit by accident. It stays inside the chip, so nothing outside removes. */
    width: 16px;
    height: 16px;
    margin-left: 3px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: none;
    color: var(--text-tertiary);
    cursor: pointer;
  }
  .tag-remove:hover { background: var(--glass-active); color: var(--text-primary); }
  .tag-remove svg { width: 9px; height: 9px; }

  /* Unstyled on purpose: the box around it already carries the border. */
  .tag-input {
    flex: 1;
    min-width: 90px;
    padding: 2px 0;
    border: none;
    background: none;
    color: var(--text-primary);
    font-family: inherit;
    font-size: 0.85rem;
  }
  .tag-input:focus { outline: none; }
</style>
