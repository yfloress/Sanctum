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

//! Finance domain DTOs.
//!
//! Covers: accounts, transactions, categories, transfers.
//!
//! Input DTOs carry raw, stringly values from the frontend. Their `into_*`
//! methods validate user input (tagging the offending field on [`AppError`])
//! and map into the domain command structs the service consumes — keeping the
//! domain layer free of these IPC-shaped types.

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::features::finance::{
    NewAccount, NewCharge, NewCredit, NewRecurring, NewTransaction, NewTransfer, UpdateAccount,
    UpdateInstallment, UpdateTransaction, UpdateTransfer,
};
use crate::ui::{normalize_account_type, parse_amount_input};

/// Default account accent color when the frontend does not supply one.
const DEFAULT_ACCOUNT_COLOR: &str = "#8b5cf6";

/// Parse a required, strictly-positive money amount, tagging `field` on failure.
fn parse_positive_amount(raw: &str, field: &str) -> Result<i64, AppError> {
    parse_amount_input(raw)
        .filter(|v| *v > 0)
        .ok_or_else(|| AppError::validation("Amount must be greater than zero").with_field(field))
}

// ==================== Accounts ====================

/// Account as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct AccountDto {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub account_type_key: String,
    pub icon_path: Option<String>,
    pub currency: String,
    pub balance: String,
    pub balance_negative: bool,
    pub initial_balance: String,
    pub is_archived: bool,
}

/// Accounts list with total balance.
#[derive(Debug, Clone, Serialize)]
pub struct AccountsResponse {
    pub accounts: Vec<AccountDto>,
    pub total_balance: String,
    pub total_balance_negative: bool,
}

/// Account detail with transaction history.
#[derive(Debug, Clone, Serialize)]
pub struct AccountDetailResponse {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub balance: String,
    pub balance_negative: bool,
    pub icon_path: Option<String>,
    pub transactions: Vec<TransactionDto>,
}

/// Input for creating or updating an account.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInput {
    pub id: Option<String>,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub initial_balance: String,
}

/// Input for updating an account icon.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountIconInput {
    pub id: String,
    pub icon: String,
}

/// Input for renaming an account.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountRenameInput {
    pub id: String,
    pub new_name: String,
}

// ==================== Transactions ====================

/// Transaction as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct TransactionDto {
    pub id: String,
    pub account_id: String,
    pub account_name: String,
    pub date: String,
    pub description: String,
    pub description_raw: String,
    pub category: String,
    pub category_raw: String,
    pub amount: String,
    pub amount_raw: String,
    pub is_expense: bool,
    pub is_transfer: bool,
    pub transfer_account_id: Option<String>,
    pub transfer_account_name: Option<String>,
    /// Fully confirmed against a statement. A transfer needs both of its
    /// accounts to have confirmed it, since one row shows up on two statements.
    pub reconciled: bool,
    /// Free-form labels, already normalized. Empty for most rows.
    pub tags: Vec<String>,
}

/// Paginated transaction list.
#[derive(Debug, Clone, Serialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<TransactionDto>,
    pub has_more: bool,
}

/// Input for creating or updating a transaction.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionInput {
    pub id: Option<String>,
    pub account_id: String,
    pub amount: String,
    pub category: String,
    pub description: String,
    pub date: String,
    pub is_expense: bool,
    /// Absent means "leave the tags alone", which is what an older caller or a
    /// form that does not offer them wants.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Filtering, ordering and paging for a transaction query.
///
/// Every field is independently optional; `None` means "do not narrow by this".
/// They travel as one value so the query can grow another dimension without
/// lengthening a positional argument list nobody can read at the call site.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TransactionFilter {
    pub query: Option<String>,
    pub account_id: Option<String>,
    pub category: Option<String>,
    /// Narrows to rows carrying this tag.
    pub tag: Option<String>,
    /// Inclusive ISO `YYYY-MM-DD` bounds.
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<usize>,
    /// One of `date-desc`, `date-asc`, `amount-desc`, `amount-asc`.
    pub sort: Option<String>,
}

// ==================== Reconciliation ====================

