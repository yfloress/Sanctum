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

//! Crypto domain callbacks
//!
//! Split into focused submodules:
//! - `helpers` - Shared helper functions (reload_wallets, reload_portfolio)
//! - `portfolio` - Portfolio viewing and price refresh callbacks
//! - `wallets` - Wallet CRUD operations
//! - `transactions` - Transaction add/edit/delete operations
//! - `catalog` - Coin catalog and ticker configuration

mod catalog;
mod helpers;
mod portfolio;
mod tax;
mod transactions;
mod wallets;

use crate::controller::AppController;
use crate::ui::format_clp_rate;
use crate::{CryptoAdapter, AppWindow};
use helpers::SETTING_CRYPTO_LAST_UPDATED;
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;

/// Sets up all CryptoAdapter callbacks by delegating to submodules
pub fn setup_crypto_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + Send + 'static,
{
    // Initial state setup
    if let Ok(Some((rate, _))) = controller.load_exchange_rate_allow_stale("CLP_USD".to_string()) {
        ui.global::<CryptoAdapter>()
            .set_clp_rate(SharedString::from(format_clp_rate(rate)));
    } else {
        ui.global::<CryptoAdapter>()
            .set_clp_rate(SharedString::from("N/A"));
    }

    if let Ok(val) = controller.get_app_setting(SETTING_CRYPTO_LAST_UPDATED)
        && !val.is_empty()
    {
        ui.global::<CryptoAdapter>().set_last_updated(val.into());
    }

    // Setup callbacks from each submodule
    portfolio::setup_portfolio_callbacks(ui, ui_weak, controller, notify.clone());
    wallets::setup_wallet_callbacks(ui, ui_weak, controller, notify.clone());
    transactions::setup_transaction_callbacks(ui, ui_weak, controller, notify.clone());
    catalog::setup_catalog_callbacks(ui, ui_weak, controller, notify.clone());
    tax::setup_tax_callbacks(ui, ui_weak, controller, notify);
}
