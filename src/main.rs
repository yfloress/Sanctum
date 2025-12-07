//! Sanctum - Personal Finance Manager
//!
//! Main entry point for the Slint-based application.

use directories::ProjectDirs;
use log::error;
use sanctum::controller::AppController;
use sanctum::security_log::init_security_logger;
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
    env_logger::init();

    // Get the application data directory using the directories crate
    let app_data_dir = get_app_data_dir();

    log::info!("Sanctum data directory: {}", app_data_dir.display());

    // Create the application controller
    let controller = Arc::new(AppController::new(app_data_dir));

    // Create the Slint UI
    let ui = AppWindow::new()?;

    // Store a weak reference to the UI for callbacks
    let _ui_weak = ui.as_weak();
    let _controller_clone = controller.clone();

    // Example: Check if a vault exists on startup
    let vault_exists = controller.check_vault_exists();
    log::info!("Vault exists: {}", vault_exists);

    // The controller is now available for the UI to use
    // You can set up callbacks here to connect Slint UI events to controller methods
    //
    // Example pattern for connecting UI callbacks:
    //
    // ui.on_create_vault(move |password| {
    //     let controller = controller_clone.clone();
    //     let ui = ui_weak.unwrap();
    //
    //     match controller.create_db(password.to_string(), None) {
    //         Ok(msg) => {
    //             // Update UI on success
    //         }
    //         Err(e) => {
    //             // Show error in UI
    //         }
    //     }
    // });

    println!("Sanctum Core Initialized.");
    println!("Data directory: {}", controller.get_db_path().unwrap_or_default());

    // Run the UI event loop
    ui.run()?;

    // Cleanup: Close the vault if open
    let _ = controller.close_db();

    Ok(())
}
