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
  import { mask } from '../../lib/currency'
  import * as financeApi from '../../lib/api/finance'
  import type { AccountDto, CategoriesResponse, RecurringDto } from '../../lib/types'
  import ConfirmDialog from '../ConfirmDialog.svelte'

  interface Props {
    accounts: AccountDto[]
    categories: CategoriesResponse | null
    /** Called after any change, so the ledger and balances refresh. */
    onchange: () => Promise<void>
  }

  let { accounts, categories, onchange }: Props = $props()

  let rules = $state<RecurringDto[]>([])
  let showForm = $state(false)
  let saving = $state(false)
  let pendingDelete = $state<RecurringDto | null>(null)

  let accountId = $state('')
  let amount = $state('')
  let category = $state('')
  let description = $state('')
  let frequency = $state<'weekly' | 'monthly' | 'yearly'>('monthly')
  let firstDate = $state(new Date().toISOString().slice(0, 10))
  let isExpense = $state(true)

  let categoryOptions = $derived(
    isExpense ? (categories?.expense ?? []) : (categories?.income ?? [])
  )

  async function load() {
    try {
      rules = await financeApi.fetchRecurring()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  $effect(() => {
    load()
  })

  function resetForm() {
    showForm = false
    accountId = accounts[0]?.id ?? ''
    amount = ''
    category = ''
    description = ''
    frequency = 'monthly'
    firstDate = new Date().toISOString().slice(0, 10)
    isExpense = true
  }

  async function submit() {
    saving = true
    try {
      await financeApi.addRecurring({
        account_id: accountId,
        amount,
        category,
        description,
        frequency,
        first_date: firstDate,
        is_expense: isExpense,
      })
      resetForm()
      await load()
      // A rule dated today or earlier owes its first occurrence right away.
      await financeApi.applyDueRecurring()
      await onchange()
      app.showToast(i18n.t('finances-recurring-added', 'Recurring entry saved'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      saving = false
    }
  }

  async function toggle(rule: RecurringDto) {
    try {
      await financeApi.setRecurringActive(rule.id, !rule.is_active)
      await load()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function remove(rule: RecurringDto) {
    try {
      await financeApi.deleteRecurring(rule.id)
      await load()
      app.showToast(i18n.t('finances-recurring-deleted', 'Recurring entry deleted'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  function frequencyLabel(value: string): string {
    if (value === 'weekly') return i18n.t('finances-recurring-weekly', 'Weekly')
    if (value === 'yearly') return i18n.t('finances-recurring-yearly', 'Yearly')
    return i18n.t('finances-recurring-monthly', 'Monthly')
  }
</script>

<div class="settings-card">
  <div class="recurring-head">
    <div>
      <h3 class="settings-card-title">{i18n.t('finances-recurring', 'Recurring Entries')}</h3>
      <p class="recurring-note">
        {i18n.t('finances-recurring-desc', 'Created automatically on their date. Opening the app after a while fills in everything it owes.')}
      </p>
    </div>
    {#if !showForm}
      <button class="glass-btn" onclick={() => { resetForm(); showForm = true }} disabled={accounts.length === 0}>
        {i18n.t('finances-recurring-new', 'New')}
      </button>
    {/if}
  </div>

  {#if accounts.length === 0}
    <p class="empty">{i18n.t('finances-no-accounts-create', 'No accounts yet. Create your first account.')}</p>
  {/if}

  {#if showForm}
    <div class="recurring-form">
      <div class="toggle-row">
        <button class="toggle-btn" class:active={isExpense} onclick={() => (isExpense = true)}>
          {i18n.t('finances-expense', 'Expense')}
        </button>
        <button class="toggle-btn" class:active={!isExpense} onclick={() => (isExpense = false)}>
          {i18n.t('finances-income', 'Income')}
        </button>
      </div>
      <select bind:value={accountId} aria-label={i18n.t('finances-account', 'Account')}>
        {#each accounts as acc}
          <option value={acc.id}>{acc.name}</option>
        {/each}
      </select>
      <input type="text" inputmode="decimal" placeholder="0.00" bind:value={amount} />
      <select bind:value={category} aria-label={i18n.t('finances-category', 'Category')}>
        <option value="">{i18n.t('finances-select', 'Select...')}</option>
        {#each categoryOptions as cat}
          <option value={cat.name}>{cat.label}</option>
        {/each}
      </select>
      <input type="text" placeholder={i18n.t('finances-description', 'Description')} bind:value={description} />
      <select bind:value={frequency} aria-label={i18n.t('finances-recurring-frequency', 'Frequency')}>
        <option value="weekly">{i18n.t('finances-recurring-weekly', 'Weekly')}</option>
        <option value="monthly">{i18n.t('finances-recurring-monthly', 'Monthly')}</option>
        <option value="yearly">{i18n.t('finances-recurring-yearly', 'Yearly')}</option>
      </select>
      <label class="recurring-date">
        <span>{i18n.t('finances-recurring-first', 'First occurrence')}</span>
        <input type="date" bind:value={firstDate} />
      </label>
      <div class="recurring-actions">
        <button class="secondary-btn" onclick={resetForm} disabled={saving}>
          {i18n.t('finances-cancel', 'Cancel')}
        </button>
        <button class="primary-btn" onclick={submit} disabled={saving || !accountId || !amount || !category}>
          {i18n.t('finances-create', 'Create')}
        </button>
      </div>
    </div>
  {/if}

  {#if rules.length > 0}
    <div class="recurring-list">
      {#each rules as rule (rule.id)}
        <div class="recurring-row" class:paused={!rule.is_active}>
          <span class="tx-type-dot" class:expense={rule.is_expense}></span>
          <div class="recurring-main">
            <span class="recurring-desc">{rule.description || rule.category_label}</span>
            <div class="recurring-meta">
              <span>{frequencyLabel(rule.frequency)}</span>
              <span>{rule.account_name}</span>
              <span>
                {rule.is_active
                  ? `${i18n.t('finances-recurring-next', 'Next')}: ${rule.next_date}`
                  : i18n.t('finances-recurring-paused', 'Paused')}
              </span>
            </div>
          </div>
          <span class="recurring-amount" class:expense={rule.is_expense}>{mask(rule.amount)}</span>
          <button class="row-btn" onclick={() => toggle(rule)} title={rule.is_active ? i18n.t('finances-recurring-pause', 'Pause') : i18n.t('finances-recurring-resume', 'Resume')} aria-label={rule.is_active ? i18n.t('finances-recurring-pause', 'Pause') : i18n.t('finances-recurring-resume', 'Resume')}>
            {#if rule.is_active}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M10 4v16M14 4v16"/></svg>
            {:else}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6 4l12 8-12 8V4z"/></svg>
            {/if}
          </button>
          <button class="delete-btn" onclick={() => (pendingDelete = rule)} aria-label={i18n.t('confirm-delete-button', 'Delete')}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<ConfirmDialog
  show={pendingDelete !== null}
  message={i18n.t('finances-recurring-delete-confirm', 'Delete this recurring entry? Transactions it already created are kept.')}
  danger
  onconfirm={async () => {
    if (pendingDelete) await remove(pendingDelete)
    pendingDelete = null
  }}
  onclose={() => (pendingDelete = null)}
/>

<style>
  .recurring-head {
    display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;
  }
  .recurring-note {
    margin: 2px 0 0; font-size: 0.78rem; color: var(--text-tertiary); max-width: 46ch; line-height: 1.45;
  }
  .recurring-form {
    display: flex; flex-direction: column; gap: 8px; margin: 12px 0;
    padding: 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
  }
  .recurring-form input, .recurring-form select {
    padding: 8px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.85rem;
  }
  .recurring-form input:focus, .recurring-form select:focus {
    border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .recurring-date { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .recurring-date span { font-size: 0.8rem; color: var(--text-secondary); }
  .recurring-actions { display: flex; justify-content: flex-end; gap: 8px; }

  .recurring-list { display: flex; flex-direction: column; margin-top: 8px; }
  .recurring-row {
    display: flex; align-items: center; gap: 10px; padding: 9px 0;
    border-bottom: 1px solid var(--glass-border);
  }
  .recurring-row:last-child { border-bottom: none; }
  .recurring-row.paused { opacity: 0.55; }
  .recurring-main { flex: 1; min-width: 0; }
  .recurring-desc {
    display: block; font-size: 0.88rem; color: var(--text-primary);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .recurring-meta {
    display: flex; gap: 8px; flex-wrap: wrap; font-size: 0.74rem; color: var(--text-tertiary);
  }
  .recurring-amount { font-size: 0.88rem; color: var(--success); font-variant-numeric: tabular-nums; }
  .recurring-amount.expense { color: var(--danger); }
</style>
