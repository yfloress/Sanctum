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

//! Data types for the ingestion system
//!
//! Contains intermediate representations for imported data and result summaries.

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use super::parsers::exchange::ExchangeSource;

/// File format detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    Json,
    CsvTransactions,
    CsvHabitLogs,
    CsvCrypto,
    TextMixed,                   // Mixed content with prefixes (T;, H;, C;)
    ExchangeCsv(ExchangeSource), // Exchange/wallet-specific CSV (Kraken, Binance, Feather, etc.)
}

impl ImportFormat {
    pub fn name(&self) -> &'static str {
        match self {
            ImportFormat::Json => "JSON",
            ImportFormat::CsvTransactions => "CSV",
            ImportFormat::CsvHabitLogs => "CSV",
            ImportFormat::CsvCrypto => "CSV",
            ImportFormat::TextMixed => "Plain Text",
            ImportFormat::ExchangeCsv(source) => source.label(),
        }
    }

    pub fn data_type(&self) -> &'static str {
        match self {
            ImportFormat::Json | ImportFormat::TextMixed => "Mixed",
            ImportFormat::CsvTransactions => "Transactions",
            ImportFormat::CsvHabitLogs => "Habit Logs",
            ImportFormat::CsvCrypto => "Crypto",
            ImportFormat::ExchangeCsv(_) => "Crypto",
        }
    }
}

/// Intermediate transaction representation (parsed from any format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTransaction {
    pub date: String,
    pub account: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub amount: f64,
    pub currency: String,
    pub category: String,
    pub description: String,
    pub transfer_to_account: Option<String>,
}

/// Intermediate habit log representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportHabitLog {
    pub habit: String,
    pub date: String,
    pub completed: bool,
}

/// Intermediate crypto transaction representation.
///
/// JSON format — `type` is the transaction type category, `subtype` the specific action:
/// ```json
/// { "type": "income", "subtype": "airdrop", "amount": 0.5, ... }
/// { "type": "trade",  "subtype": "buy",     "amount": 1.0, ... }
/// ```
///
/// Valid `type` values: `trade`, `income`, `expense`, `transfer`.
/// Each has its own set of valid subtypes (see `SUBTYPES_*` constants).
///
/// The `type` field maps directly to `CryptoTransaction.transaction_type`
/// (transaction type category) and `subtype` maps to `CryptoTransaction.subtype`.
/// No intermediate "resolved" fields — type/subtype info is stored as-is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCryptoTransaction {
    pub date: String,
    pub wallet: String,
    pub symbol: String, // e.g., "BTC", "ETH"
    #[serde(rename = "type")]
    pub transaction_type: String, // type: trade, income, expense, transfer
    pub amount: f64,
    /// Subtype within the category (e.g. "buy", "airdrop", "deposit").
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub price_per_coin: Option<f64>, // USD price at transaction time
    #[serde(default)]
    pub fee: Option<f64>, // Fee in USD
    #[serde(default)]
    pub override_proceeds: Option<f64>,
    #[serde(default)]
    pub override_cost_basis: Option<f64>,
    #[serde(default)]
    pub swap_to_symbol: Option<String>, // Swap target symbol (e.g., "ETH")
    #[serde(default)]
    pub swap_to_amount: Option<f64>, // Swap target amount
    #[serde(default)]
    pub fee_coin_symbol: Option<String>, // Fee coin symbol if paid in crypto
    #[serde(default)]
    pub fee_amount: Option<f64>, // Fee amount in crypto (if applicable)
    #[serde(default)]
    pub notes: Option<String>,
}

impl ImportCryptoTransaction {
    /// Returns the mechanical transaction type derived from
    /// `type` (transaction type category) + `subtype`.
    ///
    /// Result is one of: `"buy"`, `"sell"`, `"swap"`, `"transfer_in"`, `"transfer_out"`.
    pub fn mechanical_type(&self) -> &str {
        let tx_type = self.transaction_type.trim().to_lowercase();
        let normalized_subtype = self
            .subtype
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase);
        crate::features::crypto::tax::types::derive_mechanical_type(
            &tx_type,
            normalized_subtype.as_deref(),
        )
    }
}

