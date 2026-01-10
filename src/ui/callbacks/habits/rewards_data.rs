//! Rewards data loading functions
//!
//! Contains functions for loading rewards, goals, and achievements data into the UI.

use crate::controller::AppController;
use crate::models::Habit;
use crate::{
    AchievementData, AppWindow, CheckpointData, GoalData, MilestoneData, RewardsAdapter,
    StreakRewardData,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::sync::Arc;

// ==================== Data Loading Functions ====================

/// Loads streak rewards data into the UI
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

/// Loads goals data into the UI
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

/// Loads achievements data into the UI
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
