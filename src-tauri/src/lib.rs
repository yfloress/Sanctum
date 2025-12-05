// Módulos
pub mod commands;
pub mod crypto;
pub mod db;
pub mod models;
pub mod security_log;

use log::error;
use security_log::init_security_logger;
use std::time::Duration;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize security logger before anything else
    init_security_logger();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::DbState::new())
        .setup(|app| {
            // Get the main window
            let window = app
                .get_webview_window("main")
                .expect("main window not found");

            // Show window immediately with a small delay to allow WebView to initialize
            // This prevents the user from waiting with an invisible window
            std::thread::spawn(move || {
                // Minimal delay to let WebView render the HTML splash screen
                std::thread::sleep(Duration::from_millis(100));

                if let Err(e) = window.show() {
                    error!("Failed to show window: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Database Management
            commands::create_db,
            commands::open_db,
            commands::is_db_initialized,
            commands::close_db,
            commands::get_db_path,
            commands::get_session_remaining,
            // FIAT Accounts
            commands::create_account,
            commands::get_accounts,
            commands::get_account_balances,
            commands::update_account,
            commands::archive_account,
            commands::transfer_funds,
            // Financial Transactions
            commands::add_transaction,
            commands::get_transactions,
            commands::get_balance,
            commands::delete_transaction,
            // Crypto Prices
            commands::get_crypto_prices,
            // Legacy Crypto Holdings (backwards compatibility)
            commands::add_crypto_holding,
            commands::get_crypto_holdings,
            commands::delete_crypto_holding,
            // Crypto Wallets
            commands::add_wallet,
            commands::get_wallets,
            commands::delete_wallet,
            // Crypto Transactions
            commands::add_crypto_transaction,
            commands::add_swap_transaction,
            commands::add_transfer_transaction,
            commands::get_wallet_transactions,
            commands::delete_crypto_transaction,
            // Portfolio Aggregation
            commands::get_aggregated_portfolio,
            commands::get_wallet_holdings,
            // Habits
            commands::create_habit,
            commands::get_habits,
            commands::update_habit,
            commands::archive_habit,
            commands::delete_habit,
            commands::toggle_habit_completion,
            commands::get_habit_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
