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
  import { i18n } from '../lib/stores/i18n.svelte'
  import type { CustomCsvMapping } from '../lib/types'

  let { headers, onmap }: { headers: string[], onmap: (mapping: CustomCsvMapping) => void } = $props()
  let response = $state('')
  let invalid = $state(false)
  const keys = ['date_col', 'asset_col', 'amount_col', 'type_col', 'fee_col', 'fee_currency_col', 'price_col', 'notes_col'] as const

  // Only fixed vocabulary can leave this component in the suggested prompt.
  // Headers may themselves contain an account number, address, or personal name.
  const vocabulary = new Map<string, [string, string, string]>()
  function define(words: string, label: string, first: string, second: string) {
    for (const word of words.split(' ')) vocabulary.set(word, [label, first, second])
  }
  define('date fecha time timestamp datetime utcdate utctime', 'date/time', '2000-01-01', '2000-01-02')
  define('asset coin token currency symbol moneda activo', 'asset symbol', 'BTC', 'LTC')
  define('amount quantity qty cantidad monto importe volume netamount', 'quantity', '1.25', '-2')
  define('type side operation direction tipo operacion', 'operation type', 'deposit', 'withdrawal')
  define('fee fees commission comision', 'fee', '0.001', '0.002')
  define('feecoin feeasset feesymbol feecurrency monedacomision', 'fee currency', 'BTC', 'LTC')
  define('unitpriceusd usdprice priceusd', 'USD unit price', '100', '50')
  define('price rate unitprice precio cotizacion', 'price (unit unknown)', '100', '50')
  define('note notes memo description descripcion label comment', 'notes', 'Example A', 'Example B')

  const prompt = $derived.by(() => {
    const cols = headers.map(h => vocabulary.get(h.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase().replace(/[^a-z0-9]/g, '')) ?? ['unknown', '?', '?'])
    return [
      'Suggest a column mapping for a single-row-per-event crypto CSV. Only column numbers and synthetic examples follow.',
      'Column labels: ' + cols.map((c, i) => (i + 1) + '=' + c[0]).join(' | '),
      'Synthetic row A: ' + cols.map(c => c[1]).join(' | '),
      'Synthetic row B: ' + cols.map(c => c[2]).join(' | '),
      'Return only a JSON object with these keys: ' + keys.join(', ') + '.',
      'For each key use a unique 1-based column number or null if ambiguous. price_col means USD per unit. Do not infer roles from synthetic values.'
    ].join('\n')
  })

  function apply() {
    try {
      if (response.length > 8192) throw Error()
      const object: unknown = JSON.parse(response)
      if (!object || Array.isArray(object) || typeof object !== 'object') throw Error()
      const values = object as Record<string, unknown>
      if (Object.keys(values).length !== keys.length || !keys.every(k => Object.prototype.hasOwnProperty.call(values, k))) throw Error()
      const used = new Set<number>()
      function read(key: typeof keys[number]) {
        const value = values[key]
        if (value === null) return null
        if (typeof value !== 'number' || !Number.isInteger(value) || value < 1 || value > headers.length || used.has(value)) throw Error()
        used.add(value)
        return headers[value - 1]
      }
      onmap({ date_col: read('date_col') ?? '', asset_col: read('asset_col') ?? '', amount_col: read('amount_col') ?? '',
        type_col: read('type_col'), fee_col: read('fee_col'), fee_currency_col: read('fee_currency_col'),
        price_col: read('price_col'), notes_col: read('notes_col') })
      invalid = false
    } catch {
      invalid = true
    }
  }
</script>

<details>
  <summary>{i18n.t('mapping-assistant')}</summary>
  <label>{i18n.t('mapping-prompt')}<textarea readonly value={prompt} rows="5"></textarea></label>
  <label>{i18n.t('mapping-response')}<textarea bind:value={response} maxlength="8192" rows="2"></textarea></label>
  <button type="button" class="secondary-btn" onclick={apply} disabled={!response.trim()}>{i18n.t('mapping-apply')}</button>
  {#if invalid}<p role="alert">{i18n.t('mapping-invalid')}</p>{/if}
</details>

<style>
  summary { cursor: pointer; }
  label { display: grid; gap: .25rem; margin: .65rem 0; }
  textarea { width: 100%; box-sizing: border-box; font: inherit; color: inherit; background: var(--bg-secondary); }
</style>
