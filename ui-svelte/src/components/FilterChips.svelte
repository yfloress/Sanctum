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

<!-- The filters in force, each removable on its own, so a filtered list never
     passes for the whole one. -->
<script module lang="ts">
  export interface FilterChip {
    label: string
    onremove: () => void
  }
</script>

<script lang="ts">
  import { i18n } from '../lib/stores/i18n.svelte'

  let { chips }: { chips: FilterChip[] } = $props()
</script>

{#if chips.length > 0}
  <div class="filter-chips">
    {#each chips as chip (chip.label)}
      <span class="filter-chip">
        {chip.label}
        <button onclick={chip.onremove} aria-label={i18n.t('filters-remove', 'Remove filter')}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12"/></svg>
        </button>
      </span>
    {/each}
  </div>
{/if}

<style>
  .filter-chips { display: flex; flex-wrap: wrap; gap: 6px; margin: -4px 0 14px; }
  .filter-chip {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 3px 4px 3px 10px; border-radius: 20px;
    font-size: 0.78rem; color: var(--text-primary);
    background: var(--accent-bg); border: 1px solid var(--accent-border);
  }
  .filter-chip button {
    display: grid; place-items: center; width: 18px; height: 18px; padding: 0;
    border: none; border-radius: 50%; background: none; cursor: pointer;
    color: var(--text-secondary);
  }
  .filter-chip button:hover { background: var(--glass-active); color: var(--text-primary); }
  .filter-chip svg { width: 11px; height: 11px; }
</style>
