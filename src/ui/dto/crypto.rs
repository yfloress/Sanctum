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

//! Crypto domain DTOs.
//!
//! Covers: portfolio, wallets, transactions, tickers, catalog, tax.
//!
//! The transaction input DTOs expose `into_command` mappings that parse their
//! stringly amounts into the domain command structs the `CryptoService`
//! consumes, tagging the offending field on [`AppError`].

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::features::crypto::{
    NewCryptoSwap, NewCryptoTransaction, NewCryptoTransfer, UpdateCryptoTransaction,
};

/// Parse a required `f64` amount, tagging `field` on failure.
fn parse_required_f64(raw: &str, field: &str) -> Result<f64, AppError> {
    raw.parse::<f64>()
        .map_err(|_| AppError::validation(format!("Invalid {field}")).with_field(field))
}

/// Parse an optional, soft `f64` (ignored when empty/zero/unparseable), matching
/// the previous lenient command behavior for fees.
fn parse_optional_nonzero_f64(raw: &str) -> Option<f64> {
    raw.parse::<f64>().ok().filter(|v| *v != 0.0)
}

/// Parse an optional `f64` from an optional string, ignoring parse failures.
fn parse_optional_f64(raw: &Option<String>) -> Option<f64> {
    raw.as_ref().and_then(|s| s.parse::<f64>().ok())
}

// ==================== Portfolio ====================

/// Portfolio overview for the main crypto page.
#[derive(Debug, Clone, Serialize)]
pub struct PortfolioResponse {
    pub total_value: String,
    pub unrealized_pnl: String,
    pub unrealized_pnl_negative: bool,
    pub realized_ytd: String,
    pub realized_ytd_negative: bool,
    pub roi: String,
    pub roi_negative: bool,
    pub assets: Vec<PortfolioAssetDto>,
    pub distribution: Vec<DistributionItem>,
    pub fx_rate: Option<FxRateDto>,
    pub last_updated: Option<String>,
}

/// A single asset in the portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct PortfolioAssetDto {
    pub coin_id: String,
    pub symbol: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub price: String,
    pub price_change_24h: String,
    pub price_change_24h_negative: bool,
    pub amount: String,
    pub value: String,
    pub allocation_pct: f64,
}

/// Distribution chart item (for pie/donut chart).
#[derive(Debug, Clone, Serialize)]
pub struct DistributionItem {
    pub coin_id: String,
    pub symbol: String,
    pub value: f64,
    pub percentage: f64,
}

/// FX rate badge data.
#[derive(Debug, Clone, Serialize)]
pub struct FxRateDto {
    pub pair: String,
    pub rate: String,
    pub is_live: bool,
}

// ==================== Wallets ====================

/// Wallet as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct WalletDto {
    pub id: String,
    pub name: String,
    pub category: String,
    pub icon_path: Option<String>,
    pub total_value: String,
    pub assets_count: i32,
}

/// Wallets list response.
#[derive(Debug, Clone, Serialize)]
pub struct WalletsResponse {
    pub wallets: Vec<WalletDto>,
    pub simple_list: Vec<WalletSimpleDto>,
}

/// Simplified wallet for dropdowns.
#[derive(Debug, Clone, Serialize)]
pub struct WalletSimpleDto {
    pub id: String,
    pub name: String,
    pub category: String,
}

/// Wallet detail with holdings and transaction history.
#[derive(Debug, Clone, Serialize)]
pub struct WalletDetailResponse {
    pub id: String,
    pub name: String,
    pub category: String,
    pub icon_path: Option<String>,
    pub total_value: String,
    pub holdings: Vec<WalletHoldingDto>,
    pub transactions: Vec<CryptoTransactionDto>,
}

/// A single holding within a wallet.
#[derive(Debug, Clone, Serialize)]
pub struct WalletHoldingDto {
    pub coin_id: String,
    pub symbol: String,
    pub amount: String,
    pub value: String,
    pub price: String,
}

/// Input for creating a wallet.
#[derive(Debug, Clone, Deserialize)]
pub struct WalletInput {
    pub name: String,
    pub category: String,
    pub icon: Option<String>,
}

