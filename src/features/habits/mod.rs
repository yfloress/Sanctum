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

//! Habits feature module
//!
//! Handles habit tracking, logs, streaks, and rewards.

pub mod repository;
pub mod rewards_repository;
pub mod rewards_service;
pub mod service;

pub use repository::HabitsRepository;
pub use rewards_repository::RewardsRepository;
pub use rewards_service::RewardsService;
pub use service::HabitService;
