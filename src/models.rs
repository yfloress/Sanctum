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

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a cryptocurrency asset with market data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoAsset {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: f64,
    #[serde(default)]
    pub price_change_percentage_24h: f64,
    pub last_updated: String,
}

/// Represents a user-facing crypto catalog entry (default or custom)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoCatalogCoin {
    pub id: String,
    pub name: String,
    pub symbol: String,
    #[serde(default)]
    pub custom: bool,
}

// ==================== Crypto Ledger System ====================

/// Wallet category types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WalletCategory {
    Exchange,
    Hardware,
    Software,
}

impl WalletCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            WalletCategory::Exchange => "exchange",
            WalletCategory::Hardware => "hardware",
            WalletCategory::Software => "software",
        }
    }
}

impl FromStr for WalletCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exchange" => Ok(WalletCategory::Exchange),
            "hardware" => Ok(WalletCategory::Hardware),
            "software" => Ok(WalletCategory::Software),
            _ => Err(()),
        }
    }
}

/// Represents a crypto wallet (exchange, hardware wallet, software wallet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoWallet {
    pub id: String,
    pub name: String,
    pub category: String, // "exchange", "hardware", "software"
    pub icon: Option<String>,
}

impl CryptoWallet {
    pub fn new(id: String, name: String, category: String, icon: Option<String>) -> Self {
        Self {
            id,
            name,
            category,
            icon,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.trim().is_empty() && self.category.parse::<WalletCategory>().is_ok()
    }
}

/// Transaction types for the ledger
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CryptoTransactionType {
    Buy,
    Sell,
    TransferIn,
    TransferOut,
    Swap,
}

impl CryptoTransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CryptoTransactionType::Buy => "buy",
            CryptoTransactionType::Sell => "sell",
            CryptoTransactionType::TransferIn => "transfer_in",
            CryptoTransactionType::TransferOut => "transfer_out",
            CryptoTransactionType::Swap => "swap",
        }
    }

    /// Returns true if this transaction type adds to the balance
    pub fn is_inflow(&self) -> bool {
        matches!(
            self,
            CryptoTransactionType::Buy | CryptoTransactionType::TransferIn
        )
    }

    /// Returns true if this transaction type subtracts from the balance
    pub fn is_outflow(&self) -> bool {
        matches!(
            self,
            CryptoTransactionType::Sell | CryptoTransactionType::TransferOut
        )
    }
}

impl FromStr for CryptoTransactionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "buy" => Ok(CryptoTransactionType::Buy),
            "sell" => Ok(CryptoTransactionType::Sell),
            "transfer_in" => Ok(CryptoTransactionType::TransferIn),
            "transfer_out" => Ok(CryptoTransactionType::TransferOut),
            "swap" => Ok(CryptoTransactionType::Swap),
            _ => Err(()),
        }
    }
}

/// Represents a single crypto transaction in the ledger.
///
/// `transaction_type` holds the **transaction type category**: `trade`, `income`,
/// `expense`, or `transfer`.  `subtype` holds the specific action
/// (`buy`, `sell`, `swap`, `airdrop`, `deposit`, …).
///
/// The **mechanical type** (buy/sell/swap/transfer_in/transfer_out) used by
/// balance and portfolio logic is derived via [`mechanical_type()`](Self::mechanical_type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoTransaction {
    pub id: String,
    pub wallet_id: String,
    pub coin_id: String, // CoinGecko ID (e.g., "bitcoin")
    pub symbol: String,  // Symbol (e.g., "BTC")
    #[serde(rename = "type")]
    pub transaction_type: String, // type: trade, income, expense, transfer
    pub amount: f64,     // Amount of coins
    pub price_per_coin: Option<f64>, // Price in USD at transaction time
    pub fee: Option<f64>, // Fee paid (in USD)
    pub fee_coin_id: Option<String>, // If fee was paid in crypto
    pub fee_amount: Option<f64>, // Fee amount in crypto (if applicable)
    #[serde(default)]
    pub subtype: Option<String>, // e.g., buy, sell, swap, airdrop, deposit, withdrawal, …
    #[serde(default)]
    pub override_proceeds: Option<f64>, // Optional manual proceeds override (tax)
    #[serde(default)]
    pub override_cost_basis: Option<f64>, // Optional manual cost basis override (tax)
    pub date: String,    // ISO-8601 date
    pub notes: Option<String>, // Optional notes
    pub related_tx_id: Option<String>, // For swaps/transfers, links to the counterpart transaction
}

