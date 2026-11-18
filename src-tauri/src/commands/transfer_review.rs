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
use sanctum::error::AppError;
use sanctum::features::crypto::CryptoService;
use sanctum::ui::dto::transfer_review::{TransferMatchDto, TransferReviewDto};
use tauri::State;

#[tauri::command]
pub fn list_transfer_reviews(
    crypto: State<'_, CryptoService>,
) -> Result<TransferReviewDto, AppError> {
    let review = crypto.list_transfer_reviews()?;
    Ok(TransferReviewDto {
        more_suggestions: review.more_suggestions,
        rows: review
            .rows
            .into_iter()
            .map(|row| TransferMatchDto {
                outgoing_id: row.outgoing_id,
                incoming_id: row.incoming_id,
                from_wallet: row.from_wallet,
                to_wallet: row.to_wallet,
                sent_at: row.sent_at,
                received_at: row.received_at,
                symbol: row.symbol,
                outgoing_notes: row.outgoing_notes,
                incoming_notes: row.incoming_notes,
                sent: row.sent,
                received: row.received,
                linked: row.linked,
            })
            .collect(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_transfer_link(
    crypto: State<'_, CryptoService>,
    outgoing_id: String,
    incoming_id: String,
    linked: bool,
) -> Result<(), AppError> {
    Ok(crypto.review_transfer_link(&outgoing_id, &incoming_id, linked)?)
}
