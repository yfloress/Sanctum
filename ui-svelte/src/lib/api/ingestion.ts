import { invoke } from '@tauri-apps/api/core'
import type {
  ImportPreviewResponse, ImportResultsResponse,
  ExchangeDetectionResult
} from '../types'

export async function previewImport(content: string): Promise<ImportPreviewResponse> {
  return invoke<ImportPreviewResponse>('preview_import', { content })
}

export async function importData(content: string): Promise<ImportResultsResponse> {
  return invoke<ImportResultsResponse>('import_data', { content })
}

export async function maxImportFileSize(): Promise<number> {
  return invoke<number>('max_import_file_size')
}

export async function detectExchangeSource(content: string): Promise<ExchangeDetectionResult | null> {
  return invoke<ExchangeDetectionResult | null>('detect_exchange_source', { content })
}

export async function previewExchangeCsv(
  exchangeId: string, walletName: string, content: string
): Promise<ImportPreviewResponse> {
  return invoke<ImportPreviewResponse>('preview_exchange_csv', { exchangeId, walletName, content })
}

export async function importExchangeCsv(
  exchangeId: string, walletName: string, content: string
): Promise<ImportResultsResponse> {
  return invoke<ImportResultsResponse>('import_exchange_csv', { exchangeId, walletName, content })
}
