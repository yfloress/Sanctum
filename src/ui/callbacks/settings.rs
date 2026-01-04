//! Settings callbacks
//!
//! Handles SettingsAdapter state and persistence.

use crate::controller::{
    AppController, SETTING_AUTO_FETCH, SETTING_CRYPTO_PROXY_ENABLED, SETTING_CRYPTO_PROXY_URL,
    SETTING_DARK_MODE,
};
use crate::{AppState, CryptoAdapter, SettingsAdapter, AppWindow};
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
                    ui.global::<CryptoAdapter>().invoke_refresh_prices();
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
                            ui.global::<CryptoAdapter>().invoke_refresh_prices();
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
                    // Start periodic timer and trigger immediate refresh
                    if let Some(ui) = ui_weak.upgrade() {
                        start_auto_fetch_timer(&timer, ui.as_weak());
                        ui.global::<CryptoAdapter>().invoke_refresh_prices();
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
        ui.global::<SettingsAdapter>()
            .on_set_proxy_url(move |url| {
                if let Err(err) = controller.set_crypto_proxy_url(url.to_string()) {
                    notify(err.to_string(), true);
                }
            });
    }
}
