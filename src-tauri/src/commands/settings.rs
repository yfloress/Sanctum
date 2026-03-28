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

//! Settings domain Tauri commands.
//!
//! Covers: loading all settings, toggling individual settings,
//! session timeout, currency, language, sidebar, and reset.

use sanctum::controller::{
    AppController, SETTING_AUTO_FETCH, SETTING_CRYPTO_PROXY_ENABLED, SETTING_CRYPTO_PROXY_URL,
    SETTING_DARK_MODE, SETTING_PREFERRED_CURRENCY, SETTING_PREFERRED_LANGUAGE,
    SETTING_SESSION_TIMEOUT, SETTING_SIDEBAR_COLLAPSED,
};
use sanctum::ui::dto::settings::{AppInfo, AppSettings};
use std::sync::Arc;
use tauri::State;

/// Load all application settings at once.
///
/// Returns the full settings bundle for the frontend to initialize its state.
#[tauri::command]
pub fn load_settings(controller: State<'_, Arc<AppController>>) -> Result<AppSettings, String> {
    let dark_mode = controller
        .get_app_setting(SETTING_DARK_MODE)
        .map(|v| v != "false")
        .unwrap_or(true);

    let auto_fetch = controller
        .get_app_setting(SETTING_AUTO_FETCH)
        .map(|v| v == "true")
        .unwrap_or(false);

    let proxy_enabled = controller
        .get_app_setting(SETTING_CRYPTO_PROXY_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false);

    let proxy_url = controller
        .get_app_setting(SETTING_CRYPTO_PROXY_URL)
        .unwrap_or_default();

    let session_timeout_secs = controller
        .get_app_setting(SETTING_SESSION_TIMEOUT)
        .ok()
        .and_then(|s| s.parse::<i32>().ok())
        .map(|v| v.clamp(60, 3600))
        .unwrap_or(900);

    let preferred_currency = controller
        .get_app_setting(SETTING_PREFERRED_CURRENCY)
        .unwrap_or_else(|_| "USD".to_string());

    let preferred_language = controller
        .get_app_setting(SETTING_PREFERRED_LANGUAGE)
        .unwrap_or_else(|_| "en".to_string());

    let sidebar_collapsed = controller
        .get_app_setting(SETTING_SIDEBAR_COLLAPSED)
        .map(|v| v == "true")
        .unwrap_or(false);

    let login_wallpaper_path = controller
        .get_login_wallpaper_path()
        .map(|p| p.to_string_lossy().to_string());

    Ok(AppSettings {
        dark_mode,
        auto_fetch,
        proxy_enabled,
        proxy_url,
        session_timeout_secs,
        preferred_currency,
        preferred_language,
        sidebar_collapsed,
        login_wallpaper_path,
    })
}

/// Toggle dark mode on or off.
#[tauri::command]
pub fn set_dark_mode(
    controller: State<'_, Arc<AppController>>,
    enabled: bool,
) -> Result<(), String> {
    let val = if enabled { "true" } else { "false" };
    controller
        .set_app_setting(SETTING_DARK_MODE, val)
        .map_err(|e| e.to_string())
}

/// Toggle automatic crypto price fetching.
#[tauri::command]
pub fn set_auto_fetch(
    controller: State<'_, Arc<AppController>>,
    enabled: bool,
) -> Result<(), String> {
    let val = if enabled { "true" } else { "false" };
    controller
        .set_app_setting(SETTING_AUTO_FETCH, val)
        .map_err(|e| e.to_string())
}

/// Toggle crypto API proxy usage.
///
/// When enabling, validates the current proxy URL first.
#[tauri::command]
pub fn set_proxy_enabled(
    controller: State<'_, Arc<AppController>>,
    enabled: bool,
    current_url: String,
) -> Result<(), String> {
    if enabled {
        controller
            .validate_crypto_proxy_url(current_url)
            .map_err(|e| e.to_string())?;
    }
    controller
        .set_crypto_proxy_enabled(enabled)
        .map_err(|e| e.to_string())
}

