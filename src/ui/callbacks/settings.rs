//! Settings callbacks
//!
//! Handles SettingsAdapter state and persistence.

use crate::controller::{
    AppController, SETTING_AUTO_FETCH, SETTING_CRYPTO_PROXY_ENABLED, SETTING_CRYPTO_PROXY_URL,
};
use crate::{CryptoAdapter, SettingsAdapter, AppWindow};
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;

pub fn setup_settings_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // Load settings
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<SettingsAdapter>().on_load_settings(move || {
            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(val) = controller.get_app_setting(SETTING_AUTO_FETCH) {
                    let enabled = val == "true";
                    ui.global::<SettingsAdapter>()
                        .set_auto_fetch_enabled(enabled);

                    if enabled {
                        let needs_update = if let Ok(Some((_, updated_at))) =
                            controller.load_exchange_rate_allow_stale("CLP_USD".to_string())
                        {
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&updated_at) {
                                let now = chrono::Utc::now();
                                let age = now
                                    .signed_duration_since(dt.with_timezone(&chrono::Utc))
                                    .num_minutes();
                                age > 10
                            } else {
                                true
                            }
                        } else {
                            true
                        };

                        if needs_update {
                            ui.global::<CryptoAdapter>().invoke_refresh_prices();
                        }
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
            }
        });
    }

    // Auto-fetch toggle
    {
        let controller = controller.clone();
        ui.global::<SettingsAdapter>()
            .on_set_auto_fetch(move |enabled| {
                let val = if enabled { "true" } else { "false" };
                let _ = controller.set_app_setting(SETTING_AUTO_FETCH, val);
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
