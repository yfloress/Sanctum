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

//! Internationalization (i18n) service using Mozilla's Project Fluent
//!
//! Provides translation support with easy language switching.
//! Designed for extensibility - adding new languages requires only a new .ftl file.
//!
//! # Architecture
//! - Uses `concurrent::FluentBundle` for thread-safety
//! - Translations loaded from `locales/{lang}.ftl` files (embedded at compile time)
//! - Global state managed via `OnceLock` + `RwLock`
//!
//! # Adding a New Language
//! 1. Create `locales/{code}.ftl` (copy from en.ftl)
//! 2. Translate all messages
//! 3. Add to `get_ftl_content()` match
//! 4. Add language code to SUPPORTED_LANGUAGES
//! 5. Update settings UI with new option

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use std::sync::{OnceLock, RwLock};
use unic_langid::LanguageIdentifier;

/// Embedded locale files (compiled into binary)
const EN_FTL: &str = include_str!("../../locales/en.ftl");
const ES_FTL: &str = include_str!("../../locales/es.ftl");

/// Supported language codes
pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "es"];

/// Default language if detection fails
pub const DEFAULT_LANGUAGE: &str = "en";

/// Global i18n state
static I18N_STATE: OnceLock<RwLock<I18nState>> = OnceLock::new();

/// Returns FTL content for a language code
fn get_ftl_content(lang: &str) -> &'static str {
    match lang {
        "es" => ES_FTL,
        _ => EN_FTL,
    }
}

/// Thread-safe i18n state
struct I18nState {
    bundle: FluentBundle<FluentResource>,
    current_lang: String,
}

impl I18nState {
    /// Creates a new I18nState with the specified language
    fn new(lang: &str) -> Self {
        let lang_code = if SUPPORTED_LANGUAGES.contains(&lang) {
            lang
        } else {
            DEFAULT_LANGUAGE
        };

        let ftl_content = get_ftl_content(lang_code);
        let langid: LanguageIdentifier =
            lang_code.parse().unwrap_or_else(|_| "en".parse().unwrap());

        let resource = match FluentResource::try_new(ftl_content.to_string()) {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to parse FTL resource for '{}': {:?}", lang_code, e);
                FluentResource::try_new(String::new()).unwrap_or(FluentResource::try_new(" ".to_string()).unwrap())
            }
        };

        let mut bundle = FluentBundle::new_concurrent(vec![langid]);
        bundle.set_use_isolating(false);
        if let Err(e) = bundle.add_resource(resource) {
            log::error!("Failed to add FTL resource for '{}': {:?}", lang_code, e);
        }

        Self {
            bundle,
            current_lang: lang_code.to_string(),
        }
    }

    /// Gets a translated message by key
    fn get(&self, key: &str) -> String {
        self.get_with_args(key, None)
    }

    /// Gets a translated message with arguments
    fn get_with_args(&self, key: &str, args: Option<&FluentArgs>) -> String {
        let msg = match self.bundle.get_message(key) {
            Some(m) => m,
            None => {
                log::warn!("Missing translation key: {}", key);
                return key.to_string();
            }
        };

        let pattern = match msg.value() {
            Some(p) => p,
            None => return key.to_string(),
        };

        let mut errors = vec![];
        let result = self.bundle.format_pattern(pattern, args, &mut errors);

        if !errors.is_empty() {
            log::warn!("Fluent format errors for '{}': {:?}", key, errors);
        }

        result.to_string()
    }

    /// Returns current language code
    fn lang(&self) -> &str {
        &self.current_lang
    }
}

// ==================== Public API ====================

/// Initializes the i18n system with the given language
/// Should be called once at app startup
pub fn init(lang: &str) {
    let state = I18nState::new(lang);
    let _ = I18N_STATE.set(RwLock::new(state));
}

/// Switches to a different language
/// Returns true if language was changed successfully
pub fn set_language(lang: &str) -> bool {
    if !SUPPORTED_LANGUAGES.contains(&lang) {
        log::warn!("Unsupported language: {}", lang);
        return false;
    }

    if let Some(lock) = I18N_STATE.get()
        && let Ok(mut guard) = lock.write()
    {
        *guard = I18nState::new(lang);
        log::info!("Language changed to: {}", lang);
        return true;
    }
    false
}

/// Gets current language code
pub fn current_language() -> String {
    if let Some(lock) = I18N_STATE.get()
        && let Ok(guard) = lock.read()
    {
        return guard.lang().to_string();
    }
    DEFAULT_LANGUAGE.to_string()
}

/// Translates a message key
pub fn t(key: &str) -> String {
    if let Some(lock) = I18N_STATE.get()
        && let Ok(guard) = lock.read()
    {
        return guard.get(key);
    }
    // Fallback: return key itself
    key.to_string()
}

/// Translates a message key with arguments
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    if let Some(lock) = I18N_STATE.get()
        && let Ok(guard) = lock.read()
    {
        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            fluent_args.set(*k, (*v).to_string());
        }
        return guard.get_with_args(key, Some(&fluent_args));
    }
    key.to_string()
}

/// Returns all translation key-value pairs for the current language.
/// Keys with arguments are returned with their raw pattern (variables unresolved).
pub fn get_all_translations() -> std::collections::HashMap<String, String> {
    let lang = current_language();
    let ftl = get_ftl_content(&lang);
    let mut map = std::collections::HashMap::new();
    for line in ftl.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('.') {
            continue;
        }
        if let Some(eq_pos) = trimmed.find(" = ") {
            let key = &trimmed[..eq_pos];
            if key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                map.insert(key.to_string(), t(key));
            }
        }
    }
    map
}

/// Detects system language and returns best match
pub fn detect_system_language() -> String {
    if let Some(locale) = sys_locale::get_locale() {
        // Extract language code (e.g., "es-CL" -> "es")
        let lang = locale.split('-').next().unwrap_or(DEFAULT_LANGUAGE);
        let lang = lang.split('_').next().unwrap_or(DEFAULT_LANGUAGE);

        if SUPPORTED_LANGUAGES.contains(&lang) {
            return lang.to_string();
        }
    }
    DEFAULT_LANGUAGE.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests share global state (OnceLock), so we test behavior
    // rather than specific language state after init.

    #[test]
    fn test_init_and_translate() {
        init("en");
        // After init, t() should return a valid translation (not the key itself)
        let result = t("app-name");
        assert_eq!(result, "SANCTUM"); // Same in both languages
    }

    #[test]
    fn test_language_switch() {
        init("en");
        // Switch to Spanish and verify it works
        let switched = set_language("es");
        assert!(switched);

        let result = t("app-subtitle");
        assert_eq!(result, "Tu Fortaleza Financiera Personal");

        // Switch back to English
        set_language("en");
    }

    #[test]
    fn test_missing_key_returns_key() {
        init("en");
        let result = t("nonexistent-key-12345");
        assert_eq!(result, "nonexistent-key-12345");
    }

    #[test]
    fn test_detect_system_language() {
        // Should return a valid language code
        let lang = detect_system_language();
        assert!(SUPPORTED_LANGUAGES.contains(&lang.as_str()) || lang == DEFAULT_LANGUAGE);
    }

    #[test]
    fn test_unsupported_language_rejected() {
        init("en");
        let switched = set_language("xx");
        assert!(!switched); // Should fail for unsupported language
    }
}
