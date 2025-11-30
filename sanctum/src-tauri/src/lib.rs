// Módulos
pub mod commands;
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
            commands::create_db,
            commands::open_db,
            commands::is_db_initialized,
            commands::close_db,
            commands::get_db_path,
            commands::add_transaction,
            commands::get_transactions,
            commands::get_balance,
            commands::delete_transaction
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
