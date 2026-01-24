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

//! Rewards data loading functions
//!
//! This module handles the transformation of backend data into UI-ready formats.
//! It serves as the bridge between the controller layer and the Slint UI adapters.
//!
//! ## Architecture
//!
//! The rewards system follows a layered pattern:
//! - `db/rewards.rs` - Raw SQL operations
//! - `rewards_repository.rs` - Database wrapper (consistent with project patterns)
//! - `rewards_service.rs` - Business logic (validation, auto-completion, achievements)
//! - `controller/rewards.rs` - Input validation and orchestration
//! - `rewards.rs` - Callback registration (UI event handlers)
//! - `rewards_data.rs` (this file) - Data transformation for UI
//!
//! ## Data Flow
//!
//! 1. UI triggers a callback (e.g., `fetch_rewards`)
//! 2. Callback schedules a deferred load via `Timer::single_shot`
//! 3. This module's functions query the controller and transform data
//! 4. Transformed data is set on the appropriate Slint adapter
//!
//! ## Key Patterns
//!
//! - **Progress calculation**: Streak rewards calculate progress relative to the
//!   maximum milestone, not the next one, ensuring consistent progress bar behavior.
//! - **Color parsing**: Habit colors are stored as hex strings and parsed into
//!   Slint `Color` values with a fallback to purple (#8b5cf6).

