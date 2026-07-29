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

//! Ingestion service — crypto transaction processing.

use super::{
    IngestionError, IngestionService, MexcTransferOverlapProbe,
    has_mexc_transfer_overlap_duplicate, kraken_trade_ref_key, matches_swap_rollup_duplicate,
    normalize_import_symbol_key, notbank_trade_ref_key, notbank_transaction_ref_key,
    note_is_exchange_overlap_prone, uses_price_agnostic_dedup,
};
use crate::db::Database;
use crate::features::crypto::tax::types::{derive_mechanical_type, normalize_subtype};
use crate::features::ingestion::repository::IngestionRepository;
use crate::features::ingestion::types::{
    CryptoDedupKey, ImportCryptoTransaction, ImportSummary, RowError,
};
use crate::features::ingestion::validation::validate_import_crypto_transaction;
use crate::models::CryptoTransaction;
use crate::services::i18n::{t, t_args};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

impl IngestionService {
    /// Process and insert crypto transactions (with validation and deduplication)
    pub(super) fn process_crypto_transactions(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(transactions, format_name, false, false)
    }

    /// Preview crypto transactions (validation and deduplication without inserts)
    pub(super) fn preview_crypto_transactions(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(transactions, format_name, true, false)
    }

    /// Process crypto transactions, optionally skipping balance validation.
    /// Used by exchange imports where the wallet export is authoritative.
    pub(super) fn process_crypto_transactions_ext(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        skip_balance_validation: bool,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(
            transactions,
            format_name,
            false,
            skip_balance_validation,
        )
    }

    /// Preview crypto transactions, optionally skipping balance validation.
    pub(super) fn preview_crypto_transactions_ext(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        skip_balance_validation: bool,
    ) -> Result<ImportSummary, IngestionError> {
        self.process_crypto_transactions_internal(
            transactions,
            format_name,
            true,
            skip_balance_validation,
        )
    }

    pub(super) fn process_crypto_transactions_internal(
        &self,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        dry_run: bool,
        skip_balance_validation: bool,
    ) -> Result<ImportSummary, IngestionError> {
        if dry_run {
            self.with_db_readonly(|db| {
                self.process_crypto_transactions_with_db(
                    db,
                    transactions,
                    format_name,
                    dry_run,
                    skip_balance_validation,
                )
            })
        } else {
            self.with_db(|db| {
                self.process_crypto_transactions_with_db(
                    db,
                    transactions,
                    format_name,
                    dry_run,
                    skip_balance_validation,
                )
            })
        }
    }

