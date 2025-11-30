use serde::{Deserialize, Serialize};

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

/// Represents a cryptocurrency holding in the user's portfolio
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
