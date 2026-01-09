//! Rewards callback registrations
//!
//! Connects RewardsAdapter callbacks to the backend controller.

use crate::controller::AppController;
use crate::models::Habit;
use crate::{
    AchievementData, AppWindow, CheckpointData, GoalData, MilestoneData, RewardsAdapter,
    StreakRewardData,
};
use slint::{ComponentHandle, ModelRc, SharedString, Timer, VecModel, Weak};
use std::sync::Arc;
use std::time::Duration;

/// Delay for deferred UI updates to avoid recursion during initialization
const UI_UPDATE_DELAY_MS: u64 = 100;

/// Sets up all RewardsAdapter callbacks
pub fn setup_rewards_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    setup_fetch_callbacks(ui, ui_weak, controller, notify.clone());
    setup_streak_reward_callbacks(ui, ui_weak, controller, notify.clone());
    setup_goal_callbacks(ui, ui_weak, controller, notify.clone());
    setup_checkpoint_callbacks(ui, ui_weak, controller, notify);
}

fn setup_fetch_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // on_fetch_rewards
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_fetch_rewards(move || {
            let ui_weak = ui_weak.clone();
            let controller = controller.clone();
            let notify = notify.clone();
            Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
                if ui_weak.upgrade().is_some() {
                    load_rewards_data(&ui_weak, &controller, &notify);
                }
            });
        });
    }

    // on_fetch_goals
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_fetch_goals(move || {
            let ui_weak = ui_weak.clone();
            let controller = controller.clone();
            let notify = notify.clone();
            Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
                if ui_weak.upgrade().is_some() {
                    load_goals_data(&ui_weak, &controller, &notify);
                }
            });
        });
    }

    // on_fetch_achievements
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>()
            .on_fetch_achievements(move || {
                let ui_weak = ui_weak.clone();
                let controller = controller.clone();
                let notify = notify.clone();
                Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
                    if ui_weak.upgrade().is_some() {
                        load_achievements_data(&ui_weak, &controller, &notify);
                    }
                });
            });
    }
}