/// Input for renaming a wallet.
#[derive(Debug, Clone, Deserialize)]
pub struct WalletRenameInput {
    pub id: String,
    pub new_name: String,
}

/// Input for updating wallet icon.
#[derive(Debug, Clone, Deserialize)]
pub struct WalletIconInput {
    pub id: String,
    pub icon: String,
}

/// Input for deleting a wallet.
#[derive(Debug, Clone, Deserialize)]
pub struct WalletDeleteInput {
    pub id: String,
    pub force: bool,
}

// ==================== Crypto Transactions ====================

/// Crypto transaction as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct CryptoTransactionDto {
    pub id: String,
    pub wallet_id: String,
    pub wallet_name: String,
    pub coin_id: String,
    pub symbol: String,
    pub transaction_type: String,
    pub subtype: Option<String>,
    pub amount: String,
    pub price: String,
    pub fee: String,
    pub fee_coin_id: Option<String>,
    pub fee_amount: Option<String>,
    pub value: String,
    pub date: String,
    pub notes: Option<String>,
    pub has_related_tx: bool,
}

/// Paginated crypto transaction list.
#[derive(Debug, Clone, Serialize)]
pub struct CryptoTransactionListResponse {
    pub transactions: Vec<CryptoTransactionDto>,
    pub has_more: bool,
}

/// Input for adding a crypto transaction (buy/sell/income/expense).
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoTransactionInput {
    pub wallet_id: String,
    pub coin_id: String,
    pub symbol: String,
    pub transaction_type: String,
    pub amount: String,
    pub price: String,
    pub fee: String,
    pub fee_coin_id: Option<String>,
    pub fee_coin_amount: Option<String>,
    pub date: String,
    pub notes: Option<String>,
    pub subtype: Option<String>,
    pub override_proceeds: Option<String>,
    pub override_cost_basis: Option<String>,
}

/// Input for adding a crypto transfer between wallets.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoTransferInput {
    pub from_wallet_id: String,
    pub to_wallet_id: String,
    pub coin_id: String,
    pub symbol: String,
    pub from_amount: String,
    pub to_amount: String,
    pub fee: String,
    pub fee_coin_id: Option<String>,
    pub fee_coin_amount: Option<String>,
    pub date: String,
    pub notes: Option<String>,
}

/// Input for adding a crypto swap.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoSwapInput {
    pub wallet_id: String,
    pub from_coin_id: String,
    pub from_symbol: String,
    pub from_amount: String,
    pub to_coin_id: String,
    pub to_symbol: String,
    pub to_amount: String,
    pub fee: String,
    pub fee_coin_id: Option<String>,
    pub fee_coin_amount: Option<String>,
    pub date: String,
    pub notes: Option<String>,
}

/// Input for updating a crypto transaction.
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoTransactionUpdateInput {
    pub id: String,
    pub amount: String,
    pub price: String,
    pub fee: String,
    pub fee_coin_id: Option<String>,
    pub fee_coin_amount: Option<String>,
    pub date: String,
    pub notes: Option<String>,
    pub subtype: Option<String>,
    pub override_proceeds: Option<String>,
    pub override_cost_basis: Option<String>,
}

// ==================== DTO -> domain command mapping ====================

impl CryptoTransactionInput {
    /// Parse amounts and map into the add-transaction command.
    pub fn into_command(self) -> Result<NewCryptoTransaction, AppError> {
        Ok(NewCryptoTransaction {
            wallet_id: self.wallet_id,
            coin_id: self.coin_id,
            symbol: self.symbol,
            transaction_type: self.transaction_type,
            amount: parse_required_f64(&self.amount, "amount")?,
            price_per_coin: Some(parse_required_f64(&self.price, "price")?),
            fee: parse_optional_nonzero_f64(&self.fee),
            fee_coin_id: self.fee_coin_id,
            fee_amount: parse_optional_f64(&self.fee_coin_amount),
            date: self.date,
            notes: self.notes,
            subtype: self.subtype,
            override_proceeds: parse_optional_f64(&self.override_proceeds),
            override_cost_basis: parse_optional_f64(&self.override_cost_basis),
        })
    }
}

