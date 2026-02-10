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
    SETTING_CRYPTO_TAX_SETTINGS, SETTING_PREFERRED_CURRENCY,
};
use super::tax::{
    IpcEntry, IpcImportSummary, IpcSummary, TaxJurisdiction, TaxPeriodSettings, TaxReadinessItem,
    TaxReport, TaxSettingsStore, TaxSummaryPayload, TaxTxType, build_import_summary,
    build_tax_report, map_to_entries, parse_ipc_csv, resolve_tax_type, summarize_ipc,
};
use crate::core::csv_escape;
use crate::features::crypto::tax::engine::{TaxPeriod, is_in_period, parse_date, parse_period};
use crate::models::CryptoTransaction;
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

    pub fn load_tax_settings(&self, period_id: String) -> Result<TaxPeriodSettings, CryptoError> {
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
        let excluded = settings.excluded_wallet_ids.clone();

        self.with_db(|db| {
            let transactions: Vec<CryptoTransaction> = db
                .get_all_crypto_transactions()
                .map_err(CryptoError::Database)?
                .into_iter()
                .filter(|tx| !excluded.contains(&tx.wallet_id))
                .collect();
            let ipc_entries = load_ipc_entries(db)?;

            build_tax_report(transactions, settings, ipc_entries).map_err(CryptoError::Validation)
        })
    }

    pub fn generate_tax_summary(
        &self,
        period_id: String,
    ) -> Result<TaxSummaryPayload, CryptoError> {
        let period_id = period_id.trim().to_string();
        if period_id.is_empty() {
            return Err(CryptoError::Validation(
                "Tax period is required".to_string(),
            ));
        }

        let settings = self.load_tax_settings(period_id.clone())?;
        let jurisdiction = settings.jurisdiction;
        let excluded = settings.excluded_wallet_ids.clone();
        let period = parse_period(&period_id).map_err(CryptoError::Validation)?;

        self.with_db(|db| {
            let transactions: Vec<CryptoTransaction> = db
                .get_all_crypto_transactions()
                .map_err(CryptoError::Database)?
                .into_iter()
                .filter(|tx| !excluded.contains(&tx.wallet_id))
                .collect();
            let ipc_entries = load_ipc_entries(db)?;

            // Compute income and period stats from the borrowed slice BEFORE moving
            // transactions into build_tax_report (which takes ownership).
            let (income_total, income_count, income_warnings) =
                compute_taxable_income(&transactions, &period);

            let transactions_in_period = transactions
                .iter()
                .filter(|tx| {
                    parse_date(&tx.date)
                        .map(|date| is_in_period(&period, date))
                        .unwrap_or(false)
                })
                .count();

            let unpaired_transfers = count_unpaired_transfers(&transactions);

            let (end_balance_value, end_balance_missing) =
                compute_end_balance(db, &transactions, period.end)?;

            let mut report = build_tax_report(transactions, settings, ipc_entries)
                .map_err(CryptoError::Validation)?;

            report.warnings.extend(income_warnings);

            let readiness = build_readiness(
                &report,
                transactions_in_period,
                end_balance_missing,
                unpaired_transfers,
                jurisdiction,
            );

            let volume_processed = report.summary.total_proceeds;

            Ok(TaxSummaryPayload {
                report,
                taxable_income_total: income_total,
                taxable_income_count: income_count,
                end_balance_value,
                end_balance_missing,
                transactions_in_period,
                volume_processed,
                readiness,
            })
        })
    }

    pub fn export_tax_report_csv(&self, period_id: String, path: &str) -> Result<(), CryptoError> {
        let report = self.generate_tax_report(period_id)?;
        let (currency, clp_rate) = self.resolve_export_currency()?;
        let csv = report.to_csv_with_currency(&currency, clp_rate);
        std::fs::write(path, csv)
            .map_err(|e| CryptoError::Validation(format!("Failed to write report: {}", e)))?;
        Ok(())
    }

    pub fn export_tax_history_csv(&self, period_id: String, path: &str) -> Result<(), CryptoError> {
        let period_id = period_id.trim().to_string();
        if period_id.is_empty() {
            return Err(CryptoError::Validation(
                "Tax period is required".to_string(),
            ));
        }

        let settings = self.load_tax_settings(period_id.clone())?;
        let excluded = settings.excluded_wallet_ids;
        let period = parse_period(&period_id).map_err(CryptoError::Validation)?;
        let (currency, clp_rate) = self.resolve_export_currency()?;

        self.with_db(|db| {
            let transactions: Vec<CryptoTransaction> = db
                .get_all_crypto_transactions()
                .map_err(CryptoError::Database)?
                .into_iter()
                .filter(|tx| !excluded.contains(&tx.wallet_id))
                .collect();
            let csv = build_transaction_history_csv(&transactions, &period, &currency, clp_rate);
            std::fs::write(path, csv)
                .map_err(|e| CryptoError::Validation(format!("Failed to write history: {}", e)))?;
            Ok(())
        })
    }

    fn resolve_export_currency(&self) -> Result<(String, f64), CryptoError> {
        let preferred = self
            .get_app_setting(SETTING_PREFERRED_CURRENCY)
            .unwrap_or_else(|_| "USD".to_string())
            .to_uppercase();

        if preferred != "USD" {
            let pair = format!("{}_USD", preferred.as_str());
            let rate = self.with_db(|db| {
                let rate = db
                    .load_exchange_rate(&pair)
                    .map_err(CryptoError::Database)?
                    .map(|(value, _)| value)
                    .unwrap_or(0.0);
                Ok(rate)
            })?;

            if rate <= 0.0 {
                return Err(CryptoError::Validation(
                    format!(
                        "{} export requires a valid USD/{} rate. Please sync prices first.",
                        preferred, preferred
                    ),
                ));
            }
            return Ok((preferred, rate));
        }

        Ok(("USD".to_string(), 0.0))
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

fn compute_taxable_income(
    transactions: &[CryptoTransaction],
    period: &TaxPeriod,
) -> (f64, usize, Vec<crate::features::crypto::TaxWarning>) {
    let mut total = 0.0;
    let mut count = 0;
    let mut warnings = Vec::new();

    for tx in transactions {
        let Some(date) = parse_date(&tx.date) else {
            continue;
        };
        if !is_in_period(period, date) {
            continue;
        }

        let tax_type = resolve_tax_type(tx);
        if tax_type != TaxTxType::Income {
            continue;
        }

        count += 1;
        if let Some(value) = tx.override_proceeds {
            total += value;
            continue;
        }
        if let Some(price) = tx.price_per_coin {
            total += price * tx.amount;
        } else {
            warnings.push(crate::features::crypto::TaxWarning {
                code: "income_missing_price".to_string(),
                message: format!("Missing price for income {}", tx.id),
                tx_id: Some(tx.id.clone()),
            });
        }
    }

    (total, count, warnings)
}

fn compute_end_balance(
    db: &crate::db::Database,
    transactions: &[CryptoTransaction],
    period_end: chrono::NaiveDate,
) -> Result<(Option<f64>, usize), CryptoError> {
    let filtered: Vec<CryptoTransaction> = transactions
        .iter()
        .filter(|tx| {
            parse_date(&tx.date)
                .map(|date| date <= period_end)
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    let assets = crate::db::Database::aggregate_crypto_transactions(filtered);
    let mut missing = 0usize;
    let mut total = 0.0;

    for asset in assets {
        if asset.total_amount <= 0.0 {
            continue;
        }
        match db
            .load_crypto_price(&asset.coin_id)
            .map_err(CryptoError::Database)?
        {
            Some((price, _)) => {
                total += asset.total_amount * price;
            }
            None => {
                missing += 1;
            }
        }
    }

    let value = if missing == 0 { Some(total) } else { None };
    Ok((value, missing))
}

fn count_unpaired_transfers(transactions: &[CryptoTransaction]) -> usize {
    transactions
        .iter()
        .filter(|tx| {
            let mech = tx.mechanical_type();
            (mech == "transfer_in" || mech == "transfer_out") && tx.related_tx_id.is_none()
        })
        .count()
}

fn build_readiness(
    report: &TaxReport,
    transactions_in_period: usize,
    end_balance_missing: usize,
    unpaired_transfers: usize,
    jurisdiction: TaxJurisdiction,
) -> Vec<TaxReadinessItem> {
    let warning_codes: std::collections::HashSet<&str> =
        report.warnings.iter().map(|w| w.code.as_str()).collect();

    let has_invalid =
        warning_codes.contains("invalid_date") || warning_codes.contains("invalid_type");

    let missing_price_codes = [
        "missing_price",
        "fee_missing_price",
        "swap_missing_price",
        "income_missing_price",
        "ipc_missing",
    ];
    let has_missing_prices = missing_price_codes
        .iter()
        .any(|c| warning_codes.contains(c))
        || end_balance_missing > 0;

    let has_insufficient = warning_codes.contains("insufficient_lots");

    let missing_price_count = report
        .warnings
        .iter()
        .filter(|w| missing_price_codes.contains(&w.code.as_str()))
        .count()
        + end_balance_missing;

    let insufficient_count = report
        .warnings
        .iter()
        .filter(|w| w.code == "insufficient_lots")
        .count();

    vec![
        TaxReadinessItem {
            code: "settings".to_string(),
            status: if transactions_in_period > 0 {
                "ok"
            } else {
                "warn"
            }
            .to_string(),
            detail: transactions_in_period.to_string(),
        },
        TaxReadinessItem {
            code: "history".to_string(),
            status: if has_insufficient { "warn" } else { "ok" }.to_string(),
            detail: insufficient_count.to_string(),
        },
        TaxReadinessItem {
            code: "balances".to_string(),
            status: if end_balance_missing > 0 {
                "warn"
            } else {
                "ok"
            }
            .to_string(),
            detail: end_balance_missing.to_string(),
        },
        TaxReadinessItem {
            code: "prices".to_string(),
            status: if has_invalid {
                "error"
            } else if has_missing_prices {
                "warn"
            } else {
                "ok"
            }
            .to_string(),
            detail: if has_invalid {
                "invalid".to_string()
            } else {
                missing_price_count.to_string()
            },
        },
        TaxReadinessItem {
            code: "transfers".to_string(),
            status: if unpaired_transfers > 0 { "warn" } else { "ok" }.to_string(),
            detail: unpaired_transfers.to_string(),
        },
        // Chile-specific: F22 casilla guidance
        if matches!(jurisdiction, TaxJurisdiction::Chile) {
            let has_gain = report.summary.total_gain > 0.0;
            let has_loss = report.summary.total_gain < 0.0;
            TaxReadinessItem {
                code: "sii_f22".to_string(),
                status: "info".to_string(),
                detail: if has_gain {
                    "gain".to_string()
                } else if has_loss {
                    "loss".to_string()
                } else {
                    "neutral".to_string()
                },
            }
        } else if matches!(jurisdiction, TaxJurisdiction::Usa) {
            TaxReadinessItem {
                code: "filing".to_string(),
                status: "info".to_string(),
                detail: "usa".to_string(),
            }
        } else {
            TaxReadinessItem {
                code: "filing".to_string(),
                status: "info".to_string(),
                detail: "other".to_string(),
            }
        },
    ]
}

fn convert_usd_for_export(value: f64, currency: &str, clp_rate: f64) -> f64 {
    if currency != "USD" {
        value * clp_rate
    } else {
        value
    }
}

fn build_transaction_history_csv(
    transactions: &[CryptoTransaction],
    period: &TaxPeriod,
    currency: &str,
    clp_rate: f64,
) -> String {
    let mut out = String::new();
    out.push_str("tx_id,date,coin_id,symbol,type,subtype,mechanical_type,amount,price_per_coin,fee,fee_coin_id,fee_amount,override_proceeds,override_cost_basis,fiat_currency,notes,related_tx_id\n");

    for tx in transactions {
        let Some(date) = parse_date(&tx.date) else {
            continue;
        };
        if !is_in_period(period, date) {
            continue;
        }

        let subtype_str = tx.subtype.as_deref().unwrap_or("");
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{:.8},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&tx.id),
            csv_escape(&tx.date),
            csv_escape(&tx.coin_id),
            csv_escape(&tx.symbol),
            csv_escape(&tx.transaction_type),
            csv_escape(subtype_str),
            csv_escape(tx.mechanical_type()),
            tx.amount,
            tx.price_per_coin
                .map(|v| format!("{:.8}", convert_usd_for_export(v, currency, clp_rate)))
                .unwrap_or_default(),
            tx.fee
                .map(|v| format!("{:.8}", convert_usd_for_export(v, currency, clp_rate)))
                .unwrap_or_default(),
            csv_escape(tx.fee_coin_id.as_deref().unwrap_or("")),
            tx.fee_amount
                .map(|v| format!("{:.8}", v))
                .unwrap_or_default(),
            tx.override_proceeds
                .map(|v| format!("{:.8}", convert_usd_for_export(v, currency, clp_rate)))
                .unwrap_or_default(),
            tx.override_cost_basis
                .map(|v| format!("{:.8}", convert_usd_for_export(v, currency, clp_rate)))
                .unwrap_or_default(),
            csv_escape(currency),
            csv_escape(tx.notes.as_deref().unwrap_or("")),
            csv_escape(tx.related_tx_id.as_deref().unwrap_or("")),
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::features::crypto::TaxReportSummary;
    use secrecy::SecretString;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    struct TestServiceHarness {
        service: CryptoService,
        test_dir: PathBuf,
    }

    impl Drop for TestServiceHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.test_dir);
        }
    }

    fn new_test_service() -> TestServiceHarness {
        let base_dir = std::env::temp_dir().join(format!("sanctum-tax-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&base_dir).expect("create test dir");
        let db_path = base_dir.join("vault.db");
        let password = SecretString::from("test-password-123".to_string());
        let db = Database::init(db_path, &password).expect("init test database");
        let service = CryptoService::new(Arc::new(Mutex::new(Some(db))));
        TestServiceHarness {
            service,
            test_dir: base_dir,
        }
    }

    fn tx(
        id: &str,
        tx_type: &str,
        subtype: Option<&str>,
        amount: f64,
        price: Option<f64>,
        date: &str,
    ) -> CryptoTransaction {
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet-1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            tx_type.to_string(),
            amount,
            price,
            None,
            date.to_string(),
            None,
        );
        tx.subtype = subtype.map(str::to_string);
        tx
    }

    #[test]
    fn compute_taxable_income_uses_override_proceeds_priority() {
        let period = parse_period("2024").expect("valid period");

        let income_with_price = tx("i1", "income", Some("airdrop"), 1.0, Some(100.0), "2024-03-10");
        let mut income_with_override =
            tx("i2", "income", Some("reward"), 2.0, Some(200.0), "2024-05-10");
        income_with_override.override_proceeds = Some(50.0);
        let trade = tx("t1", "trade", Some("buy"), 1.0, Some(999.0), "2024-06-10");
        let outside_period = tx("i3", "income", Some("staking"), 1.0, Some(10.0), "2023-12-31");

        let (total, count, warnings) = compute_taxable_income(
            &[income_with_price, income_with_override, trade, outside_period],
            &period,
        );

        assert!((total - 150.0).abs() < 0.0001);
        assert_eq!(count, 2);
        assert!(warnings.is_empty());
    }

    #[test]
    fn compute_taxable_income_warns_when_price_missing() {
        let period = parse_period("2024").expect("valid period");
        let income_missing_price = tx("i1", "income", Some("gift"), 1.0, None, "2024-01-10");

        let (total, count, warnings) = compute_taxable_income(&[income_missing_price], &period);
        assert_eq!(total, 0.0);
        assert_eq!(count, 1);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "income_missing_price");
        assert_eq!(warnings[0].tx_id.as_deref(), Some("i1"));
    }

    #[test]
    fn build_readiness_sets_prices_error_on_invalid_warning() {
        let report = TaxReport {
            period_id: "2024".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            jurisdiction: "chile".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary::default(),
            disposals: Vec::new(),
            warnings: vec![crate::features::crypto::TaxWarning {
                code: "invalid_date".to_string(),
                message: "bad date".to_string(),
                tx_id: None,
            }],
        };

        let readiness = build_readiness(&report, 3, 0, 0, TaxJurisdiction::Chile);
        let prices = readiness
            .iter()
            .find(|r| r.code == "prices")
            .expect("prices readiness item");
        assert_eq!(prices.status, "error");
        assert_eq!(prices.detail, "invalid");
    }

    #[test]
    fn history_csv_includes_fiscal_type_subtype_and_mechanical_type() {
        let period = parse_period("2024").expect("valid period");
        let tx = tx("s1", "trade", Some("swap"), 0.1, Some(50000.0), "2024-01-10");

        let csv = build_transaction_history_csv(&[tx], &period, "USD", 0.0);
        assert!(csv.contains("type,subtype,mechanical_type"));
        assert!(csv.contains(",trade,swap,swap,"));
    }

    #[test]
    fn history_csv_converts_fiat_values_for_clp_export() {
        let period = parse_period("2024").expect("valid period");
        let tx = tx("s1", "trade", Some("sell"), 0.1, Some(10.0), "2024-01-10");

        let csv = build_transaction_history_csv(&[tx], &period, "CLP", 1000.0);
        assert!(csv.contains("fiat_currency"));
        assert!(csv.contains(",10000.00000000,"));
        assert!(csv.contains(",CLP,"));
    }

    #[test]
    fn generate_tax_report_excludes_wallet_ids_from_settings() {
        let harness = new_test_service();
        let service = &harness.service;

        let wallet_excluded = service
            .add_wallet("Excluded wallet".to_string(), "exchange".to_string(), None)
            .expect("create excluded wallet");
        let wallet_included = service
            .add_wallet("Included wallet".to_string(), "exchange".to_string(), None)
            .expect("create included wallet");

        service
            .add_crypto_transaction(
                wallet_excluded.clone(),
                "bitcoin".to_string(),
                "BTC".to_string(),
                "trade".to_string(),
                1.0,
                Some(100.0),
                None,
                None,
                None,
                "2024-01-10".to_string(),
                None,
                Some("buy".to_string()),
                None,
                None,
            )
            .expect("buy in excluded wallet");
        let excluded_sell_id = service
            .add_crypto_transaction(
                wallet_excluded.clone(),
                "bitcoin".to_string(),
                "BTC".to_string(),
                "trade".to_string(),
                1.0,
                Some(150.0),
                None,
                None,
                None,
                "2024-02-10".to_string(),
                None,
                Some("sell".to_string()),
                None,
                None,
            )
            .expect("sell in excluded wallet");

        service
            .add_crypto_transaction(
                wallet_included.clone(),
                "bitcoin".to_string(),
                "BTC".to_string(),
                "trade".to_string(),
                1.0,
                Some(200.0),
                None,
                None,
                None,
                "2024-03-10".to_string(),
                None,
                Some("buy".to_string()),
                None,
                None,
            )
            .expect("buy in included wallet");
        let included_sell_id = service
            .add_crypto_transaction(
                wallet_included,
                "bitcoin".to_string(),
                "BTC".to_string(),
                "trade".to_string(),
                1.0,
                Some(260.0),
                None,
                None,
                None,
                "2024-04-10".to_string(),
                None,
                Some("sell".to_string()),
                None,
                None,
            )
            .expect("sell in included wallet");

        let mut settings = TaxPeriodSettings::defaults_for("2024");
        settings.jurisdiction = TaxJurisdiction::Usa;
        settings.excluded_wallet_ids = vec![wallet_excluded];

        service
            .save_tax_settings(settings)
            .expect("save tax settings with exclusion");

        let report = service
            .generate_tax_report("2024".to_string())
            .expect("generate tax report");

        assert_eq!(report.disposals.len(), 1);
        assert_eq!(report.summary.disposals, 1);
        assert_eq!(report.disposals[0].tx_id, included_sell_id);
        assert_ne!(report.disposals[0].tx_id, excluded_sell_id);

        drop(harness);
    }
}