impl CryptoTransaction {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        wallet_id: String,
        coin_id: String,
        symbol: String,
        transaction_type: String,
        amount: f64,
        price_per_coin: Option<f64>,
        fee: Option<f64>,
        date: String,
        notes: Option<String>,
    ) -> Self {
        Self {
            id,
            wallet_id,
            coin_id,
            symbol,
            transaction_type,
            amount,
            price_per_coin,
            fee,
            fee_coin_id: None,
            fee_amount: None,
            subtype: None,
            override_proceeds: None,
            override_cost_basis: None,
            date,
            notes,
            related_tx_id: None,
        }
    }

    /// Derives the mechanical transaction type from `type` + `subtype`.
    ///
    /// Returns one of: `"buy"`, `"sell"`, `"swap"`, `"transfer_in"`, `"transfer_out"`.
    pub fn mechanical_type(&self) -> &str {
        let sub = self.subtype.as_deref().unwrap_or("");
        match self.transaction_type.as_str() {
            "trade" => match sub {
                "sell" => "sell",
                "swap" => "swap",
                _ => "buy", // buy, other, or missing
            },
            "transfer" => match sub {
                "withdrawal" => "transfer_out",
                _ => "transfer_in", // deposit or missing
            },
            "income" => "buy",   // all income is an inflow
            "expense" => "sell", // all expense is an outflow
            // Fallback for any unknown value — treat as buy
            _ => "buy",
        }
    }

    pub fn validate(&self) -> bool {
        let valid_types = ["trade", "income", "expense", "transfer"];
        !self.wallet_id.is_empty()
            && !self.coin_id.is_empty()
            && !self.symbol.is_empty()
            && self.amount > 0.0
            && valid_types.contains(&self.transaction_type.as_str())
    }

    /// Returns the mechanical type as a [`CryptoTransactionType`] enum.
    pub fn get_type(&self) -> Option<CryptoTransactionType> {
        self.mechanical_type().parse::<CryptoTransactionType>().ok()
    }

    /// Returns the cost basis for this transaction (amount * price + fees)
    pub fn cost_basis(&self) -> f64 {
        let base = self.amount * self.price_per_coin.unwrap_or(0.0);
        let fee = self.fee.unwrap_or(0.0);
        base + fee
    }
}

/// Aggregated asset data for portfolio overview (calculated, not stored)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedAsset {
    pub coin_id: String,
    pub symbol: String,
    pub total_amount: f64,
    pub total_cost_basis: f64, // Total USD spent acquiring this asset
    pub avg_buy_price: f64,    // Weighted average purchase price
    pub current_price: f64,    // Current market price (injected from API)
    pub current_value: f64,    // total_amount * current_price
    pub unrealized_pnl: f64,   // current_value - total_cost_basis
    pub unrealized_pnl_percentage: f64,
}

impl AggregatedAsset {
    pub fn new(coin_id: String, symbol: String) -> Self {
        Self {
            coin_id,
            symbol,
            total_amount: 0.0,
            total_cost_basis: 0.0,
            avg_buy_price: 0.0,
            current_price: 0.0,
            current_value: 0.0,
            unrealized_pnl: 0.0,
            unrealized_pnl_percentage: 0.0,
        }
    }

    /// Updates current values based on market price
    pub fn update_with_price(&mut self, current_price: f64) {
        self.current_price = current_price;
        self.current_value = self.total_amount * current_price;
        self.unrealized_pnl = self.current_value - self.total_cost_basis;
        self.unrealized_pnl_percentage = if self.total_cost_basis > 0.0 {
            (self.unrealized_pnl / self.total_cost_basis) * 100.0
        } else {
            0.0
        };
    }

