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
  import { onMount } from 'svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import { errorMessage } from '../../lib/errors'
  import type { WalletDto } from '../../lib/types/crypto'
  import { listExportFolders, chooseExportFolder, removeExportFolder, updateExportFolders, type ExportFolder, type FolderUpdate } from '../../lib/api/exportFolders'

  let { wallets, onupdated }: {wallets: WalletDto[]; onupdated: () => void} = $props()
  let folders = $state<ExportFolder[]>([])
  let walletId = $state('')
  let provider = $state('feather')
  let busy = $state(false)
  let error = $state('')
  let result = $state<FolderUpdate | null>(null)
  const formats: Record<string, string> = {feather:'Feather',monero_gui:'Monero GUI',kraken:'Kraken',binance:'Binance',notbank:'NotBank',mexc:'MEXC'}
  async function perform(action: () => Promise<void>) {
    if (busy) return
    busy = true
    error = ''
    try {
      await action()
      folders = await listExportFolders()
    } catch (e) { error = errorMessage(e) }
    finally { busy = false }
  }
  onMount(() => { void perform(async () => {}) })
  async function scan() {
    result = await updateExportFolders()
    if (result.inserted) onupdated()
  }
</script>

<p>{i18n.t('crypto-folders-help', 'Choose a dedicated folder for one wallet and format. New CSV exports are imported locally after unlock.')}</p>
{#each folders as folder}
  <div>
    <strong>{wallets.find(w => w.id === folder.wallet_id)?.name ?? '—'} · {formats[folder.provider]}</strong>
    <span class="path">{folder.path}</span>
    <button class="secondary-btn" disabled={busy} onclick={() => perform(() => removeExportFolder(folder.wallet_id, folder.provider))}>{i18n.t('crypto-folders-remove','Disconnect')}</button>
  </div>
{/each}
<label>{i18n.t('crypto-folders-wallet','Wallet')}
  <select bind:value={walletId}><option value="">—</option>{#each wallets as wallet}<option value={wallet.id}>{wallet.name}</option>{/each}</select>
</label>
<label>{i18n.t('crypto-folders-format','CSV format')}
  <select bind:value={provider}>{#each Object.entries(formats) as [id, name]}<option value={id}>{name}</option>{/each}</select>
</label>
<button class="glass-btn" disabled={busy || !walletId} onclick={() => perform(() => chooseExportFolder(walletId, provider))}>{i18n.t('crypto-folders-add','Choose folder')}</button>
{#if folders.length}
  <button class="glass-btn" disabled={busy} onclick={() => perform(scan)}>{i18n.t('crypto-folders-scan','Import new files')}</button>
{/if}
{#if result}
  <p role="status">{i18n.t('crypto-folders-count','Imported movements')} · {result.inserted}</p>
  {#if result.issues.length}<p>{i18n.t('crypto-folders-review','Review these files in Import history')}</p><ul>{#each result.issues as issue}<li>{issue}</li>{/each}</ul>{/if}
{/if}
{#if error}<p role="alert">{error}</p>{/if}
<style>
  label, .path { display: block; margin: .4rem 0; }
  .path { overflow-wrap: anywhere; }
</style>
