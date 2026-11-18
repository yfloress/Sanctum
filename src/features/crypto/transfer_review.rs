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
use std::collections::HashMap;

use jiff::{Span, civil::Date};

use super::{CryptoError, CryptoService};
use crate::models::CryptoTransaction;

const WINDOW_DAYS: i32 = 3;
const MAX_SUGGESTIONS: usize = 200;

pub struct TransferReview {
    pub rows: Vec<TransferMatch>,
    pub more_suggestions: bool,
}

pub struct TransferMatch {
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

fn day(tx: &CryptoTransaction) -> Option<Date> {
    Date::strptime("%Y-%m-%d", tx.date.get(..10)?).ok()
}

fn eligible(out: &CryptoTransaction, inc: &CryptoTransaction) -> bool {
    let (Some(sent), Some(received)) = (day(out), day(inc)) else {
        return false;
    };
    // The tax engine accounts for explicit fees on a linked pair. Inferring
    // a fee from different principal amounts would silently lose that disposal.
    out.wallet_id != inc.wallet_id
        && out.coin_id == inc.coin_id
        && out.mechanical_type() == "transfer_out"
        && inc.mechanical_type() == "transfer_in"
        && out.amount.is_finite()
        && out.amount > 0.0
        && out.amount.to_bits() == inc.amount.to_bits()
        && (sent - received).get_days().abs() <= WINDOW_DAYS
}

impl CryptoService {
    pub fn list_transfer_reviews(&self) -> Result<TransferReview, CryptoError> {
        self.with_db(|db| {
            let txs = db.transfer_movements()?;
            let wallets: HashMap<_, _> = db
                .get_wallets()?
                .into_iter()
                .map(|w| (w.id, w.name))
                .collect();
            let by_id: HashMap<_, _> = txs.iter().map(|t| (t.id.as_str(), t)).collect();
            let mut incoming: HashMap<_, Vec<&CryptoTransaction>> = HashMap::new();
            for tx in &txs {
                if tx.mechanical_type() == "transfer_in"
                    && tx.related_tx_id.is_none()
                    && let Some(date) = day(tx)
                {
                    incoming
                        .entry((tx.coin_id.as_str(), tx.amount.to_bits(), date))
                        .or_default()
                        .push(tx);
                }
            }

            let row = |out: &CryptoTransaction, inc: &CryptoTransaction, linked| TransferMatch {
                outgoing_id: out.id.clone(),
                incoming_id: inc.id.clone(),
                from_wallet: wallets
                    .get(&out.wallet_id)
                    .unwrap_or(&out.wallet_id)
                    .clone(),
                to_wallet: wallets
                    .get(&inc.wallet_id)
                    .unwrap_or(&inc.wallet_id)
                    .clone(),
                sent_at: out.date.clone(),
                received_at: inc.date.clone(),
                outgoing_notes: out.notes.clone(),
                incoming_notes: inc.notes.clone(),
                symbol: out.symbol.clone(),
                sent: out.amount,
                received: inc.amount,
                linked,
            };
            let mut rows = Vec::new();
            let mut suggestions = 0;
            let mut more_suggestions = false;
            for out in txs
                .iter()
                .filter(|tx| tx.mechanical_type() == "transfer_out")
            {
                if let Some(id) = out.related_tx_id.as_deref() {
                    if let Some(inc) = by_id.get(id)
                        && inc.mechanical_type() == "transfer_in"
                        && inc.related_tx_id.as_ref() == Some(&out.id)
                    {
                        rows.push(row(out, inc, true));
                    }
                    continue;
                }
                if more_suggestions {
                    continue;
                }
                let Some(date) = day(out) else {
                    continue;
                };
                'days: for offset in -WINDOW_DAYS..=WINDOW_DAYS {
                    let Ok(candidate_day) = date.checked_add(Span::new().days(offset)) else {
                        continue;
                    };
                    if let Some(matches) =
                        incoming.get(&(out.coin_id.as_str(), out.amount.to_bits(), candidate_day))
                    {
                        for inc in matches.iter().filter(|inc| eligible(out, inc)) {
                            if suggestions == MAX_SUGGESTIONS {
                                more_suggestions = true;
                                break 'days;
                            }
                            rows.push(row(out, inc, false));
                            suggestions += 1;
                        }
                    }
                }
            }
            Ok(TransferReview {
                rows,
                more_suggestions,
            })
        })
    }

    pub fn review_transfer_link(
        &self,
        outgoing_id: &str,
        incoming_id: &str,
        linked: bool,
    ) -> Result<(), CryptoError> {
        self.with_db(|db| {
            if !db.set_transfer_link(outgoing_id, incoming_id, linked, eligible)? {
                return Err(CryptoError::Validation(
                    "Transfer movements are no longer compatible".into(),
                ));
            }
            Ok(())
        })
    }
}
