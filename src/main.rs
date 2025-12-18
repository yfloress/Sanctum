//! Sanctum - Personal Finance Manager
//!
//! Main entry point for the Slint-based application.

use chrono::Datelike;
use directories::ProjectDirs;
use log::error;
use plotters::prelude::*;
use plotters::series::{AreaSeries, LineSeries};
use rand::Rng; // For title animation
use sanctum::controller::{AppController, MonthlyTrendPoint, SETTING_AUTO_FETCH};
use sanctum::models::CryptoAsset;
use sanctum::security_log::init_security_logger;
use slint::SharedString;
use slint::{Image, Model, ModelRc, VecModel, Weak};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc; // Added for CryptoAdapter logic

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

    log::info!("Sanctum data directory initialized");

    // Create the application controller
    let controller = Arc::new(AppController::new(app_data_dir));

    // Create the Slint UI
    let ui = AppWindow::new()?;

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
                Ok(msg) => {
                    log::info!("Vault created successfully: {}", msg);
                    notify("Vault created successfully".into(), false);
                    session_monitor();
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.global::<SettingsAdapter>().invoke_load_settings();
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
                Ok(msg) => {
                    log::info!("Vault unlocked successfully: {}", msg);
                    notify("Vault unlocked successfully".into(), false);
                    session_monitor();
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.global::<SettingsAdapter>().invoke_load_settings();
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

    // =============== Wire Adapters ===============

    // Current date (YYYY-MM-DD) for default form values
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    ui.global::<AppState>()
        .set_current_date(SharedString::from(today));

    // Helpers to refresh UI models
    fn format_amount(amount_cents: i64) -> String {
        let abs = amount_cents.abs();
        let units = abs / 100;
        let cents = abs % 100;

        let units_str = units.to_string();
        let mut formatted_units = String::new();
        for (count, c) in units_str.chars().rev().enumerate() {
            if count > 0 && count % 3 == 0 {
                formatted_units.insert(0, ',');
            }
            formatted_units.insert(0, c);
        }
        format!("{formatted_units}.{cents:02}")
    }

    fn format_money(amount_cents: i64, currency: &str) -> String {
        let code = currency.to_uppercase();
        format!("{code} {}", format_amount(amount_cents))
    }

    fn format_clp_rate(rate: f64) -> String {
        let rounded = rate.round() as i64;
        let mut digits = rounded.abs().to_string();
        let mut grouped = String::new();

        while digits.len() > 3 {
            let chunk = digits.split_off(digits.len() - 3);
            grouped = format!(",{chunk}{grouped}");
        }

        let formatted = format!("{digits}{grouped}");
        format!("$ {formatted}")
    }

    fn color_from_hex(hex: &str) -> slint::Color {
        if let Some(stripped) = hex.strip_prefix('#')
            && stripped.len() == 6
            && let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&stripped[0..2], 16),
                u8::from_str_radix(&stripped[2..4], 16),
                u8::from_str_radix(&stripped[4..6], 16),
            )
        {
            return slint::Color::from_rgb_u8(r, g, b);
        }
        slint::Color::from_rgb_u8(139, 92, 246)
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
                            ui.global::<AnalyticsAdapter>()
                                .invoke_fetch_analytics("ALL".into());
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
                            ui.global::<AnalyticsAdapter>()
                                .invoke_fetch_analytics("ALL".into());
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
        ui.global::<AccountAdapter>()
            .on_delete_account(move |id| -> SharedString {
                let result = controller.archive_account(id.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
                        notify("Account archived".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // TransactionAdapter callbacks
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<TransactionAdapter>()
            .on_fetch_transactions(move || {
                let _ = reload_transactions(&ui_weak, &controller);
                let _ = reload_recent(&ui_weak, &controller);
            });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
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
            // 1. Load Exchange Rate (CLP -> USD)
            let clp_rate = match controller.load_exchange_rate("CLP_USD".to_string()) {
                Ok(Some((r, _))) => r,
                _ => 0.0,
            };

            // 2. Fetch Accounts & Balances (for normalized calculation)
            let accounts_res = controller.get_accounts();
            let balances_res = controller.get_account_balances();

            // 3. Fetch Crypto Portfolio
            let crypto_result = controller.get_aggregated_portfolio();
            let prices = controller.load_crypto_prices().unwrap_or_default();

            // Create price map for O(1) lookup
            let price_map: HashMap<String, f64> = prices
                .into_iter()
                .map(|p| (p.id, p.current_price))
                .collect();

            if let Ok(accounts) = accounts_res
                && let Ok(balances) = balances_res
                && let Ok(assets) = crypto_result
                && let Some(ui) = ui_weak.upgrade()
            {
                // Create Currency Map (Account ID -> Currency)
                let currency_map: HashMap<String, String> = accounts
                    .into_iter()
                    .map(|a| (a.id, a.currency.to_uppercase()))
                    .collect();

                // Calculate Normalized Fiat Totals
                let mut total_fiat_usd: f64 = 0.0;
                let mut total_income_usd: f64 = 0.0;
                let mut total_expense_usd: f64 = 0.0;

                for bal in balances {
                    let currency = currency_map
                        .get(&bal.account_id)
                        .map(|s| s.as_str())
                        .unwrap_or("USD");
                    let rate = if currency == "CLP" { clp_rate } else { 1.0 };

                    if rate > 0.0 {
                        total_fiat_usd += (bal.current_balance as f64) / rate;
                        total_income_usd += (bal.total_income as f64) / rate;
                        total_expense_usd += (bal.total_expense as f64) / rate;
                    }
                }

                // Calculate Total Crypto Value (in USD)
                let crypto_total: f64 = assets
                    .iter()
                    .map(|asset| {
                        let price = price_map.get(&asset.coin_id).cloned().unwrap_or(0.0);
                        asset.total_amount * price
                    })
                    .sum();

                // Net Worth (Normalized Fiat + Crypto)
                // Fiat sums are in cents (converted to USD cents), Crypto is in dollars
                // Convert Fiat USD cents to dollars for the sum
                let fiat_total_dollars = total_fiat_usd / 100.0;
                let net_worth = fiat_total_dollars + crypto_total;

                let dash = ui.global::<DashboardAdapter>();
                dash.set_balance(BalanceData {
                    total_balance: format_money((net_worth * 100.0) as i64, "USD").into(),
                    total_income: format_money(total_income_usd as i64, "USD").into(),
                    total_expense: format_money(total_expense_usd as i64, "USD").into(),
                });
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
        ui.global::<AnalyticsAdapter>()
            .on_fetch_analytics(move |range| {
                if let Ok(summary) = controller.get_analytics_summary(range.to_string())
                    && let Some(ui) = ui_weak.upgrade()
                {
                    let adapter = ui.global::<AnalyticsAdapter>();

                    let breakdown: Vec<CategoryData> = summary
                        .expense_slices
                        .iter()
                        .map(|slice| CategoryData {
                            name: SharedString::from(&slice.category),
                            amount: SharedString::from(format_money(slice.amount, "USD")),
                            percentage: slice.percentage,
                            color: color_from_hex(&slice.color),
                        })
                        .collect();

                    adapter.set_summary(AnalyticsData {
                        chart_path: SharedString::from(summary.chart_path),
                        net_worth: SharedString::from(summary.net_worth),
                        max_value: SharedString::from(summary.max_value),
                        min_value: SharedString::from(summary.min_value),
                        expense_breakdown: ModelRc::new(VecModel::from(breakdown)),
                    });
                }
            });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<TransactionAdapter>()
            .on_delete_transaction(move |id| -> SharedString {
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
            });
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

        let Some(start_date) = chrono::NaiveDate::from_ymd_opt(year, month, 1) else {
            return;
        };
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };
        let Some(end_date) =
            chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1).and_then(|d| d.pred_opt())
        else {
            return;
        };

        let days_in_month = end_date.day();

        if let Ok(habits) = controller.get_habits() {
            let start_str = start_date.format("%Y-%m-%d").to_string();
            let end_str = end_date.format("%Y-%m-%d").to_string();

            // Fetch logs for the current month view (optimized: single query)
            let logs = controller
                .get_habit_logs(start_str, end_str)
                .unwrap_or_default();

            let mut log_map: std::collections::HashSet<(String, String)> =
                std::collections::HashSet::new();
            for log in logs {
                log_map.insert((log.habit_id, log.completed_date));
            }

            // OPTIMIZATION: Fetch ALL historical logs once for streak calculations
            // Instead of querying per habit inside the loop (N+1 problem), we query once.
            // "1970-01-01" to "2100-01-01" covers all reasonable dates.
            let all_history_logs = controller
                .get_habit_logs("1970-01-01".to_string(), "2100-01-01".to_string())
                .unwrap_or_default();

            // Group history logs by habit_id -> Sorted Vec of NaiveDates
            let mut history_map: HashMap<String, Vec<chrono::NaiveDate>> = HashMap::new();

            for log in all_history_logs {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d")
                {
                    history_map.entry(log.habit_id).or_default().push(date);
                }
            }

            // Sort and dedup dates for each habit
            for dates in history_map.values_mut() {
                dates.sort();
                dates.dedup();
            }

            let mapped_habits: Vec<HabitData> = habits
                .into_iter()
                .map(|h| {
                    let mut days_vec: Vec<HabitDay> = Vec::new();
                    let mut completions = 0;
                    let today = chrono::Local::now().date_naive();

                    // Build monthly view
                    for d in 1..=days_in_month {
                        let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, d) else {
                            continue;
                        };
                        let date_str = date.format("%Y-%m-%d").to_string();
                        let is_future = date > today;

                        let completed = log_map.contains(&(h.id.clone(), date_str.clone()));
                        if completed {
                            completions += 1;
                        }

                        days_vec.push(HabitDay {
                            day: d as i32,
                            completed,
                            date: SharedString::from(date_str),
                            is_future,
                        });
                    }

                    let completion_rate = if days_in_month > 0 {
                        ((completions as f32 / days_in_month as f32) * 100.0) as i32
                    } else {
                        0
                    };

                    // Retrieve pre-processed historical dates for this habit
                    let habit_dates = history_map.get(&h.id).cloned().unwrap_or_default();

                    // Calculate Current Streak
                    let mut current_streak = 0;
                    if !habit_dates.is_empty() {
                        let mut check_date = today;
                        if !habit_dates.contains(&today)
                            && let Some(prev) = today.pred_opt()
                        {
                            check_date = prev;
                        }

                        while habit_dates.contains(&check_date) {
                            current_streak += 1;
                            if let Some(prev) = check_date.pred_opt() {
                                check_date = prev;
                            } else {
                                break;
                            }
                        }
                    }

                    // Calculate Best Streak
                    let mut best_streak = 0;
                    let mut temp_streak = 0;
                    let mut prev_date: Option<chrono::NaiveDate> = None;

                    for date in &habit_dates {
                        if let Some(prev) = prev_date {
                            if let Some(next) = prev.succ_opt() {
                                if *date == next {
                                    temp_streak += 1;
                                } else {
                                    temp_streak = 1;
                                }
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
                })
                .collect();

            if let Some(ui) = ui_weak.upgrade() {
                let adapter = ui.global::<HabitAdapter>();
                adapter.set_habits(ModelRc::new(VecModel::from(mapped_habits)));
                adapter.set_current_month_name(SharedString::from(
                    start_date.format("%B").to_string().to_uppercase(),
                ));
                adapter.set_current_year(year);
                adapter.set_current_month_index(month as i32);

                // Auto-scroll context
                let now = chrono::Local::now().date_naive();
                let is_current = year == now.year() && month == now.month();
                adapter.set_is_viewing_current_month(is_current);
                adapter.set_current_day_int(now.day() as i32);
            }
        }
    }

    fn reload_heatmap(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, year: i32) {
        // 1. Calculate Date Range (Selected Calendar Year)
        let today = chrono::Local::now().date_naive();

        let Some(first_day_year) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) else {
            return;
        };
        let Some(last_day_year) = chrono::NaiveDate::from_ymd_opt(year, 12, 31) else {
            return;
        };

        // Align start to Monday
        let days_from_mon = first_day_year.weekday().num_days_from_monday();
        let start_date = first_day_year - chrono::Duration::days(days_from_mon as i64);

        // Align end to Sunday (so we finish the last week grid)
        let days_to_sun = 6 - last_day_year.weekday().num_days_from_monday();
        let end_date = last_day_year + chrono::Duration::days(days_to_sun as i64);

        // 2. Fetch Logs (Only up to today if current year, otherwise full year)
        // If year < current year, show all. If year == current year, show up to today. If year > current, show none (but grid exists).

        let query_end = if year == today.year() {
            today
        } else if year < today.year() {
            end_date
        } else {
            start_date
        };

        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = query_end.format("%Y-%m-%d").to_string();

        let mut daily_counts: HashMap<String, i32> = HashMap::new();

        if year <= today.year()
            && let Ok(logs) = controller.get_habit_logs(start_str, end_str)
        {
            for log in logs {
                *daily_counts.entry(log.completed_date).or_insert(0) += 1;
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

                let is_future = if year == today.year() {
                    current_day > today
                } else {
                    year > today.year()
                };

                let level = if is_future || count == 0 {
                    0
                } else if count <= 1 {
                    1
                } else if count <= 2 {
                    2
                } else if count <= 4 {
                    3
                } else {
                    4
                };

                week_days.push(HeatmapDay {
                    date: SharedString::from(date_str),
                    count,
                    level,
                });

                current_day += chrono::Duration::days(1);
            }

            weeks_vec.push(HeatmapWeek {
                days: ModelRc::new(VecModel::from(week_days)),
            });
        }

        if let Some(ui) = ui_weak.upgrade() {
            let adapter = ui.global::<HabitAdapter>();
            adapter.set_heatmap_data(ModelRc::new(VecModel::from(weeks_vec)));
            adapter.set_heatmap_year(year);

            // Auto-scroll context (Heatmap)
            // Determine week number (1-52/53)
            // If year matches current year, send current week. Else send 1 (start of year).
            let current_week = if year == today.year() {
                today.iso_week().week() as i32
            } else {
                1
            };
            adapter.set_current_week_int(current_week);
        }
    }

    fn render_monthly_chart_image(data: &[MonthlyTrendPoint]) -> Option<Image> {
        if data.is_empty() {
            return None;
        }

        // Generate SVG with plotters
        let temp_svg = std::env::temp_dir().join("sanctum_monthly_chart_temp.svg");
        let root = SVGBackend::new(&temp_svg, (1200, 360)).into_drawing_area();
        root.fill(&RGBColor(10, 10, 10)).ok()?;

        let max_val = data.iter().map(|d| d.avg_per_day).fold(0.0_f32, f32::max);
        let upper = if max_val <= 0.0 { 1.0 } else { (max_val * 1.2).ceil() };
        let x_max = data.len().max(1) as i32;

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(45)
            .build_cartesian_2d(0..x_max, 0f32..upper)
            .ok()?;

        chart
            .configure_mesh()
            .disable_mesh()
            .disable_y_axis()
            .x_labels(data.len())
            .x_label_formatter(&|v| {
                data.get(*v as usize)
                    .map(|d| d.month_name.clone())
                    .unwrap_or_default()
            })
            .label_style(("sans-serif", 14).into_font().color(&RGBColor(163, 163, 163)))
            .axis_style(ShapeStyle::from(&RGBColor(51, 51, 51)).stroke_width(1))
            .draw()
            .ok()?;

        let area_points: Vec<(i32, f32)> = data
            .iter()
            .enumerate()
            .map(|(i, d)| (i as i32, d.avg_per_day))
            .collect();

        chart
            .draw_series(AreaSeries::new(
                area_points.iter().copied(),
                0.0,
                RGBColor(139, 92, 246).mix(0.2),
            ))
            .ok()?;

        chart
            .draw_series(LineSeries::new(
                area_points.iter().copied(),
                ShapeStyle::from(&RGBColor(139, 92, 246)).stroke_width(2),
            ))
            .ok()?;

        chart
            .draw_series(area_points.iter().map(|&(x, y)| {
                Circle::new((x, y), 3, ShapeStyle::from(&RGBColor(236, 72, 153)).filled())
            }))
            .ok()?;

        root.present().ok()?;

        // Configure fontdb with DejaVu Sans
        let mut fontdb = fontdb::Database::new();
        let font_path = std::path::PathBuf::from("ui/fonts/DejaVuSans.ttf");
        if font_path.exists() {
            fontdb.load_font_file(&font_path).ok()?;
        } else {
            fontdb.load_system_fonts();
        }

        fontdb.set_serif_family("DejaVu Sans");
        fontdb.set_sans_serif_family("DejaVu Sans");
        fontdb.set_monospace_family("DejaVu Sans");

        // Convert SVG text to paths
        let svg_data = std::fs::read_to_string(&temp_svg).ok()?;
        let opt = usvg::Options {
            fontdb: std::sync::Arc::new(fontdb),
            ..Default::default()
        };
        let tree = usvg::Tree::from_str(&svg_data, &opt).ok()?;

        // Write final SVG with text as paths
        let final_svg = std::env::temp_dir().join("sanctum_monthly_chart.svg");
        std::fs::write(&final_svg, tree.to_string(&usvg::WriteOptions::default())).ok()?;

        Image::load_from_path(&final_svg).ok()
    }

    fn refresh_habit_analytics(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>) {
        if let Ok(analytics) = controller.get_habit_analytics(365) {
            let monthly_image = render_monthly_chart_image(&analytics.monthly_data);

            let weekday_data: Vec<WeekdayEfficiencyData> = analytics
                .weekday_data
                .iter()
                .map(|w| WeekdayEfficiencyData {
                    day_name: SharedString::from(&w.day_name),
                    day_short: SharedString::from(&w.day_short),
                    avg_count: w.avg_count,
                    is_best: w.is_best,
                    bar_height_percent: w.bar_height_percent,
                })
                .collect();

            let monthly_data: Vec<MonthlyTrendData> = analytics
                .monthly_data
                .iter()
                .map(|m| MonthlyTrendData {
                    month_name: SharedString::from(&m.month_name),
                    avg_per_day: m.avg_per_day,
                    x_percent: m.x_percent,
                    y_percent: m.y_percent,
                })
                .collect();

            if let Some(ui) = ui_weak.upgrade() {
                let adapter = ui.global::<HabitAdapter>();
                adapter.set_weekday_efficiency(ModelRc::new(VecModel::from(weekday_data)));
                adapter.set_monthly_trend(ModelRc::new(VecModel::from(monthly_data)));
                adapter.set_monthly_chart_image(monthly_image.unwrap_or_default());
            }
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
        ui.global::<HabitAdapter>()
            .on_fetch_habits(move |month, year| {
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
        ui.global::<HabitAdapter>()
            .on_create_habit(move |name, desc, color| -> SharedString {
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
                        refresh_habit_analytics(&ui_weak, &controller);
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
        ui.global::<HabitAdapter>()
            .on_delete_habit(move |id| -> SharedString {
                let result = controller.delete_habit(id.to_string());
                match result {
                    Ok(_) => {
                        let d = *date_lock.lock().unwrap();
                        let y = *year_lock.lock().unwrap();
                        reload_habits(&ui_weak, &controller, d);
                        reload_heatmap(&ui_weak, &controller, y); // Refresh heatmap
                        refresh_habit_analytics(&ui_weak, &controller);
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
        ui.global::<HabitAdapter>()
            .on_toggle_habit(move |id, date| {
                if controller
                    .toggle_habit_completion(id.to_string(), date.to_string())
                    .is_ok()
                {
                    let d = *date_lock.lock().unwrap();
                    let y = *year_lock.lock().unwrap();
                    reload_habits(&ui_weak, &controller, d);
                    reload_heatmap(&ui_weak, &controller, y); // Refresh heatmap
                    refresh_habit_analytics(&ui_weak, &controller);
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
            let (new_y, new_m) = if month == 1 {
                (year - 1, 12)
            } else {
                (year, month - 1)
            };
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
            let (new_y, new_m) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };
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

    // Habit Analytics callback
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<HabitAdapter>()
            .on_fetch_habit_analytics(move || {
                refresh_habit_analytics(&ui_weak, &controller);
            });
    }

    // ==================== CryptoAdapter Logic ====================

    fn reload_wallets(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>) {
        if let Ok(wallets) = controller.get_wallets() {
            let mut wallet_data: Vec<CryptoWalletData> = Vec::new();
            let mut wallet_simple: Vec<WalletSimple> = Vec::new(); // For dropdowns

            let prices = controller.load_crypto_prices().unwrap_or_default();
            let price_map: HashMap<String, f64> = prices
                .into_iter()
                .map(|p| (p.id, p.current_price))
                .collect();

            for w in wallets {
                // Populate simple list
                wallet_simple.push(WalletSimple {
                    id: SharedString::from(&w.id),
                    name: SharedString::from(&w.name),
                });

                let holdings = controller
                    .get_wallet_holdings(w.id.clone())
                    .unwrap_or_default();
                let total_bal: f64 = holdings
                    .iter()
                    .map(|h| {
                        let price = price_map.get(&h.coin_id).cloned().unwrap_or(0.0);
                        h.total_amount * price
                    })
                    .sum();

                wallet_data.push(CryptoWalletData {
                    id: SharedString::from(w.id),
                    name: SharedString::from(w.name),
                    category: SharedString::from(w.category.clone()),
                    icon: SharedString::from(w.icon.unwrap_or_default()),
                    balance: SharedString::from(format_money((total_bal * 100.0) as i64, "USD")),
                    asset_count: holdings.len() as i32,
                });
            }

            if let Some(ui) = ui_weak.upgrade() {
                ui.global::<CryptoAdapter>()
                    .set_wallets(ModelRc::new(VecModel::from(wallet_data)));
                ui.global::<CryptoAdapter>()
                    .set_wallet_list(ModelRc::new(VecModel::from(wallet_simple)));
            }
        }
    }

    fn reload_portfolio(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>) {
        if let Ok(mut assets) = controller.get_aggregated_portfolio() {
            // Load cached prices to update current value
            let prices = controller.load_crypto_prices().unwrap_or_default();
            let price_map: HashMap<String, CryptoAsset> = prices
                .clone()
                .into_iter()
                .map(|p| (p.id.clone(), p))
                .collect();

            // Update assets with current prices
            for asset in &mut assets {
                if let Some(price_data) = price_map.get(&asset.coin_id) {
                    asset.update_with_price(price_data.current_price);
                }
            }

            // Sort by value descending
            assets.sort_by(|a, b| {
                b.current_value
                    .partial_cmp(&a.current_value)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut total_val = 0.0;
            let mut total_cost = 0.0;

            let mapped_assets: Vec<CryptoAssetData> = assets
                .iter()
                .map(|a| {
                    total_val += a.current_value;
                    total_cost += a.total_cost_basis;

                    let price_data = price_map.get(&a.coin_id);

                    let change_percent = price_data
                        .map(|p| p.price_change_percentage_24h)
                        .unwrap_or(0.0);

                    let change_str = if price_data.is_none() {
                        "N/A".to_string() // Explicitly N/A if no price data for change
                    } else if change_percent >= 0.0 {
                        format!("+ {:.2}%", change_percent)
                    } else {
                        format!("{:.2}%", change_percent)
                    };

                    let asset_name = price_data
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| a.symbol.clone());

                    let price_fmt = if price_data.is_none() {
                        "N/A".to_string() // Explicitly N/A if no price data for price
                    } else if a.current_price < 1.0 {
                        format!("$ {:.4}", a.current_price)
                    } else {
                        format_money((a.current_price * 100.0) as i64, "USD")
                    };

                    let value_fmt = if price_data.is_none() {
                        "N/A".to_string() // Explicitly N/A if no price data for value
                    } else {
                        format_money((a.current_value * 100.0) as i64, "USD")
                    };

                    CryptoAssetData {
                        id: SharedString::from(&a.coin_id),
                        symbol: SharedString::from(&a.symbol),
                        name: SharedString::from(asset_name),
                        price: SharedString::from(price_fmt),
                        amount: SharedString::from(format!("{:.4} {}", a.total_amount, a.symbol)),
                        value: SharedString::from(value_fmt),
                        change_24h: SharedString::from(change_str),
                        is_positive: change_percent >= 0.0,
                        allocation: 0.0,
                    }
                })
                .collect();

            // Tickers
            let ticker_ids = controller.get_active_ticker_ids();
            let mut tickers: Vec<CryptoAssetData> = Vec::new();

            for id in ticker_ids {
                if let Some(data) = prices.iter().find(|p| p.id == id) {
                    let change_str = if data.price_change_percentage_24h >= 0.0 {
                        format!("+ {:.2}%", data.price_change_percentage_24h)
                    } else {
                        format!("{:.2}%", data.price_change_percentage_24h)
                    };
                    let price_fmt = if data.current_price < 1.0 {
                        format!("$ {:.4}", data.current_price)
                    } else {
                        format_money((data.current_price * 100.0) as i64, "USD")
                    };

                    tickers.push(CryptoAssetData {
                        id: SharedString::from(id),
                        symbol: SharedString::from(&data.symbol),
                        name: SharedString::from(&data.name),
                        price: SharedString::from(price_fmt),
                        amount: "".into(),
                        value: "".into(),
                        change_24h: SharedString::from(change_str),
                        is_positive: data.price_change_percentage_24h >= 0.0,
                        allocation: 0.0,
                    });
                }
            }

            let total_pnl_val = total_val - total_cost;
            let pnl_sign = if total_pnl_val >= 0.0 { "+" } else { "-" };

            // Try to load CLP rate
            let clp_cached = controller
                .load_exchange_rate("CLP_USD".to_string())
                .ok()
                .flatten();

            let clp_display = clp_cached
                .and_then(|(r, _)| {
                    if r > 0.0 {
                        Some(format_clp_rate(r))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "N/A".to_string());

            if let Some(ui) = ui_weak.upgrade() {
                let adapter = ui.global::<CryptoAdapter>();
                adapter.set_portfolio(ModelRc::new(VecModel::from(mapped_assets)));
                adapter.set_market_tickers(ModelRc::new(VecModel::from(tickers)));
                adapter.set_total_value(SharedString::from(format_money(
                    (total_val * 100.0) as i64,
                    "USD",
                )));
                adapter.set_total_pnl_positive(total_pnl_val >= 0.0);
                adapter.set_total_pnl(SharedString::from(format!(
                    "{} {}",
                    pnl_sign,
                    format_money((total_pnl_val.abs() * 100.0) as i64, "USD")
                )));
                adapter.set_clp_rate(SharedString::from(clp_display));
            }
        }
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>().on_fetch_portfolio(move || {
            reload_portfolio(&ui_weak, &controller);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let show_notification_clone_for_refresh = show_notification.clone(); // Clone for refresh_prices callback

        ui.global::<CryptoAdapter>().on_refresh_prices(move || {
            let controller_async = controller.clone();
            let ui_weak_async = ui_weak.clone();
            let notify_start = show_notification_clone_for_refresh.clone();
            notify_start("Fetching prices...".into(), false);

            let notify_for_async_block = show_notification_clone_for_refresh.clone(); // Clone for the async block

            tokio::spawn(async move {
                // 1. Get coins to update (from settings)
                let coins = controller_async.get_active_ticker_ids();

                if !coins.is_empty() {
                    match controller_async.get_crypto_prices(coins).await {
                        Ok(prices) => {
                            let _ = controller_async.save_crypto_prices(prices);
                        }
                        Err(e) => {
                            let notify_fail = notify_for_async_block.clone(); // Clone for failure message
                            let _ = ui_weak_async.upgrade_in_event_loop(move |_| {
                                notify_fail(format!("Price update failed: {}", e), true);
                            });
                        }
                    }
                }

                // 2. Get CLP Rate
                let clp_display = match controller_async.get_clp_usd_rate().await {
                    Ok(rate) => {
                        let _ = controller_async.save_exchange_rate("CLP_USD".to_string(), rate);
                        format_clp_rate(rate)
                    }
                    Err(_) => {
                        // Try fallback to cache
                        if let Ok(Some((rate, _))) =
                            controller_async.load_exchange_rate("CLP_USD".to_string())
                        {
                            format_clp_rate(rate)
                        } else {
                            "N/A".to_string()
                        }
                    }
                };

                // 3. Reload UI on main thread
                let notify_success = notify_for_async_block.clone(); // Clone for success message
                let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                    ui.global::<CryptoAdapter>().invoke_fetch_portfolio();
                    ui.global::<CryptoAdapter>()
                        .set_clp_rate(SharedString::from(clp_display));
                    notify_success("Prices updated".into(), false);
                });
            });
        });
    }

    // Add Transaction Callback
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();

        // (wallet_id, coin_id, symbol, type, amount, price, fee, date)
        ui.global::<CryptoAdapter>().on_add_transaction(
            move |wallet_id_raw,
                  coin_id,
                  symbol,
                  type_str,
                  amount_str,
                  price_str,
                  fee_str,
                  date|
                  -> SharedString {
                // 1. Parse Amount (String -> f64)
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
                    price_clean.parse().ok()
                };

                let fee_clean = fee_str.replace(",", "").replace("$", "").trim().to_string();
                let fee: Option<f64> = if fee_clean.is_empty() {
                    None
                } else {
                    fee_clean.parse().ok()
                };

                // 3. Add Transaction
                let result = controller.add_crypto_transaction(
                    wallet_id_raw.to_string(),
                    coin_id.to_string(),
                    symbol.to_string(),
                    type_str.to_string(),
                    amount,
                    price_per_coin,
                    fee,
                    date.to_string(),
                    None,
                );

                match result {
                    Ok(_) => {
                        reload_portfolio(&ui_weak, &controller);
                        notify("Asset added successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // Wallet Callbacks
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>().on_fetch_wallets(move || {
            reload_wallets(&ui_weak, &controller);
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<CryptoAdapter>()
            .on_create_wallet(move |name, category| -> SharedString {
                match controller.add_wallet(name.to_string(), category.to_string(), None) {
                    Ok(_) => {
                        reload_wallets(&ui_weak, &controller);
                        notify("Wallet created successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<CryptoAdapter>()
            .on_delete_wallet(move |id| -> SharedString {
                match controller.delete_wallet(id.to_string()) {
                    Ok(_) => {
                        reload_wallets(&ui_weak, &controller);
                        notify("Wallet deleted".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // Ticker Config
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>()
            .on_load_ticker_options(move || {
                let available_coins = vec![
                    ("bitcoin", "Bitcoin", "BTC"),
                    ("ethereum", "Ethereum", "ETH"),
                    ("litecoin", "Litecoin", "LTC"),
                    ("monero", "Monero", "XMR"),
                    ("solana", "Solana", "SOL"),
                    ("polkadot", "Polkadot", "DOT"),
                    ("cardano", "Cardano", "ADA"),
                    ("dogecoin", "Dogecoin", "DOGE"),
                    ("tether", "Tether", "USDT"),
                    ("ripple", "XRP", "XRP"),
                ];

                let active_ids = controller.get_active_ticker_ids();

                let options: Vec<TickerOption> = available_coins
                    .iter()
                    .map(|(id, name, symbol)| TickerOption {
                        id: SharedString::from(*id),
                        name: SharedString::from(*name),
                        symbol: SharedString::from(*symbol),
                        enabled: active_ids.contains(&id.to_string()),
                    })
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.global::<CryptoAdapter>()
                        .set_ticker_options(ModelRc::new(VecModel::from(options)));
                }
            });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();

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

                    ui.global::<CryptoAdapter>().invoke_refresh_prices();
                    notify("Ticker updated".into(), false);
                }
            });
    }

    // Initial Load
    if let Ok(Some((rate, _))) = controller.load_exchange_rate("CLP_USD".to_string()) {
        ui.global::<CryptoAdapter>()
            .set_clp_rate(SharedString::from(format_clp_rate(rate)));
    } else {
        ui.global::<CryptoAdapter>()
            .set_clp_rate(SharedString::from("N/A"));
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>()
            .on_fetch_asset_details(move |coin_id| {
                let coin_id_str = coin_id.to_string();

                // 1. Get Selected Asset Info
                if let Ok(assets) = controller.get_aggregated_portfolio()
                    && let Some(asset) = assets.iter().find(|a| a.coin_id == coin_id_str)
                {
                    let prices = controller.load_crypto_prices().unwrap_or_default();
                    let price_data = prices.iter().find(|p| p.id == coin_id_str);
                    let current_price = price_data.map(|p| p.current_price).unwrap_or(0.0);
                    let price_change = price_data
                        .map(|p| p.price_change_percentage_24h)
                        .unwrap_or(0.0);
                    let asset_name = price_data
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| asset.symbol.clone());

                    let mut updated_asset = asset.clone();
                    if current_price > 0.0 {
                        updated_asset.update_with_price(current_price);
                    }

                    let change_str = if price_change >= 0.0 {
                        format!("+ {:.2}%", price_change)
                    } else {
                        format!("{:.2}%", price_change)
                    };

                    let price_fmt = if updated_asset.current_price < 1.0 {
                        format!("$ {:.4}", updated_asset.current_price)
                    } else {
                        format_money((updated_asset.current_price * 100.0) as i64, "USD")
                    };

                    let selected = CryptoAssetData {
                        id: SharedString::from(&updated_asset.coin_id),
                        symbol: SharedString::from(&updated_asset.symbol),
                        name: SharedString::from(asset_name),
                        price: SharedString::from(price_fmt),
                        amount: SharedString::from(format!(
                            "{:.4} {}",
                            updated_asset.total_amount, updated_asset.symbol
                        )),
                        value: SharedString::from(format_money(
                            (updated_asset.current_value * 100.0) as i64,
                            "USD",
                        )),
                        change_24h: SharedString::from(change_str),
                        is_positive: price_change >= 0.0,
                        allocation: 0.0,
                    };

                    // 2. Wallet Breakdown
                    let wallets = controller.get_wallets().unwrap_or_default();
                    let mut wallet_breakdown: Vec<AssetWalletBreakdown> = Vec::new();

                    for w in wallets {
                        let holdings = controller
                            .get_wallet_holdings(w.id.clone())
                            .unwrap_or_default();
                        if let Some(h) = holdings.iter().find(|h| h.coin_id == coin_id_str)
                            && h.total_amount > 0.0
                        {
                            let val = h.total_amount * current_price;
                            wallet_breakdown.push(AssetWalletBreakdown {
                                wallet_name: SharedString::from(w.name),
                                amount: SharedString::from(format!("{:.4}", h.total_amount)),
                                value: SharedString::from(format_money(
                                    (val * 100.0) as i64,
                                    "USD",
                                )),
                            });
                        }
                    }

                    // 3. Transactions History
                    let history = controller
                        .get_crypto_transactions_by_coin(coin_id_str)
                        .unwrap_or_default();
                    let history_mapped: Vec<AssetTransaction> = history
                        .iter()
                        .map(|tx| {
                            let price_val = tx.price_per_coin.unwrap_or(0.0);
                            let p_fmt = if price_val < 1.0 && price_val > 0.0 {
                                format!("$ {:.4}", price_val)
                            } else {
                                format_money((price_val * 100.0) as i64, "USD")
                            };

                            AssetTransaction {
                                id: SharedString::from(&tx.id),
                                date: SharedString::from(&tx.date),
                                r#type: SharedString::from(tx.transaction_type.to_uppercase()),
                                amount: SharedString::from(format!("{:.4}", tx.amount)),
                                price: SharedString::from(p_fmt),
                            }
                        })
                        .collect();

                    if let Some(ui) = ui_weak.upgrade() {
                        let adapter = ui.global::<CryptoAdapter>();
                        adapter.set_selected_asset(selected);
                        adapter.set_asset_wallets(ModelRc::new(VecModel::from(wallet_breakdown)));
                        adapter.set_asset_history(ModelRc::new(VecModel::from(history_mapped)));
                    }
                }
            });
    }

    // ==================== SettingsAdapter Logic ====================

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<SettingsAdapter>().on_load_settings(move || {
            // Load auto-fetch setting
            if let Ok(val) = controller.get_app_setting(SETTING_AUTO_FETCH) {
                let enabled = val == "true";
                if let Some(ui) = ui_weak.upgrade() {
                    ui.global::<SettingsAdapter>()
                        .set_auto_fetch_enabled(enabled);

                    // SMART FETCH LOGIC
                    // If enabled, check if we need to update prices
                    if enabled {
                        // Check if we have recent prices. We check a benchmark coin (e.g. bitcoin)
                        // Or simply check the CLP rate timestamp as a proxy for all prices
                        let needs_update = if let Ok(Some((_, updated_at))) =
                            controller.load_exchange_rate("CLP_USD".to_string())
                        {
                            // Check age
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&updated_at) {
                                let now = chrono::Utc::now();
                                let age = now
                                    .signed_duration_since(dt.with_timezone(&chrono::Utc))
                                    .num_minutes();
                                age > 10 // Refresh if older than 10 minutes
                            } else {
                                true
                            }
                        } else {
                            true // No cache, update needed
                        };

                        if needs_update {
                            ui.global::<CryptoAdapter>().invoke_refresh_prices();
                        }
                    }
                }
            }
        });
    }

    {
        let controller = controller.clone();
        ui.global::<SettingsAdapter>()
            .on_set_auto_fetch(move |enabled| {
                let val = if enabled { "true" } else { "false" };
                let _ = controller.set_app_setting(SETTING_AUTO_FETCH, val);
            });
    }

    // Run the UI event loop
    ui.run()?;

    // Cleanup: Close the vault if open
    let _ = controller.close_db();

    log::info!("Sanctum shutting down gracefully");

    Ok(())
}
