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

use super::lots::{build_term, consume_lots, update_summary};
use super::period::{is_in_period, parse_date};
use super::types::{Lot, TaxPeriod};
use crate::features::crypto::tax::{TaxJurisdiction, TaxMethod};
use crate::features::crypto::{TaxDisposal, TaxReport, TaxWarning};
use crate::models::CryptoTransaction;
use std::collections::{BTreeMap, HashMap};

pub(super) fn apply_swap_pair(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    period: &TaxPeriod,
    source: &CryptoTransaction,
    target: &CryptoTransaction,
    method: TaxMethod,
    jurisdiction: TaxJurisdiction,
    ipc_map: &BTreeMap<String, f64>,
    include_swaps: bool,
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

    let taxable = include_swaps && proceeds.is_some();
    let disposal_proceeds = proceeds.unwrap_or(0.0);

    let (allocations, cost_basis, short_gain, long_gain) = consume_lots(
        report,
        lots,
        &source.coin_id,
        source.amount,
        source_date,
        method,
        jurisdiction,
        ipc_map,
        source.id.as_str(),
        disposal_proceeds,
        taxable,
    );

    if taxable && is_in_period(period, source_date) {
        let gain = disposal_proceeds - cost_basis;
        let term = build_term(short_gain, long_gain, jurisdiction);
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
        update_summary(report, disposal_proceeds, cost_basis, gain, short_gain, long_gain);
    } else if include_swaps && proceeds.is_none() {
        report.warnings.push(TaxWarning {
            code: "swap_missing_price".to_string(),
            message: format!("Swap {} missing price; excluded from report", source.id),
            tx_id: Some(source.id.clone()),
        });
    }

    let acquisition_cost = if taxable { disposal_proceeds } else { cost_basis };
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
    let a_has_fee = a.fee.is_some() || a.fee_coin_id.is_some() || a.fee_amount.is_some();
    let b_has_fee = b.fee.is_some() || b.fee_coin_id.is_some() || b.fee_amount.is_some();

    if a_has_fee && !b_has_fee {
        return (a.clone(), b.clone(), false);
    }
    if b_has_fee && !a_has_fee {
        return (b.clone(), a.clone(), false);
    }

    if a.id < b.id {
        (a.clone(), b.clone(), true)
    } else {
        (b.clone(), a.clone(), true)
    }
}
