import { invoke } from '@tauri-apps/api/core'
import type {
  PortfolioResponse, PortfolioTrendData,
  WalletsResponse, WalletDetailResponse,
  CryptoTransactionDto, CryptoTransactionInput,
  CryptoTransferInput, CryptoSwapInput,
  CryptoTransactionUpdateInput, CryptoTransactionEditData,
  CoinCatalogDto, CryptoAssetPriceDto,
  TaxSettingsDto, TaxReportDto, IpcSummaryDto
} from '../types'

export async function fetchPortfolio(): Promise<PortfolioResponse> {
  return invoke<PortfolioResponse>('fetch_portfolio')
}

export async function fetchPortfolioTrend(days: number = 30): Promise<PortfolioTrendData> {
  return invoke<PortfolioTrendData>('fetch_portfolio_trend', { days })
}

export async function fetchWallets(): Promise<WalletsResponse> {
  return invoke<WalletsResponse>('fetch_wallets')
}

export async function fetchWalletDetail(wallet_id: string): Promise<WalletDetailResponse> {
  return invoke<WalletDetailResponse>('fetch_wallet_detail', { walletId: wallet_id })
}

export async function addWallet(name: string, category: string, icon?: string): Promise<void> {
  return invoke('add_wallet', { name, category, icon: icon ?? null })
}

export async function deleteWallet(id: string, force: boolean): Promise<void> {
  return invoke('delete_wallet', { id, force })
}

export async function getWalletTransactionCount(id: string): Promise<number> {
  return invoke<number>('get_wallet_transaction_count', { id })
}

export async function updateWalletName(id: string, new_name: string): Promise<void> {
  return invoke('update_wallet_name', { id, newName: new_name })
}

export async function updateWalletIcon(id: string, icon: string | null): Promise<void> {
  return invoke('update_wallet_icon', { id, icon })
}

export async function addCryptoTransaction(input: CryptoTransactionInput): Promise<void> {
  return invoke('add_crypto_transaction', { input })
}

export async function addCryptoTransfer(input: CryptoTransferInput): Promise<void> {
  return invoke('add_crypto_transfer', { input })
}

export async function addCryptoSwap(input: CryptoSwapInput): Promise<void> {
  return invoke('add_crypto_swap', { input })
}

export async function updateCryptoTransaction(input: CryptoTransactionUpdateInput): Promise<void> {
  return invoke('update_crypto_transaction', { input })
}

export async function deleteCryptoTransaction(id: string): Promise<void> {
  return invoke('delete_crypto_transaction', { id })
}

export async function getCryptoTransaction(id: string): Promise<CryptoTransactionEditData> {
  return invoke<CryptoTransactionEditData>('get_crypto_transaction', { id })
}

export async function getCryptoTransactionsByCoin(coin_id: string): Promise<CryptoTransactionDto[]> {
  return invoke<CryptoTransactionDto[]>('get_crypto_transactions_by_coin', { coinId: coin_id })
}

export async function getCoinCatalog(): Promise<CoinCatalogDto[]> {
  return invoke<CoinCatalogDto[]>('get_coin_catalog')
}

export async function setFavoriteCoin(id: string, favorite: boolean): Promise<void> {
  return invoke('set_favorite_coin', { id, favorite })
}

export async function addCustomCoin(id: string, name: string, symbol: string): Promise<void> {
  return invoke('add_custom_coin', { id, name, symbol })
}

export async function deleteCustomCoin(id: string): Promise<void> {
  return invoke('delete_custom_coin', { id })
}

export async function getActiveTickerIds(): Promise<string[]> {
  return invoke<string[]>('get_active_ticker_ids')
}

export async function saveActiveTickerIds(ids: string[]): Promise<void> {
  return invoke('save_active_ticker_ids', { ids })
}

export async function saveCryptoPrices(prices: CryptoAssetPriceDto[]): Promise<void> {
  return invoke('save_crypto_prices', { prices })
}

export async function loadCryptoPrices(): Promise<CryptoAssetPriceDto[]> {
  return invoke<CryptoAssetPriceDto[]>('load_crypto_prices')
}

export async function getMonitoredCoinIds(): Promise<string[]> {
  return invoke<string[]>('get_monitored_coin_ids')
}

export async function loadTaxSettings(period_id: string): Promise<TaxSettingsDto> {
  return invoke<TaxSettingsDto>('load_tax_settings', { periodId: period_id })
}

export async function saveTaxSettings(settings: TaxSettingsDto): Promise<void> {
  return invoke('save_tax_settings', { settings })
}

export async function generateTaxReport(period_id: string): Promise<TaxReportDto> {
  return invoke<TaxReportDto>('generate_tax_report', { periodId: period_id })
}

export async function exportTaxReportCsv(period_id: string, path: string): Promise<void> {
  return invoke('export_tax_report_csv', { periodId: period_id, path })
}

export async function exportTaxHistoryCsv(period_id: string, path: string): Promise<void> {
  return invoke('export_tax_history_csv', { periodId: period_id, path })
}

export async function importIpcCsv(content: string): Promise<IpcSummaryDto> {
  return invoke<IpcSummaryDto>('import_ipc_csv', { content })
}

export async function getIpcSummary(): Promise<IpcSummaryDto> {
  return invoke<IpcSummaryDto>('get_ipc_summary')
}

export async function fillMissingTaxPrices(
  tx_id: string, price_per_coin?: number, fee_usd?: number, override_proceeds?: number
): Promise<boolean> {
  return invoke<boolean>('fill_missing_tax_prices', {
    txId: tx_id,
    pricePerCoin: price_per_coin ?? null,
    feeUsd: fee_usd ?? null,
    overrideProceeds: override_proceeds ?? null,
  })
}

export async function saveExchangeRate(pair: string, rate: number): Promise<void> {
  return invoke('save_exchange_rate', { pair, rate })
}

export async function loadExchangeRate(pair: string): Promise<[number, string] | null> {
  return invoke<[number, string] | null>('load_exchange_rate', { pair })
}
