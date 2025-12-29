//! Crypto transaction callbacks (add, edit, delete)

use super::helpers::{
    reload_portfolio, reload_wallets, SETTING_CRYPTO_LAST_COIN_ID, SETTING_CRYPTO_LAST_WALLET_ID,
};
use crate::controller::AppController;
use crate::ui::format_crypto_amount;
use crate::{CryptoAdapter, AppWindow};
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;

/// Sets up transaction-related callbacks
pub fn setup_transaction_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + Send + 'static,
{
    // on_add_transaction
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_add_transaction(
            move |wallet_id_raw,
                  coin_id,
                  symbol,
                  type_str,
                  amount_str,
                  price_str,
                  fee_str,
                  fee_coin_id_str,
                  fee_coin_amount_str,
                  date,
                  notes_str|
                  -> SharedString {
                let amount_clean = amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let amount: f64 = match amount_clean.parse() {
                    Ok(v) => v,
                    Err(_) => return SharedString::from("Invalid amount format"),
                };

                let price_clean = price_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let price_per_coin: Option<f64> = if price_clean.is_empty() {
                    None
                } else {
                    match price_clean.parse() {
                        Ok(v) => Some(v),
                        Err(_) => return SharedString::from("Invalid price format"),
                    }
                };

                let fee_clean = fee_str.replace(",", "").replace("$", "").trim().to_string();
                let fee: Option<f64> = if fee_clean.is_empty() {
                    None
                } else {
                    match fee_clean.parse() {
                        Ok(v) => Some(v),
                        Err(_) => return SharedString::from("Invalid fee format"),
                    }
                };

                let fee_coin_amount_clean = fee_coin_amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let fee_coin_amount: Option<f64> = if fee_coin_amount_clean.is_empty() {
                    None
                } else {
                    match fee_coin_amount_clean.parse() {
                        Ok(v) if v > 0.0 => Some(v),
                        Ok(0.0) => None,
                        Ok(_) => return SharedString::from("Fee amount cannot be negative"),
                        Err(_) => return SharedString::from("Invalid fee amount format"),
                    }
                };
                let fee_coin_id = if fee_coin_id_str.trim().is_empty() {
                    None
                } else {
                    Some(fee_coin_id_str.to_string())
                };

                let notes = if notes_str.is_empty() {
                    None
                } else {
                    Some(notes_str.to_string())
                };

                let result = controller.add_crypto_transaction(
                    wallet_id_raw.to_string(),
                    coin_id.to_string(),
                    symbol.to_string(),
                    type_str.to_string(),
                    amount,
                    price_per_coin,
                    fee,
                    fee_coin_id,
                    fee_coin_amount,
                    date.to_string(),
                    notes,
                );

                match result {
                    Ok(_) => {
                        let _ = controller
                            .set_app_setting(SETTING_CRYPTO_LAST_WALLET_ID, wallet_id_raw.as_ref());
                        let _ = controller
                            .set_app_setting(SETTING_CRYPTO_LAST_COIN_ID, coin_id.as_ref());
                        reload_portfolio(&ui_weak, &controller);
                        reload_wallets(&ui_weak, &controller);
                        notify("Asset added successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_add_transfer
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_add_transfer(
            move |from_wallet_id,
                  to_wallet_id,
                  coin_id,
                  symbol,
                  from_amount_str,
                  to_amount_str,
                  fee_str,
                  fee_coin_id_str,
                  fee_coin_amount_str,
                  date,
                  notes_str|
                  -> SharedString {
                let parse_amount = |raw: SharedString, label: &str| -> Result<f64, SharedString> {
                    let cleaned = raw.replace(",", "").replace("$", "").trim().to_string();
                    cleaned
                        .parse()
                        .map_err(|_| SharedString::from(format!("Invalid {} format", label)))
                };

                let from_amount = match parse_amount(from_amount_str, "from amount") {
                    Ok(v) => v,
                    Err(e) => return e,
                };

                let to_amount = if to_amount_str.trim().is_empty() {
                    from_amount
                } else {
                    match parse_amount(to_amount_str, "to amount") {
                        Ok(v) => v,
                        Err(e) => return e,
                    }
                };

                let fee_clean = fee_str.replace(",", "").replace("$", "").trim().to_string();
                let fee: Option<f64> = if fee_clean.is_empty() {
                    None
                } else {
                    match fee_clean.parse() {
                        Ok(v) => Some(v),
                        Err(_) => return SharedString::from("Invalid fee format"),
                    }
                };

                let fee_coin_amount_clean = fee_coin_amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let fee_coin_amount: Option<f64> = if fee_coin_amount_clean.is_empty() {
                    None
                } else {
                    match fee_coin_amount_clean.parse() {
                        Ok(v) if v > 0.0 => Some(v),
                        Ok(0.0) => None,
                        Ok(_) => return SharedString::from("Fee amount cannot be negative"),
                        Err(_) => return SharedString::from("Invalid fee amount format"),
                    }
                };
                let fee_coin_id = if fee_coin_id_str.trim().is_empty() {
                    None
                } else {
                    Some(fee_coin_id_str.to_string())
                };

                let notes = if notes_str.is_empty() {
                    None
                } else {
                    Some(notes_str.to_string())
                };

                let result = controller.add_crypto_transfer(
                    from_wallet_id.to_string(),
                    to_wallet_id.to_string(),
                    coin_id.to_string(),
                    symbol.to_string(),
                    from_amount,
                    to_amount,
                    fee,
                    fee_coin_id,
                    fee_coin_amount,
                    date.to_string(),
                    notes,
                );

                match result {
                    Ok(_) => {
                        let _ = controller.set_app_setting(
                            SETTING_CRYPTO_LAST_WALLET_ID,
                            from_wallet_id.as_ref(),
                        );
                        let _ = controller
                            .set_app_setting(SETTING_CRYPTO_LAST_COIN_ID, coin_id.as_ref());
                        reload_portfolio(&ui_weak, &controller);
                        reload_wallets(&ui_weak, &controller);
                        notify("Transfer added successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_add_swap
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_add_swap(
            move |wallet_id_raw,
                  from_coin_id,
                  from_symbol,
                  from_amount_str,
                  to_coin_id,
                  to_symbol,
                  to_amount_str,
                  fee_str,
                  fee_coin_id_str,
                  fee_coin_amount_str,
                  date,
                  notes_str|
                  -> SharedString {
                let parse_amount = |raw: SharedString, label: &str| -> Result<f64, SharedString> {
                    let cleaned = raw.replace(",", "").replace("$", "").trim().to_string();
                    cleaned
                        .parse()
                        .map_err(|_| SharedString::from(format!("Invalid {} format", label)))
                };

                let from_amount = match parse_amount(from_amount_str, "from amount") {
                    Ok(v) => v,
                    Err(e) => return e,
                };

                let to_amount = match parse_amount(to_amount_str, "to amount") {
                    Ok(v) => v,
                    Err(e) => return e,
                };

                let fee_clean = fee_str.replace(",", "").replace("$", "").trim().to_string();
                let fee: Option<f64> = if fee_clean.is_empty() {
                    None
                } else {
                    match fee_clean.parse() {
                        Ok(v) => Some(v),
                        Err(_) => return SharedString::from("Invalid fee format"),
                    }
                };

                let fee_coin_amount_clean = fee_coin_amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let fee_coin_amount: Option<f64> = if fee_coin_amount_clean.is_empty() {
                    None
                } else {
                    match fee_coin_amount_clean.parse() {
                        Ok(v) if v > 0.0 => Some(v),
                        Ok(0.0) => None,
                        Ok(_) => return SharedString::from("Fee amount cannot be negative"),
                        Err(_) => return SharedString::from("Invalid fee amount format"),
                    }
                };
                let fee_coin_id = if fee_coin_id_str.trim().is_empty() {
                    None
                } else {
                    Some(fee_coin_id_str.to_string())
                };

                let notes = if notes_str.is_empty() {
                    None
                } else {
                    Some(notes_str.to_string())
                };

                let result = controller.add_crypto_swap(
                    wallet_id_raw.to_string(),
                    from_coin_id.to_string(),
                    from_symbol.to_string(),
                    from_amount,
                    to_coin_id.to_string(),
                    to_symbol.to_string(),
                    to_amount,
                    fee,
                    fee_coin_id,
                    fee_coin_amount,
                    date.to_string(),
                    notes,
                );

                match result {
                    Ok(_) => {
                        let _ = controller
                            .set_app_setting(SETTING_CRYPTO_LAST_WALLET_ID, wallet_id_raw.as_ref());
                        let _ = controller
                            .set_app_setting(SETTING_CRYPTO_LAST_COIN_ID, from_coin_id.as_ref());
                        reload_portfolio(&ui_weak, &controller);
                        reload_wallets(&ui_weak, &controller);
                        notify("Swap added successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_load_edit_transaction
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>()
            .on_load_edit_transaction(move |id| -> SharedString {
                let tx = match controller.get_crypto_transaction(id.to_string()) {
                    Ok(Some(t)) => t,
                    Ok(None) => return SharedString::from("Transaction not found"),
                    Err(e) => return SharedString::from(e.to_string()),
                };

                if tx.transaction_type == "swap" || tx.related_tx_id.is_some() {
                    return SharedString::from("Editing paired transactions is not supported");
                }

                let wallet_name = controller
                    .get_wallets()
                    .ok()
                    .and_then(|wallets| {
                        wallets
                            .into_iter()
                            .find(|w| w.id == tx.wallet_id)
                            .map(|w| w.name)
                    })
                    .unwrap_or_else(|| "Wallet".to_string());

                let price_str = tx
                    .price_per_coin
                    .map(|p| format!("{:.4}", p))
                    .unwrap_or_default();
                let fee_str = tx.fee.map(|f| format!("{:.4}", f)).unwrap_or_default();
                let fee_coin_id = tx.fee_coin_id.clone().unwrap_or_default();
                let fee_coin_amount = tx.fee_amount.map(format_crypto_amount).unwrap_or_default();
                let amount_str = format!("{:.4}", tx.amount);
                let notes_str = tx.notes.unwrap_or_default();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.global::<CryptoAdapter>()
                        .set_edit_tx_id(SharedString::from(&tx.id));
                    ui.global::<CryptoAdapter>()
                        .set_edit_wallet_id(SharedString::from(&tx.wallet_id));
                    ui.global::<CryptoAdapter>()
                        .set_edit_wallet_name(SharedString::from(wallet_name));
                    ui.global::<CryptoAdapter>()
                        .set_edit_coin_id(SharedString::from(&tx.coin_id));
                    ui.global::<CryptoAdapter>()
                        .set_edit_symbol(SharedString::from(&tx.symbol));
                    ui.global::<CryptoAdapter>()
                        .set_edit_type(SharedString::from(tx.transaction_type.to_uppercase()));
                    ui.global::<CryptoAdapter>()
                        .set_edit_amount(SharedString::from(amount_str));
                    ui.global::<CryptoAdapter>()
                        .set_edit_price(SharedString::from(price_str));
                    ui.global::<CryptoAdapter>()
                        .set_edit_fee(SharedString::from(fee_str));
                    ui.global::<CryptoAdapter>()
                        .set_edit_fee_coin_id(SharedString::from(fee_coin_id));
                    ui.global::<CryptoAdapter>()
                        .set_edit_fee_coin_amount(SharedString::from(fee_coin_amount));
                    ui.global::<CryptoAdapter>()
                        .set_edit_date(SharedString::from(&tx.date));
                    ui.global::<CryptoAdapter>()
                        .set_edit_notes(SharedString::from(notes_str));
                }

                SharedString::from("")
            });
    }

    // on_update_transaction
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>().on_update_transaction(
            move |id,
                  amount_str,
                  price_str,
                  fee_str,
                  fee_coin_id_str,
                  fee_coin_amount_str,
                  date,
                  notes_str| {
                let amount_clean = amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let amount: f64 = match amount_clean.parse() {
                    Ok(v) => v,
                    Err(_) => return SharedString::from("Invalid amount format"),
                };

                let price_clean = price_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let price_per_coin: Option<f64> = if price_clean.is_empty() {
                    None
                } else {
                    match price_clean.parse() {
                        Ok(v) => Some(v),
                        Err(_) => return SharedString::from("Invalid price format"),
                    }
                };

                let fee_clean = fee_str.replace(",", "").replace("$", "").trim().to_string();
                let fee: Option<f64> = if fee_clean.is_empty() {
                    None
                } else {
                    match fee_clean.parse() {
                        Ok(v) => Some(v),
                        Err(_) => return SharedString::from("Invalid fee format"),
                    }
                };

                let fee_coin_amount_clean = fee_coin_amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let fee_coin_amount: Option<f64> = if fee_coin_amount_clean.is_empty() {
                    None
                } else {
                    match fee_coin_amount_clean.parse() {
                        Ok(v) if v > 0.0 => Some(v),
                        Ok(0.0) => None,
                        Ok(_) => return SharedString::from("Fee amount cannot be negative"),
                        Err(_) => return SharedString::from("Invalid fee amount format"),
                    }
                };
                let fee_coin_id = if fee_coin_id_str.trim().is_empty() {
                    None
                } else {
                    Some(fee_coin_id_str.to_string())
                };

                let notes = if notes_str.is_empty() {
                    None
                } else {
                    Some(notes_str.to_string())
                };

                match controller.update_crypto_transaction(
                    id.to_string(),
                    amount,
                    price_per_coin,
                    fee,
                    fee_coin_id,
                    fee_coin_amount,
                    date.to_string(),
                    notes,
                ) {
                    Ok(_) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<CryptoAdapter>().invoke_fetch_portfolio();
                            ui.global::<CryptoAdapter>().invoke_fetch_wallets();
                        }
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_delete_crypto_transaction
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<CryptoAdapter>()
            .on_delete_crypto_transaction(move |id| -> SharedString {
                match controller.delete_crypto_transaction(id.to_string()) {
                    Ok(_) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            let coin_id = ui.global::<CryptoAdapter>().get_selected_asset().id;
                            ui.global::<CryptoAdapter>()
                                .invoke_fetch_asset_details(coin_id);
                            ui.global::<CryptoAdapter>().invoke_fetch_portfolio();
                            ui.global::<CryptoAdapter>().invoke_fetch_wallets();
                        }
                        notify("Transaction deleted".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }
}
