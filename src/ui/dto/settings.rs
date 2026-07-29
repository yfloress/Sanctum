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

//! Settings domain DTOs.
//!
//! Covers: all app settings (appearance, regional, security, data sync).

use serde::{Deserialize, Serialize};

/// All application settings bundled for initial load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub dark_mode: bool,
    pub auto_fetch: bool,
    pub proxy_enabled: bool,
    pub proxy_url: String,
    pub session_timeout_secs: i32,
    pub preferred_currency: String,
    pub preferred_language: String,
    pub sidebar_collapsed: bool,
    pub login_wallpaper_path: Option<String>,
    /// RFC 3339 timestamp of the last vault export, `None` if never exported.
    pub last_backup_at: Option<String>,
}

/// Input for updating a single boolean setting.
#[derive(Debug, Clone, Deserialize)]
pub struct BoolSettingInput {
    pub value: bool,
}

/// Input for updating a single string setting.
#[derive(Debug, Clone, Deserialize)]
pub struct StringSettingInput {
    pub value: String,
}

/// Input for updating session timeout.
#[derive(Debug, Clone, Deserialize)]
pub struct SessionTimeoutInput {
    pub timeout_secs: i32,
}

/// About/version info for the settings page.
#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub version: String,
    pub encryption: String,
    pub storage: String,
}
