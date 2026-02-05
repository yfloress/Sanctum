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

//! Lot management and disposal calculations.

use super::period::{is_in_period, parse_date, prev_month_key};
use super::types::{AllocationInfo, Lot, TaxPeriod};
use crate::features::crypto::tax::{TaxJurisdiction, TaxMethod};
use crate::features::crypto::{TaxDisposal, TaxReport, TaxWarning};
use crate::models::CryptoTransaction;
use std::collections::{BTreeMap, HashMap};

pub(super) fn add_lot(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    tx: &CryptoTransaction,
    tx_date: chrono::NaiveDate,
) {
    let price = tx.price_per_coin.unwrap_or(0.0);
    if tx.price_per_coin.is_none() {
        report.warnings.push(TaxWarning {
            code: "missing_price".to_string(),
            message: format!("Missing price for acquisition {}", tx.id),
            tx_id: Some(tx.id.clone()),
        });
    }

    let fee = tx.fee.unwrap_or(0.0);
    let total_cost = (tx.amount * price) + fee;
    let unit_cost = if tx.amount > 0.0 {
        total_cost / tx.amount
    } else {
        0.0
    };

    let lot = Lot {
        lot_id: tx.id.clone(),
        acquired_date: tx_date,
        acquired_date_raw: tx.date.clone(),
        acquired_prev_month: prev_month_key(tx_date),
        quantity: tx.amount,
        unit_cost,
    };

    lots.entry(tx.coin_id.clone()).or_default().push(lot);
}

pub(super) fn apply_disposal(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    period: &TaxPeriod,
    tx: &CryptoTransaction,
    tx_date: chrono::NaiveDate,
    method: TaxMethod,
    jurisdiction: TaxJurisdiction,
    ipc_map: &BTreeMap<String, f64>,
    taxable: bool,
) {
    let mut proceeds = match tx.price_per_coin {
        Some(price) => tx.amount * price,
        None => 0.0,
    };
    if tx.price_per_coin.is_none() && taxable {
        report.warnings.push(TaxWarning {
            code: "missing_price".to_string(),
            message: format!("Missing price for disposal {}", tx.id),
            tx_id: Some(tx.id.clone()),
        });
    }

    if taxable {
        let fee = tx.fee.unwrap_or(0.0);
        proceeds = (proceeds - fee).max(0.0);
    }

    let (allocations, cost_basis, short_gain, long_gain) = consume_lots(
        report,
        lots,
        &tx.coin_id,
        tx.amount,
        tx_date,
        method,
        jurisdiction,
        ipc_map,
        tx.id.as_str(),
        proceeds,
        taxable,
    );

    if taxable && is_in_period(period, tx_date) && tx.price_per_coin.is_some() {
        let gain = proceeds - cost_basis;
        let term = build_term(short_gain, long_gain, jurisdiction);
        report.disposals.push(TaxDisposal {
            tx_id: tx.id.clone(),
            date: tx.date.clone(),
            coin_id: tx.coin_id.clone(),
            symbol: tx.symbol.clone(),
            amount: tx.amount,
            proceeds,
            cost_basis,
            gain,
            term,
            disposal_type: tx.transaction_type.clone(),
            allocations: allocations.iter().map(|a| a.allocation.clone()).collect(),
        });
        update_summary(report, proceeds, cost_basis, gain, short_gain, long_gain);
    }
}

pub(super) fn apply_fee_disposal(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    period: &TaxPeriod,
    tx: &CryptoTransaction,
    method: TaxMethod,
    jurisdiction: TaxJurisdiction,
    ipc_map: &BTreeMap<String, f64>,
) {
    let fee_coin_id = match tx.fee_coin_id.as_deref() {
        Some(id) => id,
        None => return,
    };
    let fee_amount = match tx.fee_amount {
        Some(amount) if amount > 0.0 => amount,
        _ => return,
    };
    let tx_date = match parse_date(&tx.date) {
        Some(date) => date,
        None => return,
    };

    let mut fee_price = None;
    if fee_coin_id == tx.coin_id {
        fee_price = tx.price_per_coin;
    }

    let proceeds = match fee_price {
        Some(price) => fee_amount * price,
        None => {
            report.warnings.push(TaxWarning {
                code: "fee_missing_price".to_string(),
                message: format!("Missing price for fee disposal {}", tx.id),
                tx_id: Some(tx.id.clone()),
            });
            0.0
        }
    };

    let (allocations, cost_basis, short_gain, long_gain) = consume_lots(
        report,
        lots,
        fee_coin_id,
        fee_amount,
        tx_date,
        method,
        jurisdiction,
        ipc_map,
        tx.id.as_str(),
        proceeds,
        true,
    );

    if is_in_period(period, tx_date) && fee_price.is_some() {
        let gain = proceeds - cost_basis;
        let term = build_term(short_gain, long_gain, jurisdiction);
        report.disposals.push(TaxDisposal {
            tx_id: format!("{}:fee", tx.id),
            date: tx.date.clone(),
            coin_id: fee_coin_id.to_string(),
            symbol: fee_coin_id.to_uppercase(),
            amount: fee_amount,
            proceeds,
            cost_basis,
            gain,
            term,
            disposal_type: "fee".to_string(),
            allocations: allocations.iter().map(|a| a.allocation.clone()).collect(),
        });
        update_summary(report, proceeds, cost_basis, gain, short_gain, long_gain);
    }
}

