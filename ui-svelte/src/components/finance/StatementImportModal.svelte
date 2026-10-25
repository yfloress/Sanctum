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

<!-- Bank statement import: pick the account and the PDF, review every row
     (category, transfer, sign), then write the kept rows. Rows the account
     already has start unchecked. -->

<script lang="ts">
  import { errorMessage } from '../../lib/errors'
  import { formatDate } from '../../lib/dates'
  import { formatMoney, mask } from '../../lib/currency'
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import { dialog } from '../../lib/actions/dialog'
  import * as ingestionApi from '../../lib/api/ingestion'
  import type {
    AccountDto,
    CategoriesResponse,
    StatementFormatDto,
    StatementPreviewDto,
  } from '../../lib/types'

  interface Props {
    show: boolean
    accounts: AccountDto[]
    categories: CategoriesResponse
    onimported: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), accounts, categories, onimported, onclose }: Props = $props()

  /** A preview row as the user is editing it. */
  interface ReviewRow {
    include: boolean
    date: string
    description: string
    amount: number
    /** A category name, or `transfer:<account id>`. */
    target: string
    learned: boolean
    duplicate: boolean
    signGuessed: boolean
    installment: string | null
  }

  const TRANSFER = 'transfer:'

  let clpAccounts = $derived(accounts.filter(a => a.currency.toUpperCase() === 'CLP' && !a.is_archived))

  let step = $state<'pick' | 'review'>('pick')
  let accountId = $state('')
  let file = $state<File | null>(null)
  let formatId = $state('')
  let formats = $state<StatementFormatDto[]>([])
  let password = $state('')
  let needsPassword = $state(false)
  let wrongPassword = $state(false)
  let busy = $state(false)
  let dragging = $state(false)
  let preview = $state<StatementPreviewDto | null>(null)
  let rows = $state<ReviewRow[]>([])
  let showProblems = $state(false)
  let fileInput = $state<HTMLInputElement | null>(null)

  $effect(() => {
    if (!show) return
    step = 'pick'
    file = null
    formatId = ''
    password = ''
    needsPassword = false
    wrongPassword = false
    preview = null
    rows = []
    showProblems = false
    if (!clpAccounts.some(a => a.id === accountId)) accountId = clpAccounts[0]?.id ?? ''
    if (formats.length === 0) {
      ingestionApi.statementFormats().then(f => { formats = f }).catch(() => {})
    }
  })

  let selected = $derived(rows.filter(r => r.include))
  let canRead = $derived(!!file && !!accountId && !busy && (!needsPassword || password.length > 0))

  function categoryOptions(amount: number) {
    return amount < 0 ? categories.expense : categories.income
  }

  function defaultTarget(amount: number, wanted: string): string {
    const options = categoryOptions(amount)
    if (options.some(c => c.name === wanted)) return wanted
    return options.find(c => c.name === 'OTHER')?.name ?? options[0]?.name ?? ''
  }

  function pickFile(list: FileList | null | undefined) {
    const picked = list?.[0]
    if (!picked) return
    file = picked
    needsPassword = false
    wrongPassword = false
    password = ''
  }

  async function read() {
    if (!canRead || !file) return
    busy = true
    try {
      const bytes = new Uint8Array(await file.arrayBuffer())
      const result = await ingestionApi.previewStatement(
        bytes,
        accountId,
        needsPassword ? password : null,
        formatId || null,
      )
      if (result.status !== 'ready' || !result.preview) {
        wrongPassword = result.status === 'wrong_password'
        needsPassword = true
        return
      }
      preview = result.preview
      rows = result.preview.rows.map(r => ({
        include: !r.duplicate,
        date: r.date,
        description: r.description,
        amount: r.amount,
        target: defaultTarget(r.amount, r.category),
        learned: r.learned,
        duplicate: r.duplicate,
        signGuessed: r.sign_guessed,
        installment: r.installment,
      }))
      step = 'review'
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      // The password is not needed past this call.
      password = ''
      busy = false
    }
  }

  function flip(row: ReviewRow) {
    row.amount = -row.amount
    if (!row.target.startsWith(TRANSFER)) row.target = defaultTarget(row.amount, row.target)
    row.learned = false
  }

  async function submit() {
    if (step === 'pick') return read()
    if (busy || selected.length === 0) return
    busy = true
    try {
      const count = await ingestionApi.importStatement(
        accountId,
        selected.map(r => ({
          date: r.date,
          description: r.description,
          amount: r.amount,
          category: r.target.startsWith(TRANSFER) ? '' : r.target,
          transfer_account_id: r.target.startsWith(TRANSFER) ? r.target.slice(TRANSFER.length) : null,
        })),
      )
      show = false
      await onimported()
      app.showToast(i18n.tArgs('statement-imported', { count }, `${count} movements imported`))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      busy = false
    }
  }

  function close() {
    password = ''
    show = false
    onclose()
  }

  function onDrop(event: DragEvent) {
    event.preventDefault()
    dragging = false
    pickFile(event.dataTransfer?.files)
  }

  let period = $derived(
    preview?.period_start && preview?.period_end
      ? `${formatDate(preview.period_start)} – ${formatDate(preview.period_end)}`
      : ''
  )
  let duplicateCount = $derived(rows.filter(r => r.duplicate).length)
  let otherAccounts = $derived(accounts.filter(a => a.id !== accountId && !a.is_archived && a.currency.toUpperCase() === 'CLP'))
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close}></div>
  <div class="modal-wrapper">
    <div class="modal statement-modal" class:review={step === 'review'} use:dialog={{ onclose: close, onsubmit: submit }}>
      <h3>{i18n.t('statement-title', 'Import statement')}</h3>

      {#if step === 'pick'}
        {#if clpAccounts.length === 0}
          <p class="note">{i18n.t('statement-no-clp-accounts', 'You need a peso (CLP) account to import statements.')}</p>
        {:else}
          <div class="form-grid">
            <label>
              {i18n.t('statement-account', 'Account')}
              <select bind:value={accountId}>
                {#each clpAccounts as acc}
                  <option value={acc.id}>{acc.name}</option>
                {/each}
              </select>
            </label>

            <div class="field">
              <span class="field-label">{i18n.t('statement-file', 'PDF file')}</span>
              <button
                type="button"
                class="dropzone"
                class:dragging
                class:has-file={!!file}
                onclick={() => fileInput?.click()}
                ondragover={(e) => { e.preventDefault(); dragging = true }}
                ondragleave={() => dragging = false}
                ondrop={onDrop}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true">
                  <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5M12 18v-6M9 15l3-3 3 3"/>
                </svg>
                <span>{file ? file.name : i18n.t('statement-drop', 'Drop the statement here or click to choose it')}</span>
              </button>
              <input
                bind:this={fileInput}
                class="hidden-input"
                type="file"
                accept="application/pdf,.pdf"
                onchange={(e) => pickFile((e.currentTarget as HTMLInputElement).files)}
              />
            </div>

            <label>
              {i18n.t('statement-bank', 'Bank')}
              <select bind:value={formatId}>
                <option value="">{i18n.t('statement-bank-auto', 'Detect automatically')}</option>
                {#each formats as f}
                  <option value={f.id}>{f.label}</option>
                {/each}
              </select>
            </label>

            {#if needsPassword}
              <label>
                {i18n.t('statement-password', 'PDF password')}
                <input type="password" bind:value={password} autocomplete="off" />
                <span class="hint" class:error={wrongPassword}>
                  {wrongPassword
                    ? i18n.t('statement-wrong-password', 'The password is not correct.')
                    : i18n.t('statement-password-hint', 'Many banks use your RUT, without dots or dash. It is not saved.')}
                </span>
              </label>
            {/if}
          </div>
        {/if}

        <div class="modal-actions">
          <button class="secondary-btn" onclick={close}>{i18n.t('finances-cancel', 'Cancel')}</button>
          <button class="primary-btn" onclick={read} disabled={!canRead}>
            {busy ? i18n.t('statement-reading', 'Reading…') : i18n.t('statement-read', 'Read statement')}
          </button>
        </div>

      {:else if preview}
        <div class="summary">
          <span class="format">{preview.format_label}</span>
          <span class="muted">
            {i18n.tArgs('statement-count', { count: rows.length }, `${rows.length} movements`)}{period ? ` · ${period}` : ''}
          </span>
        </div>
        {#if preview.credit_card}
          <p class="note">{i18n.t('statement-credit-note', 'Card statement: this month\'s purchases, installments and fees are imported, not the payments to the card.')}</p>
        {/if}
        {#if duplicateCount > 0}
          <p class="note">{i18n.tArgs('statement-duplicates', { count: duplicateCount }, `${duplicateCount} were already in the account and start unchecked.`)}</p>
        {/if}
        {#each preview.warnings as warning}
          <p class="alert">{warning}</p>
        {/each}
        {#if preview.problems.length > 0}
          <button type="button" class="problems-toggle" onclick={() => showProblems = !showProblems} aria-expanded={showProblems}>
            {i18n.tArgs('statement-problems', { count: preview.problems.length }, `${preview.problems.length} rows could not be read`)}
          </button>
          {#if showProblems}
            <ul class="problems">
              {#each preview.problems as problem}
                <li><span class="reason">{problem.reason}</span><code>{mask(problem.text)}</code></li>
              {/each}
            </ul>
          {/if}
        {/if}

        <div class="rows" role="list">
          {#each rows as row}
            <div class="row" class:excluded={!row.include} role="listitem">
              <input type="checkbox" bind:checked={row.include} aria-label={i18n.t('statement-include', 'Import this row')} />
              <span class="date">{formatDate(row.date)}</span>
              <div class="desc">
                <input type="text" bind:value={row.description} aria-label={i18n.t('finances-description', 'Description')} />
                <div class="badges">
                  {#if row.duplicate}<span class="badge">{i18n.t('statement-duplicate', 'Already in the account')}</span>{/if}
                  {#if row.installment}<span class="badge">{i18n.tArgs('statement-installment', { n: row.installment }, `Installment ${row.installment}`)}</span>{/if}
                  {#if row.learned}<span class="badge soft" title={i18n.t('statement-learned', 'Category learned from earlier movements')}>{i18n.t('statement-learned-short', 'Learned')}</span>{/if}
                </div>
              </div>
              <select bind:value={row.target} aria-label={i18n.t('finances-category', 'Category')}>
                {#each categoryOptions(row.amount) as cat}
                  <option value={cat.name}>{cat.label}</option>
                {/each}
                {#if otherAccounts.length > 0}
                  <optgroup label={i18n.t('statement-transfer-group', 'Transfer with another account')}>
                    {#each otherAccounts as acc}
                      <option value={TRANSFER + acc.id}>{acc.name}</option>
                    {/each}
                  </optgroup>
                {/if}
              </select>
              <span class="amount" class:out={row.amount < 0}>{mask(formatMoney(row.amount, 'CLP'))}</span>
              {#if row.signGuessed}
                <button
                  type="button"
                  class="flip"
                  title={i18n.t('statement-sign-guessed', 'The direction was read from the description. Check it.')}
                  aria-label={i18n.t('statement-flip', 'Flip direction')}
                  onclick={() => flip(row)}
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M7 4v16M3 8l4-4 4 4M17 20V4M13 16l4 4 4-4"/></svg>
                </button>
              {:else}
                <span class="flip-spacer"></span>
              {/if}
            </div>
          {/each}
        </div>

        <div class="modal-actions">
          <button class="secondary-btn" onclick={() => step = 'pick'}>{i18n.t('statement-back', 'Back')}</button>
          <button class="primary-btn" onclick={submit} disabled={busy || selected.length === 0}>
            {i18n.tArgs('statement-import', { count: selected.length }, `Import ${selected.length}`)}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .statement-modal { width: 460px; }
  .statement-modal.review { width: min(880px, 92vw); }

  .field { display: flex; flex-direction: column; gap: 4px; }
  .field-label { font-size: 0.8rem; color: var(--text-secondary); }
  .hidden-input { display: none; }

  .dropzone {
    display: flex; align-items: center; gap: 12px;
    padding: 18px 16px; border: 1px dashed var(--glass-border); border-radius: var(--radius-sm);
    background: var(--row-bg); color: var(--text-secondary); font-size: 0.85rem;
    cursor: pointer; text-align: left; transition: border-color 0.15s, background 0.15s;
  }
  .dropzone svg { width: 22px; height: 22px; flex-shrink: 0; }
  .dropzone:hover, .dropzone.dragging { border-color: var(--accent-border); background: var(--row-bg-hover); }
  .dropzone.has-file { color: var(--text-primary); border-style: solid; }
  .dropzone span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .hint { font-size: 0.75rem; color: var(--text-secondary); }
  .hint.error { color: var(--danger-vivid); }

  .summary { display: flex; flex-wrap: wrap; align-items: baseline; gap: 8px; margin-bottom: 10px; }
  .format { font-weight: 600; color: var(--text-primary); }
  .muted { color: var(--text-secondary); font-size: 0.85rem; }
  .note { margin: 0 0 8px; font-size: 0.8rem; color: var(--text-secondary); }
  .alert {
    margin: 0 0 8px; padding: 8px 12px; font-size: 0.8rem; border-radius: var(--radius-sm);
    color: var(--text-primary); background: color-mix(in srgb, var(--warning) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--warning) 35%, transparent);
  }

  .problems-toggle {
    background: none; border: none; padding: 0; margin: 0 0 8px; cursor: pointer;
    color: var(--danger-vivid); font-size: 0.8rem; text-decoration: underline;
  }
  .problems { margin: 0 0 10px; padding: 0; list-style: none; display: flex; flex-direction: column; gap: 4px; }
  .problems li { display: flex; flex-direction: column; gap: 2px; font-size: 0.75rem; color: var(--text-secondary); }
  .problems code { font-size: 0.75rem; color: var(--text-primary); word-break: break-word; }
  .reason { color: var(--danger-vivid); }

  .rows {
    display: flex; flex-direction: column; gap: 4px;
    max-height: 52vh; overflow-y: auto; margin-top: 6px; padding-right: 2px;
  }
  .row {
    display: grid; grid-template-columns: 18px 96px minmax(0, 1fr) 190px 120px 28px;
    align-items: center; gap: 10px; padding: 6px 10px;
    background: var(--row-bg); border: 1px solid var(--row-border); border-radius: var(--radius-sm);
  }
  .row.excluded { opacity: 0.5; }
  .row input[type="checkbox"] { width: 16px; height: 16px; cursor: pointer; }
  .row input[type="text"], .row select {
    width: 100%; min-width: 0; padding: 6px 8px; font-size: 0.82rem;
    border: 1px solid transparent; border-radius: var(--radius-sm);
    background: transparent; color: var(--text-primary);
  }
  .row input[type="text"]:hover, .row select:hover { border-color: var(--glass-border); }
  .row input[type="text"]:focus, .row select:focus { border-color: var(--accent); outline: none; background: var(--select-bg); }
  .row select { background: var(--select-bg); border-color: var(--glass-border); }
  .date { font-size: 0.8rem; color: var(--text-secondary); white-space: nowrap; }
  .desc { min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .badges { display: flex; flex-wrap: wrap; gap: 4px; padding-left: 8px; }
  .badges:empty { display: none; }
  .badge {
    font-size: 0.68rem; padding: 1px 6px; border-radius: 999px;
    color: var(--text-secondary); border: 1px solid var(--glass-border);
  }
  .badge.soft { color: var(--accent); border-color: var(--accent-border); }
  .amount { text-align: right; font-size: 0.85rem; font-weight: 600; color: var(--success-vivid); white-space: nowrap; }
  .amount.out { color: var(--danger-vivid); }
  .flip, .flip-spacer { width: 28px; height: 28px; }
  .flip {
    display: grid; place-items: center; padding: 0; cursor: pointer;
    border: 1px solid color-mix(in srgb, var(--warning) 45%, transparent); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary);
  }
  .flip:hover { color: var(--text-primary); }
  .flip svg { width: 14px; height: 14px; }

  @media (max-width: 640px) {
    .row { grid-template-columns: 18px minmax(0, 1fr) 28px; }
    .row .date { grid-column: 2; }
    .row .desc, .row select, .row .amount { grid-column: 2 / 4; }
    .row .amount { text-align: left; }
  }
</style>