    pub(super) fn process_crypto_transactions_with_db(
        &self,
        db: &Database,
        transactions: Vec<(usize, ImportCryptoTransaction)>,
        format_name: &str,
        dry_run: bool,
        skip_balance_validation: bool,
    ) -> Result<ImportSummary, IngestionError> {
        // Sort transactions by date to ensure chronological processing.
        // This prevents balance validation failures when the CSV has rows
        // out of order (e.g. a withdrawal before the deposit that funds it).
        let mut transactions = transactions;
        transactions.sort_by(|a, b| a.1.date.cmp(&b.1.date));

        let mut summary = ImportSummary::new(format_name, "Crypto");
        let skipped_duplicate = t("import-skipped-duplicate-crypto");
        let skipped_missing_coin = t("import-skipped-crypto-not-found");
        let is_mexc_spot_order_history_source =
            format_name.eq_ignore_ascii_case("MEXC Spot Trade History");
        let use_price_agnostic_dedup = uses_price_agnostic_dedup(format_name);

        let wallet_lookup =
            IngestionRepository::build_wallet_lookup(db).map_err(IngestionError::Database)?;
        let coin_lookup =
            IngestionRepository::build_coin_lookup(db).map_err(IngestionError::Database)?;

        let existing = IngestionRepository::get_all_crypto_transactions(db)
            .map_err(IngestionError::Database)?;
        let existing_map: HashMap<String, &CryptoTransaction> =
            existing.iter().map(|tx| (tx.id.clone(), tx)).collect();
        let mut existing_swap_amounts: HashMap<(String, String, String, String), Vec<f64>> =
            HashMap::new();
        if is_mexc_spot_order_history_source {
            for tx in &existing {
                if tx.mechanical_type() != "swap" {
                    continue;
                }
                let pair_coin_id = tx
                    .related_tx_id
                    .as_ref()
                    .and_then(|id| existing_map.get(id))
                    .map(|related| related.coin_id.as_str());
                if let Some(pair_coin_id) = pair_coin_id {
                    existing_swap_amounts
                        .entry((
                            tx.date.clone(),
                            tx.wallet_id.clone(),
                            tx.coin_id.clone(),
                            pair_coin_id.to_string(),
                        ))
                        .or_default()
                        .push(tx.amount);
                }
            }
        }

        let mut dedup_set: HashSet<CryptoDedupKey> = existing
            .iter()
            .map(|tx| {
                let pair_coin_id = if tx.mechanical_type() == "swap" {
                    tx.related_tx_id
                        .as_ref()
                        .and_then(|id| existing_map.get(id))
                        .map(|related| related.coin_id.as_str())
                } else {
                    None
                };
                CryptoDedupKey::new(
                    &tx.date,
                    &tx.wallet_id,
                    &tx.coin_id,
                    tx.mechanical_type(),
                    &tx.transaction_type,
                    tx.subtype.as_deref(),
                    tx.amount,
                    tx.price_per_coin,
                    pair_coin_id,
                )
            })
            .collect();
        let mut dedup_set_price_agnostic: HashSet<CryptoDedupKey> = existing
            .iter()
            .filter(|tx| note_is_exchange_overlap_prone(tx.notes.as_deref()))
            .map(|tx| {
                let pair_coin_id = if tx.mechanical_type() == "swap" {
                    tx.related_tx_id
                        .as_ref()
                        .and_then(|id| existing_map.get(id))
                        .map(|related| related.coin_id.as_str())
                } else {
                    None
                };
                CryptoDedupKey::new(
                    &tx.date,
                    &tx.wallet_id,
                    &tx.coin_id,
                    tx.mechanical_type(),
                    &tx.transaction_type,
                    tx.subtype.as_deref(),
                    tx.amount,
                    None,
                    pair_coin_id,
                )
            })
            .collect();
        let mut kraken_trade_ref_set: HashSet<(String, String)> = existing
            .iter()
            .filter_map(|tx| kraken_trade_ref_key(&tx.wallet_id, tx.notes.as_deref()))
            .collect();
        let mut notbank_trade_ref_set: HashSet<(String, String)> = existing
            .iter()
            .filter_map(|tx| notbank_trade_ref_key(&tx.wallet_id, tx.notes.as_deref()))
            .collect();
        let mut notbank_transaction_ref_set: HashSet<(String, String)> = existing
            .iter()
            .filter_map(|tx| notbank_transaction_ref_key(&tx.wallet_id, tx.notes.as_deref()))
            .collect();

        // Track pending balance changes for dry_run validation
        // Key: (wallet_id, coin_id), Value: pending balance delta
        let mut pending_balance_changes: std::collections::HashMap<(String, String), f64> =
            std::collections::HashMap::new();

        for (line_num, import_tx) in transactions {
            if let Err(mut error) = validate_import_crypto_transaction(&import_tx, line_num) {
                error.raw_data = Some(format!("{:?}", import_tx));
                summary.record_error(error);
                continue;
            }

            let category_type = import_tx.transaction_type.trim().to_lowercase();
            let normalized_subtype = import_tx
                .subtype
                .as_deref()
                .and_then(|s| normalize_subtype(&category_type, s));
            let mechanical_type =
                derive_mechanical_type(&category_type, normalized_subtype.as_deref());

            // Resolve wallet
            let wallet_key = import_tx.wallet.trim().to_lowercase();
            let wallet = match wallet_lookup.get(&wallet_key) {
                Some(w) => w,
                None => {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("wallet"),
                        t_args(
                            "import-error-wallet-not-found",
                            &[("name", import_tx.wallet.trim())],
                        ),
                    ));
                    continue;
                }
            };
            let kraken_ref_key = kraken_trade_ref_key(&wallet.id, import_tx.notes.as_deref());
            if let Some(ref_key) = kraken_ref_key.as_ref()
                && kraken_trade_ref_set.contains(ref_key)
            {
                summary.record_skipped(&skipped_duplicate);
                continue;
            }
            let notbank_ref_key = notbank_trade_ref_key(&wallet.id, import_tx.notes.as_deref());
            if let Some(ref_key) = notbank_ref_key.as_ref()
                && notbank_trade_ref_set.contains(ref_key)
            {
                summary.record_skipped(&skipped_duplicate);
                continue;
            }
            let notbank_tx_ref_key =
                notbank_transaction_ref_key(&wallet.id, import_tx.notes.as_deref());
            if let Some(ref_key) = notbank_tx_ref_key.as_ref()
                && notbank_transaction_ref_set.contains(ref_key)
            {
                summary.record_skipped(&skipped_duplicate);
                continue;
            }

