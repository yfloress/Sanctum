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
  import type { BudgetDto, CategoriesResponse } from '../../lib/types'

  interface Props {
    categories: CategoriesResponse | null
    /** Bumped by the parent when transactions change, to re-read progress. */
    revision?: number
  }

  let { categories, revision = 0 }: Props = $props()

  let budgets = $state<BudgetDto[]>([])
  let showForm = $state(false)
  let category = $state('')
  let amount = $state('')
  let saving = $state(false)

  /** Expense categories without a budget yet: one limit per category. */
  let available = $derived(
    (categories?.expense ?? []).filter(c => !budgets.some(b => b.category === c.name))
  )

  async function load() {
    try {
      budgets = await financeApi.fetchBudgets()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  $effect(() => {
    // Re-reads whenever the parent signals the ledger moved.
    void revision
    load()
  })

  async function submit() {
    saving = true
    try {
      await financeApi.setBudget(category, amount)
      showForm = false
      category = ''
      amount = ''
      await load()
      app.showToast(i18n.t('finances-budget-saved', 'Budget saved'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      saving = false
    }
  }

  async function remove(budget: BudgetDto) {
    try {
      await financeApi.deleteBudget(budget.category)
      await load()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }
</script>

<div class="settings-card">
  <div class="budget-head">
    <div>
      <h3 class="settings-card-title">{i18n.t('finances-budgets', 'Monthly Budgets')}</h3>
      <p class="budget-note">
        {i18n.t('finances-budgets-desc', 'A spending limit per category. Progress covers the current month and resets on the 1st.')}
      </p>
    </div>
    {#if !showForm && available.length > 0}
      <button class="glass-btn" onclick={() => { category = available[0]?.name ?? ''; showForm = true }}>
        {i18n.t('finances-budget-new', 'New')}
      </button>
    {/if}
  </div>

  {#if showForm}
    <div class="budget-form">
      <select bind:value={category} aria-label={i18n.t('finances-category', 'Category')}>
        {#each available as cat}
          <option value={cat.name}>{cat.label}</option>
        {/each}
      </select>
      <input type="text" inputmode="decimal" placeholder="0.00" bind:value={amount} />
      <div class="budget-actions">
        <button class="secondary-btn" onclick={() => (showForm = false)} disabled={saving}>
          {i18n.t('finances-cancel', 'Cancel')}
        </button>
        <button class="primary-btn" onclick={submit} disabled={saving || !category || !amount}>
          {i18n.t('finances-create', 'Create')}
        </button>
      </div>
    </div>
  {/if}

  {#if budgets.length === 0}
    <p class="empty">{i18n.t('finances-no-budgets', 'No budgets yet.')}</p>
  {:else}
    <div class="budget-list">
      {#each budgets as budget (budget.category)}
        <div class="budget-row">
          <div class="budget-info">
            <span class="budget-cat">{budget.category_label}</span>
            <span class="budget-figures">
              {mask(budget.spent)} / {mask(budget.limit)}
            </span>
          </div>
          <div class="budget-bar">
            <div
              class="budget-fill"
              class:over={budget.over_budget}
              style="width: {budget.percentage}%"
            ></div>
          </div>
          <div class="budget-foot">
            <span class:over={budget.over_budget}>
              {budget.over_budget
                ? `${i18n.t('finances-budget-over', 'Over by')} ${mask(budget.remaining)}`
                : `${mask(budget.remaining)} ${i18n.t('finances-budget-left', 'left')}`}
            </span>
            <button class="budget-remove" onclick={() => remove(budget)}>
              {i18n.t('action-delete', 'Delete')}
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .budget-head {
    display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;
  }
  .budget-note {
    margin: 2px 0 0; font-size: 0.78rem; color: var(--text-tertiary); max-width: 46ch; line-height: 1.45;
  }
  .budget-form {
    display: flex; flex-direction: column; gap: 8px; margin: 12px 0;
    padding: 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
  }
  .budget-form input, .budget-form select {
    padding: 8px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.85rem;
  }
  .budget-form input:focus, .budget-form select:focus {
    border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .budget-actions { display: flex; justify-content: flex-end; gap: 8px; }

  .budget-list { display: flex; flex-direction: column; gap: 14px; margin-top: 10px; }
  .budget-info, .budget-foot {
    display: flex; justify-content: space-between; align-items: baseline; gap: 8px;
  }
  .budget-cat { font-size: 0.88rem; color: var(--text-primary); }
  .budget-figures { font-size: 0.8rem; color: var(--text-secondary); font-variant-numeric: tabular-nums; }
  .budget-bar {
    height: 6px; margin: 6px 0 4px; border-radius: 3px;
    background: var(--glass); overflow: hidden;
  }
  .budget-fill {
    height: 100%; border-radius: 3px; background: var(--accent);
    transition: width 0.3s ease;
  }
  .budget-fill.over { background: var(--danger); }
  .budget-foot { font-size: 0.75rem; color: var(--text-tertiary); }
  .budget-foot .over { color: var(--danger); }
  .budget-remove {
    background: none; border: none; padding: 0; cursor: pointer;
    font: inherit; color: var(--text-tertiary); transition: color 0.15s;
  }
  .budget-remove:hover { color: var(--danger); }
</style>
