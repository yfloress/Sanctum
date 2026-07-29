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

//! Lot management and disposal calculations.

use super::period::{is_in_period, parse_date, prev_month_key};
use super::types::{AllocationInfo, DisposalRequest, Lot, TaxConfig};
use crate::features::crypto::tax::{TaxJurisdiction, TaxMethod};
use crate::features::crypto::{TaxDisposal, TaxReport, TaxWarning};
use crate::models::CryptoTransaction;
use chrono::Datelike;
use std::collections::HashMap;

fn is_stablecoin_symbol(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_uppercase().as_str(),
        "USDT"
            | "USDC"
            | "BUSD"
            | "DAI"
            | "TUSD"
            | "FDUSD"
            | "USDD"
            | "USDP"
            | "PYUSD"
            | "UST"
            | "FRAX"
    )
}

fn is_stablecoin_coin_id(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "tether"
            | "usd-coin"
            | "binance-usd"
            | "dai"
            | "true-usd"
            | "first-digital-usd"
            | "usdd"
            | "pax-dollar"
            | "paypal-usd"
            | "terrausd"
            | "frax"
    )
}

fn has_non_usd_quote_marker(tx: &CryptoTransaction) -> bool {
    tx.notes
        .as_deref()
        .map(|notes| notes.contains("tax_reason=non_usd_quote"))
        .unwrap_or(false)
}

fn missing_price_code(tx: &CryptoTransaction) -> &'static str {
    if has_non_usd_quote_marker(tx) {
        "missing_price_non_usd_quote"
    } else {
        "missing_price"
    }
}

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
    subtype: Option<&str>,
) {
    let rules = jurisdiction.rules();

    // Some regimes recognise certain income subtypes at cost $0 (e.g. Chile
    // airdrops, staking and forks).
    let force_zero_cost = rules.is_zero_cost_income(subtype);

    let price = if force_zero_cost {
        0.0
    } else {
        tx.price_per_coin.unwrap_or(0.0)
    };

    if !force_zero_cost && tx.price_per_coin.is_none() && tx.override_cost_basis.is_none() {
        let code = missing_price_code(tx);
        report.warnings.push(TaxWarning {
            code: code.to_string(),
            message: format!("Missing price for acquisition {}", tx.id),
            tx_id: Some(tx.id.clone()),
        });
    }

    // Some regimes exclude fees from the cost basis (e.g. Chile personas naturales).
    let fee = if rules.fee_in_cost_basis() {
        tx.fee.unwrap_or(0.0)
    } else {
        0.0
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
        let code = missing_price_code(tx);
        report.warnings.push(TaxWarning {
            code: code.to_string(),
            message: format!("Missing price for disposal {}", tx.id),
            tx_id: Some(tx.id.clone()),
        });
    }

    if let Some(override_proceeds) = tx.override_proceeds {
        proceeds = override_proceeds;
    } else if taxable && cfg.rules().deduct_disposal_fees() {
        // Business-style regimes deduct disposal commissions from proceeds.
        // Chile persona natural (art. 17 N°8 m) does not deduct these in the
        // mayor valor determination.
        let fee = tx.fee.unwrap_or(0.0);
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
            disposal_type: tx.mechanical_type().to_string(),
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
    if fee_coin_id.trim().eq_ignore_ascii_case("usd")
        || is_stablecoin_symbol(fee_coin_id)
        || is_stablecoin_coin_id(fee_coin_id)
    {
        fee_price = Some(1.0);
    } else if fee_coin_id == tx.coin_id && tx.subtype.as_deref() != Some("swap") {
        fee_price = tx.price_per_coin;
    }

    if fee_price.is_none()
        && let Some(fee_usd) = tx.fee
        && fee_usd > 0.0
    {
        let inferred = fee_usd / fee_amount;
        if inferred.is_finite() && inferred > 0.0 {
            fee_price = Some(inferred);
        }
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
        return (Vec::new(), 0.0, 0.0, 0.0);
    }

    // The method strategy owns *how* lots are consumed; the jurisdiction owns
    // *whether* the resulting gain is split into holding-term buckets.
    let (allocations, cost_basis) = cfg.method.selection().consume(report, entry, cfg, req);

    let (short_gain, long_gain) =
        split_term_gain(&allocations, req.proceeds, req.sale_date, cfg.jurisdiction);
    (allocations, cost_basis, short_gain, long_gain)
}

// ---------------------------------------------------------------------------
// Lot selection strategies (cost-basis method)
// ---------------------------------------------------------------------------

/// Strategy for consuming acquisition lots on a disposal.
///
/// FIFO/LIFO/HIFO drain lots sequentially in a method-specific order; CPP
/// (average cost) spreads the disposal proportionally across all open lots.
/// Adding a method means writing one impl and one factory arm
/// ([`TaxMethod::selection`]).
pub(super) trait LotSelection {
    /// Consumes `req.amount` of the coin from `lots`, returning the per-lot
    /// allocations and the total (IPC-adjusted) cost basis.
    ///
    /// The caller ([`consume_lots`]) guarantees `lots` is non-empty with a
    /// positive total quantity.
    fn consume(
        &self,
        report: &mut TaxReport,
        lots: &mut Vec<Lot>,
        cfg: &TaxConfig,
        req: &DisposalRequest,
    ) -> (Vec<AllocationInfo>, f64);
}

struct Fifo;
struct Lifo;
struct Hifo;
struct AverageCost;

impl TaxMethod {
    /// Returns the [`LotSelection`] strategy for this method.
    ///
    /// This is the single point that maps the method enum to a consumption
    /// algorithm; every disposal dispatches polymorphically through it.
    pub(super) fn selection(self) -> &'static dyn LotSelection {
        match self {
            TaxMethod::Fifo => &Fifo,
            TaxMethod::Lifo => &Lifo,
            TaxMethod::Hifo => &Hifo,
            TaxMethod::Cpp => &AverageCost,
        }
    }
}

