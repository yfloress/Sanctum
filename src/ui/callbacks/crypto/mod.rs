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
    catalog::setup_catalog_callbacks(ui, ui_weak, controller, notify);
}
