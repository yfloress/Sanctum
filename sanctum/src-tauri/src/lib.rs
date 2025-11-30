// Módulos
pub mod commands;
pub mod crypto;
pub mod db;
pub mod models;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::DbState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Database Management
            commands::create_db,
            commands::open_db,
            commands::is_db_initialized,
            commands::close_db,
            commands::get_db_path,
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
            commands::get_wallet,
            commands::update_wallet,
            commands::delete_wallet,
            // Crypto Transactions
            commands::add_crypto_transaction,
            commands::add_swap_transaction,
            commands::add_transfer_transaction,
            commands::get_wallet_transactions,
            commands::get_all_crypto_transactions,
            commands::delete_crypto_transaction,
            // Portfolio Aggregation
            commands::get_aggregated_portfolio,
            commands::get_wallet_holdings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