use crate::controller::AppController;
use crate::models::Habit;
use crate::{
    AchievementData, AppWindow, CheckpointData, GoalData, MilestoneData, RewardsAdapter,
    StreakRewardData,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::sync::Arc;

// ==================== Data Loading Functions ====================

/// Loads streak rewards data into the UI.
///
/// This function:
/// 1. Fetches all streak rewards from the database
/// 2. Enriches each reward with habit name/color and milestone data
/// 3. Calculates progress percentage relative to the maximum milestone
/// 4. Sets the data on `RewardsAdapter.streak_rewards`
pub fn load_rewards_data<N>(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, notify: &N)
where
    N: Fn(String, bool) + Clone + 'static,
{
    let Some(ui) = ui_weak.upgrade() else {
        return;
    };

    let habits: Vec<Habit> = controller.get_habits().unwrap_or_default();

    let rewards = match controller.get_streak_rewards() {
        Ok(r) => r,
        Err(e) => {
            notify(format!("Failed to load rewards: {}", e), true);
            return;
        }
    };

    let mut reward_data: Vec<StreakRewardData> = Vec::with_capacity(rewards.len());

    for reward in rewards {
        let habit = habits.iter().find(|h| h.id == reward.habit_id);
        let habit_name = habit.map(|h| h.name.clone()).unwrap_or_default();
        let habit_color = habit
            .map(|h| parse_hex_color(&h.color))
            .unwrap_or_else(default_color);

        let milestones = controller
            .get_milestones(reward.id.clone())
            .unwrap_or_default();

        let progress = controller
            .get_streak_progress(reward.id.clone())
            .unwrap_or(0);

        let next_milestone = milestones
            .iter()
            .filter(|m| !m.unlocked)
            .min_by_key(|m| m.target_days);

        let next_days = next_milestone.map(|m| m.target_days).unwrap_or(0);
        let next_reward = next_milestone
            .map(|m| m.reward_text.clone())
            .unwrap_or_default();

        // Calculate progress relative to the MAX milestone (last one), not the next one
        // This ensures the progress bar and milestone markers use the same reference
        let max_milestone_days = milestones.iter().map(|m| m.target_days).max().unwrap_or(1);

        let progress_percent = calculate_progress_percent(progress, max_milestone_days);

        let milestone_data: Vec<MilestoneData> = milestones
            .iter()
            .map(|m| MilestoneData {
                id: SharedString::from(&m.id),
                target_days: m.target_days,
                reward_text: SharedString::from(&m.reward_text),
                unlocked: m.unlocked,
                unlocked_at: SharedString::from(m.unlocked_at.as_deref().unwrap_or("")),
            })
            .collect();

        reward_data.push(StreakRewardData {
            id: SharedString::from(&reward.id),
            habit_id: SharedString::from(&reward.habit_id),
            habit_name: SharedString::from(&habit_name),
            habit_color,
            is_consecutive: reward.is_consecutive,
            target_days: reward.target_days.unwrap_or(0),
            target_total: reward.target_total.unwrap_or(0),
            current_progress: progress,
            milestones: ModelRc::new(VecModel::from(milestone_data)),
            next_milestone_days: next_days,
            next_milestone_reward: SharedString::from(&next_reward),
            progress_percent,
        });
    }

    ui.global::<RewardsAdapter>()
        .set_streak_rewards(ModelRc::new(VecModel::from(reward_data)));
}

/// Loads goals data into the UI.
///
/// This function:
/// 1. Fetches all active (non-archived) goals
/// 2. Loads checkpoints for each goal
/// 3. Calculates completion progress (completed_count / total_count)
/// 4. Sets the data on `RewardsAdapter.goals`
///
/// Note: Archived goals are filtered out by the database query.
pub fn load_goals_data<N>(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, notify: &N)
where
    N: Fn(String, bool) + Clone + 'static,
{
    let Some(ui) = ui_weak.upgrade() else {
        return;
    };

    let goals = match controller.get_goals() {
        Ok(g) => g,
        Err(e) => {
            notify(format!("Failed to load goals: {}", e), true);
            return;
        }
    };

    let mut goal_data: Vec<GoalData> = Vec::with_capacity(goals.len());

    for goal in goals {
        let checkpoints = controller
            .get_checkpoints(goal.id.clone())
            .unwrap_or_default();

        let completed_count = checkpoints.iter().filter(|c| c.completed).count() as i32;
        let total_count = checkpoints.len() as i32;
        let progress_percent = if total_count > 0 {
            completed_count as f32 / total_count as f32 * 100.0
        } else {
            0.0
        };

        let checkpoint_data: Vec<CheckpointData> = checkpoints
            .iter()
            .map(|c| CheckpointData {
                id: SharedString::from(&c.id),
                description: SharedString::from(&c.description),
                completed: c.completed,
                completed_at: SharedString::from(c.completed_at.as_deref().unwrap_or("")),
                sort_order: c.sort_order,
            })
            .collect();

        goal_data.push(GoalData {
            id: SharedString::from(&goal.id),
            name: SharedString::from(&goal.name),
            description: SharedString::from(goal.description.as_deref().unwrap_or("")),
            reward_text: SharedString::from(&goal.reward_text),
            deadline: SharedString::from(goal.deadline.as_deref().unwrap_or("")),
            checkpoints: ModelRc::new(VecModel::from(checkpoint_data)),
            completed_count,
            total_count,
            progress_percent,
            is_completed: goal.is_completed,
            completed_at: SharedString::from(goal.completed_at.as_deref().unwrap_or("")),
        });
    }

    ui.global::<RewardsAdapter>()
        .set_goals(ModelRc::new(VecModel::from(goal_data)));
}

/// Loads achievements data into the UI.
///
/// Achievements are created automatically when:
/// - A streak reward milestone is unlocked
/// - A goal is completed (either manually or when all checkpoints are done)
///
/// The `achieved_at` timestamp is truncated to YYYY-MM-DD for display.
pub fn load_achievements_data<N>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: &N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    let Some(ui) = ui_weak.upgrade() else {
        return;
    };

    let achievements = match controller.get_achievements() {
        Ok(a) => a,
        Err(e) => {
            notify(format!("Failed to load achievements: {}", e), true);
            return;
        }
    };

    let achievement_data: Vec<AchievementData> = achievements
        .iter()
        .map(|a| {
            // Format date as YYYY-MM-DD (take first 10 chars from RFC3339)
            let short_date = if a.achieved_at.len() >= 10 {
                &a.achieved_at[..10]
            } else {
                &a.achieved_at
            };
            AchievementData {
                id: SharedString::from(&a.id),
                title: SharedString::from(&a.title),
                description: SharedString::from(&a.description),
                icon: slint::Image::default(),
                achieved_at: SharedString::from(short_date),
                achievement_type: SharedString::from(&a.achievement_type),
            }
        })
        .collect();

    ui.global::<RewardsAdapter>()
        .set_achievements(ModelRc::new(VecModel::from(achievement_data)));
}

// ==================== Utility Functions ====================

/// Parses a hex color string (e.g., "#8b5cf6") into a Slint Color
pub fn parse_hex_color(hex: &str) -> slint::Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return default_color();
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(139);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(92);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(246);

    slint::Color::from_rgb_u8(r, g, b)
}

/// Returns the default purple color used when parsing fails
pub fn default_color() -> slint::Color {
    slint::Color::from_rgb_u8(139, 92, 246)
}

/// Calculates progress percentage, capped at 100%
pub fn calculate_progress_percent(progress: i32, target: i32) -> f32 {
    if target > 0 {
        (progress as f32 / target as f32 * 100.0).min(100.0)
    } else {
        100.0
    }
}
