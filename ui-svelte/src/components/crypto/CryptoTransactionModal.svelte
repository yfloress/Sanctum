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
  import { errorMessage } from '../../lib/errors'
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import * as cryptoApi from '../../lib/api/crypto'
  import type { CoinCatalogDto } from '../../lib/types'

  interface Props {
    show: boolean
    wallets: { id: string; name: string }[]
    coinCatalog: CoinCatalogDto[]
    onsubmit: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), wallets, coinCatalog, onsubmit, onclose }: Props = $props()

  let txMode = $state<'buy' | 'sell' | 'income' | 'fee' | 'transfer' | 'swap'>('buy')
  let txWalletId = $state('')
  let txCoinId = $state('')
  let txSymbol = $state('')
  let txAmount = $state('')
  let txPrice = $state('')
  let txFee = $state('0')
  let txDate = $state(new Date().toISOString().slice(0, 10))
  let txNotes = $state('')
  let txFromWalletId = $state('')
  let txToWalletId = $state('')
  let txFromAmount = $state('')
  let txToAmount = $state('')
  let txFromCoinId = $state('')
  let txFromSymbol = $state('')
  let txToCoinId = $state('')
  let txToSymbol = $state('')
  let txSwapFromAmount = $state('')
  let txSwapToAmount = $state('')

  let coinSearch = $state('')
  let fromCoinSearch = $state('')
  let toCoinSearch = $state('')

  let filteredCoins = $derived(
    coinSearch.length < 1 ? coinCatalog.slice(0, 50) :
    coinCatalog.filter(c =>
      c.symbol.toLowerCase().includes(coinSearch.toLowerCase()) ||
      c.name.toLowerCase().includes(coinSearch.toLowerCase())
    ).slice(0, 50)
  )
  let filteredFromCoins = $derived(
    fromCoinSearch.length < 1
      ? coinCatalog.slice(0, 50)
      : coinCatalog.filter(c =>
          c.symbol.toLowerCase().includes(fromCoinSearch.toLowerCase()) ||
          c.name.toLowerCase().includes(fromCoinSearch.toLowerCase())
        ).slice(0, 50)
  )
  let filteredToCoins = $derived(
    toCoinSearch.length < 1
      ? coinCatalog.slice(0, 50)
      : coinCatalog.filter(c =>
          c.symbol.toLowerCase().includes(toCoinSearch.toLowerCase()) ||
          c.name.toLowerCase().includes(toCoinSearch.toLowerCase())
        ).slice(0, 50)
  )

  function resetState() {
    txMode = 'buy'
    txWalletId = wallets[0]?.id ?? ''
    txCoinId = ''
    txSymbol = ''
    txAmount = ''
    txPrice = ''
    txFee = '0'
    txDate = new Date().toISOString().slice(0, 10)
    txNotes = ''
    txFromWalletId = wallets[0]?.id ?? ''
    txToWalletId = wallets[1]?.id ?? wallets[0]?.id ?? ''
    txFromAmount = ''
    txToAmount = ''
    txFromCoinId = ''
    txFromSymbol = ''
    txToCoinId = ''
    txToSymbol = ''
    txSwapFromAmount = ''
    txSwapToAmount = ''
    coinSearch = ''
    fromCoinSearch = ''
    toCoinSearch = ''
  }

  $effect(() => {
    wallets
    show
    if (show) resetState()
  })

  function selectCoin(coin: CoinCatalogDto) {
    txCoinId = coin.id
    txSymbol = coin.symbol
    coinSearch = coin.symbol
  }

  function selectFromCoin(coin: CoinCatalogDto) {
    txFromCoinId = coin.id
    txFromSymbol = coin.symbol
    fromCoinSearch = coin.symbol
  }

  function selectToCoin(coin: CoinCatalogDto) {
    txToCoinId = coin.id
    txToSymbol = coin.symbol
    toCoinSearch = coin.symbol
  }

  async function submitCryptoTransaction() {
    try {
      if (txMode === 'transfer') {
        await cryptoApi.addCryptoTransfer({
          from_wallet_id: txFromWalletId,
          to_wallet_id: txToWalletId,
          coin_id: txCoinId,
          symbol: txSymbol,
          from_amount: txFromAmount,
          to_amount: txToAmount || txFromAmount,
          fee: txFee,
          date: txDate,
          notes: txNotes || undefined,
        })
      } else if (txMode === 'swap') {
        await cryptoApi.addCryptoSwap({
          wallet_id: txWalletId,
          from_coin_id: txFromCoinId,
          from_symbol: txFromSymbol,
          from_amount: txSwapFromAmount,
          to_coin_id: txToCoinId,
          to_symbol: txToSymbol,
          to_amount: txSwapToAmount,
          fee: txFee,
          date: txDate,
          notes: txNotes || undefined,
        })
      } else {
        const backendType =
          txMode === 'buy' ? 'trade' :
          txMode === 'sell' ? 'trade' :
          txMode === 'fee' ? 'expense' : txMode
        const subtype = txMode === 'fee' ? 'fee' : (txMode === 'buy' || txMode === 'sell') ? txMode : undefined
        await cryptoApi.addCryptoTransaction({
          wallet_id: txWalletId,
          coin_id: txCoinId,
          symbol: txSymbol,
          transaction_type: backendType,
          subtype,
          amount: txAmount,
          price: txPrice,
          fee: txFee,
          date: txDate,
          notes: txNotes || undefined,
        })
      }
      show = false
      await onsubmit()
      app.showToast(i18n.t('crypto-toast-tx-added', 'Transaction added'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
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
    <div class="modal wide">
      <h3>{i18n.t('crypto-tx-title', 'New Transaction')}</h3>
      <div class="tx-type-bar">
        {#each (['buy', 'sell', 'income', 'fee', 'transfer', 'swap'] as const) as t}
          <button class="tx-type-btn" class:active={txMode === t} onclick={() => txMode = t}>
            {i18n.t(`crypto-tx-${t}`, t.charAt(0).toUpperCase() + t.slice(1))}
          </button>
        {/each}
      </div>

      <div class="form-grid">
        {#if txMode === 'transfer'}
          <label>
            {i18n.t('crypto-tx-coin', 'Coin')}
            <div class="coin-search-wrap">
              <input type="text" bind:value={coinSearch} placeholder={i18n.t('crypto-tx-search-coin', 'Search coin...')} />
              {#if txCoinId}
                <button class="clear-coin" onclick={() => { txCoinId = ''; txSymbol = ''; coinSearch = '' }}>x</button>
              {/if}
            </div>
            {#if coinSearch.length >= 1 && coinSearch !== txSymbol}
              <div class="coin-dropdown">
                {#each filteredCoins as c}
                  <button class="coin-option" onclick={() => selectCoin(c)}>{c.symbol} - {c.name}</button>
                {/each}
              </div>
            {/if}
          </label>
          <label>
            {i18n.t('crypto-tx-from-wallet', 'From Wallet')}
            <select bind:value={txFromWalletId}>
              {#each wallets as w}
                <option value={w.id}>{w.name}</option>
              {/each}
            </select>
          </label>
          <label>
            {i18n.t('crypto-tx-to-wallet', 'To Wallet')}
            <select bind:value={txToWalletId}>
              {#each wallets as w}
                <option value={w.id}>{w.name}</option>
              {/each}
            </select>
          </label>
          <label>
            {i18n.t('crypto-tx-amount', 'Amount')}
            <input type="text" bind:value={txFromAmount} placeholder="0.00" />
          </label>
          <label>
            {i18n.t('crypto-tx-received-amount', 'Received Amount (optional)')}
            <input type="text" bind:value={txToAmount} placeholder={i18n.t('crypto-tx-received-placeholder', 'Same as sent if empty')} />
          </label>
        {:else if txMode === 'swap'}
          <label>
            {i18n.t('crypto-tx-wallet', 'Wallet')}
            <select bind:value={txWalletId}>
              {#each wallets as w}
                <option value={w.id}>{w.name}</option>
              {/each}
            </select>
          </label>
          <label>
            {i18n.t('crypto-tx-from-coin', 'From Coin')}
            <div class="coin-search-wrap">
              <input type="text" bind:value={fromCoinSearch} placeholder={i18n.t('crypto-tx-search-coin', 'Search coin...')} />
              {#if txFromCoinId}
                <button class="clear-coin" onclick={() => { txFromCoinId = ''; txFromSymbol = ''; fromCoinSearch = '' }}>x</button>
              {/if}
            </div>
            {#if fromCoinSearch.length >= 1 && fromCoinSearch !== txFromSymbol}
              <div class="coin-dropdown">
                {#each filteredFromCoins as c}
                  <button class="coin-option" onclick={() => selectFromCoin(c)}>{c.symbol} - {c.name}</button>
                {/each}
              </div>
            {/if}
          </label>
          <label>
            {i18n.t('crypto-tx-from-amount', 'From Amount')}
            <input type="text" bind:value={txSwapFromAmount} placeholder="0.00" />
          </label>
          <label>
            {i18n.t('crypto-tx-to-coin', 'To Coin')}
            <div class="coin-search-wrap">
              <input type="text" bind:value={toCoinSearch} placeholder={i18n.t('crypto-tx-search-coin', 'Search coin...')} />
              {#if txToCoinId}
                <button class="clear-coin" onclick={() => { txToCoinId = ''; txToSymbol = ''; toCoinSearch = '' }}>x</button>
              {/if}
            </div>
            {#if toCoinSearch.length >= 1 && toCoinSearch !== txToSymbol}
              <div class="coin-dropdown">
                {#each filteredToCoins as c}
                  <button class="coin-option" onclick={() => selectToCoin(c)}>{c.symbol} - {c.name}</button>
                {/each}
              </div>
            {/if}
          </label>
          <label>
            {i18n.t('crypto-tx-to-amount', 'To Amount')}
            <input type="text" bind:value={txSwapToAmount} placeholder="0.00" />
          </label>
        {:else}
          <label>
            {i18n.t('crypto-tx-wallet', 'Wallet')}
            <select bind:value={txWalletId}>
              {#each wallets as w}
                <option value={w.id}>{w.name}</option>
              {/each}
            </select>
          </label>
          <label>
            {i18n.t('crypto-tx-coin', 'Coin')}
            <div class="coin-search-wrap">
              <input type="text" bind:value={coinSearch} placeholder={i18n.t('crypto-tx-search-coin', 'Search coin...')} />
              {#if txCoinId}
                <button class="clear-coin" onclick={() => { txCoinId = ''; txSymbol = ''; coinSearch = '' }}>x</button>
              {/if}
            </div>
            {#if coinSearch.length >= 1 && coinSearch !== txSymbol}
              <div class="coin-dropdown">
                {#each filteredCoins as c}
                  <button class="coin-option" onclick={() => selectCoin(c)}>{c.symbol} - {c.name}</button>
                {/each}
              </div>
            {/if}
          </label>
          <label>
            {i18n.t('crypto-tx-amount', 'Amount')}
            <input type="text" bind:value={txAmount} placeholder="0.00" />
          </label>
          <label>
            {i18n.t('crypto-tx-price', 'Price (per coin)')}
            <input type="text" bind:value={txPrice} placeholder="0.00" />
          </label>
        {/if}

        <label>
          {i18n.t('crypto-tx-fee-label', 'Fee')}
          <input type="text" bind:value={txFee} placeholder="0" />
        </label>
        <label>
          {i18n.t('crypto-tx-date', 'Date')}
          <input type="date" bind:value={txDate} />
        </label>
        <label>
          {i18n.t('crypto-tx-notes', 'Notes (optional)')}
          <input type="text" bind:value={txNotes} placeholder={i18n.t('crypto-tx-notes-placeholder', 'Notes...')} />
        </label>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" onclick={close}>{i18n.t('crypto-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitCryptoTransaction}>{i18n.t('crypto-tx-add', 'Add')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .tx-type-bar { display: flex; gap: 4px; margin-bottom: 16px; flex-wrap: wrap; }
  .tx-type-btn {
    flex: 1; min-width: 60px; padding: 8px 4px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.8rem; text-align: center;
    transition: all 0.15s;
  }
  .tx-type-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .tx-type-btn.active {
    background: var(--glass-active); color: var(--text-primary);
    border-color: var(--accent-border); box-shadow: 0 0 0 1px var(--accent-glow) inset;
  }
  .coin-search-wrap { position: relative; display: flex; align-items: center; }
  .coin-search-wrap input { flex: 1; }
  .coin-search-wrap .clear-coin { position: absolute; right: 8px; }
  .clear-coin {
    background: none; border: none; color: var(--text-tertiary); cursor: pointer; font-size: 0.9rem;
    margin-left: auto; padding: 0 4px; transition: color 0.15s;
  }
  .clear-coin:hover { color: var(--text-primary); }
  .coin-dropdown {
    max-height: 150px; overflow-y: auto; border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm); background: rgba(20, 20, 24, 0.95); margin-top: 4px;
  }
  .coin-option {
    display: block; width: 100%; padding: 8px 12px; background: none; border: none;
    color: var(--text-secondary); cursor: pointer; font-size: 0.8rem; text-align: left;
    transition: background 0.1s;
  }
  .coin-option:hover { background: var(--glass-hover); color: var(--text-primary); }
</style>
