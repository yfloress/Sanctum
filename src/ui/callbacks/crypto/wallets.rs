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

//! Wallet CRUD callbacks

use super::helpers::reload_wallets;
use crate::controller::AppController;
use crate::models::{CryptoAsset, CryptoTransaction};
use crate::ui::{
    crypto_icon_for_symbol, format_crypto_tx_display, format_fee_display, format_money, format_usd,
    load_wallet_icon,
};
use crate::{AssetTransaction, CryptoAdapter, CryptoAssetData, AppWindow};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::Arc;

/// Sets up wallet-related callbacks
pub fn setup_wallet_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + Send + 'static,
{
    // on_fetch_wallets
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>().on_fetch_wallets(move || {
            reload_wallets::<fn(String, bool)>(&ui_weak, &controller, None);
        });
    }

    // on_fetch_wallet_details
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>()
            .on_fetch_wallet_details(move |wallet_id| {
                let wallet_id_str = wallet_id.to_string();
                let wallets = controller.get_wallets().unwrap_or_default();
                let wallet = wallets.iter().find(|w| w.id == wallet_id_str);

                if let Some(w) = wallet {
                    let mut holdings = controller
                        .get_wallet_holdings(wallet_id_str.clone())
                        .unwrap_or_default();

                    let prices = controller.load_crypto_prices().unwrap_or_default();
                    let price_map: HashMap<String, CryptoAsset> =
                        prices.into_iter().map(|p| (p.id.clone(), p)).collect();

                    let mut total_value = 0.0;
                    let holdings_data: Vec<CryptoAssetData> = holdings
                        .iter_mut()
                        .map(|asset| {
                            if let Some(price_data) = price_map.get(&asset.coin_id) {
                                asset.update_with_price(price_data.current_price);
                            }

                            total_value += asset.current_value;

                            let price_data = price_map.get(&asset.coin_id);
                            let asset_name = price_data
                                .map(|p| p.name.clone())
                                .unwrap_or_else(|| asset.symbol.clone());

                            let price_fmt = if price_data.is_none() {
                                "N/A".to_string()
                            } else if asset.current_price < 1.0 {
                                format!("$ {:.4}", asset.current_price)
                            } else {
                                format_usd(asset.current_price)
                            };

                            let value_fmt = if price_data.is_none() {
                                "N/A".to_string()
                            } else {
                                format_usd(asset.current_value)
                            };

                            CryptoAssetData {
                                id: SharedString::from(&asset.coin_id),
                                symbol: SharedString::from(&asset.symbol),
                                icon: crypto_icon_for_symbol(&asset.symbol),
                                name: SharedString::from(asset_name),
                                price: SharedString::from(price_fmt),
                                amount: SharedString::from(format!(
                                    "{:.4} {}",
                                    asset.total_amount, asset.symbol
                                )),
                                value: SharedString::from(value_fmt),
                                change_24h: SharedString::from(""),
                                is_positive: true,
                                allocation: 0.0,
                            }
                        })
                        .collect();

                    let history = controller
                        .get_wallet_transactions(wallet_id_str.clone())
                        .unwrap_or_default();
                    let symbol_map: HashMap<String, String> = controller
                        .get_coin_catalog_or_default()
                        .into_iter()
                        .map(|coin| (coin.id, coin.symbol))
                        .collect();
                    let history_map: HashMap<String, CryptoTransaction> = history
                        .iter()
                        .cloned()
                        .map(|tx| (tx.id.clone(), tx))
                        .collect();
                    let history_mapped: Vec<AssetTransaction> = history
                        .iter()
                        .map(|tx| {
                            let related =
                                tx.related_tx_id.as_ref().and_then(|id| history_map.get(id));
                            let (label, amount_display, price_display, is_swap) =
                                format_crypto_tx_display(tx, related);
                            let fee_fmt = format_fee_display(tx, &symbol_map);
                            let notes = tx.notes.clone().unwrap_or_default();

                            AssetTransaction {
                                id: SharedString::from(&tx.id),
                                date: SharedString::from(&tx.date),
                                r#type: SharedString::from(label),
                                amount: SharedString::from(amount_display),
                                price: SharedString::from(price_display),
                                fee: SharedString::from(fee_fmt),
                                notes: SharedString::from(notes),
                                is_swap,
                            }
                        })
                        .collect();

                    if let Some(ui) = ui_weak.upgrade() {
                        let adapter = ui.global::<CryptoAdapter>();
                        let category_label = match w.category.as_str() {
                            "exchange" => "Exchange",
                            "wallet_multi" => "Hardware Wallet",
                            _ => "Software Wallet",
                        };
                        adapter.set_selected_wallet_id(SharedString::from(&w.id));
                        adapter.set_selected_wallet_name(SharedString::from(&w.name));
                        adapter.set_selected_wallet_category(SharedString::from(category_label));
                        adapter.set_selected_wallet_category_key(SharedString::from(&w.category));
                        adapter.set_selected_wallet_icon(load_wallet_icon(w.icon.clone(), &w.category));
                        adapter.set_selected_wallet_icon_path(SharedString::from(
                            w.icon.clone().unwrap_or_default(),
                        ));
                        adapter.set_selected_wallet_balance(SharedString::from(format_money(
                            (total_value * 100.0) as i64,
                            "USD",
                        )));
                        adapter.set_wallet_holdings(ModelRc::new(VecModel::from(holdings_data)));
                        adapter.set_wallet_history(ModelRc::new(VecModel::from(history_mapped)));
                    }
                }
            });
    }

    // on_create_wallet
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<CryptoAdapter>()
            .on_create_wallet(move |name, category| -> SharedString {
                let category_value = category.to_string();
                match controller.add_wallet(name.to_string(), category_value.clone(), None) {
                    Ok(id) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            let adapter = ui.global::<CryptoAdapter>();
                            adapter.set_icon_edit_wallet_id(SharedString::from(&id));
                            adapter.set_icon_edit_wallet_category(SharedString::from(&category_value));
                            adapter.set_icon_edit_wallet_icon(SharedString::from(""));
                        }
                        reload_wallets(&ui_weak, &controller, Some(&notify));
                        notify("Wallet created successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_create_wallet_with_icon
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<CryptoAdapter>()
            .on_create_wallet_with_icon(move |name, category, icon| -> SharedString {
                let icon_path = if icon.is_empty() {
                    None
                } else {
                    Some(icon.to_string())
                };
                match controller.add_wallet(name.to_string(), category.to_string(), icon_path) {
                    Ok(_) => {
                        reload_wallets(&ui_weak, &controller, Some(&notify));
                        notify("Wallet created successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_delete_wallet
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<CryptoAdapter>()
            .on_delete_wallet(move |id| -> SharedString {
                match controller.delete_wallet(id.to_string()) {
                    Ok(_) => {
                        reload_wallets(&ui_weak, &controller, Some(&notify));
                        notify("Wallet deleted".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_update_wallet_name
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<CryptoAdapter>()
            .on_update_wallet_name(move |id, new_name| -> SharedString {
                match controller.update_wallet_name(id.to_string(), new_name.to_string()) {
                    Ok(_) => {
                        reload_wallets(&ui_weak, &controller, Some(&notify));
                        if let Some(ui) = ui_weak.upgrade()
                            && ui.global::<CryptoAdapter>().get_show_wallet_detail()
                        {
                            ui.global::<CryptoAdapter>().invoke_fetch_wallet_details(id);
                        }
                        notify("Wallet renamed successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_update_wallet_icon
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<CryptoAdapter>()
            .on_update_wallet_icon(move |id, icon| -> SharedString {
                let icon_path = if icon.is_empty() {
                    None
                } else {
                    Some(icon.to_string())
                };
                match controller.update_wallet_icon(id.to_string(), icon_path) {
                    Ok(_) => {
                        reload_wallets(&ui_weak, &controller, Some(&notify));
                        if let Some(ui) = ui_weak.upgrade()
                            && ui.global::<CryptoAdapter>().get_show_wallet_detail()
                        {
                            ui.global::<CryptoAdapter>().invoke_fetch_wallet_details(id);
                        }
                        notify("Wallet icon updated".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }
}