    /// Calculates the average buy price from cost basis and amount
    pub fn calculate_avg_price(&mut self) {
        self.avg_buy_price = if self.total_amount > 0.0 {
            self.total_cost_basis / self.total_amount
        } else {
            0.0
        };
    }
}

/// Summary of a wallet's holdings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSummary {
    pub wallet: CryptoWallet,
    pub total_value: f64,
    pub assets_count: usize,
}

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
    pub account_type: String, // "bank", "cash", "savings", "credit_card", "other"
    pub currency: String,     // ISO 4217: "USD", "EUR", "CLP", etc.
    pub initial_balance: i64, // In cents
    pub color: String,        // Hex color for UI
    pub icon: Option<String>, // Optional emoji/icon
    pub is_archived: bool,    // Soft delete flag
    pub created_at: String,   // ISO 8601
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
    pub account_id: String, // Required: which account this belongs to
    pub amount: i64,        // In cents: $10.00 = 1000
    pub category: String,
    pub description: String,
    pub date: String, // ISO 8601
    #[serde(rename = "type")]
    pub transaction_type: String, // "income", "expense", or "transfer"
    pub transfer_account_id: Option<String>, // Only for transfers: destination account
}

/// Represents a transaction category (income or expense)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCategory {
    pub id: String,
    pub name: String,
    pub category_type: String, // "income" or "expense"
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
    pub current_balance: i64, // initial_balance + incomes - expenses
    pub total_income: i64,
    pub total_expense: i64,
}

impl Transaction {
    /// Creates a new transaction
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

    /// Validates that the transaction type is valid
    pub fn validate_type(&self) -> bool {
        self.transaction_type
            .parse::<FinancialTransactionType>()
            .is_ok()
    }

    /// Validates that the transaction is consistent
    pub fn validate(&self) -> bool {
        if !self.validate_type() {
            return false;
        }

        // Transfers must have a destination account
        if self.transaction_type == "transfer" && self.transfer_account_id.is_none() {
            return false;
        }

        // Non-transfers should not have a destination account
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

// ==================== Habits System ====================

/// Represents a habit to track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,      // Hex color code (e.g., "#8b5cf6")
    pub category: String,   // mind, body, spirit
    pub created_at: String, // ISO-8601 datetime
    pub archived: bool,     // Soft delete flag
}

impl Habit {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        created_at: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            color,
            category,
            created_at,
            archived: false,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.trim().is_empty()
            && self.color.starts_with('#')
            && self.color.len() == 7
            && !self.category.trim().is_empty()
    }
}

/// Represents a single habit completion log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitLog {
    pub id: String,
    pub habit_id: String,
    pub completed_date: String, // ISO-8601 date (YYYY-MM-DD)
}

impl HabitLog {
    pub fn new(id: String, habit_id: String, completed_date: String) -> Self {
        Self {
            id,
            habit_id,
            completed_date,
        }
    }

    pub fn validate(&self) -> bool {
        !self.habit_id.is_empty() && !self.completed_date.is_empty()
    }
}

// ==================== Rewards System ====================

/// Represents a streak-based reward linked to a habit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakReward {
    pub id: String,
    pub habit_id: String,
    pub is_consecutive: bool, // true = consecutive days, false = X of Y days
    pub target_days: Option<i32>, // For accumulative: X days required
    pub target_total: Option<i32>, // For accumulative: Y total days window
    pub created_at: String,   // ISO-8601 datetime
}

impl StreakReward {
    pub fn new(
        id: String,
        habit_id: String,
        is_consecutive: bool,
        target_days: Option<i32>,
        target_total: Option<i32>,
    ) -> Self {
        Self {
            id,
            habit_id,
            is_consecutive,
            target_days,
            target_total,
            created_at: chrono::Local::now().to_rfc3339(),
        }
    }

    pub fn validate(&self) -> bool {
        !self.habit_id.is_empty()
            && (self.is_consecutive || (self.target_days.is_some() && self.target_total.is_some()))
    }
}

/// Represents a milestone within a streak reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub reward_id: String,
    pub target_days: i32,
    pub reward_text: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>, // ISO-8601 datetime when unlocked
}