impl CryptoTransferInput {
    /// Parse amounts and map into the transfer command.
    pub fn into_command(self) -> Result<NewCryptoTransfer, AppError> {
        Ok(NewCryptoTransfer {
            from_wallet_id: self.from_wallet_id,
            to_wallet_id: self.to_wallet_id,
            coin_id: self.coin_id,
            symbol: self.symbol,
            from_amount: parse_required_f64(&self.from_amount, "from_amount")?,
            to_amount: parse_required_f64(&self.to_amount, "to_amount")?,
            fee: parse_optional_nonzero_f64(&self.fee),
            fee_coin_id: self.fee_coin_id,
            fee_amount: parse_optional_f64(&self.fee_coin_amount),
            date: self.date,
            notes: self.notes,
        })
    }
}

impl CryptoSwapInput {
    /// Parse amounts and map into the swap command.
    pub fn into_command(self) -> Result<NewCryptoSwap, AppError> {
        Ok(NewCryptoSwap {
            wallet_id: self.wallet_id,
            from_coin_id: self.from_coin_id,
            from_symbol: self.from_symbol,
            from_amount: parse_required_f64(&self.from_amount, "from_amount")?,
            to_coin_id: self.to_coin_id,
            to_symbol: self.to_symbol,
            to_amount: parse_required_f64(&self.to_amount, "to_amount")?,
            fee: parse_optional_nonzero_f64(&self.fee),
            fee_coin_id: self.fee_coin_id,
            fee_amount: parse_optional_f64(&self.fee_coin_amount),
            date: self.date,
            notes: self.notes,
        })
    }
}

impl CryptoTransactionUpdateInput {
    /// Parse amounts and map into the update-transaction command.
    pub fn into_command(self) -> Result<UpdateCryptoTransaction, AppError> {
        Ok(UpdateCryptoTransaction {
            id: self.id,
            amount: parse_required_f64(&self.amount, "amount")?,
            price_per_coin: Some(parse_required_f64(&self.price, "price")?),
            fee: parse_optional_nonzero_f64(&self.fee),
            fee_coin_id: self.fee_coin_id,
            fee_amount: parse_optional_f64(&self.fee_coin_amount),
            date: self.date,
            notes: self.notes,
            subtype: self.subtype,
            override_proceeds: parse_optional_f64(&self.override_proceeds),
            override_cost_basis: parse_optional_f64(&self.override_cost_basis),
        })
    }
}

/// Data to populate the edit transaction form.
#[derive(Debug, Clone, Serialize)]
pub struct CryptoTransactionEditData {
    pub id: String,
    pub wallet_name: String,
    pub coin_id: String,
    pub symbol: String,
    pub transaction_type: String,
    pub subtype: Option<String>,
    pub amount: String,
    pub price: String,
    pub fee: String,
    pub fee_coin_id: Option<String>,
    pub fee_coin_amount: Option<String>,
    pub date: String,
    pub notes: Option<String>,
    pub override_proceeds: Option<String>,
    pub override_cost_basis: Option<String>,
    pub is_paired_swap: bool,
}

/// Asset detail view data.
#[derive(Debug, Clone, Serialize)]
pub struct AssetDetailResponse {
    pub coin_id: String,
    pub symbol: String,
    pub name: String,
    pub price: String,
    pub total_amount: String,
    pub total_value: String,
    pub unrealized_pnl: String,
    pub unrealized_pnl_negative: bool,
    pub wallet_breakdown: Vec<AssetWalletBreakdown>,
    pub transactions: Vec<CryptoTransactionDto>,
}

/// Per-wallet breakdown for an asset.
#[derive(Debug, Clone, Serialize)]
pub struct AssetWalletBreakdown {
    pub wallet_id: String,
    pub wallet_name: String,
    pub amount: String,
    pub value: String,
}

// ==================== Tickers & Catalog ====================

