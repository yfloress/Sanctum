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

//! Kraken CSV parsers
//!
//! Supports two export formats:
//!
//! ## Ledger CSV (recommended — covers ALL transaction types)
//!
//! Each trade generates **two rows** linked by `refid` (one per asset).
//! `spend`/`receive` pairs with the same `refid` are also trades.
//! Internal staking transfers (subtypes like `spottostaking`) are skipped.
//!
//! ## Trades CSV (spot/margin trades only)
//!
//! Simpler format with one row per trade including pair, price, cost, volume.

use std::collections::HashMap;

use chrono::NaiveDateTime;
use csv::StringRecord;

use super::common::{
    append_tax_non_usd_quote_reason, format_datetime, is_fiat, is_usd_valued_quote,
    normalize_header, normalize_kraken_currency, parse_decimal, parse_kraken_pair, parse_timestamp,
};
use super::{ExchangeParser, ExchangeSource, ParseResult};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

fn annotate_non_usd_quote_note(notes: Option<String>, quote_symbol: &str) -> Option<String> {
    append_tax_non_usd_quote_reason(notes, quote_symbol)
}

// ─── Kraken Ledger types ────────────────────────────────────────────────────

/// Known ledger entry types from Kraken's export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LedgerType {
    Trade,
    MarginTrade,
    Earn,
    Rollover,
    Deposit,
    Withdrawal,
    Transfer,
    Adjustment,
    Spend,
    Receive,
    Settled,
    Staking,
    InviteBonus,
    Unknown,
}

impl LedgerType {
    fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "trade" => LedgerType::Trade,
            "margin trade" => LedgerType::MarginTrade,
            "earn" => LedgerType::Earn,
            "rollover" => LedgerType::Rollover,
            "deposit" => LedgerType::Deposit,
            "withdrawal" => LedgerType::Withdrawal,
            "transfer" => LedgerType::Transfer,
            "adjustment" => LedgerType::Adjustment,
            "spend" => LedgerType::Spend,
            "receive" => LedgerType::Receive,
            "settled" => LedgerType::Settled,
            "staking" => LedgerType::Staking,
            "invite bonus" => LedgerType::InviteBonus,
            _ => LedgerType::Unknown,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            LedgerType::Trade => "trade",
            LedgerType::MarginTrade => "margin trade",
            LedgerType::Earn => "earn",
            LedgerType::Rollover => "rollover",
            LedgerType::Deposit => "deposit",
            LedgerType::Withdrawal => "withdrawal",
            LedgerType::Transfer => "transfer",
            LedgerType::Adjustment => "adjustment",
            LedgerType::Spend => "spend",
            LedgerType::Receive => "receive",
            LedgerType::Settled => "settled",
            LedgerType::Staking => "staking",
            LedgerType::InviteBonus => "invite bonus",
            LedgerType::Unknown => "unknown",
        }
    }
}

/// Known ledger subtypes (primarily staking-related).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LedgerSubtype {
    Allocation,
    Deallocation,
    Autoallocate,
    Reward,
    Migration,
    SpotToStaking,
    StakingFromSpot,
    StakingToSpot,
    SpotFromStaking,
    SpotToFutures,
    SpotFromFutures,
    Other,
}

impl LedgerSubtype {
    fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim().to_lowercase();
        if trimmed.is_empty() {
            return None;
        }
        Some(match trimmed.as_str() {
            "allocation" => LedgerSubtype::Allocation,
            "deallocation" => LedgerSubtype::Deallocation,
            "autoallocate" => LedgerSubtype::Autoallocate,
            "reward" => LedgerSubtype::Reward,
            "migration" => LedgerSubtype::Migration,
            "spottostaking" => LedgerSubtype::SpotToStaking,
            "stakingfromspot" => LedgerSubtype::StakingFromSpot,
            "stakingtospot" => LedgerSubtype::StakingToSpot,
            "spotfromstaking" => LedgerSubtype::SpotFromStaking,
            "spottofutures" => LedgerSubtype::SpotToFutures,
            "spotfromfutures" => LedgerSubtype::SpotFromFutures,
            _ => LedgerSubtype::Other,
        })
    }

    /// Returns `true` for subtypes that represent internal balance movements
    /// (spot <-> staking, spot <-> futures) which should be skipped.
    fn is_internal_transfer(&self) -> bool {
        matches!(
            self,
            LedgerSubtype::Allocation
                | LedgerSubtype::Deallocation
                | LedgerSubtype::Autoallocate
                | LedgerSubtype::Migration
                | LedgerSubtype::SpotToStaking
                | LedgerSubtype::StakingFromSpot
                | LedgerSubtype::StakingToSpot
                | LedgerSubtype::SpotFromStaking
                | LedgerSubtype::SpotToFutures
                | LedgerSubtype::SpotFromFutures
        )
    }
}