/// One row awaiting confirmation, as the reconcile screen needs it.
#[derive(Debug, Clone, Serialize)]
pub struct ReconcileRowDto {
    pub id: String,
    pub date: String,
    pub description: String,
    /// Signed as this account sees it: negative means money left. A transfer
    /// flips sign depending on which side is being reconciled.
    ///
    /// Raw rather than formatted: the screen adds up ticked rows as the user
    /// goes, so it has to format arbitrary sums anyway and a preformatted
    /// string here would only let the two disagree.
    pub amount_cents: i64,
}

/// Everything the reconcile screen needs for one account.
#[derive(Debug, Clone, Serialize)]
pub struct ReconciliationResponse {
    pub account_id: String,
    pub account_name: String,
    pub currency: String,
    /// Balance of what is already confirmed, in cents so the frontend can do
    /// the arithmetic of the difference without parsing a formatted string.
    pub confirmed_cents: i64,
    /// Balance of everything, confirmed or not.
    pub current_cents: i64,
    pub pending: Vec<ReconcileRowDto>,
}

// ==================== Transfers ====================

/// Input for creating or updating a fund transfer.
#[derive(Debug, Clone, Deserialize)]
pub struct TransferInput {
    pub id: Option<String>,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: String,
    pub description: String,
    pub date: String,
}

// ==================== Categories ====================

/// Transaction category as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryDto {
    /// Stored name. Stays raw because transactions reference categories by it,
    /// so it is what filters and new transactions must send back.
    pub name: String,
    /// Translated, display-ready version of `name`.
    pub label: String,
    pub id: String,
    pub is_default: bool,
}

/// Both expense and income categories grouped.
#[derive(Debug, Clone, Serialize)]
pub struct CategoriesResponse {
    pub expense: Vec<CategoryDto>,
    pub income: Vec<CategoryDto>,
}

/// Input for creating or updating a category.
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryInput {
    pub id: Option<String>,
    pub name: String,
    pub category_type: String,
}

// ==================== DTO -> domain command mapping ====================

impl AccountInput {
    /// Map into a create command. Initial balance defaults to 0 when unparseable
    /// (matches the previous command behavior).
    pub fn into_new_account(self) -> Result<NewAccount, AppError> {
        Ok(NewAccount {
            name: self.name,
            account_type: normalize_account_type(&self.account_type),
            currency: self.currency,
            initial_balance_cents: parse_amount_input(&self.initial_balance).unwrap_or(0),
            color: DEFAULT_ACCOUNT_COLOR.to_string(),
            icon: None,
        })
    }

    /// Map into an update command. `existing_icon` is preserved (the frontend
    /// edits accounts without re-sending the icon).
    pub fn into_update_account(
        self,
        existing_icon: Option<String>,
    ) -> Result<UpdateAccount, AppError> {
        let id = self
            .id
            .ok_or_else(|| AppError::validation("Account id is required").with_field("id"))?;
        Ok(UpdateAccount {
            id,
            name: self.name,
            account_type: normalize_account_type(&self.account_type),
            currency: self.currency,
            initial_balance_cents: parse_amount_input(&self.initial_balance).unwrap_or(0),
            color: DEFAULT_ACCOUNT_COLOR.to_string(),
            icon: existing_icon,
        })
    }
}

impl TransactionInput {
    /// Map into an add command, validating the amount.
    pub fn into_new(self) -> Result<NewTransaction, AppError> {
        Ok(NewTransaction {
            account_id: self.account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            category: self.category,
            description: self.description,
            date: self.date,
            is_expense: self.is_expense,
        })
    }

    /// Map into an update command, requiring an id and validating the amount.
    pub fn into_update(self) -> Result<UpdateTransaction, AppError> {
        let id = self
            .id
            .ok_or_else(|| AppError::validation("Transaction id is required").with_field("id"))?;
        Ok(UpdateTransaction {
            id,
            account_id: self.account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            category: self.category,
            description: self.description,
            date: self.date,
            is_expense: self.is_expense,
        })
    }
}

impl TransferInput {
    /// Map into a transfer command, validating the amount.
    pub fn into_new(self) -> Result<NewTransfer, AppError> {
        Ok(NewTransfer {
            from_account_id: self.from_account_id,
            to_account_id: self.to_account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            description: self.description,
            date: self.date,
        })
    }

