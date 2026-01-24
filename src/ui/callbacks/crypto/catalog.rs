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

//! Coin catalog and ticker configuration callbacks

use super::helpers::{reload_portfolio, SETTING_CRYPTO_LAST_COIN_ID};
use crate::controller::AppController;
use crate::{AppWindow, CatalogCoin, CryptoAdapter, TickerOption};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashSet;
use std::sync::Arc;

/// Sets up catalog and ticker-related callbacks
pub fn setup_catalog_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + Send + 'static,
{
    // on_load_ticker_options
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>()
            .on_load_ticker_options(move || {
                let active_ids = controller.get_active_ticker_ids();
                let catalog = controller.get_coin_catalog_or_default();

                let options: Vec<TickerOption> = catalog
                    .into_iter()
                    .map(|coin| TickerOption {
                        id: SharedString::from(coin.id.clone()),
                        name: SharedString::from(coin.name),
                        symbol: SharedString::from(coin.symbol),
                        enabled: active_ids.contains(&coin.id),
                        custom: coin.custom,
                        visible: true,
                    })
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.global::<CryptoAdapter>()
                        .set_ticker_options(ModelRc::new(VecModel::from(options)));
                }
            });
    }

    // on_load_coin_catalog
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>().on_load_coin_catalog(move || {
            let catalog = controller.get_coin_catalog_or_default();
            let last_coin_id = controller
                .get_app_setting(SETTING_CRYPTO_LAST_COIN_ID)
                .ok()
                .filter(|val| !val.is_empty());
            let last_coin_index = last_coin_id
                .as_ref()
                .and_then(|id| catalog.iter().position(|coin| coin.id == *id))
                .unwrap_or(0) as i32;
            let favorites: HashSet<String> =
                controller.get_favorite_coin_ids().into_iter().collect();

            let options: Vec<CatalogCoin> = catalog
                .into_iter()
                .map(|coin| {
                    let is_favorite = favorites.contains(&coin.id);
                    CatalogCoin {
                        id: SharedString::from(coin.id),
                        name: SharedString::from(coin.name),
                        symbol: SharedString::from(coin.symbol),
                        custom: coin.custom,
                        favorite: is_favorite,
                        visible: true,
                        selected: false,
                    }
                })
                .collect();

            if let Some(ui) = ui_weak.upgrade() {
                ui.global::<CryptoAdapter>()
                    .set_coin_catalog(ModelRc::new(VecModel::from(options)));
                ui.global::<CryptoAdapter>()
                    .set_default_coin_index(last_coin_index);
            }
        });
    }

    // on_save_ticker_options
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>()
            .on_save_ticker_options(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let options = ui.global::<CryptoAdapter>().get_ticker_options();
                    let mut new_active_ids: Vec<String> = Vec::new();

                    for opt in options.iter() {
                        if opt.enabled {
                            new_active_ids.push(opt.id.to_string());
                        }
                    }

                    if let Err(e) = controller.save_active_ticker_ids(new_active_ids) {
                        notify(format!("Failed to save: {}", e), true);
                        return;
                    }

                    reload_portfolio(&ui_weak, &controller, Some(&notify));
                    notify("Configuration saved".into(), false);
                }
            });
    }

    // on_add_custom_coin
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>()
            .on_add_custom_coin(move |id, name, symbol| -> SharedString {
                match controller.add_custom_coin(
                    id.to_string(),
                    name.to_string(),
                    symbol.to_string(),
                ) {
                    Ok(_) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<CryptoAdapter>().invoke_load_coin_catalog();
                            ui.global::<CryptoAdapter>().invoke_load_ticker_options();
                        }
                        notify("Coin added".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_set_favorite_coin
    {
        let controller = controller.clone();

        ui.global::<CryptoAdapter>()
            .on_set_favorite_coin(move |id, favorite| -> SharedString {
                match controller.set_favorite_coin(id.to_string(), favorite) {
                    Ok(_) => SharedString::from(""),
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_delete_custom_coin
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>()
            .on_delete_custom_coin(move |id| -> SharedString {
                match controller.delete_custom_coin(id.to_string()) {
                    Ok(_) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<CryptoAdapter>().invoke_load_coin_catalog();
                            ui.global::<CryptoAdapter>().invoke_load_ticker_options();
                        }
                        notify("Coin removed".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_filter_ticker_options
    {
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>()
            .on_filter_ticker_options(move |query| {
                if let Some(ui) = ui_weak.upgrade() {
                    let options = ui.global::<CryptoAdapter>().get_ticker_options();
                    let mut options: Vec<TickerOption> = options.iter().collect();
                    let query = query.to_lowercase();

                    for opt in options.iter_mut() {
                        let haystack = format!(
                            "{} {} {}",
                            opt.id.to_lowercase(),
                            opt.name.to_lowercase(),
                            opt.symbol.to_lowercase()
                        );
                        opt.visible = query.is_empty() || haystack.contains(&query);
                    }

                    ui.global::<CryptoAdapter>()
                        .set_ticker_options(ModelRc::new(VecModel::from(options)));
                }
            });
    }

    // on_filter_coin_catalog
    {
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>()
            .on_filter_coin_catalog(move |query| {
                if let Some(ui) = ui_weak.upgrade() {
                    let options = ui.global::<CryptoAdapter>().get_coin_catalog();
                    let mut options: Vec<CatalogCoin> = options.iter().collect();
                    let query = query.to_lowercase();

                    for opt in options.iter_mut() {
                        let haystack = format!(
                            "{} {} {}",
                            opt.id.to_lowercase(),
                            opt.name.to_lowercase(),
                            opt.symbol.to_lowercase()
                        );
                        opt.visible = query.is_empty() || haystack.contains(&query);
                    }

                    ui.global::<CryptoAdapter>()
                        .set_coin_catalog(ModelRc::new(VecModel::from(options)));
                }
            });
    }

    // on_select_all_coins
    {
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>().on_select_all_coins(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let options = ui.global::<CryptoAdapter>().get_coin_catalog();
                let mut options: Vec<CatalogCoin> = options.iter().collect();

                for opt in options.iter_mut() {
                    if opt.visible {
                        opt.selected = true;
                    }
                }

                ui.global::<CryptoAdapter>()
                    .set_coin_catalog(ModelRc::new(VecModel::from(options)));
            }
        });
    }

    // on_clear_coin_selection
    {
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>()
            .on_clear_coin_selection(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let options = ui.global::<CryptoAdapter>().get_coin_catalog();
                    let mut options: Vec<CatalogCoin> = options.iter().collect();

                    for opt in options.iter_mut() {
                        opt.selected = false;
                    }

                    ui.global::<CryptoAdapter>()
                        .set_coin_catalog(ModelRc::new(VecModel::from(options)));
                }
            });
    }

    // on_delete_selected_coins
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>()
            .on_delete_selected_coins(move || -> SharedString {
                if let Some(ui) = ui_weak.upgrade() {
                    let options = ui.global::<CryptoAdapter>().get_coin_catalog();
                    let selected: Vec<String> = options
                        .iter()
                        .filter(|coin| coin.selected)
                        .map(|coin| coin.id.to_string())
                        .collect();

                    if selected.is_empty() {
                        return SharedString::from("No coins selected");
                    }

                    let mut error: Option<String> = None;
                    for id in selected {
                        if let Err(e) = controller.delete_custom_coin(id) {
                            error = Some(e.to_string());
                        }
                    }

                    ui.global::<CryptoAdapter>().invoke_load_coin_catalog();
                    ui.global::<CryptoAdapter>().invoke_load_ticker_options();

                    if let Some(err) = error {
                        return SharedString::from(err);
                    }

                    notify("Coins removed".into(), false);
                    SharedString::from("")
                } else {
                    SharedString::from("")
                }
            });
    }
}
