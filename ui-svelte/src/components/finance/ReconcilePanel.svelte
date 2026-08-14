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
  import { i18n } from '../../lib/stores/i18n.svelte'
  import { dialog } from '../../lib/actions/dialog'
  import { formatCurrency } from '../../lib/currency'
  import type { ReconciliationResponse } from '../../lib/types'

  interface Props {
    show: boolean
    data: ReconciliationResponse | null
    onconfirm: (ids: string[]) => void
    onclose: () => void
  }

  let { show, data, onconfirm, onclose }: Props = $props()

  /** What the bank says, as typed. Kept as text so an empty box is not zero. */
  let statement = $state('')
  let ticked = $state<string[]>([])

  let tickedSet = $derived(new Set(ticked))

  // Reset per opening: a leftover figure from the last account would silently
  // start the next reconciliation from the wrong place.
  $effect(() => {
    if (show) {
      statement = ''
      ticked = []
    }
  })

  function toggle(id: string) {
    ticked = tickedSet.has(id) ? ticked.filter(t => t !== id) : [...ticked, id]
  }

  let tickedCents = $derived(
    (data?.pending ?? [])
      .filter(row => tickedSet.has(row.id))
      .reduce((sum, row) => sum + row.amount_cents, 0)
  )

  /** Where the account stands counting what is confirmed plus what is ticked. */
  let markedCents = $derived((data?.confirmed_cents ?? 0) + tickedCents)

  /** The typed figure in cents, or null while the box is empty or unparseable. */
  let statementCents = $derived.by(() => {
    const raw = statement.trim().replace(/\s/g, '')
    if (!raw) return null
    // Accept both separators: a comma decimal is as common here as a dot.
    const cleaned = raw.replace(/\.(?=\d{3}\b)/g, '').replace(',', '.')
    const value = Number(cleaned)
    return Number.isFinite(value) ? Math.round(value * 100) : null
  })

  let difference = $derived(statementCents === null ? null : statementCents - markedCents)
  let balanced = $derived(difference === 0)

  function money(cents: number): string {
    return formatCurrency(cents / 100, data?.currency ?? 'USD')
  }
</script>

{#if show && data}
  <div class="overlay-backdrop" role="presentation" onclick={onclose}></div>
  <div class="detail-panel" use:dialog={{ onclose }}>
    <div class="panel-header">
      <h3>{i18n.tArgs('reconcile-title', { account: data.account_name }, `Reconcile ${data.account_name}`)}</h3>
      <button class="close-panel" onclick={onclose} aria-label={i18n.t('finances-close', 'Close')}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <p class="reconcile-hint">{i18n.t('reconcile-hint', 'Enter the balance your bank shows, then tick the rows that appear on your statement.')}</p>

    <label class="reconcile-field">
      <span>{i18n.t('reconcile-statement', 'Balance according to your bank')}</span>
      <input type="text" inputmode="decimal" bind:value={statement} placeholder="0" />
    </label>

    <div class="reconcile-summary">
      <div class="reconcile-line">
        <span>{i18n.t('reconcile-marked', 'Marked in Sanctum')}</span>
        <span class="reconcile-value">{money(markedCents)}</span>
      </div>
      <div class="reconcile-line total" class:balanced>
        <span>{i18n.t('reconcile-difference', 'Difference')}</span>
        <span class="reconcile-value">{difference === null ? '--' : money(difference)}</span>
      </div>
    </div>

    {#if balanced}
      <p class="reconcile-ok">{i18n.t('reconcile-balanced', 'Everything matches.')}</p>
    {/if}

    {#if data.pending.length === 0}
      <p class="reconcile-empty">{i18n.t('reconcile-nothing-pending', 'Nothing left to confirm on this account.')}</p>
    {:else}
      <div class="reconcile-list">
        {#each data.pending as row (row.id)}
          <label class="reconcile-row" class:ticked={tickedSet.has(row.id)}>
            <input type="checkbox" checked={tickedSet.has(row.id)} onchange={() => toggle(row.id)} />
            <span class="reconcile-date">{row.date}</span>
            <span class="reconcile-desc">{row.description}</span>
            <!-- Formatted here rather than taking the backend's string, so a row
                 and the totals below it cannot disagree on how money looks. -->
            <span class="reconcile-amount" class:negative={row.amount_cents < 0}>{money(row.amount_cents)}</span>
          </label>
        {/each}
      </div>
    {/if}

    <div class="modal-actions">
      <button class="secondary-btn" onclick={onclose}>{i18n.t('finances-cancel', 'Cancel')}</button>
      <!-- Enabled only at zero: confirming with a difference would record a
           check that did not actually pass. -->
      <button class="primary-btn" disabled={!balanced || ticked.length === 0}
        onclick={() => onconfirm(ticked)}>
        {i18n.tArgs('reconcile-confirm', { count: ticked.length }, `Confirm ${ticked.length}`)}
      </button>
    </div>
  </div>
{/if}

<style>
  .reconcile-hint {
    margin: 0 0 16px;
    color: var(--text-tertiary);
    font-size: 0.78rem;
    line-height: 1.45;
  }

  .reconcile-field { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: var(--text-secondary); }
  .reconcile-field input {
    padding: 9px 12px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass-active);
    color: var(--text-primary);
    font-family: inherit;
    font-size: 0.9rem;
  }
  .reconcile-field input:focus { border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow); }

  .reconcile-summary {
    margin: 16px 0;
    padding: 12px 14px;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    background: var(--glass-active);
  }
  .reconcile-line { display: flex; justify-content: space-between; gap: 12px; font-size: 0.8rem; color: var(--text-secondary); }
  .reconcile-line + .reconcile-line { margin-top: 8px; padding-top: 8px; border-top: 1px solid var(--glass-border); }
  .reconcile-line.total { color: var(--text-primary); font-weight: 600; }
  /* Green only at exactly zero: "close" is not reconciled. */
  .reconcile-line.total.balanced .reconcile-value { color: var(--success, #4ade80); }
  .reconcile-value { font-variant-numeric: tabular-nums; }

  .reconcile-ok { margin: 0 0 12px; color: var(--success, #4ade80); font-size: 0.8rem; }
  .reconcile-empty { margin: 16px 0; color: var(--text-tertiary); font-size: 0.82rem; text-align: center; }

  .reconcile-list { max-height: 44vh; overflow-y: auto; margin-bottom: 8px; }
  .reconcile-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 6px;
    border-bottom: 1px solid var(--glass-border);
    cursor: pointer;
    font-size: 0.8rem;
  }
  .reconcile-row:last-child { border-bottom: none; }
  .reconcile-row:hover { background: var(--glass-hover); }
  .reconcile-row.ticked { background: var(--accent-glow); }
  .reconcile-row input[type="checkbox"] { width: 15px; height: 15px; flex-shrink: 0; cursor: pointer; }
  .reconcile-date { color: var(--text-tertiary); font-size: 0.72rem; white-space: nowrap; }
  .reconcile-desc { flex: 1; min-width: 0; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .reconcile-amount { color: var(--text-primary); font-variant-numeric: tabular-nums; white-space: nowrap; }
  .reconcile-amount.negative { color: var(--text-secondary); }
</style>
