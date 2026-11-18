// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
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

use sanctum::error::AppError;
use sanctum::features::crypto::CryptoService;
use sanctum::features::ingestion::IngestionService;
use sanctum::ui::dto::kraken_sync::{KrakenSyncResult, KrakenSyncStatus};
use secrecy::SecretString;
use tauri::State;

#[tauri::command]
pub fn kraken_sync_status(
    ingestion: State<'_, IngestionService>,
) -> Result<KrakenSyncStatus, AppError> {
    Ok(ingestion.kraken_status()?)
}

#[tauri::command(rename_all = "snake_case")]
pub fn configure_kraken_sync(
    ingestion: State<'_, IngestionService>,
    wallet_id: String,
    api_key: String,
    api_secret: String,
    on_unlock: bool,
) -> Result<(), AppError> {
    Ok(ingestion.configure_kraken(
        wallet_id,
        SecretString::from(api_key),
        SecretString::from(api_secret),
        on_unlock,
    )?)
}

#[tauri::command]
pub fn disconnect_kraken_sync(ingestion: State<'_, IngestionService>) -> Result<(), AppError> {
    Ok(ingestion.disconnect_kraken()?)
}

#[tauri::command]
pub async fn run_kraken_sync(
    ingestion: State<'_, IngestionService>,
    crypto: State<'_, CryptoService>,
    automatic: bool,
) -> Result<KrakenSyncResult, AppError> {
    let result = ingestion.sync_kraken(automatic).await?;
    if !automatic && result.inserted > 0 {
        let _ = crypto.backfill_portfolio_snapshots();
    }
    Ok(result)
}
