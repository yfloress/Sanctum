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

<!-- One-line entry: "5000 almuerzo" and Enter files a spending of 5000 dated
     today, under the category that description had last time. A leading "+"
     makes it income. -->

<script lang="ts">
  import { localIsoDate } from '../../lib/dates'
  import { errorMessage } from '../../lib/errors'
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import * as financeApi from '../../lib/api/finance'
  import type { AccountDto, CategoriesResponse } from '../../lib/types'
  import type { DescriptionHistoryEntry } from './FinanceTransactionModal.svelte'

  interface Props {
    accounts: AccountDto[]
    categories: CategoriesResponse
    descriptionHistory: DescriptionHistoryEntry[]
    onadded: () => Promise<void>
  }

  let { accounts, categories, descriptionHistory, onadded }: Props = $props()

  const ACCOUNT_KEY = 'sanctum.quickEntryAccount'

  let text = $state('')
  let busy = $state(false)
  let openAccounts = $derived(accounts.filter(a => !a.is_archived))
  let accountId = $state(readAccount())

  function readAccount(): string {
    try {
      return localStorage.getItem(ACCOUNT_KEY) ?? ''
    } catch {
      return ''
    }
  }

  $effect(() => {
    if (!openAccounts.some(a => a.id === accountId)) accountId = openAccounts[0]?.id ?? ''
  })

  function rememberAccount() {
    try {
      localStorage.setItem(ACCOUNT_KEY, accountId)
    } catch {
      // Only a convenience; the first account is a fine default.
    }
  }

  /** "5.000 almuerzo", "+250000 sueldo", "$3990 uber". */
  let parsed = $derived.by(() => {
    const match = text.match(/^\s*([+-]?)\s*\$?\s*(\d[\d.,]*)\s+(.+?)\s*$/)
    if (!match) return null
    return { income: match[1] === '+', amount: match[2], description: match[3] }
  })

  function normalizeAmount(raw: string, currency: string): string {
    // Pesos have no cents, so both separators are thousands; elsewhere a
    // comma is the decimal point the backend wants as a dot.
    return currency.toUpperCase() === 'CLP' ? raw.replace(/[.,]/g, '') : raw.replace(/,/g, '.')
  }

  function categoryFor(description: string, income: boolean): string {
    const options = (income ? categories.income : categories.expense) ?? []
    const key = description.trim().toLowerCase()
    const remembered = descriptionHistory.find(e => e.description.trim().toLowerCase() === key)?.category
    const match = remembered && options.find(c => c.name.toUpperCase() === remembered.toUpperCase())
    if (match) return match.name
    return options.find(c => c.name === 'OTHER')?.name ?? options[0]?.name ?? ''
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault()
    const entry = parsed
    const account = openAccounts.find(a => a.id === accountId)
    if (!entry || !account || busy) return
    busy = true
    try {
      await financeApi.addTransaction(
        account.id,
        normalizeAmount(entry.amount, account.currency),
        categoryFor(entry.description, entry.income),
        entry.description,
        localIsoDate(),
        !entry.income,
      )
      text = ''
      await onadded()
      app.showToast(i18n.t('finances-tx-added', 'Transaction added'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      busy = false
    }
  }
</script>

{#if openAccounts.length > 0}
  <form class="quick-entry" onsubmit={submit}>
    <input
      type="text"
      bind:value={text}
      placeholder={i18n.t('finances-quick-placeholder', 'Quick add: 5000 lunch, +250000 salary')}
      aria-label={i18n.t('finances-quick-label', 'Quick add')}
      autocomplete="off"
      disabled={busy}
    />
    <select bind:value={accountId} onchange={rememberAccount} aria-label={i18n.t('finances-account', 'Account')}>
      {#each openAccounts as acc}
        <option value={acc.id}>{acc.name}</option>
      {/each}
    </select>
    <button class="glass-btn" type="submit" disabled={!parsed || busy}>{i18n.t('finances-quick-add', 'Add')}</button>
  </form>
{/if}

<style>
  .quick-entry {
    display: flex; gap: 8px; align-items: center; margin-bottom: 14px;
    padding: 6px; border: 1px solid var(--row-border); border-radius: var(--radius-sm);
    background: var(--row-bg);
  }
  .quick-entry input {
    flex: 1; min-width: 0; padding: 8px 10px; border: none; background: transparent;
    color: var(--text-primary); font-size: 0.88rem;
  }
  .quick-entry input:focus { outline: none; }
  .quick-entry:focus-within { border-color: var(--accent-border); }
  .quick-entry select {
    max-width: 180px; padding: 6px 8px; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: var(--select-bg); color: var(--text-primary);
    font-size: 0.82rem;
  }
  @media (max-width: 640px) {
    .quick-entry { flex-wrap: wrap; }
    .quick-entry input { flex-basis: 100%; }
    .quick-entry select { flex: 1; max-width: none; }
  }
</style>
