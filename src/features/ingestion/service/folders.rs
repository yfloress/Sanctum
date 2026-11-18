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

use super::{IngestionError, IngestionService};
use crate::db::Database;
use crate::features::ingestion::{
    MAX_FILE_SIZE,
    parsers::{detect_exchange_source, parser_for},
};
use crate::ui::dto::sources::{ExportFolderDto, FolderUpdateDto};
use ring::digest::{SHA256, digest};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
};

const FOLDERS: &str = "crypto_export_folders_v1";
static UPDATE: Mutex<()> = Mutex::new(());

#[derive(Serialize, Deserialize)]
struct ExportFolder {
    path: PathBuf,
    wallet_id: String,
    provider: String,
    imported: BTreeSet<String>,
}

fn fail(message: &str) -> IngestionError {
    IngestionError::Validation(message.to_owned())
}

fn read_folders(db: &Database) -> Result<Vec<ExportFolder>, IngestionError> {
    match db.get_setting(FOLDERS)? {
        Some(value) => {
            serde_json::from_str(&value).map_err(|_| fail("Invalid export folder configuration"))
        }
        None => Ok(Vec::new()),
    }
}

fn save(db: &Database, folders: &[ExportFolder]) -> Result<(), IngestionError> {
    let value = serde_json::to_string(folders)
        .map_err(|_| fail("Cannot save export folder configuration"))?;
    db.set_setting(FOLDERS, &value)?;
    Ok(())
}

impl IngestionService {
    pub fn export_folders(&self) -> Result<Vec<ExportFolderDto>, IngestionError> {
        self.with_db_readonly(|db| {
            Ok(read_folders(db)?
                .into_iter()
                .map(|f| ExportFolderDto {
                    wallet_id: f.wallet_id,
                    provider: f.provider,
                    path: f.path.to_string_lossy().into_owned(),
                })
                .collect())
        })
    }

    pub fn set_export_folder(
        &self,
        wallet_id: String,
        provider: String,
        directory: Option<PathBuf>,
    ) -> Result<(), IngestionError> {
        if !matches!(
            provider.as_str(),
            "feather" | "monero_gui" | "kraken" | "binance" | "mexc" | "notbank"
        ) {
            return Err(fail("Unknown export format"));
        }
        let _guard = UPDATE
            .lock()
            .map_err(|_| fail("Folder update unavailable"))?;
        self.with_db(|db| {
            let mut folders = read_folders(db)?;
            if let Some(path) = directory {
                let path = path
                    .canonicalize()
                    .map_err(|_| fail("Cannot open selected folder"))?;
                if !path.is_dir() || db.get_wallet(&wallet_id)?.is_none() {
                    return Err(fail("Select an existing folder and wallet"));
                }
                if folders
                    .iter()
                    .any(|f| f.provider == provider && f.path == path && f.wallet_id != wallet_id)
                {
                    return Err(fail(
                        "Folder already linked to another wallet for this format",
                    ));
                }
                if folders
                    .iter()
                    .any(|f| f.wallet_id == wallet_id && f.provider == provider && f.path == path)
                {
                    return Ok(());
                }
                folders.retain(|f| f.wallet_id != wallet_id || f.provider != provider);
                folders.push(ExportFolder {
                    path,
                    wallet_id,
                    provider,
                    imported: BTreeSet::new(),
                });
            } else {
                folders.retain(|f| f.wallet_id != wallet_id || f.provider != provider);
            }
            save(db, &folders)
        })
    }

    pub fn update_export_folders(&self) -> Result<FolderUpdateDto, IngestionError> {
        let _guard = UPDATE
            .lock()
            .map_err(|_| fail("Folder update unavailable"))?;
        self.with_db_readonly(|db| {
            let mut folders = read_folders(db)?;
            let mut result = FolderUpdateDto::default();
            for index in 0..folders.len() {
                let folder = &folders[index];
                let Some(wallet) = db.get_wallet(&folder.wallet_id)? else {
                    result.issues.push(folder.path.display().to_string());
                    continue;
                };
                if !crate::features::ingestion::IngestionRepository::build_wallet_lookup(db)?
                    .get(&wallet.name.trim().to_lowercase())
                    .is_some_and(|w| w.id == wallet.id)
                {
                    result.issues.push(folder.path.display().to_string());
                    continue;
                }
                let Ok(entries) = fs::read_dir(&folder.path) else {
                    result.issues.push(folder.path.display().to_string());
                    continue;
                };
                let mut paths = Vec::new();
                for entry in entries {
                    match entry {
                        Ok(e)
                            if e.file_type().is_ok_and(|t| t.is_file())
                                && e.path()
                                    .extension()
                                    .is_some_and(|s| s.eq_ignore_ascii_case("csv")) =>
                        {
                            paths.push(e.path())
                        }
                        Err(_) => result.issues.push(folder.path.display().to_string()),
                        _ => {}
                    }
                }
                paths.sort();
                for path in paths {
                    db.check_session_timeout_readonly()?;
                    match self.import_folder_file(
                        db,
                        &folders[index],
                        &path,
                        &wallet.name,
                        &mut result,
                    ) {
                        Ok(Some(hash)) => {
                            folders[index].imported.insert(hash);
                            save(db, &folders)?;
                        }
                        Ok(None) => {}
                        Err(_) => result.issues.push(
                            path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned(),
                        ),
                    }
                }
            }
            Ok(result)
        })
    }

    fn import_folder_file(
        &self,
        db: &Database,
        folder: &ExportFolder,
        path: &Path,
        wallet: &str,
        result: &mut FolderUpdateDto,
    ) -> Result<Option<String>, IngestionError> {
        let file = fs::File::open(path).map_err(|_| fail("Cannot open export"))?;
        let mut bytes = Vec::new();
        file.take(MAX_FILE_SIZE as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| fail("Cannot read export"))?;
        if bytes.len() > MAX_FILE_SIZE {
            return Err(fail("Export too large"));
        }
        let hash = digest(&SHA256, &bytes)
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if folder.imported.contains(&hash) {
            return Ok(None);
        }
        let content = std::str::from_utf8(&bytes)
            .map_err(|_| fail("Invalid CSV encoding"))?
            .trim_start_matches('\u{feff}');
        let source = detect_exchange_source(content).ok_or_else(|| fail("Unrecognized CSV"))?;
        let id = source.id();
        if id != folder.provider
            && !id
                .strip_prefix(folder.provider.as_str())
                .is_some_and(|suffix| suffix.starts_with('_'))
        {
            return Err(fail("Different CSV format"));
        }
        let parsed = parser_for(source)
            .parse(content, wallet)
            .map_err(|_| fail("Invalid CSV"))?;
        if !parsed.errors.is_empty() {
            return Err(fail("Review CSV in batch import"));
        }
        let summary = self.process_crypto_transactions_with_db(
            db,
            parsed.items,
            source.label(),
            false,
            true,
        )?;
        result.inserted += summary.inserted;
        if summary
            .skipped_reasons
            .iter()
            .any(|r| r != &crate::services::i18n::t("import-skipped-duplicate-crypto"))
            || summary.errors != 0
            || !summary.missing_wallets.is_empty()
        {
            return Err(fail("Review import"));
        }
        Ok(Some(hash))
    }
}
