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

//! Habits service
//!
//! Business logic for habit tracking.

use crate::db::{Database, DbError};
// Use shared models from the central domain layer
use crate::models::{Habit, HabitLog};
use super::repository::HabitsRepository;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct HabitService {
    db: Arc<Mutex<Option<Database>>>,
}

impl HabitService {
    pub fn new(db: Arc<Mutex<Option<Database>>>) -> Self {
        Self { db }
    }

    fn with_db<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Database) -> Result<T, DbError>,
    {
        let guard = self.db.lock().map_err(|_| DbError::InvalidPassword)?;
        if let Some(db) = guard.as_ref() {
            f(db)
        } else {
            Err(DbError::InvalidPassword)
        }
    }

    pub fn create_habit(
        &self,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Local::now().to_rfc3339();

        let habit = Habit {
            id: id.clone(),
            name,
            description,
            color,
            category,
            created_at: now,
            archived: false,
        };

        self.with_db(|db| {
            HabitsRepository::create_habit(db, &habit)?;
            Ok(())
        })?;

        Ok(id)
    }

    pub fn get_habits(&self) -> Result<Vec<Habit>, DbError> {
        self.with_db(HabitsRepository::get_habits)
    }

    pub fn update_habit(
        &self,
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        is_archived: bool,
    ) -> Result<(), DbError> {
        self.with_db(|db| {
            match HabitsRepository::get_habit(db, &id)? {
                Some(mut habit) => {
                    habit.name = name;
                    habit.description = description;
                    habit.color = color;
                    habit.category = category;

                    HabitsRepository::update_habit(db, &habit)?;

                    if is_archived {
                        HabitsRepository::archive_habit(db, &id)?;
                    }
                    Ok(())
                }
                None => Err(DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows)),
            }
        })
    }

    pub fn archive_habit(&self, id: String) -> Result<(), DbError> {
        self.with_db(|db| HabitsRepository::archive_habit(db, &id))
    }

    pub fn delete_habit(&self, id: String) -> Result<(), DbError> {
        self.with_db(|db| HabitsRepository::delete_habit(db, &id))
    }

    pub fn toggle_habit_completion(&self, habit_id: String, date: String) -> Result<bool, DbError> {
        let (active, _id) = self.with_db(|db| HabitsRepository::toggle_habit_log(db, &habit_id, &date))?;
        Ok(active)
    }

    pub fn get_habit_logs(&self, start_date: String, end_date: String) -> Result<Vec<HabitLog>, DbError> {
        self.with_db(|db| HabitsRepository::get_habit_logs(db, &start_date, &end_date))
    }
}
