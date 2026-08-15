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
//! Credit operations
//!
//! A credit here is any debt repaid over a fixed number of dated payments. It
//! comes in the two forms a borrower is ever told, never both at once:
//!
//! - `Installments`: the payment is quoted ("12 x 25.000") and the rate is
//!   buried inside it. Card and store plans, buy-now-pay-later, and flat-rate
//!   lending all describe themselves this way.
//! - `Loan`: a principal at a rate, from which the payment follows. Ordinary
//!   amortising lending.
//!
//! Deliberately not modelled here: revolving credit, which has no term and no
//! schedule and belongs to an account rather than to a plan.
//!
//! Nothing in this module is particular to one country. No tax, insurance or
//! late-fee rule is encoded, and a fee the lender adds is recorded as the user
//! was actually charged rather than predicted from a local rule. The rate is
//! held as a plain monthly figure, which every market can be converted into.
//!
//! Paying a row writes an ordinary expense into the ledger. That is the whole
//! integration: budgets, charts and the activity list count it like any other
//! expense, and the row simply remembers which transaction it was.

use chrono::{Months, NaiveDate};
use std::collections::HashMap;
use uuid::Uuid;

use crate::db::{Database, DbError};
use crate::models::{Credit, CreditInstallment, CreditKind, InstallmentKind, Transaction};

use super::FinanceError;
use super::commands::{NewCharge, NewCredit, UpdateInstallment};
use super::repository::FinanceRepository;
use super::validation::{
    MAX_CATEGORY_LENGTH, MAX_CREDIT_NAME_LENGTH, MAX_INSTALLMENTS, MAX_MONTHLY_RATE_PPM,
    sanitize_string, sanitize_text, validate_date, validate_field_length, validate_uuid,
};

/// Rates are held in millionths, so this many make the whole.
const PPM_PER_UNIT: f64 = 1_000_000.0;

/// Where a credit stands against its own schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditStatus {
    /// Every payment of the plan is made.
    Done,
    /// At least one payment is past its date and unmade.
    Overdue,
    /// Paid further than the calendar asks for.
    Ahead,
    /// Everything owed so far is paid, and nothing beyond it.
    OnTrack,
}

impl CreditStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Overdue => "overdue",
            Self::Ahead => "ahead",
            Self::OnTrack => "on_track",
        }
    }
}

/// A credit's progress, measured against a given day.
#[derive(Debug, Clone)]
pub struct CreditProgress {
    /// Installments paid, not counting the down payment or any charge.
    pub paid_count: usize,
    /// Plan rows that are unpaid and past their date.
    pub overdue_count: usize,
    /// Date of the first plan row still to pay.
    pub next_due_date: Option<String>,
    pub status: CreditStatus,
}

/// What a credit adds up to, in cents.
#[derive(Debug, Clone, Copy, Default)]
pub struct CreditTotals {
    /// The plan itself: down payment plus every installment.
    pub plan: i64,
    /// How much of the plan is paid.
    pub paid: i64,
    /// Fees the lender added on top, whether paid or not.
    pub charges: i64,
}

impl CreditTotals {
    pub fn remaining(&self) -> i64 {
        self.plan - self.paid
    }

    /// Share of the plan paid, capped at 100, for a progress bar.
    ///
    /// Measured in money rather than in rows, because a schedule may be
    /// irregular: with a balloon final payment, counting rows would show a plan
    /// as nearly finished while most of the money is still owed.
    pub fn percentage(&self) -> f32 {
        if self.plan <= 0 {
            return 0.0;
        }
        ((self.paid as f64 / self.plan as f64) * 100.0).clamp(0.0, 100.0) as f32
    }
}

/// One line of an amortisation table.
#[derive(Debug, Clone)]
pub struct AmortizationRow {
    pub number: i32,
    pub due_date: String,
    pub payment: i64,
    /// The part of the payment that only rents the money.
    pub interest: i64,
    /// The part that actually reduces the debt.
    pub principal: i64,
    /// What is still owed after this payment.
    pub balance: i64,
}