pub(super) fn consume_lots(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    coin_id: &str,
    amount: f64,
    sale_date: chrono::NaiveDate,
    method: TaxMethod,
    jurisdiction: TaxJurisdiction,
    ipc_map: &BTreeMap<String, f64>,
    tx_id: &str,
    proceeds: f64,
    taxable: bool,
) -> (Vec<AllocationInfo>, f64, f64, f64) {
    let mut allocations = Vec::new();
    let mut cost_basis = 0.0;
    let mut short_gain = 0.0;
    let mut long_gain = 0.0;

    let entry = lots.entry(coin_id.to_string()).or_default();
    let total_available: f64 = entry.iter().map(|lot| lot.quantity).sum();

    if total_available <= 0.0 {
        if taxable {
            report.warnings.push(TaxWarning {
                code: "no_lots".to_string(),
                message: format!("No lots available for {}", tx_id),
                tx_id: Some(tx_id.to_string()),
            });
        }
        return (allocations, 0.0, 0.0, 0.0);
    }

    let mut remaining = amount;

    if method == TaxMethod::Cpp {
        let (allocs, cost) = consume_lots_cpp(
            report,
            entry,
            amount,
            sale_date,
            jurisdiction,
            ipc_map,
            tx_id,
        );
        allocations = allocs;
        cost_basis = cost;
        let (short, long) = split_term_gain(&allocations, proceeds, sale_date, jurisdiction);
        short_gain = short;
        long_gain = long;
        return (allocations, cost_basis, short_gain, long_gain);
    }

    let mut indices: Vec<usize> = (0..entry.len()).collect();
    match method {
        TaxMethod::Fifo => {}
        TaxMethod::Lifo => indices.reverse(),
        TaxMethod::Hifo => indices.sort_by(|a, b| {
            entry[*b]
                .unit_cost
                .partial_cmp(&entry[*a].unit_cost)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| entry[*a].acquired_date.cmp(&entry[*b].acquired_date))
        }),
        TaxMethod::Cpp => {}
    }

    let sale_prev_month = prev_month_key(sale_date);

    for idx in indices {
        if remaining <= 0.0 {
            break;
        }
        if idx >= entry.len() {
            continue;
        }
        let lot = &mut entry[idx];
        if lot.quantity <= 0.0 {
            continue;
        }

        let qty = remaining.min(lot.quantity);
        remaining -= qty;
        lot.quantity -= qty;

        let base_cost = qty * lot.unit_cost;
        let (adjusted_cost, cost_used) = apply_ipc_adjustment(
            report,
            jurisdiction,
            ipc_map,
            &lot.acquired_prev_month,
            &sale_prev_month,
            base_cost,
            tx_id,
        );

        cost_basis += cost_used;

        allocations.push(AllocationInfo {
            allocation: crate::features::crypto::LotAllocation {
                lot_id: lot.lot_id.clone(),
                lot_date: lot.acquired_date_raw.clone(),
                quantity: qty,
                unit_cost: lot.unit_cost,
                cost: base_cost,
                adjusted_cost,
            },
            lot_date: lot.acquired_date,
        });
    }

    entry.retain(|lot| lot.quantity > 0.0);

    if remaining > 0.0 {
        report.warnings.push(TaxWarning {
            code: "insufficient_lots".to_string(),
            message: format!("Not enough lots to cover {}", tx_id),
            tx_id: Some(tx_id.to_string()),
        });
    }

    let (short, long) = split_term_gain(&allocations, proceeds, sale_date, jurisdiction);
    short_gain = short;
    long_gain = long;

    (allocations, cost_basis, short_gain, long_gain)
}

