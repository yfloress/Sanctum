use serde::{Deserialize, Serialize};
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

/// Represents a cryptocurrency holding in the user's portfolio (LEGACY - kept for migration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoHolding {
    pub id: String,
    pub coin_id: String,
    pub symbol: String,
    pub amount: f64,
    pub purchase_price: f64,
    pub purchase_date: String,
}

impl CryptoHolding {
    /// Creates a new crypto holding
    pub fn new(
        id: String,
        coin_id: String,
        symbol: String,
        amount: f64,
        purchase_price: f64,
        purchase_date: String,
    ) -> Self {
        Self {
            id,
            coin_id,
            symbol,
            amount,
            purchase_price,
            purchase_date,
        }
    }

    /// Validates that amount and price are positive
    pub fn validate(&self) -> bool {
        self.amount > 0.0 && self.purchase_price >= 0.0 && !self.coin_id.is_empty()
    }
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

// ==================== Financial Transactions ====================

/// Representa una transacción financiera
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub amount: i64, // Centavos: $10.00 = 1000
    pub category: String,
    pub description: String,
    pub date: String, // ISO 8601
    #[serde(rename = "type")]
    pub transaction_type: String, // "income" o "expense"
}

/// Resumen de balance financiero
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSummary {
    pub total_balance: i64,
    pub total_income: i64,
    pub total_expense: i64,
}

impl Transaction {
    /// Crea una nueva transacción
    pub fn new(
        id: String,
        amount: i64,
        category: String,
        description: String,
        date: String,
        transaction_type: String,
    ) -> Self {
        Self {
            id,
            amount,
            category,
            description,
            date,
            transaction_type,
        }
    }

    /// Valida que el tipo de transacción sea válido
    pub fn validate_type(&self) -> bool {
        matches!(self.transaction_type.as_str(), "income" | "expense")
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
