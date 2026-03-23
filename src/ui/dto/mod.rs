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

//! Data Transfer Objects for the Tauri IPC boundary.
//!
//! These structs define the contract between the Rust backend and the Svelte frontend.
//! They are serialized to JSON via serde. Each DTO maps 1:1 with what the frontend
//! needs — no internal types are exposed.

pub mod charts;
pub mod crypto;
pub mod dashboard;
pub mod finance;
pub mod habits;
pub mod ingestion;
pub mod settings;
pub mod vault;
