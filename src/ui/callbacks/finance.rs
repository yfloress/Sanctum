//! Finance domain callbacks
//!
//! Callback setup for AccountAdapter, TransactionAdapter, and CategoryAdapter.

use crate::controller::AppController;
use crate::ui::{normalize_account_type, parse_amount_input};
use crate::{AccountAdapter, AnalyticsAdapter, AppState, AppWindow, DashboardAdapter};
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;

/// Context for finance callback setup
pub struct FinanceCallbackContext<F, G, H, I>
where
    F: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
    G: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
    H: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
    I: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
{
    pub reload_accounts: F,
    pub reload_transactions: G,
    pub reload_recent: H,
    pub reload_categories: I,
}

/// Sets up all AccountAdapter callbacks
pub fn setup_account_callbacks<F, G, H, N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    reload_accounts: F,
    reload_transactions: G,
    reload_recent: H,
    notify: N,
) where
    F: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
    G: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
    H: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
    N: Fn(String, bool) + Clone + 'static,
{
    // on_fetch_accounts
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        ui.global::<AccountAdapter>().on_fetch_accounts(move || {
            if reload_accounts(&ui_weak, &controller).is_err() {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.global::<AccountAdapter>().set_is_loading(false);
                }
            }
        });
    }

    // on_create_account
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let notify = notify.clone();
        ui.global::<AccountAdapter>().on_create_account(
            move |name, account_type, currency, initial_balance| -> SharedString {
                let amount_cents = parse_amount_input(&initial_balance).unwrap_or(0);

                let result = controller.create_account(
                    name.to_string(),
                    normalize_account_type(&account_type),
                    currency.to_string().to_uppercase(),
                    amount_cents,
                    "#8b5cf6".to_string(),
                    None,
                );

                match result {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_add_account(false);
                            ui.global::<AnalyticsAdapter>()
                                .invoke_fetch_analytics("ALL".into());
                        }
                        notify("Account created successfully".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_update_account
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let reload_recent = reload_recent.clone();
        let notify = notify.clone();
        ui.global::<AccountAdapter>().on_update_account(
            move |id, name, account_type, currency, initial_balance| -> SharedString {
                let amount_cents = parse_amount_input(&initial_balance).unwrap_or(0);

                let result = controller.update_account(
                    id.to_string(),
                    name.to_string(),
                    normalize_account_type(&account_type),
                    currency.to_string().to_uppercase(),
                    amount_cents,
                    "#8b5cf6".to_string(),
                    None,
                );

                match result {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        let _ = reload_recent(&ui_weak, &controller);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_add_account(false);
                            ui.global::<DashboardAdapter>().invoke_fetch_balance();
                            ui.global::<AnalyticsAdapter>()
                                .invoke_fetch_analytics("ALL".into());
                        }
                        notify("Account updated successfully".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_transfer_funds
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let reload_transactions = reload_transactions.clone();
        let reload_recent = reload_recent.clone();
        let notify = notify.clone();
        ui.global::<AccountAdapter>().on_transfer_funds(
            move |from_id, to_id, amount, description, date| -> SharedString {
                let amount_cents = match parse_amount_input(&amount) {
                    Some(v) if v > 0 => v,
                    _ => return SharedString::from("Amount must be greater than zero"),
                };

                let result = controller.transfer_funds(
                    from_id.to_string(),
                    to_id.to_string(),
                    amount_cents,
                    description.to_string(),
                    date.to_string(),
                );

                match result {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        let _ = reload_transactions(&ui_weak, &controller);
                        let _ = reload_recent(&ui_weak, &controller);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_transfer_modal(false);
                            ui.global::<DashboardAdapter>().invoke_fetch_balance();
                            ui.global::<AnalyticsAdapter>().invoke_fetch_analytics(
                                ui.global::<AnalyticsAdapter>().get_active_range(),
                            );
                        }
                        notify("Transfer successful".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_update_transfer
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let reload_transactions = reload_transactions.clone();
        let reload_recent = reload_recent.clone();
        let notify = notify.clone();
        ui.global::<AccountAdapter>().on_update_transfer(
            move |id, from_id, to_id, amount, description, date| -> SharedString {
                let amount_cents = match parse_amount_input(&amount) {
                    Some(v) if v > 0 => v,
                    _ => return SharedString::from("Amount must be greater than zero"),
                };

                let result = controller.update_transfer(
                    id.to_string(),
                    from_id.to_string(),
                    to_id.to_string(),
                    amount_cents,
                    description.to_string(),
                    date.to_string(),
                );

                match result {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        let _ = reload_transactions(&ui_weak, &controller);
                        let _ = reload_recent(&ui_weak, &controller);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_transfer_modal(false);
                            ui.global::<DashboardAdapter>().invoke_fetch_balance();
                            ui.global::<AnalyticsAdapter>().invoke_fetch_analytics(
                                ui.global::<AnalyticsAdapter>().get_active_range(),
                            );
                        }
                        notify("Transfer updated successfully".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_delete_account (archives the account)
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let notify = notify.clone();
        ui.global::<AccountAdapter>()
            .on_delete_account(move |id| -> SharedString {
                match controller.archive_account(id.to_string()) {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        notify("Account archived".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }
}
