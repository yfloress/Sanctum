//! Sanctum - Personal Finance Manager
//!
//! Main entry point for the Slint-based application.

use chrono::Datelike;
use directories::ProjectDirs;
use log::error;
use plotters::prelude::*;
use plotters::series::{AreaSeries, LineSeries};
use rand::Rng; // For title animation
use sanctum::crypto;
use sanctum::controller::{
    AppController, MonthlyTrendPoint, SETTING_AUTO_FETCH, SETTING_CRYPTO_LAST_COIN_ID,
    SETTING_CRYPTO_LAST_UPDATED, SETTING_CRYPTO_LAST_WALLET_ID,
};
use sanctum::models::CryptoAsset;
use sanctum::security_log::init_security_logger;
use slint::SharedString;
use slint::{Image, Model, ModelRc, VecModel, Weak};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
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

    fn fallback_chart_color(index: usize) -> (u8, u8, u8) {
        match index % 6 {
            0 => (139, 92, 246),
            1 => (236, 72, 153),
            2 => (56, 189, 248),
            3 => (34, 197, 94),
            4 => (245, 158, 11),
            _ => (168, 85, 247),
        }
    }

    fn symbol_chart_color(symbol: &str, index: usize) -> (u8, u8, u8) {
        match symbol.to_uppercase().as_str() {
            "BTC" => (247, 147, 26),
            "ETH" => (98, 126, 234),
            "USDT" => (38, 161, 123),
            "USDC" => (39, 117, 202),
            "BNB" => (243, 186, 47),
            "SOL" => (20, 241, 149),
            "XMR" => (255, 102, 0),
            "LTC" => (191, 191, 191),
            "ADA" => (0, 51, 173),
            "DOGE" => (194, 166, 51),
            "XRP" => (0, 136, 204),
            "MATIC" => (130, 71, 229),
            "DOT" => (230, 0, 122),
            "AVAX" => (232, 65, 66),
            _ => fallback_chart_color(index),
        }
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
            let clp_rate = match controller.load_exchange_rate_allow_stale("CLP_USD".to_string())
            {
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

        // Generate SVG with plotters (high resolution for crisp rendering)
        let temp_svg = std::env::temp_dir().join("sanctum_monthly_chart_temp.svg");
        let root = SVGBackend::new(&temp_svg, (2400, 720)).into_drawing_area();
        root.fill(&RGBColor(10, 10, 10)).ok()?;

        let max_val = data.iter().map(|d| d.avg_per_day).fold(0.0_f32, f32::max);
        let upper = if max_val <= 0.0 { 1.0 } else { (max_val * 1.2).ceil() };
        let x_max = data.len().max(1) as i32;

        let mut chart = ChartBuilder::on(&root)
            .margin(40)
            .x_label_area_size(80)
            .y_label_area_size(90)
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
            .label_style(("sans-serif", 28).into_font().color(&RGBColor(163, 163, 163)))
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
                ShapeStyle::from(&RGBColor(139, 92, 246)).stroke_width(5),
            ))
            .ok()?;

        chart
            .draw_series(area_points.iter().map(|&(x, y)| {
                Circle::new((x, y), 8, ShapeStyle::from(&RGBColor(236, 72, 153)).filled())
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

    fn render_portfolio_distribution_chart(data: &[(String, f64)]) -> Option<Image> {
        if data.is_empty() {
            return None;
        }

        let total: f64 = data.iter().map(|(_, value)| *value).sum();
        if total <= 0.0 {
            return None;
        }

        let temp_svg = std::env::temp_dir().join("sanctum_portfolio_dist_temp.svg");
        let root = SVGBackend::new(&temp_svg, (600, 600)).into_drawing_area();

        let sizes: Vec<f64> = data.iter().map(|(_, value)| *value).collect();
        let labels_empty: Vec<String> = vec![String::new(); data.len()];
        let colors: Vec<RGBColor> = data
            .iter()
            .enumerate()
            .map(|(idx, (label, _))| {
                let (r, g, b) = symbol_chart_color(label, idx);
                RGBColor(r, g, b)
            })
            .collect();

        let center = (300, 300);
        let radius = 220.0;
        let mut pie = Pie::new(&center, &radius, &sizes, &colors, &labels_empty);
        pie.start_angle(-90.0);
        pie.donut_hole(radius * 0.6);

        root.draw(&pie).ok()?;

        root.present().ok()?;

        Image::load_from_path(&temp_svg).ok()
    }

    fn render_portfolio_trend_chart(data: &[(String, f64, f64)]) -> Option<Image> {
        if data.len() < 2 {
            return None;
        }

        let mut min_val = f64::MAX;
        let mut max_val = 0.0_f64;

        for (_, total_value, total_cost) in data {
            if *total_value > max_val {
                max_val = *total_value;
            }
            if *total_cost > max_val {
                max_val = *total_cost;
            }
            if *total_value < min_val {
                min_val = *total_value;
            }
            if *total_cost < min_val {
                min_val = *total_cost;
            }
        }

        if max_val <= 0.0 {
            return None;
        }

        let padding = ((max_val - min_val) * 0.1).max(max_val * 0.05);
        let lower = (min_val - padding).max(0.0);
        let upper = max_val + padding;

        let temp_svg = std::env::temp_dir().join("sanctum_portfolio_trend.svg");
        let root = SVGBackend::new(&temp_svg, (1800, 520)).into_drawing_area();
        root.fill(&RGBColor(10, 10, 15)).ok()?;

        let x_max = (data.len() - 1) as i32;
        let mut chart = ChartBuilder::on(&root)
            .margin(18)
            .build_cartesian_2d(0..x_max, lower..upper)
            .ok()?;

        chart
            .configure_mesh()
            .disable_mesh()
            .disable_x_axis()
            .disable_y_axis()
            .draw()
            .ok()?;

        let value_points: Vec<(i32, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, (_, total_value, _))| (i as i32, *total_value))
            .collect();
        let cost_points: Vec<(i32, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, (_, _, total_cost))| (i as i32, *total_cost))
            .collect();

        chart
            .draw_series(AreaSeries::new(
                value_points.iter().copied(),
                lower,
                RGBColor(139, 92, 246).mix(0.2),
            ))
            .ok()?;

        chart
            .draw_series(LineSeries::new(
                value_points.iter().copied(),
                ShapeStyle::from(&RGBColor(139, 92, 246)).stroke_width(4),
            ))
            .ok()?;

        chart
            .draw_series(LineSeries::new(
                cost_points.iter().copied(),
                ShapeStyle::from(&RGBColor(148, 163, 184)).stroke_width(2),
            ))
            .ok()?;

        root.present().ok()?;

        Image::load_from_path(&temp_svg).ok()
    }

    // TODO(remove-demo): temporary mock data for trend chart preview.
    fn build_mock_portfolio_trend(total_value: f64, total_cost: f64) -> Vec<(String, f64, f64)> {
        let base_value = if total_value > 0.0 {
            total_value
        } else if total_cost > 0.0 {
            total_cost
        } else {
            1000.0
        };
        let base_cost = if total_cost > 0.0 {
            total_cost
        } else {
            base_value * 0.85
        };

        let points = 36usize;
        let today = chrono::Local::now().date_naive();
        let mut data = Vec::with_capacity(points);

        for i in 0..points {
            let t = i as f64;
            let drift = t / (points as f64 - 1.0) * 0.08;
            let wave = (t / 3.0).sin() * 0.05 + (t / 7.0).cos() * 0.02;
            let value = (base_value * (1.0 + drift + wave)).max(0.01);

            let cost_wave = (t / 5.0).sin() * 0.01;
            let cost = (base_cost * (1.0 + drift * 0.4 + cost_wave)).max(0.01);

            let date = today - chrono::Duration::days((points - 1 - i) as i64);
            data.push((date.format("%Y-%m-%d").to_string(), value, cost));
        }

        data
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
                let last_wallet_id = controller
                    .get_app_setting(SETTING_CRYPTO_LAST_WALLET_ID)
                    .ok()
                    .filter(|val| !val.is_empty());
                let last_wallet_index = last_wallet_id
                    .as_ref()
                    .and_then(|id| {
                        wallet_simple
                            .iter()
                            .position(|wallet| wallet.id.as_str() == id)
                    })
                    .unwrap_or(0) as i32;

                ui.global::<CryptoAdapter>()
                    .set_wallets(ModelRc::new(VecModel::from(wallet_data)));
                ui.global::<CryptoAdapter>()
                    .set_wallet_list(ModelRc::new(VecModel::from(wallet_simple)));
                ui.global::<CryptoAdapter>()
                    .set_default_wallet_index(last_wallet_index);
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
            let catalog = controller
                .get_coin_catalog()
                .unwrap_or_else(|_| crypto::default_coin_catalog());
            let catalog_map: HashMap<String, (String, String)> = catalog
                .into_iter()
                .map(|coin| (coin.id, (coin.name, coin.symbol)))
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

            let mut chart_assets: Vec<(String, f64)> = assets
                .iter()
                .filter(|asset| price_map.contains_key(&asset.coin_id) && asset.current_value > 0.0)
                .map(|asset| (asset.symbol.clone(), asset.current_value))
                .collect();
            chart_assets.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let chart_assets = if chart_assets.len() > 6 {
                let mut trimmed = chart_assets[..6].to_vec();
                let other_sum: f64 = chart_assets[6..].iter().map(|(_, v)| *v).sum();
                if other_sum > 0.0 {
                    trimmed.push(("OTHER".to_string(), other_sum));
                }
                trimmed
            } else {
                chart_assets
            };
            let chart_total: f64 = chart_assets.iter().map(|(_, value)| *value).sum();
            let distribution: Vec<CryptoDistributionSlice> = if chart_total > 0.0 {
                chart_assets
                    .iter()
                    .enumerate()
                    .map(|(idx, (label, value))| {
                        let percent = (*value / chart_total) * 100.0;
                        let (r, g, b) = symbol_chart_color(label, idx);
                        CryptoDistributionSlice {
                            label: SharedString::from(label),
                            value: SharedString::from(format_money((value * 100.0) as i64, "USD")),
                            percent: SharedString::from(format!("{:.1}%", percent)),
                            color: slint::Color::from_rgb_u8(r, g, b),
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

            let chart_image = render_portfolio_distribution_chart(&chart_assets);
            let chart_ready = chart_image.is_some();

            let mut total_val = 0.0;
            let mut total_cost = 0.0;
            let mut priced_assets = 0;

            let mapped_assets: Vec<CryptoAssetData> = assets
                .iter()
                .map(|a| {
                    let price_data = price_map.get(&a.coin_id);
                    if price_data.is_some() {
                        total_val += a.current_value;
                        total_cost += a.total_cost_basis;
                        priced_assets += 1;
                    }

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
                if let Some(data) = price_map.get(&id) {
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
                        id: SharedString::from(&id),
                        symbol: SharedString::from(&data.symbol),
                        name: SharedString::from(&data.name),
                        price: SharedString::from(price_fmt),
                        amount: "".into(),
                        value: "".into(),
                        change_24h: SharedString::from(change_str),
                        is_positive: data.price_change_percentage_24h >= 0.0,
                        allocation: 0.0,
                    });
                } else {
                    let (name, symbol) = catalog_map
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(|| (id.clone(), id.to_uppercase()));

                    tickers.push(CryptoAssetData {
                        id: SharedString::from(&id),
                        symbol: SharedString::from(symbol),
                        name: SharedString::from(name),
                        price: "N/A".into(),
                        amount: "".into(),
                        value: "".into(),
                        change_24h: "N/A".into(),
                        is_positive: true,
                        allocation: 0.0,
                    });
                }
            }

            let (total_value_label, total_pnl_label, total_pnl_positive) = if priced_assets > 0 {
                let total_pnl_val = total_val - total_cost;
                let pnl_sign = if total_pnl_val >= 0.0 { "+" } else { "-" };
                (
                    format_money((total_val * 100.0) as i64, "USD"),
                    format!(
                        "{} {}",
                        pnl_sign,
                        format_money((total_pnl_val.abs() * 100.0) as i64, "USD")
                    ),
                    total_pnl_val >= 0.0,
                )
            } else {
                ("N/A".to_string(), "N/A".to_string(), true)
            };

            let mut trend_image = None;
            let mut trend_ready = false;
            if priced_assets > 0 {
                let _ = controller.save_crypto_portfolio_snapshot(total_val, total_cost);
                let snapshots = controller
                    .get_crypto_portfolio_snapshots(180)
                    .unwrap_or_default();
                let mut trend_points: Vec<(String, f64, f64)> = snapshots
                    .into_iter()
                    .filter(|(_, value, cost)| *value > 0.0 || *cost > 0.0)
                    .collect();
                if trend_points.len() < 2 {
                    trend_points = build_mock_portfolio_trend(total_val, total_cost);
                }
                trend_image = render_portfolio_trend_chart(&trend_points);
                trend_ready = trend_image.is_some();
            }

            // Try to load CLP rate
            let clp_cached = controller
                .load_exchange_rate_allow_stale("CLP_USD".to_string())
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

            let last_updated_label = controller
                .get_app_setting(SETTING_CRYPTO_LAST_UPDATED)
                .ok()
                .filter(|val| !val.is_empty())
                .or_else(|| {
                    prices
                        .iter()
                        .filter_map(|price| {
                            chrono::DateTime::parse_from_rfc3339(&price.last_updated).ok()
                        })
                        .max()
                        .map(|dt| {
                            let local = dt.with_timezone(&chrono::Local);
                            let now = chrono::Local::now();
                            if local.date_naive() == now.date_naive() {
                                format!("Today at {}", local.format("%H:%M"))
                            } else {
                                local.format("%Y-%m-%d %H:%M").to_string()
                            }
                        })
                });

            if let Some(ui) = ui_weak.upgrade() {
                let adapter = ui.global::<CryptoAdapter>();
                adapter.set_portfolio(ModelRc::new(VecModel::from(mapped_assets)));
                adapter.set_market_tickers(ModelRc::new(VecModel::from(tickers)));
                adapter.set_total_value(SharedString::from(total_value_label));
                adapter.set_total_pnl_positive(total_pnl_positive);
                adapter.set_total_pnl(SharedString::from(total_pnl_label));
                adapter.set_clp_rate(SharedString::from(clp_display));
                adapter.set_portfolio_trend_image(trend_image.unwrap_or_default());
                adapter.set_portfolio_trend_ready(trend_ready);
                adapter.set_portfolio_chart_image(chart_image.unwrap_or_default());
                adapter.set_portfolio_chart_ready(chart_ready);
                adapter.set_portfolio_distribution(ModelRc::new(VecModel::from(distribution)));
                if let Some(label) = last_updated_label {
                    adapter.set_last_updated(label.into());
                }
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
                // 1. Get coins to update (Settings + Wallets)
                let coins = controller_async
                    .get_monitored_coin_ids()
                    .unwrap_or_default();
                
                let limit_reached = coins.len() > 50;
                let limit_excluded = if limit_reached {
                    let extra_count = coins.len().saturating_sub(50);
                    let preview: Vec<String> =
                        coins.iter().skip(50).take(3).cloned().collect();
                    if preview.is_empty() {
                        String::new()
                    } else if extra_count > preview.len() {
                        format!("{} +{} more", preview.join(", "), extra_count - preview.len())
                    } else {
                        preview.join(", ")
                    }
                } else {
                    String::new()
                };
                let has_coins = !coins.is_empty();

                let mut prices_updated = false;
                if !coins.is_empty() {
                    match controller_async.get_crypto_prices(coins).await {
                        Ok(prices) => {
                            let _ = controller_async.save_crypto_prices(prices);
                            prices_updated = true;
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
                let (clp_display, clp_updated) = match controller_async.get_clp_usd_rate().await {
                    Ok(rate) => {
                        let _ = controller_async.save_exchange_rate("CLP_USD".to_string(), rate);
                        (format_clp_rate(rate), true)
                    }
                    Err(_) => {
                        // Try fallback to cache
                        if let Ok(Some((rate, _))) =
                            controller_async.load_exchange_rate_allow_stale("CLP_USD".to_string())
                        {
                            (format_clp_rate(rate), true)
                        } else {
                            ("N/A".to_string(), false)
                        }
                    }
                };

                // 3. Reload UI on main thread
                let notify_success = notify_for_async_block.clone(); // Clone for success message
                let now = chrono::Local::now().format("%H:%M").to_string();
                let last_updated_label =
                    if prices_updated { Some(format!("Today at {}", now)) } else { None };

                if let Some(label) = last_updated_label.as_ref() {
                    let _ = controller_async.set_app_setting(SETTING_CRYPTO_LAST_UPDATED, label);
                }

                let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                    ui.global::<CryptoAdapter>().invoke_fetch_portfolio();
                    ui.global::<CryptoAdapter>()
                        .set_clp_rate(SharedString::from(clp_display));
                    if let Some(label) = last_updated_label {
                        ui.global::<CryptoAdapter>().set_last_updated(label.into());
                    }
                    ui.global::<CryptoAdapter>().set_limit_reached(limit_reached);
                    ui.global::<CryptoAdapter>()
                        .set_limit_excluded(limit_excluded.into());
                    if prices_updated {
                        notify_success("Prices updated".into(), false);
                    } else if !has_coins && clp_updated {
                        notify_success("Rates updated".into(), false);
                    }
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
        // (wallet_id, coin_id, symbol, type, amount, price, fee, date, notes)
        ui.global::<CryptoAdapter>().on_add_transaction(
            move |wallet_id_raw,
                  coin_id,
                  symbol,
                  type_str,
                  amount_str,
                  price_str,
                  fee_str,
                  date,
                  notes_str|
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

                let notes = if notes_str.is_empty() {
                    None
                } else {
                    Some(notes_str.to_string())
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
                    notes,
                );

                match result {
                    Ok(_) => {
                        let _ = controller.set_app_setting(
                            SETTING_CRYPTO_LAST_WALLET_ID,
                            wallet_id_raw.as_ref(),
                        );
                        let _ = controller
                            .set_app_setting(SETTING_CRYPTO_LAST_COIN_ID, coin_id.as_ref());
                        reload_portfolio(&ui_weak, &controller);
                        notify("Asset added successfully".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // Add Transfer Callback (between wallets)
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();

        // (from_wallet_id, to_wallet_id, coin_id, symbol, from_amount, to_amount, fee, date, notes)
        ui.global::<CryptoAdapter>().on_add_transfer(
            move |from_wallet_id,
                  to_wallet_id,
                  coin_id,
                  symbol,
                  from_amount_str,
                  to_amount_str,
                  fee_str,
                  date,
                  notes_str|
                  -> SharedString {
                let parse_amount = |raw: SharedString, label: &str| -> Result<f64, SharedString> {
                    let cleaned = raw
                        .replace(",", "")
                        .replace("$", "")
                        .trim()
                        .to_string();
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

    // Add Swap Callback
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();

        ui.global::<CryptoAdapter>().on_add_swap(
            move |wallet_id_raw,
                  from_coin_id,
                  from_symbol,
                  from_amount_str,
                  to_coin_id,
                  to_symbol,
                  to_amount_str,
                  fee_str,
                  date,
                  notes_str|
                  -> SharedString {
                let parse_amount = |raw: SharedString, label: &str| -> Result<f64, SharedString> {
                    let cleaned = raw
                        .replace(",", "")
                        .replace("$", "")
                        .trim()
                        .to_string();
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
                    date.to_string(),
                    notes,
                );

                match result {
                    Ok(_) => {
                        let _ = controller.set_app_setting(
                            SETTING_CRYPTO_LAST_WALLET_ID,
                            wallet_id_raw.as_ref(),
                        );
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

    // Load Transaction for Edit Callback
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
                        .set_edit_date(SharedString::from(&tx.date));
                    ui.global::<CryptoAdapter>()
                        .set_edit_notes(SharedString::from(notes_str));
                }

                SharedString::from("")
            });
    }

    // Update Transaction Callback
    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>()
            .on_update_transaction(move |id, amount_str, price_str, fee_str, date, notes_str| {
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
                    date.to_string(),
                    notes,
                ) {
                    Ok(_) => SharedString::from(""),
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();
        ui.global::<CryptoAdapter>()
            .on_delete_crypto_transaction(move |id| -> SharedString {
                match controller.delete_crypto_transaction(id.to_string()) {
                    Ok(_) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            let coin_id = ui.global::<CryptoAdapter>().get_selected_asset().id;
                            ui.global::<CryptoAdapter>().invoke_fetch_asset_details(coin_id);
                            ui.global::<CryptoAdapter>().invoke_fetch_portfolio();
                        }
                        notify("Transaction deleted".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
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
                    let price_map: HashMap<String, CryptoAsset> = prices
                        .into_iter()
                        .map(|p| (p.id.clone(), p))
                        .collect();

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
                                format_money((asset.current_price * 100.0) as i64, "USD")
                            };

                            let value_fmt = if price_data.is_none() {
                                "N/A".to_string()
                            } else {
                                format_money((asset.current_value * 100.0) as i64, "USD")
                            };

                            CryptoAssetData {
                                id: SharedString::from(&asset.coin_id),
                                symbol: SharedString::from(&asset.symbol),
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
                    let history_type_map: HashMap<String, String> = history
                        .iter()
                        .map(|tx| (tx.id.clone(), tx.transaction_type.clone()))
                        .collect();
                    let history_mapped: Vec<AssetTransaction> = history
                        .iter()
                        .map(|tx| {
                            let price_val = tx.price_per_coin.unwrap_or(0.0);
                            let p_fmt = if price_val < 1.0 && price_val > 0.0 {
                                format!("$ {:.4}", price_val)
                            } else if price_val > 0.0 {
                                format_money((price_val * 100.0) as i64, "USD")
                            } else {
                                "N/A".to_string()
                            };
                            let fee_val = tx.fee.unwrap_or(0.0);
                            let fee_fmt = if fee_val > 0.0 {
                                format_money((fee_val * 100.0) as i64, "USD")
                            } else {
                                "".to_string()
                            };
                            let notes = tx.notes.clone().unwrap_or_default();
                            let is_swap = tx.transaction_type == "swap"
                                || (tx.transaction_type == "transfer_in"
                                    && tx
                                        .related_tx_id
                                        .as_ref()
                                        .and_then(|id| history_type_map.get(id))
                                        .map(|t| t == "swap")
                                        .unwrap_or(false));

                            AssetTransaction {
                                id: SharedString::from(&tx.id),
                                date: SharedString::from(&tx.date),
                                r#type: SharedString::from(tx.transaction_type.to_uppercase()),
                                amount: SharedString::from(format!("{:.4}", tx.amount)),
                                price: SharedString::from(p_fmt),
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
                let active_ids = controller.get_active_ticker_ids();
                let catalog = controller
                    .get_coin_catalog()
                    .unwrap_or_else(|_| crypto::default_coin_catalog());

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

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>()
            .on_load_coin_catalog(move || {
                let catalog = controller
                    .get_coin_catalog()
                    .unwrap_or_else(|_| crypto::default_coin_catalog());
                let last_coin_id = controller
                    .get_app_setting(SETTING_CRYPTO_LAST_COIN_ID)
                    .ok()
                    .filter(|val| !val.is_empty());
                let last_coin_index = last_coin_id
                    .as_ref()
                    .and_then(|id| catalog.iter().position(|coin| coin.id == *id))
                    .unwrap_or(0) as i32;
                let favorites: HashSet<String> = controller
                    .get_favorite_coin_ids()
                    .into_iter()
                    .collect();

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

                    reload_portfolio(&ui_weak, &controller);
                    notify("Configuration saved".into(), false);
                }
            });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();

        ui.global::<CryptoAdapter>()
            .on_add_custom_coin(move |id, name, symbol| -> SharedString {
                match controller.add_custom_coin(id.to_string(), name.to_string(), symbol.to_string())
                {
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

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();

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

    {
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>()
            .on_select_all_coins(move || {
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

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = show_notification.clone();

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

    // Initial Load
    if let Ok(Some((rate, _))) = controller.load_exchange_rate_allow_stale("CLP_USD".to_string())
    {
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

    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>()
            .on_get_last_price(move |coin_id| {
                let prices = controller.load_crypto_prices().unwrap_or_default();
                if let Some(data) = prices.iter().find(|p| p.id == coin_id.as_str()) {
                    format!("{:.4}", data.current_price).into()
                } else {
                    "".into()
                }
            });
    }

    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>()
            .on_get_swap_quote(move |from_coin_id, to_coin_id, amount_str| {
                let amount_clean = amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let amount: f64 = match amount_clean.parse() {
                    Ok(value) if value > 0.0 => value,
                    _ => return SharedString::from(""),
                };

                let prices = controller.load_crypto_prices().unwrap_or_default();
                let from_price = prices
                    .iter()
                    .find(|p| p.id == from_coin_id.as_str())
                    .map(|p| p.current_price)
                    .unwrap_or(0.0);
                let to_price = prices
                    .iter()
                    .find(|p| p.id == to_coin_id.as_str())
                    .map(|p| p.current_price)
                    .unwrap_or(0.0);

                if from_price <= 0.0 || to_price <= 0.0 {
                    return SharedString::from("");
                }

                let to_amount = amount * (from_price / to_price);
                let mut formatted = format!("{:.8}", to_amount);
                while formatted.contains('.') && formatted.ends_with('0') {
                    formatted.pop();
                }
                if formatted.ends_with('.') {
                    formatted.pop();
                }

                SharedString::from(formatted)
            });
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

                    let missing_price = price_data.is_none();
                    let change_str = if missing_price {
                        "N/A".to_string()
                    } else if price_change >= 0.0 {
                        format!("+ {:.2}%", price_change)
                    } else {
                        format!("{:.2}%", price_change)
                    };

                    let price_fmt = if missing_price {
                        "N/A".to_string()
                    } else if updated_asset.current_price < 1.0 {
                        format!("$ {:.4}", updated_asset.current_price)
                    } else {
                        format_money((updated_asset.current_price * 100.0) as i64, "USD")
                    };

                    let value_fmt = if missing_price {
                        "N/A".to_string()
                    } else {
                        format_money(
                            (updated_asset.current_value * 100.0) as i64,
                            "USD",
                        )
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
                        value: SharedString::from(value_fmt),
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
                    let history_type_map: HashMap<String, String> = history
                        .iter()
                        .map(|tx| (tx.id.clone(), tx.transaction_type.clone()))
                        .collect();
                    let history_mapped: Vec<AssetTransaction> = history
                        .iter()
                        .map(|tx| {
                            let price_val = tx.price_per_coin.unwrap_or(0.0);
                            let p_fmt = if price_val < 1.0 && price_val > 0.0 {
                                format!("$ {:.4}", price_val)
                            } else if price_val > 0.0 {
                                format_money((price_val * 100.0) as i64, "USD")
                            } else {
                                "N/A".to_string()
                            };
                            let fee_val = tx.fee.unwrap_or(0.0);
                            let fee_fmt = if fee_val > 0.0 {
                                format_money((fee_val * 100.0) as i64, "USD")
                            } else {
                                "".to_string()
                            };
                            let notes = tx.notes.clone().unwrap_or_default();
                            let is_swap = tx.transaction_type == "swap"
                                || (tx.transaction_type == "transfer_in"
                                    && tx
                                        .related_tx_id
                                        .as_ref()
                                        .and_then(|id| history_type_map.get(id))
                                        .map(|t| t == "swap")
                                        .unwrap_or(false));

                            AssetTransaction {
                                id: SharedString::from(&tx.id),
                                date: SharedString::from(&tx.date),
                                r#type: SharedString::from(tx.transaction_type.to_uppercase()),
                                amount: SharedString::from(format!("{:.4}", tx.amount)),
                                price: SharedString::from(p_fmt),
                                fee: SharedString::from(fee_fmt),
                                notes: SharedString::from(notes),
                                is_swap,
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
                        let needs_update = if let Ok(Some((_, updated_at))) = controller
                            .load_exchange_rate_allow_stale("CLP_USD".to_string())
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
