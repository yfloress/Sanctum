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

use serde::{Deserialize, Serialize};

// ==================== Habits System ====================

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

/// Represents a single habit completion log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitLog {
    pub id: String,
    pub habit_id: String,
    pub completed_date: String,
}

impl HabitLog {
    pub fn new(id: String, habit_id: String, completed_date: String) -> Self {
        Self {
            id,
            habit_id,
            completed_date,
        }
    }

    pub fn validate(&self) -> bool {
        !self.habit_id.is_empty() && !self.completed_date.is_empty()
    }
}
