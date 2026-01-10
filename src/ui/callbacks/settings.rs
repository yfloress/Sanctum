//! Settings callbacks
//!
//! Handles SettingsAdapter state and persistence.

use crate::controller::{
    AppController, SETTING_AUTO_FETCH, SETTING_CRYPTO_PROXY_ENABLED, SETTING_CRYPTO_PROXY_URL,
    SETTING_DARK_MODE, SETTING_PREFERRED_CURRENCY, SETTING_PREFERRED_LANGUAGE,
    SETTING_SESSION_TIMEOUT,
};
use crate::ui::callbacks::translations::{change_language, load_all_translations};
use crate::{AppState, AppWindow, CryptoAdapter, SettingsAdapter};
use slint::{ComponentHandle, SharedString, Weak};
use std::rc::Rc;
use std::sync::Arc;

/// Helper to start the auto-fetch timer (refreshes crypto prices every 60 seconds)
fn start_auto_fetch_timer(timer: &slint::Timer, ui_weak: Weak<AppWindow>) {
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(60),
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                // Only refresh if user is still logged in (vault open)
                if ui.global::<AppState>().get_is_logged_in() {
                    // Use silent refresh to avoid notification popups
                    ui.global::<CryptoAdapter>().invoke_refresh_prices_silent();
                }
            }
        },
    );
}

