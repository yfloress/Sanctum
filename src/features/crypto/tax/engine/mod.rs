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

//! Tax report engine (offline-only).

mod lots;
mod period;
mod swaps;
mod types;

pub(crate) use period::{is_in_period, parse_date, parse_period};
pub(crate) use types::TaxPeriod;

use crate::features::crypto::tax::IpcEntry;
use crate::features::crypto::tax::{
    TaxJurisdiction, TaxTxType, is_loss_only_subtype, normalize_tax_subtype, resolve_tax_subtype,
    resolve_tax_type,
};
use crate::features::crypto::{TaxReport, TaxReportSummary, TaxWarning};
use crate::models::{CryptoTransaction, CryptoTransactionType};
use lots::{add_lot, apply_disposal, apply_fee_disposal};
use std::collections::{BTreeMap, HashMap, HashSet};
use swaps::{apply_swap_pair, resolve_swap_pair};
use types::TaxConfig;

pub fn build_tax_report(
    mut transactions: Vec<CryptoTransaction>,
    settings: crate::features::crypto::TaxPeriodSettings,
    ipc_entries: Vec<IpcEntry>,
) -> Result<TaxReport, String> {
    let period = parse_period(&settings.period_id)?;
    let method = settings.method;
    let jurisdiction = settings.jurisdiction;

    let mut ipc_map = BTreeMap::new();
    for entry in ipc_entries {
        ipc_map.insert(entry.period, entry.index);
    }

    let cfg = TaxConfig {
        period: &period,
        method,
        jurisdiction,
        ipc_map: &ipc_map,
    };

    let mut warnings: Vec<TaxWarning> = Vec::new();
    if matches!(cfg.jurisdiction, TaxJurisdiction::Chile) && cfg.ipc_map.is_empty() {
        warnings.push(TaxWarning {
            code: "ipc_missing".to_string(),
            message: "IPC data not loaded. Chilean inflation adjustment cannot be applied."
                .to_string(),
            tx_id: None,
        });
    }

    transactions.sort_by(|a, b| {
        let type_order = |mech: &str| -> u8 {
            match mech {
                "buy" => 0,
                "transfer_in" => 1,
                "sell" => 2,
                "transfer_out" => 3,
                "swap" => 4,
                _ => 5,
            }
        };
        a.date
            .cmp(&b.date)
            .then_with(|| type_order(a.mechanical_type()).cmp(&type_order(b.mechanical_type())))
            .then_with(|| a.id.cmp(&b.id))
    });

    let tx_map: HashMap<String, CryptoTransaction> = transactions
        .iter()
        .cloned()
        .map(|tx| (tx.id.clone(), tx))
        .collect();

    let mut processed: HashSet<String> = HashSet::new();
    let mut lots: HashMap<String, Vec<types::Lot>> = HashMap::new();

    let mut report = TaxReport {
        period_id: period.id.clone(),
        period_start: period.start.format("%Y-%m-%d").to_string(),
        period_end: period.end.format("%Y-%m-%d").to_string(),
        jurisdiction: settings.jurisdiction_str().to_string(),
        method: settings.method_str().to_string(),
        summary: TaxReportSummary {
            disposals: 0,
            total_proceeds: 0.0,
            total_cost: 0.0,
            total_gain: 0.0,
            short_term_gain: if matches!(
                jurisdiction,
                TaxJurisdiction::Usa | TaxJurisdiction::Other
            ) {
                Some(0.0)
            } else {
                None
            },
            long_term_gain: if matches!(jurisdiction, TaxJurisdiction::Usa | TaxJurisdiction::Other)
            {
                Some(0.0)
            } else {
                None
            },
        },
        disposals: Vec::new(),
        warnings,
    };

    for tx in transactions {
        if processed.contains(&tx.id) {
            continue;
        }

        let tx_date = match parse_date(&tx.date) {
            Some(date) => date,
            None => {
                report.warnings.push(TaxWarning {
                    code: "invalid_date".to_string(),
                    message: format!("Invalid date for transaction {}", tx.id),
                    tx_id: Some(tx.id.clone()),
                });
                continue;
            }
        };

        if tx_date > period.end {
            break;
        }

        if let Some(rel_id) = &tx.related_tx_id
            && let Some(counter) = tx_map.get(rel_id)
        {
            if processed.contains(rel_id) {
                continue;
            }

            let tx_mech = tx.mechanical_type();
            let counter_mech = counter.mechanical_type();
            let is_transfer_pair = (tx_mech == "transfer_out" && counter_mech == "transfer_in")
                || (tx_mech == "transfer_in" && counter_mech == "transfer_out");
            let is_swap_pair = tx_mech == "swap" && counter_mech == "swap";

            if is_transfer_pair {
                processed.insert(tx.id.clone());
                processed.insert(rel_id.clone());

                if settings.include_fee_crypto {
                    apply_fee_disposal(&mut report, &mut lots, &cfg, &tx);
                    apply_fee_disposal(&mut report, &mut lots, &cfg, counter);
                }

                continue;
            }

            if is_swap_pair {
                processed.insert(tx.id.clone());
                processed.insert(rel_id.clone());

                let (source, target, inferred) = resolve_swap_pair(&tx, counter);
                if inferred {
                    report.warnings.push(TaxWarning {
                        code: "swap_inferred".to_string(),
                        message: format!("Swap direction inferred for {}", source.id),
                        tx_id: Some(source.id.clone()),
                    });
                }

                let source_tax_type = resolve_tax_type(&source);
                let source_subtype = resolve_tax_subtype(&source);
                let loss_only = source_subtype
                    .as_deref()
                    .map(is_loss_only_subtype)
                    .unwrap_or(false);
                let swap_taxable = settings.include_swaps
                    && !matches!(source_tax_type, TaxTxType::Transfer)
                    && !(matches!(source_tax_type, TaxTxType::Expense) && loss_only);

                apply_swap_pair(&mut report, &mut lots, &cfg, &source, &target, swap_taxable);

                if settings.include_fee_crypto && !matches!(source_tax_type, TaxTxType::Transfer) {
                    apply_fee_disposal(&mut report, &mut lots, &cfg, &source);
                }
                continue;
            }
        }

        let parsed_tax_type = TaxTxType::parse(&tx.transaction_type);
        if parsed_tax_type.is_none() {
            report.warnings.push(TaxWarning {
                code: "invalid_type".to_string(),
                message: format!(
                    "Invalid fiscal type '{}' for transaction {}. Defaulted to trade.",
                    tx.transaction_type, tx.id
                ),
                tx_id: Some(tx.id.clone()),
            });
        }
        let tax_type = parsed_tax_type.unwrap_or(TaxTxType::Trade);
        let tax_subtype = tx
            .subtype
            .as_deref()
            .and_then(|sub| normalize_tax_subtype(tax_type.as_str(), sub));
        let loss_only = tax_subtype
            .as_deref()
            .map(is_loss_only_subtype)
            .unwrap_or(false);

        if tax_type == TaxTxType::Income {
            add_lot(
                &mut report,
                &mut lots,
                &tx,
                tx_date,
                jurisdiction,
                tax_subtype.as_deref(),
            );
            if settings.include_fee_crypto {
                apply_fee_disposal(&mut report, &mut lots, &cfg, &tx);
            }
            continue;
        }

        let tx_type = match tx.mechanical_type().parse::<CryptoTransactionType>() {
            Ok(t) => t,
            Err(_) => {
                report.warnings.push(TaxWarning {
                    code: "invalid_type".to_string(),
                    message: format!("Invalid transaction type for {}", tx.id),
                    tx_id: Some(tx.id.clone()),
                });
                continue;
            }
        };

        let taxable = !(matches!(tax_type, TaxTxType::Transfer)
            || matches!(tax_type, TaxTxType::Expense) && loss_only);

        match tx_type {
            CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                add_lot(
                    &mut report,
                    &mut lots,
                    &tx,
                    tx_date,
                    jurisdiction,
                    tax_subtype.as_deref(),
                );
            }
            CryptoTransactionType::Sell => {
                apply_disposal(&mut report, &mut lots, &cfg, &tx, tx_date, taxable);
                if settings.include_fee_crypto {
                    apply_fee_disposal(&mut report, &mut lots, &cfg, &tx);
                }
            }
            CryptoTransactionType::TransferOut => {
                apply_disposal(&mut report, &mut lots, &cfg, &tx, tx_date, taxable);
                if settings.include_fee_crypto {
                    apply_fee_disposal(&mut report, &mut lots, &cfg, &tx);
                }
            }
            CryptoTransactionType::Swap => {
                report.warnings.push(TaxWarning {
                    code: "swap_unpaired".to_string(),
                    message: format!("Swap transaction {} has no pair", tx.id),
                    tx_id: Some(tx.id.clone()),
                });
                apply_disposal(
                    &mut report,
                    &mut lots,
                    &cfg,
                    &tx,
                    tx_date,
                    settings.include_swaps && taxable,
                );
                if settings.include_fee_crypto {
                    apply_fee_disposal(&mut report, &mut lots, &cfg, &tx);
                }
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crypto::TaxPeriodSettings;
    use crate::features::crypto::tax::TaxMethod;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.0001
    }

    fn tx(id: &str, kind: &str, amount: f64, price: Option<f64>, date: &str) -> CryptoTransaction {
        let (fiscal_type, subtype) = match kind {
            "buy" => ("trade", Some("buy")),
            "sell" => ("trade", Some("sell")),
            "swap" => ("trade", Some("swap")),
            "transfer_in" => ("transfer", Some("deposit")),
            "transfer_out" => ("transfer", Some("withdrawal")),
            _ => ("trade", None),
        };
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet".to_string(),
            "btc".to_string(),
            "BTC".to_string(),
            fiscal_type.to_string(),
            amount,
            price,
            None,
            date.to_string(),
            None,
        );
        tx.subtype = subtype.map(str::to_string);
        tx
    }

    fn tx_fiscal(
        id: &str,
        fiscal_type: &str,
        subtype: Option<&str>,
        amount: f64,
        price: Option<f64>,
        date: &str,
    ) -> CryptoTransaction {
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet".to_string(),
            "btc".to_string(),
            "BTC".to_string(),
            fiscal_type.to_string(),
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
    fn build_report_fifo_usa() {
        let buy = tx("b1", "buy", 1.0, Some(100.0), "2024-01-10");
        let sell = tx("s1", "sell", 1.0, Some(150.0), "2024-02-10");

        let settings = TaxPeriodSettings {
            period_id: "2024".to_string(),
            jurisdiction: TaxJurisdiction::Usa,
            method: TaxMethod::Fifo,
            include_swaps: false,
            include_fee_crypto: false,
            excluded_wallet_ids: Vec::new(),
        };

        let report = build_tax_report(vec![buy, sell], settings, vec![]).expect("report");

        assert_eq!(report.summary.disposals, 1);
        assert!(approx_eq(report.summary.total_proceeds, 150.0));
        assert!(approx_eq(report.summary.total_cost, 100.0));
        assert!(approx_eq(report.summary.total_gain, 50.0));
        assert_eq!(report.disposals.len(), 1);
        assert_eq!(report.disposals[0].term.as_deref(), Some("short"));
        assert!(report.summary.short_term_gain.is_some());
    }

    #[test]
    fn build_report_warns_on_missing_price() {
        let buy = tx("b1", "buy", 1.0, Some(100.0), "2024-01-10");
        let sell = tx("s1", "sell", 1.0, None, "2024-02-10");

        let settings = TaxPeriodSettings {
            period_id: "2024".to_string(),
            jurisdiction: TaxJurisdiction::Usa,
            method: TaxMethod::Fifo,
            include_swaps: false,
            include_fee_crypto: false,
            excluded_wallet_ids: Vec::new(),
        };

        let report = build_tax_report(vec![buy, sell], settings, vec![]).expect("report");
        assert!(report.disposals.is_empty());
        assert!(report.warnings.iter().any(|w| w.code == "missing_price"));
    }

    #[test]
    fn chile_missing_ipc_emits_warning() {
        let buy = tx("b1", "buy", 1.0, Some(100.0), "2024-01-10");
        let sell = tx("s1", "sell", 1.0, Some(150.0), "2024-02-10");

        let settings = TaxPeriodSettings {
            period_id: "2024".to_string(),
            jurisdiction: TaxJurisdiction::Chile,
            method: TaxMethod::Fifo,
            include_swaps: false,
            include_fee_crypto: false,
            excluded_wallet_ids: Vec::new(),
        };

        let report = build_tax_report(vec![buy, sell], settings, vec![]).expect("report");
        assert!(report.warnings.iter().any(|w| w.code == "ipc_missing"));
        assert_eq!(report.summary.disposals, 1);
    }

    #[test]
    fn swap_pair_not_taxable_when_include_swaps_is_false() {
        let buy = tx("b1", "buy", 1.0, Some(100.0), "2024-01-10");
        let mut swap_out = tx("s1", "swap", 1.0, Some(150.0), "2024-02-10");
        swap_out.related_tx_id = Some("s2".to_string());

        let mut swap_in = CryptoTransaction::new(
            "s2".to_string(),
            "wallet".to_string(),
            "eth".to_string(),
            "ETH".to_string(),
            "trade".to_string(),
            10.0,
            Some(15.0),
            None,
            "2024-02-10".to_string(),
            None,
        );
        swap_in.subtype = Some("swap".to_string());
        swap_in.related_tx_id = Some("s1".to_string());

        let settings = TaxPeriodSettings {
            period_id: "2024".to_string(),
            jurisdiction: TaxJurisdiction::Chile,
            method: TaxMethod::Fifo,
            include_swaps: false,
            include_fee_crypto: false,
            excluded_wallet_ids: Vec::new(),
        };

        let report = build_tax_report(vec![buy, swap_out, swap_in], settings, vec![]).expect("report");
        assert_eq!(report.summary.disposals, 0);
        assert!(report.disposals.is_empty());
    }

    #[test]
    fn expense_lost_is_not_taxable() {
        let buy = tx("b1", "buy", 1.0, Some(100.0), "2024-01-10");
        let lost = tx_fiscal(
            "e1",
            "expense",
            Some("lost"),
            1.0,
            Some(200.0),
            "2024-02-10",
        );

        let settings = TaxPeriodSettings {
            period_id: "2024".to_string(),
            jurisdiction: TaxJurisdiction::Chile,
            method: TaxMethod::Fifo,
            include_swaps: true,
            include_fee_crypto: false,
            excluded_wallet_ids: Vec::new(),
        };

        let report = build_tax_report(vec![buy, lost], settings, vec![]).expect("report");
        assert_eq!(report.summary.disposals, 0);
        assert!(report.disposals.is_empty());
    }

    #[test]
    fn transfer_pair_with_fee_crypto_records_fee_disposal() {
        let buy = tx("b1", "buy", 1.0, Some(100.0), "2024-01-10");
        let mut out = tx("t1", "transfer_out", 0.5, Some(120.0), "2024-02-10");
        out.related_tx_id = Some("t2".to_string());
        out.fee_coin_id = Some("btc".to_string());
        out.fee_amount = Some(0.01);

        let mut inc = tx("t2", "transfer_in", 0.5, None, "2024-02-10");
        inc.related_tx_id = Some("t1".to_string());

        let settings = TaxPeriodSettings {
            period_id: "2024".to_string(),
            jurisdiction: TaxJurisdiction::Chile,
            method: TaxMethod::Fifo,
            include_swaps: true,
            include_fee_crypto: true,
            excluded_wallet_ids: Vec::new(),
        };

        let report = build_tax_report(vec![buy, out, inc], settings, vec![]).expect("report");
        assert_eq!(report.summary.disposals, 1);
        assert_eq!(report.disposals[0].disposal_type, "fee");
    }

    #[test]
    fn invalid_fiscal_type_emits_warning_and_defaults_to_trade_behavior() {
        let invalid = tx_fiscal("x1", "banana", None, 1.0, Some(100.0), "2024-01-10");
        let settings = TaxPeriodSettings {
            period_id: "2024".to_string(),
            jurisdiction: TaxJurisdiction::Chile,
            method: TaxMethod::Fifo,
            include_swaps: true,
            include_fee_crypto: false,
            excluded_wallet_ids: Vec::new(),
        };

        let report = build_tax_report(vec![invalid], settings, vec![]).expect("report");
        assert!(report.warnings.iter().any(|w| w.code == "invalid_type"));
        // Defaulted to trade + missing subtype => mechanical buy (acquisition), no disposal.
        assert_eq!(report.summary.disposals, 0);
    }
}