impl LotSelection for Fifo {
    fn consume(
        &self,
        report: &mut TaxReport,
        lots: &mut Vec<Lot>,
        cfg: &TaxConfig,
        req: &DisposalRequest,
    ) -> (Vec<AllocationInfo>, f64) {
        let order: Vec<usize> = (0..lots.len()).collect();
        drain_in_order(report, lots, cfg, req, order)
    }
}

impl LotSelection for Lifo {
    fn consume(
        &self,
        report: &mut TaxReport,
        lots: &mut Vec<Lot>,
        cfg: &TaxConfig,
        req: &DisposalRequest,
    ) -> (Vec<AllocationInfo>, f64) {
        let mut order: Vec<usize> = (0..lots.len()).collect();
        order.reverse();
        drain_in_order(report, lots, cfg, req, order)
    }
}

impl LotSelection for Hifo {
    fn consume(
        &self,
        report: &mut TaxReport,
        lots: &mut Vec<Lot>,
        cfg: &TaxConfig,
        req: &DisposalRequest,
    ) -> (Vec<AllocationInfo>, f64) {
        let sale_prev_month = prev_month_key(req.sale_date);
        let mut order: Vec<usize> = (0..lots.len()).collect();
        order.sort_by(|a, b| {
            let a_cost = hifo_effective_unit_cost(&lots[*a], cfg, &sale_prev_month);
            let b_cost = hifo_effective_unit_cost(&lots[*b], cfg, &sale_prev_month);

            b_cost
                .partial_cmp(&a_cost)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| lots[*a].acquired_date.cmp(&lots[*b].acquired_date))
        });
        drain_in_order(report, lots, cfg, req, order)
    }
}

