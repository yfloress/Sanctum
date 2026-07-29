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
