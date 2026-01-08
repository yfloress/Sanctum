//! Finance domain callbacks
//!
//! Callback setup for AccountAdapter, TransactionAdapter, and CategoryAdapter.

use crate::controller::AppController;
use crate::ui::{
    format_category_label, format_decimal_from_cents, format_money, load_account_icon,
    normalize_account_type, parse_amount_input,
};
use crate::{
    AccountAdapter, AnalyticsAdapter, AppState, AppWindow, CategoryAdapter, DashboardAdapter,
    TransactionAdapter, TransactionData,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
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
            if reload_accounts(&ui_weak, &controller).is_err()
                && let Some(ui) = ui_weak.upgrade()
            {
                ui.global::<AccountAdapter>().set_is_loading(false);
            }
        });
    }

    // on_fetch_account_details
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<AccountAdapter>()
            .on_fetch_account_details(move |account_id| {
                let account_id = account_id.to_string();
                let accounts = match controller.get_accounts() {
                    Ok(list) => list,
                    Err(e) => {
                        notify(format!("Failed to load accounts: {}", e), true);
                        return;
                    }
                };

                let account = match accounts.iter().find(|acc| acc.id == account_id) {
                    Some(acc) => acc,
                    None => {
                        notify("Account not found".to_string(), true);
                        return;
                    }
                };

                let balances = match controller.get_account_balances() {
                    Ok(list) => list,
                    Err(e) => {
                        notify(format!("Failed to load balances: {}", e), true);
                        return;
                    }
                };

                let balance_cents = balances
                    .iter()
                    .find(|bal| bal.account_id == account.id)
                    .map(|bal| bal.current_balance)
                    .unwrap_or(account.initial_balance);

                let account_type = match account.account_type.as_str() {
                    "bank" | "Bank" => "Bank",
                    "cash" | "Cash" => "Cash",
                    "savings" | "Savings" => "Savings",
                    "credit_card" | "CreditCard" => "Credit Card",
                    "other" | "Other" => "Other",
                    _ => account.account_type.as_str(),
                };

                let mut account_lookup: HashMap<String, (String, String)> = HashMap::new();
                let mut account_index_map: HashMap<String, i32> = HashMap::new();
                for (idx, acc) in accounts.iter().enumerate() {
                    account_lookup.insert(
                        acc.id.clone(),
                        (acc.currency.clone(), acc.name.clone()),
                    );
                    account_index_map.insert(acc.id.clone(), idx as i32);
                }

                let expense_categories = match controller.get_transaction_categories("expense".to_string()) {
                    Ok(list) => list,
                    Err(e) => {
                        notify(format!("Failed to load categories: {}", e), true);
                        return;
                    }
                };
                let income_categories = match controller.get_transaction_categories("income".to_string()) {
                    Ok(list) => list,
                    Err(e) => {
                        notify(format!("Failed to load categories: {}", e), true);
                        return;
                    }
                };

                let expense_index_map: HashMap<String, i32> = expense_categories
                    .iter()
                    .enumerate()
                    .map(|(idx, cat)| (cat.name.to_uppercase(), idx as i32))
                    .collect();
                let income_index_map: HashMap<String, i32> = income_categories
                    .iter()
                    .enumerate()
                    .map(|(idx, cat)| (cat.name.to_uppercase(), idx as i32))
                    .collect();

                let transactions = match controller.get_transactions() {
                    Ok(list) => list,
                    Err(e) => {
                        notify(format!("Failed to load transactions: {}", e), true);
                        return;
                    }
                };

                let mapped: Vec<TransactionData> = transactions
                    .into_iter()
                    .filter_map(|tx| {
                        if tx.account_id != account_id
                            && tx.transfer_account_id.as_deref() != Some(account_id.as_str())
                        {
                            return None;
                        }

                        let (currency, from_name) = account_lookup
                            .get(&tx.account_id)
                            .cloned()
                            .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));

                        let is_transfer = tx.transaction_type == "transfer";
                        let is_expense = tx.transaction_type == "expense";
                        let amount_str = format_money(tx.amount.abs(), &currency);

                        let transfer_label = tx
                            .transfer_account_id
                            .as_ref()
                            .and_then(|id| account_lookup.get(id))
                            .map(|(_, name)| name.as_str())
                            .unwrap_or("Account");

                        let description = if is_transfer {
                            if tx.description.is_empty() {
                                format!("{from_name} → {transfer_label}")
                            } else {
                                format!("{} ({from_name} → {transfer_label})", tx.description)
                            }
                        } else {
                            tx.description.clone()
                        };

                        let category_raw = if is_transfer {
                            "TRANSFER".to_string()
                        } else {
                            tx.category.to_uppercase()
                        };
                        let category = format_category_label(&category_raw);
                        let category_key = tx.category.to_uppercase();
                        let category_index = if is_expense {
                            expense_index_map.get(&category_key).cloned().unwrap_or(0)
                        } else if is_transfer {
                            0
                        } else {
                            income_index_map.get(&category_key).cloned().unwrap_or(0)
                        };

                        Some(TransactionData {
                            id: tx.id.clone().into(),
                            account_id: tx.account_id.clone().into(),
                            account_index: account_index_map.get(&tx.account_id).cloned().unwrap_or(0),
                            transfer_account_id: tx.transfer_account_id.clone().unwrap_or_default().into(),
                            transfer_account_index: tx
                                .transfer_account_id
                                .as_ref()
                                .and_then(|id| account_index_map.get(id).cloned())
                                .unwrap_or(0),
                            date: tx.date.clone().into(),
                            description: description.into(),
                            description_raw: tx.description.clone().into(),
                            category: category.into(),
                            category_raw: category_raw.into(),
                            category_index,
                            amount: amount_str.into(),
                            amount_raw: format_decimal_from_cents(tx.amount).into(),
                            is_expense,
                            is_transfer,
                        })
                    })
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    let adapter = ui.global::<AccountAdapter>();
                    adapter.set_selected_account_id(SharedString::from(&account.id));
                    adapter.set_selected_account_name(SharedString::from(&account.name));
                    adapter.set_selected_account_type(SharedString::from(account_type));
                    adapter.set_selected_account_currency(SharedString::from(&account.currency));
                    adapter.set_selected_account_balance(SharedString::from(format_money(
                        balance_cents,
                        &account.currency,
                    )));
                    adapter.set_selected_account_icon(load_account_icon(account.icon.clone()));
                    adapter.set_selected_account_icon_path(SharedString::from(
                        account.icon.clone().unwrap_or_default(),
                    ));
                    adapter.set_account_history(ModelRc::new(VecModel::from(mapped)));
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
                let account_type_key = normalize_account_type(&account_type);

                let result = controller.create_account(
                    name.to_string(),
                    account_type_key.clone(),
                    currency.to_string().to_uppercase(),
                    amount_cents,
                    "#8b5cf6".to_string(),
                    None,
                );

                match result {
                    Ok(id) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        if let Some(ui) = ui_weak.upgrade() {
                            let adapter = ui.global::<AccountAdapter>();
                            adapter.set_edit_account_id(SharedString::from(&id));
                            adapter.set_edit_account_icon(SharedString::from(""));
                            ui.global::<AppState>().set_show_add_account(false);
                            ui.global::<AppState>().set_show_edit_account_icon(true);
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
                let id_value = id.to_string();
                let existing_icon = match controller.get_accounts() {
                    Ok(accounts) => accounts
                        .iter()
                        .find(|acc| acc.id == id_value)
                        .and_then(|acc| acc.icon.clone()),
                    Err(e) => return SharedString::from(e.to_string()),
                };

                let result = controller.update_account(
                    id_value,
                    name.to_string(),
                    normalize_account_type(&account_type),
                    currency.to_string().to_uppercase(),
                    amount_cents,
                    "#8b5cf6".to_string(),
                    existing_icon,
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

    // on_update_account_icon
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let notify = notify.clone();
        ui.global::<AccountAdapter>()
            .on_update_account_icon(move |id, icon| -> SharedString {
                let icon_path = if icon.is_empty() {
                    None
                } else {
                    Some(icon.to_string())
                };
                match controller.update_account_icon(id.to_string(), icon_path.clone()) {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        if let Some(ui) = ui_weak.upgrade()
                            && ui.global::<AccountAdapter>().get_show_account_detail()
                        {
                            ui.global::<AccountAdapter>().invoke_fetch_account_details(id);
                        }
                        notify("Account icon updated".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_update_account_name
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let notify = notify.clone();
        ui.global::<AccountAdapter>()
            .on_update_account_name(move |id, new_name| -> SharedString {
                match controller.update_account_name(id.to_string(), new_name.to_string()) {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        if let Some(ui) = ui_weak.upgrade()
                            && ui.global::<AccountAdapter>().get_show_account_detail()
                            && let Ok(accounts) = controller.get_accounts()
                            && let Some(account) = accounts.iter().find(|a| id == a.id)
                        {
                            ui.global::<AccountAdapter>()
                                .set_selected_account_name(SharedString::from(&account.name));
                        }
                        notify("Account name updated".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }
}

/// Sets up all TransactionAdapter callbacks
pub fn setup_transaction_callbacks<F, G, H, N>(
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
    // on_fetch_transactions
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_transactions = reload_transactions.clone();
        let reload_recent = reload_recent.clone();
        ui.global::<TransactionAdapter>()
            .on_fetch_transactions(move || {
                let tx_result = reload_transactions(&ui_weak, &controller);
                let recent_result = reload_recent(&ui_weak, &controller);
                if (tx_result.is_err() || recent_result.is_err())
                    && let Some(ui) = ui_weak.upgrade()
                {
                    ui.global::<TransactionAdapter>().set_is_loading(false);
                }
            });
    }

    // on_add_transaction
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let reload_transactions = reload_transactions.clone();
        let reload_recent = reload_recent.clone();
        let notify = notify.clone();
        ui.global::<TransactionAdapter>().on_add_transaction(
            move |account_id, amount, category, description, date, is_expense| -> SharedString {
                let amount_cents = match parse_amount_input(&amount) {
                    Some(v) if v > 0 => v,
                    _ => return SharedString::from("Amount must be greater than zero"),
                };

                let result = controller.add_transaction(
                    account_id.to_string(),
                    amount_cents,
                    category.to_string(),
                    description.to_string(),
                    date.to_string(),
                    is_expense,
                );

                match result {
                    Ok(_) => {
                        let _ = reload_transactions(&ui_weak, &controller);
                        let _ = reload_accounts(&ui_weak, &controller);
                        let _ = reload_recent(&ui_weak, &controller);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_add_transaction(false);
                            ui.global::<DashboardAdapter>().invoke_fetch_balance();
                            ui.global::<AnalyticsAdapter>().invoke_fetch_analytics(
                                ui.global::<AnalyticsAdapter>().get_active_range(),
                            );
                        }
                        notify("Transaction added".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_update_transaction
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let reload_transactions = reload_transactions.clone();
        let reload_recent = reload_recent.clone();
        let notify = notify.clone();
        ui.global::<TransactionAdapter>().on_update_transaction(
            move |id, account_id, amount, category, description, date, is_expense| -> SharedString {
                let amount_cents = match parse_amount_input(&amount) {
                    Some(v) if v > 0 => v,
                    _ => return SharedString::from("Amount must be greater than zero"),
                };

                let result = controller.update_transaction(
                    id.to_string(),
                    account_id.to_string(),
                    amount_cents,
                    category.to_string(),
                    description.to_string(),
                    date.to_string(),
                    is_expense,
                );

                match result {
                    Ok(_) => {
                        let _ = reload_transactions(&ui_weak, &controller);
                        let _ = reload_accounts(&ui_weak, &controller);
                        let _ = reload_recent(&ui_weak, &controller);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_add_transaction(false);
                            ui.global::<DashboardAdapter>().invoke_fetch_balance();
                            ui.global::<AnalyticsAdapter>().invoke_fetch_analytics(
                                ui.global::<AnalyticsAdapter>().get_active_range(),
                            );
                        }
                        notify("Transaction updated".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_delete_transaction
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_accounts = reload_accounts.clone();
        let reload_transactions = reload_transactions.clone();
        let reload_recent = reload_recent.clone();
        let notify = notify.clone();
        ui.global::<TransactionAdapter>()
            .on_delete_transaction(move |id| -> SharedString {
                let result = controller.delete_transaction(id.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_transactions(&ui_weak, &controller);
                        let _ = reload_accounts(&ui_weak, &controller);
                        let _ = reload_recent(&ui_weak, &controller);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<DashboardAdapter>().invoke_fetch_balance();
                            ui.global::<AnalyticsAdapter>().invoke_fetch_analytics(
                                ui.global::<AnalyticsAdapter>().get_active_range(),
                            );
                        }
                        notify("Transaction deleted".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }
}

/// Sets up all CategoryAdapter callbacks
pub fn setup_category_callbacks<F, N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    reload_categories: F,
    notify: N,
) where
    F: Fn(&Weak<AppWindow>, &Arc<AppController>) -> Result<(), crate::controller::ControllerError>
        + Clone
        + 'static,
    N: Fn(String, bool) + Clone + 'static,
{
    // on_load_categories
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_categories = reload_categories.clone();
        ui.global::<CategoryAdapter>().on_load_categories(move || {
            let _ = reload_categories(&ui_weak, &controller);
        });
    }

    // on_add_category
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_categories = reload_categories.clone();
        let notify = notify.clone();
        ui.global::<CategoryAdapter>().on_add_category(
            move |name, category_type| -> SharedString {
                let result = controller
                    .add_transaction_category(name.to_string(), category_type.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_categories(&ui_weak, &controller);
                        notify("Category added".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_update_category
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_categories = reload_categories.clone();
        let notify = notify.clone();
        ui.global::<CategoryAdapter>()
            .on_update_category(move |id, new_name| -> SharedString {
                let result =
                    controller.update_transaction_category(id.to_string(), new_name.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_categories(&ui_weak, &controller);
                        notify("Category updated".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_delete_category
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let reload_categories = reload_categories.clone();
        let notify = notify.clone();
        ui.global::<CategoryAdapter>()
            .on_delete_category(move |id| -> SharedString {
                let result = controller.delete_transaction_category(id.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_categories(&ui_weak, &controller);
                        notify("Category deleted".to_string(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }
}