            // Resolve coin (source)
            let symbol_key = normalize_import_symbol_key(&import_tx.symbol);
            let coin = match coin_lookup.get(&symbol_key) {
                Some(c) => c,
                None => {
                    summary.record_skipped(&skipped_missing_coin);
                    continue;
                }
            };

            let mut swap_to_coin = None;
            if mechanical_type == "swap" {
                let to_symbol = import_tx
                    .swap_to_symbol
                    .as_ref()
                    .map(|s| s.trim())
                    .unwrap_or("");
                let to_key = normalize_import_symbol_key(to_symbol);
                swap_to_coin = match coin_lookup.get(&to_key) {
                    Some(c) => Some(c),
                    None => {
                        summary.record_skipped(&skipped_missing_coin);
                        continue;
                    }
                };
            }

            // Resolve fee coin for ALL transaction types (not just swaps)
            let mut fee_coin = None;
            if let Some(symbol) = import_tx
                .fee_coin_symbol
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            {
                let fee_key = normalize_import_symbol_key(symbol);
                fee_coin = match coin_lookup.get(&fee_key) {
                    Some(c) => Some(c),
                    None => {
                        summary.record_skipped(&skipped_missing_coin);
                        continue;
                    }
                };
            }

            // Normalize fee_coin_id/fee_amount pair (require both or neither)
            let (resolved_fee_coin_id, resolved_fee_amount) = match (
                fee_coin.as_ref().map(|c| c.id.clone()),
                import_tx.fee_amount,
            ) {
                (Some(id), Some(amount)) if amount > 0.0 => (Some(id), Some(amount)),
                (None, Some(_)) => {
                    // fee_amount without fee_coin_symbol: already caught in validation
                    (None, None)
                }
                _ => (None, None),
            };

            if has_mexc_transfer_overlap_duplicate(
                &existing,
                format_name,
                &MexcTransferOverlapProbe {
                    wallet_id: &wallet.id,
                    coin_id: &coin.id,
                    mechanical_type,
                    amount: import_tx.amount,
                    fee_amount: resolved_fee_amount,
                    date: &import_tx.date,
                },
            ) {
                summary.record_skipped(&skipped_duplicate);
                continue;
            }

