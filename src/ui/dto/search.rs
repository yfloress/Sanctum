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

//! IPC types for the global search box.

use serde::{Deserialize, Serialize};

use crate::services::search::HitKind;

/// What the caller is searching for.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchInput {
    pub query: String,
    /// Falls back to the service default when absent, and is clamped there too.
    pub limit: Option<usize>,
}

/// One row of the result list.
///
/// `kind` and `id` are all the frontend needs to navigate: the page that owns
/// that kind knows how to open one of its own by id.
#[derive(Debug, Clone, Serialize)]
pub struct SearchHitDto {
    pub kind: HitKind,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    /// Set for hits that live inside an account, so the activity list can be
    /// narrowed to the right one before the row is looked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}
