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
use super::types::{AllocationInfo, DisposalRequest, Lot, TaxConfig};
use crate::features::crypto::tax::{TaxJurisdiction, TaxMethod};
use crate::features::crypto::{TaxDisposal, TaxReport, TaxWarning};
use crate::models::CryptoTransaction;
use chrono::Datelike;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Lot creation
// ---------------------------------------------------------------------------

/// Adds a new acquisition lot from a transaction.
///
/// Jurisdiction-aware behaviour:
/// - **Chile**: fees are **not** added to the cost basis for personas naturales
///   (SII ruling). Additionally, income subtypes `airdrop`, `staking`, and
///   `fork` are recognised with cost **$0** regardless of market price
///   (Oficio Ord. Nº979/2022).
/// - **USA**: fees are added to the cost basis. Income items use FMV as cost.
pub(super) fn add_lot(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    tx: &CryptoTransaction,
    tx_date: chrono::NaiveDate,
    jurisdiction: TaxJurisdiction,
    tax_subtype: Option<&str>,
) {
    // Chile: airdrops, staking and forks are recognised at cost $0.
    let force_zero_cost = matches!(jurisdiction, TaxJurisdiction::Chile)
        && matches!(
            tax_subtype,
            Some("airdrop") | Some("staking") | Some("fork")
        );

    let price = if force_zero_cost {
        0.0
    } else {
        tx.price_per_coin.unwrap_or(0.0)
    };

    if !force_zero_cost && tx.price_per_coin.is_none() && tx.override_cost_basis.is_none() {
        report.warnings.push(TaxWarning {
            code: "missing_price".to_string(),
            message: format!("Missing price for acquisition {}", tx.id),
            tx_id: Some(tx.id.clone()),
        });
    }

    // Chile: fees cannot be part of the cost basis for personas naturales.
    let fee = match jurisdiction {
        TaxJurisdiction::Chile => 0.0,
        _ => tx.fee.unwrap_or(0.0),
    };

    let total_cost = match tx.override_cost_basis {
        Some(override_cost) if !force_zero_cost => override_cost,
        _ if force_zero_cost => 0.0,
        _ => (tx.amount * price) + fee,
    };

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

// ---------------------------------------------------------------------------
// Disposals
// ---------------------------------------------------------------------------

pub(super) fn apply_disposal(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    cfg: &TaxConfig,
    tx: &CryptoTransaction,
    tx_date: chrono::NaiveDate,
    taxable: bool,
) {
    let mut proceeds = match tx.price_per_coin {
        Some(price) => tx.amount * price,
        None => 0.0,
    };
    if tx.price_per_coin.is_none() && tx.override_proceeds.is_none() && taxable {
        report.warnings.push(TaxWarning {
            code: "missing_price".to_string(),
            message: format!("Missing price for disposal {}", tx.id),
            tx_id: Some(tx.id.clone()),
        });
    }

    if let Some(override_proceeds) = tx.override_proceeds {
        proceeds = override_proceeds;
    } else if taxable {
        // For USA, fee reduces proceeds. For Chile, fees are not deductible
        // for personas naturales, so we do not subtract them.
        let fee = match cfg.jurisdiction {
            TaxJurisdiction::Chile => 0.0,
            _ => tx.fee.unwrap_or(0.0),
        };
        proceeds = (proceeds - fee).max(0.0);
    }

    let req = DisposalRequest {
        coin_id: &tx.coin_id,
        amount: tx.amount,
        sale_date: tx_date,
        tx_id: &tx.id,
        proceeds,
        taxable,
    };

    let (allocations, cost_basis, short_gain, long_gain) = consume_lots(report, lots, cfg, &req);

    if taxable
        && is_in_period(cfg.period, tx_date)
        && (tx.price_per_coin.is_some() || tx.override_proceeds.is_some())
    {
        let raw_gain = proceeds - cost_basis;

        // Chile: apply second IPC adjustment to the gain (sale → end of year).
        let gain = apply_gain_ipc_adjustment(report, cfg, tx_date, raw_gain, &tx.id);

        let term = build_term(short_gain, long_gain, cfg.jurisdiction);
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
    cfg: &TaxConfig,
    tx: &CryptoTransaction,
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

    let req = DisposalRequest {
        coin_id: fee_coin_id,
        amount: fee_amount,
        sale_date: tx_date,
        tx_id: &tx.id,
        proceeds,
        taxable: true,
    };

    let (allocations, cost_basis, short_gain, long_gain) = consume_lots(report, lots, cfg, &req);

    if is_in_period(cfg.period, tx_date) && fee_price.is_some() {
        let raw_gain = proceeds - cost_basis;
        let gain = apply_gain_ipc_adjustment(report, cfg, tx_date, raw_gain, &tx.id);

        let term = build_term(short_gain, long_gain, cfg.jurisdiction);
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

// ---------------------------------------------------------------------------
// Lot consumption (core cost-basis engine)
// ---------------------------------------------------------------------------

pub(super) fn consume_lots(
    report: &mut TaxReport,
    lots: &mut HashMap<String, Vec<Lot>>,
    cfg: &TaxConfig,
    req: &DisposalRequest,
) -> (Vec<AllocationInfo>, f64, f64, f64) {
    let mut allocations = Vec::new();
    let mut cost_basis = 0.0;

    let entry = lots.entry(req.coin_id.to_string()).or_default();
    let total_available: f64 = entry.iter().map(|lot| lot.quantity).sum();

    if total_available <= 0.0 {
        if req.taxable {
            report.warnings.push(TaxWarning {
                code: "no_lots".to_string(),
                message: format!("No lots available for {}", req.tx_id),
                tx_id: Some(req.tx_id.to_string()),
            });
        }
        return (allocations, 0.0, 0.0, 0.0);
    }

    let mut remaining = req.amount;

    if cfg.method == TaxMethod::Cpp {
        let (allocs, cost) = consume_lots_cpp(
            report,
            entry,
            cfg,
            req.amount,
            req.sale_date,
            req.tx_id,
            req.taxable,
        );
        allocations = allocs;
        cost_basis = cost;
        let (short_gain, long_gain) =
            split_term_gain(&allocations, req.proceeds, req.sale_date, cfg.jurisdiction);
        return (allocations, cost_basis, short_gain, long_gain);
    }

    let mut indices: Vec<usize> = (0..entry.len()).collect();
    match cfg.method {
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

    let sale_prev_month = prev_month_key(req.sale_date);

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
        let (adjusted_cost, cost_used) = apply_ipc_cost_adjustment(
            report,
            cfg,
            &lot.acquired_prev_month,
            &sale_prev_month,
            base_cost,
            req.tx_id,
            req.taxable,
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
            message: format!("Not enough lots to cover {}", req.tx_id),
            tx_id: Some(req.tx_id.to_string()),
        });
    }

    let (short_gain, long_gain) =
        split_term_gain(&allocations, req.proceeds, req.sale_date, cfg.jurisdiction);
    (allocations, cost_basis, short_gain, long_gain)
}

fn consume_lots_cpp(
    report: &mut TaxReport,
    lots: &mut Vec<Lot>,
    cfg: &TaxConfig,
    amount: f64,
    sale_date: chrono::NaiveDate,
    tx_id: &str,
    warn: bool,
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

    let lot_count = lots.len();
    for (idx, lot) in lots.iter_mut().enumerate() {
        if lot.quantity <= 0.0 {
            continue;
        }
        let share = if idx + 1 == lot_count {
            remaining
        } else {
            amount * (lot.quantity / total_qty)
        };
        let qty = share.min(lot.quantity);
        remaining -= qty;
        lot.quantity -= qty;

        let base_cost = qty * lot.unit_cost;
        let (adjusted_cost, cost_used) = apply_ipc_cost_adjustment(
            report,
            cfg,
            &lot.acquired_prev_month,
            &sale_prev_month,
            base_cost,
            tx_id,
            warn,
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

// ---------------------------------------------------------------------------
// IPC adjustments (Chile only)
// ---------------------------------------------------------------------------

/// **First IPC adjustment** — adjusts the cost basis.
///
/// Reajuste del costo de adquisición por variación del IPC entre el mes
/// anterior a la compra y el mes anterior a la venta.
fn apply_ipc_cost_adjustment(
    report: &mut TaxReport,
    cfg: &TaxConfig,
    buy_prev: &str,
    sale_prev: &str,
    base_cost: f64,
    tx_id: &str,
    warn: bool,
) -> (Option<f64>, f64) {
    if !matches!(cfg.jurisdiction, TaxJurisdiction::Chile) {
        return (None, base_cost);
    }

    if cfg.ipc_map.is_empty() || !warn {
        return (None, base_cost);
    }

    let buy_idx = cfg.ipc_map.get(buy_prev).copied();
    let sale_idx = cfg.ipc_map.get(sale_prev).copied();

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

/// **Second IPC adjustment** — adjusts the realised gain/loss.
///
/// Reajuste de la ganancia/pérdida por variación del IPC entre el mes
/// anterior a la venta y noviembre (mes anterior al cierre del año fiscal,
/// 31 de diciembre).
///
/// For non-Chile jurisdictions this is a no-op and returns the gain unchanged.
pub(super) fn apply_gain_ipc_adjustment(
    report: &mut TaxReport,
    cfg: &TaxConfig,
    sale_date: chrono::NaiveDate,
    raw_gain: f64,
    tx_id: &str,
) -> f64 {
    if !matches!(cfg.jurisdiction, TaxJurisdiction::Chile) {
        return raw_gain;
    }

    if cfg.ipc_map.is_empty() {
        return raw_gain;
    }

    let sale_prev = prev_month_key(sale_date);

    // End-of-year key: November of the fiscal year (month before December 31).
    let year_end_prev = format!("{:04}-11", cfg.period.end.year());

    // If the sale is already in December, sale_prev == November == year_end_prev,
    // so the adjustment factor would be 1.0 (no change), which is correct.
    if sale_prev == year_end_prev {
        return raw_gain;
    }

    let sale_idx = cfg.ipc_map.get(&sale_prev).copied();
    let eoy_idx = cfg.ipc_map.get(&year_end_prev).copied();

    match (sale_idx, eoy_idx) {
        (Some(sale), Some(eoy)) if sale > 0.0 => raw_gain * (eoy / sale),
        _ => {
            report.warnings.push(TaxWarning {
                code: "ipc_missing".to_string(),
                message: format!(
                    "IPC index missing for gain adjustment on {} ({} → {})",
                    tx_id, sale_prev, year_end_prev
                ),
                tx_id: Some(tx_id.to_string()),
            });
            raw_gain
        }
    }
}

// ---------------------------------------------------------------------------
// Term classification (USA + Other)
// ---------------------------------------------------------------------------

fn split_term_gain(
    allocations: &[AllocationInfo],
    proceeds: f64,
    sale_date: chrono::NaiveDate,
    jurisdiction: TaxJurisdiction,
) -> (f64, f64) {
    if !matches!(jurisdiction, TaxJurisdiction::Usa | TaxJurisdiction::Other) {
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
    if !matches!(jurisdiction, TaxJurisdiction::Usa | TaxJurisdiction::Other) {
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

// ---------------------------------------------------------------------------
// Summary helpers
// ---------------------------------------------------------------------------

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

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::super::types::TaxPeriod;
    use super::*;
    use crate::features::crypto::TaxReportSummary;
    use chrono::NaiveDate;
    use std::collections::BTreeMap;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base_report() -> TaxReport {
        TaxReport {
            period_id: "2024".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            jurisdiction: "usa".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary::default(),
            disposals: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn chile_report() -> TaxReport {
        TaxReport {
            period_id: "2024".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            jurisdiction: "chile".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary::default(),
            disposals: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn test_period() -> TaxPeriod {
        TaxPeriod {
            id: "2024".to_string(),
            start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        }
    }

    fn tx(id: &str, kind: &str, amount: f64, price: f64, date: &str) -> CryptoTransaction {
        CryptoTransaction::new(
            id.to_string(),
            "wallet".to_string(),
            "btc".to_string(),
            "BTC".to_string(),
            kind.to_string(),
            amount,
            Some(price),
            None,
            date.to_string(),
            None,
        )
    }

    fn tx_with_fee(
        id: &str,
        kind: &str,
        amount: f64,
        price: f64,
        fee: f64,
        date: &str,
    ) -> CryptoTransaction {
        CryptoTransaction::new(
            id.to_string(),
            "wallet".to_string(),
            "btc".to_string(),
            "BTC".to_string(),
            kind.to_string(),
            amount,
            Some(price),
            Some(fee),
            date.to_string(),
            None,
        )
    }

    // -----------------------------------------------------------------------
    // HIFO ordering
    // -----------------------------------------------------------------------

    #[test]
    fn hifo_uses_highest_cost_lot() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();
        let ipc_map = BTreeMap::new();

        let buy1 = tx("b1", "buy", 1.0, 100.0, "2024-01-05");
        let buy2 = tx("b2", "buy", 1.0, 200.0, "2024-02-05");

        add_lot(
            &mut report,
            &mut lots,
            &buy1,
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );
        add_lot(
            &mut report,
            &mut lots,
            &buy2,
            NaiveDate::from_ymd_opt(2024, 2, 5).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Hifo,
            jurisdiction: TaxJurisdiction::Usa,
            ipc_map: &ipc_map,
        };

        let req = DisposalRequest {
            coin_id: "btc",
            amount: 1.0,
            sale_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            tx_id: "s1",
            proceeds: 300.0,
            taxable: true,
        };

        let (allocs, cost, _, _) = consume_lots(&mut report, &mut lots, &cfg, &req);

        assert_eq!(allocs.len(), 1);
        assert_eq!(allocs[0].allocation.lot_id, "b2");
        assert!(approx_eq(cost, 200.0));
    }

    // -----------------------------------------------------------------------
    // CPP (weighted average)
    // -----------------------------------------------------------------------

    #[test]
    fn cpp_uses_weighted_average_cost() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();
        let ipc_map = BTreeMap::new();

        let buy1 = tx("b1", "buy", 1.0, 100.0, "2024-01-05");
        let buy2 = tx("b2", "buy", 3.0, 300.0, "2024-02-05");

        add_lot(
            &mut report,
            &mut lots,
            &buy1,
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );
        add_lot(
            &mut report,
            &mut lots,
            &buy2,
            NaiveDate::from_ymd_opt(2024, 2, 5).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Cpp,
            jurisdiction: TaxJurisdiction::Usa,
            ipc_map: &ipc_map,
        };

        let req = DisposalRequest {
            coin_id: "btc",
            amount: 2.0,
            sale_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            tx_id: "s1",
            proceeds: 600.0,
            taxable: true,
        };

        let (_, cost, _, _) = consume_lots(&mut report, &mut lots, &cfg, &req);

        // 1×100 + 3×300 = 1000 total for 4 units → avg 250/unit → 2 × 250 = 500
        assert!(approx_eq(cost, 500.0));
    }

    // -----------------------------------------------------------------------
    // IPC cost adjustment (first adjustment)
    // -----------------------------------------------------------------------

    #[test]
    fn ipc_cost_adjustment_applies_for_chile() {
        let mut report = chile_report();
        let mut ipc = BTreeMap::new();
        ipc.insert("2023-12".to_string(), 100.0);
        ipc.insert("2024-01".to_string(), 110.0);
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc,
        };

        let (adjusted, cost) =
            apply_ipc_cost_adjustment(&mut report, &cfg, "2023-12", "2024-01", 100.0, "tx1", true);

        assert!(adjusted.map(|v| approx_eq(v, 110.0)).unwrap_or(false));
        assert!(approx_eq(cost, 110.0));
    }

    #[test]
    fn ipc_missing_emits_warning() {
        let mut report = chile_report();
        let ipc = BTreeMap::new();
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc,
        };

        let (adjusted, cost) =
            apply_ipc_cost_adjustment(&mut report, &cfg, "2023-12", "2024-01", 100.0, "tx1", true);

        assert!(adjusted.is_none());
        assert!(approx_eq(cost, 100.0));
    }

    // -----------------------------------------------------------------------
    // IPC gain adjustment (second adjustment)
    // -----------------------------------------------------------------------

    #[test]
    fn gain_ipc_adjustment_applies_for_chile() {
        let mut report = chile_report();
        let mut ipc = BTreeMap::new();
        // Sale in August → sale_prev = "2024-07"
        // End of year prev = "2024-11" (November)
        ipc.insert("2024-07".to_string(), 120.0);
        ipc.insert("2024-11".to_string(), 122.0);
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc,
        };

        let sale_date = NaiveDate::from_ymd_opt(2024, 8, 15).unwrap();
        let adjusted = apply_gain_ipc_adjustment(&mut report, &cfg, sale_date, 354.5, "tx1");

        // 354.5 × (122 / 120) = 354.5 × 1.01667 ≈ 360.41
        let expected = 354.5 * (122.0 / 120.0);
        assert!(approx_eq(adjusted, expected));
    }

    #[test]
    fn gain_ipc_adjustment_noop_for_december_sale() {
        let mut report = chile_report();
        let mut ipc = BTreeMap::new();
        // Sale in December → sale_prev = "2024-11" == year_end_prev
        ipc.insert("2024-11".to_string(), 122.0);
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc,
        };

        let sale_date = NaiveDate::from_ymd_opt(2024, 12, 10).unwrap();
        let adjusted = apply_gain_ipc_adjustment(&mut report, &cfg, sale_date, 500.0, "tx1");

        assert!(approx_eq(adjusted, 500.0));
    }

    #[test]
    fn gain_ipc_adjustment_noop_for_usa() {
        let mut report = base_report();
        let ipc = BTreeMap::new();
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Usa,
            ipc_map: &ipc,
        };

        let sale_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let adjusted = apply_gain_ipc_adjustment(&mut report, &cfg, sale_date, 200.0, "tx1");

        assert!(approx_eq(adjusted, 200.0));
    }

    // -----------------------------------------------------------------------
    // Chile: fees excluded from cost basis
    // -----------------------------------------------------------------------

    #[test]
    fn chile_fee_excluded_from_cost_basis() {
        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        let buy = tx_with_fee("b1", "buy", 1.0, 100.0, 5.0, "2024-01-10");

        add_lot(
            &mut report,
            &mut lots,
            &buy,
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            TaxJurisdiction::Chile,
            None,
        );

        let lot = &lots["btc"][0];
        // Chile: cost = amount × price (no fee) = 1.0 × 100.0 = 100.0
        assert!(approx_eq(lot.unit_cost, 100.0));
    }

    #[test]
    fn usa_fee_included_in_cost_basis() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        let buy = tx_with_fee("b1", "buy", 1.0, 100.0, 5.0, "2024-01-10");

        add_lot(
            &mut report,
            &mut lots,
            &buy,
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );

        let lot = &lots["btc"][0];
        // USA: cost = (amount × price) + fee = 100.0 + 5.0 = 105.0
        assert!(approx_eq(lot.unit_cost, 105.0));
    }

    // -----------------------------------------------------------------------
    // Chile: airdrops / staking have zero cost
    // -----------------------------------------------------------------------

    #[test]
    fn chile_airdrop_has_zero_cost() {
        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        // Airdrop with a known market price — Chile should still use $0.
        let airdrop = tx("a1", "buy", 10.0, 50.0, "2024-03-01");

        add_lot(
            &mut report,
            &mut lots,
            &airdrop,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            TaxJurisdiction::Chile,
            Some("airdrop"),
        );

        let lot = &lots["btc"][0];
        assert!(approx_eq(lot.unit_cost, 0.0));
        // No "missing_price" warning should be emitted for zero-cost items.
        assert!(!report.warnings.iter().any(|w| w.code == "missing_price"));
    }

    #[test]
    fn chile_staking_has_zero_cost() {
        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        let staking = tx("s1", "buy", 5.0, 200.0, "2024-04-01");

        add_lot(
            &mut report,
            &mut lots,
            &staking,
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            TaxJurisdiction::Chile,
            Some("staking"),
        );

        let lot = &lots["btc"][0];
        assert!(approx_eq(lot.unit_cost, 0.0));
    }

    #[test]
    fn usa_airdrop_uses_fmv() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        let airdrop = tx("a1", "buy", 10.0, 50.0, "2024-03-01");

        add_lot(
            &mut report,
            &mut lots,
            &airdrop,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            TaxJurisdiction::Usa,
            Some("airdrop"),
        );

        let lot = &lots["btc"][0];
        // USA: FMV is cost basis = 10.0 × 50.0 = 500.0 → 50.0/unit
        assert!(approx_eq(lot.unit_cost, 50.0));
    }

    // -----------------------------------------------------------------------
    // Full Chile example from LedgiFi guide
    // -----------------------------------------------------------------------

    #[test]
    fn chile_full_example_fifo_with_double_ipc() {
        // Reproduces the LedgiFi example:
        // Buy 1 BTC on 2024-01-05 at $1,000
        // Buy 1 BTC on 2024-02-21 at $1,200
        // Sell 1.5 BTC in August 2024 at $2,000
        // IPC adjustments bring the cost up and the gain is further adjusted.

        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();

        let mut ipc = BTreeMap::new();
        // Made-up IPC values that match the LedgiFi percentages:
        // Jan cost adj 3.1% → IPC dec2023=100, IPC jul2024=103.1
        // Feb cost adj 2.4% → IPC jan2024=100, IPC jul2024=102.4
        // Gain adj 1.6% → IPC jul2024 → IPC nov2024
        ipc.insert("2023-12".to_string(), 100.0);
        ipc.insert("2024-01".to_string(), 100.0);
        ipc.insert("2024-07".to_string(), 103.1); // sale_prev for August sale
        ipc.insert("2024-11".to_string(), 104.75); // ~1.6% above 103.1

        // For the Feb buy, we need IPC jan → jul: 100 → 102.4
        // But we already have 2024-01 = 100.0 and 2024-07 = 103.1.
        // The LedgiFi example uses different % per lot. Let's use exact ratios.
        // To match exactly: buy1 cost adj = 1000 × (103.1/100) = 1031
        //                   buy2 cost adj = 1200 × (103.1/100) = 1237.2
        // But LedgiFi says buy2 adj = 1229 (2.4%), meaning IPC jan=100, IPC jul=102.4
        // This shows each lot uses its OWN buy_prev month.
        // buy1: prev_month = 2023-12 (dec), sale_prev = 2024-07 → 100 → 103.1 ✓
        // buy2: prev_month = 2024-01 (jan), sale_prev = 2024-07 → 100 → 103.1
        //   but LedgiFi says 2.4% for buy2, not 3.1%. This is because each month
        //   has different IPC values. Let's adjust to match the example exactly.
        // We'll set IPC values to produce the exact LedgiFi numbers.
        let mut ipc_exact = BTreeMap::new();
        ipc_exact.insert("2023-12".to_string(), 100.0); // buy1 prev
        ipc_exact.insert("2024-01".to_string(), 100.0); // buy2 prev
        ipc_exact.insert("2024-07".to_string(), 103.1); // sale prev (Aug sale)
        ipc_exact.insert("2024-11".to_string(), 104.7496); // gain adj: 1.6% above 103.1

        // Recalculate: buy2 adj would be 1200 × (103.1/100) = 1237.2 not 1229.
        // The difference is that LedgiFi uses DIFFERENT IPC per buy month.
        // Let's just use 102.4 for buy2's ratio → IPC jan=100, IPC jul would need
        // to be 102.4 for buy2. But it's the SAME sale month for both.
        // Actually the IPC is a single series. The difference in % comes from
        // different buy months having different IPC values.
        // buy1 dec2023 → jul2024: 3.1% means IPC_dec = X, IPC_jul = X × 1.031
        // buy2 jan2024 → jul2024: 2.4% means IPC_jan = Y, IPC_jul = Y × 1.024
        // So IPC_dec/IPC_jan ratio = (IPC_jul/1.031) / (IPC_jul/1.024)
        // = 1.024/1.031 ≈ 0.9932
        // Let's set: IPC_dec=99.32, IPC_jan=100.0, IPC_jul=102.4
        let mut ipc_ledgifi = BTreeMap::new();
        ipc_ledgifi.insert("2023-12".to_string(), 99.3219); // so 102.4/99.3219 ≈ 1.031
        ipc_ledgifi.insert("2024-01".to_string(), 100.0);
        ipc_ledgifi.insert("2024-07".to_string(), 102.4);
        ipc_ledgifi.insert("2024-11".to_string(), 104.0384); // 102.4 × 1.016

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc_ledgifi,
        };

        let buy1 = tx("b1", "buy", 1.0, 1000.0, "2024-01-05");
        let buy2 = tx("b2", "buy", 1.0, 1200.0, "2024-02-21");

        add_lot(
            &mut report,
            &mut lots,
            &buy1,
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            TaxJurisdiction::Chile,
            None,
        );
        add_lot(
            &mut report,
            &mut lots,
            &buy2,
            NaiveDate::from_ymd_opt(2024, 2, 21).unwrap(),
            TaxJurisdiction::Chile,
            None,
        );

        // Sell 1.5 BTC in August
        let mut sell = tx("s1", "sell", 1.5, 1333.3333, "2024-08-15");
        // Total proceeds = 1.5 × 1333.3333 ≈ 2000
        sell.override_proceeds = Some(2000.0);

        let sale_date = NaiveDate::from_ymd_opt(2024, 8, 15).unwrap();
        apply_disposal(&mut report, &mut lots, &cfg, &sell, sale_date, true);

        assert_eq!(report.disposals.len(), 1);
        let d = &report.disposals[0];

        // FIFO: consume 1.0 BTC from buy1 + 0.5 BTC from buy2
        // buy1 adj cost: 1000 × (102.4 / 99.3219) ≈ 1031.0
        // buy2 adj cost for 0.5: 0.5 × 1200 × (102.4 / 100.0) = 0.5 × 1228.8 ≈ 614.4
        // total cost ≈ 1031 + 614.4 = 1645.4
        // raw gain = 2000 - 1645.4 = 354.6
        // gain adj = 354.6 × (104.0384 / 102.4) ≈ 354.6 × 1.016 ≈ 360.3
        assert!(d.proceeds == 2000.0);
        assert!(d.cost_basis > 1640.0 && d.cost_basis < 1650.0);
        // The gain should be the IPC-adjusted value, roughly ~360
        assert!(d.gain > 355.0 && d.gain < 365.0);

        // term should be None for Chile
        assert!(d.term.is_none());
    }
}
