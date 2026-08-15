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

//! Finance command objects (CQRS-lite inputs).
//!
//! Domain-owned, already-parsed inputs for the mutating [`super::FinanceService`]
//! operations. They replace long positional parameter lists and keep the service
//! decoupled from the IPC DTO layer (`crate::ui::dto`): the command layer maps
//! `ui::dto::*Input` (raw, stringly amounts) into these structs, validating and
//! tagging the offending field along the way. Amounts arrive here as `i64` cents.

/// Create a new financial account.
#[derive(Debug, Clone)]
pub struct NewAccount {
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub initial_balance_cents: i64,
    pub color: String,
    pub icon: Option<String>,
}

/// Update an existing financial account.
#[derive(Debug, Clone)]
pub struct UpdateAccount {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub initial_balance_cents: i64,
    pub color: String,
    pub icon: Option<String>,
}

/// Add a new transaction.
#[derive(Debug, Clone)]
pub struct NewTransaction {
    pub account_id: String,
    pub amount_cents: i64,
    pub category: String,
    pub description: String,
    pub date: String,
    pub is_expense: bool,
}

/// Update an existing transaction.
#[derive(Debug, Clone)]
pub struct UpdateTransaction {
    pub id: String,
    pub account_id: String,
    pub amount_cents: i64,
    pub category: String,
    pub description: String,
    pub date: String,
    pub is_expense: bool,
}

/// Transfer funds between two accounts.
#[derive(Debug, Clone)]
pub struct NewTransfer {
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount_cents: i64,
    pub description: String,
    pub date: String,
}

/// Update an existing transfer.
#[derive(Debug, Clone)]
pub struct UpdateTransfer {
    pub id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount_cents: i64,
    pub description: String,
    pub date: String,
}

/// Create a credit: a debt repaid over a fixed number of dated payments.
///
/// The fields come in two sets because a borrower is told one of two things.
/// `installments` uses `installment_amount_cents` and `cash_price_cents`;
/// `loan` uses `principal_cents` and `monthly_rate_bp`, with the installment
/// derived from them and possibly corrected against the contract.
#[derive(Debug, Clone)]
pub struct NewCredit {
    pub account_id: String,
    pub name: String,
    pub category: String,
    /// `installments` or `loan`.
    pub kind: String,
    /// Paid up front, before the schedule starts. Zero when there is none.
    pub down_payment_cents: i64,
    /// ISO date the down payment is handed over. Defaults to `first_due_date`.
    pub down_payment_date: Option<String>,
    pub installment_amount_cents: i64,
    pub installment_count: i32,
    /// ISO date the first installment falls due.
    pub first_due_date: String,
    /// `installments` only: what it would have cost paid outright, when known.
    pub cash_price_cents: Option<i64>,
    /// `loan` only: the amount actually financed.
    pub principal_cents: Option<i64>,
    /// `loan` only: the monthly rate in millionths (a monthly 1.5% is 15000).
    pub monthly_rate_ppm: Option<i64>,
}

/// Correct one unpaid row of a credit's schedule.
#[derive(Debug, Clone)]
pub struct UpdateInstallment {
    pub installment_id: String,
    pub amount_cents: i64,
    pub due_date: String,
}

/// Record a fee the lender charged on top of a credit's plan.
#[derive(Debug, Clone)]
pub struct NewCharge {
    pub credit_id: String,
    pub amount_cents: i64,
    pub date: String,
    /// Why it was charged, in the user's own words.
    pub note: String,
}

/// Create a recurring transaction rule.
#[derive(Debug, Clone)]
pub struct NewRecurring {
    pub account_id: String,
    pub amount_cents: i64,
    pub category: String,
    pub description: String,
    /// `weekly`, `monthly` or `yearly`.
    pub frequency: String,
    /// ISO date of the first occurrence.
    pub first_date: String,
    pub is_expense: bool,
}