impl Milestone {
    pub fn new(id: String, reward_id: String, target_days: i32, reward_text: String) -> Self {
        Self {
            id,
            reward_id,
            target_days,
            reward_text,
            unlocked: false,
            unlocked_at: None,
        }
    }

    pub fn validate(&self) -> bool {
        !self.reward_id.is_empty() && self.target_days > 0 && !self.reward_text.trim().is_empty()
    }

    pub fn unlock(&mut self) {
        self.unlocked = true;
        self.unlocked_at = Some(chrono::Local::now().to_rfc3339());
    }
}

/// Represents an independent goal with checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub reward_text: String,
    pub deadline: Option<String>, // ISO-8601 date (YYYY-MM-DD)
    pub is_completed: bool,
    pub completed_at: Option<String>, // ISO-8601 datetime
    pub created_at: String,           // ISO-8601 datetime
    pub archived: bool,               // Hide from active goals list
}

impl Goal {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        reward_text: String,
        deadline: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            reward_text,
            deadline,
            is_completed: false,
            completed_at: None,
            created_at: chrono::Local::now().to_rfc3339(),
            archived: false,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.trim().is_empty() && !self.reward_text.trim().is_empty()
    }

    pub fn complete(&mut self) {
        self.is_completed = true;
        self.completed_at = Some(chrono::Local::now().to_rfc3339());
    }

    pub fn archive(&mut self) {
        self.archived = true;
    }
}

/// Represents a checkpoint within a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub goal_id: String,
    pub description: String,
    pub completed: bool,
    pub completed_at: Option<String>, // ISO-8601 datetime
    pub sort_order: i32,
}

impl Checkpoint {
    pub fn new(id: String, goal_id: String, description: String, sort_order: i32) -> Self {
        Self {
            id,
            goal_id,
            description,
            completed: false,
            completed_at: None,
            sort_order,
        }
    }

    pub fn validate(&self) -> bool {
        !self.goal_id.is_empty() && !self.description.trim().is_empty()
    }

    pub fn toggle(&mut self) -> bool {
        self.completed = !self.completed;
        self.completed_at = if self.completed {
            Some(chrono::Local::now().to_rfc3339())
        } else {
            None
        };
        self.completed
    }
}

/// Represents an unlocked achievement (trophy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_path: String,
    pub achievement_type: String, // "streak" | "goal"
    pub source_id: String,        // reward_id or goal_id
    pub achieved_at: String,      // ISO-8601 datetime
}