            // Validate balance for outflow operations
            // Skipped for exchange imports — the wallet export is authoritative.
            if !skip_balance_validation
                && (mechanical_type == "sell" || mechanical_type == "transfer_out")
            {
                let db_balance = match IngestionRepository::get_wallet_coin_balance(
                    db,
                    &wallet.id,
                    &coin.id,
                    import_tx.date.trim(),
                ) {
                    Ok(b) => b,
                    Err(e) => {
                        summary.record_error(RowError::new(
                            line_num,
                            None,
                            format!("Database error checking balance: {}", e),
                        ));
                        continue;
                    }
                };

                // Include pending changes from previous transactions in this import batch
                let balance_key = (wallet.id.clone(), coin.id.clone());
                let pending_delta = pending_balance_changes
                    .get(&balance_key)
                    .copied()
                    .unwrap_or(0.0);
                let available_balance = db_balance + pending_delta;

                if available_balance < import_tx.amount {
                    summary.record_error(RowError::new(
                        line_num,
                        Some("amount"),
                        t_args(
                            "import-error-insufficient-crypto-balance",
                            &[
                                ("symbol", coin.symbol.as_str()),
                                ("wallet", wallet.name.as_str()),
                                ("available", &format!("{:.8}", available_balance)),
                                ("required", &format!("{:.8}", import_tx.amount)),
                            ],
                        ),
                    ));
                    continue;
                }

                // Validate fee balance for non-swap outflows with fee in same coin
                if let (Some(fee_coin_ref), Some(fee_amt)) =
                    (resolved_fee_coin_id.as_deref(), resolved_fee_amount)
                {
                    if fee_coin_ref == coin.id {
                        // Fee is in the same coin as the main outflow
                        let total_required = import_tx.amount + fee_amt;
                        if available_balance < total_required {
                            summary.record_error(RowError::new(
                                line_num,
                                Some("fee_amount"),
                                t_args(
                                    "import-error-insufficient-crypto-balance",
                                    &[
                                        ("symbol", coin.symbol.as_str()),
                                        ("wallet", wallet.name.as_str()),
                                        ("available", &format!("{:.8}", available_balance)),
                                        ("required", &format!("{:.8}", total_required)),
                                    ],
                                ),
                            ));
                            continue;
                        }
                    } else {
                        // Fee is in a different coin — check that coin's balance
                        let fee_db_balance = match IngestionRepository::get_wallet_coin_balance(
                            db,
                            &wallet.id,
                            fee_coin_ref,
                            import_tx.date.trim(),
                        ) {
                            Ok(b) => b,
                            Err(e) => {
                                summary.record_error(RowError::new(
                                    line_num,
                                    None,
                                    format!("Database error checking fee balance: {}", e),
                                ));
                                continue;
                            }
                        };
                        let fee_key = (wallet.id.clone(), fee_coin_ref.to_string());
                        let fee_pending = pending_balance_changes
                            .get(&fee_key)
                            .copied()
                            .unwrap_or(0.0);
                        let available_fee = fee_db_balance + fee_pending;
                        if fee_amt > available_fee {
                            summary.record_error(RowError::new(
                                line_num,
                                Some("fee_amount"),
                                t_args(
                                    "import-error-insufficient-crypto-balance",
                                    &[
                                        (
                                            "symbol",
                                            fee_coin
                                                .as_ref()
                                                .map(|c| c.symbol.as_str())
                                                .unwrap_or("?"),
                                        ),
                                        ("wallet", wallet.name.as_str()),
                                        ("available", &format!("{:.8}", available_fee)),
                                        ("required", &format!("{:.8}", fee_amt)),
                                    ],
                                ),
                            ));
                            continue;
                        }
                    }
                }
            } else if !skip_balance_validation
                && (mechanical_type == "buy" || mechanical_type == "transfer_in")
            {
                // For inflows, validate fee balance if fee is in a different coin
                if let (Some(fee_coin_ref), Some(fee_amt)) =
                    (resolved_fee_coin_id.as_deref(), resolved_fee_amount)
                {
                    if fee_coin_ref == coin.id {
                        // Fee is in same coin as inflow — after the buy we get `amount`,
                        // but we also need to pay `fee_amt` from that coin.
                        let db_balance = match IngestionRepository::get_wallet_coin_balance(
                            db,
                            &wallet.id,
                            &coin.id,
                            import_tx.date.trim(),
                        ) {
                            Ok(b) => b,
                            Err(e) => {
                                summary.record_error(RowError::new(
                                    line_num,
                                    None,
                                    format!("Database error checking balance: {}", e),
                                ));
                                continue;
                            }
                        };
                        let balance_key = (wallet.id.clone(), coin.id.clone());
                        let pending_delta = pending_balance_changes
                            .get(&balance_key)
                            .copied()
                            .unwrap_or(0.0);
                        let available = db_balance + pending_delta + import_tx.amount;
                        if fee_amt > available {
                            summary.record_error(RowError::new(
                                line_num,
                                Some("fee_amount"),
                                "Fee amount exceeds available balance after inflow".to_string(),
                            ));
                            continue;
                        }
                    } else {
                        let fee_db_balance = match IngestionRepository::get_wallet_coin_balance(
                            db,
                            &wallet.id,
                            fee_coin_ref,
                            import_tx.date.trim(),
                        ) {
                            Ok(b) => b,
                            Err(e) => {
                                summary.record_error(RowError::new(
                                    line_num,
                                    None,
                                    format!("Database error checking fee balance: {}", e),
                                ));
                                continue;
                            }
                        };
                        let fee_key = (wallet.id.clone(), fee_coin_ref.to_string());
                        let fee_pending = pending_balance_changes
                            .get(&fee_key)
                            .copied()
                            .unwrap_or(0.0);
                        let available_fee = fee_db_balance + fee_pending;
                        if fee_amt > available_fee {
                            summary.record_error(RowError::new(
                                line_num,
                                Some("fee_amount"),
                                t_args(
                                    "import-error-insufficient-crypto-balance",
                                    &[
                                        (
                                            "symbol",
                                            fee_coin
                                                .as_ref()
                                                .map(|c| c.symbol.as_str())
                                                .unwrap_or("?"),
                                        ),
                                        ("wallet", wallet.name.as_str()),
                                        ("available", &format!("{:.8}", available_fee)),
                                        ("required", &format!("{:.8}", fee_amt)),
                                    ],
                                ),
                            ));
                            continue;
                        }
                    }
                }
            }

