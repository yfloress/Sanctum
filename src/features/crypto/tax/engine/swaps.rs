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

//! Swap handling helpers.

use super::lots::{apply_gain_ipc_adjustment, build_term, consume_lots, update_summary};
use super::period::{is_in_period, parse_date};
use super::types::{DisposalRequest, Lot, TaxConfig};
use crate::features::crypto::{TaxDisposal, TaxReport, TaxWarning};
use crate::models::CryptoTransaction;
use std::collections::HashMap;

pub(super) fn apply_swap_pair(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    cfg: &TaxConfig,
    source: &CryptoTransaction,
    target: &CryptoTransaction,
    taxable_swap: bool,
) {
    let source_date = match parse_date(&source.date) {
        Some(date) => date,
        None => {
            report.warnings.push(TaxWarning {
                code: "invalid_date".to_string(),
                message: format!("Invalid swap date for {}", source.id),
                tx_id: Some(source.id.clone()),
            });
            return;
        }
    };

    let proceeds = compute_swap_proceeds(source, target);

    let taxable = taxable_swap && proceeds.is_some();
    let disposal_proceeds = proceeds.unwrap_or(0.0);

    let req = DisposalRequest {
        coin_id: &source.coin_id,
        amount: source.amount,
        sale_date: source_date,
        tx_id: &source.id,
        proceeds: disposal_proceeds,
        taxable,
    };

    let (allocations, cost_basis, short_gain, long_gain) = consume_lots(report, lots, cfg, &req);

    if taxable && is_in_period(cfg.period, source_date) {
        let raw_gain = disposal_proceeds - cost_basis;
        let gain = apply_gain_ipc_adjustment(report, cfg, source_date, raw_gain, &source.id);
        let term = build_term(short_gain, long_gain, cfg.jurisdiction);
        report.disposals.push(TaxDisposal {
            tx_id: source.id.clone(),
            date: source.date.clone(),
            coin_id: source.coin_id.clone(),
            symbol: source.symbol.clone(),
            amount: source.amount,
            proceeds: disposal_proceeds,
            cost_basis,
            gain,
            term,
            disposal_type: "swap".to_string(),
            allocations: allocations.iter().map(|a| a.allocation.clone()).collect(),
        });
        update_summary(
            report,
            disposal_proceeds,
            cost_basis,
            gain,
            short_gain,
            long_gain,
        );
    } else if taxable_swap && proceeds.is_none() {
        report.warnings.push(TaxWarning {
            code: "swap_missing_price".to_string(),
            message: format!("Swap {} missing price; excluded from report", source.id),
            tx_id: Some(source.id.clone()),
        });
    }

    let acquisition_cost = match target.override_cost_basis {
        Some(override_cost) => override_cost,
        None => {
            if taxable {
                disposal_proceeds
            } else {
                cost_basis
            }
        }
    };
    let unit_cost = if target.amount > 0.0 {
        acquisition_cost / target.amount
    } else {
        0.0
    };

    if let Some(target_date) = parse_date(&target.date) {
        let lot = Lot {
            lot_id: target.id.clone(),
            acquired_date: target_date,
            acquired_date_raw: target.date.clone(),
            acquired_prev_month: super::period::prev_month_key(target_date),
            quantity: target.amount,
            unit_cost,
        };
        lots.entry(target.coin_id.clone()).or_default().push(lot);
    } else {
        report.warnings.push(TaxWarning {
            code: "invalid_date".to_string(),
            message: format!("Invalid swap target date for {}", target.id),
            tx_id: Some(target.id.clone()),
        });
    }
}

pub(super) fn compute_swap_proceeds(
    source: &CryptoTransaction,
    target: &CryptoTransaction,
) -> Option<f64> {
    if let Some(override_proceeds) = source.override_proceeds {
        return Some(override_proceeds);
    }
    if let Some(price) = source.price_per_coin {
        return Some(source.amount * price);
    }
    if let Some(price) = target.price_per_coin {
        return Some(target.amount * price);
    }
    None
}

pub(super) fn resolve_swap_pair(
    a: &CryptoTransaction,
    b: &CryptoTransaction,
) -> (CryptoTransaction, CryptoTransaction, bool) {
    // Source-side scoring to avoid relying on UUID ordering.
    // Higher score means "more likely to be disposal/source".
    let source_score = |tx: &CryptoTransaction| -> i32 {
        let mut score = 0;
        if tx.override_proceeds.is_some() {
            score += 8;
        }
        if tx.override_cost_basis.is_some() {
            score -= 8;
        }
        if tx.fee_coin_id.is_some() || tx.fee_amount.is_some() {
            score += 4;
        }
        if tx.fee.is_some() {
            score += 2;
        }
        if tx.price_per_coin.is_some() {
            score += 1;
        }
        score
    };

    let a_score = source_score(a);
    let b_score = source_score(b);
    if a_score > b_score {
        return (a.clone(), b.clone(), false);
    }
    if b_score > a_score {
        return (b.clone(), a.clone(), false);
    }

    if a.id < b.id {
        (a.clone(), b.clone(), true)
    } else {
        (b.clone(), a.clone(), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn swap_tx(id: &str, amount: f64, price: Option<f64>) -> CryptoTransaction {
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet".to_string(),
            "btc".to_string(),
            "BTC".to_string(),
            "trade".to_string(),
            amount,
            price,
            None,
            "2024-01-10".to_string(),
            None,
        );
        tx.subtype = Some("swap".to_string());
        tx
    }

    #[test]
    fn compute_swap_proceeds_prefers_source_price() {
        let source = swap_tx("s1", 1.0, Some(150.0));
        let target = swap_tx("s2", 2.0, None);
        assert_eq!(compute_swap_proceeds(&source, &target), Some(150.0));
    }

    #[test]
    fn compute_swap_proceeds_uses_target_price() {
        let source = swap_tx("s1", 1.0, None);
        let target = swap_tx("s2", 2.0, Some(200.0));
        assert_eq!(compute_swap_proceeds(&source, &target), Some(400.0));
    }

    #[test]
    fn resolve_swap_pair_prefers_fee_side() {
        let mut a = swap_tx("a", 1.0, Some(100.0));
        a.fee = Some(1.0);
        let b = swap_tx("b", 2.0, Some(50.0));

        let (source, _target, inferred) = resolve_swap_pair(&a, &b);
        assert_eq!(source.id, "a");
        assert!(!inferred);
    }

    #[test]
    fn resolve_swap_pair_prefers_override_proceeds_side() {
        let mut a = swap_tx("a", 1.0, None);
        a.override_proceeds = Some(100.0);
        let mut b = swap_tx("b", 2.0, None);
        b.override_cost_basis = Some(100.0);

        let (source, target, inferred) = resolve_swap_pair(&a, &b);
        assert_eq!(source.id, "a");
        assert_eq!(target.id, "b");
        assert!(!inferred);
    }

    #[test]
    fn resolve_swap_pair_marks_inferred_on_tie() {
        let a = swap_tx("a", 1.0, None);
        let b = swap_tx("b", 2.0, None);
        let (_source, _target, inferred) = resolve_swap_pair(&a, &b);
        assert!(inferred);
    }
}
