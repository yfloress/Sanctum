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
use std::sync::Arc;

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
        auth_adapter.on_create_vault(move |password: SharedString| {
            log::info!("create_vault called");

            let password_str = password.to_string();

            match controller_clone.create_db(password_str, None) {
                Ok(msg) => {
                    log::info!("Vault created successfully: {}", msg);
                    SharedString::from("")
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::error!("Failed to create vault: {}", error_msg);
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
        auth_adapter.on_unlock_vault(move |password: SharedString| {
            log::info!("unlock_vault called");

            let password_str = password.to_string();

            match controller_clone.open_db(password_str, None) {
                Ok(msg) => {
                    log::info!("Vault unlocked successfully: {}", msg);
                    SharedString::from("")
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::error!("Failed to unlock vault: {}", error_msg);
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
        auth_adapter.on_lock_vault(move || {
            log::info!("lock_vault called");

            match controller_clone.close_db() {
                Ok(msg) => {
                    log::info!("Vault locked successfully: {}", msg);
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

    let ui_weak = ui.as_weak();

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
                    currency: acc.currency.clone().into(),
                    balance: format_money(current_balance, &acc.currency).into(),
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
                        }
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
        ui.global::<AccountAdapter>().on_delete_account(
            move |id| -> SharedString {
                let result = controller.archive_account(id.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_accounts(&ui_weak, &controller);
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
        });
    }

    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
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
        ui.global::<TransactionAdapter>().on_delete_transaction(
            move |id| -> SharedString {
                let result = controller.delete_transaction(id.to_string());
                match result {
                    Ok(_) => {
                        let _ = reload_transactions(&ui_weak, &controller);
                        let _ = reload_accounts(&ui_weak, &controller);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // Run the UI event loop
    ui.run()?;

    // Cleanup: Close the vault if open
    let _ = controller.close_db();

    log::info!("Sanctum shutting down gracefully");

    Ok(())
}
