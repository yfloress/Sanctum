// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
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

use sanctum::error::AppError;
use sanctum::features::crypto::{
    CryptoService, SETTING_AUTO_FETCH, SETTING_CRYPTO_PROXY_ENABLED, SETTING_CRYPTO_PROXY_URL,
};
use sanctum::features::settings::{
    SETTING_DARK_MODE, SETTING_PREFERRED_CURRENCY, SETTING_PREFERRED_LANGUAGE,
    SETTING_SESSION_TIMEOUT, SETTING_SIDEBAR_COLLAPSED, SettingsService,
};
use sanctum::ui::dto::settings::{AppInfo, AppSettings};
use sanctum::vault_manager::VaultManager;
use tauri::State;

/// Load all application settings at once.
///
/// Returns the full settings bundle for the frontend to initialize its state.
#[tauri::command]
pub fn load_settings(
    settings: State<'_, SettingsService>,
    vault: State<'_, VaultManager>,
) -> Result<AppSettings, AppError> {
    let dark_mode = settings
        .get_app_setting(SETTING_DARK_MODE)
        .map(|v| v != "false")
        .unwrap_or(true);

    let auto_fetch = settings
        .get_app_setting(SETTING_AUTO_FETCH)
        .map(|v| v == "true")
        .unwrap_or(false);

    let proxy_enabled = settings
        .get_app_setting(SETTING_CRYPTO_PROXY_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false);

    let proxy_url = settings
        .get_app_setting(SETTING_CRYPTO_PROXY_URL)
        .unwrap_or_default();

    let session_timeout_secs = settings
        .get_app_setting(SETTING_SESSION_TIMEOUT)
        .ok()
        .and_then(|s| s.parse::<i32>().ok())
        .map(|v| v.clamp(60, 3600))
        .unwrap_or(900);

    let preferred_currency = settings
        .get_app_setting(SETTING_PREFERRED_CURRENCY)
        .unwrap_or_else(|_| "USD".to_string());

    let preferred_language = settings
        .get_app_setting(SETTING_PREFERRED_LANGUAGE)
        .unwrap_or_else(|_| "en".to_string());

    // Switch the i18n bundle to the user's saved preference
    sanctum::services::i18n::set_language(&preferred_language);

    let sidebar_collapsed = settings
        .get_app_setting(SETTING_SIDEBAR_COLLAPSED)
        .map(|v| v == "true")
        .unwrap_or(false);

    let login_wallpaper_path = vault
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
pub fn set_dark_mode(settings: State<'_, SettingsService>, enabled: bool) -> Result<(), AppError> {
    let val = if enabled { "true" } else { "false" };
    Ok(settings.set_app_setting(SETTING_DARK_MODE, val)?)
}

/// Toggle automatic crypto price fetching.
#[tauri::command]
pub fn set_auto_fetch(settings: State<'_, SettingsService>, enabled: bool) -> Result<(), AppError> {
    let val = if enabled { "true" } else { "false" };
    Ok(settings.set_app_setting(SETTING_AUTO_FETCH, val)?)
}

/// Toggle crypto API proxy usage.
///
/// When enabling, validates the current proxy URL first.
#[tauri::command]
pub fn set_proxy_enabled(
    crypto: State<'_, CryptoService>,
    enabled: bool,
    current_url: String,
) -> Result<(), AppError> {
    if enabled {
        crypto.validate_proxy_url(&current_url)?;
    }
    Ok(crypto.set_proxy_enabled(enabled)?)
}

/// Set the crypto API proxy URL.
///
/// Non-empty URLs are validated before persisting so an enabled proxy never
/// silently stores an unreachable / malformed value. Empty strings clear the
/// setting and are allowed through without validation.
#[tauri::command]
pub fn set_proxy_url(crypto: State<'_, CryptoService>, url: String) -> Result<(), AppError> {
    let trimmed = url.trim().to_string();
    if !trimmed.is_empty() {
        crypto.validate_proxy_url(&trimmed)?;
    }
    Ok(crypto.set_proxy_url(trimmed)?)
}

/// Set the vault auto-lock timeout in seconds (60–3600).
#[tauri::command]
pub fn set_session_timeout(
    settings: State<'_, SettingsService>,
    timeout_secs: i32,
) -> Result<(), AppError> {
    let clamped = timeout_secs.clamp(60, 3600);
    Ok(settings.set_app_setting(SETTING_SESSION_TIMEOUT, &clamped.to_string())?)
}

/// Change the preferred display currency.
#[tauri::command]
pub fn set_preferred_currency(
    settings: State<'_, SettingsService>,
    currency: String,
) -> Result<(), AppError> {
    Ok(settings.set_app_setting(SETTING_PREFERRED_CURRENCY, &currency)?)
}

/// Change the preferred UI language and switch the i18n bundle.
#[tauri::command]
pub fn set_preferred_language(
    settings: State<'_, SettingsService>,
    language: String,
) -> Result<(), AppError> {
    settings.set_app_setting(SETTING_PREFERRED_LANGUAGE, &language)?;
    sanctum::services::i18n::set_language(&language);
    Ok(())
}

/// Persist sidebar collapsed/expanded state.
#[tauri::command]
pub fn set_sidebar_collapsed(
    settings: State<'_, SettingsService>,
    collapsed: bool,
) -> Result<(), AppError> {
    let val = if collapsed { "true" } else { "false" };
    Ok(settings.set_app_setting(SETTING_SIDEBAR_COLLAPSED, val)?)
}

/// Reset all settings to their default values.
#[tauri::command]
pub fn reset_settings(
    settings: State<'_, SettingsService>,
    vault: State<'_, VaultManager>,
) -> Result<(), AppError> {
    let defaults = [
        (SETTING_DARK_MODE, "true"),
        (SETTING_AUTO_FETCH, "false"),
        (SETTING_CRYPTO_PROXY_ENABLED, "false"),
        (SETTING_CRYPTO_PROXY_URL, ""),
        (SETTING_SESSION_TIMEOUT, "900"),
        (SETTING_PREFERRED_CURRENCY, "USD"),
        (SETTING_PREFERRED_LANGUAGE, "en"),
        (SETTING_SIDEBAR_COLLAPSED, "false"),
    ];
    for (key, value) in defaults {
        settings.set_app_setting(key, value)?;
    }
    vault.set_login_wallpaper_path(None)?;

    Ok(())
}

/// Get static application info for the About section.
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        encryption: "SQLCipher (AES-256)".to_string(),
        storage: "SQLite (encrypted)".to_string(),
    }
}

/// Get remaining session time before auto-lock, in seconds.
#[tauri::command]
pub fn get_session_remaining(vault: State<'_, VaultManager>) -> Result<i64, AppError> {
    Ok(vault.get_session_remaining()?)
}

/// Returns all translation key-value pairs for the current language.
///
/// The frontend stores these in a reactive map and uses them for i18n.
#[tauri::command]
pub fn get_translations() -> std::collections::HashMap<String, String> {
    sanctum::services::i18n::get_all_translations()
}

/// Set the login wallpaper image path.
#[tauri::command]
pub fn set_login_wallpaper(
    vault: State<'_, VaultManager>,
    path: Option<String>,
) -> Result<(), AppError> {
    Ok(vault.set_login_wallpaper_path(path.map(std::path::PathBuf::from))?)
}
