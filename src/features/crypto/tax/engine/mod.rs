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

use lots::{add_lot, apply_disposal, apply_fee_disposal};
use period::{parse_date, parse_period};
use swaps::{apply_swap_pair, resolve_swap_pair};
use crate::features::crypto::tax::{TaxJurisdiction, TaxMethod};
use crate::features::crypto::{
    TaxReport, TaxReportSummary, TaxWarning,
};
use crate::features::crypto::tax::IpcEntry;
use crate::models::{CryptoTransaction, CryptoTransactionType};
use std::collections::{BTreeMap, HashMap, HashSet};

pub fn build_tax_report(
    mut transactions: Vec<CryptoTransaction>,
    settings: crate::features::crypto::TaxPeriodSettings,
    ipc_entries: Vec<IpcEntry>,
) -> Result<TaxReport, String> {
    let period = parse_period(&settings.period_id)?;
    let method = TaxMethod::from_str(&settings.method);
    let jurisdiction = TaxJurisdiction::from_str(&settings.jurisdiction);

    let mut ipc_map = BTreeMap::new();
    for entry in ipc_entries {
        ipc_map.insert(entry.period, entry.index);
    }

    let mut warnings: Vec<TaxWarning> = Vec::new();
    if matches!(jurisdiction, TaxJurisdiction::Chile) && ipc_map.is_empty() {
        warnings.push(TaxWarning {
            code: "ipc_missing".to_string(),
            message: "IPC data not loaded. Chilean inflation adjustment cannot be applied.".to_string(),
            tx_id: None,
        });
    }

    transactions.sort_by(|a, b| {
        let type_order = |tx_type: &str| -> u8 {
            match tx_type {
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
            .then_with(|| type_order(&a.transaction_type).cmp(&type_order(&b.transaction_type)))
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
        jurisdiction: settings.jurisdiction.clone(),
        method: settings.method.clone(),
        summary: TaxReportSummary {
            disposals: 0,
            total_proceeds: 0.0,
            total_cost: 0.0,
            total_gain: 0.0,
            short_term_gain: if matches!(jurisdiction, TaxJurisdiction::Usa) {
                Some(0.0)
            } else {
                None
            },
            long_term_gain: if matches!(jurisdiction, TaxJurisdiction::Usa) {
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

        if let Some(rel_id) = &tx.related_tx_id {
            if let Some(counter) = tx_map.get(rel_id) {
                if processed.contains(rel_id) {
                    continue;
                }

                let is_transfer_pair = (tx.transaction_type == "transfer_out"
                    && counter.transaction_type == "transfer_in")
                    || (tx.transaction_type == "transfer_in"
                        && counter.transaction_type == "transfer_out");
                let is_swap_pair = tx.transaction_type == "swap"
                    && counter.transaction_type == "swap";

                if is_transfer_pair {
                    processed.insert(tx.id.clone());
                    processed.insert(rel_id.clone());

                    if settings.include_fee_crypto {
                        apply_fee_disposal(
                            &mut report,
                            &mut lots,
                            &period,
                            &tx,
                            method,
                            jurisdiction,
                            &ipc_map,
                        );
                        apply_fee_disposal(
                            &mut report,
                            &mut lots,
                            &period,
                            counter,
                            method,
                            jurisdiction,
                            &ipc_map,
                        );
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

                    apply_swap_pair(
                        &mut report,
                        &mut lots,
                        &period,
                        &source,
                        &target,
                        method,
                        jurisdiction,
                        &ipc_map,
                        settings.include_swaps,
                    );

                    if settings.include_fee_crypto {
                        apply_fee_disposal(
                            &mut report,
                            &mut lots,
                            &period,
                            &source,
                            method,
                            jurisdiction,
                            &ipc_map,
                        );
                    }
                    continue;
                }
            }
        }

        let tx_type = match tx.transaction_type.parse::<CryptoTransactionType>() {
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

        match tx_type {
            CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                add_lot(&mut report, &mut lots, &tx, tx_date);
            }
            CryptoTransactionType::Sell => {
                apply_disposal(
                    &mut report,
                    &mut lots,
                    &period,
                    &tx,
                    tx_date,
                    method,
                    jurisdiction,
                    &ipc_map,
                    true,
                );
                if settings.include_fee_crypto {
                    apply_fee_disposal(
                        &mut report,
                        &mut lots,
                        &period,
                        &tx,
                        method,
                        jurisdiction,
                        &ipc_map,
                    );
                }
            }
            CryptoTransactionType::TransferOut => {
                apply_disposal(
                    &mut report,
                    &mut lots,
                    &period,
                    &tx,
                    tx_date,
                    method,
                    jurisdiction,
                    &ipc_map,
                    false,
                );
                if settings.include_fee_crypto {
                    apply_fee_disposal(
                        &mut report,
                        &mut lots,
                        &period,
                        &tx,
                        method,
                        jurisdiction,
                        &ipc_map,
                    );
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
                    &period,
                    &tx,
                    tx_date,
                    method,
                    jurisdiction,
                    &ipc_map,
                    false,
                );
                if settings.include_fee_crypto {
                    apply_fee_disposal(
                        &mut report,
                        &mut lots,
                        &period,
                        &tx,
                        method,
                        jurisdiction,
                        &ipc_map,
                    );
                }
            }
        }
    }

    Ok(report)
}
