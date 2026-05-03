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

  interface Props {
    show: boolean
    taxJurisdiction: string
    taxMethod: string
    taxIncludeSwaps: boolean
    taxIncludeFeeCrypto: boolean
    taxExcludedWalletIds: string[]
    taxLoading: boolean
    wallets: { id: string; name: string; category: string; total_value: string; assets_count: number; icon_path: string | null }[]
    onsave: () => Promise<void>
    onclose: () => void
  }

  let {
    show = $bindable(false),
    taxJurisdiction = $bindable('usa'),
    taxMethod = $bindable('fifo'),
    taxIncludeSwaps = $bindable(true),
    taxIncludeFeeCrypto = $bindable(false),
    taxExcludedWalletIds = $bindable([]),
    taxLoading,
    wallets,
    onsave,
    onclose,
  }: Props = $props()

  async function handleSave() {
    await onsave()
  }

  function close() {
    show = false
    onclose()
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{i18n.t('crypto-tax-settings-title', 'Tax Settings')}</h3>
      <div class="form-grid">
        <label>
          {i18n.t('crypto-tax-jurisdiction', 'Jurisdiction')}
          <select bind:value={taxJurisdiction}>
            <option value="usa">{i18n.t('crypto-tax-jurisdiction-us', 'United States')}</option>
            <option value="chile">{i18n.t('crypto-tax-jurisdiction-cl', 'Chile')}</option>
            <option value="other">{i18n.t('crypto-tax-jurisdiction-other', 'Other')}</option>
          </select>
        </label>
        <label>
          {i18n.t('crypto-tax-cost-basis-method', 'Cost Basis Method')}
          <select bind:value={taxMethod}>
            <option value="fifo">{i18n.t('crypto-tax-method-fifo', 'FIFO')}</option>
            <option value="lifo">{i18n.t('crypto-tax-method-lifo', 'LIFO')}</option>
            <option value="hifo">{i18n.t('crypto-tax-method-hifo', 'HIFO')}</option>
            <option value="cpp">{i18n.t('crypto-tax-method-avg', 'Average Cost')}</option>
          </select>
        </label>
        <label>
          <input type="checkbox" bind:checked={taxIncludeSwaps} />
          {i18n.t('crypto-tax-include-swaps-label', 'Include Swaps in Disposals')}
        </label>
        <label>
          <input type="checkbox" bind:checked={taxIncludeFeeCrypto} />
          {i18n.t('crypto-tax-include-fee-label', 'Include Fee Crypto as Disposal')}
        </label>
        {#if wallets.length > 0}
          <div class="exclusion-section">
            <span class="exclusion-title">{i18n.t('crypto-tax-exclude-wallets', 'Exclude Wallets')}</span>
            {#each wallets as w}
              <label class="exclusion-row">
                <input
                  type="checkbox"
                  checked={taxExcludedWalletIds.includes(w.id)}
                  onchange={() => {
                    if (taxExcludedWalletIds.includes(w.id)) {
                      taxExcludedWalletIds = taxExcludedWalletIds.filter(x => x !== w.id)
                    } else {
                      taxExcludedWalletIds = [...taxExcludedWalletIds, w.id]
                    }
                  }}
                />
                <span>{w.name}</span>
              </label>
            {/each}
          </div>
        {/if}
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('crypto-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={handleSave} disabled={taxLoading}>{i18n.t('crypto-save', 'Save')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .exclusion-section { display: flex; flex-direction: column; gap: 6px; margin-top: 4px; }
  .exclusion-title { font-size: 0.8rem; color: var(--text-secondary); font-weight: 500; }
  .exclusion-row {
    display: flex; align-items: center; gap: 8px; font-size: 0.85rem; color: var(--text-secondary); cursor: pointer;
  }
  .exclusion-row input[type="checkbox"] { width: 16px; height: 16px; cursor: pointer; }
</style>
