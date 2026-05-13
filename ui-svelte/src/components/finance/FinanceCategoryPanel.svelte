<!-- Sanctum — a privacy-first personal finance, crypto, and habits vault.
     Copyright (C) 2026  Kyronix

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
  import type { CategoriesResponse } from '../../lib/types'
  import ConfirmDialog from '../ConfirmDialog.svelte'

  interface Props {
    categories: CategoriesResponse | null
    onadd: (name: string, type: 'expense' | 'income') => Promise<void>
    ondelete: (id: string) => Promise<void>
  }

  let { categories, onadd, ondelete }: Props = $props()

  let newCatName = $state('')
  let newCatType = $state<'expense' | 'income'>('expense')
  let pendingDeleteCat = $state<{ id: string; name: string } | null>(null)

  async function addCategory() {
    if (!newCatName.trim()) return
    await onadd(newCatName, newCatType)
    newCatName = ''
  }
</script>

<section class="tab-content">
  <div class="settings-card">
    <span class="settings-card-label">{i18n.t('finances-new-category', 'New Category')}</span>
    <div class="cat-add-row">
      <input
        class="cat-name-input"
        type="text"
        placeholder={i18n.t('finances-category-placeholder', 'Category name...')}
        bind:value={newCatName}
        onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter' && newCatName.trim()) addCategory() }}
      />
      <div class="toggle-row cat-type-toggle">
        <button class="toggle-btn" class:active={newCatType === 'expense'} onclick={() => newCatType = 'expense'}>{i18n.t('finances-expense', 'Expense')}</button>
        <button class="toggle-btn" class:active={newCatType === 'income'} onclick={() => newCatType = 'income'}>{i18n.t('finances-income', 'Income')}</button>
      </div>
      <button class="primary-btn" onclick={addCategory} disabled={!newCatName.trim()}>{i18n.t('finances-add', 'Add')}</button>
    </div>
  </div>

  {#if categories}
    <div class="cat-columns">
      <div class="cat-col">
        <div class="cat-col-header">
          <span class="cat-col-dot cat-col-dot--expense"></span>
          <h4>{i18n.t('finances-expense', 'Expense')}</h4>
          <span class="cat-count">{categories.expense.length}</span>
        </div>
        <div class="cat-chips">
          {#each categories.expense as cat}
            <div class="cat-chip" class:cat-chip--default={cat.is_default}>
              <span class="cat-chip-name">{cat.name}</span>
              {#if cat.is_default}
                <svg class="cat-chip-lock" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/>
                </svg>
              {:else}
                <button class="cat-chip-del" onclick={() => pendingDeleteCat = { id: cat.id, name: cat.name }} aria-label="Delete {cat.name}">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <path d="M18 6L6 18M6 6l12 12"/>
                  </svg>
                </button>
              {/if}
            </div>
          {/each}
        </div>
      </div>
      <div class="cat-col">
        <div class="cat-col-header">
          <span class="cat-col-dot cat-col-dot--income"></span>
          <h4>{i18n.t('finances-income', 'Income')}</h4>
          <span class="cat-count">{categories.income.length}</span>
        </div>
        <div class="cat-chips">
          {#each categories.income as cat}
            <div class="cat-chip" class:cat-chip--default={cat.is_default}>
              <span class="cat-chip-name">{cat.name}</span>
              {#if cat.is_default}
                <svg class="cat-chip-lock" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/>
                </svg>
              {:else}
                <button class="cat-chip-del" onclick={() => pendingDeleteCat = { id: cat.id, name: cat.name }} aria-label="Delete {cat.name}">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <path d="M18 6L6 18M6 6l12 12"/>
                  </svg>
                </button>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</section>

<ConfirmDialog
  show={pendingDeleteCat !== null}
  message={i18n.t('confirm-delete-category', 'Are you sure you want to delete this category?')}
  detail={pendingDeleteCat?.name ?? ''}
  danger
  onconfirm={async () => {
    if (pendingDeleteCat) await ondelete(pendingDeleteCat.id)
    pendingDeleteCat = null
  }}
  onclose={() => pendingDeleteCat = null}
/>

<style>
  .tab-content { display: flex; flex-direction: column; gap: 20px; padding-top: 20px; }

  .settings-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 22px;
    margin-bottom: 20px;
    box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .settings-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.6;
  }
  .settings-card-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.68rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    font-weight: 600;
    margin-bottom: 14px;
  }
  .settings-card-label::before {
    content: '';
    width: 3px;
    height: 12px;
    border-radius: 2px;
    background: linear-gradient(180deg, var(--accent) 0%, var(--accent-hover) 100%);
    box-shadow: 0 0 6px var(--accent-glow);
  }

  .cat-add-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
  .cat-name-input {
    flex: 1; min-width: 140px; padding: 9px 12px;
    border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--glass-active); color: var(--text-primary); font-size: 0.875rem;
    transition: border-color 0.2s;
  }
  .cat-name-input:focus {
    border-color: var(--accent); outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .cat-type-toggle { flex-shrink: 0; }

  .cat-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  .cat-col {
    position: relative;
    background: var(--card-bg);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    padding: 18px;
    box-shadow: var(--card-shadow);
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .cat-col:hover {
    border-color: var(--glass-border-hover);
    box-shadow: var(--glass-shadow-lg), inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }
  .cat-col-header { display: flex; align-items: center; gap: 8px; margin-bottom: 16px; }
  .cat-col-header h4 { font-size: 0.875rem; font-weight: 600; color: var(--text-primary); margin: 0; flex: 1; }
  .cat-col-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .cat-col-dot--expense { background: var(--danger); }
  .cat-col-dot--income  { background: var(--success); }
  .cat-count {
    font-size: 0.68rem;
    color: var(--text-tertiary);
    background: var(--glass-active);
    border: 1px solid var(--glass-border);
    border-radius: 20px;
    padding: 1px 8px;
  }
  .cat-chips { display: flex; flex-wrap: wrap; gap: 8px; }
  .cat-chip {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 8px 5px 12px;
    background: var(--glass-active);
    border: 1px solid var(--glass-border);
    border-radius: 20px;
    transition: border-color 0.15s;
  }
  .cat-chip--default {
    border-color: rgba(168, 85, 247, 0.2);
    background: rgba(168, 85, 247, 0.06);
    padding-right: 12px;
  }
  .cat-chip-name {
    font-size: 0.8rem;
    color: var(--text-primary);
  }
  .cat-chip-del {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    display: flex;
    color: var(--text-tertiary);
    transition: color 0.15s;
    line-height: 0;
  }
  .cat-chip-del:hover { color: var(--danger); }
  .cat-chip-del svg { width: 11px; height: 11px; }
  .cat-chip-lock { width: 11px; height: 11px; color: var(--accent); flex-shrink: 0; }
</style>
