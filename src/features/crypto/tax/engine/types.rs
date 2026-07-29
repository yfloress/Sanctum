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

//! Tax engine internal types.

use super::super::report::LotAllocation;
use crate::features::crypto::tax::rules::JurisdictionRules;
use crate::features::crypto::tax::{TaxJurisdiction, TaxMethod};
use chrono::NaiveDate;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(super) struct Lot {
    pub lot_id: String,
    pub acquired_date: NaiveDate,
    pub acquired_date_raw: String,
    pub acquired_prev_month: String,
    pub quantity: f64,
    pub unit_cost: f64,
}

#[derive(Clone, Debug)]
pub(super) struct AllocationInfo {
    pub allocation: LotAllocation,
    pub lot_date: NaiveDate,
}

#[derive(Clone, Debug)]
pub(crate) struct TaxPeriod {
    pub id: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

/// Immutable configuration shared across all engine operations in a single report run.
pub(super) struct TaxConfig<'a> {
    pub period: &'a TaxPeriod,
    pub method: TaxMethod,
    pub jurisdiction: TaxJurisdiction,
    pub ipc_map: &'a BTreeMap<String, f64>,
}

impl TaxConfig<'_> {
    /// The jurisdiction's rule strategy for this report run.
    pub(super) fn rules(&self) -> &'static dyn JurisdictionRules {
        self.jurisdiction.rules()
    }
}

/// Parameters describing a single lot-consumption (disposal) request.
pub(super) struct DisposalRequest<'a> {
    pub coin_id: &'a str,
    pub amount: f64,
    pub sale_date: NaiveDate,
    pub tx_id: &'a str,
    pub proceeds: f64,
    pub taxable: bool,
}
