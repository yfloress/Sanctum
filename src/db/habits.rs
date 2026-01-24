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

//! Habits database operations
//!
//! CRUD operations for habits and habit logs.

use super::{Database, DbError};
use crate::models::{Habit, HabitLog};
use rusqlite::params;

impl Database {
    // ==================== Habits CRUD ====================

    /// Creates a new habit
    pub fn create_habit(&self, habit: &Habit) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO habits (id, name, description, color, category, created_at, archived)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &habit.id,
                &habit.name,
                &habit.description,
                &habit.color,
                &habit.category,
                &habit.created_at,
                habit.archived as i32,
            ],
        )?;
        Ok(())
    }

    /// Gets all active (non-archived) habits
    pub fn get_habits(&self) -> Result<Vec<Habit>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, color, category, created_at, archived
             FROM habits
             WHERE archived = 0
             ORDER BY created_at ASC",
        )?;

        let habits = stmt
            .query_map([], |row| {
                Ok(Habit {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    category: row.get(4)?,
                    created_at: row.get(5)?,
                    archived: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(habits)
    }

    /// Gets a single habit by ID
    pub fn get_habit(&self, id: &str) -> Result<Option<Habit>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, name, description, color, category, created_at, archived
             FROM habits WHERE id = ?1",
            params![id],
            |row| {
                Ok(Habit {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    category: row.get(4)?,
                    created_at: row.get(5)?,
                    archived: row.get::<_, i32>(6)? != 0,
                })
            },
        );

        match result {
            Ok(habit) => Ok(Some(habit)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Updates an existing habit
    pub fn update_habit(&self, habit: &Habit) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE habits SET name = ?1, description = ?2, color = ?3, category = ?4 WHERE id = ?5",
            params![
                &habit.name,
                &habit.description,
                &habit.color,
                &habit.category,
                &habit.id
            ],
        )?;
        Ok(())
    }

    /// Archives a habit (soft delete)
    pub fn archive_habit(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("UPDATE habits SET archived = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Permanently deletes a habit and all its logs
    pub fn delete_habit(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM habits WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== Habit Logs CRUD ====================

    /// Creates a habit completion log
    pub fn create_habit_log(&self, log: &HabitLog) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO habit_logs (id, habit_id, completed_date)
             VALUES (?1, ?2, ?3)",
            params![&log.id, &log.habit_id, &log.completed_date],
        )?;
        Ok(())
    }

    /// Deletes a habit log (uncomplete)
    pub fn delete_habit_log(&self, habit_id: &str, date: &str) -> Result<bool, DbError> {
        let rows = self.conn.execute(
            "DELETE FROM habit_logs WHERE habit_id = ?1 AND completed_date = ?2",
            params![habit_id, date],
        )?;
        Ok(rows > 0)
    }

    /// Checks if a habit log exists for a given date
    pub fn habit_log_exists(&self, habit_id: &str, date: &str) -> Result<bool, DbError> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM habit_logs WHERE habit_id = ?1 AND completed_date = ?2",
            params![habit_id, date],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Gets all habit logs within a date range
    pub fn get_habit_logs(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<HabitLog>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, habit_id, completed_date
             FROM habit_logs
             WHERE completed_date >= ?1 AND completed_date <= ?2
             ORDER BY completed_date ASC",
        )?;

        let logs = stmt
            .query_map(params![start_date, end_date], |row| {
                Ok(HabitLog {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    completed_date: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Toggles a habit completion for a specific date
    pub fn toggle_habit_log(
        &self,
        habit_id: &str,
        date: &str,
    ) -> Result<(bool, Option<String>), DbError> {
        if self.habit_log_exists(habit_id, date)? {
            self.delete_habit_log(habit_id, date)?;
            Ok((false, None))
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            let log = HabitLog::new(id.clone(), habit_id.to_string(), date.to_string());
            self.create_habit_log(&log)?;
            Ok((true, Some(id)))
        }
    }
}