impl LotSelection for AverageCost {
    fn consume(
        &self,
        report: &mut TaxReport,
        lots: &mut Vec<Lot>,
        cfg: &TaxConfig,
        req: &DisposalRequest,
    ) -> (Vec<AllocationInfo>, f64) {
        consume_lots_cpp(
            report,
            lots,
            cfg,
            req.amount,
            req.sale_date,
            req.tx_id,
            req.taxable,
        )
    }
}

/// Sequentially drains `lots` in the given index `order`, accumulating
/// allocations and IPC-adjusted cost basis. Emits an `insufficient_lots`
/// warning when the order cannot cover `req.amount`.
fn drain_in_order(
    report: &mut TaxReport,
    lots: &mut Vec<Lot>,
    cfg: &TaxConfig,
    req: &DisposalRequest,
    order: Vec<usize>,
) -> (Vec<AllocationInfo>, f64) {
    let sale_prev_month = prev_month_key(req.sale_date);
    let mut allocations = Vec::new();
    let mut cost_basis = 0.0;
    let mut remaining = req.amount;

    for idx in order {
        if remaining <= 0.0 {
            break;
        }
        if idx >= lots.len() {
            continue;
        }
        let lot = &mut lots[idx];
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

    lots.retain(|lot| lot.quantity > 0.0);

    if remaining > 0.0 {
        report.warnings.push(TaxWarning {
            code: "insufficient_lots".to_string(),
            message: format!("Not enough lots to cover {}", req.tx_id),
            tx_id: Some(req.tx_id.to_string()),
        });
    }

    (allocations, cost_basis)
}

fn hifo_effective_unit_cost(lot: &Lot, cfg: &TaxConfig, sale_prev_month: &str) -> f64 {
    if !cfg.rules().inflation_indexed() || cfg.ipc_map.is_empty() {
        return lot.unit_cost;
    }

    let buy_idx = cfg.ipc_map.get(&lot.acquired_prev_month).copied();
    let sale_idx = cfg.ipc_map.get(sale_prev_month).copied();

    match (buy_idx, sale_idx) {
        (Some(buy), Some(sale)) if buy > 0.0 => {
            let adjusted = lot.unit_cost * (sale / buy);
            if adjusted.is_finite() && adjusted > 0.0 {
                adjusted
            } else {
                lot.unit_cost
            }
        }
        _ => lot.unit_cost,
    }
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
    if !cfg.rules().inflation_indexed() {
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
/// anterior a la venta y noviembre (mes anterior al cierre del año tributario,
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
    if !cfg.rules().inflation_indexed() {
        return raw_gain;
    }

    if cfg.ipc_map.is_empty() {
        return raw_gain;
    }

    let sale_prev = prev_month_key(sale_date);

    // End-of-year key: November of the tax year (month before December 31).
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
    if !jurisdiction.rules().classifies_holding_term() {
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
        let alloc_proceeds = proceeds_per_unit * alloc.allocation.quantity;
        let alloc_cost = alloc
            .allocation
            .adjusted_cost
            .unwrap_or(alloc.allocation.cost);
        let gain = alloc_proceeds - alloc_cost;

        if is_long_term(alloc.lot_date, sale_date) {
            long_gain += gain;
        } else {
            short_gain += gain;
        }
    }

    (short_gain, long_gain)
}

/// Long-term capital gain test (USA / generic): the asset must be held for
/// **more than one year**, measured by calendar anniversary. The holding
/// period begins the day after acquisition, so this uses the one-year
/// anniversary date rather than a flat 365-day count — keeping leap years and
/// the exact one-year boundary correct.
fn is_long_term(acquired: chrono::NaiveDate, sale: chrono::NaiveDate) -> bool {
    match acquired.checked_add_months(chrono::Months::new(12)) {
        Some(anniversary) => sale > anniversary,
        None => (sale - acquired).num_days() > 365,
    }
}

pub(super) fn build_term(
    short_gain: f64,
    long_gain: f64,
    jurisdiction: TaxJurisdiction,
) -> Option<String> {
    if !jurisdiction.rules().classifies_holding_term() {
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
mod tests;
