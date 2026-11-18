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
  import type { CryptoTransactionDto } from '../../lib/types'
  import PrivateAmount from '../PrivateAmount.svelte'

  let { tx, onedit, ondeleteTx }: {
    tx: CryptoTransactionDto
    onedit: (id: string) => void
    ondeleteTx: (id: string) => void
  } = $props()
</script>

<div class="movement">
  <span class="tx-date">{tx.date}</span>
  <span class="tx-type">{tx.transaction_type === 'transfer' && tx.subtype === 'deposit'
    ? i18n.t('crypto-history-deposit', 'Deposit')
    : tx.transaction_type === 'transfer' && tx.subtype === 'withdrawal'
      ? i18n.t('crypto-history-withdrawal', 'Withdrawal')
      : i18n.t('crypto-type-' + tx.transaction_type, tx.transaction_type)}</span>
  <span class="tx-amount"><PrivateAmount value={tx.amount} unit={tx.symbol} /></span>
  <div class="panel-tx-actions">
    <button class="icon-btn-mini" onclick={() => onedit(tx.id)} aria-label={i18n.t('crypto-edit', 'Edit')} title={i18n.t('crypto-edit', 'Edit')}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
    </button>
    <button class="delete-btn" onclick={() => ondeleteTx(tx.id)} aria-label={i18n.t('crypto-delete', 'Delete')} title={i18n.t('crypto-delete', 'Delete')}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
    </button>
  </div>
</div>

<style>
  .movement {
    display: grid; grid-template-columns: auto minmax(0, 1fr) auto auto;
    gap: .4rem .6rem; align-items: center; padding: .5rem 0;
    border-bottom: 1px solid var(--glass-border);
    font-size: .9rem;
  }
  .tx-amount { overflow-wrap: anywhere; text-align: end; }
  .panel-tx-actions button { min-width: 26px; min-height: 26px; justify-content: center; }
  @container (max-width: 24rem) {
    .movement { grid-template-columns: minmax(0, 1fr) auto; }
    .tx-date { grid-column: 1; grid-row: 2; }
    .tx-type { grid-column: 1; grid-row: 1; }
    .panel-tx-actions { justify-self: end; }
  }
</style>
