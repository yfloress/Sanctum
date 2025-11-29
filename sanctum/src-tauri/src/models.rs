use serde::{Deserialize, Serialize};

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