/// Error details for a single row
#[derive(Debug, Clone)]
pub struct RowError {
    pub line_number: usize,
    pub field: Option<String>,
    pub message: String,
    pub raw_data: Option<String>,
}

impl RowError {
    pub fn new(line_number: usize, field: Option<&str>, message: impl Into<String>) -> Self {
        Self {
            line_number,
            field: field.map(String::from),
            message: message.into(),
            raw_data: None,
        }
    }

    pub fn with_raw_data(mut self, data: impl Into<String>) -> Self {
        self.raw_data = Some(data.into());
        self
    }
}

/// Represents a proposed change during preview
#[derive(Debug, Clone)]
pub struct PreviewChange {
    pub change_type: String, // e.g. "Transaction", "Crypto", "Habit"
    pub summary: String,     // e.g. "+ 1000 USD (Salary)"
    pub details: String,     // e.g. "Account: Bank -> Savings"
}

/// Summary of the import process
#[derive(Debug, Default)]
pub struct ImportSummary {
    pub format: String,
    pub data_type: String,
    pub total_processed: usize,
    pub inserted: usize,
    pub skipped: usize,
    pub errors: usize,
    pub error_details: Vec<RowError>,
    pub skipped_reasons: Vec<String>,
    pub preview_changes: Vec<PreviewChange>,
}

impl ImportSummary {
    pub fn new(format: &str, data_type: &str) -> Self {
        Self {
            format: format.to_string(),
            data_type: data_type.to_string(),
            ..Default::default()
        }
    }

    pub fn merge(&mut self, other: ImportSummary) {
        self.total_processed += other.total_processed;
        self.inserted += other.inserted;
        self.skipped += other.skipped;
        self.errors += other.errors;
        self.error_details.extend(other.error_details);
        self.skipped_reasons.extend(other.skipped_reasons);
        self.preview_changes.extend(other.preview_changes);
    }

    pub fn record_error(&mut self, error: RowError) {
        self.errors += 1;
        self.total_processed += 1;
        self.error_details.push(error);
    }

    pub fn record_skipped(&mut self, reason: &str) {
        self.skipped += 1;
        self.total_processed += 1;
        self.skipped_reasons.push(reason.to_string());
    }

    pub fn record_inserted(&mut self) {
        self.inserted += 1;
        self.total_processed += 1;
    }

    pub fn record_preview_change(&mut self, change_type: &str, summary: String, details: String) {
        self.inserted += 1; // Count as "would be inserted"
        self.total_processed += 1;
        self.preview_changes.push(PreviewChange {
            change_type: change_type.to_string(),
            summary,
            details,
        });
    }
}

/// Deduplication key for transactions
#[derive(Debug, Clone, Eq)]
pub struct TransactionDedupKey {
    pub date: String,
    pub account_id: String,
    pub transfer_account_id: Option<String>,
    pub currency: String,
    pub amount_cents: i64,
    pub transaction_type: String,
    pub description_normalized: String,
}

impl TransactionDedupKey {
    pub fn new(
        date: &str,
        account_id: &str,
        transfer_account_id: Option<&str>,
        currency: &str,
        amount_cents: i64,
        tx_type: &str,
        description: &str,
    ) -> Self {
        Self {
            date: date.to_string(),
            account_id: account_id.to_string(),
            transfer_account_id: transfer_account_id.map(str::to_string),
            currency: currency.to_uppercase(),
            amount_cents,
            transaction_type: tx_type.to_lowercase(),
            description_normalized: description.trim().to_lowercase(),
        }
    }
}

impl PartialEq for TransactionDedupKey {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
            && self.account_id == other.account_id
            && self.transfer_account_id == other.transfer_account_id
            && self.currency == other.currency
            && self.amount_cents == other.amount_cents
            && self.transaction_type == other.transaction_type
            && self.description_normalized == other.description_normalized
    }
}

impl Hash for TransactionDedupKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.date.hash(state);
        self.account_id.hash(state);
        self.transfer_account_id.hash(state);
        self.currency.hash(state);
        self.amount_cents.hash(state);
        self.transaction_type.hash(state);
        self.description_normalized.hash(state);
    }
}