/// Set the crypto API proxy URL.
#[tauri::command]
pub fn set_proxy_url(
    controller: State<'_, Arc<AppController>>,
    url: String,
) -> Result<(), String> {
    controller
        .set_crypto_proxy_url(url)
        .map_err(|e| e.to_string())
}

/// Set the vault auto-lock timeout in seconds (60–3600).
#[tauri::command]
pub fn set_session_timeout(
    controller: State<'_, Arc<AppController>>,
    timeout_secs: i32,
) -> Result<(), String> {
    let clamped = timeout_secs.clamp(60, 3600);
    controller
        .set_session_timeout(clamped as i64)
        .map_err(|e| e.to_string())
}

/// Change the preferred display currency.
#[tauri::command]
pub fn set_preferred_currency(
    controller: State<'_, Arc<AppController>>,
    currency: String,
) -> Result<(), String> {
    controller
        .set_app_setting(SETTING_PREFERRED_CURRENCY, &currency)
        .map_err(|e| e.to_string())
}

/// Change the preferred UI language and switch the i18n bundle.
#[tauri::command]
pub fn set_preferred_language(
    controller: State<'_, Arc<AppController>>,
    language: String,
) -> Result<(), String> {
    controller
        .set_app_setting(SETTING_PREFERRED_LANGUAGE, &language)
        .map_err(|e| e.to_string())?;
    sanctum::services::i18n::set_language(&language);
    Ok(())
}

/// Persist sidebar collapsed/expanded state.
#[tauri::command]
pub fn set_sidebar_collapsed(
    controller: State<'_, Arc<AppController>>,
    collapsed: bool,
) -> Result<(), String> {
    let val = if collapsed { "true" } else { "false" };
    controller
        .set_app_setting(SETTING_SIDEBAR_COLLAPSED, val)
        .map_err(|e| e.to_string())
}

/// Reset all settings to their default values.
#[tauri::command]
pub fn reset_settings(controller: State<'_, Arc<AppController>>) -> Result<(), String> {
    controller
        .set_app_setting(SETTING_DARK_MODE, "true")
        .map_err(|e| e.to_string())?;
    controller
        .set_app_setting(SETTING_AUTO_FETCH, "false")
        .map_err(|e| e.to_string())?;
    controller
        .set_app_setting(SETTING_CRYPTO_PROXY_ENABLED, "false")
        .map_err(|e| e.to_string())?;
    controller
        .set_app_setting(SETTING_CRYPTO_PROXY_URL, "")
        .map_err(|e| e.to_string())?;
    controller
        .set_app_setting(SETTING_SESSION_TIMEOUT, "900")
        .map_err(|e| e.to_string())?;
    controller
        .set_app_setting(SETTING_PREFERRED_CURRENCY, "USD")
        .map_err(|e| e.to_string())?;
    controller
        .set_app_setting(SETTING_PREFERRED_LANGUAGE, "en")
        .map_err(|e| e.to_string())?;
    controller
        .set_app_setting(SETTING_SIDEBAR_COLLAPSED, "false")
        .map_err(|e| e.to_string())?;
    controller
        .set_login_wallpaper_path(None)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get static application info for the About section.
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: "1.0.0".to_string(),
        encryption: "SQLCipher (AES-256)".to_string(),
        storage: "SQLite (encrypted)".to_string(),
    }
}

/// Get remaining session time before auto-lock, in seconds.
#[tauri::command]
pub fn get_session_remaining(
    controller: State<'_, Arc<AppController>>,
) -> Result<i64, String> {
    controller
        .get_session_remaining()
        .map_err(|e| e.to_string())
}

/// Returns all translation key-value pairs for the current language.
///
/// The frontend stores these in a reactive map and uses them for i18n.
#[tauri::command]
pub fn get_translations() -> std::collections::HashMap<String, String> {
    sanctum::services::i18n::get_all_translations()
}
