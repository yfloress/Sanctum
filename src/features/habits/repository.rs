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
