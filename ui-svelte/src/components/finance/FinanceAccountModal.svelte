<!-- Sanctum — a privacy-first personal finance and crypto vault.
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
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import * as financeApi from '../../lib/api/finance'
  import { ACCOUNT_ICONS, getDefaultIconPath } from '../../lib/accountDisplay'
  import type { AccountDetailResponse, AccountsResponse } from '../../lib/types'

  interface Props {
    show: boolean
    editing: AccountDetailResponse | null
    accountsData: AccountsResponse | null
    onsubmit: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), editing, accountsData, onsubmit, onclose }: Props = $props()

  let accName = $state('')
  let accType = $state('bank')
  let accCurrency = $state('USD')
  let accInitialBalance = $state('0')
  let accIcon = $state('')
  let showIconPicker = $state(false)
  let pickedIconSrc = $state('')
  let pickedIconGeneric = $state(true)

  $effect(() => {
    const found = accIcon ? ACCOUNT_ICONS.find(i => i.value === accIcon) : null
    pickedIconSrc = found ? found.src : getDefaultIconPath(accType)
    pickedIconGeneric = found ? found.generic : true
  })

  $effect(() => {
    show
    editing
    if (!show) return
    if (editing) {
      accName = editing.name
      accType = editing.account_type === 'credit_card' ? 'credit' : editing.account_type
      accCurrency = editing.currency
      const fullAcc = accountsData?.accounts.find(a => a.id === editing.id)
      accInitialBalance = fullAcc?.initial_balance ?? '0'
      accIcon = ''
      showIconPicker = false
    } else {
      accName = ''
      accType = 'bank'
      accCurrency = 'USD'
      accInitialBalance = '0'
      accIcon = ''
      showIconPicker = false
    }
  })

  async function submit() {
    try {
      const isEditing = !!editing
      if (isEditing) {
        await financeApi.updateAccount(editing!.id, accName, accType, accCurrency, accInitialBalance)
      } else {
        const before = new Set(accountsData?.accounts.map(a => a.id) ?? [])
        await financeApi.createAccount(accName, accType, accCurrency, accInitialBalance)
        if (accIcon) {
          const fresh = await financeApi.fetchAccounts()
          const newAcc = fresh.accounts.find(a => !before.has(a.id))
          if (newAcc) {
            await financeApi.updateAccountIcon(newAcc.id, accIcon)
          }
        }
      }
      show = false
      await onsubmit()
      app.showToast(isEditing
        ? i18n.t('finances-acc-updated', 'Account updated')
        : i18n.t('finances-acc-created', 'Account created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
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
      <h3>{editing ? i18n.t('finances-edit-account-modal', 'Edit Account') : i18n.t('finances-new-account-modal', 'New Account')}</h3>
      <div class="form-grid">
        <label>
          {i18n.t('finances-name', 'Name')}
          <input type="text" bind:value={accName} placeholder={i18n.t('finances-account-name-placeholder', 'Account name')} />
        </label>
        <label>
          {i18n.t('finances-type', 'Type')}
          <select bind:value={accType}>
            <option value="bank">{i18n.t('finances-account-type-bank', 'Bank')}</option>
            <option value="savings">{i18n.t('finances-account-type-savings', 'Savings')}</option>
            <option value="credit">{i18n.t('finances-account-type-credit', 'Credit Card')}</option>
            <option value="cash">{i18n.t('finances-account-type-cash', 'Cash')}</option>
            <option value="other">{i18n.t('finances-account-type-other', 'Other')}</option>
          </select>
        </label>
        <label>
          {i18n.t('finances-currency', 'Currency')}
          <select bind:value={accCurrency}>
            {#each ['USD', 'CLP', 'EUR', 'GBP', 'BRL', 'MXN', 'ARS', 'CAD', 'AUD', 'CHF', 'JPY'] as cur}
              <option value={cur}>{cur}</option>
            {/each}
          </select>
        </label>
        <label>
          {i18n.t('finances-initial-balance', 'Initial Balance')}
          <input type="text" inputmode="decimal" bind:value={accInitialBalance} placeholder="0.00" />
        </label>
        {#if !editing}
          <div class="icon-select-label">
            <span>{i18n.t('finances-icon', 'Icon')}</span>
            <button class="change-icon-btn" onclick={() => showIconPicker = !showIconPicker}>
              <img
                src={pickedIconSrc || getDefaultIconPath(accType)}
                alt=""
                class="selected-icon-preview"
                class:themed-icon={pickedIconGeneric}
              />
              {showIconPicker ? i18n.t('finances-close', 'Close') : i18n.t('finances-change', 'Change')}
            </button>
          </div>
          {#if showIconPicker}
            <div class="icon-picker">
              {#each ACCOUNT_ICONS as icon}
                <button class="icon-option" class:selected={accIcon === icon.value} onclick={() => { accIcon = icon.value; showIconPicker = false }} title={icon.value}>
                  <img src={icon.src} alt={icon.value} class:themed-icon={icon.generic} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
                </button>
              {/each}
            </div>
          {/if}
        {/if}
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('finances-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submit} disabled={!accName.trim()}>
          {editing ? i18n.t('finances-update', 'Update') : i18n.t('finances-create', 'Create')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .icon-select-label {
    display: flex; justify-content: space-between; align-items: center;
    font-size: 0.8rem; color: var(--text-secondary);
  }
  .selected-icon-preview { width: 20px; height: 20px; margin-right: 6px; vertical-align: middle; }
</style>
