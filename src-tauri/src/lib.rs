// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

mod commands;

use directories::ProjectDirs;
use sanctum::controller::AppController;
use std::sync::Arc;

/// Returns the platform-appropriate application data directory.
fn get_app_data_dir() -> std::path::PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "Sanctum") {
        let data_dir = proj_dirs.data_dir().to_path_buf();
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            log::error!("Failed to create data directory: {}", e);
        }
        data_dir
    } else {
        log::error!(
            "Could not determine application data directory, using current directory"
        );
        std::path::PathBuf::from(".")
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_data_dir = get_app_data_dir();
    let controller = Arc::new(AppController::new(app_data_dir));

    // Initialize i18n with system-detected language (must happen before any t() call)
    let detected_lang = sanctum::services::i18n::detect_system_language();
    sanctum::services::i18n::init(&detected_lang);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(controller)
        .invoke_handler(tauri::generate_handler![
            // Vault domain
            commands::vault::check_vault_exists,
            commands::vault::create_vault,
            commands::vault::unlock_vault,
            commands::vault::lock_vault,
            commands::vault::check_password_strength,
            commands::vault::export_vault,
            commands::vault::restore_vault,
            commands::vault::rollback_restore,
            // Settings domain
            commands::settings::load_settings,
            commands::settings::set_dark_mode,
            commands::settings::set_auto_fetch,
            commands::settings::set_proxy_enabled,
            commands::settings::set_proxy_url,
            commands::settings::set_session_timeout,
            commands::settings::set_preferred_currency,
            commands::settings::set_preferred_language,
            commands::settings::set_sidebar_collapsed,
            commands::settings::reset_settings,
            commands::settings::get_app_info,
            commands::settings::get_session_remaining,
            commands::settings::get_translations,
            // Dashboard domain
            commands::dashboard::fetch_balance,
            commands::dashboard::fetch_recent,
            commands::dashboard::fetch_analytics,
            // Finance domain
            commands::finance::fetch_accounts,
            commands::finance::fetch_account_details,
            commands::finance::create_account,
            commands::finance::update_account,
            commands::finance::transfer_funds,
            commands::finance::update_transfer,
            commands::finance::delete_account,
            commands::finance::update_account_icon,
            commands::finance::update_account_name,
            commands::finance::fetch_transactions,
            commands::finance::add_transaction,
            commands::finance::update_transaction,
            commands::finance::delete_transaction,
            commands::finance::load_categories,
            commands::finance::add_category,
            commands::finance::update_category,
            commands::finance::delete_category,
            // Habits domain
            commands::habits::fetch_habits,
            commands::habits::create_habit,
            commands::habits::update_habit,
            commands::habits::delete_habit,
            commands::habits::toggle_habit,
            commands::habits::fetch_habit_summary,
            commands::habits::fetch_heatmap,
            commands::habits::fetch_habit_analytics,
            commands::habits::fetch_rewards,
            commands::habits::create_streak_reward,
            commands::habits::update_streak_reward,
            commands::habits::delete_streak_reward,
            commands::habits::add_milestone,
            commands::habits::fetch_goals,
            commands::habits::create_goal,
            commands::habits::update_goal,
            commands::habits::update_goal_with_checkpoints,
            commands::habits::delete_goal,
            commands::habits::complete_goal,
            commands::habits::archive_goal,
            commands::habits::add_checkpoint,
            commands::habits::update_checkpoint,
            commands::habits::delete_checkpoint,
            commands::habits::toggle_checkpoint,
            commands::habits::fetch_achievements,
            // Ingestion domain
            commands::ingestion::preview_import,
            commands::ingestion::import_data,
            commands::ingestion::max_import_file_size,
            commands::ingestion::detect_exchange_source,
            commands::ingestion::preview_exchange_csv,
            commands::ingestion::import_exchange_csv,
            // Crypto domain
            commands::crypto::fetch_portfolio,
            commands::crypto::fetch_portfolio_trend,
            commands::crypto::fetch_wallets,
            commands::crypto::fetch_wallet_detail,
            commands::crypto::add_wallet,
            commands::crypto::delete_wallet,
            commands::crypto::get_wallet_transaction_count,
            commands::crypto::update_wallet_name,
            commands::crypto::update_wallet_icon,
            commands::crypto::add_crypto_transaction,
            commands::crypto::add_crypto_transfer,
            commands::crypto::add_crypto_swap,
            commands::crypto::update_crypto_transaction,
            commands::crypto::delete_crypto_transaction,
            commands::crypto::get_crypto_transaction,
            commands::crypto::get_crypto_transactions_by_coin,
            commands::crypto::fetch_all_crypto_transactions,
            commands::crypto::get_coin_catalog,
            commands::crypto::set_favorite_coin,
            commands::crypto::add_custom_coin,
            commands::crypto::delete_custom_coin,
            commands::crypto::get_active_ticker_ids,
            commands::crypto::save_active_ticker_ids,
            commands::crypto::save_crypto_prices,
            commands::crypto::load_crypto_prices,
            commands::crypto::get_monitored_coin_ids,
            commands::crypto::load_tax_settings,
            commands::crypto::save_tax_settings,
            commands::crypto::generate_tax_report,
            commands::crypto::generate_tax_summary,
            commands::crypto::export_tax_report_csv,
            commands::crypto::export_tax_history_csv,
            commands::crypto::import_ipc_csv,
            commands::crypto::get_ipc_summary,
            commands::crypto::fill_missing_tax_prices,
            commands::crypto::get_crypto_historical_price_usd,
            commands::crypto::save_exchange_rate,
            commands::crypto::load_exchange_rate,
            commands::crypto::sync_crypto_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
