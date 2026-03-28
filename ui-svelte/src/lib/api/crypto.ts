import { invoke } from '@tauri-apps/api/core'
import type {
  PortfolioResponse, PortfolioTrendData,
  WalletsResponse, WalletDetailResponse, WalletInput,
  CryptoTransactionDto, CryptoTransactionInput,
  CryptoTransferInput, CryptoSwapInput,
  CryptoTransactionUpdateInput, CryptoTransactionEditData,
  CoinCatalogDto, CryptoAssetPriceDto,
  TaxSettingsDto, TaxReportDto, IpcSummaryDto
} from '../types'

export async function fetchPortfolio(): Promise<PortfolioResponse> {
  return invoke<PortfolioResponse>('fetch_portfolio')
}

export async function fetchPortfolioTrend(): Promise<PortfolioTrendData> {
  return invoke<PortfolioTrendData>('fetch_portfolio_trend')
}

export async function fetchWallets(): Promise<WalletsResponse> {
  return invoke<WalletsResponse>('fetch_wallets')
}

export async function fetchWalletDetail(id: string): Promise<WalletDetailResponse> {
  return invoke<WalletDetailResponse>('fetch_wallet_detail', { id })
}

export async function addWallet(input: WalletInput): Promise<void> {
  return invoke('add_wallet', { ...input })
}

export async function deleteWallet(id: string, force: boolean): Promise<void> {
  return invoke('delete_wallet', { id, force })
}

export async function getWalletTransactionCount(id: string): Promise<number> {
  return invoke<number>('get_wallet_transaction_count', { id })
}

export async function updateWalletName(id: string, newName: string): Promise<void> {
  return invoke('update_wallet_name', { id, newName })
}

export async function updateWalletIcon(id: string, icon: string): Promise<void> {
  return invoke('update_wallet_icon', { id, icon })
}

export async function addCryptoTransaction(input: CryptoTransactionInput): Promise<void> {
  return invoke('add_crypto_transaction', { ...input })
}

export async function addCryptoTransfer(input: CryptoTransferInput): Promise<void> {
  return invoke('add_crypto_transfer', { ...input })
}

export async function addCryptoSwap(input: CryptoSwapInput): Promise<void> {
  return invoke('add_crypto_swap', { ...input })
}

export async function updateCryptoTransaction(input: CryptoTransactionUpdateInput): Promise<void> {
  return invoke('update_crypto_transaction', { ...input })
}

export async function deleteCryptoTransaction(id: string): Promise<void> {
  return invoke('delete_crypto_transaction', { id })
}

export async function getCryptoTransaction(id: string): Promise<CryptoTransactionEditData> {
  return invoke<CryptoTransactionEditData>('get_crypto_transaction', { id })
}

export async function getCryptoTransactionsByCoin(coinId: string): Promise<CryptoTransactionDto[]> {
  return invoke<CryptoTransactionDto[]>('get_crypto_transactions_by_coin', { coinId })
}

export async function getCoinCatalog(): Promise<CoinCatalogDto[]> {
  return invoke<CoinCatalogDto[]>('get_coin_catalog')
}

export async function setFavoriteCoin(coinId: string, favorite: boolean): Promise<void> {
  return invoke('set_favorite_coin', { coinId, favorite })
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

export async function loadTaxSettings(periodId: string): Promise<TaxSettingsDto> {
  return invoke<TaxSettingsDto>('load_tax_settings', { periodId })
}

export async function saveTaxSettings(settings: TaxSettingsDto): Promise<void> {
  return invoke('save_tax_settings', { ...settings })
}

export async function generateTaxReport(periodId: string): Promise<TaxReportDto> {
  return invoke<TaxReportDto>('generate_tax_report', { periodId })
}

export async function exportTaxReportCsv(periodId: string): Promise<string> {
  return invoke<string>('export_tax_report_csv', { periodId })
}

export async function exportTaxHistoryCsv(): Promise<string> {
  return invoke<string>('export_tax_history_csv')
}

export async function importIpcCsv(content: string): Promise<number> {
  return invoke<number>('import_ipc_csv', { content })
}

export async function getIpcSummary(): Promise<IpcSummaryDto> {
  return invoke<IpcSummaryDto>('get_ipc_summary')
}

export async function fillMissingTaxPrices(periodId: string): Promise<number> {
  return invoke<number>('fill_missing_tax_prices', { periodId })
}

export async function saveExchangeRate(pair: string, rate: number): Promise<void> {
  return invoke('save_exchange_rate', { pair, rate })
}

export async function loadExchangeRate(pair: string): Promise<[number, string] | null> {
  return invoke<[number, string] | null>('load_exchange_rate', { pair })
}
