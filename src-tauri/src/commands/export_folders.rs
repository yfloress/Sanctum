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

use sanctum::{
    error::AppError,
    features::ingestion::IngestionService,
    ui::dto::sources::{ExportFolderDto, FolderUpdateDto},
};
use tauri::State;

#[tauri::command]
pub fn list_export_folders(
    service: State<'_, IngestionService>,
) -> Result<Vec<ExportFolderDto>, AppError> {
    Ok(service.export_folders()?)
}

#[tauri::command]
pub async fn choose_export_folder(
    app: tauri::AppHandle,
    service: State<'_, IngestionService>,
    wallet_id: String,
    provider: String,
) -> Result<(), AppError> {
    #[cfg(desktop)]
    {
        use tauri_plugin_dialog::DialogExt;
        let path = app.dialog().file().blocking_pick_folder();
        if let Some(tauri_plugin_dialog::FilePath::Path(path)) = path {
            service.set_export_folder(wallet_id, provider, Some(path))?;
        }
        Ok(())
    }
    #[cfg(not(desktop))]
    {
        let _ = (app, service, wallet_id, provider);
        Err(sanctum::features::ingestion::IngestionError::Validation(
            "Export folders require desktop".into(),
        )
        .into())
    }
}

#[tauri::command]
pub fn remove_export_folder(
    service: State<'_, IngestionService>,
    wallet_id: String,
    provider: String,
) -> Result<(), AppError> {
    Ok(service.set_export_folder(wallet_id, provider, None)?)
}

#[tauri::command]
pub async fn update_export_folders(app: tauri::AppHandle) -> Result<FolderUpdateDto, AppError> {
    use tauri::Manager;
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<IngestionService>().update_export_folders()
    })
    .await
    .map_err(|_| {
        sanctum::features::ingestion::IngestionError::Parse("Folder update interrupted".into())
    })?
    .map_err(Into::into)
}