/// A parsed row from the Kraken ledger CSV.
#[derive(Debug, Clone)]
struct LedgerRow {
    time: NaiveDateTime,
    ledger_type: LedgerType,
    /// Normalised ticker (e.g. `BTC` instead of `XXBT`).
    symbol: String,
    /// Signed amount: negative = debit, positive = credit.
    amount: f64,
    fee: f64,
    line_number: usize,
}

impl LedgerRow {
    fn is_incoming(&self) -> bool {
        self.amount >= 0.0
    }

    fn abs_amount(&self) -> f64 {
        self.amount.abs()
    }
}

/// Accumulates the two sides of a single Kraken trade (linked by `refid`).
///
/// A trade can manifest as:
/// - Two `trade` rows (one negative, one positive)
/// - A `spend` + `receive` pair
/// - Mixed combinations in edge cases
#[derive(Debug, Default)]
struct PendingTrade {
    spend: Option<LedgerRow>,
    receive: Option<LedgerRow>,
    trade_out: Option<LedgerRow>,
    trade_in: Option<LedgerRow>,
}

impl PendingTrade {
    fn insert(&mut self, row: LedgerRow) {
        match row.ledger_type {
            LedgerType::Spend => self.spend = Some(row),
            LedgerType::Receive => self.receive = Some(row),
            LedgerType::Trade | LedgerType::MarginTrade => {
                if row.is_incoming() {
                    self.trade_in = Some(row);
                } else {
                    self.trade_out = Some(row);
                }
            }
            _ => {}
        }
    }

    fn is_complete(&self) -> bool {
        (self.spend.is_some() && self.receive.is_some())
            || (self.trade_out.is_some() && self.trade_in.is_some())
    }

