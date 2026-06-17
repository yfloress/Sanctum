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

use super::*;

/// Converts a single (non-paired) Binance row into an `ImportCryptoTransaction`.
/// Returns `None` for fiat rows, skippable ops, or rows that need pairing.
pub(super) fn single_row_to_transaction(
    row: &BinanceRow,
    wallet_name: &str,
) -> Option<ImportCryptoTransaction> {
    if is_fiat(&row.symbol) {
        return None;
    }

    if row.operation.needs_pairing() {
        return None;
    }

    if row.operation.should_skip() {
        return None;
    }

    let date = format_datetime(row.timestamp);
    let notes = Some(format!("Binance {} | {}", row.operation_raw, row.remark));

    let (tx_type, subtype) = match row.operation {
        BinanceOperation::Buy | BinanceOperation::TransactionBuy => {
            ("trade".to_string(), Some("buy".to_string()))
        }
        BinanceOperation::Sell | BinanceOperation::TransactionSold => {
            ("trade".to_string(), Some("sell".to_string()))
        }
        BinanceOperation::P2PTrade => {
            if row.change > 0.0 {
                ("trade".to_string(), Some("buy".to_string()))
            } else {
                ("trade".to_string(), Some("sell".to_string()))
            }
        }
        BinanceOperation::Deposit => ("transfer".to_string(), Some("deposit".to_string())),
        BinanceOperation::Withdraw => ("transfer".to_string(), Some("withdrawal".to_string())),
        BinanceOperation::Distribution | BinanceOperation::AirdropAssets => {
            ("income".to_string(), Some("airdrop".to_string()))
        }
        BinanceOperation::StakingRewards => ("income".to_string(), Some("staking".to_string())),
        BinanceOperation::Fee => ("expense".to_string(), Some("fee".to_string())),
        BinanceOperation::CardCashback => ("income".to_string(), Some("rebate".to_string())),
        BinanceOperation::CardSpending => {
            if row.change > 0.0 {
                // Refund
                ("income".to_string(), Some("rebate".to_string()))
            } else {
                ("expense".to_string(), Some("payment".to_string()))
            }
        }
        BinanceOperation::TransactionSpend => {
            // Standalone spend without a matching revenue — treat as expense
            ("expense".to_string(), Some("payment".to_string()))
        }
        BinanceOperation::TransactionRevenue => {
            // Standalone revenue without a matching spend — treat as income
            ("income".to_string(), Some("other".to_string()))
        }
        BinanceOperation::Unknown => {
            if row.change > 0.0 {
                ("income".to_string(), Some("other".to_string()))
            } else {
                ("expense".to_string(), Some("other".to_string()))
            }
        }
        _ => return None,
    };

    Some(ImportCryptoTransaction {
        date,
        wallet: wallet_name.to_string(),
        symbol: row.symbol.clone(),
        transaction_type: tx_type,
        amount: row.change.abs(),
        subtype,
        price_per_coin: None,
        fee: None,
        override_proceeds: None,
        override_cost_basis: None,
        swap_to_symbol: None,
        swap_to_amount: None,
        fee_coin_symbol: None,
        fee_amount: None,
        notes,
    })
}

/// Best-effort fallback for rows that are normally expected to be paired
/// (`Binance Convert`, `Small Assets Exchange BNB`) but arrive unpaired in
/// a partial CSV export window.
///
/// We avoid silently dropping legit crypto balance deltas:
/// - positive change -> `trade:buy`
/// - negative change -> `trade:sell`
///
/// Price cannot be inferred safely without the counterpart row, so it stays `None`.
pub(super) fn unpaired_row_to_transaction(
    row: &BinanceRow,
    wallet_name: &str,
) -> Option<ImportCryptoTransaction> {
    if !row.operation.needs_pairing() {
        return single_row_to_transaction(row, wallet_name);
    }

    if is_fiat(&row.symbol) || row.operation.should_skip() {
        return None;
    }

    let date = format_datetime(row.timestamp);
    let subtype = if row.change >= 0.0 { "buy" } else { "sell" };
    let notes = Some(format!(
        "Binance {} (unpaired fallback) | {}",
        row.operation_raw, row.remark
    ));

    Some(ImportCryptoTransaction {
        date,
        wallet: wallet_name.to_string(),
        symbol: row.symbol.clone(),
        transaction_type: "trade".to_string(),
        amount: row.change.abs(),
        subtype: Some(subtype.to_string()),
        price_per_coin: None,
        fee: None,
        override_proceeds: None,
        override_cost_basis: None,
        swap_to_symbol: None,
        swap_to_amount: None,
        fee_coin_symbol: None,
        fee_amount: None,
        notes,
    })
}
