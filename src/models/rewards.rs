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

// ==================== Rewards System ====================

/// Represents a streak-based reward linked to a habit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakReward {
    pub id: String,
    pub habit_id: String,
    pub is_consecutive: bool,
    pub target_days: Option<i32>,
    pub target_total: Option<i32>,
    pub created_at: String,
}

impl StreakReward {
    pub fn new(
        id: String,
        habit_id: String,
        is_consecutive: bool,
        target_days: Option<i32>,
        target_total: Option<i32>,
    ) -> Self {
        Self {
            id,
            habit_id,
            is_consecutive,
            target_days,
            target_total,
            created_at: chrono::Local::now().to_rfc3339(),
        }
    }

    pub fn validate(&self) -> bool {
        !self.habit_id.is_empty()
            && (self.is_consecutive || (self.target_days.is_some() && self.target_total.is_some()))
    }
}

/// Represents a milestone within a streak reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub reward_id: String,
    pub target_days: i32,
    pub reward_text: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}

impl Milestone {
    pub fn new(id: String, reward_id: String, target_days: i32, reward_text: String) -> Self {
        Self {
            id,
            reward_id,
            target_days,
            reward_text,
            unlocked: false,
            unlocked_at: None,
        }
    }

    pub fn validate(&self) -> bool {
        !self.reward_id.is_empty() && self.target_days > 0 && !self.reward_text.trim().is_empty()
    }

    pub fn unlock(&mut self) {
        self.unlocked = true;
        self.unlocked_at = Some(chrono::Local::now().to_rfc3339());
    }
}

/// Represents an independent goal with checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub reward_text: String,
    pub deadline: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub archived: bool,
}

impl Goal {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        reward_text: String,
        deadline: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            reward_text,
            deadline,
            is_completed: false,
            completed_at: None,
            created_at: chrono::Local::now().to_rfc3339(),
            archived: false,
        }
    }

    pub fn validate(&self) -> bool {
        !self.name.trim().is_empty() && !self.reward_text.trim().is_empty()
    }

    pub fn complete(&mut self) {
        self.is_completed = true;
        self.completed_at = Some(chrono::Local::now().to_rfc3339());
    }

    pub fn archive(&mut self) {
        self.archived = true;
    }
}

/// Represents a checkpoint within a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub goal_id: String,
    pub description: String,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub sort_order: i32,
}

impl Checkpoint {
    pub fn new(id: String, goal_id: String, description: String, sort_order: i32) -> Self {
        Self {
            id,
            goal_id,
            description,
            completed: false,
            completed_at: None,
            sort_order,
        }
    }

    pub fn validate(&self) -> bool {
        !self.goal_id.is_empty() && !self.description.trim().is_empty()
    }

    pub fn toggle(&mut self) -> bool {
        self.completed = !self.completed;
        self.completed_at = if self.completed {
            Some(chrono::Local::now().to_rfc3339())
        } else {
            None
        };
        self.completed
    }
}

/// Represents an unlocked achievement (trophy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_path: String,
    pub achievement_type: String,
    pub source_id: String,
    pub achieved_at: String,
}

impl Achievement {
    pub fn new(
        id: String,
        title: String,
        description: String,
        icon_path: String,
        achievement_type: String,
        source_id: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            icon_path,
            achievement_type,
            source_id,
            achieved_at: chrono::Local::now().to_rfc3339(),
        }
    }

    pub fn validate(&self) -> bool {
        !self.title.trim().is_empty()
            && !self.source_id.is_empty()
            && (self.achievement_type == "streak" || self.achievement_type == "goal")
    }
}
