<!--
Sanctum — a privacy-first personal finance and crypto vault.
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
along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.

-->
<script lang="ts">
  import { onMount } from 'svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import { errorMessage } from '../../lib/errors'
  import type { WalletDto } from '../../lib/types/crypto'
  import { configureKraken, disconnectKraken, krakenStatus, syncKraken, type KrakenSource } from '../../lib/api/krakenSync'

  let { wallets, onupdated }: { wallets: WalletDto[]; onupdated: () => void } = $props()
  let source = $state<KrakenSource | null>(null)
  let walletId = $state('')
  let key = $state('')
  let secret = $state('')
  let onUnlock = $state(false)
  let busy = $state(false)
  let error = $state('')
  let message = $state('')

  onMount(() => { void refresh() })

  async function refresh() {
    try {
      source = await krakenStatus()
    } catch (e) {
      error = errorMessage(e)
    }
  }

  async function act(action: 'configure' | 'sync' | 'disconnect') {
    if (busy) return
    busy = true
    error = ''
    message = ''
    try {
      if (action === 'configure') {
        await configureKraken(walletId, key, secret, onUnlock)
        key = ''
        secret = ''
      } else if (action === 'disconnect') {
        await disconnectKraken()
        onUnlock = false
      } else {
        const result = await syncKraken()
        message = i18n.tArgs('kraken-sync-result', { count: result.inserted, duplicates: result.duplicates }, '{ $count } imported, { $duplicates } duplicates')
        onupdated()
      }
      await refresh()
    } catch (e) {
      error = errorMessage(e)
    } finally {
      busy = false
    }
  }
</script>

<details>
  <summary>Kraken</summary>
  {#if source?.wallet_id}
    <p>{wallets.find(w => w.id === source?.wallet_id)?.name ?? i18n.t('kraken-wallet-missing', 'Wallet unavailable')}</p>
    {#if source.synced_through}
      <p>{i18n.t('kraken-synced-through', 'Updated through')} {new Date(source.synced_through * 1000).toLocaleString()}</p>
    {/if}
    <button class="glass-btn" disabled={busy} onclick={() => act('sync')}>{i18n.t('kraken-sync-now', 'Update ledger')}</button>
    <button class="glass-btn" disabled={busy} onclick={() => act('disconnect')}>{i18n.t('kraken-disconnect', 'Disconnect')}</button>
  {:else if source}
    <p>{i18n.t('kraken-key-help', 'Create a dedicated Kraken API key with only Query ledger entries enabled. Credentials are stored in your encrypted vault.')}</p>
    <form onsubmit={(event) => { event.preventDefault(); void act('configure') }}>
      <label>{i18n.t('kraken-wallet', 'Destination wallet')}
        <select required bind:value={walletId}>
          <option value="">{i18n.t('kraken-select-wallet', 'Select a wallet')}</option>
          {#each wallets as wallet (wallet.id)}<option value={wallet.id}>{wallet.name}</option>{/each}
        </select>
      </label>
      <label>API key<input required type="password" bind:value={key} autocomplete="off" maxlength="512" /></label>
      <label>API secret<input required type="password" bind:value={secret} autocomplete="off" maxlength="512" /></label>
      <label><input type="checkbox" bind:checked={onUnlock} /> {i18n.t('kraken-sync-on-unlock', 'Update when I unlock this vault')}</label>
      <button class="glass-btn" disabled={busy || !walletId || !key || !secret}>{i18n.t('kraken-connect', 'Save connection')}</button>
    </form>
  {/if}
  {#if busy}<p role="status">{i18n.t('kraken-working', 'Working…')}</p>{/if}
  {#if message}<p role="status">{message}</p>{/if}
  {#if error}<p role="alert">{error}</p>{/if}
</details>

<style>
  form { display: grid; gap: 0.7rem; }
  summary { cursor: pointer; }
  label { display: grid; gap: 0.25rem; }
  label:has(input[type="checkbox"]) { display: block; }
</style>