/// Deduplication key for crypto transactions
#[derive(Debug, Clone, Eq)]
pub struct CryptoDedupKey {
    pub date: String,
    pub wallet_id: String,
    pub coin_id: String,
    pub transaction_type: String, // mechanical type: buy/sell/swap/...
    pub category_type: String,    // category type: trade/income/expense/transfer
    pub subtype: String,          // normalized subtype, empty when absent
    pub amount_satoshis: i64, // amount * 10^8 for precision
    pub price_micros: Option<i64>, // price_per_coin * 10^6
    pub pair_coin_id: String, // for swaps, track the counterpart asset
}

impl CryptoDedupKey {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        date: &str,
        wallet_id: &str,
        coin_id: &str,
        mechanical_type: &str,
        category_type: &str,
        subtype: Option<&str>,
        amount: f64,
        price_per_coin: Option<f64>,
        pair_coin_id: Option<&str>,
    ) -> Self {
        Self {
            date: date.to_string(),
            wallet_id: wallet_id.to_string(),
            coin_id: coin_id.to_string(),
            transaction_type: mechanical_type.to_lowercase(),
            category_type: category_type.to_lowercase(),
            subtype: subtype.unwrap_or_default().to_lowercase(),
            amount_satoshis: (amount * 100_000_000.0).round() as i64,
            price_micros: price_per_coin.map(|p| (p * 1_000_000.0).round() as i64),
            pair_coin_id: pair_coin_id.unwrap_or_default().to_string(),
        }
    }
}

impl PartialEq for CryptoDedupKey {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
            && self.wallet_id == other.wallet_id
            && self.coin_id == other.coin_id
            && self.transaction_type == other.transaction_type
            && self.category_type == other.category_type
            && self.subtype == other.subtype
            && self.amount_satoshis == other.amount_satoshis
            && self.price_micros == other.price_micros
            && self.pair_coin_id == other.pair_coin_id
    }
}

