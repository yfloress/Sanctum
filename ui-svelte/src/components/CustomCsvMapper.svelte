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
  import { untrack } from 'svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import type { CustomCsvMapping } from '../lib/types'

  interface Props {
    headers: string[]
    sampleRow: string[]
    wallets: string[]
    loading?: boolean
    onimport: (mapping: CustomCsvMapping, walletName: string) => void
    oncancel: () => void
  }

  let { headers, sampleRow, wallets, loading = false, onimport, oncancel }: Props = $props()

  type FieldKey = 'date' | 'asset' | 'amount' | 'type' | 'fee' | 'feeCurrency' | 'price' | 'notes'

  interface FieldDef {
    key: FieldKey
    labelKey: string
    fallback: string
    required: boolean
  }

  const fields: FieldDef[] = [
    { key: 'date', labelKey: 'settings-import-custom-date', fallback: 'Date', required: true },
    { key: 'asset', labelKey: 'settings-import-custom-asset', fallback: 'Asset (coin)', required: true },
    { key: 'amount', labelKey: 'settings-import-custom-amount', fallback: 'Amount', required: true },
    { key: 'type', labelKey: 'settings-import-custom-type', fallback: 'Type', required: false },
    { key: 'fee', labelKey: 'settings-import-custom-fee', fallback: 'Fee', required: false },
    { key: 'feeCurrency', labelKey: 'settings-import-custom-fee-currency', fallback: 'Fee currency', required: false },
    { key: 'price', labelKey: 'settings-import-custom-price', fallback: 'Price', required: false },
    { key: 'notes', labelKey: 'settings-import-custom-notes', fallback: 'Notes', required: false },
  ]

  // Mirrors the backend `normalize_header`: lower-case, strip non-alphanumerics.
  function normalize(header: string): string {
    return header.toLowerCase().replace(/[^a-z0-9]/g, '')
  }

  // Greedy heuristic pre-selection: pick the first still-unused header whose
  // normalized form contains one of the candidate keywords. More specific
  // fields are resolved first so they claim their column before broader ones
  // (e.g. "Fee Currency" before "Fee", "Fee" before "Currency"/asset).
  function autoMap(cols: string[]): Record<FieldKey, string> {
    const normalized = cols.map((raw) => ({ raw, n: normalize(raw) }))
    const used = new Set<string>()
    const pick = (candidates: string[]): string => {
      for (const candidate of candidates) {
        const hit = normalized.find((h) => !used.has(h.raw) && h.n.includes(candidate))
        if (hit) {
          used.add(hit.raw)
          return hit.raw
        }
      }
      return ''
    }
    const date = pick(['date', 'time', 'fecha', 'timestamp'])
    const price = pick(['unitprice', 'price', 'rate', 'precio', 'cotizacion'])
    const feeCurrency = pick(['feecurrency', 'feecoin', 'feeasset', 'feesymbol', 'monedacomision'])
    const fee = pick(['fee', 'commission', 'comision', 'comisin'])
    const asset = pick(['asset', 'coin', 'symbol', 'crypto', 'token', 'currency', 'moneda', 'activo'])
    const amount = pick(['amount', 'quantity', 'qty', 'cantidad', 'monto', 'importe', 'volume', 'vol'])
    const type = pick(['type', 'side', 'operation', 'direction', 'tipo', 'operacion', 'accion'])
    const notes = pick(['notes', 'note', 'memo', 'description', 'descripcion', 'nota', 'remark', 'label', 'comment'])
    return { date, asset, amount, type, fee, feeCurrency, price, notes }
  }

  // Snapshot the props once: this component is freshly mounted per analysed
  // file, so the heuristic guesses and default wallet are intentionally seeded
  // from the initial values (and remain user-editable thereafter).
  let sel = $state<Record<FieldKey, string>>(untrack(() => autoMap(headers)))
  let walletName = $state(untrack(() => wallets[0] ?? ''))

  const canImport = $derived(!!sel.date && !!sel.asset && !!sel.amount && !!walletName)

  function submit() {
    if (!canImport || loading) return
    const mapping: CustomCsvMapping = {
      date_col: sel.date,
      asset_col: sel.asset,
      amount_col: sel.amount,
      type_col: sel.type || null,
      fee_col: sel.fee || null,
      fee_currency_col: sel.feeCurrency || null,
      price_col: sel.price || null,
      notes_col: sel.notes || null,
    }
    onimport(mapping, walletName)
  }
</script>

<div class="mapper">
  <p class="mapper-intro">
    {i18n.t('settings-import-custom-intro', 'Match each Sanctum field to a column from your CSV. Date, asset and amount are required.')}
  </p>

  <div class="preview">
    <span class="preview-title">{i18n.t('settings-import-custom-preview', 'Column preview (first row)')}</span>
    <div class="preview-grid">
      {#each headers as header, i (i)}
        <div class="preview-cell">
          <span class="preview-head" title={header}>{header}</span>
          <span class="preview-val" title={sampleRow[i] ?? ''}>{sampleRow[i] ?? '—'}</span>
        </div>
      {/each}
    </div>
  </div>

  <div class="fields">
    {#each fields as field (field.key)}
      <label class="field-row">
        <span class="field-label">
          {i18n.t(field.labelKey, field.fallback)}{#if field.required}<span class="req">*</span>{/if}
        </span>
        <select
          bind:value={sel[field.key]}
          class:invalid={field.required && !sel[field.key]}
        >
          <option value="">
            {field.required
              ? i18n.t('settings-import-custom-select', '— Select column —')
              : i18n.t('settings-import-custom-none', '— None —')}
          </option>
          {#each headers as header (header)}
            <option value={header}>{header}</option>
          {/each}
        </select>
      </label>
    {/each}
  </div>

  <label class="field-row">
    <span class="field-label">
      {i18n.t('settings-import-target-wallet', 'Target Wallet')}<span class="req">*</span>
    </span>
    {#if wallets.length > 0}
      <select bind:value={walletName}>
        {#each wallets as wallet (wallet)}
          <option value={wallet}>{wallet}</option>
        {/each}
      </select>
    {:else}
      <span class="no-wallets">
        {i18n.t('settings-import-custom-no-wallets', 'Create a wallet in the Crypto section first.')}
      </span>
    {/if}
  </label>

  <div class="actions">
    <button class="secondary-btn" onclick={oncancel} disabled={loading}>
      {i18n.t('settings-cancel', 'Cancel')}
    </button>
    <button class="primary-btn" onclick={submit} disabled={loading || !canImport}>
      {loading
        ? i18n.t('settings-import-importing', 'Importing...')
        : i18n.t('settings-import-confirm', 'Confirm Import')}
    </button>
  </div>
</div>

<style>
  .mapper {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .mapper-intro {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.85rem;
    line-height: 1.5;
  }

  .preview {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .preview-title {
    color: var(--text-tertiary);
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .preview-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .preview-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 110px;
    max-width: 180px;
    padding: 6px 10px;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    background: var(--bg-tertiary);
  }

  .preview-head {
    color: var(--text-secondary);
    font-size: 0.74rem;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .preview-val {
    color: var(--text-primary);
    font-size: 0.82rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .fields {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 10px 16px;
  }

  .field-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    color: var(--text-secondary);
    font-size: 0.82rem;
  }

  .req {
    color: var(--danger, #e5484d);
    margin-left: 2px;
  }

  select {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.85rem;
  }

  select.invalid {
    border-color: var(--danger, #e5484d);
  }

  .no-wallets {
    color: var(--text-tertiary);
    font-size: 0.82rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }
</style>
