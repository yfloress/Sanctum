export interface ImportResultsResponse {
  total_processed: number
  inserted: number
  skipped: number
  errors: ImportErrorDto[]
}

export interface ImportErrorDto {
  line: number | null
  message: string
}

export interface ExchangeDetectionResult {
  exchange_id: string
  exchange: string
  suggested_wallet: string
  file_count: number
  total_records: number
}
