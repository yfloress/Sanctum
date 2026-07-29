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

//! Crypto command objects (CQRS-lite inputs).
//!
//! Domain-owned, already-parsed inputs for the mutating [`super::CryptoService`]
//! transaction operations. They replace long positional parameter lists and keep
//! the service decoupled from the IPC DTO layer (`crate::ui::dto`): the command
//! layer maps `ui::dto::crypto::*Input` (raw, stringly amounts) into these
//! structs, parsing amounts to `f64` and tagging the offending field on error.

/// Add a crypto buy/sell/etc. transaction.
#[derive(Debug, Clone)]
pub struct NewCryptoTransaction {
    pub wallet_id: String,
    pub coin_id: String,
    pub symbol: String,
    pub transaction_type: String,
    pub amount: f64,
    pub price_per_coin: Option<f64>,
    pub fee: Option<f64>,
    pub fee_coin_id: Option<String>,
    pub fee_amount: Option<f64>,
    pub date: String,
    pub notes: Option<String>,
    pub subtype: Option<String>,
    pub override_proceeds: Option<f64>,
    pub override_cost_basis: Option<f64>,
}

/// Add a transfer of one coin between two wallets.
#[derive(Debug, Clone)]
pub struct NewCryptoTransfer {
    pub from_wallet_id: String,
    pub to_wallet_id: String,
    pub coin_id: String,
    pub symbol: String,
    pub from_amount: f64,
    pub to_amount: f64,
    pub fee: Option<f64>,
    pub fee_coin_id: Option<String>,
    pub fee_amount: Option<f64>,
    pub date: String,
    pub notes: Option<String>,
}

/// Add a swap of one coin for another within a wallet.
#[derive(Debug, Clone)]
pub struct NewCryptoSwap {
    pub wallet_id: String,
    pub from_coin_id: String,
    pub from_symbol: String,
    pub from_amount: f64,
    pub to_coin_id: String,
    pub to_symbol: String,
    pub to_amount: f64,
    pub fee: Option<f64>,
    pub fee_coin_id: Option<String>,
    pub fee_amount: Option<f64>,
    pub date: String,
    pub notes: Option<String>,
}

/// Update an existing crypto transaction.
#[derive(Debug, Clone)]
pub struct UpdateCryptoTransaction {
    pub id: String,
    pub amount: f64,
    pub price_per_coin: Option<f64>,
    pub fee: Option<f64>,
    pub fee_coin_id: Option<String>,
    pub fee_amount: Option<f64>,
    pub date: String,
    pub notes: Option<String>,
    pub subtype: Option<String>,
    pub override_proceeds: Option<f64>,
    pub override_cost_basis: Option<f64>,
}