pub fn setup_settings_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // Shared timer for periodic crypto price refresh
    let auto_fetch_timer = Rc::new(slint::Timer::default());

    // Load settings
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let timer = auto_fetch_timer.clone();
        ui.global::<SettingsAdapter>().on_load_settings(move || {
            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(val) = controller.get_app_setting(SETTING_AUTO_FETCH) {
                    let enabled = val == "true";
                    ui.global::<SettingsAdapter>()
                        .set_auto_fetch_enabled(enabled);

                    if enabled {
                        // Start the periodic timer
                        start_auto_fetch_timer(&timer, ui.as_weak());

                        // Check if we need an immediate refresh (data older than 1 min)
                        let needs_update = if let Ok(Some((_, updated_at))) =
                            controller.load_exchange_rate_allow_stale("CLP_USD".to_string())
                        {
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&updated_at) {
                                let now = chrono::Utc::now();
                                let age = now
                                    .signed_duration_since(dt.with_timezone(&chrono::Utc))
                                    .num_minutes();
                                age >= 1
                            } else {
                                true
                            }
                        } else {
                            true
                        };

                        if needs_update {
                            ui.global::<CryptoAdapter>().invoke_refresh_prices_silent();
                        }
                    } else {
                        timer.stop();
                    }
                }

                let proxy_enabled = controller
                    .get_app_setting(SETTING_CRYPTO_PROXY_ENABLED)
                    .unwrap_or_default()
                    == "true";
                ui.global::<SettingsAdapter>()
                    .set_proxy_enabled(proxy_enabled);

                if let Ok(val) = controller.get_app_setting(SETTING_CRYPTO_PROXY_URL) {
                    ui.global::<SettingsAdapter>()
                        .set_proxy_url(SharedString::from(val));
                }

                // Load dark mode setting (default to true if not set)
                let dark_mode = controller
                    .get_app_setting(SETTING_DARK_MODE)
                    .map(|v| v != "false")
                    .unwrap_or(true);
                ui.global::<SettingsAdapter>().set_dark_mode(dark_mode);

                // Load session timeout setting (default to 15 min)
                let session_timeout = controller
                    .get_app_setting(SETTING_SESSION_TIMEOUT)
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(900);
                ui.global::<SettingsAdapter>()
                    .set_session_timeout(session_timeout);

                // Load preferred currency setting (default to USD)
                let currency = controller
                    .get_app_setting(SETTING_PREFERRED_CURRENCY)
                    .unwrap_or_else(|_| "USD".to_string());
                ui.global::<SettingsAdapter>()
                    .set_preferred_currency(SharedString::from(currency));

                // Load preferred language setting (default to English)
                let language = controller
                    .get_app_setting(SETTING_PREFERRED_LANGUAGE)
                    .unwrap_or_else(|_| "en".to_string());
                ui.global::<SettingsAdapter>()
                    .set_preferred_language(SharedString::from(language.clone()));

                // Apply saved language to i18n and reload translations
                // This ensures the UI updates to the user's saved preference after login
                if change_language(&language) {
                    load_all_translations(&ui);
                }
            }
        });
    }

    // Dark mode toggle
    {
        let controller = controller.clone();
        ui.global::<SettingsAdapter>()
            .on_set_dark_mode(move |enabled| {
                let val = if enabled { "true" } else { "false" };
                let _ = controller.set_app_setting(SETTING_DARK_MODE, val);
            });
    }

    // Auto-fetch toggle
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let timer = auto_fetch_timer.clone();
        ui.global::<SettingsAdapter>()
            .on_set_auto_fetch(move |enabled| {
                let val = if enabled { "true" } else { "false" };
                let _ = controller.set_app_setting(SETTING_AUTO_FETCH, val);

                if enabled {
                    // Start periodic timer and trigger immediate silent refresh
                    if let Some(ui) = ui_weak.upgrade() {
                        start_auto_fetch_timer(&timer, ui.as_weak());
                        ui.global::<CryptoAdapter>().invoke_refresh_prices_silent();
                    }
                } else {
                    timer.stop();
                }
            });
    }

    // Proxy enabled toggle
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<SettingsAdapter>()
            .on_set_proxy_enabled(move |enabled| {
                if enabled {
                    let current_url = ui_weak
                        .upgrade()
                        .map(|ui| ui.global::<SettingsAdapter>().get_proxy_url().to_string())
                        .unwrap_or_default();

                    if let Err(err) = controller.validate_crypto_proxy_url(current_url) {
                        notify(err.to_string(), true);
                    }
                }

                if let Err(err) = controller.set_crypto_proxy_enabled(enabled) {
                    notify(err.to_string(), true);
                }
            });
    }

    // Proxy URL update
    {
        let controller = controller.clone();
        let notify = notify.clone();
        ui.global::<SettingsAdapter>().on_set_proxy_url(move |url| {
            if let Err(err) = controller.set_crypto_proxy_url(url.to_string()) {
                notify(err.to_string(), true);
            }
        });
    }

    // Session timeout update
    {
        let controller = controller.clone();
        let notify = notify.clone();
        ui.global::<SettingsAdapter>()
            .on_set_session_timeout(move |timeout_secs| {
                let timeout = timeout_secs as i64;
                if let Err(err) = controller.set_session_timeout(timeout) {
                    notify(err.to_string(), true);
                } else {
                    notify(
                        "Session timeout updated. Will apply on next vault open.".into(),
                        false,
                    );
                }
            });
    }

    // Preferred currency update (UI only - no backend yet)
    {
        let controller = controller.clone();
        ui.global::<SettingsAdapter>()
            .on_set_preferred_currency(move |currency| {
                let _ = controller.set_app_setting(SETTING_PREFERRED_CURRENCY, currency.as_str());
            });
    }

    // Preferred language update - changes language and reloads translations
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<SettingsAdapter>()
            .on_set_preferred_language(move |language| {
                let _ = controller.set_app_setting(SETTING_PREFERRED_LANGUAGE, language.as_str());

                // Change language in i18n service and reload translations
                if change_language(language.as_str())
                    && let Some(ui) = ui_weak.upgrade() {
                        load_all_translations(&ui);
                    }
            });
    }

    // Reset all settings to defaults
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<SettingsAdapter>().on_reset_settings(move || {
            // Reset to defaults
            let _ = controller.set_app_setting(SETTING_DARK_MODE, "true");
            let _ = controller.set_app_setting(SETTING_AUTO_FETCH, "false");
            let _ = controller.set_app_setting(SETTING_CRYPTO_PROXY_ENABLED, "false");
            let _ = controller.set_app_setting(SETTING_CRYPTO_PROXY_URL, "");
            let _ = controller.set_app_setting(SETTING_SESSION_TIMEOUT, "900"); // 15 min
            let _ = controller.set_app_setting(SETTING_PREFERRED_CURRENCY, "USD");
            let _ = controller.set_app_setting(SETTING_PREFERRED_LANGUAGE, "en");

            // Reset language to English
            let _ = change_language("en");

            // Reload settings and translations
            if let Some(ui) = ui_weak.upgrade() {
                ui.global::<SettingsAdapter>().invoke_load_settings();
                load_all_translations(&ui);
            }

            notify("Settings reset to defaults".into(), false);
        });
    }
}
