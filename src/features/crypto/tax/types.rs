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

//! Tax settings types (per period)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxJurisdiction {
    Chile,
    Usa,
    Other,
}

impl TaxJurisdiction {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaxJurisdiction::Chile => "chile",
            TaxJurisdiction::Usa => "usa",
            TaxJurisdiction::Other => "other",
        }
    }

    pub fn parse_or_default(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "chile" | "cl" => TaxJurisdiction::Chile,
            "usa" | "us" | "united_states" => TaxJurisdiction::Usa,
            "other" | "generic" | "international" => TaxJurisdiction::Other,
            other => {
                log::warn!(
                    "Unrecognized tax jurisdiction '{}', defaulting to Chile",
                    other
                );
                TaxJurisdiction::Chile
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxMethod {
    Fifo,
    Lifo,
    Hifo,
    Cpp,
}

impl TaxMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaxMethod::Fifo => "fifo",
            TaxMethod::Lifo => "lifo",
            TaxMethod::Hifo => "hifo",
            TaxMethod::Cpp => "cpp",
        }
    }

    pub fn parse_or_default(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "fifo" => TaxMethod::Fifo,
            "lifo" => TaxMethod::Lifo,
            "hifo" => TaxMethod::Hifo,
            "cpp" | "avg" | "average" => TaxMethod::Cpp,
            other => {
                log::warn!("Unrecognized tax method '{}', defaulting to FIFO", other);
                TaxMethod::Fifo
            }
        }
    }
}

// ==================== Tax Transaction Classification ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaxTxType {
    Trade,
    Income,
    Expense,
    Transfer,
}

impl TaxTxType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaxTxType::Trade => "trade",
            TaxTxType::Income => "income",
            TaxTxType::Expense => "expense",
            TaxTxType::Transfer => "transfer",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "trade" | "buy" | "sell" | "swap" => Some(TaxTxType::Trade),
            "income" => Some(TaxTxType::Income),
            "expense" => Some(TaxTxType::Expense),
            "transfer" | "move" => Some(TaxTxType::Transfer),
            _ => None,
        }
    }
}

pub const TAX_SUBTYPES_INCOME: [&str; 10] = [
    "interest", "reward", "airdrop", "gift", "staking", "mining", "fork", "payment", "rebate",
    "other",
];

pub const TAX_SUBTYPES_EXPENSE: [&str; 8] = [
    "payment", "gift", "fee", "lost", "stolen", "donation", "sell", "other",
];

pub const TAX_SUBTYPES_TRANSFER: [&str; 2] = ["deposit", "withdrawal"];

pub const TAX_SUBTYPES_TRADE: [&str; 4] = ["buy", "sell", "swap", "other"];

pub fn normalize_tax_type(value: &str) -> Option<String> {
    TaxTxType::parse(value).map(|t| t.as_str().to_string())
}

pub fn normalize_tax_subtype(tax_type: &str, value: &str) -> Option<String> {
    let trimmed = value.trim().to_lowercase();
    if trimmed.is_empty() {
        return None;
    }
    let list = match TaxTxType::parse(tax_type) {
        Some(TaxTxType::Income) => TAX_SUBTYPES_INCOME.as_slice(),
        Some(TaxTxType::Expense) => TAX_SUBTYPES_EXPENSE.as_slice(),
        Some(TaxTxType::Transfer) => TAX_SUBTYPES_TRANSFER.as_slice(),
        Some(TaxTxType::Trade) => TAX_SUBTYPES_TRADE.as_slice(),
        None => return None,
    };
    if list
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(&trimmed))
    {
        Some(trimmed)
    } else {
        None
    }
}

pub fn is_loss_only_subtype(subtype: &str) -> bool {
    matches!(subtype, "lost" | "stolen")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tax_type_accepts_aliases() {
        assert_eq!(normalize_tax_type("trade").as_deref(), Some("trade"));
        assert_eq!(normalize_tax_type("buy").as_deref(), Some("trade"));
        assert_eq!(normalize_tax_type("sell").as_deref(), Some("trade"));
        assert_eq!(normalize_tax_type("swap").as_deref(), Some("trade"));
        assert_eq!(normalize_tax_type("income").as_deref(), Some("income"));
        assert_eq!(normalize_tax_type("expense").as_deref(), Some("expense"));
        assert_eq!(normalize_tax_type("transfer").as_deref(), Some("transfer"));
        assert_eq!(normalize_tax_type("move").as_deref(), Some("transfer"));
        assert!(normalize_tax_type("unknown").is_none());
    }

    #[test]
    fn normalize_tax_subtype_requires_matching_type() {
        assert_eq!(
            normalize_tax_subtype("income", "airdrop").as_deref(),
            Some("airdrop")
        );
        assert_eq!(
            normalize_tax_subtype("expense", "stolen").as_deref(),
            Some("stolen")
        );
        assert_eq!(
            normalize_tax_subtype("transfer", "deposit").as_deref(),
            Some("deposit")
        );
        assert_eq!(
            normalize_tax_subtype("trade", "swap").as_deref(),
            Some("swap")
        );
        assert!(normalize_tax_subtype("income", "sell").is_none());
        assert!(normalize_tax_subtype("trade", "airdrop").is_none());
    }

    #[test]
    fn loss_only_subtypes_are_detected() {
        assert!(is_loss_only_subtype("lost"));
        assert!(is_loss_only_subtype("stolen"));
        assert!(!is_loss_only_subtype("fee"));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxPeriodSettings {
    pub period_id: String, // Year (YYYY) for now
    pub jurisdiction: TaxJurisdiction,
    pub method: TaxMethod,
    pub include_swaps: bool,
    pub include_fee_crypto: bool,
}

impl TaxPeriodSettings {
    pub fn defaults_for(period_id: &str) -> Self {
        Self {
            period_id: period_id.to_string(),
            jurisdiction: TaxJurisdiction::Chile,
            method: TaxMethod::Hifo,
            include_swaps: true,
            include_fee_crypto: true,
        }
    }

    /// Helper for backward-compatible string access to jurisdiction.
    pub fn jurisdiction_str(&self) -> &'static str {
        self.jurisdiction.as_str()
    }

    /// Helper for backward-compatible string access to method.
    pub fn method_str(&self) -> &'static str {
        self.method.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaxSettingsStore {
    pub periods: Vec<TaxPeriodSettings>,
}

impl TaxSettingsStore {
    pub fn get_for_period(&self, period_id: &str) -> Option<TaxPeriodSettings> {
        self.periods
            .iter()
            .find(|p| p.period_id == period_id)
            .cloned()
    }

    pub fn upsert(&mut self, settings: TaxPeriodSettings) {
        if let Some(existing) = self
            .periods
            .iter_mut()
            .find(|p| p.period_id == settings.period_id)
        {
            *existing = settings;
        } else {
            self.periods.push(settings);
        }
    }
}
