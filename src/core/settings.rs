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

//! Application-level settings constants.
//!
//! These keys are used across multiple feature modules and should not live
//! inside any single domain (crypto, finance, etc.).

/// Whether dark mode is enabled (`"true"` / `"false"`).
pub const SETTING_DARK_MODE: &str = "dark_mode";

/// RFC 3339 timestamp of the last successful vault export.
pub const SETTING_LAST_BACKUP_AT: &str = "last_backup_at";

/// Session inactivity timeout in seconds (e.g. `"900"`).
pub const SETTING_SESSION_TIMEOUT: &str = "session_timeout";

/// Preferred display currency code (e.g. `"USD"`, `"CLP"`).
pub const SETTING_PREFERRED_CURRENCY: &str = "preferred_currency";

/// Preferred UI language (e.g. `"en"`, `"es"`).
pub const SETTING_PREFERRED_LANGUAGE: &str = "preferred_language";

/// Whether the sidebar is collapsed (`"true"` / `"false"`).
pub const SETTING_SIDEBAR_COLLAPSED: &str = "sidebar_collapsed";
