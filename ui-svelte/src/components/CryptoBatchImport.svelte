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
  import * as ingestion from '../lib/api/ingestion'
  import * as crypto from '../lib/api/crypto'
  import { errorMessage } from '../lib/errors'
  import { i18n } from '../lib/stores/i18n.svelte'
  import type { CsvAnalysisResult, CustomCsvMapping, ImportResultsResponse, WalletDto, WalletDetailResponse } from '../lib/types'
  import CustomCsvMapper from './CustomCsvMapper.svelte'

  let { busy = $bindable(false), onimported }: {
    busy?: boolean
    onimported: () => Promise<void>
  } = $props()

  interface Entry {
    id: number
    file: File
    wallet: string
    source: string
    analysis?: CsvAnalysisResult
    mapping?: CustomCsvMapping
    preview?: ImportResultsResponse
    result?: ImportResultsResponse
    error?: string
    allowPartial: boolean
  }

  let sequence = 0
  let entries = $state<Entry[]>([])
  let wallets = $state<WalletDto[]>([])
  let error = $state('')
  let activeMapper = $state<number | null>(null)
  let mapperContent = $state('')
  let walletName = $state('')
  let walletCategory = $state('exchange')
  let drag = $state(false)
  let balances = $state<WalletDetailResponse[]>([])
  let expected = $state<Record<string, string>>({})
  let pending = $derived(entries.filter(row => !row.result))
  let ready = $derived(pending.length > 0 && pending.every(row => row.preview && !row.error))
  let totals = $derived(entries.reduce((sum, row) => {
    if (row.result) {
      sum.inserted += row.result.inserted
      sum.skipped += row.result.skipped
      sum.errors += row.result.errors.length
    }
    return sum
  }, { inserted: 0, skipped: 0, errors: 0 }))

  async function preview(row: Entry) {
    row.error = undefined
    row.preview = undefined
    row.allowPartial = false
    if (!row.wallet || (row.analysis && !row.mapping)) return
    const content = await row.file.text()
    row.preview = row.mapping
      ? await ingestion.previewCustomCsv(content, row.mapping, row.wallet)
      : await ingestion.previewExchangeCsv(content, row.wallet)
  }

  async function addFiles(files: File[]) {
    if (busy || !files.length) return
    busy = true
    error = ''
    try {
      const limit = await ingestion.maxImportFileSize()
      wallets = (await crypto.fetchWallets()).wallets
      for (const file of files) {
        entries.push({ id: ++sequence, file, wallet: '', source: '', allowPartial: false })
        const row = entries[entries.length - 1]
        try {
          if (file.size > limit) throw new Error(i18n.t('crypto-batch-file-large', 'File exceeds the import size limit.'))
          const content = await file.text()
          const detected = await ingestion.detectExchangeSource(content)
          if (detected) {
            row.source = detected.exchange
            row.wallet = wallets.find(w => w.name.toLowerCase() === detected.suggested_wallet.toLowerCase())?.name ?? ''
          } else {
            row.analysis = await ingestion.analyzeCustomCsv(content)
            row.mapping = await ingestion.loadCustomCsvMapping(row.analysis.headers) ?? undefined
          }
          await preview(row)
        } catch (e) {
          row.error = errorMessage(e)
        }
      }
    } catch (e) {
      error = errorMessage(e)
    } finally {
      busy = false
    }
  }

  async function changeWallet(row: Entry, name: string) {
    if (busy) return
    busy = true
    row.wallet = name
    try {
      await preview(row)
    } catch (e) {
      row.error = errorMessage(e)
    } finally {
      busy = false
    }
  }

  async function createWallet() {
    if (busy || !walletName.trim()) return
    busy = true
    error = ''
    try {
      await crypto.addWallet(walletName.trim(), walletCategory)
      wallets = (await crypto.fetchWallets()).wallets
      walletName = ''
    } catch (e) {
      error = errorMessage(e)
    } finally {
      busy = false
    }
  }

  async function editMapping(row: Entry) {
    if (busy) return
    busy = true
    try {
      mapperContent = await row.file.text()
      activeMapper = row.id
    } catch (e) {
      row.error = errorMessage(e)
    } finally {
      busy = false
    }
  }

  function closeMapper() {
    activeMapper = null
    mapperContent = ''
  }

  async function loadBalances() {
    balances = []
    for (const name of new Set(entries.filter(row => row.result).map(row => row.wallet))) {
      const wallet = wallets.find(w => w.name === name)
      if (wallet) balances.push(await crypto.fetchWalletDetail(wallet.id))
    }
  }

  async function apply() {
    if (busy || !ready) return
    busy = true
    error = ''
    try {
      for (const row of pending) {
        try {
          const content = await row.file.text()
          const current = row.mapping
            ? await ingestion.previewCustomCsv(content, row.mapping, row.wallet)
            : await ingestion.previewExchangeCsv(content, row.wallet)
          if (current.errors.length && (!row.allowPartial ||
              JSON.stringify(current.errors) !== JSON.stringify(row.preview?.errors))) {
            row.preview = current
            row.allowPartial = false
            break
          }
          row.result = row.mapping
            ? await ingestion.importCustomCsv(content, row.mapping, row.wallet)
            : await ingestion.importExchangeCsv(content, row.wallet)
        } catch (e) {
          row.error = errorMessage(e)
          break
        }
      }
      if (entries.some(row => row.result)) {
        await onimported()
        await loadBalances()
      }
    } catch (e) {
      error = errorMessage(e)
    } finally {
      busy = false
    }
  }

  function difference(text: string, recorded: number): string {
    if (!text || !Number.isFinite(Number(text))) return ''
    const delta = Number(text) - recorded
    const tolerance = 4 * Number.EPSILON * Math.max(Math.abs(recorded), Math.abs(Number(text)))
    return Math.abs(delta) <= tolerance
      ? i18n.t('crypto-batch-matches', 'Matches')
      : i18n.t('crypto-batch-difference', 'Difference') + ' ' + Number(delta.toPrecision(15))
  }