fn consume_lots_cpp(
    report: &mut TaxReport,
    lots: &mut Vec<Lot>,
    amount: f64,
    sale_date: chrono::NaiveDate,
    jurisdiction: TaxJurisdiction,
    ipc_map: &BTreeMap<String, f64>,
    tx_id: &str,
) -> (Vec<AllocationInfo>, f64) {
    let mut allocations = Vec::new();
    let total_qty: f64 = lots.iter().map(|lot| lot.quantity).sum();
    if total_qty <= 0.0 {
        report.warnings.push(TaxWarning {
            code: "no_lots".to_string(),
            message: format!("No lots available for {}", tx_id),
            tx_id: Some(tx_id.to_string()),
        });
        return (allocations, 0.0);
    }

    let mut remaining = amount;
    let sale_prev_month = prev_month_key(sale_date);
    let mut total_cost = 0.0;

    for (idx, lot) in lots.iter_mut().enumerate() {
        if lot.quantity <= 0.0 {
            continue;
        }
        let share = if idx == lots.len() - 1 {
            remaining
        } else {
            amount * (lot.quantity / total_qty)
        };
        let qty = share.min(lot.quantity);
        remaining -= qty;
        lot.quantity -= qty;

        let base_cost = qty * lot.unit_cost;
        let (adjusted_cost, cost_used) = apply_ipc_adjustment(
            report,
            jurisdiction,
            ipc_map,
            &lot.acquired_prev_month,
            &sale_prev_month,
            base_cost,
            tx_id,
        );

        total_cost += cost_used;
        allocations.push(AllocationInfo {
            allocation: crate::features::crypto::LotAllocation {
                lot_id: lot.lot_id.clone(),
                lot_date: lot.acquired_date_raw.clone(),
                quantity: qty,
                unit_cost: lot.unit_cost,
                cost: base_cost,
                adjusted_cost,
            },
            lot_date: lot.acquired_date,
        });
    }

    lots.retain(|lot| lot.quantity > 0.0);

    if remaining > 0.0 {
        report.warnings.push(TaxWarning {
            code: "insufficient_lots".to_string(),
            message: format!("Not enough lots to cover {}", tx_id),
            tx_id: Some(tx_id.to_string()),
        });
    }

    (allocations, total_cost)
}

fn apply_ipc_adjustment(
    report: &mut TaxReport,
    jurisdiction: TaxJurisdiction,
    ipc_map: &BTreeMap<String, f64>,
    buy_prev: &str,
    sale_prev: &str,
    base_cost: f64,
    tx_id: &str,
) -> (Option<f64>, f64) {
    if !matches!(jurisdiction, TaxJurisdiction::Chile) {
        return (None, base_cost);
    }

    let buy_idx = ipc_map.get(buy_prev).copied();
    let sale_idx = ipc_map.get(sale_prev).copied();

    match (buy_idx, sale_idx) {
        (Some(buy), Some(sale)) if buy > 0.0 => {
            let adjusted = base_cost * (sale / buy);
            (Some(adjusted), adjusted)
        }
        _ => {
            report.warnings.push(TaxWarning {
                code: "ipc_missing".to_string(),
                message: format!("IPC index missing for {}", tx_id),
                tx_id: Some(tx_id.to_string()),
            });
            (None, base_cost)
        }
    }
}

fn split_term_gain(
    allocations: &[AllocationInfo],
    proceeds: f64,
    sale_date: chrono::NaiveDate,
    jurisdiction: TaxJurisdiction,
) -> (f64, f64) {
    if !matches!(jurisdiction, TaxJurisdiction::Usa) {
        return (0.0, 0.0);
    }

    let total_qty: f64 = allocations.iter().map(|a| a.allocation.quantity).sum();
    if total_qty <= 0.0 {
        return (0.0, 0.0);
    }

    let proceeds_per_unit = proceeds / total_qty;
    let mut short_gain = 0.0;
    let mut long_gain = 0.0;

    for alloc in allocations {
        let holding_days = (sale_date - alloc.lot_date).num_days();
        let alloc_proceeds = proceeds_per_unit * alloc.allocation.quantity;
        let alloc_cost = alloc
            .allocation
            .adjusted_cost
            .unwrap_or(alloc.allocation.cost);
        let gain = alloc_proceeds - alloc_cost;

        if holding_days >= 365 {
            long_gain += gain;
        } else {
            short_gain += gain;
        }
    }

    (short_gain, long_gain)
}

pub(super) fn build_term(
    short_gain: f64,
    long_gain: f64,
    jurisdiction: TaxJurisdiction,
) -> Option<String> {
    if !matches!(jurisdiction, TaxJurisdiction::Usa) {
        return None;
    }

    let short = short_gain.abs() > f64::EPSILON;
    let long = long_gain.abs() > f64::EPSILON;

    let term = if short && long {
        "mixed"
    } else if long {
        "long"
    } else {
        "short"
    };

    Some(term.to_string())
}

pub(super) fn update_summary(
    report: &mut TaxReport,
    proceeds: f64,
    cost_basis: f64,
    gain: f64,
    short_gain: f64,
    long_gain: f64,
) {
    report.summary.disposals += 1;
    report.summary.total_proceeds += proceeds;
    report.summary.total_cost += cost_basis;
    report.summary.total_gain += gain;

    if let Some(val) = report.summary.short_term_gain.as_mut() {
        *val += short_gain;
    }
    if let Some(val) = report.summary.long_term_gain.as_mut() {
        *val += long_gain;
    }
}
