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

//! Tax engine internal types.

use super::super::report::LotAllocation;
use chrono::NaiveDate;

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
pub(super) struct TaxPeriod {
    pub id: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
}