impl Achievement {
    pub fn new(
        id: String,
        title: String,
        description: String,
        icon_path: String,
        achievement_type: String,
        source_id: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            icon_path,
            achievement_type,
            source_id,
            achieved_at: chrono::Local::now().to_rfc3339(),
        }
    }

    pub fn validate(&self) -> bool {
        !self.title.trim().is_empty()
            && !self.source_id.is_empty()
            && (self.achievement_type == "streak" || self.achievement_type == "goal")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Account Tests ====================

    #[test]
    fn test_account_new() {
        let account = Account::new(
            "123".to_string(),
            "My Bank".to_string(),
            "bank".to_string(),
            "USD".to_string(),
            10000,
            "#8b5cf6".to_string(),
            Some("🏦".to_string()),
            "2024-01-01T00:00:00Z".to_string(),
        );

        assert_eq!(account.id, "123");
        assert_eq!(account.name, "My Bank");
        assert_eq!(account.account_type, "bank");
        assert_eq!(account.currency, "USD");
        assert_eq!(account.initial_balance, 10000);
        assert!(!account.is_archived);
    }

    #[test]
    fn test_account_validate_valid() {
        let account = Account::new(
            "123".to_string(),
            "My Bank".to_string(),
            "bank".to_string(),
            "USD".to_string(),
            0,
            "#8b5cf6".to_string(),
            None,
            "2024-01-01T00:00:00Z".to_string(),
        );
        assert!(account.validate());
    }

    #[test]
    fn test_account_validate_empty_name() {
        let account = Account::new(
            "123".to_string(),
            "   ".to_string(), // Empty after trim
            "bank".to_string(),
            "USD".to_string(),
            0,
            "#8b5cf6".to_string(),
            None,
            "2024-01-01T00:00:00Z".to_string(),
        );
        assert!(!account.validate());
    }

    #[test]
    fn test_account_validate_invalid_type() {
        let account = Account::new(
            "123".to_string(),
            "My Bank".to_string(),
            "invalid_type".to_string(),
            "USD".to_string(),
            0,
            "#8b5cf6".to_string(),
            None,
            "2024-01-01T00:00:00Z".to_string(),
        );
        assert!(!account.validate());
    }

    #[test]
    fn test_account_validate_invalid_color() {
        let account = Account::new(
            "123".to_string(),
            "My Bank".to_string(),
            "bank".to_string(),
            "USD".to_string(),
            0,
            "not-a-color".to_string(),
            None,
            "2024-01-01T00:00:00Z".to_string(),
        );
        assert!(!account.validate());
    }

    #[test]
    fn test_account_type_parsing() {
        assert_eq!(AccountType::from_str("bank").unwrap(), AccountType::Bank);
        assert_eq!(AccountType::from_str("cash").unwrap(), AccountType::Cash);
        assert_eq!(
            AccountType::from_str("savings").unwrap(),
            AccountType::Savings
        );
        assert_eq!(
            AccountType::from_str("credit_card").unwrap(),
            AccountType::CreditCard
        );
        assert_eq!(AccountType::from_str("other").unwrap(), AccountType::Other);
        assert!(AccountType::from_str("invalid").is_err());
    }

    #[test]
    fn test_account_type_as_str() {
        assert_eq!(AccountType::Bank.as_str(), "bank");
        assert_eq!(AccountType::Cash.as_str(), "cash");
        assert_eq!(AccountType::Savings.as_str(), "savings");
        assert_eq!(AccountType::CreditCard.as_str(), "credit_card");
        assert_eq!(AccountType::Other.as_str(), "other");
    }

    // ==================== Transaction Tests ====================

    #[test]
    fn test_transaction_new() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Groceries".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );

        assert_eq!(tx.id, "tx1");
        assert_eq!(tx.account_id, "acc1");
        assert_eq!(tx.amount, 5000);
        assert_eq!(tx.category, "Food");
        assert_eq!(tx.transaction_type, "expense");
        assert!(tx.transfer_account_id.is_none());
    }

    #[test]
    fn test_transaction_validate_valid_expense() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );
        assert!(tx.validate());
    }

    #[test]
    fn test_transaction_validate_valid_income() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Salary".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "income".to_string(),
            None,
        );
        assert!(tx.validate());
    }

    #[test]
    fn test_transaction_validate_valid_transfer() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Transfer".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "transfer".to_string(),
            Some("acc2".to_string()),
        );
        assert!(tx.validate());
    }

    #[test]
    fn test_transaction_validate_transfer_missing_destination() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Transfer".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "transfer".to_string(),
            None, // Missing destination
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_validate_expense_with_destination() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            Some("acc2".to_string()), // Should not have destination
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_validate_zero_amount() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            0, // Zero amount
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_validate_invalid_type() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "invalid".to_string(),
            None,
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_validate_empty_account() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "".to_string(), // Empty account
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_type_parsing() {
        assert_eq!(
            FinancialTransactionType::from_str("income").unwrap(),
            FinancialTransactionType::Income
        );
        assert_eq!(
            FinancialTransactionType::from_str("expense").unwrap(),
            FinancialTransactionType::Expense
        );
        assert_eq!(
            FinancialTransactionType::from_str("transfer").unwrap(),
            FinancialTransactionType::Transfer
        );
        assert!(FinancialTransactionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_transaction_type_as_str() {
        assert_eq!(FinancialTransactionType::Income.as_str(), "income");
        assert_eq!(FinancialTransactionType::Expense.as_str(), "expense");
        assert_eq!(FinancialTransactionType::Transfer.as_str(), "transfer");
    }

    #[test]
    fn test_transaction_get_type() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );
        assert_eq!(tx.get_type(), Some(FinancialTransactionType::Expense));
    }

    // ==================== Crypto Transaction Tests ====================

    #[test]
    fn test_crypto_transaction_type_parsing_and_flags() {
        assert_eq!(
            "buy".parse::<CryptoTransactionType>().expect("buy parse"),
            CryptoTransactionType::Buy
        );
        assert_eq!(
            "swap".parse::<CryptoTransactionType>().expect("swap parse"),
            CryptoTransactionType::Swap
        );
        assert!("unknown".parse::<CryptoTransactionType>().is_err());

        assert!(CryptoTransactionType::Buy.is_inflow());
        assert!(CryptoTransactionType::TransferIn.is_inflow());
        assert!(!CryptoTransactionType::Sell.is_inflow());

        assert!(CryptoTransactionType::Sell.is_outflow());
        assert!(CryptoTransactionType::TransferOut.is_outflow());
        assert!(!CryptoTransactionType::Swap.is_outflow());
    }

    #[test]
    fn test_crypto_mechanical_type_from_type_and_subtype() {
        let mut tx = CryptoTransaction::new(
            "c1".to_string(),
            "w1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            "trade".to_string(),
            1.0,
            Some(100.0),
            None,
            "2024-01-01".to_string(),
            None,
        );
        tx.subtype = Some("sell".to_string());
        assert_eq!(tx.mechanical_type(), "sell");
        assert_eq!(tx.get_type(), Some(CryptoTransactionType::Sell));

        tx.transaction_type = "transfer".to_string();
        tx.subtype = Some("withdrawal".to_string());
        assert_eq!(tx.mechanical_type(), "transfer_out");

        tx.transaction_type = "income".to_string();
        tx.subtype = Some("airdrop".to_string());
        assert_eq!(tx.mechanical_type(), "buy");
    }

    #[test]
    fn test_crypto_cost_basis_includes_fee() {
        let tx = CryptoTransaction::new(
            "c2".to_string(),
            "w1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            "trade".to_string(),
            2.0,
            Some(150.0),
            Some(5.0),
            "2024-01-01".to_string(),
            None,
        );
        assert!((tx.cost_basis() - 305.0).abs() < 0.0000001);
    }

    #[test]
    fn test_crypto_transaction_validate_requires_type() {
        let valid = CryptoTransaction::new(
            "c3".to_string(),
            "w1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            "expense".to_string(),
            0.5,
            None,
            None,
            "2024-01-01".to_string(),
            None,
        );
        assert!(valid.validate());

        let invalid_type = CryptoTransaction::new(
            "c4".to_string(),
            "w1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            "buy".to_string(),
            0.5,
            None,
            None,
            "2024-01-01".to_string(),
            None,
        );
        assert!(!invalid_type.validate());
    }

    // ==================== Rewards System Tests ====================

    #[test]
    fn test_streak_reward_new_consecutive() {
        let reward = StreakReward::new(
            "reward1".to_string(),
            "habit1".to_string(),
            true,
            None,
            None,
        );
        assert_eq!(reward.id, "reward1");
        assert_eq!(reward.habit_id, "habit1");
        assert!(reward.is_consecutive);
        assert!(reward.target_days.is_none());
        assert!(reward.target_total.is_none());
        assert!(!reward.created_at.is_empty());
    }

    #[test]
    fn test_streak_reward_new_accumulative() {
        let reward = StreakReward::new(
            "reward2".to_string(),
            "habit2".to_string(),
            false,
            Some(5),
            Some(7),
        );
        assert!(!reward.is_consecutive);
        assert_eq!(reward.target_days, Some(5));
        assert_eq!(reward.target_total, Some(7));
    }

    #[test]
    fn test_streak_reward_validate_consecutive() {
        let reward = StreakReward::new("r1".to_string(), "h1".to_string(), true, None, None);
        assert!(reward.validate());
    }

    #[test]
    fn test_streak_reward_validate_accumulative_valid() {
        let reward = StreakReward::new("r2".to_string(), "h2".to_string(), false, Some(5), Some(7));
        assert!(reward.validate());
    }

    #[test]
    fn test_streak_reward_validate_accumulative_missing_fields() {
        let reward = StreakReward::new("r3".to_string(), "h3".to_string(), false, None, None);
        assert!(!reward.validate());
    }

    #[test]
    fn test_streak_reward_validate_empty_habit_id() {
        let reward = StreakReward::new("r4".to_string(), "".to_string(), true, None, None);
        assert!(!reward.validate());
    }

    #[test]
    fn test_milestone_new() {
        let milestone = Milestone::new(
            "m1".to_string(),
            "r1".to_string(),
            7,
            "One week reward!".to_string(),
        );
        assert_eq!(milestone.id, "m1");
        assert_eq!(milestone.reward_id, "r1");
        assert_eq!(milestone.target_days, 7);
        assert_eq!(milestone.reward_text, "One week reward!");
        assert!(!milestone.unlocked);
        assert!(milestone.unlocked_at.is_none());
    }

    #[test]
    fn test_milestone_unlock() {
        let mut milestone = Milestone::new(
            "m2".to_string(),
            "r2".to_string(),
            30,
            "One month reward!".to_string(),
        );
        assert!(!milestone.unlocked);

        milestone.unlock();

        assert!(milestone.unlocked);
        assert!(milestone.unlocked_at.is_some());
    }

    #[test]
    fn test_milestone_validate_valid() {
        let milestone = Milestone::new(
            "m3".to_string(),
            "r3".to_string(),
            14,
            "Two weeks!".to_string(),
        );
        assert!(milestone.validate());
    }

    #[test]
    fn test_milestone_validate_zero_days() {
        let milestone =
            Milestone::new("m4".to_string(), "r4".to_string(), 0, "Invalid".to_string());
        assert!(!milestone.validate());
    }

    #[test]
    fn test_milestone_validate_empty_reward_text() {
        let milestone = Milestone::new("m5".to_string(), "r5".to_string(), 7, "   ".to_string());
        assert!(!milestone.validate());
    }

    #[test]
    fn test_milestone_validate_empty_reward_id() {
        let milestone = Milestone::new("m6".to_string(), "".to_string(), 7, "Reward".to_string());
        assert!(!milestone.validate());
    }

    #[test]
    fn test_goal_new() {
        let goal = Goal::new(
            "g1".to_string(),
            "Learn Rust".to_string(),
            Some("Complete the Rust book".to_string()),
            "Buy a new keyboard".to_string(),
            Some("2025-12-31".to_string()),
        );
        assert_eq!(goal.id, "g1");
        assert_eq!(goal.name, "Learn Rust");
        assert_eq!(goal.description, Some("Complete the Rust book".to_string()));
        assert_eq!(goal.reward_text, "Buy a new keyboard");
        assert_eq!(goal.deadline, Some("2025-12-31".to_string()));
        assert!(!goal.is_completed);
        assert!(goal.completed_at.is_none());
    }

    #[test]
    fn test_goal_complete() {
        let mut goal = Goal::new(
            "g2".to_string(),
            "Exercise".to_string(),
            None,
            "Ice cream".to_string(),
            None,
        );
        assert!(!goal.is_completed);

        goal.complete();

        assert!(goal.is_completed);
        assert!(goal.completed_at.is_some());
    }

    #[test]
    fn test_goal_validate_valid() {
        let goal = Goal::new(
            "g3".to_string(),
            "Read books".to_string(),
            None,
            "Movie night".to_string(),
            None,
        );
        assert!(goal.validate());
    }

    #[test]
    fn test_goal_validate_empty_name() {
        let goal = Goal::new(
            "g4".to_string(),
            "   ".to_string(),
            None,
            "Reward".to_string(),
            None,
        );
        assert!(!goal.validate());
    }

    #[test]
    fn test_goal_validate_empty_reward_text() {
        let goal = Goal::new(
            "g5".to_string(),
            "Valid name".to_string(),
            None,
            "".to_string(),
            None,
        );
        assert!(!goal.validate());
    }

    #[test]
    fn test_checkpoint_new() {
        let checkpoint = Checkpoint::new(
            "c1".to_string(),
            "g1".to_string(),
            "Read chapter 1".to_string(),
            0,
        );
        assert_eq!(checkpoint.id, "c1");
        assert_eq!(checkpoint.goal_id, "g1");
        assert_eq!(checkpoint.description, "Read chapter 1");
        assert!(!checkpoint.completed);
        assert!(checkpoint.completed_at.is_none());
        assert_eq!(checkpoint.sort_order, 0);
    }

    #[test]
    fn test_checkpoint_toggle() {
        let mut checkpoint =
            Checkpoint::new("c2".to_string(), "g2".to_string(), "Step 1".to_string(), 0);
        assert!(!checkpoint.completed);

        // Toggle on
        let result = checkpoint.toggle();
        assert!(result);
        assert!(checkpoint.completed);
        assert!(checkpoint.completed_at.is_some());

        // Toggle off
        let result = checkpoint.toggle();
        assert!(!result);
        assert!(!checkpoint.completed);
        assert!(checkpoint.completed_at.is_none());
    }

    #[test]
    fn test_checkpoint_validate_valid() {
        let checkpoint = Checkpoint::new(
            "c3".to_string(),
            "g3".to_string(),
            "Do something".to_string(),
            1,
        );
        assert!(checkpoint.validate());
    }

    #[test]
    fn test_checkpoint_validate_empty_goal_id() {
        let checkpoint = Checkpoint::new(
            "c4".to_string(),
            "".to_string(),
            "Description".to_string(),
            0,
        );
        assert!(!checkpoint.validate());
    }

    #[test]
    fn test_checkpoint_validate_empty_description() {
        let checkpoint = Checkpoint::new("c5".to_string(), "g5".to_string(), "   ".to_string(), 0);
        assert!(!checkpoint.validate());
    }

    #[test]
    fn test_achievement_new() {
        let achievement = Achievement::new(
            "a1".to_string(),
            "7 Day Streak".to_string(),
            "Completed a week of exercise!".to_string(),
            "trophy.svg".to_string(),
            "streak".to_string(),
            "r1".to_string(),
        );
        assert_eq!(achievement.id, "a1");
        assert_eq!(achievement.title, "7 Day Streak");
        assert_eq!(achievement.description, "Completed a week of exercise!");
        assert_eq!(achievement.icon_path, "trophy.svg");
        assert_eq!(achievement.achievement_type, "streak");
        assert_eq!(achievement.source_id, "r1");
        assert!(!achievement.achieved_at.is_empty());
    }

    #[test]
    fn test_achievement_validate_streak_type() {
        let achievement = Achievement::new(
            "a2".to_string(),
            "Title".to_string(),
            "Desc".to_string(),
            "icon.svg".to_string(),
            "streak".to_string(),
            "source1".to_string(),
        );
        assert!(achievement.validate());
    }

    #[test]
    fn test_achievement_validate_goal_type() {
        let achievement = Achievement::new(
            "a3".to_string(),
            "Title".to_string(),
            "Desc".to_string(),
            "icon.svg".to_string(),
            "goal".to_string(),
            "source2".to_string(),
        );
        assert!(achievement.validate());
    }

    #[test]
    fn test_achievement_validate_invalid_type() {
        let achievement = Achievement::new(
            "a4".to_string(),
            "Title".to_string(),
            "Desc".to_string(),
            "icon.svg".to_string(),
            "invalid".to_string(),
            "source3".to_string(),
        );
        assert!(!achievement.validate());
    }

    #[test]
    fn test_achievement_validate_empty_title() {
        let achievement = Achievement::new(
            "a5".to_string(),
            "   ".to_string(),
            "Desc".to_string(),
            "icon.svg".to_string(),
            "streak".to_string(),
            "source4".to_string(),
        );
        assert!(!achievement.validate());
    }

    #[test]
    fn test_achievement_validate_empty_source_id() {
        let achievement = Achievement::new(
            "a6".to_string(),
            "Title".to_string(),
            "Desc".to_string(),
            "icon.svg".to_string(),
            "goal".to_string(),
            "".to_string(),
        );
        assert!(!achievement.validate());
    }
}
