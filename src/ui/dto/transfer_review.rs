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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransferReviewDto {
    pub rows: Vec<TransferMatchDto>,
    pub more_suggestions: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TransferMatchDto {
    pub outgoing_id: String,
    pub incoming_id: String,
    pub from_wallet: String,
    pub to_wallet: String,
    pub sent_at: String,
    pub received_at: String,
    pub outgoing_notes: Option<String>,
    pub incoming_notes: Option<String>,
    pub symbol: String,
    pub sent: f64,
    pub received: f64,
    pub linked: bool,
}