    /// Resolves the pending trade into zero or more `ImportCryptoTransaction`s.
    fn resolve(mut self, refid: &str, wallet_name: &str) -> Vec<ImportCryptoTransaction> {
        // Prefer trade_out/trade_in pair, fall back to spend/receive.
        // Use `.take()` so every field is consumed at most once,
        // keeping the fallback branch valid after earlier checks.
        let (outgoing, incoming) =
            if let (Some(out), Some(inc)) = (self.trade_out.take(), self.trade_in.take()) {
                (out, inc)
            } else if let (Some(spend), Some(recv)) = (self.spend.take(), self.receive.take()) {
                (spend, recv)
            } else {
                // Incomplete pair — emit single-sided transactions for whatever we have
                let mut results = Vec::new();
                for row in [self.spend, self.receive, self.trade_out, self.trade_in]
                    .into_iter()
                    .flatten()
                {
                    if let Some(tx) = single_row_to_transaction(&row, wallet_name, refid) {
                        results.push(tx);
                    }
                }
                return results;
            };

        let out_fiat = is_fiat(&outgoing.symbol);
        let in_fiat = is_fiat(&incoming.symbol);

        // Only true fiat should be treated as the pricing side for
        // buy/sell. Stablecoins remain crypto so their balances are
        // updated through swaps (USDT outflow/inflow is tracked).
        let out_is_pricing = out_fiat;
        let in_is_pricing = in_fiat;

        // Both fiat — skip entirely
        if out_fiat && in_fiat {
            return Vec::new();
        }

        let date = format_datetime(outgoing.time);
        let mut notes = Some(format!(
            "Kraken {} | Ref: {}",
            outgoing.ledger_type.label(),
            refid
        ));

        // Fiat -> Crypto = buy
        if out_is_pricing && !in_is_pricing {
            let out_is_usd_valued = is_usd_valued_quote(&outgoing.symbol);
            if !out_is_usd_valued {
                notes = annotate_non_usd_quote_note(notes, &outgoing.symbol);
            }
            let price = if incoming.abs_amount() > 0.0 && out_is_usd_valued {
                Some(outgoing.abs_amount() / incoming.abs_amount())
            } else {
                None
            };

            let fee = if outgoing.fee.abs() > f64::EPSILON && out_is_usd_valued {
                Some(outgoing.fee.abs())
            } else if incoming.fee.abs() > f64::EPSILON {
                // Fee in crypto
                None // handled below
            } else {
                None
            };

            let mut tx = ImportCryptoTransaction {
                date,
                wallet: wallet_name.to_string(),
                symbol: incoming.symbol.clone(),
                transaction_type: "trade".to_string(),
                amount: incoming.abs_amount(),
                subtype: Some("buy".to_string()),
                price_per_coin: price,
                fee,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: None,
                swap_to_amount: None,
                fee_coin_symbol: None,
                fee_amount: None,
                notes,
            };

            // If the fee is in the crypto asset
            if incoming.fee.abs() > f64::EPSILON && fee.is_none() {
                tx.fee_coin_symbol = Some(incoming.symbol.clone());
                tx.fee_amount = Some(incoming.fee.abs());
            }

            return vec![tx];
        }

        // Crypto -> Fiat = sell
        if !out_is_pricing && in_is_pricing {
            let in_is_usd_valued = is_usd_valued_quote(&incoming.symbol);
            if !in_is_usd_valued {
                notes = annotate_non_usd_quote_note(notes, &incoming.symbol);
            }
            let price = if outgoing.abs_amount() > 0.0 && in_is_usd_valued {
                Some(incoming.abs_amount() / outgoing.abs_amount())
            } else {
                None
            };

            let fee = if incoming.fee.abs() > f64::EPSILON && in_is_usd_valued {
                Some(incoming.fee.abs())
            } else {
                None
            };

            let mut tx = ImportCryptoTransaction {
                date,
                wallet: wallet_name.to_string(),
                symbol: outgoing.symbol.clone(),
                transaction_type: "trade".to_string(),
                amount: outgoing.abs_amount(),
                subtype: Some("sell".to_string()),
                price_per_coin: price,
                fee,
                override_proceeds: None,
                override_cost_basis: None,
                swap_to_symbol: None,
                swap_to_amount: None,
                fee_coin_symbol: None,
                fee_amount: None,
                notes,
            };

            // If the fee is in the sold crypto
            if outgoing.fee.abs() > f64::EPSILON && fee.is_none() {
                tx.fee_coin_symbol = Some(outgoing.symbol.clone());
                tx.fee_amount = Some(outgoing.fee.abs());
            }

            return vec![tx];
        }

        // Crypto -> Crypto = swap (includes stablecoin-to-stablecoin)
        // Guard: same-symbol pairs are no-ops (e.g. internal movements that
        // slipped through subtype filtering). Skip silently instead of
        // producing an invalid swap that fails downstream validation.
        if outgoing.symbol.eq_ignore_ascii_case(&incoming.symbol) {
            return Vec::new();
        }

        let fee_coin_symbol;
        let fee_amount;
        if outgoing.fee.abs() > f64::EPSILON {
            fee_coin_symbol = Some(outgoing.symbol.clone());
            fee_amount = Some(outgoing.fee.abs());
        } else if incoming.fee.abs() > f64::EPSILON {
            fee_coin_symbol = Some(incoming.symbol.clone());
            fee_amount = Some(incoming.fee.abs());
        } else {
            fee_coin_symbol = None;
            fee_amount = None;
        }

        let tx = ImportCryptoTransaction {
            date,
            wallet: wallet_name.to_string(),
            symbol: outgoing.symbol.clone(),
            transaction_type: "trade".to_string(),
            amount: outgoing.abs_amount(),
            subtype: Some("swap".to_string()),
            price_per_coin: None,
            fee: None,
            override_proceeds: None,
            override_cost_basis: None,
            swap_to_symbol: Some(incoming.symbol.clone()),
            swap_to_amount: Some(incoming.abs_amount()),
            fee_coin_symbol,
            fee_amount,
            notes,
        };

        vec![tx]
    }
}

