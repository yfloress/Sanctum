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

//! Habits repository
//!
//! Database operations for habits feature.

use crate::db::{Database, DbError};
// Use original models for compatibility with db.rs
use crate::models::{Habit, HabitLog};

/// Repository for habit-related database operations
pub struct HabitsRepository;

impl HabitsRepository {
    pub fn create_habit(db: &Database, habit: &Habit) -> Result<(), DbError> {
        db.create_habit(habit)
    }

    pub fn get_habits(db: &Database) -> Result<Vec<Habit>, DbError> {
        db.get_habits()
    }

    pub fn get_habit(db: &Database, id: &str) -> Result<Option<Habit>, DbError> {
        db.get_habit(id)
    }

    pub fn update_habit(db: &Database, habit: &Habit) -> Result<(), DbError> {
        db.update_habit(habit)
    }

    pub fn archive_habit(db: &Database, id: &str) -> Result<(), DbError> {
        db.archive_habit(id)
    }

    pub fn delete_habit(db: &Database, id: &str) -> Result<(), DbError> {
        db.delete_habit(id)
    }

    pub fn toggle_habit_log(db: &Database, habit_id: &str, date: &str) -> Result<(bool, Option<String>), DbError> {
        db.toggle_habit_log(habit_id, date)
    }

    pub fn get_habit_logs(db: &Database, start_date: &str, end_date: &str) -> Result<Vec<HabitLog>, DbError> {
        db.get_habit_logs(start_date, end_date)
    }
}
