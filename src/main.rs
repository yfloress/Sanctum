//! Sanctum - Personal Finance Manager
//!
//! Main entry point for the Slint-based application.

use directories::ProjectDirs;
use log::error;
use sanctum::controller::AppController;
use sanctum::security_log::init_security_logger;
use slint::SharedString;
use slint::{ModelRc, VecModel, Weak};
use std::collections::HashMap;
use std::cell::Cell;
use std::sync::Arc;
use chrono::Datelike; // Import Datelike trait

slint::include_modules!();

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

    log::info!("Sanctum data directory: {}", app_data_dir.display());

    // Create the application controller
    let controller = Arc::new(AppController::new(app_data_dir));

    // Create the Slint UI
    let ui = AppWindow::new()?;

    // =============== Wire Adapters ===============

    let ui_weak = ui.as_weak();

    // Notification Helper
    let notification_timer = std::rc::Rc::new(slint::Timer::default());
    let show_notification = {
        let ui_weak = ui_weak.clone();
        let timer = notification_timer.clone();
        move |message: String, is_error: bool| {
            println!("DEBUG: Notification requested: '{}', error: {}", message, is_error); // DEBUG LOG
            if let Some(ui) = ui_weak.upgrade() {
                let adapter = ui.global::<NotificationAdapter>();
                adapter.set_message(SharedString::from(message));
                adapter.set_is_error(is_error);
                adapter.set_active(true);
                println!("DEBUG: Notification active set to true"); // DEBUG LOG
                
                let ui_weak_inner = ui_weak.clone();
                timer.start(slint::TimerMode::SingleShot, std::time::Duration::from_secs(3), move || {
                    println!("DEBUG: Timer fired, hiding notification"); // DEBUG LOG
                    if let Some(ui) = ui_weak_inner.upgrade() {
                        ui.global::<NotificationAdapter>().set_active(false);
                    }
                });
            } else {
                println!("DEBUG: Failed to upgrade UI weak ref for notification");
            }
        }
    };

    // Session monitor: warn and auto-logout on inactivity
    let session_timer = std::rc::Rc::new(slint::Timer::default());
    let session_warned = std::rc::Rc::new(Cell::new(false));
    let start_session_monitor = std::rc::Rc::new({
        let ui_weak = ui_weak.clone();
        let controller = controller.clone();
        let notify = show_notification.clone();
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
                    let notify = notify.clone();
                    let timer = timer.clone();
                    let warned = warned.clone();
                    move || {
                        match controller.get_session_remaining() {
                            Ok(remaining) => {
                                if remaining <= 0 {
                                    timer.stop();
                                    let _ = controller.close_db();
                                    if let Some(ui) = ui_weak.upgrade() {
                                        ui.global::<AppState>().set_is_logged_in(false);
                                    }
                                    notify("Session expired due to inactivity".into(), true);
                                    warned.set(false);
                                    return;
                                }
                                if remaining <= 120 {
                                    if !warned.get() {
                                        let mins = (remaining + 59) / 60;
                                        notify(
                                            format!("Session expires in {mins} minute(s)").into(),
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
                                notify("Session ended".into(), true);
                                warned.set(false);
                            }
                        }
                    }
                },
            );
        }
    });

    // Register NotificationAdapter callback so UI can trigger it
    {
        let notify = show_notification.clone();
        ui.global::<NotificationAdapter>().on_show(move |message, is_error| {
            notify(message.to_string(), is_error);
        });
    }

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
        auth_adapter.on_create_vault(move |password: SharedString| {
            log::info!("create_vault called");

            let password_str = password.to_string();

            match controller_clone.create_db(password_str, None) {
                Ok(msg) => {
                    log::info!("Vault created successfully: {}", msg);
                    notify("Vault created successfully".into(), false);
                    session_monitor();
                    SharedString::from("")
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::error!("Failed to create vault: {}", error_msg);
                    notify(error_msg.clone().into(), true);
                    SharedString::from(error_msg)
                }
            }
        });
    }

    // Callback: unlock_vault
    // Attempts to unlock an existing vault with the given password
    // Returns empty string on success, error message on failure
    {
        let controller_clone = controller.clone();
        let notify = show_notification.clone();
        let session_monitor = start_session_monitor.clone();
        auth_adapter.on_unlock_vault(move |password: SharedString| {
            log::info!("unlock_vault called");

            let password_str = password.to_string();

            match controller_clone.open_db(password_str, None) {
                Ok(msg) => {
                    log::info!("Vault unlocked successfully: {}", msg);
                    notify("Vault unlocked successfully".into(), false);
                    session_monitor();
                    SharedString::from("")
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::error!("Failed to unlock vault: {}", error_msg);
                    notify(error_msg.clone().into(), true);
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
                Ok(msg) => {
                    log::info!("Vault locked successfully: {}", msg);
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
    println!("Data directory: {}", controller.get_db_path().unwrap_or_default());

    // =============== Wire Adapters ===============

    // Current date (YYYY-MM-DD) for default form values
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    ui.global::<AppState>().set_current_date(SharedString::from(today));

    // Helpers to refresh UI models
    fn format_amount(amount_cents: i64) -> String {
        let abs = amount_cents.abs();
        let units = abs / 100;
        let cents = abs % 100;
        format!("{units}.{cents:02}")
    }

    fn currency_symbol(code: &str) -> &str {
        match code.to_uppercase().as_str() {
            "USD" => "$",
            "CLP" => "$",
            _ => "",
        }
    }

    fn format_money(amount_cents: i64, currency: &str) -> String {
        let symbol = currency_symbol(currency);
        format!("{symbol} {}", format_amount(amount_cents))
    }

    fn parse_amount_input(value: &str) -> Option<i64> {
        let cleaned = value.trim().replace(',', "");
        if cleaned.is_empty() {
            return None;
        }
        let parsed: f64 = cleaned.parse().ok()?;
        if !parsed.is_finite() {
            return None;
        }
        Some((parsed * 100.0).round() as i64)
    }

    fn reload_accounts(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
    ) -> Result<(), sanctum::controller::ControllerError> {
        let accounts = controller.get_accounts()?;
        let balances = controller.get_account_balances()?;

        let mut balance_map: HashMap<String, i64> = HashMap::new();
        for bal in balances {
            balance_map.insert(bal.account_id.clone(), bal.current_balance);
        }

        fn format_decimal_from_cents(value: i64) -> String {
            let units = value / 100;
            let cents = value.abs() % 100;
            format!("{units}.{cents:02}")
        }

        let mapped: Vec<AccountData> = accounts
            .iter()
            .map(|acc| {
                let current_balance = balance_map
                    .get(&acc.id)
                    .cloned()
                    .unwrap_or(acc.initial_balance);

                let account_type = match acc.account_type.as_str() {
                    "bank" | "Bank" => "Bank",
                    "cash" | "Cash" => "Cash",
                    "savings" | "Savings" => "Savings",
                    "credit_card" | "CreditCard" => "Credit",
                    _ => acc.account_type.as_str(),
                };

                AccountData {
                    id: acc.id.clone().into(),
                    name: acc.name.clone().into(),
                    account_type: account_type.into(),
                    account_type_key: acc.account_type.clone().into(),
                    currency: acc.currency.clone().into(),
                    balance: format_money(current_balance, &acc.currency).into(),
                    initial_balance: format_decimal_from_cents(acc.initial_balance).into(),
                    is_archived: acc.is_archived,
                }
            })
            .collect();

        if let Some(ui) = ui_weak.upgrade() {
            let account_adapter = ui.global::<AccountAdapter>();
            account_adapter.set_accounts(ModelRc::new(VecModel::from(mapped)));
        }

        Ok(())
    }

    fn reload_transactions(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
    ) -> Result<(), sanctum::controller::ControllerError> {
        let accounts = controller.get_accounts()?;
        let account_lookup: HashMap<String, (String, String)> = accounts
            .iter()
            .map(|a| (a.id.clone(), (a.currency.clone(), a.name.clone())))
            .collect();

        let transactions = controller.get_transactions()?;

        let mapped: Vec<TransactionData> = transactions
            .iter()
            .map(|tx| {
                let (currency, _name) = account_lookup
                    .get(&tx.account_id)
                    .cloned()
                    .unwrap_or_else(|| ("USD".to_string(), "Unknown".to_string()));

                let is_expense = tx.transaction_type == "expense";
                let sign = if is_expense { "-" } else { "+" };
                let amount_str = format!("{sign} {}", format_money(tx.amount, &currency));

                TransactionData {
                    id: tx.id.clone().into(),
                    date: tx.date.clone().into(),
                    description: tx.description.clone().into(),
                    category: tx.category.to_uppercase().into(),
                    amount: amount_str.into(),
                    is_expense,
                }
            })
            .collect();

        if let Some(ui) = ui_weak.upgrade() {
            let transaction_adapter = ui.global::<TransactionAdapter>();
            transaction_adapter.set_transactions(ModelRc::new(VecModel::from(mapped)));
        }

        Ok(())
    }

    fn reload_recent(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
    ) -> Result<(), sanctum::controller::ControllerError> {
        let accounts = controller.get_accounts()?;
        let account_lookup: HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.id.clone(), a.currency.clone()))
            .collect();

        let mut transactions = controller.get_transactions()?;
        transactions.sort_by(|a, b| b.date.cmp(&a.date));
        let transactions = transactions.into_iter().take(5).collect::<Vec<_>>();

        let mapped: Vec<TransactionData> = transactions
            .iter()
            .map(|tx| {
                let currency = account_lookup
                    .get(&tx.account_id)
                    .cloned()
                    .unwrap_or_else(|| "USD".to_string());

                let is_expense = tx.transaction_type == "expense";
                let sign = if is_expense { "-" } else { "+" };
                let amount_str = format!("{sign} {}", format_money(tx.amount, &currency));

                TransactionData {
                    id: tx.id.clone().into(),
                    date: tx.date.clone().into(),
                    description: tx.description.clone().into(),
                    category: tx.category.to_uppercase().into(),
                    amount: amount_str.into(),
                    is_expense,
                }
            })
            .collect();

        if let Some(ui) = ui_weak.upgrade() {
            let dash = ui.global::<DashboardAdapter>();
            dash.set_recent(ModelRc::new(VecModel::from(mapped)));
        }

        Ok(())
    }

    // AccountAdapter callbacks
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<AccountAdapter>().on_fetch_accounts(move || {
            let _ = reload_accounts(&ui_weak, &controller);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<AccountAdapter>().on_create_account(
            move |name, account_type, currency, initial_balance| -> SharedString {
                let amount_cents = parse_amount_input(&initial_balance).unwrap_or(0);

                let result = controller.create_account(
                    name.to_string(),
                    account_type.to_string().to_lowercase(),
                    currency.to_string().to_uppercase(),
                    amount_cents,
                    "#8b5cf6".to_string(), // Default accent color
                    None,
                );

                match result {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_add_account(false);
                            ui.global::<AnalyticsAdapter>().invoke_fetch_analytics("ALL".into());
                        }
                        notify("Account created successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<AccountAdapter>().on_update_account(
            move |id, name, account_type, currency, initial_balance| -> SharedString {
                let amount_cents = parse_amount_input(&initial_balance).unwrap_or(0);

                let result = controller.update_account(
                    id.to_string(),
                    name.to_string(),
                    account_type.to_string().to_lowercase(),
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
                            ui.global::<AnalyticsAdapter>().invoke_fetch_analytics("ALL".into());
                        }
                        notify("Account updated successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
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
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_transfer_modal(false);
                        }
                        notify("Transfer successful".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<AccountAdapter>().on_delete_account(
            move |id| -> SharedString {
                let result = controller.archive_account(id.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        notify("Account archived".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // TransactionAdapter callbacks
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<TransactionAdapter>().on_fetch_transactions(move || {
            let _ = reload_transactions(&ui_weak, &controller);
            let _ = reload_recent(&ui_weak, &controller);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<TransactionAdapter>().on_add_transaction(
            move |account_id,
                  amount,
                  category,
                  description,
                  date,
                  is_expense|
                  -> SharedString {
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

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.global::<AppState>().set_show_add_transaction(false);
                        }
                        notify("Transaction added".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // DashboardAdapter callbacks
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<DashboardAdapter>().on_fetch_balance(move || {
            let result = controller.get_balance();
            if let Ok(balance) = result {
                if let Some(ui) = ui_weak.upgrade() {
                    let dash = ui.global::<DashboardAdapter>();
                    dash.set_balance(BalanceData {
                        total_balance: format_money(balance.total_balance, "USD").into(),
                        total_income: format_money(balance.total_income, "USD").into(),
                        total_expense: format_money(balance.total_expense, "USD").into(),
                    });
                }
            }
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<DashboardAdapter>().on_fetch_recent(move || {
            let _ = reload_recent(&ui_weak, &controller);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<AnalyticsAdapter>().on_fetch_analytics(move |range| {
            // 1. Net Worth History
            if let Ok((path, net_worth, min_val, max_val)) = controller.get_net_worth_history(&range) {
                if let Some(ui) = ui_weak.upgrade() {
                    let adapter = ui.global::<AnalyticsAdapter>();
                    adapter.set_chart_path(SharedString::from(path));
                    adapter.set_net_worth(SharedString::from(net_worth));
                    adapter.set_min_value(SharedString::from(min_val));
                    adapter.set_max_value(SharedString::from(max_val));
                }
            }

            // 2. Expense Breakdown (Keep it global/ALL for now, or filter later if needed)
            if let Ok(expenses) = controller.get_expenses_by_category() {
                 let total_expense: i64 = expenses.iter().map(|(_, amt)| amt).sum();
                 
                 let colors = [
                    slint::Color::from_rgb_u8(139, 92, 246), // #8b5cf6
                    slint::Color::from_rgb_u8(236, 72, 153), // #ec4899
                    slint::Color::from_rgb_u8(59, 130, 246), // #3b82f6
                    slint::Color::from_rgb_u8(16, 185, 129), // #10b981
                    slint::Color::from_rgb_u8(245, 158, 11), // #f59e0b
                    slint::Color::from_rgb_u8(239, 68, 68),  // #ef4444
                    slint::Color::from_rgb_u8(99, 102, 241), // #6366f1
                    slint::Color::from_rgb_u8(20, 184, 166), // #14b8a6
                 ];

                 let mapped: Vec<CategoryData> = expenses.iter().enumerate().map(|(i, (cat, amt))| {
                     let percentage = if total_expense > 0 {
                         *amt as f32 / total_expense as f32
                     } else {
                         0.0
                     };
                     
                     let color = colors[i % colors.len()];
                     
                     CategoryData {
                         name: SharedString::from(cat),
                         amount: SharedString::from(format_money(*amt, "USD")),
                         percentage,
                         color,
                     }
                 }).collect();
                 
                 if let Some(ui) = ui_weak.upgrade() {
                     let adapter = ui.global::<AnalyticsAdapter>();
                     adapter.set_expense_breakdown(ModelRc::new(VecModel::from(mapped)));
                 }
            }
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<TransactionAdapter>().on_delete_transaction(
            move |id| -> SharedString {
                let result = controller.delete_transaction(id.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_transactions(&ui_weak, &controller);
                        let _ = reload_accounts(&ui_weak, &controller);
                        notify("Transaction deleted".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // ==================== HabitAdapter Logic ====================
    
    let current_habit_date = Arc::new(std::sync::Mutex::new(chrono::Local::now().date_naive()));
    let current_heatmap_year = Arc::new(std::sync::Mutex::new(chrono::Local::now().year()));

    fn reload_habits(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
        current_date: chrono::NaiveDate,
    ) {
        let year = current_date.year();
        let month = current_date.month();
        
        let start_date = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };
        let end_date = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .unwrap()
            .pred_opt()
            .unwrap();
            
        let days_in_month = end_date.day();
        
        if let Ok(habits) = controller.get_habits() {
            let start_str = start_date.format("%Y-%m-%d").to_string();
            let end_str = end_date.format("%Y-%m-%d").to_string();
            
            // Fetch logs for the current month view (optimized: single query)
            let logs = controller.get_habit_logs(start_str, end_str).unwrap_or_default();
            
            let mut log_map: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
            for log in logs {
                log_map.insert((log.habit_id, log.completed_date));
            }
            
            // OPTIMIZATION: Fetch ALL historical logs once for streak calculations
            // Instead of querying per habit inside the loop (N+1 problem), we query once.
            // "1970-01-01" to "2100-01-01" covers all reasonable dates.
            let all_history_logs = controller.get_habit_logs("1970-01-01".to_string(), "2100-01-01".to_string()).unwrap_or_default();
            
            // Group history logs by habit_id -> Sorted Vec of NaiveDates
            let mut history_map: HashMap<String, Vec<chrono::NaiveDate>> = HashMap::new();
            
            for log in all_history_logs {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d") {
                    history_map.entry(log.habit_id).or_default().push(date);
                }
            }
            
            // Sort and dedup dates for each habit
            for dates in history_map.values_mut() {
                dates.sort();
                dates.dedup();
            }
            
            let mapped_habits: Vec<HabitData> = habits.into_iter().map(|h| {
                let mut days_vec: Vec<HabitDay> = Vec::new();
                let mut completions = 0;
                let today = chrono::Local::now().date_naive();
                
                // Build monthly view
                for d in 1..=days_in_month {
                    let date = chrono::NaiveDate::from_ymd_opt(year, month, d).unwrap();
                    let date_str = date.format("%Y-%m-%d").to_string();
                    let is_future = date > today;
                    
                    let completed = log_map.contains(&(h.id.clone(), date_str.clone()));
                    if completed { completions += 1; }
                    
                    days_vec.push(HabitDay {
                        day: d as i32,
                        completed,
                        date: SharedString::from(date_str),
                        is_future,
                    });
                }
                
                let completion_rate = if days_in_month > 0 {
                    ((completions as f32 / days_in_month as f32) * 100.0) as i32
                } else { 0 };
                
                // Retrieve pre-processed historical dates for this habit
                let habit_dates = history_map.get(&h.id).cloned().unwrap_or_default();
                
                // Calculate Current Streak
                let mut current_streak = 0;
                if !habit_dates.is_empty() {
                    let mut check_date = today;
                    if !habit_dates.contains(&today) {
                         // If today is not done, check if yesterday was done to keep streak alive
                         check_date = today.pred_opt().unwrap();
                    }
                    
                    while habit_dates.contains(&check_date) {
                        current_streak += 1;
                        check_date = check_date.pred_opt().unwrap();
                    }
                }
                
                // Calculate Best Streak
                let mut best_streak = 0;
                let mut temp_streak = 0;
                let mut prev_date: Option<chrono::NaiveDate> = None;
                
                for date in &habit_dates {
                    if let Some(prev) = prev_date {
                        if *date == prev.succ_opt().unwrap() {
                            temp_streak += 1;
                        } else {
                            temp_streak = 1;
                        }
                    } else {
                        temp_streak = 1;
                    }
                    if temp_streak > best_streak {
                        best_streak = temp_streak;
                    }
                    prev_date = Some(*date);
                }

                let color = if h.color.starts_with("#") && h.color.len() == 7 {
                    let r = u8::from_str_radix(&h.color[1..3], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&h.color[3..5], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&h.color[5..7], 16).unwrap_or(0);
                    slint::Color::from_rgb_u8(r, g, b)
                } else {
                    slint::Color::from_rgb_u8(139, 92, 246)
                };

                HabitData {
                    id: SharedString::from(h.id),
                    name: SharedString::from(h.name),
                    description: SharedString::from(h.description.unwrap_or_default()),
                    color,
                    streak: current_streak,
                    best_streak,
                    completion_rate,
                    days: ModelRc::new(VecModel::from(days_vec)),
                }
            }).collect();
            
            if let Some(ui) = ui_weak.upgrade() {
                let adapter = ui.global::<HabitAdapter>();
                adapter.set_habits(ModelRc::new(VecModel::from(mapped_habits)));
                adapter.set_current_month_name(SharedString::from(start_date.format("%B").to_string().to_uppercase()));
                adapter.set_current_year(year);
                adapter.set_current_month_index(month as i32);
            }
        }
    }
    
    fn reload_barchart(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
        year: i32,
    ) {
        let start_str = format!("{}-01-01", year);
        let end_str = format!("{}-12-31", year);
        
        // Fetch all habits to map IDs to Colors
        let habit_colors: HashMap<String, slint::Color> = if let Ok(habits) = controller.get_habits() {
            habits.into_iter().map(|h| {
                 let color = if h.color.starts_with("#") && h.color.len() == 7 {
                    let r = u8::from_str_radix(&h.color[1..3], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&h.color[3..5], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&h.color[5..7], 16).unwrap_or(0);
                    slint::Color::from_rgb_u8(r, g, b)
                } else {
                    slint::Color::from_rgb_u8(139, 92, 246)
                };
                (h.id, color)
            }).collect()
        } else {
            HashMap::new()
        };
        
        if let Ok(logs) = controller.get_habit_logs(start_str, end_str) {
            // Aggregate by month (0-11) -> Total Count, and Dominant Habit ID tracking
            // Structure: month_index -> (total_count, HashMap<habit_id, count>)
            let mut month_data: Vec<(i32, HashMap<String, i32>)> = vec![(0, HashMap::new()); 12];
            
            for log in logs {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d") {
                     let m = date.month0() as usize;
                     if m < 12 {
                         month_data[m].0 += 1;
                         *month_data[m].1.entry(log.habit_id).or_insert(0) += 1;
                     }
                }
            }
            
            let max_val = month_data.iter().map(|(total, _)| *total).max().unwrap_or(1);
            let scale_max = if max_val == 0 { 10 } else { max_val }; 
            
            let month_names = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];
            
            let stats: Vec<MonthlyStats> = month_data.iter().enumerate().map(|(i, (total, habits_map))| {
                // Find dominant habit
                let mut dominant_color = slint::Color::from_rgb_u8(100, 100, 100); // Default gray
                
                if *total > 0 {
                    if let Some((best_habit_id, _)) = habits_map.iter().max_by_key(|(_, count)| **count) {
                        if let Some(c) = habit_colors.get(best_habit_id) {
                            dominant_color = *c;
                        }
                    }
                }
                
                MonthlyStats {
                    month_name: SharedString::from(month_names[i]),
                    total_completions: *total,
                    max_completions: scale_max,
                    dominant_color,
                }
            }).collect();
            
            if let Some(ui) = ui_weak.upgrade() {
                let adapter = ui.global::<HabitAdapter>();
                adapter.set_chart_data(ModelRc::new(VecModel::from(stats)));
            }
        }
    }

    fn reload_heatmap(
        ui_weak: &Weak<AppWindow>,
        controller: &Arc<AppController>,
        year: i32,
    ) {
        // Reload bar chart whenever heatmap (year) changes
        reload_barchart(ui_weak, controller, year);
        
        // 1. Calculate Date Range (Selected Calendar Year)
        let today = chrono::Local::now().date_naive();
        
        let first_day_year = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let last_day_year = chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
        
        // Align start to Monday
        let days_from_mon = first_day_year.weekday().num_days_from_monday();
        let start_date = first_day_year - chrono::Duration::days(days_from_mon as i64);
        
        // Align end to Sunday (so we finish the last week grid)
        let days_to_sun = 6 - last_day_year.weekday().num_days_from_monday(); 
        let end_date = last_day_year + chrono::Duration::days(days_to_sun as i64);

        // 2. Fetch Logs (Only up to today if current year, otherwise full year)
        // If year < current year, show all. If year == current year, show up to today. If year > current, show none (but grid exists).
        
        let query_end = if year == today.year() { today } else if year < today.year() { end_date } else { start_date };
        
        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = query_end.format("%Y-%m-%d").to_string();
        
        let mut daily_counts: HashMap<String, i32> = HashMap::new();
        
        if year <= today.year() {
             if let Ok(logs) = controller.get_habit_logs(start_str, end_str) {
                for log in logs {
                    *daily_counts.entry(log.completed_date).or_insert(0) += 1;
                }
             }
        }
            
        // 4. Build Structure
        let mut weeks_vec: Vec<HeatmapWeek> = Vec::new();
        let mut current_day = start_date;
        
        // Iterate until we cover the full end_date
        while current_day <= end_date {
            let mut week_days: Vec<HeatmapDay> = Vec::new();
            
            for _ in 0..7 {
                let date_str = current_day.format("%Y-%m-%d").to_string();
                let count = *daily_counts.get(&date_str).unwrap_or(&0);
                
                // Logic: 
                // If viewing current year: hide future days (> today).
                // If viewing past year: show all.
                // If viewing future year: show empty grid.
                
                let is_future = if year == today.year() { current_day > today } else { year > today.year() };
                
                let level = if is_future { 0 }
                else if count == 0 { 0 }
                else if count <= 1 { 1 }
                else if count <= 2 { 2 }
                else if count <= 4 { 3 }
                else { 4 };
                
                week_days.push(HeatmapDay {
                    date: SharedString::from(date_str),
                    count,
                    level,
                });
                
                current_day = current_day + chrono::Duration::days(1);
            }
            
            weeks_vec.push(HeatmapWeek {
                days: ModelRc::new(VecModel::from(week_days)),
            });
        }
        
        if let Some(ui) = ui_weak.upgrade() {
            let adapter = ui.global::<HabitAdapter>();
            adapter.set_heatmap_data(ModelRc::new(VecModel::from(weeks_vec)));
            adapter.set_heatmap_year(year);
        }
    }

    // Callbacks
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        ui.global::<HabitAdapter>().on_load_initial_data(move || {
             let now = chrono::Local::now().date_naive();
             *date_lock.lock().unwrap() = now;
             *year_lock.lock().unwrap() = now.year();
             reload_habits(&ui_weak, &controller, now);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        ui.global::<HabitAdapter>().on_fetch_habits(move |month, year| {
             if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month as u32, 1) {
                 *date_lock.lock().unwrap() = date;
                 reload_habits(&ui_weak, &controller, date);
             }
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        let notify = show_notification.clone();
        ui.global::<HabitAdapter>().on_create_habit(move |name, desc, color| -> SharedString {
            let result = controller.create_habit(
                name.to_string(),
                Some(desc.to_string()),
                color.to_string(),
            );
            match result {
                Ok(_) => {
                    let d = *date_lock.lock().unwrap();
                    let y = *year_lock.lock().unwrap();
                    reload_habits(&ui_weak, &controller, d);
                    reload_heatmap(&ui_weak, &controller, y); // Refresh heatmap
                    notify("Habit created".into(), false);
                    SharedString::from("")
                }
                Err(e) => SharedString::from(e.to_string()),
            }
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        let notify = show_notification.clone();
        ui.global::<HabitAdapter>().on_delete_habit(move |id| -> SharedString {
            let result = controller.delete_habit(id.to_string());
            match result {
                Ok(_) => {
                    let d = *date_lock.lock().unwrap();
                    let y = *year_lock.lock().unwrap();
                    reload_habits(&ui_weak, &controller, d);
                    reload_heatmap(&ui_weak, &controller, y); // Refresh heatmap
                    notify("Habit deleted".into(), false);
                    SharedString::from("")
                }
                Err(e) => SharedString::from(e.to_string()),
            }
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        ui.global::<HabitAdapter>().on_toggle_habit(move |id, date| {
            if let Ok(_) = controller.toggle_habit_completion(id.to_string(), date.to_string()) {
                 let d = *date_lock.lock().unwrap();
                 let y = *year_lock.lock().unwrap();
                 reload_habits(&ui_weak, &controller, d);
                 reload_heatmap(&ui_weak, &controller, y); // Refresh heatmap
            }
        });
    }
    
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        ui.global::<HabitAdapter>().on_prev_month(move || {
             let mut d = date_lock.lock().unwrap();
             // Subtract 1 month safely
             let month = d.month();
             let year = d.year();
             let (new_y, new_m) = if month == 1 { (year - 1, 12) } else { (year, month - 1) };
             *d = chrono::NaiveDate::from_ymd_opt(new_y, new_m, 1).unwrap();
             reload_habits(&ui_weak, &controller, *d);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        ui.global::<HabitAdapter>().on_next_month(move || {
             let mut d = date_lock.lock().unwrap();
             let month = d.month();
             let year = d.year();
             let (new_y, new_m) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
             *d = chrono::NaiveDate::from_ymd_opt(new_y, new_m, 1).unwrap();
             reload_habits(&ui_weak, &controller, *d);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let year_lock = current_heatmap_year.clone();
        ui.global::<HabitAdapter>().on_fetch_heatmap_data(move || {
            let y = *year_lock.lock().unwrap();
            reload_heatmap(&ui_weak, &controller, y);
        });
    }
    
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let year_lock = current_heatmap_year.clone();
        ui.global::<HabitAdapter>().on_prev_heatmap_year(move || {
            let mut y = year_lock.lock().unwrap();
            *y -= 1;
            reload_heatmap(&ui_weak, &controller, *y);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let year_lock = current_heatmap_year.clone();
        ui.global::<HabitAdapter>().on_next_heatmap_year(move || {
            let mut y = year_lock.lock().unwrap();
            *y += 1;
            reload_heatmap(&ui_weak, &controller, *y);
        });
    }

    // Run the UI event loop
    ui.run()?;

    // Cleanup: Close the vault if open
    let _ = controller.close_db();

    log::info!("Sanctum shutting down gracefully");

    Ok(())
}