    /// Map into a transfer-update command, requiring an id and validating the amount.
    pub fn into_update(self) -> Result<UpdateTransfer, AppError> {
        let id = self
            .id
            .ok_or_else(|| AppError::validation("Transfer id is required").with_field("id"))?;
        Ok(UpdateTransfer {
            id,
            from_account_id: self.from_account_id,
            to_account_id: self.to_account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            description: self.description,
            date: self.date,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn transaction_invalid_amount_tags_field() {
        let input = TransactionInput {
            id: None,
            account_id: "acc".to_string(),
            amount: "0".to_string(),
            category: "food".to_string(),
            description: String::new(),
            date: "2026-01-01".to_string(),
            is_expense: true,
            tags: None,
        };
        let err = input.into_new().unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(err.field.as_deref(), Some("amount"));
    }

    #[test]
    fn transfer_update_requires_id() {
        let input = TransferInput {
            id: None,
            from_account_id: "a".to_string(),
            to_account_id: "b".to_string(),
            amount: "100".to_string(),
            description: String::new(),
            date: "2026-01-01".to_string(),
        };
        let err = input.into_update().unwrap_err();
        assert_eq!(err.field.as_deref(), Some("id"));
    }

    fn credit_with_rate(rate: &str, period: &str) -> CreditInput {
        CreditInput {
            account_id: "acc".to_string(),
            name: "Loan".to_string(),
            category: "OTHER".to_string(),
            kind: "loan".to_string(),
            down_payment: None,
            down_payment_date: None,
            installment_amount: "100".to_string(),
            installment_count: 12,
            first_due_date: "2026-01-01".to_string(),
            cash_price: None,
            principal: Some("1000".to_string()),
            rate: Some(rate.to_string()),
            rate_period: Some(period.to_string()),
        }
    }

    #[test]
    fn an_annual_rate_becomes_a_monthly_one() {
        // 18% a year is 1.5% a month, which is 15000 millionths.
        let cmd = credit_with_rate("18", "annual").into_new().unwrap();
        assert_eq!(cmd.monthly_rate_ppm, Some(15_000));

        // The same figure quoted monthly is taken as it stands.
        let cmd = credit_with_rate("1.5", "monthly").into_new().unwrap();
        assert_eq!(cmd.monthly_rate_ppm, Some(15_000));
    }

    #[test]
    fn a_small_annual_rate_survives_the_division_by_twelve() {
        // 1.5% a year is 0.125% a month. Held in basis points this would round
        // to 0.13% and the schedule would stop adding up.
        let cmd = credit_with_rate("1.5", "annual").into_new().unwrap();
        assert_eq!(cmd.monthly_rate_ppm, Some(1_250));
    }

    #[test]
    fn account_create_maps_defaults() {
        let input = AccountInput {
            id: None,
            name: "Checking".to_string(),
            account_type: "bank".to_string(),
            currency: "usd".to_string(),
            initial_balance: "10.00".to_string(),
            // no color/icon in the DTO
        };
        let cmd = input.into_new_account().unwrap();
        assert_eq!(cmd.color, DEFAULT_ACCOUNT_COLOR);
        assert!(cmd.icon.is_none());
        assert_eq!(cmd.initial_balance_cents, 1000);
    }
}

/// A recurring rule as the interface shows it.
#[derive(Debug, Clone, Serialize)]
pub struct RecurringDto {
    pub id: String,
    pub account_id: String,
    pub account_name: String,
    pub amount: String,
    pub amount_raw: String,
    /// Stored category name, for sending back on edits.
    pub category: String,
    /// Translated, display-ready category.
    pub category_label: String,
    pub description: String,
    pub frequency: String,
    pub next_date: String,
    pub is_expense: bool,
    pub is_active: bool,
}

/// Input for creating a recurring rule.
#[derive(Debug, Clone, Deserialize)]
pub struct RecurringInput {
    pub account_id: String,
    pub amount: String,
    pub category: String,
    pub description: String,
    pub frequency: String,
    pub first_date: String,
    pub is_expense: bool,
}

impl RecurringInput {
    pub fn into_new(self) -> Result<NewRecurring, AppError> {
        Ok(NewRecurring {
            account_id: self.account_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            category: self.category,
            description: self.description,
            frequency: self.frequency,
            first_date: self.first_date,
            is_expense: self.is_expense,
        })
    }
}

/// One row of a credit's schedule, ready to render.
#[derive(Debug, Clone, Serialize)]
pub struct CreditInstallmentDto {
    pub id: String,
    /// `down_payment`, `installment` or `charge`.
    pub kind: String,
    /// 1-based position within its own kind.
    pub number: i32,
    pub amount: String,
    /// The same figure unformatted, for the correction form to start from.
    pub amount_raw: String,
    pub due_date: String,
    /// When it was actually paid, which need not be the due date.
    pub paid_date: Option<String>,
    /// Why a charge was made. Only charges carry one.
    pub note: Option<String>,
    pub is_paid: bool,
    /// Unpaid and past its date.
    pub is_overdue: bool,
}

/// One line of an amortisation table, ready to render.
#[derive(Debug, Clone, Serialize)]
pub struct AmortizationRowDto {
    pub number: i32,
    pub due_date: String,
    pub payment: String,
    /// The part of the payment that only rents the money.
    pub interest: String,
    /// The part that actually reduces the debt.
    pub principal: String,
    /// What is still owed after this payment.
    pub balance: String,
}

/// A credit with its progress, ready to render.
#[derive(Debug, Clone, Serialize)]
pub struct CreditDto {
    pub id: String,
    pub name: String,
    pub account_id: String,
    pub account_name: String,
    /// Stored category name, for showing what the payments are filed under.
    pub category: String,
    /// Translated, display-ready category.
    pub category_label: String,
    /// `installments` or `loan`.
    pub kind: String,
    /// Paid up front. Absent when the credit had none.
    pub down_payment: Option<String>,
    pub installment_amount: String,
    pub installment_count: i32,
    pub paid_count: usize,
    pub overdue_count: usize,
    pub total: String,
    pub paid: String,
    pub remaining: String,
    /// Fees the lender added on top of the plan. Absent when there are none.
    pub charges: Option<String>,
    /// Share of the plan paid, in money rather than in rows, for the bar.
    pub percentage: f32,
    /// Date of the first plan row still to pay.
    pub next_due_date: Option<String>,
    /// One of `done`, `overdue`, `ahead`, `on_track`.
    pub status: String,
    /// What the credit costs beyond the thing it bought. Absent when there is
    /// nothing to compare against, which differs from costing nothing extra.
    pub interest: Option<String>,
    pub cash_price: Option<String>,
    /// `loan` only: the amount financed.
    pub principal: Option<String>,
    /// `loan` only: the monthly rate as a percentage, e.g. `1.79`.
    pub monthly_rate: Option<String>,
    pub installments: Vec<CreditInstallmentDto>,
    /// `loan` only: how each payment splits between interest and principal.
    pub amortization: Vec<AmortizationRowDto>,
}

/// Input for creating a credit.
///
/// Two sets of fields, one per kind: `installments` fills `installment_amount`
/// and `cash_price`, `loan` fills `principal` and the rate. Both may carry a
/// down payment.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditInput {
    pub account_id: String,
    pub name: String,
    pub category: String,
    pub kind: String,
    #[serde(default)]
    pub down_payment: Option<String>,
    #[serde(default)]
    pub down_payment_date: Option<String>,
    pub installment_amount: String,
    pub installment_count: i32,
    pub first_due_date: String,
    #[serde(default)]
    pub cash_price: Option<String>,
    #[serde(default)]
    pub principal: Option<String>,
    /// The rate as typed, e.g. `1.79`, read together with `rate_period`.
    #[serde(default)]
    pub rate: Option<String>,
    /// `monthly` or `annual`. Markets quote one or the other, so the app takes
    /// whichever the borrower has in front of them and converts.
    #[serde(default)]
    pub rate_period: Option<String>,
}

/// Reads an optional money field: blank means absent, unparseable is an error.
fn parse_optional_amount(raw: Option<&str>, field: &str) -> Result<Option<i64>, AppError> {
    match raw.map(str::trim) {
        None | Some("") => Ok(None),
        Some(value) => Ok(Some(parse_positive_amount(value, field)?)),
    }
}

impl CreditInput {
    /// The monthly rate in millionths, from the rate as the user typed it.
    ///
    /// An annual figure is divided by twelve, the nominal convention consumer
    /// lending quotes in. Millionths rather than basis points because that
    /// division needs the room: 1.5% a year is 0.125% a month, and basis points
    /// would flatten it to 0.13% and throw the whole schedule off.
    ///
    /// The frontend quantises the same way before suggesting a payment, so the
    /// figure it offers and the breakdown drawn from this cannot disagree.
    fn monthly_rate_ppm(&self) -> Result<Option<i64>, AppError> {
        let Some(raw) = self
            .rate
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            return Ok(None);
        };
        // Percent-to-basis-points is the same scaling as money-to-cents, so the
        // shared parser does it and accepts either decimal separator on the way.
        let bp = parse_amount_input(raw)
            .filter(|value| *value >= 0)
            .ok_or_else(|| AppError::validation("Rate is not a number").with_field("rate"))?;
        let annual_ppm = bp * 100;

        let annual = matches!(self.rate_period.as_deref(), Some("annual"));
        Ok(Some(if annual {
            (annual_ppm + 6) / 12
        } else {
            annual_ppm
        }))
    }

    pub fn into_new(self) -> Result<NewCredit, AppError> {
        let monthly_rate_ppm = self.monthly_rate_ppm()?;
        let cash_price_cents = parse_optional_amount(self.cash_price.as_deref(), "cash_price")?;
        let principal_cents = parse_optional_amount(self.principal.as_deref(), "principal")?;
        let down_payment_cents =
            parse_optional_amount(self.down_payment.as_deref(), "down_payment")?.unwrap_or(0);

        Ok(NewCredit {
            account_id: self.account_id,
            name: self.name,
            category: self.category,
            kind: self.kind,
            down_payment_cents,
            down_payment_date: self.down_payment_date,
            installment_amount_cents: parse_positive_amount(
                &self.installment_amount,
                "installment_amount",
            )?,
            installment_count: self.installment_count,
            first_due_date: self.first_due_date,
            cash_price_cents,
            principal_cents,
            monthly_rate_ppm,
        })
    }
}

/// Input for correcting one unpaid row of a schedule.
#[derive(Debug, Clone, Deserialize)]
pub struct InstallmentUpdateInput {
    pub installment_id: String,
    pub amount: String,
    pub due_date: String,
}

impl InstallmentUpdateInput {
    pub fn into_update(self) -> Result<UpdateInstallment, AppError> {
        Ok(UpdateInstallment {
            installment_id: self.installment_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            due_date: self.due_date,
        })
    }
}

/// Input for recording a fee the lender charged on top of a plan.
#[derive(Debug, Clone, Deserialize)]
pub struct ChargeInput {
    pub credit_id: String,
    pub amount: String,
    pub date: String,
    #[serde(default)]
    pub note: String,
}

impl ChargeInput {
    pub fn into_new(self) -> Result<NewCharge, AppError> {
        Ok(NewCharge {
            credit_id: self.credit_id,
            amount_cents: parse_positive_amount(&self.amount, "amount")?,
            date: self.date,
            note: self.note,
        })
    }
}

/// A budget with its progress for the month, ready to render.
#[derive(Debug, Clone, Serialize)]
pub struct BudgetDto {
    /// Stored category name, for sending back on edits.
    pub category: String,
    /// Translated, display-ready category.
    pub category_label: String,
    pub limit: String,
    pub limit_raw: String,
    pub spent: String,
    /// Spent as a share of the limit, capped at 100 for the bar's width.
    pub percentage: f32,
    /// True once spending passes the limit, so the bar can turn red.
    pub over_budget: bool,
    /// Remaining amount, or the overspend when `over_budget`.
    pub remaining: String,
}

/// Input for setting a category budget.
#[derive(Debug, Clone, Deserialize)]
pub struct BudgetInput {
    pub category: String,
    pub amount: String,
}
