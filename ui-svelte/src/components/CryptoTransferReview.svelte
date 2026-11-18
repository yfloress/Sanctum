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
  import PrivateAmount from './PrivateAmount.svelte'
  import { invoke } from '../lib/api/ipc'
  import { errorMessage } from '../lib/errors'
  import { i18n } from '../lib/stores/i18n.svelte'

  interface Pair {
    outgoing_id: string
    incoming_id: string
    from_wallet: string
    to_wallet: string
    sent_at: string
    received_at: string
    outgoing_notes: string | null
    incoming_notes: string | null
    symbol: string
    sent: number
    received: number
    linked: boolean
  }

  let { onchanged }: { onchanged: () => Promise<void> } = $props()
  let review = $state<{ rows: Pair[]; more_suggestions: boolean } | null>(null)
  let busy = $state(false)
  let error = $state('')

  async function load() {
    review = await invoke('list_transfer_reviews')
  }

  async function refresh() {
    if (busy) return
    busy = true
    error = ''
    try { await load() }
    catch (e) { error = errorMessage(e) }
    finally { busy = false }
  }

  async function change(pair: Pair) {
    if (busy) return
    busy = true
    error = ''
    try {
      await invoke('set_transfer_link', { outgoing_id: pair.outgoing_id, incoming_id: pair.incoming_id, linked: !pair.linked })
      await load()
      await onchanged()
    } catch (e) {
      error = errorMessage(e)
    } finally {
      busy = false
    }
  }
</script>

<details ontoggle={e => { if (e.currentTarget.open) void refresh() }}>
  <summary>{i18n.t('crypto-transfer-review', 'Review transfers between my wallets')}</summary>
  <div class="review" aria-busy={busy}>
    {#if error}<p role="alert">{error}</p><button class="glass-btn" disabled={busy} onclick={refresh}>{i18n.t('crypto-transfer-refresh', 'Refresh')}</button>{/if}
    {#if busy}<p role="status">{i18n.t('settings-import-loading', 'Loading...')}</p>{/if}
    {#if review}
      <p>{i18n.t('crypto-transfer-match-note', 'Suggestions have equal amounts in the same asset within three days. Confirm only transfers you recognize.')}</p>
      {#each review.rows as pair (pair.outgoing_id + ':' + pair.incoming_id)}
        <div class="pair">
          <span>{pair.sent_at} · {pair.from_wallet} ·  <PrivateAmount value={'−' + pair.sent} unit={pair.symbol} /></span>
          <span>{pair.received_at} · {pair.to_wallet} ·  <PrivateAmount value={'+' + pair.received} unit={pair.symbol} /></span>
          <details><summary>{i18n.t('crypto-transfer-ids', 'Movement identifiers')}</summary>
            <div><code>{pair.outgoing_id}</code> {pair.outgoing_notes ?? ''}</div>
            <div><code>{pair.incoming_id}</code> {pair.incoming_notes ?? ''}</div>
          </details>
          <button class="glass-btn" disabled={busy} onclick={() => change(pair)}>
            {pair.linked ? i18n.t('crypto-transfer-unlink', 'Unlink movements') : i18n.t('crypto-transfer-link', 'Confirm own transfer')}
          </button>
        </div>
      {:else}
        <p>{i18n.t('crypto-transfer-no-matches', 'No matching movements found.')}</p>
      {/each}
      {#if review.more_suggestions}<p>{i18n.t('crypto-transfer-more', 'Showing the first 200 suggestions. Confirm matches to see more.')}</p>{/if}
    {/if}
  </div>
</details>

<style>
  summary { cursor: pointer; }
  .review, .pair { display: grid; gap: .5rem; padding: .5rem 0; }
  .pair { border-top: 1px solid var(--glass-border); }
  .pair button { justify-self: start; }
  code { overflow-wrap: anywhere; }
  p { margin: 0; }
</style>
