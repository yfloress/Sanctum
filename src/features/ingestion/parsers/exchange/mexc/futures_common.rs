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

use super::spot::parse_mexc_pair;
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

pub(super) fn resolve_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = col.trim().trim_matches('"');
        match key {
            "Time(UTC+00:00)" => {
                map.insert("time", i);
            }
            "Futures Trading Pair" => {
                map.insert("pair", i);
            }
            "futures" => {
                map.insert("pair", i);
            }
            "Futures" => {
                map.insert("pair", i);
            }
            "Crypto" => {
                map.insert("symbol", i);
            }
            "Fund Type" => {
                map.insert("fund_type", i);
            }
            "Fund Flow Type" => {
                map.insert("flow_type", i);
            }
            "Amount" => {
                map.insert("amount", i);
            }
            "copy_trader_uid" => {
                map.insert("copy_trader_uid", i);
            }
            "copy_state" => {
                map.insert("copy_state", i);
            }
            "fee" | "Fee" => {
                map.insert("fee", i);
            }
            "create_time(UTC+00:00)" => {
                map.insert("create_time", i);
            }
            "close_time(UTC+00:00)" => {
                map.insert("close_time", i);
            }
            "Open Time(UTC+00:00)" => {
                map.insert("open_time", i);
            }
            "Close Time" => {
                map.insert("close_time", i);
            }
            "Position Profit/Loss(USDT)" => {
                map.insert("pnl", i);
            }
            "Closing PNL" => {
                map.insert("pnl", i);
            }
            "Realized PNL" => {
                map.insert("pnl", i);
            }
            "Trading Fee" => {
                map.insert("trading_fee", i);
            }
            "Fee-payment Crypto" => {
                map.insert("fee_symbol", i);
            }
            "Status" => {
                map.insert("status", i);
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

pub(super) fn parse_pair_quote_symbol(pair_raw: &str) -> Option<String> {
    let (base, quote) = parse_mexc_pair(pair_raw)?;
    if base.is_empty() || quote.is_empty() {
        None
    } else {
        Some(quote)
    }
}

pub(super) fn pick_time<'a>(primary: &'a str, secondary: &'a str) -> &'a str {
    if primary.trim().is_empty() {
        secondary
    } else {
        primary
    }
}

pub(super) fn status_is_final(raw: &str) -> bool {
    let status = raw.trim().to_lowercase();
    if status.is_empty() {
        return true;
    }
    if status.contains("pending")
        || status.contains("cancel")
        || status.contains("open")
        || status.contains("new")
        || status.contains("reject")
    {
        return false;
    }
    status.contains("fill")
        || status.contains("close")
        || status.contains("complete")
        || status.contains("success")
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_tx(
    result: &mut Vec<(usize, ImportCryptoTransaction)>,
    line_number: usize,
    date: &str,
    wallet_name: &str,
    symbol: &str,
    amount: f64,
    transaction_type: &str,
    subtype: &str,
    notes: String,
) {
    if amount <= 0.0 {
        return;
    }

    result.push((
        line_number,
        ImportCryptoTransaction {
            date: date.to_string(),
            wallet: wallet_name.to_string(),
            symbol: symbol.to_string(),
            transaction_type: transaction_type.to_string(),
            amount,
            subtype: Some(subtype.to_string()),
            price_per_coin: None,
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: None,
            swap_to_amount: None,
            fee_coin_symbol: None,
            fee_amount: None,
            notes: Some(notes),
        },
    ));
}

pub(super) struct PnlFeeContext<'a> {
    pub line_number: usize,
    pub date: &'a str,
    pub wallet_name: &'a str,
    pub pnl_symbol: &'a str,
    pub fee_symbol: &'a str,
    pub note_prefix: &'a str,
}

pub(super) fn push_pnl_and_fee(
    items: &mut Vec<(usize, ImportCryptoTransaction)>,
    ctx: &PnlFeeContext<'_>,
    pnl: Option<f64>,
    fee: Option<f64>,
) {
    if let Some(pnl_value) = pnl
        && pnl_value.abs() > 0.0
    {
        if pnl_value > 0.0 {
            push_tx(
                items,
                ctx.line_number,
                ctx.date,
                ctx.wallet_name,
                ctx.pnl_symbol,
                pnl_value,
                "income",
                "reward",
                format!("{} | component=pnl", ctx.note_prefix),
            );
        } else {
            push_tx(
                items,
                ctx.line_number,
                ctx.date,
                ctx.wallet_name,
                ctx.pnl_symbol,
                pnl_value.abs(),
                "expense",
                "other",
                format!("{} | component=pnl", ctx.note_prefix),
            );
        }
    }

    if let Some(fee_value) = fee
        && fee_value > 0.0
    {
        push_tx(
            items,
            ctx.line_number,
            ctx.date,
            ctx.wallet_name,
            ctx.fee_symbol,
            fee_value,
            "expense",
            "fee",
            format!("{} | component=fee", ctx.note_prefix),
        );
    }
}

pub(super) fn missing_required(display: &str) -> RowError {
    RowError::new(1, None, format!("Missing required MEXC column: '{display}'"))
}
