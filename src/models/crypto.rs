// Sanctum — a privacy-first personal finance and crypto vault.
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
    pub category: String,
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

    pub fn is_inflow(&self) -> bool {
        matches!(
            self,
            CryptoTransactionType::Buy | CryptoTransactionType::TransferIn
        )
    }

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
    pub coin_id: String,
    pub symbol: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub amount: f64,
    pub price_per_coin: Option<f64>,
    pub fee: Option<f64>,
    pub fee_coin_id: Option<String>,
    pub fee_amount: Option<f64>,
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub override_proceeds: Option<f64>,
    #[serde(default)]
    pub override_cost_basis: Option<f64>,
    pub date: String,
    pub notes: Option<String>,
    pub related_tx_id: Option<String>,
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

    pub fn mechanical_type(&self) -> &str {
        let sub = self.subtype.as_deref().unwrap_or("");
        match self.transaction_type.as_str() {
            "trade" => match sub {
                "sell" => "sell",
                "swap" => "swap",
                _ => "buy",
            },
            "transfer" => match sub {
                "withdrawal" => "transfer_out",
                _ => "transfer_in",
            },
            "income" => "buy",
            "expense" => "sell",
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

    pub fn get_type(&self) -> Option<CryptoTransactionType> {
        self.mechanical_type().parse::<CryptoTransactionType>().ok()
    }

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
    pub total_cost_basis: f64,
    pub avg_buy_price: f64,
    pub current_price: f64,
    pub current_value: f64,
    pub unrealized_pnl: f64,
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
