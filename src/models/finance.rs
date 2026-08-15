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

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ==================== FIAT Accounts System ====================

/// Account types for FIAT money management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Bank,
    Cash,
    Savings,
    CreditCard,
    Other,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Bank => "bank",
            AccountType::Cash => "cash",
            AccountType::Savings => "savings",
            AccountType::CreditCard => "credit_card",
            AccountType::Other => "other",
        }
    }
}

impl FromStr for AccountType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bank" => Ok(AccountType::Bank),
            "cash" => Ok(AccountType::Cash),
            "savings" => Ok(AccountType::Savings),
            "credit_card" => Ok(AccountType::CreditCard),
            "other" => Ok(AccountType::Other),
            _ => Err(()),
        }
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a FIAT money account (bank, cash, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency: String,
    pub initial_balance: i64,
    pub color: String,
    pub icon: Option<String>,
    pub is_archived: bool,
    pub created_at: String,
}

impl Account {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        account_type: String,
        currency: String,
        initial_balance: i64,
        color: String,
        icon: Option<String>,
        created_at: String,
    ) -> Self {
        Self {
            id,
            name,
            account_type,
            currency,
            initial_balance,
            color,
            icon,
            is_archived: false,
            created_at,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.trim().is_empty()
            && self.account_type.parse::<AccountType>().is_ok()
            && !self.currency.trim().is_empty()
            && self.color.starts_with('#')
            && self.color.len() == 7
    }

    pub fn get_type(&self) -> Option<AccountType> {
        self.account_type.parse::<AccountType>().ok()
    }
}

/// Transaction type for financial transactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FinancialTransactionType {
    Income,
    Expense,
    Transfer,
}

impl FinancialTransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FinancialTransactionType::Income => "income",
            FinancialTransactionType::Expense => "expense",
            FinancialTransactionType::Transfer => "transfer",
        }
    }
}

impl FromStr for FinancialTransactionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "income" => Ok(FinancialTransactionType::Income),
            "expense" => Ok(FinancialTransactionType::Expense),
            "transfer" => Ok(FinancialTransactionType::Transfer),
            _ => Err(()),
        }
    }
}

impl fmt::Display for FinancialTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ==================== Financial Transactions ====================

/// Represents a financial transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub amount: i64,
    pub category: String,
    pub description: String,
    pub date: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub transfer_account_id: Option<String>,
    /// Confirmed against the statement of `account_id`.
    #[serde(default)]
    pub reconciled: bool,
    /// Confirmed against the statement of `transfer_account_id`. A transfer is
    /// one row seen by two accounts, and each side clears its bank separately.
    #[serde(default)]
    pub transfer_reconciled: bool,
}

/// How often a recurring rule fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecurrenceFrequency {
    Weekly,
    Monthly,
    Yearly,
}

impl RecurrenceFrequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "weekly" => Some(Self::Weekly),
            "monthly" => Some(Self::Monthly),
            "yearly" => Some(Self::Yearly),
            _ => None,
        }
    }

    /// The occurrence after `date`.
    ///
    /// Month and year steps clamp to the end of a shorter month, so a rule on
    /// the 31st fires on the 28th in February and returns to the 31st after —
    /// the anchor day is never lost, unlike repeatedly adding 30 days.
    pub fn next_after(&self, date: chrono::NaiveDate) -> Option<chrono::NaiveDate> {
        match self {
            Self::Weekly => date.checked_add_days(chrono::Days::new(7)),
            Self::Monthly => date.checked_add_months(chrono::Months::new(1)),
            Self::Yearly => date.checked_add_months(chrono::Months::new(12)),
        }
    }
}

/// A template that materialises transactions on a schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTransaction {
    pub id: String,
    pub account_id: String,
    pub amount: i64,
    pub category: String,
    pub description: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub frequency: String,
    /// ISO date of the next occurrence still to be created.
    pub next_date: String,
    pub last_created_date: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

/// How a credit was described to the borrower.
///
/// Not a product catalogue: it is the two forms the same information takes
/// anywhere in the world. Either the payment is quoted and the rate is buried
/// inside it, or a rate is quoted and the payment follows from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditKind {
    /// A quoted payment repeated N times, the rate left implicit.
    Installments,
    /// A principal at a rate, the payment derived from both.
    Loan,
}

impl CreditKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Installments => "installments",
            Self::Loan => "loan",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "installments" => Some(Self::Installments),
            "loan" => Some(Self::Loan),
            _ => None,
        }
    }
}