fn setup_streak_reward_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // on_create_streak_reward
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_create_streak_reward(
            move |habit_id: SharedString,
                  is_consecutive: bool,
                  target_days: i32,
                  target_total: i32|
                  -> SharedString {
                let result = controller.create_streak_reward(
                    habit_id.to_string(),
                    is_consecutive,
                    target_days,
                    target_total,
                );
                match result {
                    Ok(id) => {
                        schedule_rewards_refresh(
                            &ui_weak,
                            &controller,
                            &notify,
                            "Streak reward created",
                        );
                        // Return the reward ID on success
                        SharedString::from(id)
                    }
                    Err(e) => {
                        // Show error notification and return empty string
                        notify(format!("Failed to create reward: {}", e), true);
                        SharedString::from("")
                    }
                }
            },
        );
    }

    // on_add_milestone
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_add_milestone(
            move |reward_id: SharedString,
                  target_days: i32,
                  reward_text: SharedString|
                  -> SharedString {
                let result = controller.add_milestone(
                    reward_id.to_string(),
                    target_days,
                    reward_text.to_string(),
                );
                match result {
                    Ok(_id) => {
                        schedule_rewards_refresh(&ui_weak, &controller, &notify, "Milestone added");
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_delete_streak_reward
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_delete_streak_reward(
            move |id: SharedString| -> SharedString {
                let result = controller.delete_streak_reward(id.to_string());
                match result {
                    Ok(_) => {
                        schedule_rewards_refresh(
                            &ui_weak,
                            &controller,
                            &notify,
                            "Streak reward deleted",
                        );
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_update_streak_progress
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>()
            .on_update_streak_progress(move |reward_id: SharedString| {
                let reward_id_str = reward_id.to_string();
                let ui_weak = ui_weak.clone();
                let controller = controller.clone();
                let notify = notify.clone();
                Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
                    handle_streak_progress_update(&ui_weak, &controller, &notify, &reward_id_str);
                });
            });
    }
}

fn setup_goal_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // on_create_goal
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_create_goal(
            move |name: SharedString,
                  description: SharedString,
                  reward_text: SharedString,
                  deadline: SharedString|
                  -> SharedString {
                let result = controller.create_goal(
                    name.to_string(),
                    description.to_string(),
                    reward_text.to_string(),
                    deadline.to_string(),
                );
                match result {
                    Ok(id) => {
                        schedule_goals_refresh(&ui_weak, &controller, &notify, "Goal created");
                        // Return the goal ID on success
                        SharedString::from(id)
                    }
                    Err(e) => {
                        // Show error notification and return empty string
                        notify(format!("Failed to create goal: {}", e), true);
                        SharedString::from("")
                    }
                }
            },
        );
    }

    // on_delete_goal
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>()
            .on_delete_goal(move |id: SharedString| -> SharedString {
                let result = controller.delete_goal(id.to_string());
                match result {
                    Ok(_) => {
                        schedule_goals_refresh(&ui_weak, &controller, &notify, "Goal deleted");
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_complete_goal
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>()
            .on_complete_goal(move |id: SharedString| -> SharedString {
                let result = controller.complete_goal(id.to_string());
                match result {
                    Ok(Some(_)) => {
                        let ui_weak = ui_weak.clone();
                        let controller = controller.clone();
                        let notify = notify.clone();
                        Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
                            load_goals_data(&ui_weak, &controller, &notify);
                            load_achievements_data(&ui_weak, &controller, &notify);
                            notify("🎉 Goal completed!".into(), false);
                        });
                        SharedString::from("")
                    }
                    Ok(None) => SharedString::from("Goal not found or already completed"),
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }
}

fn setup_checkpoint_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // on_add_checkpoint
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_add_checkpoint(
            move |goal_id: SharedString, description: SharedString| -> SharedString {
                let result =
                    controller.add_checkpoint(goal_id.to_string(), description.to_string());
                match result {
                    Ok(_id) => {
                        let ui_weak = ui_weak.clone();
                        let controller = controller.clone();
                        let notify = notify.clone();
                        Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
                            load_goals_data(&ui_weak, &controller, &notify);
                        });
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_toggle_checkpoint
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_toggle_checkpoint(
            move |goal_id: SharedString, checkpoint_id: SharedString| -> SharedString {
                let result =
                    controller.toggle_checkpoint(goal_id.to_string(), checkpoint_id.to_string());
                match result {
                    Ok(_) => {
                        let ui_weak = ui_weak.clone();
                        let controller = controller.clone();
                        let notify = notify.clone();
                        Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
                            load_goals_data(&ui_weak, &controller, &notify);
                            load_achievements_data(&ui_weak, &controller, &notify);
                        });
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }
}

// ==================== Helper Functions ====================

fn schedule_rewards_refresh<N>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: &N,
    message: &str,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    let ui_weak = ui_weak.clone();
    let controller = controller.clone();
    let notify = notify.clone();
    let msg = message.to_string();
    Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
        if ui_weak.upgrade().is_some() {
            load_rewards_data(&ui_weak, &controller, &notify);
            notify(msg, false);
        }
    });
}

fn schedule_goals_refresh<N>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: &N,
    message: &str,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    let ui_weak = ui_weak.clone();
    let controller = controller.clone();
    let notify = notify.clone();
    let msg = message.to_string();
    Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
        if ui_weak.upgrade().is_some() {
            load_goals_data(&ui_weak, &controller, &notify);
            notify(msg, false);
        }
    });
}

fn handle_streak_progress_update<N>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: &N,
    reward_id: &str,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    if ui_weak.upgrade().is_none() {
        return;
    }

    match controller.check_and_unlock_milestones(reward_id.to_string()) {
        Ok(unlocked) => {
            if !unlocked.is_empty() {
                notify(
                    format!("🎉 {} milestone(s) unlocked!", unlocked.len()),
                    false,
                );
                load_achievements_data(ui_weak, controller, notify);
            }
            load_rewards_data(ui_weak, controller, notify);
        }
        Err(e) => {
            notify(format!("Failed to update progress: {}", e), true);
        }
    }
}

// ==================== Data Loading Functions ====================

fn load_rewards_data<N>(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, notify: &N)
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

        let progress_percent = calculate_progress_percent(progress, next_days);

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

fn load_goals_data<N>(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, notify: &N)
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

fn load_achievements_data<N>(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, notify: &N)
where
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
            let icon = load_achievement_icon(&a.icon_path);
            AchievementData {
                id: SharedString::from(&a.id),
                title: SharedString::from(&a.title),
                description: SharedString::from(&a.description),
                icon,
                achieved_at: SharedString::from(&a.achieved_at),
                achievement_type: SharedString::from(&a.achievement_type),
            }
        })
        .collect();

    ui.global::<RewardsAdapter>()
        .set_achievements(ModelRc::new(VecModel::from(achievement_data)));
}

// ==================== Utility Functions ====================

fn parse_hex_color(hex: &str) -> slint::Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return default_color();
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(139);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(92);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(246);

    slint::Color::from_rgb_u8(r, g, b)
}

fn default_color() -> slint::Color {
    slint::Color::from_rgb_u8(139, 92, 246) // Purple
}

fn calculate_progress_percent(progress: i32, target: i32) -> f32 {
    if target > 0 {
        (progress as f32 / target as f32 * 100.0).min(100.0)
    } else {
        100.0
    }
}

fn load_achievement_icon(icon_path: &str) -> slint::Image {
    let path = format!("ui/assets/icons/{}", icon_path);
    slint::Image::load_from_path(std::path::Path::new(&path)).unwrap_or_default()
}
