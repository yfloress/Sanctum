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

  // Jurisdiction / method / toggles are edited inline on the tax tab; this modal
  // is dedicated to the bulkier wallet-exclusion list.
  interface Props {
    show: boolean
    taxExcludedWalletIds: string[]
    taxLoading: boolean
    wallets: { id: string; name: string; category: string; total_value: string; assets_count: number; icon_path: string | null }[]
    onsave: () => Promise<void>
    onclose: () => void
  }

  let {
    show = $bindable(false),
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

  function toggleWallet(id: string) {
    if (taxExcludedWalletIds.includes(id)) {
      taxExcludedWalletIds = taxExcludedWalletIds.filter(x => x !== id)
    } else {
      taxExcludedWalletIds = [...taxExcludedWalletIds, id]
    }
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{i18n.t('crypto-tax-exclude-wallets', 'Exclude Wallets')}</h3>
      <p class="modal-desc">{i18n.t('crypto-tax-exclude-wallets-desc', 'Wallets you exclude are left out of tax calculations (e.g. DeFi play wallets or donation-only wallets).')}</p>
      <div class="form-grid">
        {#if wallets.length > 0}
          <div class="exclusion-section">
            {#each wallets as w}
              <label class="exclusion-row">
                <input
                  type="checkbox"
                  checked={taxExcludedWalletIds.includes(w.id)}
                  onchange={() => toggleWallet(w.id)}
                />
                <span>{w.name}</span>
              </label>
            {/each}
          </div>
        {:else}
          <p class="empty-note">{i18n.t('crypto-tax-no-wallets', 'No wallets to exclude.')}</p>
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
  .modal-desc { margin: 0 0 12px; font-size: 0.8rem; color: var(--text-secondary); line-height: 1.4; }
  .empty-note { font-size: 0.85rem; color: var(--text-secondary); }
  .exclusion-section { display: flex; flex-direction: column; gap: 6px; margin-top: 4px; }
  .exclusion-row {
    display: flex; align-items: center; gap: 8px; font-size: 0.85rem; color: var(--text-secondary); cursor: pointer;
  }
  .exclusion-row input[type="checkbox"] { width: 16px; height: 16px; cursor: pointer; }
</style>
