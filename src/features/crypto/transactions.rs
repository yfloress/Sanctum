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

//! Crypto transaction operations
//!
//! Handles adding, updating, and deleting crypto transactions including transfers and swaps.

use crate::models::{CryptoTransaction, CryptoTransactionType};
use crate::security_log::{SecurityEvent, log_security_event};
use uuid::Uuid;

use super::commands::{
    NewCryptoSwap, NewCryptoTransaction, NewCryptoTransfer, UpdateCryptoTransaction,
};
use super::service::{CryptoError, CryptoService};
use super::validation::{
    FeeBalanceContext, MAX_NOTES_LENGTH, normalize_fee_coin, sanitize_string, validate_coin_id_str,
    validate_date, validate_fee_balance, validate_field_length, validate_non_negative,
    validate_positive_amount, validate_subtype, validate_sufficient_balance, validate_symbol,
    validate_uuid,
};

impl CryptoService {
    pub fn add_crypto_transaction(&self, cmd: NewCryptoTransaction) -> Result<String, CryptoError> {
        let NewCryptoTransaction {
            wallet_id,
            coin_id,
            symbol,
            transaction_type,
            amount,
            price_per_coin,
            fee,
            fee_coin_id,
            fee_amount,
            date,
            notes,
            subtype,
            override_proceeds,
            override_cost_basis,
        } = cmd;
        self.with_db(|db| {
            let wallet_id = wallet_id.trim().to_string();
            if wallet_id.is_empty() {
                return Err(CryptoError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
            }

            let coin_id = validate_coin_id_str(&coin_id)?;
            let symbol = validate_symbol(&symbol)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            validate_positive_amount(amount, "Amount")?;

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;
            let subtype = validate_subtype(Some(transaction_type.as_str()), subtype)?;
            let override_proceeds = validate_non_negative(override_proceeds, "Override proceeds")?;
            let override_cost_basis =
                validate_non_negative(override_cost_basis, "Override cost basis")?;

            let valid_types = ["trade", "income", "expense", "transfer"];
            if !valid_types.contains(&transaction_type.as_str()) {
                return Err(CryptoError::Validation(format!(
                    "Invalid transaction type. Must be one of: {}",
                    valid_types.join(", ")
                )));
            }

            // Build a temporary struct to derive mechanical type for validation
            let mech = crate::features::crypto::tax::types::derive_mechanical_type(
                &transaction_type,
                subtype.as_deref(),
            );

            if mech == "swap" {
                return Err(CryptoError::Validation(
                    "Swap requires paired transactions. Use the swap flow.".to_string(),
                ));
            }

            let price = if mech == "buy" || mech == "sell" {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => {
                        return Err(CryptoError::Validation(
                            "Price per coin is required and must be greater than zero".to_string(),
                        ));
                    }
                }
            } else {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => None,
                }
            };

            let is_outflow = mech == "sell" || mech == "transfer_out" || mech == "swap";

            if is_outflow {
                validate_sufficient_balance(
                    db, &wallet_id, &coin_id, &symbol, amount, &date, None,
                )?;
            }

            let fee_context = FeeBalanceContext {
                db,
                wallet_id: &wallet_id,
                main_coin_id: &coin_id,
                main_symbol: &symbol,
                main_amount: amount,
                is_outflow,
                date: &date,
                exclude_tx_id: None,
            };
            validate_fee_balance(fee_context, fee_coin_id.as_deref(), fee_amount)?;

            log_security_event(
                SecurityEvent::CryptoTransactionCreated,
                Some(&transaction_type),
            );

            let id = Uuid::new_v4().to_string();
            let mut transaction = CryptoTransaction::new(
                id.clone(),
                wallet_id,
                coin_id.to_lowercase(),
                symbol.to_uppercase(),
                transaction_type,
                amount,
                price,
                fee,
                date,
                notes,
            );
            transaction.fee_coin_id = fee_coin_id;
            transaction.fee_amount = fee_amount;
            transaction.subtype = subtype;
            transaction.override_proceeds = override_proceeds;
            transaction.override_cost_basis = override_cost_basis;

            db.create_crypto_transaction(&transaction)?;
            Ok(id)
        })
    }

    pub fn add_crypto_transfer(&self, cmd: NewCryptoTransfer) -> Result<String, CryptoError> {
        let NewCryptoTransfer {
            from_wallet_id,
            to_wallet_id,
            coin_id,
            symbol,
            from_amount,
            to_amount,
            fee,
            fee_coin_id,
            fee_amount,
            date,
            notes,
        } = cmd;
        self.with_db(|db| {
            let from_wallet_id = from_wallet_id.trim().to_string();
            let to_wallet_id = to_wallet_id.trim().to_string();
            if from_wallet_id.is_empty() || to_wallet_id.is_empty() {
                return Err(CryptoError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
            }
            if from_wallet_id == to_wallet_id {
                return Err(CryptoError::Validation(
                    "Source and destination wallets must be different".to_string(),
                ));
            }

            if db.get_wallet(&from_wallet_id)?.is_none() {
                return Err(CryptoError::Validation(
                    "Source wallet not found".to_string(),
                ));
            }
            if db.get_wallet(&to_wallet_id)?.is_none() {
                return Err(CryptoError::Validation(
                    "Destination wallet not found".to_string(),
                ));
            }

            let coin_id = validate_coin_id_str(&coin_id)?;
            let symbol = validate_symbol(&symbol)?;
            validate_positive_amount(from_amount, "From amount")?;
            validate_positive_amount(to_amount, "To amount")?;
            if to_amount > from_amount {
                return Err(CryptoError::Validation(
                    "To amount cannot exceed from amount".to_string(),
                ));
            }

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            let current_balance =
                db.get_wallet_coin_balance_at(&from_wallet_id, &coin_id, &date, None)?;
            if from_amount > current_balance {
                return Err(CryptoError::Validation(format!(
                    "Insufficient funds. Available: {:.8} {}",
                    current_balance, symbol
                )));
            }

            let fee_context = FeeBalanceContext {
                db,
                wallet_id: &from_wallet_id,
                main_coin_id: &coin_id,
                main_symbol: &symbol,
                main_amount: from_amount,
                is_outflow: true,
                date: &date,
                exclude_tx_id: None,
            };
            validate_fee_balance(fee_context, fee_coin_id.as_deref(), fee_amount)?;

            if let (Some(fee_coin), Some(_)) = (fee_coin_id.as_deref(), fee_amount)
                && fee_coin == coin_id
                && to_amount < from_amount
            {
                return Err(CryptoError::Validation(
                    "When using a same-coin network fee, keep the TO amount equal to FROM (the fee is recorded separately)".to_string(),
                ));
            }

            let (total_amount, total_cost) =
                db.get_wallet_coin_state_at(&from_wallet_id, &coin_id, &date)?;
            let avg_price = if total_amount > 0.0 {
                total_cost / total_amount
            } else {
                0.0
            };
            let transfer_price = if avg_price > 0.0 { Some(avg_price) } else { None };

            log_security_event(SecurityEvent::CryptoTransactionCreated, Some("transfer"));

            let source_id = Uuid::new_v4().to_string();
            let target_id = Uuid::new_v4().to_string();

            let source = CryptoTransaction {
                id: source_id.clone(),
                wallet_id: from_wallet_id,
                coin_id: coin_id.clone(),
                symbol: symbol.clone(),
                transaction_type: "transfer".to_string(),
                amount: from_amount,
                price_per_coin: None,
                fee: None,
                fee_coin_id: fee_coin_id.clone(),
                fee_amount,
                subtype: Some("withdrawal".to_string()),
                override_proceeds: None,
                override_cost_basis: None,
                date: date.clone(),
                notes: notes.clone(),
                related_tx_id: Some(target_id.clone()),
            };

            let target = CryptoTransaction {
                id: target_id.clone(),
                wallet_id: to_wallet_id,
                coin_id,
                symbol,
                transaction_type: "transfer".to_string(),
                amount: to_amount,
                price_per_coin: transfer_price,
                fee,
                fee_coin_id: None,
                fee_amount: None,
                subtype: Some("deposit".to_string()),
                override_proceeds: None,
                override_cost_basis: None,
                date,
                notes,
                related_tx_id: Some(source_id.clone()),
            };

            db.create_crypto_transaction(&source)?;
            if let Err(err) = db.create_crypto_transaction(&target) {
                if let Err(rollback_err) = db.delete_crypto_transaction(&source_id) {
                    log::error!(
                        "CRITICAL: Failed to rollback transfer source transaction {}: {:?}. Database may be inconsistent.",
                        source_id, rollback_err
                    );
                    return Err(CryptoError::Validation(format!(
                        "Transfer failed and rollback failed: {}. Please check transaction {}",
                        err, source_id
                    )));
                }
                return Err(CryptoError::Database(err));
            }

            Ok(source_id)
        })
    }

    pub fn add_crypto_swap(&self, cmd: NewCryptoSwap) -> Result<String, CryptoError> {
        let NewCryptoSwap {
            wallet_id,
            from_coin_id,
            from_symbol,
            from_amount,
            to_coin_id,
            to_symbol,
            to_amount,
            fee,
            fee_coin_id,
            fee_amount,
            date,
            notes,
        } = cmd;
        self.with_db(|db| {
            let wallet_id = wallet_id.trim().to_string();
            if wallet_id.is_empty() {
                return Err(CryptoError::Validation(
                    "Wallet ID cannot be empty".to_string(),
                ));
            }

            let from_coin_id = validate_coin_id_str(&from_coin_id)?;
            let to_coin_id = validate_coin_id_str(&to_coin_id)?;
            if from_coin_id == to_coin_id {
                return Err(CryptoError::Validation(
                    "Swap requires two different assets".to_string(),
                ));
            }

            let from_symbol = validate_symbol(&from_symbol)?;
            let to_symbol = validate_symbol(&to_symbol)?;
            validate_positive_amount(from_amount, "From amount")?;
            validate_positive_amount(to_amount, "To amount")?;

            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            validate_sufficient_balance(
                db,
                &wallet_id,
                &from_coin_id,
                &from_symbol,
                from_amount,
                &date,
                None,
            )?;

            if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id.as_deref(), fee_amount) {
                if fee_coin == from_coin_id {
                    let total_required = from_amount + fee_amt;
                    validate_sufficient_balance(
                        db,
                        &wallet_id,
                        &from_coin_id,
                        &from_symbol,
                        total_required,
                        &date,
                        None,
                    )?;
                } else if fee_coin == to_coin_id {
                    let to_balance = db.get_wallet_coin_balance_at(&wallet_id, fee_coin, &date, None)?;
                    if fee_amt > to_amount + to_balance {
                        return Err(CryptoError::Validation(
                            "Fee amount exceeds available output balance".to_string(),
                        ));
                    }
                } else {
                    validate_sufficient_balance(
                        db,
                        &wallet_id,
                        fee_coin,
                        fee_coin,
                        fee_amt,
                        &date,
                        None,
                    )?;
                }
            }

            log_security_event(SecurityEvent::CryptoTransactionCreated, Some("swap"));

            let first_id = Uuid::new_v4().to_string();
            let second_id = Uuid::new_v4().to_string();
            let (source_id, target_id) = if first_id <= second_id {
                (first_id, second_id)
            } else {
                (second_id, first_id)
            };

            let source = CryptoTransaction {
                id: source_id.clone(),
                wallet_id: wallet_id.clone(),
                coin_id: from_coin_id,
                symbol: from_symbol,
                transaction_type: "trade".to_string(),
                amount: from_amount,
                price_per_coin: None,
                fee,
                fee_coin_id: fee_coin_id.clone(),
                fee_amount,
                subtype: Some("swap".to_string()),
                override_proceeds: None,
                override_cost_basis: None,
                date: date.clone(),
                notes: notes.clone(),
                related_tx_id: Some(target_id.clone()),
            };

            let target = CryptoTransaction {
                id: target_id.clone(),
                wallet_id,
                coin_id: to_coin_id,
                symbol: to_symbol,
                transaction_type: "trade".to_string(),
                amount: to_amount,
                price_per_coin: None,
                fee: None,
                fee_coin_id: None,
                fee_amount: None,
                subtype: Some("swap".to_string()),
                override_proceeds: None,
                override_cost_basis: None,
                date,
                notes,
                related_tx_id: Some(source_id.clone()),
            };

            db.create_crypto_transaction(&source)?;
            if let Err(err) = db.create_crypto_transaction(&target) {
                if let Err(rollback_err) = db.delete_crypto_transaction(&source_id) {
                    log::error!(
                        "CRITICAL: Failed to rollback swap source transaction {}: {:?}. Database may be inconsistent.",
                        source_id, rollback_err
                    );
                    return Err(CryptoError::Validation(format!(
                        "Swap failed and rollback failed: {}. Please check transaction {}",
                        err, source_id
                    )));
                }
                return Err(CryptoError::Database(err));
            }

            Ok(source_id)
        })
    }

    pub fn get_wallet_transactions(
        &self,
        wallet_id: String,
    ) -> Result<Vec<CryptoTransaction>, CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&wallet_id)?;
            db.get_wallet_transactions(&validated_id)
                .map_err(CryptoError::Database)
        })
    }

    pub fn get_crypto_transaction(
        &self,
        id: String,
    ) -> Result<Option<CryptoTransaction>, CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            db.get_crypto_transaction(&validated_id)
                .map_err(CryptoError::Database)
        })
    }

    pub fn get_crypto_transactions_by_coin(
        &self,
        coin_id: String,
    ) -> Result<Vec<CryptoTransaction>, CryptoError> {
        self.with_db(|db| {
            let validated = validate_coin_id_str(&coin_id)?;
            db.get_crypto_transactions_by_coin(&validated)
                .map_err(CryptoError::Database)
        })
    }

    pub fn get_all_crypto_transactions(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<CryptoTransaction>, CryptoError> {
        self.with_db(|db| {
            db.get_all_crypto_transactions(offset, limit)
                .map_err(CryptoError::Database)
        })
    }

    pub fn update_crypto_transaction(
        &self,
        cmd: UpdateCryptoTransaction,
    ) -> Result<(), CryptoError> {
        let UpdateCryptoTransaction {
            id,
            amount,
            price_per_coin,
            fee,
            fee_coin_id,
            fee_amount,
            date,
            notes,
            subtype,
            override_proceeds,
            override_cost_basis,
        } = cmd;
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;
            let existing = db.get_crypto_transaction(&validated_id)?;
            let existing = match existing {
                Some(tx) => tx,
                None => return Err(CryptoError::Validation("Transaction not found".to_string())),
            };

            if existing.mechanical_type() == "swap" || existing.related_tx_id.is_some() {
                return Err(CryptoError::Validation(
                    "Editing paired transactions is not supported".to_string(),
                ));
            }

            validate_positive_amount(amount, "Amount")?;
            let existing_mech = existing.mechanical_type();
            let price = if existing_mech == "buy" || existing_mech == "sell" {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => {
                        return Err(CryptoError::Validation(
                            "Price per coin is required and must be greater than zero".to_string(),
                        ));
                    }
                }
            } else {
                match price_per_coin {
                    Some(p) => Some(validate_positive_amount(p, "Price per coin")?),
                    None => None,
                }
            };
            let fee = validate_non_negative(fee, "Fee")?;
            let (fee_coin_id, fee_amount) = normalize_fee_coin(fee_coin_id, fee_amount)?;
            let date = validate_date(&date)?;
            let subtype = validate_subtype(Some(existing.transaction_type.as_str()), subtype)?;
            let override_proceeds = validate_non_negative(override_proceeds, "Override proceeds")?;
            let override_cost_basis =
                validate_non_negative(override_cost_basis, "Override cost basis")?;
            let next_mech = crate::features::crypto::tax::types::derive_mechanical_type(
                &existing.transaction_type,
                subtype.as_deref(),
            );
            if next_mech != existing_mech {
                return Err(CryptoError::Validation(
                    "Changing subtype that alters transaction direction is not supported"
                        .to_string(),
                ));
            }

            let notes = match notes {
                Some(n) => {
                    let validated = validate_field_length(&n, MAX_NOTES_LENGTH, "Notes")?;
                    Some(sanitize_string(&validated))
                }
                None => None,
            };

            let is_outflow = next_mech == "sell" || next_mech == "transfer_out";

            let existing_type = existing.get_type().unwrap_or(CryptoTransactionType::Buy);

            let mut balance_excluding =
                db.get_wallet_coin_balance_at(&existing.wallet_id, &existing.coin_id, &date, None)?;
            match existing_type {
                CryptoTransactionType::Buy | CryptoTransactionType::TransferIn => {
                    balance_excluding -= existing.amount;
                }
                CryptoTransactionType::Sell
                | CryptoTransactionType::TransferOut
                | CryptoTransactionType::Swap => {
                    balance_excluding += existing.amount;
                }
            }
            if existing.fee_coin_id.as_deref() == Some(existing.coin_id.as_str())
                && let Some(fee_amt) = existing.fee_amount
            {
                balance_excluding += fee_amt;
            }

            if is_outflow && amount > balance_excluding {
                return Err(CryptoError::Validation(format!(
                    "Insufficient funds. Available: {:.8} {}",
                    balance_excluding, existing.symbol
                )));
            }

            if let (Some(fee_coin), Some(fee_amt)) = (fee_coin_id.as_deref(), fee_amount) {
                let mut fee_balance_excluding = if fee_coin == existing.coin_id {
                    balance_excluding
                } else {
                    db.get_wallet_coin_balance_at(&existing.wallet_id, fee_coin, &date, None)?
                };
                if existing.fee_coin_id.as_deref() == Some(fee_coin)
                    && let Some(existing_fee_amt) = existing.fee_amount
                {
                    fee_balance_excluding += existing_fee_amt;
                }
                if fee_coin == existing.coin_id {
                    if is_outflow {
                        let total_required = amount + fee_amt;
                        if total_required > fee_balance_excluding {
                            return Err(CryptoError::Validation(format!(
                                "Insufficient funds for fee. Available: {:.8} {}",
                                fee_balance_excluding, existing.symbol
                            )));
                        }
                    } else {
                        let total_available = fee_balance_excluding + amount;
                        if fee_amt > total_available {
                            return Err(CryptoError::Validation(
                                "Fee amount exceeds available balance".to_string(),
                            ));
                        }
                    }
                } else if fee_amt > fee_balance_excluding {
                    return Err(CryptoError::Validation(format!(
                        "Insufficient funds for fee. Available: {:.8} {}",
                        fee_balance_excluding, fee_coin
                    )));
                }
            }

            db.update_crypto_transaction_fields(
                &validated_id,
                amount,
                price,
                fee,
                fee_coin_id.as_deref(),
                fee_amount,
                &date,
                notes.as_deref(),
                subtype.as_deref(),
                override_proceeds,
                override_cost_basis,
            )?;

            Ok(())
        })
    }

    pub fn delete_crypto_transaction(&self, id: String) -> Result<(), CryptoError> {
        self.with_db(|db| {
            let validated_id = validate_uuid(&id)?;

            if let Ok(Some(tx)) = db.get_crypto_transaction(&validated_id)
                && let Some(related_id) = tx.related_tx_id
            {
                let _ = db.delete_crypto_transaction(&related_id);
            }

            db.delete_crypto_transaction(&validated_id)?;
            Ok(())
        })
    }
}