impl Hash for CryptoDedupKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.date.hash(state);
        self.wallet_id.hash(state);
        self.coin_id.hash(state);
        self.transaction_type.hash(state);
        self.category_type.hash(state);
        self.subtype.hash(state);
        self.amount_satoshis.hash(state);
        self.price_micros.hash(state);
        self.pair_coin_id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::{CryptoDedupKey, ImportCryptoTransaction, TransactionDedupKey};

    fn base_import(tx_type: &str) -> ImportCryptoTransaction {
        ImportCryptoTransaction {
            date: "2026-01-10".to_string(),
            wallet: "Ledger".to_string(),
            symbol: "BTC".to_string(),
            transaction_type: tx_type.to_string(),
            amount: 0.5,
            subtype: None,
            price_per_coin: None,
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: None,
            swap_to_amount: None,
            fee_coin_symbol: None,
            fee_amount: None,
            notes: None,
        }
    }

    // ── mechanical_type() derivation ──

    #[test]
    fn test_mechanical_trade_buy() {
        let mut tx = base_import("trade");
        tx.subtype = Some("buy".to_string());
        assert_eq!(tx.mechanical_type(), "buy");
        assert_eq!(tx.transaction_type, "trade");
        assert_eq!(tx.subtype.as_deref(), Some("buy"));
    }

    #[test]
    fn test_mechanical_trade_sell() {
        let mut tx = base_import("trade");
        tx.subtype = Some("sell".to_string());
        assert_eq!(tx.mechanical_type(), "sell");
    }

    #[test]
    fn test_mechanical_trade_swap() {
        let mut tx = base_import("trade");
        tx.subtype = Some("swap".to_string());
        assert_eq!(tx.mechanical_type(), "swap");
    }

    #[test]
    fn test_mechanical_type_is_case_insensitive_for_type_and_subtype() {
        let mut tx = base_import("Trade");
        tx.subtype = Some("SWAP".to_string());
        assert_eq!(tx.mechanical_type(), "swap");
    }

    #[test]
    fn test_mechanical_trade_other() {
        let tx = base_import("trade");
        // No subtype or unknown → defaults to buy
        assert_eq!(tx.mechanical_type(), "buy");
    }

    #[test]
    fn test_mechanical_transfer_deposit() {
        let mut tx = base_import("transfer");
        tx.subtype = Some("deposit".to_string());
        assert_eq!(tx.mechanical_type(), "transfer_in");
    }

    #[test]
    fn test_mechanical_transfer_withdrawal() {
        let mut tx = base_import("transfer");
        tx.subtype = Some("withdrawal".to_string());
        assert_eq!(tx.mechanical_type(), "transfer_out");
    }

    #[test]
    fn test_mechanical_income_airdrop() {
        let mut tx = base_import("income");
        tx.subtype = Some("airdrop".to_string());
        assert_eq!(tx.mechanical_type(), "buy");
    }

    #[test]
    fn test_mechanical_income_staking() {
        let mut tx = base_import("income");
        tx.subtype = Some("staking".to_string());
        assert_eq!(tx.mechanical_type(), "buy");
    }

    #[test]
    fn test_mechanical_expense_stolen() {
        let mut tx = base_import("expense");
        tx.subtype = Some("stolen".to_string());
        assert_eq!(tx.mechanical_type(), "sell");
    }

    #[test]
    fn test_mechanical_expense_donation() {
        let mut tx = base_import("expense");
        tx.subtype = Some("donation".to_string());
        assert_eq!(tx.mechanical_type(), "sell");
    }

    // ── edge cases ──

    #[test]
    fn test_mechanical_category_without_subtype() {
        let tx = base_import("income");
        // Income without subtype still derives to "buy"
        assert_eq!(tx.mechanical_type(), "buy");
    }

    #[test]
    fn test_mechanical_unknown_type_defaults_to_buy() {
        let tx = base_import("banana");
        // Unknown category falls back to "buy"
        assert_eq!(tx.mechanical_type(), "buy");
    }

    // ── Dedup tests ──

    #[test]
    fn test_transfer_dedup_includes_destination() {
        let base = TransactionDedupKey::new(
            "2024-01-15",
            "account-a",
            Some("account-b"),
            "USD",
            1000,
            "transfer",
            "Move funds",
        );
        let different_dest = TransactionDedupKey::new(
            "2024-01-15",
            "account-a",
            Some("account-c"),
            "USD",
            1000,
            "transfer",
            "Move funds",
        );

        assert_ne!(base, different_dest);
    }

    #[test]
    fn test_crypto_dedup_distinguishes_category_and_subtype() {
        let buy_trade = CryptoDedupKey::new(
            "2026-01-10 10:00:00",
            "wallet-1",
            "bitcoin",
            "buy",
            "trade",
            Some("buy"),
            1.0,
            Some(100_000.0),
            None,
        );
        let income_airdrop = CryptoDedupKey::new(
            "2026-01-10 10:00:00",
            "wallet-1",
            "bitcoin",
            "buy",
            "income",
            Some("airdrop"),
            1.0,
            Some(100_000.0),
            None,
        );
        assert_ne!(buy_trade, income_airdrop);
    }

    #[test]
    fn test_crypto_dedup_distinguishes_price_when_rest_matches() {
        let first = CryptoDedupKey::new(
            "2026-01-10 10:00:00",
            "wallet-1",
            "bitcoin",
            "buy",
            "trade",
            Some("buy"),
            0.01,
            Some(95_000.0),
            None,
        );
        let second = CryptoDedupKey::new(
            "2026-01-10 10:00:00",
            "wallet-1",
            "bitcoin",
            "buy",
            "trade",
            Some("buy"),
            0.01,
            Some(96_000.0),
            None,
        );
        assert_ne!(first, second);
    }

    #[test]
    fn test_crypto_dedup_keeps_same_day_different_time_as_distinct() {
        let earlier = CryptoDedupKey::new(
            "2026-01-10 10:00:00",
            "wallet-1",
            "bitcoin",
            "buy",
            "trade",
            Some("buy"),
            0.25,
            Some(95_000.0),
            None,
        );
        let later = CryptoDedupKey::new(
            "2026-01-10 12:00:00",
            "wallet-1",
            "bitcoin",
            "sell",
            "trade",
            Some("sell"),
            0.25,
            Some(96_000.0),
            None,
        );
        assert_ne!(earlier, later);
    }
}