/// The dates of a whole schedule, one payment per month from `first_due`.
///
/// Every date is offset from the first one rather than from the previous one,
/// so a plan anchored on the 31st falls back to the 28th in February and
/// returns to the 31st afterwards instead of drifting a few days each month.
pub fn schedule_dates(first_due: NaiveDate, count: i32) -> Vec<NaiveDate> {
    (0..count.max(0))
        .filter_map(|step| first_due.checked_add_months(Months::new(step as u32)))
        .collect()
}

/// The level payment that repays `principal` over `count` months at `rate_bp`.
///
/// The constant-payment ("French") method, which is what amortising consumer
/// lending uses across markets. It is a **suggestion**: rounding, day-count and
/// fee conventions differ from lender to lender, so the figure on the contract
/// wins and the caller may override it.
pub fn french_installment(principal: i64, monthly_rate_ppm: i64, count: i32) -> i64 {
    if principal <= 0 || count < 1 {
        return 0;
    }
    // An interest-free plan is just the principal split evenly, and the general
    // formula divides by zero there.
    if monthly_rate_ppm <= 0 {
        return principal.div_euclid(count as i64)
            + i64::from(principal.rem_euclid(count as i64) > 0);
    }

    let rate = monthly_rate_ppm as f64 / PPM_PER_UNIT;
    let factor = (1.0 + rate).powi(-count);
    let payment = principal as f64 * rate / (1.0 - factor);
    payment.round() as i64
}

/// Splits each payment of `rows` into interest and principal.
///
/// Walks the schedule the way a lender does: interest is charged on what is
/// still owed, and whatever the payment has left over reduces the debt. Since
/// the rows carry their own amounts, an irregular schedule — a grace period, a
/// balloon final payment, a payment the user corrected — comes out right
/// without any special case.
///
/// The last balance need not land exactly on zero. That residue is real and is
/// reported rather than hidden: rounding and a corrected payment both cause it.
pub fn amortization(
    principal: i64,
    monthly_rate_ppm: i64,
    rows: &[CreditInstallment],
) -> Vec<AmortizationRow> {
    let rate = monthly_rate_ppm.max(0) as f64 / PPM_PER_UNIT;
    let mut balance = principal;

    rows.iter()
        .filter(|row| row.get_kind() == InstallmentKind::Installment)
        .map(|row| {
            let interest = (balance as f64 * rate).round() as i64;
            // Can go negative when a payment does not even cover the interest,
            // which is exactly what makes such a debt grow. Left as it falls.
            let principal_part = row.amount - interest;
            balance -= principal_part;
            AmortizationRow {
                number: row.number,
                due_date: row.due_date.clone(),
                payment: row.amount,
                interest,
                principal: principal_part,
                balance,
            }
        })
        .collect()
}

/// Adds up a credit from its rows.
pub fn credit_totals(rows: &[CreditInstallment]) -> CreditTotals {
    let mut totals = CreditTotals::default();
    for row in rows {
        if row.is_plan() {
            totals.plan += row.amount;
            if row.is_paid() {
                totals.paid += row.amount;
            }
        } else {
            totals.charges += row.amount;
        }
    }
    totals
}

/// Measures the plan against `today` (an ISO date).
///
/// Charges are left out: a fee the lender added is money to pay, but it is not
/// part of the plan and cannot decide whether the plan is on schedule.
///
/// Dates are ISO text, which compares lexicographically, so no parsing is
/// needed to know what has fallen due.
pub fn credit_progress(rows: &[CreditInstallment], today: &str) -> CreditProgress {
    let plan: Vec<&CreditInstallment> = rows.iter().filter(|row| row.is_plan()).collect();

    let paid_count = plan
        .iter()
        .filter(|row| row.is_paid() && row.get_kind() == InstallmentKind::Installment)
        .count();
    let overdue_count = plan
        .iter()
        .filter(|row| !row.is_paid() && row.due_date.as_str() < today)
        .count();
    // Counted over the whole plan rather than the unpaid part: it is what the
    // calendar has asked for so far, regardless of what was paid.
    let due_so_far = plan
        .iter()
        .filter(|row| row.due_date.as_str() <= today)
        .count();
    let paid_rows = plan.iter().filter(|row| row.is_paid()).count();

    let next_due_date = plan
        .iter()
        .find(|row| !row.is_paid())
        .map(|row| row.due_date.clone());

    let status = if !plan.is_empty() && paid_rows == plan.len() {
        CreditStatus::Done
    } else if overdue_count > 0 {
        CreditStatus::Overdue
    } else if paid_rows > due_so_far {
        CreditStatus::Ahead
    } else {
        CreditStatus::OnTrack
    };

    CreditProgress {
        paid_count,
        overdue_count,
        next_due_date,
        status,
    }
}

