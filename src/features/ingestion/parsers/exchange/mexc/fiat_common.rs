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

use std::collections::HashMap;

use csv::StringRecord;

use super::super::common::{is_fiat, is_quote_currency};
use crate::features::ingestion::types::ImportCryptoTransaction;

pub(super) fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"');
        match key {
            "Order ID" => {
                map.insert("order_id", i);
            }
            "Start Time(UTC+00:00)" => {
                map.insert("start_time", i);
            }
            "Trading Token" => {
                map.insert("trading_token", i);
            }
            "Trading Direction" => {
                map.insert("direction", i);
            }
            "Status" => {
                map.insert("status", i);
            }
            "Order Quantity" => {
                map.insert("order_quantity", i);
            }
            "Settlement Token" => {
                map.insert("settlement_token", i);
            }
            "Order Amount" => {
                map.insert("order_amount", i);
            }
            "Payment Method" => {
                map.insert("payment_method", i);
            }
            "P2P Type" => {
                map.insert("p2p_type", i);
            }
            "Fee" => {
                map.insert("fee", i);
            }
            _ => {}
        }
    }
    map
}

pub(super) fn get_field<'a>(
    record: &'a StringRecord,
    cols: &HashMap<&str, usize>,
    name: &str,
) -> &'a str {
    cols.get(name)
        .and_then(|&i| record.get(i))
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or("")
}

pub(super) fn is_completed_status(raw: &str) -> bool {
    let status = raw.trim().to_lowercase();
    if status.is_empty() {
        return false;
    }
    if status.contains("pending")
        || status.contains("cancel")
        || status.contains("fail")
        || status.contains("reject")
    {
        return false;
    }
    status.contains("complete") || status.contains("success")
}

pub(super) fn parse_is_buy(raw: &str) -> Option<bool> {
    match raw.trim().to_lowercase().as_str() {
        "buy" => Some(true),
        "sell" => Some(false),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn map_spent_received_to_tx(
    date: String,
    wallet_name: &str,
    source_label: &str,
    notes_context: &str,
    spent_symbol: String,
    spent_amount: f64,
    recv_symbol: String,
    recv_amount: f64,
    fee_coin_symbol: Option<String>,
    fee_amount: Option<f64>,
) -> Option<ImportCryptoTransaction> {
    if spent_amount <= 0.0 || recv_amount <= 0.0 {
        return None;
    }

    let spent_fiat = is_fiat(&spent_symbol);
    let recv_fiat = is_fiat(&recv_symbol);
    if spent_fiat && recv_fiat {
        return None;
    }

    let notes = Some(format!(
        "MEXC {source_label} | {notes_context} | {spent_amount} {spent_symbol} -> {recv_amount} {recv_symbol}"
    ));

    if spent_fiat && !recv_fiat {
        return Some(ImportCryptoTransaction {
            date,
            wallet: wallet_name.to_string(),
            symbol: recv_symbol,
            transaction_type: "trade".to_string(),
            amount: recv_amount,
            subtype: Some("buy".to_string()),
            price_per_coin: Some(spent_amount / recv_amount),
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: None,
            swap_to_amount: None,
            fee_coin_symbol,
            fee_amount,
            notes,
        });
    }

    if !spent_fiat && recv_fiat {
        return Some(ImportCryptoTransaction {
            date,
            wallet: wallet_name.to_string(),
            symbol: spent_symbol,
            transaction_type: "trade".to_string(),
            amount: spent_amount,
            subtype: Some("sell".to_string()),
            price_per_coin: Some(recv_amount / spent_amount),
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: None,
            swap_to_amount: None,
            fee_coin_symbol,
            fee_amount,
            notes,
        });
    }

    let spent_quote = is_quote_currency(&spent_symbol);
    let recv_quote = is_quote_currency(&recv_symbol);

    if spent_quote && !recv_quote {
        return Some(ImportCryptoTransaction {
            date,
            wallet: wallet_name.to_string(),
            symbol: recv_symbol,
            transaction_type: "trade".to_string(),
            amount: recv_amount,
            subtype: Some("buy".to_string()),
            price_per_coin: Some(spent_amount / recv_amount),
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: None,
            swap_to_amount: None,
            fee_coin_symbol,
            fee_amount,
            notes,
        });
    }

    if !spent_quote && recv_quote {
        return Some(ImportCryptoTransaction {
            date,
            wallet: wallet_name.to_string(),
            symbol: spent_symbol,
            transaction_type: "trade".to_string(),
            amount: spent_amount,
            subtype: Some("sell".to_string()),
            price_per_coin: Some(recv_amount / spent_amount),
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: None,
            swap_to_amount: None,
            fee_coin_symbol,
            fee_amount,
            notes,
        });
    }

    if spent_symbol.eq_ignore_ascii_case(&recv_symbol) {
        return None;
    }

    Some(ImportCryptoTransaction {
        date,
        wallet: wallet_name.to_string(),
        symbol: spent_symbol,
        transaction_type: "trade".to_string(),
        amount: spent_amount,
        subtype: Some("swap".to_string()),
        price_per_coin: None,
        fee: None,
        override_proceeds: None,
        override_cost_basis: None,
        swap_to_symbol: Some(recv_symbol),
        swap_to_amount: Some(recv_amount),
        fee_coin_symbol,
        fee_amount,
        notes,
    })
}
