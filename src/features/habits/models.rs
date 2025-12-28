//! Habits domain models
//!
//! Contains Habit and HabitLog types.

use serde::{Deserialize, Serialize};

// ==================== Habit ====================

/// Represents a habit to track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub category: String,
    pub created_at: String,
    pub archived: bool,
}

impl Habit {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        created_at: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            color,
            category,
            created_at,
            archived: false,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.trim().is_empty()
            && self.color.starts_with('#')
            && self.color.len() == 7
            && !self.category.trim().is_empty()
    }
}

// ==================== Habit Log ====================

/// Represents a single habit completion log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitLog {
    pub id: String,
    pub habit_id: String,
    pub completed_date: String,
}

impl HabitLog {
    pub fn new(id: String, habit_id: String, completed_date: String) -> Self {
        Self { id, habit_id, completed_date }
    }

    pub fn validate(&self) -> bool {
        !self.habit_id.is_empty() && !self.completed_date.is_empty()
    }
}