/// What the credit costs beyond the thing it bought, in cents.
///
/// The two kinds know it from different sides: an installment plan compares its
/// total against the cash price, a loan compares its payments against the money
/// actually lent. `None` when there is nothing to compare with — not knowing
/// the cost is a different thing from there being none.
pub fn credit_interest(credit: &Credit, rows: &[CreditInstallment]) -> Option<i64> {
    match credit.get_kind() {
        CreditKind::Installments => {
            let cash_price = credit.cash_price?;
            Some(credit_totals(rows).plan - cash_price)
        }
        CreditKind::Loan => {
            let principal = credit.principal?;
            let installments: i64 = rows
                .iter()
                .filter(|row| row.get_kind() == InstallmentKind::Installment)
                .map(|row| row.amount)
                .sum();
            Some(installments - principal)
        }
    }
}

/// The description carried into the ledger by the payment of one row.
///
/// Locale-neutral for the plan's own rows, since they are stored and stay
/// readable whatever language the app is later set to. A charge instead carries
/// the reason the user typed, which needs no translating.
pub fn payment_description(credit: &Credit, row: &CreditInstallment) -> String {
    match row.get_kind() {
        // Zero reads as "before the first", which is what a down payment is.
        InstallmentKind::DownPayment => format!("{} 0/{}", credit.name, credit.installment_count),
        InstallmentKind::Installment => {
            format!(
                "{} {}/{}",
                credit.name, row.number, credit.installment_count
            )
        }
        InstallmentKind::Charge => match row.note.as_deref().map(str::trim) {
            Some(note) if !note.is_empty() => format!("{} - {note}", credit.name),
            _ => format!("{} +{}", credit.name, row.number),
        },
    }
}

/// Credit operations for FinanceService
pub struct CreditOps;

