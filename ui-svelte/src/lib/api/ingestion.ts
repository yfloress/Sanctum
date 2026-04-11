// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

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
  return invoke<ImportResultsResponse>('preview_exchange_csv', { content, walletName: wallet_name })
}

export async function importExchangeCsv(
  content: string, wallet_name: string
): Promise<ImportResultsResponse> {
  return invoke<ImportResultsResponse>('import_exchange_csv', { content, walletName: wallet_name })
}