</script>

<div class="batch" aria-busy={busy}>
  <label class="drop" class:drag class:inactive={busy}>
    <span class="upload-icon" aria-hidden="true"></span>
    <strong>{i18n.t('crypto-batch-select', 'Choose or drop history files')}</strong>
    <span class="extensions">CSV · TSV · TXT</span>
    <input type="file" accept=".csv,.tsv,.txt" multiple disabled={busy}
      aria-label={i18n.t('crypto-batch-select', 'Choose or drop history files')}
      ondragenter={() => drag = true}
      ondragleave={() => drag = false}
      ondrop={() => drag = false}
      onchange={e => { const files = Array.from(e.currentTarget.files ?? []); e.currentTarget.value = ''; void addFiles(files) }} />
  </label>
  {#if entries.length}
    <details>
      <summary>{i18n.t('crypto-batch-new-wallet', 'Create a destination wallet')}</summary>
      <div class="controls">
        <input aria-label={i18n.t('settings-import-wallet-placeholder', 'Wallet name')} bind:value={walletName}
          placeholder={i18n.t('settings-import-wallet-placeholder', 'Wallet name')} disabled={busy} />
        <select aria-label={i18n.t('crypto-wallet-category', 'Category')} bind:value={walletCategory} disabled={busy}>
          <option value="exchange">Exchange</option><option value="software">Software</option><option value="hardware">Hardware</option>
        </select>
        <button class="glass-btn" onclick={createWallet} disabled={busy || !walletName.trim()}>{i18n.t('crypto-batch-create', 'Create')}</button>
      </div>
    </details>
  {/if}
  {#each entries as row (row.id)}
    {@const summary = row.result ?? row.preview}
    <section>
      <div class="controls"><strong>{row.file.name}</strong>{row.source}
        {#if !row.result}
          <button class="glass-btn" disabled={busy} onclick={() => { if (activeMapper === row.id) closeMapper(); entries = entries.filter(r => r.id !== row.id) }}>
            {i18n.t('crypto-batch-remove', 'Remove')}
          </button>
        {/if}
      </div>
      {#if (row.source || row.analysis) && !row.result}
        <label>{i18n.t('settings-import-target-wallet', 'Target Wallet')}
          <select value={row.wallet} disabled={busy} onchange={e => changeWallet(row, e.currentTarget.value)}>
            <option value="">—</option>
            {#each wallets as w (w.id)}<option value={w.name}>{w.name}</option>{/each}
          </select>
        </label>
        {#if row.analysis}
          {#if activeMapper === row.id}
            <CustomCsvMapper headers={row.analysis.headers} sampleRow={row.analysis.sample_row} content={mapperContent}
              wallets={wallets.map(w => w.name)} initialWallet={row.wallet} loading={busy}
              submitLabel={i18n.t('settings-import-preview-btn', 'Preview')}
              oncancel={closeMapper} onimport={(mapping, name) => { row.mapping = mapping; closeMapper(); void changeWallet(row, name) }} />
          {:else}
            <button class="glass-btn" disabled={busy} onclick={() => editMapping(row)}>{i18n.t('crypto-batch-map', 'Review columns')}</button>
          {/if}
        {/if}
      {/if}
      {#if summary}
        <p>{row.result ? i18n.t('settings-import-inserted', 'Inserted:') : i18n.t('settings-import-preview-btn', 'Preview')} {summary.inserted}
          · {i18n.t('settings-import-skipped', 'Skipped:')} {summary.skipped}</p>
        {#if summary.errors.length}
          <details><summary>{i18n.t('settings-import-errors', 'Errors')} ({summary.errors.length})</summary>
            <ul>{#each summary.errors as issue}<li>{issue.line ?? '—'} · {issue.message}</li>{/each}</ul>
          </details>
          {#if !row.result}
            <label><input type="checkbox" bind:checked={row.allowPartial} disabled={busy} />
              {i18n.t('crypto-batch-partial', 'Import valid rows and skip these errors')}</label>
          {/if}
        {/if}
      {/if}
      {#if row.error}<p role="alert">{row.error}</p>
        {#if row.wallet && (row.source || row.mapping)}
          <button class="glass-btn" disabled={busy} onclick={() => changeWallet(row, row.wallet)}>{i18n.t('crypto-batch-retry', 'Retry preview')}</button>
        {/if}
      {/if}
    </section>
  {/each}
  {#if pending.length}
    <small>{i18n.t('crypto-batch-overlap', 'Previews are per file. Overlaps are checked again as each file is imported.')}</small>
    <button class="primary-btn" disabled={busy || !ready || pending.some(r => r.preview?.errors.length && !r.allowPartial)} onclick={apply}>
      {i18n.t('crypto-batch-apply', 'Import reviewed files')} ({pending.length})
    </button>
  {/if}
  {#if entries.some(row => row.result)}
    <p>{i18n.t('settings-import-inserted', 'Inserted:')} {totals.inserted} · {i18n.t('settings-import-skipped', 'Skipped:')} {totals.skipped} · {i18n.t('settings-import-errors', 'Errors')} {totals.errors}</p>
    {#each balances as detail (detail.id)}
      <details open>
        <summary>{detail.name} · {i18n.t('crypto-batch-balances', 'Compare balances')}</summary>
        {#each detail.holdings as holding (holding.coin_id)}
          {@const key = JSON.stringify([detail.id, holding.coin_id])}
          <label class="controls">{holding.symbol} · {holding.recorded_amount}
            <input type="number" step="any" placeholder={i18n.t('crypto-batch-expected', 'Wallet balance')}
              value={expected[key] ?? ''} oninput={e => expected[key] = e.currentTarget.value} />
            {difference(expected[key], holding.recorded_amount)}
          </label>
        {:else}
          <p>{i18n.t('crypto-batch-no-balances', 'No holdings recorded for this wallet.')}</p>
        {/each}
      </details>
    {/each}
  {/if}
  {#if error}<p role="alert">{error}</p>{/if}
  {#if busy}<p role="status">{i18n.t('settings-import-loading', 'Loading...')}</p>{/if}
</div>

<style>
  .batch, section { display: grid; gap: 0.65rem; }
  .batch { max-height: 65vh; overflow-y: auto; }
  .drop {
    position: relative; display: grid; gap: .35rem; justify-items: center; text-align: center;
    padding: 1.5rem 1rem; border: 1px dashed var(--accent); border-radius: var(--radius-md, 12px);
    background: var(--accent-glow); color: var(--text-primary);
  }
  .drop input { position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer; }
  .drop.drag, .drop:focus-within { outline: 2px solid var(--accent); outline-offset: 2px; }
  .drop.inactive { opacity: .65; }
  .upload-icon { width: 1.65rem; height: 1.65rem; background: var(--accent); mask: url('/assets/icons/upload.svg') center / contain no-repeat; }
  .extensions, small { color: var(--text-secondary); font-size: .875rem; }
  section { border-top: 1px solid var(--glass-border); padding: 0.75rem 0; }
  strong { overflow-wrap: anywhere; }
  .controls, label { display: flex; align-items: center; flex-wrap: wrap; gap: 0.5rem; }
  details > .controls { margin-top: 0.5rem; }
  p, ul { margin: 0; }
  summary { cursor: pointer; }
  small { color: var(--text-secondary); }
  input, select { min-width: 0; max-width: 100%; }
  ul { max-height: 10rem; overflow-y: auto; }
</style>
