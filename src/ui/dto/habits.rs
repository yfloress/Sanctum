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

//! Habits domain DTOs.
//!
//! Covers: habits, daily tracking, heatmap, analytics, rewards, goals, achievements.

use serde::{Deserialize, Serialize};

// ==================== Habits ====================

/// Habit as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct HabitDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub category: String,
    /// Completion status per day of the month (1-indexed, day 0 unused).
    pub days: Vec<bool>,
}

/// Habits list for a given month.
#[derive(Debug, Clone, Serialize)]
pub struct HabitsResponse {
    pub habits: Vec<HabitDto>,
    pub month: i32,
    pub year: i32,
    pub days_in_month: i32,
}

/// Input for creating or updating a habit.
#[derive(Debug, Clone, Deserialize)]
pub struct HabitInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub category: String,
}

/// Input for toggling habit completion.
#[derive(Debug, Clone, Deserialize)]
pub struct HabitToggleInput {
    pub habit_id: String,
    pub date: String,
}

// ==================== Habit Summary ====================

/// Selected habit summary stats.
#[derive(Debug, Clone, Serialize)]
pub struct HabitSummary {
    pub habit_id: String,
    pub current_streak: i32,
    pub best_streak: i32,
    pub completion_rate: f64,
    pub last_30_days: i32,
    pub best_day: Option<String>,
}

// ==================== Heatmap ====================

/// Heatmap data for one year (365/366 days).
#[derive(Debug, Clone, Serialize)]
pub struct HeatmapResponse {
    pub year: i32,
    pub data: Vec<HeatmapDay>,
}

/// Single day in the heatmap.
#[derive(Debug, Clone, Serialize)]
pub struct HeatmapDay {
    pub date: String,
    pub intensity: i32, // 0-4
}

// ==================== Analytics ====================

/// Habit analytics chart data for ECharts.
#[derive(Debug, Clone, Serialize)]
pub struct HabitAnalyticsResponse {
    pub radar: RadarChartData,
    pub weekday_efficiency: WeekdayChartData,
    pub weekly_summary: String,
    pub insight: String,
}

/// Radar chart data (habit analytics).
#[derive(Debug, Clone, Serialize)]
pub struct RadarChartData {
    pub categories: Vec<String>,
    pub values: Vec<f64>,
    pub max_value: f64,
}

/// Weekday efficiency chart data.
#[derive(Debug, Clone, Serialize)]
pub struct WeekdayChartData {
    /// Labels: ["Mon", "Tue", ..., "Sun"]
    pub labels: Vec<String>,
    /// Completion rates per weekday (0.0 - 1.0).
    pub values: Vec<f64>,
}

// ==================== Rewards ====================

/// Streak reward as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct StreakRewardDto {
    pub id: String,
    pub habit_id: String,
    pub habit_name: String,
    pub is_consecutive: bool,
    pub target_days: Option<i32>,
    pub target_total: Option<i32>,
    pub current_progress: i32,
    pub milestones: Vec<MilestoneDto>,
}

/// Milestone within a streak reward.
#[derive(Debug, Clone, Serialize)]
pub struct MilestoneDto {
    pub id: String,
    pub target_days: i32,
    pub reward_text: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}

/// Input for creating or updating a streak reward.
#[derive(Debug, Clone, Deserialize)]
pub struct StreakRewardInput {
    pub id: Option<String>,
    pub habit_id: String,
    pub is_consecutive: bool,
    pub target_days: Option<i32>,
    pub target_total: Option<i32>,
    pub milestones: Vec<MilestoneInput>,
}

/// Input for a milestone.
#[derive(Debug, Clone, Deserialize)]
pub struct MilestoneInput {
    pub id: Option<String>,
    pub target_days: i32,
    pub reward_text: String,
}

// ==================== Goals ====================

/// Goal as seen by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct GoalDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub reward_text: String,
    pub deadline: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub checkpoints: Vec<CheckpointDto>,
}

/// Checkpoint within a goal.
#[derive(Debug, Clone, Serialize)]
pub struct CheckpointDto {
    pub id: String,
    pub description: String,
    pub completed: bool,
    pub completed_at: Option<String>,
}

/// Input for creating or updating a goal.
#[derive(Debug, Clone, Deserialize)]
pub struct GoalInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub reward_text: String,
    pub deadline: Option<String>,
    pub checkpoints: Vec<CheckpointInput>,
}

/// Input for a checkpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckpointInput {
    pub id: Option<String>,
    pub description: String,
}

// ==================== Achievements ====================

/// Achievement (completed goal or unlocked streak milestone).
#[derive(Debug, Clone, Serialize)]
pub struct AchievementDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_path: String,
    pub achievement_type: String,
    pub achieved_at: String,
}
