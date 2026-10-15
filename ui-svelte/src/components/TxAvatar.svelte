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

<!-- Money in, money out or moved between accounts, readable at a glance. -->
<script lang="ts">
  interface Props {
    isExpense: boolean
    isTransfer: boolean
  }

  let { isExpense, isTransfer }: Props = $props()
</script>

<span class="tx-avatar" class:expense={isExpense && !isTransfer} class:transfer={isTransfer} aria-hidden="true">
  {#if isTransfer}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7 7h11l-3-3M17 17H6l3 3"/></svg>
  {:else if isExpense}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7 17 17 7M8 7h9v9"/></svg>
  {:else}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 7 7 17M16 17H7V8"/></svg>
  {/if}
</span>

<style>
  .tx-avatar {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    color: var(--success);
    background: color-mix(in srgb, var(--success-vivid) 14%, transparent);
  }
  .tx-avatar svg { width: 15px; height: 15px; }
  .tx-avatar.expense {
    color: var(--danger);
    background: color-mix(in srgb, var(--danger-vivid) 14%, transparent);
  }
  .tx-avatar.transfer {
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 14%, transparent);
  }
</style>