            if mechanical_type == "swap" {
                let to_coin = match swap_to_coin {
                    Some(c) => c,
                    None => continue,
                };
                let to_amount = import_tx.swap_to_amount.unwrap_or(0.0);
                let fee_amount = import_tx.fee_amount.unwrap_or(0.0);

                if !skip_balance_validation {
                    let db_balance = match IngestionRepository::get_wallet_coin_balance(
                        db,
                        &wallet.id,
                        &coin.id,
                        import_tx.date.trim(),
                    ) {
                        Ok(b) => b,
                        Err(e) => {
                            summary.record_error(RowError::new(
                                line_num,
                                None,
                                format!("Database error checking balance: {}", e),
                            ));
                            continue;
                        }
                    };
                    let balance_key = (wallet.id.clone(), coin.id.clone());
                    let pending_delta = pending_balance_changes
                        .get(&balance_key)
                        .copied()
                        .unwrap_or(0.0);
                    let mut required_from = import_tx.amount;

                    if let Some(fee_coin) = fee_coin.as_ref()
                        && fee_coin.id == coin.id
                    {
                        required_from += fee_amount;
                    }

                    let available_from = db_balance + pending_delta;
                    if available_from < required_from {
                        summary.record_error(RowError::new(
                            line_num,
                            Some("amount"),
                            t_args(
                                "import-error-insufficient-crypto-balance",
                                &[
                                    ("symbol", coin.symbol.as_str()),
                                    ("wallet", wallet.name.as_str()),
                                    ("available", &format!("{:.8}", available_from)),
                                    ("required", &format!("{:.8}", required_from)),
                                ],
                            ),
                        ));
                        continue;
                    }

                    if let Some(fee_coin) = fee_coin.as_ref() {
                        if fee_coin.id == to_coin.id {
                            let to_balance = match IngestionRepository::get_wallet_coin_balance(
                                db,
                                &wallet.id,
                                &to_coin.id,
                                import_tx.date.trim(),
                            ) {
                                Ok(b) => b,
                                Err(e) => {
                                    summary.record_error(RowError::new(
                                        line_num,
                                        None,
                                        format!("Database error checking balance: {}", e),
                                    ));
                                    continue;
                                }
                            };
                            let to_key = (wallet.id.clone(), to_coin.id.clone());
                            let to_pending =
                                pending_balance_changes.get(&to_key).copied().unwrap_or(0.0);
                            let available_to = to_balance + to_pending + to_amount;
                            if fee_amount > available_to {
                                summary.record_error(RowError::new(
                                    line_num,
                                    Some("fee_amount"),
                                    "Fee amount exceeds available output balance".to_string(),
                                ));
                                continue;
                            }
                        } else if fee_coin.id != coin.id {
                            let fee_balance = match IngestionRepository::get_wallet_coin_balance(
                                db,
                                &wallet.id,
                                &fee_coin.id,
                                import_tx.date.trim(),
                            ) {
                                Ok(b) => b,
                                Err(e) => {
                                    summary.record_error(RowError::new(
                                        line_num,
                                        None,
                                        format!("Database error checking balance: {}", e),
                                    ));
                                    continue;
                                }
                            };
                            let fee_key = (wallet.id.clone(), fee_coin.id.clone());
                            let fee_pending = pending_balance_changes
                                .get(&fee_key)
                                .copied()
                                .unwrap_or(0.0);
                            let available_fee = fee_balance + fee_pending;
                            if fee_amount > available_fee {
                                summary.record_error(RowError::new(
                                    line_num,
                                    Some("fee_amount"),
                                    t_args(
                                        "import-error-insufficient-crypto-balance",
                                        &[
                                            ("symbol", fee_coin.symbol.as_str()),
                                            ("wallet", wallet.name.as_str()),
                                            ("available", &format!("{:.8}", available_fee)),
                                            ("required", &format!("{:.8}", fee_amount)),
                                        ],
                                    ),
                                ));
                                continue;
                            }
                        }
                    }
                } // end !skip_balance_validation for swap
            }

