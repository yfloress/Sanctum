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

//! UI Callback Setup Modules
//!
//! Domain-specific callback registration for Slint UI.
//! Each module exports a `setup_*_callbacks()` function.

pub mod crypto;
pub mod dashboard;
pub mod finance;
pub mod habits;
pub mod ingestion;
pub mod settings;
pub mod translations;
pub mod vault;

pub use crypto::*;
pub use dashboard::*;
pub use finance::*;
pub use habits::*;
pub use ingestion::*;
pub use settings::*;
pub use translations::*;
pub use vault::*;