impl CreditOps {
    /// Creates a credit and writes its whole schedule. Returns the credit id.
    pub fn create_credit(db: &Database, cmd: NewCredit) -> Result<String, FinanceError> {
        let account_id = validate_uuid(&cmd.account_id)?;
        let name = sanitize_text(&validate_field_length(
            &cmd.name,
            MAX_CREDIT_NAME_LENGTH,
            "Name",
        )?);
        if name.is_empty() {
            return Err(FinanceError::Validation("Name cannot be empty".to_string()));
        }

        let category = sanitize_string(&validate_field_length(
            &cmd.category,
            MAX_CATEGORY_LENGTH,
            "Category",
        )?);
        if category.is_empty() {
            return Err(FinanceError::Validation(
                "Category cannot be empty".to_string(),
            ));
        }

        let kind = CreditKind::parse(&cmd.kind).ok_or_else(|| {
            FinanceError::Validation("Credit kind must be installments or loan".to_string())
        })?;

        if cmd.installment_amount_cents <= 0 {
            return Err(FinanceError::Validation(
                "Installment amount must be greater than zero".to_string(),
            ));
        }
        if cmd.installment_count < 1 || cmd.installment_count > MAX_INSTALLMENTS {
            return Err(FinanceError::Validation(format!(
                "A credit must have between 1 and {MAX_INSTALLMENTS} installments"
            )));
        }
        if cmd.down_payment_cents < 0 {
            return Err(FinanceError::Validation(
                "Down payment cannot be negative".to_string(),
            ));
        }

        let (cash_price, principal, monthly_rate_ppm) = match kind {
            CreditKind::Installments => {
                if let Some(price) = cmd.cash_price_cents
                    && price <= 0
                {
                    return Err(FinanceError::Validation(
                        "Cash price must be greater than zero".to_string(),
                    ));
                }
                (cmd.cash_price_cents, None, None)
            }
            CreditKind::Loan => {
                let principal =
                    cmd.principal_cents
                        .filter(|value| *value > 0)
                        .ok_or_else(|| {
                            FinanceError::Validation(
                                "A loan needs the amount financed, greater than zero".to_string(),
                            )
                        })?;
                let rate = cmd.monthly_rate_ppm.unwrap_or(0);
                if !(0..=MAX_MONTHLY_RATE_PPM).contains(&rate) {
                    return Err(FinanceError::Validation(
                        "Monthly rate is out of range".to_string(),
                    ));
                }
                (None, Some(principal), Some(rate))
            }
        };

        let first_due_date = validate_date(&cmd.first_due_date)?;
        let first_due = parse_date(&first_due_date)?;
        let down_payment_date = match cmd.down_payment_date.as_deref() {
            Some(value) if !value.trim().is_empty() => validate_date(value)?,
            // A down payment is handed over when the deal is struck, which is at
            // the latest the day the first installment is set from.
            _ => first_due_date.clone(),
        };

        // Fails before anything is written rather than at the first payment.
        FinanceRepository::get_account(db, &account_id)?;

        let credit = Credit {
            id: Uuid::new_v4().to_string(),
            account_id,
            name,
            category,
            kind: kind.as_str().to_string(),
            down_payment: cmd.down_payment_cents,
            installment_amount: cmd.installment_amount_cents,
            installment_count: cmd.installment_count,
            first_due_date,
            cash_price,
            principal,
            monthly_rate_ppm,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let mut rows: Vec<CreditInstallment> = Vec::new();
        if credit.down_payment > 0 {
            rows.push(CreditInstallment {
                id: Uuid::new_v4().to_string(),
                credit_id: credit.id.clone(),
                kind: InstallmentKind::DownPayment.as_str().to_string(),
                number: 1,
                amount: credit.down_payment,
                due_date: down_payment_date,
                note: None,
                transaction_id: None,
                paid_date: None,
            });
        }

        rows.extend(
            schedule_dates(first_due, cmd.installment_count)
                .into_iter()
                .enumerate()
                .map(|(index, date)| CreditInstallment {
                    id: Uuid::new_v4().to_string(),
                    credit_id: credit.id.clone(),
                    kind: InstallmentKind::Installment.as_str().to_string(),
                    number: index as i32 + 1,
                    amount: credit.installment_amount,
                    due_date: date.format("%Y-%m-%d").to_string(),
                    note: None,
                    transaction_id: None,
                    paid_date: None,
                }),
        );

        FinanceRepository::create_credit(db, &credit, &rows)?;
        Ok(credit.id)
    }

    pub fn get_credits(db: &Database) -> Result<Vec<Credit>, FinanceError> {
        Ok(FinanceRepository::get_credits(db)?)
    }

    pub fn get_credit_installments(
        db: &Database,
    ) -> Result<HashMap<String, Vec<CreditInstallment>>, FinanceError> {
        Ok(FinanceRepository::get_credit_installments(db)?)
    }

    /// Deletes a credit and its schedule. Payments already made are kept.
    pub fn delete_credit(db: &Database, id: &str) -> Result<(), FinanceError> {
        let id = validate_uuid(id)?;
        FinanceRepository::delete_credit(db, &id).map_err(map_credit_error)
    }

    /// Corrects one unpaid row's amount and date.
    ///
    /// This is what makes an irregular schedule possible without a third kind of
    /// credit: a balloon final payment, a promotional grace period or a plan
    /// whose payments simply are not equal are all this operation, repeated.
    pub fn update_installment(db: &Database, cmd: UpdateInstallment) -> Result<(), FinanceError> {
        let id = validate_uuid(&cmd.installment_id)?;
        if cmd.amount_cents <= 0 {
            return Err(FinanceError::Validation(
                "Amount must be greater than zero".to_string(),
            ));
        }
        let due_date = validate_date(&cmd.due_date)?;

        FinanceRepository::update_credit_installment(db, &id, cmd.amount_cents, &due_date)
            .map_err(map_credit_error)
    }

    /// Records a fee the lender charged on top of the plan.
    ///
    /// Taken as the user was actually charged rather than computed: late
    /// interest and collection fees follow local rules and published figures
    /// that this app has no business guessing at, and a number invented here
    /// would look exact while being wrong.
    pub fn add_charge(db: &Database, cmd: NewCharge) -> Result<String, FinanceError> {
        let credit_id = validate_uuid(&cmd.credit_id)?;
        if cmd.amount_cents <= 0 {
            return Err(FinanceError::Validation(
                "Charge must be greater than zero".to_string(),
            ));
        }
        let due_date = validate_date(&cmd.date)?;
        let note = sanitize_text(&validate_field_length(
            &cmd.note,
            MAX_CREDIT_NAME_LENGTH,
            "Note",
        )?);

        let charge = CreditInstallment {
            id: Uuid::new_v4().to_string(),
            credit_id,
            kind: InstallmentKind::Charge.as_str().to_string(),
            // Replaced with the next free one inside the write.
            number: 0,
            amount: cmd.amount_cents,
            due_date,
            note: if note.is_empty() { None } else { Some(note) },
            transaction_id: None,
            paid_date: None,
        };

        FinanceRepository::add_credit_charge(db, &charge).map_err(map_credit_error)?;
        Ok(charge.id)
    }

    /// Removes an unpaid charge. The plan's own rows are not optional.
    pub fn delete_charge(db: &Database, installment_id: &str) -> Result<(), FinanceError> {
        let id = validate_uuid(installment_id)?;
        FinanceRepository::delete_credit_charge(db, &id).map_err(|error| match error {
            DbError::InstallmentNotFound => FinanceError::Validation(
                "Only an unpaid charge can be removed. Undo its payment first.".to_string(),
            ),
            other => map_credit_error(other),
        })
    }

    /// Pays one row, writing the expense it stands for.
    ///
    /// `date` is when the money actually left, which is not necessarily the due
    /// date: paying early is the point of showing the schedule at all.
    pub fn pay_installment(
        db: &Database,
        installment_id: &str,
        date: &str,
    ) -> Result<String, FinanceError> {
        let installment_id = validate_uuid(installment_id)?;
        let date = validate_date(date)?;

        let installment = FinanceRepository::get_credit_installment(db, &installment_id)
            .map_err(map_credit_error)?;
        let credit =
            FinanceRepository::get_credit(db, &installment.credit_id).map_err(map_credit_error)?;

        let payment = Transaction::new(
            Uuid::new_v4().to_string(),
            credit.account_id.clone(),
            installment.amount,
            credit.category.clone(),
            payment_description(&credit, &installment),
            date,
            "expense".to_string(),
            None,
        );

        FinanceRepository::pay_credit_installment(db, &installment_id, &payment)
            .map_err(map_credit_error)?;
        Ok(payment.id)
    }

    /// Undoes a payment, deleting the expense it wrote.
    pub fn unpay_installment(db: &Database, installment_id: &str) -> Result<(), FinanceError> {
        let installment_id = validate_uuid(installment_id)?;
        FinanceRepository::unpay_credit_installment(db, &installment_id).map_err(map_credit_error)
    }
}

fn parse_date(value: &str) -> Result<NaiveDate, FinanceError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| FinanceError::Validation("Invalid date".to_string()))
}

/// Turns the states the user can walk into by clicking twice into messages,
/// rather than reporting them as internal failures.
fn map_credit_error(error: DbError) -> FinanceError {
    match error {
        DbError::CreditNotFound => FinanceError::Validation("Credit not found".to_string()),
        DbError::InstallmentNotFound => {
            FinanceError::Validation("Installment not found".to_string())
        }
        DbError::InstallmentAlreadyPaid => {
            FinanceError::Validation("This installment is already paid".to_string())
        }
        DbError::InstallmentNotPaid => {
            FinanceError::Validation("This installment has not been paid yet".to_string())
        }
        other => FinanceError::Database(other),
    }
}
