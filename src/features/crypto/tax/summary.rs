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

//! Tax summary payloads for UI.

use super::report::TaxReport;

#[derive(Debug, Clone)]
pub struct TaxReadinessItem {
    pub code: String,
    pub status: String, // ok, warn, error
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct TaxSummaryPayload {
    pub report: TaxReport,
    pub taxable_income_total: f64,
    pub taxable_income_count: usize,
    pub end_balance_value: Option<f64>,
    pub end_balance_missing: usize,
    pub transactions_in_period: usize,
    pub volume_processed: f64,
    pub readiness: Vec<TaxReadinessItem>,
}
