import { invoke } from '@tauri-apps/api/core'
import type {
  ImportResultsResponse,
  ExchangeDetectionResult
} from '../types'

export async function previewImport(content: string, filename: string): Promise<ImportResultsResponse> {
  return invoke<ImportResultsResponse>('preview_import', { content, filename })
}

export async function importData(content: string, filename: string): Promise<ImportResultsResponse> {
  return invoke<ImportResultsResponse>('import_data', { content, filename })
}

export async function maxImportFileSize(): Promise<number> {
  return invoke<number>('max_import_file_size')
}

export async function detectExchangeSource(content: string): Promise<ExchangeDetectionResult | null> {
  return invoke<ExchangeDetectionResult | null>('detect_exchange_source', { content })
}

export async function previewExchangeCsv(
  content: string, wallet_name: string
): Promise<ImportResultsResponse> {
  return invoke<ImportResultsResponse>('preview_exchange_csv', { content, wallet_name })
}

export async function importExchangeCsv(
  content: string, wallet_name: string
): Promise<ImportResultsResponse> {
  return invoke<ImportResultsResponse>('import_exchange_csv', { content, wallet_name })
}