/// A debt repaid over a fixed number of dated payments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credit {
    pub id: String,
    /// Account the payments come out of.
    pub account_id: String,
    pub name: String,
    /// Category the generated payments are filed under.
    pub category: String,
    pub kind: String,
    /// Paid up front, before the schedule starts. Zero when there was none.
    pub down_payment: i64,
    /// The nominal payment, in cents. The truth for any given payment is on its
    /// own row, since a schedule can be irregular.
    pub installment_amount: i64,
    pub installment_count: i32,
    pub first_due_date: String,
    /// `Installments` only: what the purchase would have cost paid outright.
    pub cash_price: Option<i64>,
    /// `Loan` only: the amount actually financed.
    pub principal: Option<i64>,
    /// `Loan` only: the monthly rate in millionths (a monthly 1.5% is 15000).
    pub monthly_rate_ppm: Option<i64>,
    pub created_at: String,
}

impl Credit {
    pub fn get_kind(&self) -> CreditKind {
        CreditKind::parse(&self.kind).unwrap_or(CreditKind::Installments)
    }
}

/// What a row of a credit's schedule stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallmentKind {
    /// Money handed over up front, before the schedule proper.
    DownPayment,
    /// One payment of the plan.
    Installment,
    /// A fee the lender added later. Payable, but never part of the plan.
    Charge,
}

impl InstallmentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DownPayment => "down_payment",
            Self::Installment => "installment",
            Self::Charge => "charge",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "down_payment" => Some(Self::DownPayment),
            "installment" => Some(Self::Installment),
            "charge" => Some(Self::Charge),
            _ => None,
        }
    }
}

/// One dated payment of a [`Credit`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditInstallment {
    pub id: String,
    pub credit_id: String,
    pub kind: String,
    /// 1-based position within its own kind. The down payment is number 1 of a
    /// kind that only ever has one row.
    pub number: i32,
    /// What this particular payment costs, in cents.
    pub amount: i64,
    pub due_date: String,
    /// Why a charge was made, in the user's own words. Only charges carry one.
    pub note: Option<String>,
    /// The payment. Its presence is what makes the row paid.
    pub transaction_id: Option<String>,
    /// Date of that payment, read from the transaction it points at.
    pub paid_date: Option<String>,
}

impl CreditInstallment {
    pub fn is_paid(&self) -> bool {
        self.transaction_id.is_some()
    }

    pub fn get_kind(&self) -> InstallmentKind {
        InstallmentKind::parse(&self.kind).unwrap_or(InstallmentKind::Installment)
    }

    /// Rows that make up the plan: everything except lender-added fees.
    pub fn is_plan(&self) -> bool {
        self.get_kind() != InstallmentKind::Charge
    }
}

/// A monthly spending limit for one expense category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBudget {
    pub id: String,
    pub category: String,
    pub amount: i64,
    pub created_at: String,
}

/// A budget with the spending measured against it for a given month.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub category: String,
    /// Monthly limit, in cents.
    pub limit: i64,
    /// Spent so far in the month, in cents.
    pub spent: i64,
}

/// Represents a transaction category (income or expense)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCategory {
    pub id: String,
    pub name: String,
    pub category_type: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub created_at: String,
}

/// Financial balance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSummary {
    pub total_balance: i64,
    pub total_income: i64,
    pub total_expense: i64,
}

/// Account balance summary (calculated, not stored)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub account_id: String,
    pub account_name: String,
    pub current_balance: i64,
    pub total_income: i64,
    pub total_expense: i64,
}

impl Transaction {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        account_id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        transaction_type: String,
        transfer_account_id: Option<String>,
    ) -> Self {
        Self {
            id,
            account_id,
            amount,
            category,
            description,
            date,
            transaction_type,
            transfer_account_id,
            // Nothing is confirmed the moment it is written down; confirming is
            // what the user does against a statement afterwards.
            reconciled: false,
            transfer_reconciled: false,
        }
    }

    pub fn validate_type(&self) -> bool {
        self.transaction_type
            .parse::<FinancialTransactionType>()
            .is_ok()
    }

    pub fn validate(&self) -> bool {
        if !self.validate_type() {
            return false;
        }

        if self.transaction_type == "transfer" && self.transfer_account_id.is_none() {
            return false;
        }

        if self.transaction_type != "transfer" && self.transfer_account_id.is_some() {
            return false;
        }

        self.amount > 0 && !self.account_id.is_empty()
    }

    pub fn get_type(&self) -> Option<FinancialTransactionType> {
        self.transaction_type
            .parse::<FinancialTransactionType>()
            .ok()
    }
}
