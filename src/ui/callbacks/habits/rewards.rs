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

//! Rewards callback registrations
//!
//! This module connects UI callbacks from `RewardsAdapter` to the backend controller.
//! It handles user interactions for streak rewards, goals, checkpoints, and achievements.
//!
//! ## Module Organization
//!
//! Callbacks are organized into logical groups:
//! - **Fetch callbacks**: Load data on demand (`fetch_rewards`, `fetch_goals`, `fetch_achievements`)
//! - **Streak reward callbacks**: CRUD operations for streak rewards and milestones
//! - **Goal callbacks**: CRUD operations for goals including completion and archiving
//! - **Checkpoint callbacks**: Add, delete, and toggle checkpoint completion
//!
//! ## Deferred Updates Pattern
//!
//! All callbacks that modify data use `Timer::single_shot` to schedule UI updates.
//! This prevents recursion issues during Slint's initialization phase and ensures
//! the UI reflects the latest state after backend operations complete.
//!
//! ## Error Handling
//!
//! Callbacks return `SharedString` where empty string indicates success and
//! non-empty string contains an error message. The `notify` function is used
//! to display user-facing messages (success or error).

use crate::controller::AppController;
use crate::{AppWindow, RewardsAdapter};
use slint::{ComponentHandle, SharedString, Timer, Weak};
use std::sync::Arc;
use std::time::Duration;

// Import data loading functions from the separate module
use super::rewards_data::{load_achievements_data, load_goals_data, load_rewards_data};

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

// ==================== Fetch Callbacks ====================

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

// ==================== Streak Reward Callbacks ====================

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
                        schedule_rewards_refresh(&ui_weak, &controller, &notify, "Reward created");
                        SharedString::from(id)
                    }
                    Err(e) => {
                        notify(format!("Failed to create reward: {}", e), true);
                        SharedString::from("")
                    }
                }
            },
        );
    }

    // on_update_streak_reward
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        let ui_weak_inner = ui_weak.clone();
        ui.global::<RewardsAdapter>().on_update_streak_reward(
            move |id: SharedString,
                  habit_id: SharedString,
                  is_consecutive: bool,
                  target_days: i32,
                  target_total: i32|
                  -> SharedString {
                // Read milestones from adapter properties
                let milestones = if let Some(ui) = ui_weak_inner.upgrade() {
                    let adapter = ui.global::<RewardsAdapter>();
                    let milestone_count = adapter.get_edit_reward_milestone_count();
                    let mut ms = Vec::new();

                    if milestone_count >= 1 {
                        let days = adapter.get_edit_reward_m1_days();
                        let text = adapter.get_edit_reward_m1_text();
                        if days > 0 && !text.is_empty() {
                            ms.push((days, text.to_string()));
                        }
                    }
                    if milestone_count >= 2 {
                        let days = adapter.get_edit_reward_m2_days();
                        let text = adapter.get_edit_reward_m2_text();
                        if days > 0 && !text.is_empty() {
                            ms.push((days, text.to_string()));
                        }
                    }
                    if milestone_count >= 3 {
                        let days = adapter.get_edit_reward_m3_days();
                        let text = adapter.get_edit_reward_m3_text();
                        if days > 0 && !text.is_empty() {
                            ms.push((days, text.to_string()));
                        }
                    }
                    ms
                } else {
                    Vec::new()
                };

                let result = controller.update_streak_reward_with_milestones(
                    id.to_string(),
                    habit_id.to_string(),
                    is_consecutive,
                    target_days,
                    target_total,
                    milestones,
                );

                match result {
                    Ok(_) => {
                        schedule_rewards_refresh(&ui_weak, &controller, &notify, "Reward updated");
                        SharedString::from("")
                    }
                    Err(e) => {
                        notify(format!("Failed to update reward: {}", e), true);
                        SharedString::from(e.to_string())
                    }
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
                        schedule_rewards_refresh(&ui_weak, &controller, &notify, "Reward deleted");
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
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
}

// ==================== Goal Callbacks ====================

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
                        SharedString::from(id)
                    }
                    Err(e) => {
                        notify(format!("Failed to create goal: {}", e), true);
                        SharedString::from("")
                    }
                }
            },
        );
    }

    // on_update_goal
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_update_goal(
            move |id: SharedString,
                  name: SharedString,
                  description: SharedString,
                  reward_text: SharedString,
                  deadline: SharedString|
                  -> SharedString {
                let result = controller.update_goal(
                    id.to_string(),
                    name.to_string(),
                    description.to_string(),
                    reward_text.to_string(),
                    deadline.to_string(),
                );
                match result {
                    Ok(_) => {
                        schedule_goals_refresh(&ui_weak, &controller, &notify, "Goal updated");
                        SharedString::from("")
                    }
                    Err(e) => {
                        notify(format!("Failed to update goal: {}", e), true);
                        SharedString::from(e.to_string())
                    }
                }
            },
        );
    }

    // on_update_goal_with_checkpoints
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_update_goal_with_checkpoints(
            move |id: SharedString,
                  name: SharedString,
                  description: SharedString,
                  reward_text: SharedString,
                  deadline: SharedString,
                  checkpoint_count: i32,
                  cp1_id: SharedString,
                  cp1_text: SharedString,
                  cp2_id: SharedString,
                  cp2_text: SharedString,
                  cp3_id: SharedString,
                  cp3_text: SharedString,
                  cp4_id: SharedString,
                  cp4_text: SharedString|
                  -> SharedString {
                let result = controller.update_goal_with_checkpoints(
                    id.to_string(),
                    name.to_string(),
                    description.to_string(),
                    reward_text.to_string(),
                    deadline.to_string(),
                    checkpoint_count,
                    cp1_id.to_string(),
                    cp1_text.to_string(),
                    cp2_id.to_string(),
                    cp2_text.to_string(),
                    cp3_id.to_string(),
                    cp3_text.to_string(),
                    cp4_id.to_string(),
                    cp4_text.to_string(),
                );

                match result {
                    Ok(_) => {
                        schedule_goals_refresh(&ui_weak, &controller, &notify, "Goal updated");
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
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
                            notify("Goal completed!".into(), false);
                        });
                        SharedString::from("")
                    }
                    Ok(None) => SharedString::from("Goal not found or already completed"),
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_archive_goal
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>()
            .on_archive_goal(move |id: SharedString| -> SharedString {
                let result = controller.archive_goal(id.to_string());
                match result {
                    Ok(_) => {
                        let ui_weak = ui_weak.clone();
                        let controller = controller.clone();
                        let notify = notify.clone();
                        Timer::single_shot(Duration::from_millis(UI_UPDATE_DELAY_MS), move || {
                            load_goals_data(&ui_weak, &controller, &notify);
                            notify("Goal archived".into(), false);
                        });
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }
}

// ==================== Checkpoint Callbacks ====================

fn setup_checkpoint_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // on_delete_checkpoint
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_delete_checkpoint(
            move |checkpoint_id: SharedString| -> SharedString {
                let result = controller.delete_checkpoint(checkpoint_id.to_string());
                match result {
                    Ok(_) => {
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

    // on_update_checkpoint
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();
        ui.global::<RewardsAdapter>().on_update_checkpoint(
            move |checkpoint_id: SharedString, description: SharedString| -> SharedString {
                let result =
                    controller.update_checkpoint(checkpoint_id.to_string(), description.to_string());
                match result {
                    Ok(_) => {
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
