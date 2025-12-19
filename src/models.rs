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

// ==================== Crypto Ledger System ====================

/// Wallet category types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WalletCategory {
    Exchange,
    WalletSingle,
    WalletMulti,
}

impl WalletCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            WalletCategory::Exchange => "exchange",
            WalletCategory::WalletSingle => "wallet_single",
            WalletCategory::WalletMulti => "wallet_multi",
        }
    }
}

impl FromStr for WalletCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exchange" => Ok(WalletCategory::Exchange),
            "wallet_single" => Ok(WalletCategory::WalletSingle),
            "wallet_multi" => Ok(WalletCategory::WalletMulti),
            _ => Err(()),
        }
    }
}

/// Represents a crypto wallet (exchange, hardware wallet, software wallet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoWallet {
    pub id: String,
    pub name: String,
    pub category: String, // "exchange", "wallet_single", "wallet_multi"
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

/// Represents a single crypto transaction in the ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoTransaction {
    pub id: String,
    pub wallet_id: String,
    pub coin_id: String, // CoinGecko ID (e.g., "bitcoin")
    pub symbol: String,  // Symbol (e.g., "BTC")
    #[serde(rename = "type")]
    pub transaction_type: String, // "buy", "sell", "transfer_in", "transfer_out", "swap"
    pub amount: f64,     // Amount of coins
    pub price_per_coin: Option<f64>, // Price in USD at transaction time
    pub fee: Option<f64>, // Fee paid (in USD)
    pub fee_coin_id: Option<String>, // If fee was paid in crypto
    pub fee_amount: Option<f64>, // Fee amount in crypto (if applicable)
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
            date,
            notes,
            related_tx_id: None,
        }
    }

    pub fn validate(&self) -> bool {
        !self.wallet_id.is_empty()
            && !self.coin_id.is_empty()
            && !self.symbol.is_empty()
            && self.amount > 0.0
            && self
                .transaction_type
                .parse::<CryptoTransactionType>()
                .is_ok()
    }

    pub fn get_type(&self) -> Option<CryptoTransactionType> {
        self.transaction_type.parse::<CryptoTransactionType>().ok()
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

/// Representa una transacción financiera
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

/// Resumen de balance financiero
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
    /// Crea una nueva transacción
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

    /// Valida que el tipo de transacción sea válido
    pub fn validate_type(&self) -> bool {
        self.transaction_type
            .parse::<FinancialTransactionType>()
            .is_ok()
    }

    /// Valida que la transacción sea coherente
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
    pub created_at: String, // ISO-8601 datetime
    pub archived: bool,     // Soft delete flag
}

impl Habit {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        created_at: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            color,
            created_at,
            archived: false,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.trim().is_empty() && self.color.starts_with('#') && self.color.len() == 7
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
