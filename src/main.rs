//! Sanctum - Personal Finance Manager
//!
//! Main entry point for the Slint-based application.

use chrono::Datelike;
use directories::ProjectDirs;
use log::error;
use rand::Rng; // For title animation
use sanctum::controller::AppController;
use sanctum::security_log::init_security_logger;
use sanctum::ui::{format_category_label, format_decimal_from_cents, format_money};
use slint::SharedString;
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// Slint types are now generated in lib.rs and available via sanctum::*
use sanctum::{
    AccountAdapter, AccountData, AppState, AppWindow, AuthAdapter, CategoryAdapter,
    DashboardAdapter, NotificationAdapter, TransactionAdapter, TransactionCategoryData,
    TransactionData,
};


fn get_app_data_dir() -> std::path::PathBuf {
    // Use directories crate to get platform-appropriate data directory
    if let Some(proj_dirs) = ProjectDirs::from("", "", "Sanctum") {
        let data_dir = proj_dirs.data_dir().to_path_buf();
        // Ensure the directory exists
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            error!("Failed to create data directory: {}", e);
        }
        data_dir
    } else {
        // Fallback to current directory if ProjectDirs fails
        error!("Could not determine application data directory, using current directory");
        std::path::PathBuf::from(".")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize security logger before anything else
    init_security_logger();

    // Initialize environment logger for general logging
    // env_logger::init();

    // Get the application data directory using the directories crate
    let app_data_dir = get_app_data_dir();

    log::info!("Sanctum data directory initialized");

    // Create the application controller
    let controller = Arc::new(AppController::new(app_data_dir));

    // Create the Slint UI
    let ui = AppWindow::new()?;
    let habit_analytics_cache = Rc::new(RefCell::new(sanctum::ui::HabitAnalyticsCache::default()));

    // Title Animation: Decryption Effect
    {
        let ui_weak = ui.as_weak();
        std::thread::spawn(move || {
            let target_text = "SANCTUM";
            let target_len = target_text.len();
            let total_steps = 50; // Total frames (approx 3 seconds)
            let mut rng = rand::rng();

            for step in 0..total_steps {
                let mut current_string = String::new();

                // Calculate how many characters should be resolved from the left
                // We want to hold the "resolved" state for a bit for each char.
                // Map step 0..total_steps to 0..target_len
                // Use a slight curve to make the end faster or linear.
                // Linear is fine for "one by one".
                let resolved_count =
                    (step as f64 / total_steps as f64 * (target_len as f64 + 1.0)) as usize;

                for (i, char) in target_text.chars().enumerate() {
                    if i < resolved_count {
                        // Character is resolved
                        current_string.push(char);
                    } else {
                        // Character is scrambling (A-Z only)
                        // 'A' is 65, 'Z' is 90
                        let random_char = rng.random_range(65..91) as u8 as char;
                        current_string.push(random_char);
                    }
                }

                let text = SharedString::from(current_string);
                let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_login_title(text);
                });

                std::thread::sleep(std::time::Duration::from_millis(60));
            }

            // Final state ensure clean text
            let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                ui.set_login_title("SANCTUM".into());
            });
        });
    }

    // =============== Wire Adapters ===============

    let notification_adapter = ui.global::<NotificationAdapter>();
    let ui_weak_for_notifications = ui.as_weak();
    notification_adapter.on_show(move |message, is_error| {
        if let Some(ui) = ui_weak_for_notifications.upgrade() {
            let adapter = ui.global::<NotificationAdapter>();
            adapter.set_message(message);
            adapter.set_is_error(is_error);
            adapter.set_active(true);

            // Auto-hide after 4 seconds
            let adapter_timer = ui.as_weak();
            slint::Timer::single_shot(std::time::Duration::from_secs(4), move || {
                if let Some(ui) = adapter_timer.upgrade() {
                    ui.global::<NotificationAdapter>().set_active(false);
                }
            });
        }
    });

    let ui_weak = ui.as_weak();

    // Thread-safe notification helper that upgrades to UI thread
    let show_notification = {
        let ui_weak = ui_weak.clone();
        move |message: String, is_error: bool| {
            let ui_weak_clone = ui_weak.clone();
            let _ = ui_weak_clone.upgrade_in_event_loop(move |ui| {
                ui.global::<NotificationAdapter>()
                    .invoke_show(SharedString::from(message), is_error);
            });
        }
    };

    // Session monitor: warn and auto-logout on inactivity
    let session_timer = std::rc::Rc::new(slint::Timer::default());
    let session_warned = std::rc::Rc::new(Cell::new(false));
    let start_session_monitor = std::rc::Rc::new({
        let ui_weak = ui_weak.clone();
        let controller = controller.clone();
        let show_notification_clone = show_notification.clone(); // Capture the thread-safe version
        let timer = session_timer.clone();
        let warned = session_warned.clone();
        move || {
            warned.set(false);
            timer.start(
                slint::TimerMode::Repeated,
                std::time::Duration::from_secs(30),
                {
                    let ui_weak = ui_weak.clone();
                    let controller = controller.clone();
                    let notify_inner_for_session = show_notification_clone.clone();
                    let timer = timer.clone();
                    let warned = warned.clone();
                    move || match controller.get_session_remaining() {
                        Ok(remaining) => {
                            if remaining <= 0 {
                                timer.stop();
                                let _ = controller.close_db();
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.global::<AppState>().set_is_logged_in(false);
                                }
                                notify_inner_for_session(
                                    "Session expired due to inactivity".into(),
                                    true,
                                );
                                warned.set(false);
                                return;
                            }
                            if remaining <= 120 {
                                if !warned.get() {
                                    let mins = (remaining + 59) / 60;
                                    notify_inner_for_session(
                                        format!("Session expires in {mins} minute(s)"),
                                        true,
                                    );
                                    warned.set(true);
                                }
                            } else {
                                warned.set(false);
                            }
                        }
                        Err(_) => {
                            timer.stop();
                            let _ = controller.close_db();
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.global::<AppState>().set_is_logged_in(false);
                            }
                            notify_inner_for_session("Session ended".into(), true);
                            warned.set(false);
                        }
                    }
                },
            );
        }
    });

    // ==================== AuthAdapter Callbacks ====================

    // Get handle to the AuthAdapter global
    let auth_adapter = ui.global::<AuthAdapter>();

    // Callback: check_vault_exists
    // Returns true if a vault file exists on disk
    {
        let controller_clone = controller.clone();
        auth_adapter.on_check_vault_exists(move || {
            let exists = controller_clone.check_vault_exists();
            log::info!("check_vault_exists called, result: {}", exists);
            exists
        });
    }

    // Callback: create_vault
    // Attempts to create a new vault with the given password
    // Returns empty string on success, error message on failure
    {
        let controller_clone = controller.clone();
        let notify = show_notification.clone();
        let session_monitor = start_session_monitor.clone();
        let ui_weak = ui_weak.clone();
        auth_adapter.on_create_vault(move |password: SharedString| {
            log::info!("create_vault called");

            let password_str = password.to_string();

            match controller_clone.create_db(password_str, None) {
                Ok(_) => {
                    log::info!("Vault created successfully");
                    notify("Vault created successfully".into(), false);
                    session_monitor();
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.global::<SettingsAdapter>().invoke_load_settings();
                        ui.global::<CategoryAdapter>().invoke_load_categories();
                    }
                    SharedString::from("")
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::error!("Failed to create vault: {}", error_msg);
                    notify(error_msg.clone(), true);
                    SharedString::from(error_msg)
                }
            }
        });
    }

    // Callback: check_password_strength
    // Returns warning message for weak passwords ("" if ok)
    {
        let controller_clone = controller.clone();
        auth_adapter.on_check_password_strength(move |password: SharedString| {
            SharedString::from(controller_clone.check_password_strength(password.to_string()))
        });
    }

    // Callback: unlock_vault
    // Attempts to unlock an existing vault with the given password
    // Returns empty string on success, error message on failure
    {
        let controller_clone = controller.clone();
        let notify = show_notification.clone();
        let session_monitor = start_session_monitor.clone();
        let ui_weak = ui_weak.clone();
        auth_adapter.on_unlock_vault(move |password: SharedString| {
            log::info!("unlock_vault called");

            let password_str = password.to_string();

            match controller_clone.open_db(password_str, None) {
                Ok(_) => {
                    log::info!("Vault unlocked successfully");
                    notify("Vault unlocked successfully".into(), false);
                    session_monitor();
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.global::<SettingsAdapter>().invoke_load_settings();
                        ui.global::<CategoryAdapter>().invoke_load_categories();
                    }
                    SharedString::from("")
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::error!("Failed to unlock vault: {}", error_msg);
                    notify(error_msg.clone(), true);
                    SharedString::from(error_msg)
                }
            }
        });
    }

    // Callback: lock_vault
    // Closes the current vault connection
    // Returns empty string on success, error message on failure
    {
        let controller_clone = controller.clone();
        let notify = show_notification.clone();
        let session_timer = session_timer.clone();
        let session_warned = session_warned.clone();
        auth_adapter.on_lock_vault(move || {
            log::info!("lock_vault called");

            match controller_clone.close_db() {
                Ok(_) => {
                    log::info!("Vault locked successfully");
                    notify("Vault locked".into(), false);
                    session_timer.stop();
                    session_warned.set(false);
                    SharedString::from("")
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::error!("Failed to lock vault: {}", error_msg);
                    SharedString::from(error_msg)
                }
            }
        });
    }

    // ==================== Application Startup ====================

    println!("Sanctum Core Initialized.");

    // =============== Wire Adapters ===============

    // Current date (YYYY-MM-DD) for default form values
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    ui.global::<AppState>()
        .set_current_date(SharedString::from(today));

    // Reload functions for UI state
    fn reload_accounts(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
    ) -> Result<(), sanctum::controller::ControllerError> {
        let state = sanctum::ui::load_accounts_state(controller)
            .map_err(sanctum::controller::ControllerError::Validation)?;

        let mapped: Vec<AccountData> = state
            .accounts
            .into_iter()
            .map(|acc| AccountData {
                id: acc.id.into(),
                name: acc.name.into(),
                account_type: acc.account_type.into(),
                account_type_key: acc.account_type_key.into(),
                currency: acc.currency.into(),
                balance: acc.balance.into(),
                initial_balance: acc.initial_balance.into(),
                is_archived: acc.is_archived,
            })
            .collect();

        if let Some(ui) = ui_weak.upgrade() {
            let account_adapter = ui.global::<AccountAdapter>();
            account_adapter.set_accounts(ModelRc::new(VecModel::from(mapped)));
            account_adapter.set_total_balance(state.total_balance.into());
            account_adapter.set_is_loading(false);
        }

        Ok(())
    }

    fn reload_transactions(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
    ) -> Result<(), sanctum::controller::ControllerError> {
        let (query, account_filter, category_filter) = if let Some(ui) = ui_weak.upgrade() {
            let adapter = ui.global::<TransactionAdapter>();
            (
                adapter.get_filter_query().to_string(),
                adapter.get_filter_account_id().to_string(),
                adapter.get_filter_category().to_string(),
            )
        } else {
            (String::new(), String::new(), String::new())
        };

        reload_transactions_filtered(
            ui_weak,
            controller,
            query.as_str(),
            account_filter.as_str(),
            category_filter.as_str(),
        )
    }

    fn reload_transactions_filtered(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
        query: &str,
        account_filter: &str,
        category_filter: &str,
    ) -> Result<(), sanctum::controller::ControllerError> {
        let accounts = controller.get_accounts()?;
        let mut account_lookup: HashMap<String, (String, String)> = HashMap::new();
        let mut account_index_map: HashMap<String, i32> = HashMap::new();
        for (idx, account) in accounts.iter().enumerate() {
            account_lookup.insert(
                account.id.clone(),
                (account.currency.clone(), account.name.clone()),
            );
            account_index_map.insert(account.id.clone(), idx as i32);
        }

        let expense_categories = controller.get_transaction_categories("expense".to_string())?;
        let income_categories = controller.get_transaction_categories("income".to_string())?;

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

        let query = query.trim().to_lowercase();
        let account_filter = account_filter.trim();
        let category_filter = category_filter.trim();
        let category_filter_upper = category_filter.to_uppercase();
        let display_limit = if let Some(ui) = ui_weak.upgrade() {
            ui.global::<TransactionAdapter>().get_display_limit() as usize
        } else {
            usize::MAX
        };
        let mut matched_count: usize = 0;

        let transactions = controller.get_transactions()?;

        let mapped: Vec<TransactionData> = transactions
            .into_iter()
            .filter_map(|tx| {
                let (currency, from_name) = account_lookup
                    .get(&tx.account_id)
                    .cloned()
                    .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));

                let is_transfer = tx.transaction_type == "transfer";
                let is_expense = tx.transaction_type == "expense";
                let sign = if is_transfer {
                    "↔"
                } else if is_expense {
                    "-"
                } else {
                    "+"
                };
                let amount_str = format!("{sign} {}", format_money(tx.amount, &currency));

                if !account_filter.is_empty()
                    && tx.account_id != account_filter
                    && tx.transfer_account_id.as_deref() != Some(account_filter)
                {
                    return None;
                }

                if !category_filter.is_empty()
                    && ((is_transfer && category_filter_upper != "TRANSFER")
                        || (!is_transfer
                            && (category_filter_upper == "TRANSFER"
                                || !tx.category.eq_ignore_ascii_case(category_filter))))
                {
                    return None;
                }

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

                if !query.is_empty() {
                    let mut haystack = String::new();
                    haystack.push_str(&tx.description);
                    haystack.push(' ');
                    haystack.push_str(&tx.category);
                    haystack.push(' ');
                    haystack.push_str(&tx.date);
                    haystack.push(' ');
                    haystack.push_str(&from_name);
                    if is_transfer {
                        haystack.push(' ');
                        haystack.push_str(transfer_label);
                    }
                    let haystack = haystack.to_lowercase();
                    if !haystack.contains(&query) {
                        return None;
                    }
                }

                matched_count += 1;
                if matched_count > display_limit {
                    return None;
                }

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
            let transaction_adapter = ui.global::<TransactionAdapter>();
            transaction_adapter.set_transactions(ModelRc::new(VecModel::from(mapped)));
            transaction_adapter.set_has_more(matched_count > display_limit);
            transaction_adapter.set_is_loading(false);
        }

        Ok(())
    }

    fn reload_categories(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
    ) -> Result<(), sanctum::controller::ControllerError> {
        let expense_cats = sanctum::ui::load_categories(controller, "expense")
            .map_err(sanctum::controller::ControllerError::Validation)?;
        let income_cats = sanctum::ui::load_categories(controller, "income")
            .map_err(sanctum::controller::ControllerError::Validation)?;

        let expense_mapped: Vec<TransactionCategoryData> = expense_cats
            .into_iter()
            .map(|cat| TransactionCategoryData {
                id: cat.id.into(),
                name: cat.name.into(),
                is_default: cat.is_default,
            })
            .collect();

        let income_mapped: Vec<TransactionCategoryData> = income_cats
            .into_iter()
            .map(|cat| TransactionCategoryData {
                id: cat.id.into(),
                name: cat.name.into(),
                is_default: cat.is_default,
            })
            .collect();

        if let Some(ui) = ui_weak.upgrade() {
            let adapter = ui.global::<CategoryAdapter>();
            adapter.set_expense_categories(ModelRc::new(VecModel::from(expense_mapped)));
            adapter.set_income_categories(ModelRc::new(VecModel::from(income_mapped)));
        }

        Ok(())
    }

    fn reload_recent(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
    ) -> Result<(), sanctum::controller::ControllerError> {
        let accounts = controller.get_accounts()?;
        let mut account_lookup: HashMap<String, (String, String)> = HashMap::new();
        let mut account_index_map: HashMap<String, i32> = HashMap::new();
        for (idx, account) in accounts.iter().enumerate() {
            account_lookup.insert(
                account.id.clone(),
                (account.currency.clone(), account.name.clone()),
            );
            account_index_map.insert(account.id.clone(), idx as i32);
        }

        let expense_categories = controller.get_transaction_categories("expense".to_string())?;
        let income_categories = controller.get_transaction_categories("income".to_string())?;

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

        let mut transactions = controller.get_transactions()?;
        transactions.sort_by(|a, b| b.date.cmp(&a.date));
        let transactions = transactions.into_iter().take(5).collect::<Vec<_>>();

        let mapped: Vec<TransactionData> = transactions
            .iter()
            .map(|tx| {
                let (currency, from_name) = account_lookup
                    .get(&tx.account_id)
                    .cloned()
                    .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));

                let is_transfer = tx.transaction_type == "transfer";
                let is_expense = tx.transaction_type == "expense";
                let sign = if is_transfer {
                    "↔"
                } else if is_expense {
                    "-"
                } else {
                    "+"
                };
                let amount_str = format!("{sign} {}", format_money(tx.amount, &currency));

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

                TransactionData {
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
                }
            })
            .collect();

        if let Some(ui) = ui_weak.upgrade() {
            let dash = ui.global::<DashboardAdapter>();
            dash.set_recent(ModelRc::new(VecModel::from(mapped)));
        }

        Ok(())
    }

    // AccountAdapter callbacks (extracted to ui/callbacks/finance.rs)
    sanctum::ui::setup_account_callbacks(
        &ui,
        &ui_weak,
        &controller,
        reload_accounts,
        reload_transactions,
        reload_recent,
        show_notification.clone(),
    );

    // TransactionAdapter callbacks (extracted to ui/callbacks/finance.rs)
    sanctum::ui::setup_transaction_callbacks(
        &ui,
        &ui_weak,
        &controller,
        reload_accounts,
        reload_transactions,
        reload_recent,
        show_notification.clone(),
    );

    // DashboardAdapter and AnalyticsAdapter callbacks (extracted to ui/callbacks/dashboard.rs)
    sanctum::ui::setup_dashboard_callbacks(&ui, &ui_weak, &controller, reload_recent);

    // CategoryAdapter callbacks (extracted to ui/callbacks/finance.rs)
    sanctum::ui::setup_category_callbacks(
        &ui,
        &ui_weak,
        &controller,
        reload_categories,
        show_notification.clone(),
    );

    // HabitAdapter callbacks (extracted to ui/callbacks/habits.rs)
    let current_habit_date = Arc::new(Mutex::new(chrono::Local::now().date_naive()));
    let current_heatmap_year = Arc::new(Mutex::new(chrono::Local::now().year()));
    sanctum::ui::setup_habit_callbacks(
        &ui,
        &ui_weak,
        &controller,
        current_habit_date,
        current_heatmap_year,
        habit_analytics_cache,
        show_notification.clone(),
    );

    // CryptoAdapter callbacks (extracted to ui/callbacks/crypto.rs)
    sanctum::ui::setup_crypto_callbacks(&ui, &ui_weak, &controller, show_notification.clone());

    // SettingsAdapter callbacks (extracted to ui/callbacks/settings.rs)
    sanctum::ui::setup_settings_callbacks(
        &ui,
        &ui_weak,
        &controller,
        show_notification.clone(),
    );

    // Run the UI event loop
    ui.run()?;

    // Cleanup: Close the vault if open
    let _ = controller.close_db();

    log::info!("Sanctum shutting down gracefully");

    Ok(())
}
