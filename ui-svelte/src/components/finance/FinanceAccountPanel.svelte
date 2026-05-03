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
  import type { AccountDetailResponse } from '../../lib/types'

  const ACCOUNT_ICONS: { value: string; src: string; generic: boolean }[] = [
    ...['banco-chile', 'banco-estado', 'bank-of-america', 'bci', 'citibank', 'jpmorgan', 'mercado_pago', 'santander', 'wf']
      .map(n => ({ value: `${n}.svg`, src: `/src/assets/bank-icons/${n}.svg`, generic: false })),
    ...['landmark', 'wallet', 'credit-card', 'piggy-bank', 'briefcase', 'coins', 'banknote', 'building-2']
      .map(n => ({ value: `/src/assets/icons/${n}.svg`, src: `/src/assets/icons/${n}.svg`, generic: true })),
  ]

  function getDefaultIconPath(accountType: string): string {
    const map: Record<string, string> = {
      bank: 'landmark', savings: 'piggy-bank', credit: 'credit-card', cash: 'wallet',
    }
    const icon = map[accountType] || 'wallet'
    return `/src/assets/icons/${icon}.svg`
  }

  function isGenericIcon(iconPath: string | null): boolean {
    if (!iconPath) return true
    return iconPath.startsWith('/src/assets/icons/')
  }

  function getAccountDisplayIcon(acc: { account_type: string; icon_path: string | null }): string {
    if (acc.icon_path) {
      if (acc.icon_path.startsWith('/') || acc.icon_path.startsWith('http')) return acc.icon_path
      return `/src/assets/bank-icons/${acc.icon_path}`
    }
    return getDefaultIconPath(acc.account_type)
  }

  interface Props {
    show: boolean
    account: AccountDetailResponse | null
    ondelete: (id: string) => Promise<void>
    onedit: (detail: AccountDetailResponse) => void
    onrefresh: () => Promise<void>
    oniconchange: (icon: string) => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), account, ondelete, onedit, onrefresh, oniconchange, onclose }: Props = $props()

  let showIconPicker = $state(false)

  async function changeAccountIcon(icon: string) {
    if (!account) return
    await oniconchange(icon)
    showIconPicker = false
    await onrefresh()
  }

  function close() {
    show = false
    onclose()
  }
</script>

{#if show && account}
  <div class="overlay-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <aside class="detail-panel">
    <div class="panel-header">
      <h3>{account.name}</h3>
      <button class="close-panel" aria-label="Close panel" onclick={close}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
    <div class="panel-info">
      <div class="info-row panel-icon-row">
        <img src={getAccountDisplayIcon(account)} alt="" class="panel-acc-icon" class:themed-icon={isGenericIcon(account.icon_path)} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
        <button class="change-icon-btn" onclick={() => showIconPicker = !showIconPicker}>
          {showIconPicker ? i18n.t('finances-close', 'Close') : i18n.t('finances-change-icon', 'Change Icon')}
        </button>
      </div>
      {#if showIconPicker}
        <div class="icon-picker">
          {#each ACCOUNT_ICONS as icon}
            <button class="icon-option" onclick={() => changeAccountIcon(icon.value)} title={icon.value}>
              <img src={icon.src} alt={icon.value} class:themed-icon={icon.generic} onerror={(e) => (e.target as HTMLImageElement).style.display='none'} />
            </button>
          {/each}
          <button class="icon-option icon-reset" onclick={() => changeAccountIcon('')} title="Default">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 12l9-9 9 9M5 10v9a1 1 0 001 1h4v-5h4v5h4a1 1 0 001-1v-9"/></svg>
          </button>
        </div>
      {/if}
      <div class="info-row"><span>{i18n.t('finances-type', 'Type')}</span><span>{account.account_type}</span></div>
      <div class="info-row"><span>{i18n.t('finances-currency', 'Currency')}</span><span>{account.currency}</span></div>
      <div class="info-row">
        <span>{i18n.t('finances-balance', 'Balance')}</span>
        <span class:negative={account.balance_negative}>{account.balance}</span>
      </div>
    </div>
    {#if account.transactions.length > 0}
      <h4>{i18n.t('finances-recent-transactions', 'Recent Transactions')}</h4>
      <div class="panel-tx-list">
        {#each account.transactions as tx}
          <div class="panel-tx">
            <span class="tx-date">{tx.date}</span>
            <span class="tx-desc">{tx.description}</span>
            <span class="tx-amount" class:expense={tx.is_expense}>{tx.amount}</span>
          </div>
        {/each}
      </div>
    {/if}
    <div class="panel-actions">
      <button class="primary-btn" onclick={() => onedit(account)}>{i18n.t('finances-edit-account', 'Edit Account')}</button>
      <button class="danger-btn" onclick={() => ondelete(account.id)}>{i18n.t('finances-delete-account', 'Delete Account')}</button>
    </div>
  </aside>
{/if}

<style>
  .panel-info { display: flex; flex-direction: column; gap: 8px; margin-bottom: 20px; }

  .panel-icon-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
  .panel-acc-icon { width: 32px; height: 32px; border-radius: 6px; }

  .panel-tx-list { display: flex; flex-direction: column; gap: 4px; }

  .tx-desc { color: var(--text-secondary); }
  .tx-amount.expense { color: var(--danger); }
</style>
