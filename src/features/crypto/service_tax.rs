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

//! Crypto tax/reporting service extensions.

use super::service::{
    CryptoError, CryptoService, SETTING_CRYPTO_TAX_IPC_DATA, SETTING_CRYPTO_TAX_IPC_UPDATED,
    SETTING_CRYPTO_TAX_SETTINGS,
};
use super::tax::{
    build_import_summary, build_tax_report, map_to_entries, parse_ipc_csv, summarize_ipc,
    IpcEntry, IpcImportSummary, IpcSummary, TaxPeriodSettings, TaxReport, TaxSettingsStore,
};
use chrono::Local;

impl CryptoService {
    // ==================== Tax: IPC Import (Offline) ====================

    pub fn import_ipc_csv(&self, content: &str) -> Result<IpcImportSummary, CryptoError> {
        let parsed = parse_ipc_csv(content).map_err(CryptoError::Validation)?;

        if parsed.entries.is_empty() {
            return Err(CryptoError::Validation(
                "No valid IPC rows found in CSV".to_string(),
            ));
        }

        let summary = build_import_summary(&parsed);
        let entries = map_to_entries(parsed.entries);
        let json = serde_json::to_string(&entries)
            .map_err(|e| CryptoError::Validation(format!("IPC serialization failed: {}", e)))?;
        let updated_at = Local::now().to_rfc3339();

        self.with_db(|db| {
            db.set_setting(SETTING_CRYPTO_TAX_IPC_DATA, &json)
                .map_err(CryptoError::Database)?;
            db.set_setting(SETTING_CRYPTO_TAX_IPC_UPDATED, &updated_at)
                .map_err(CryptoError::Database)?;
            Ok(())
        })?;

        Ok(summary)
    }

    pub fn get_ipc_summary(&self) -> Result<Option<IpcSummary>, CryptoError> {
        self.with_db(|db| {
            let entries = load_ipc_entries(db)?;
            if entries.is_empty() {
                return Ok(None);
            }

            let updated_at = db
                .get_setting(SETTING_CRYPTO_TAX_IPC_UPDATED)
                .map_err(CryptoError::Database)?;

            Ok(summarize_ipc(entries.as_slice(), updated_at))
        })
    }

    // ==================== Tax: Settings (Per Period) ====================

    pub fn load_tax_settings(
        &self,
        period_id: String,
    ) -> Result<TaxPeriodSettings, CryptoError> {
        let period_id = period_id.trim().to_string();
        self.with_db(|db| {
            let raw = db
                .get_setting(SETTING_CRYPTO_TAX_SETTINGS)
                .map_err(CryptoError::Database)?
                .unwrap_or_default();

            if raw.trim().is_empty() {
                return Ok(TaxPeriodSettings::defaults_for(&period_id));
            }

            let store: TaxSettingsStore = serde_json::from_str(&raw).unwrap_or_default();
            Ok(store
                .get_for_period(&period_id)
                .unwrap_or_else(|| TaxPeriodSettings::defaults_for(&period_id)))
        })
    }

    pub fn save_tax_settings(&self, settings: TaxPeriodSettings) -> Result<(), CryptoError> {
        let mut settings = settings;
        settings.period_id = settings.period_id.trim().to_string();
        self.with_db(|db| {
            let raw = db
                .get_setting(SETTING_CRYPTO_TAX_SETTINGS)
                .map_err(CryptoError::Database)?
                .unwrap_or_default();

            let mut store: TaxSettingsStore = serde_json::from_str(&raw).unwrap_or_default();
            store.upsert(settings);
            let json = serde_json::to_string(&store)
                .map_err(|e| CryptoError::Validation(format!("Tax settings invalid: {}", e)))?;
            db.set_setting(SETTING_CRYPTO_TAX_SETTINGS, &json)
                .map_err(CryptoError::Database)?;
            Ok(())
        })
    }

    // ==================== Tax: Report Generation ====================

    pub fn generate_tax_report(&self, period_id: String) -> Result<TaxReport, CryptoError> {
        let period_id = period_id.trim().to_string();
        if period_id.is_empty() {
            return Err(CryptoError::Validation(
                "Tax period is required".to_string(),
            ));
        }

        let settings = self.load_tax_settings(period_id.clone())?;

        self.with_db(|db| {
            let transactions = db.get_all_crypto_transactions().map_err(CryptoError::Database)?;
            let ipc_entries = load_ipc_entries(db)?;

            build_tax_report(transactions, settings, ipc_entries)
                .map_err(CryptoError::Validation)
        })
    }

    pub fn export_tax_report_csv(
        &self,
        period_id: String,
        path: &str,
    ) -> Result<(), CryptoError> {
        let report = self.generate_tax_report(period_id)?;
        let csv = report.to_csv();
        std::fs::write(path, csv)
            .map_err(|e| CryptoError::Validation(format!("Failed to write report: {}", e)))?;
        Ok(())
    }
}

fn load_ipc_entries(db: &crate::db::Database) -> Result<Vec<IpcEntry>, CryptoError> {
    let raw = db
        .get_setting(SETTING_CRYPTO_TAX_IPC_DATA)
        .map_err(CryptoError::Database)?
        .unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let entries: Vec<IpcEntry> = serde_json::from_str(&raw)
        .map_err(|e| CryptoError::Validation(format!("IPC data invalid: {}", e)))?;
    Ok(entries)
}