/// Market ticker data for the price bar.
#[derive(Debug, Clone, Serialize)]
pub struct TickerDto {
    pub coin_id: String,
    pub symbol: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub price: String,
    pub change_24h: String,
    pub change_24h_negative: bool,
}

/// Ticker option for the configure modal.
#[derive(Debug, Clone, Serialize)]
pub struct TickerOptionDto {
    pub coin_id: String,
    pub symbol: String,
    pub name: String,
    pub enabled: bool,
    pub is_custom: bool,
}

/// Coin catalog entry for the catalog browser.
#[derive(Debug, Clone, Serialize)]
pub struct CoinCatalogDto {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub is_custom: bool,
    pub is_favorite: bool,
}

/// Input for adding a custom coin.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomCoinInput {
    pub id: String,
    pub name: String,
    pub symbol: String,
}

// ==================== Tax ====================

/// Tax settings for a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSettingsDto {
    pub period_id: String,
    pub jurisdiction: String,
    pub method: String,
    pub include_swaps: bool,
    pub include_fee_crypto: bool,
    pub excluded_wallet_ids: Vec<String>,
}

/// Wallet tax exclusion item.
#[derive(Debug, Clone, Serialize)]
pub struct TaxWalletExclusionDto {
    pub wallet_id: String,
    pub wallet_name: String,
    pub excluded: bool,
}

/// Tax report summary for the UI.
#[derive(Debug, Clone, Serialize)]
pub struct TaxReportDto {
    pub period_id: String,
    pub jurisdiction: String,
    pub method: String,
    pub disposals_count: usize,
    pub total_proceeds: String,
    pub total_cost: String,
    pub total_gain: String,
    pub total_gain_negative: bool,
    pub short_term_gain: Option<String>,
    pub long_term_gain: Option<String>,
    pub events: Vec<TaxEventDto>,
    pub warnings: Vec<TaxWarningDto>,
    pub readiness: Vec<TaxReadinessDto>,
}

/// A single tax disposal event.
#[derive(Debug, Clone, Serialize)]
pub struct TaxEventDto {
    pub tx_id: String,
    pub date: String,
    pub coin_id: String,
    pub symbol: String,
    pub amount: String,
    pub proceeds: String,
    pub cost_basis: String,
    pub gain: String,
    pub gain_negative: bool,
    pub term: Option<String>,
    pub disposal_type: String,
}

/// Tax warning item.
#[derive(Debug, Clone, Serialize)]
pub struct TaxWarningDto {
    pub code: String,
    pub message: String,
    pub tx_id: Option<String>,
}

/// Tax readiness check item.
#[derive(Debug, Clone, Serialize)]
pub struct TaxReadinessDto {
    pub code: String,
    pub status: String,
    pub detail: String,
}

/// Full tax summary payload (report + extra readiness signals).
#[derive(Debug, Clone, Serialize)]
pub struct TaxSummaryDto {
    pub report: TaxReportDto,
    pub taxable_income_total: String,
    pub taxable_income_count: usize,
    pub end_balance_value: Option<String>,
    pub end_balance_missing: usize,
    pub transactions_in_period: usize,
    pub volume_processed: String,
}

/// IPC (Chilean tax indicator) summary.
#[derive(Debug, Clone, Serialize)]
pub struct IpcSummaryDto {
    pub records_count: usize,
    pub date_range: Option<String>,
}

// ==================== Charts ====================

/// Portfolio trend chart data for ECharts.
#[derive(Debug, Clone, Serialize)]
pub struct PortfolioTrendData {
    pub dates: Vec<String>,
    pub values: Vec<f64>,
}

/// Crypto asset price data for the IPC boundary.
/// Replaces direct use of `models::CryptoAsset` in commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoAssetPriceDto {
    pub id: String,
    pub symbol: String,
    pub name: String,
    /// Raw price in USD — used for internal calculations.
    pub current_price: f64,
    /// Price formatted and converted to the user's preferred currency — use this for display.
    #[serde(default)]
    pub current_price_display: String,
    pub price_change_percentage_24h: f64,
    pub last_updated: String,
}