/// Converts a single (unpaired) ledger row into an `ImportCryptoTransaction`.
/// Returns `None` for fiat-only rows or rows that should be skipped.
fn single_row_to_transaction(
    row: &LedgerRow,
    wallet_name: &str,
    refid: &str,
) -> Option<ImportCryptoTransaction> {
    if is_fiat(&row.symbol) {
        return None;
    }

    let date = format_datetime(row.time);
    let notes = Some(format!(
        "Kraken {} | Ref: {}",
        row.ledger_type.label(),
        refid
    ));

    let fee_coin_symbol = if row.fee.abs() > f64::EPSILON {
        Some(row.symbol.clone())
    } else {
        None
    };
    let fee_amount = if row.fee.abs() > f64::EPSILON {
        Some(row.fee.abs())
    } else {
        None
    };

    let (tx_type, subtype) = match row.ledger_type {
        LedgerType::Deposit | LedgerType::Receive | LedgerType::InviteBonus => {
            if row.is_incoming() {
                ("transfer".to_string(), Some("deposit".to_string()))
            } else {
                ("transfer".to_string(), Some("withdrawal".to_string()))
            }
        }
        LedgerType::Withdrawal | LedgerType::Spend => {
            ("transfer".to_string(), Some("withdrawal".to_string()))
        }
        LedgerType::Staking | LedgerType::Earn => {
            if row.is_incoming() {
                ("income".to_string(), Some("staking".to_string()))
            } else {
                // Unstaking / deallocation that wasn't filtered as internal
                ("transfer".to_string(), Some("withdrawal".to_string()))
            }
        }
        LedgerType::Transfer
        | LedgerType::Adjustment
        | LedgerType::Settled
        | LedgerType::Rollover
        | LedgerType::MarginTrade => {
            if row.is_incoming() {
                ("transfer".to_string(), Some("deposit".to_string()))
            } else {
                ("transfer".to_string(), Some("withdrawal".to_string()))
            }
        }
        _ => {
            if row.is_incoming() {
                ("income".to_string(), Some("other".to_string()))
            } else {
                ("expense".to_string(), Some("other".to_string()))
            }
        }
    };

    Some(ImportCryptoTransaction {
        date,
        wallet: wallet_name.to_string(),
        symbol: row.symbol.clone(),
        transaction_type: tx_type,
        amount: row.abs_amount(),
        subtype,
        price_per_coin: None,
        fee: None,
        override_proceeds: None,
        override_cost_basis: None,
        swap_to_symbol: None,
        swap_to_amount: None,
        fee_coin_symbol,
        fee_amount,
        notes,
    })
}

// ─── Column index resolution ────────────────────────────────────────────────

/// Resolves column indices from the CSV header, supporting both v1 and v2
/// Kraken ledger formats. Returns a mapping of logical field -> column index.
fn resolve_ledger_columns(headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (i, col) in headers.iter().enumerate() {
        let key = normalize_header(col);
        match key.as_str() {
            "txid" => {
                map.insert("txid", i);
            }
            "refid" => {
                map.insert("refid", i);
            }
            "time" => {
                map.insert("time", i);
            }
            "type" => {
                map.insert("type", i);
            }
            "subtype" => {
                map.insert("subtype", i);
            }
            "asset" => {
                map.insert("asset", i);
            }
            "amount" => {
                map.insert("amount", i);
            }
            "fee" => {
                map.insert("fee", i);
            }
            "balance" => {
                map.insert("balance", i);
            }
            _ => {}
        }
    }
    map
}

fn get_field<'a>(record: &'a StringRecord, cols: &HashMap<&str, usize>, name: &str) -> &'a str {
    cols.get(name)
        .and_then(|&i| record.get(i))
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or("")
}

mod ledger;
mod trades;

pub use ledger::KrakenLedgerParser;
pub use trades::KrakenTradesParser;

#[cfg(test)]
mod tests_ledger;
#[cfg(test)]
mod tests_trades_and_edges;
