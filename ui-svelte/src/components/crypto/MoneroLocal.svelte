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
  import { invoke } from '../../lib/api/ipc'
  import { previewExchangeCsv, importExchangeCsv } from '../../lib/api/ingestion'
  import { errorMessage } from '../../lib/errors'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import type { WalletDto } from '../../lib/types/crypto'
  import type { ImportResultsResponse } from '../../lib/types/ingestion'
  let { wallets, onupdated }: {wallets: WalletDto[]; onupdated: () => void} = $props()
  let walletId = $state('')
  let address = $state('')
  let user = $state('')
  let password = $state('')
  let busy = $state(false)
  let csv = $state('')
  let summary = $state<ImportResultsResponse | null>(null)
  let message = $state('')
  async function read() {
    busy = true
    message = ''
    try {
      csv = await invoke<string>('read_monero_history', { address, user, password })
      const wallet = wallets.find(w => w.id === walletId)
      if (!wallet) throw new Error('Select a wallet')
      summary = await previewExchangeCsv(csv,wallet.name)
    } catch (e) { message = errorMessage(e); csv = ''; summary = null }
    finally { password = ''; busy = false }
  }
  async function apply() {
    const wallet = wallets.find(w => w.id === walletId)
    if (!wallet || !summary) return
    busy = true
    try {
      const result = await importExchangeCsv(csv,wallet.name)
      message = i18n.t('crypto-folders-count','Imported movements') + ' · ' + result.inserted
      if (result.errors.length) message += ' · ' + result.errors[0].message
      csv = ''; summary = null; onupdated()
    } catch (e) { message = errorMessage(e) }
    finally { busy = false }
  }
</script>

<p>{i18n.t('crypto-monero-guide','Read account 0 of an authenticated local monero-wallet-rpc at 127.0.0.1:18083. Requires curl. Includes transfers with at least 10 confirmations.')}</p>
<fieldset disabled={busy || !!summary}>
  <label>{i18n.t('crypto-folders-wallet','Wallet')}<select bind:value={walletId}><option value="">—</option>{#each wallets as wallet}<option value={wallet.id}>{wallet.name}</option>{/each}</select></label>
  <label>{i18n.t('crypto-monero-address','Primary wallet address')}<input bind:value={address} /></label>
  <label>{i18n.t('crypto-monero-user','RPC username')}<input bind:value={user} autocomplete="off" /></label>
  <label>{i18n.t('crypto-monero-pass','RPC password')}<input type="password" bind:value={password} autocomplete="off" /></label>
  <button class="glass-btn" disabled={!walletId || !address.trim() || !user || !password} onclick={read}>{i18n.t('crypto-monero-read','Preview movements')}</button>
</fieldset>
{#if summary}
  <p>{i18n.t('crypto-monero-new','New movements')} · {summary.inserted}</p>
  {#each summary.errors.slice(0,5) as problem}<p>{problem.message}</p>{/each}
  <button disabled={busy || summary.errors.length > 0 || !summary.inserted} onclick={apply}>{i18n.t('crypto-monero-import','Import')}</button>
  <button disabled={busy} onclick={() => { csv = ''; summary = null }}>{i18n.t('crypto-close','Close')}</button>
{/if}
{#if message}<p role="status">{message}</p>{/if}
<style>label { display:block; margin:.4rem 0 } fieldset { border:0; padding:0 } </style>