            let dedup_key = if mechanical_type == "swap" {
                let to_coin = match swap_to_coin {
                    Some(c) => c,
                    None => continue,
                };
                if is_mexc_spot_order_history_source {
                    let key = (
                        import_tx.date.clone(),
                        wallet.id.clone(),
                        coin.id.clone(),
                        to_coin.id.clone(),
                    );
                    if let Some(existing_amounts) = existing_swap_amounts.get(&key)
                        && matches_swap_rollup_duplicate(existing_amounts, import_tx.amount)
                    {
                        summary.record_skipped(&skipped_duplicate);
                        continue;
                    }
                }
                // Match the same source-side price normalisation used when persisting swaps.
                let source_price_for_dedup = import_tx.price_per_coin.or_else(|| {
                    let to_amount = import_tx.swap_to_amount.unwrap_or(0.0);
                    if import_tx.amount > 0.0 && to_amount > 0.0 {
                        Some(to_amount / import_tx.amount)
                    } else {
                        None
                    }
                });
                let from_key = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.amount,
                    source_price_for_dedup,
                    Some(&to_coin.id),
                );
                let from_key_price_agnostic = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.amount,
                    None,
                    Some(&to_coin.id),
                );
                let to_key = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &to_coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.swap_to_amount.unwrap_or(0.0),
                    None,
                    Some(&coin.id),
                );
                if dedup_set.contains(&from_key)
                    || dedup_set.contains(&to_key)
                    || (use_price_agnostic_dedup
                        && (dedup_set_price_agnostic.contains(&from_key_price_agnostic)
                            || dedup_set_price_agnostic.contains(&to_key)))
                {
                    summary.record_skipped(&skipped_duplicate);
                    continue;
                }
                // stash both keys later
                Some((from_key, to_key))
            } else {
                let key = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.amount,
                    import_tx.price_per_coin,
                    None,
                );
                let key_price_agnostic = CryptoDedupKey::new(
                    &import_tx.date,
                    &wallet.id,
                    &coin.id,
                    mechanical_type,
                    &category_type,
                    normalized_subtype.as_deref(),
                    import_tx.amount,
                    None,
                    None,
                );
                if dedup_set.contains(&key)
                    || (use_price_agnostic_dedup
                        && dedup_set_price_agnostic.contains(&key_price_agnostic))
                {
                    summary.record_skipped(&skipped_duplicate);
                    continue;
                }
                None
            };

            // Update pending balance changes for subsequent validations
            if mechanical_type == "swap" {
                let to_coin = match swap_to_coin {
                    Some(c) => c,
                    None => continue,
                };
                let to_amount = import_tx.swap_to_amount.unwrap_or(0.0);
                *pending_balance_changes
                    .entry((wallet.id.clone(), coin.id.clone()))
                    .or_insert(0.0) -= import_tx.amount;
                *pending_balance_changes
                    .entry((wallet.id.clone(), to_coin.id.clone()))
                    .or_insert(0.0) += to_amount;

                if let (Some(fee_coin), Some(fee_amount)) =
                    (fee_coin.as_ref(), import_tx.fee_amount)
                {
                    *pending_balance_changes
                        .entry((wallet.id.clone(), fee_coin.id.clone()))
                        .or_insert(0.0) -= fee_amount;
                }
            } else {
                let balance_key = (wallet.id.clone(), coin.id.clone());
                let delta = match mechanical_type {
                    "buy" | "transfer_in" => import_tx.amount,
                    "sell" | "transfer_out" => -import_tx.amount,
                    _ => 0.0,
                };
                *pending_balance_changes.entry(balance_key).or_insert(0.0) += delta;

                // Track fee-coin balance changes for non-swap types
                if let (Some(fee_coin_id), Some(fee_amt)) =
                    (resolved_fee_coin_id.as_deref(), resolved_fee_amount)
                {
                    *pending_balance_changes
                        .entry((wallet.id.clone(), fee_coin_id.to_string()))
                        .or_insert(0.0) -= fee_amt;
                }
            }

            if dry_run {
                if let Some((from_key, to_key)) = dedup_key.clone() {
                    dedup_set.insert(from_key);
                    dedup_set.insert(to_key);
                    let to_coin = match swap_to_coin {
                        Some(c) => c,
                        None => continue,
                    };
                    if use_price_agnostic_dedup {
                        let from_key_price_agnostic = CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            import_tx.amount,
                            None,
                            Some(&to_coin.id),
                        );
                        dedup_set_price_agnostic.insert(from_key_price_agnostic);
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &to_coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            import_tx.swap_to_amount.unwrap_or(0.0),
                            None,
                            Some(&coin.id),
                        ));
                    }
                    summary.record_preview_change(
                        &t("import-preview-change-crypto"),
                        format!(
                            "{:.8} {} → {:.8} {}",
                            import_tx.amount,
                            coin.symbol,
                            import_tx.swap_to_amount.unwrap_or(0.0),
                            to_coin.symbol
                        ),
                        format!("{} - {}", wallet.name, import_tx.date),
                    );
                } else {
                    let key = CryptoDedupKey::new(
                        &import_tx.date,
                        &wallet.id,
                        &coin.id,
                        mechanical_type,
                        &category_type,
                        normalized_subtype.as_deref(),
                        import_tx.amount,
                        import_tx.price_per_coin,
                        None,
                    );
                    dedup_set.insert(key);
                    if use_price_agnostic_dedup {
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            import_tx.amount,
                            None,
                            None,
                        ));
                    }
                    summary.record_preview_change(
                        &t("import-preview-change-crypto"),
                        format!(
                            "{:.8} {} ({})",
                            import_tx.amount, coin.symbol, mechanical_type
                        ),
                        format!("{} - {}", wallet.name, import_tx.date),
                    );
                }
                if let Some(ref_key) = kraken_ref_key.clone() {
                    kraken_trade_ref_set.insert(ref_key);
                }
                if let Some(ref_key) = notbank_ref_key.clone() {
                    notbank_trade_ref_set.insert(ref_key);
                }
                if let Some(ref_key) = notbank_tx_ref_key.clone() {
                    notbank_transaction_ref_set.insert(ref_key);
                }
                continue;
            }

            if mechanical_type == "swap" {
                let to_coin = match swap_to_coin {
                    Some(c) => c,
                    None => continue,
                };
                let to_amount = import_tx.swap_to_amount.unwrap_or(0.0);
                let first_id = Uuid::new_v4().to_string();
                let second_id = Uuid::new_v4().to_string();
                let (source_id, target_id) = if first_id <= second_id {
                    (first_id, second_id)
                } else {
                    (second_id, first_id)
                };

                // Ensure source side always has price_per_coin so the portfolio
                // direction resolver can distinguish source from target (score +1).
                // If the parser didn't provide a price, derive it from the
                // exchange rate: to_amount / from_amount.
                let swap_price = import_tx.price_per_coin.or_else(|| {
                    if import_tx.amount > 0.0 && to_amount > 0.0 {
                        Some(to_amount / import_tx.amount)
                    } else {
                        None
                    }
                });

                let source = CryptoTransaction {
                    id: source_id.clone(),
                    wallet_id: wallet.id.clone(),
                    coin_id: coin.id.clone(),
                    symbol: coin.symbol.clone(),
                    transaction_type: category_type.clone(),
                    amount: import_tx.amount,
                    price_per_coin: swap_price,
                    fee: import_tx.fee,
                    fee_coin_id: resolved_fee_coin_id.clone(),
                    fee_amount: resolved_fee_amount,
                    subtype: normalized_subtype.clone(),
                    override_proceeds: import_tx.override_proceeds,
                    override_cost_basis: None,
                    date: import_tx.date.trim().to_string(),
                    notes: import_tx.notes.clone(),
                    related_tx_id: Some(target_id.clone()),
                };

                let target = CryptoTransaction {
                    id: target_id.clone(),
                    wallet_id: wallet.id.clone(),
                    coin_id: to_coin.id.clone(),
                    symbol: to_coin.symbol.clone(),
                    transaction_type: category_type.clone(),
                    amount: to_amount,
                    price_per_coin: None,
                    fee: None,
                    fee_coin_id: None,
                    fee_amount: None,
                    subtype: normalized_subtype.clone(),
                    override_proceeds: None,
                    override_cost_basis: import_tx.override_cost_basis,
                    date: import_tx.date.trim().to_string(),
                    notes: import_tx.notes.clone(),
                    related_tx_id: Some(source_id.clone()),
                };

                if let Err(e) = IngestionRepository::create_crypto_transaction(db, &source) {
                    summary.record_error(RowError::new(
                        line_num,
                        None,
                        format!("Database error: {}", e),
                    ));
                    continue;
                }

                if let Err(e) = IngestionRepository::create_crypto_transaction(db, &target) {
                    let _ = db.delete_crypto_transaction(&source_id);
                    summary.record_error(RowError::new(
                        line_num,
                        None,
                        format!("Database error: {}", e),
                    ));
                    continue;
                }

                if let Some((from_key, to_key)) = dedup_key {
                    dedup_set.insert(from_key);
                    dedup_set.insert(to_key);
                    if use_price_agnostic_dedup {
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            import_tx.amount,
                            None,
                            Some(&to_coin.id),
                        ));
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &to_coin.id,
                            mechanical_type,
                            &category_type,
                            normalized_subtype.as_deref(),
                            to_amount,
                            None,
                            Some(&coin.id),
                        ));
                    }
                }
                if let Some(ref_key) = kraken_ref_key.clone() {
                    kraken_trade_ref_set.insert(ref_key);
                }
                if let Some(ref_key) = notbank_ref_key.clone() {
                    notbank_trade_ref_set.insert(ref_key);
                }
                if let Some(ref_key) = notbank_tx_ref_key.clone() {
                    notbank_transaction_ref_set.insert(ref_key);
                }
                summary.record_inserted();
                continue;
            }

            let mut transaction = CryptoTransaction::new(
                Uuid::new_v4().to_string(),
                wallet.id.clone(),
                coin.id.clone(),
                coin.symbol.clone(),
                category_type.clone(),
                import_tx.amount,
                import_tx.price_per_coin,
                import_tx.fee,
                import_tx.date.trim().to_string(),
                import_tx.notes.clone(),
            );
            transaction.fee_coin_id = resolved_fee_coin_id.clone();
            transaction.fee_amount = resolved_fee_amount;
            transaction.subtype = normalized_subtype.clone();
            transaction.override_proceeds = import_tx.override_proceeds;
            transaction.override_cost_basis = import_tx.override_cost_basis;

            match IngestionRepository::create_crypto_transaction(db, &transaction) {
                Ok(_) => {
                    let key = CryptoDedupKey::new(
                        &import_tx.date,
                        &wallet.id,
                        &coin.id,
                        transaction.mechanical_type(),
                        &transaction.transaction_type,
                        transaction.subtype.as_deref(),
                        import_tx.amount,
                        transaction.price_per_coin,
                        None,
                    );
                    dedup_set.insert(key);
                    if use_price_agnostic_dedup {
                        dedup_set_price_agnostic.insert(CryptoDedupKey::new(
                            &import_tx.date,
                            &wallet.id,
                            &coin.id,
                            transaction.mechanical_type(),
                            &transaction.transaction_type,
                            transaction.subtype.as_deref(),
                            import_tx.amount,
                            None,
                            None,
                        ));
                    }
                    if let Some(ref_key) = kraken_ref_key {
                        kraken_trade_ref_set.insert(ref_key);
                    }
                    if let Some(ref_key) = notbank_ref_key {
                        notbank_trade_ref_set.insert(ref_key);
                    }
                    if let Some(ref_key) = notbank_tx_ref_key {
                        notbank_transaction_ref_set.insert(ref_key);
                    }
                    summary.record_inserted();
                }
                Err(e) => {
                    summary.record_error(RowError::new(
                        line_num,
                        None,
                        format!("Database error: {}", e),
                    ));
                }
            }
        }

        Ok(summary)
    }
}
