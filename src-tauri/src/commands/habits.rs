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

//! Habits domain Tauri commands.
//!
//! Covers: habits CRUD, toggle, heatmap, analytics, streak rewards,
//! goals, checkpoints, and achievements.

use chrono::{Datelike, NaiveDate};
use sanctum::controller::AppController;
use sanctum::ui::dto::habits::{
    AchievementDto, CheckpointDto, GoalDto, HabitAnalyticsResponse, HabitDto, HabitSummary,
    HabitsResponse, HeatmapDay, HeatmapResponse, MilestoneDto, RadarChartData,
    StreakRewardDto, WeekdayChartData,
};
use std::sync::Arc;
use tauri::State;

// ==================== Habits CRUD ====================

/// Fetch all habits for a given month/year.
#[tauri::command]
pub fn fetch_habits(
    controller: State<'_, Arc<AppController>>,
    month: i32,
    year: i32,
) -> Result<HabitsResponse, String> {
    let date = NaiveDate::from_ymd_opt(year, month as u32, 1)
        .ok_or_else(|| "Invalid month/year".to_string())?;

    let habits = controller.get_habits().map_err(|e| e.to_string())?;

    // Calculate days in month
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1)
    };
    let days_in_month = next_month
        .map(|d| d.pred_opt().unwrap_or(d).day())
        .unwrap_or(30) as i32;

    // Get habit logs for the month
    let start_date = date.format("%Y-%m-%d").to_string();
    let end_date = NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { (month + 1) as u32 },
        1,
    )
    .and_then(|d| d.pred_opt())
    .map(|d| d.format("%Y-%m-%d").to_string())
    .unwrap_or_else(|| format!("{year}-{month:02}-{days_in_month:02}"));

    let logs = controller
        .get_habit_logs(start_date, end_date)
        .unwrap_or_default();

    let habit_dtos: Vec<HabitDto> = habits
        .into_iter()
        .filter(|h| !h.archived)
        .map(|h| {
            let mut days = vec![false; (days_in_month + 1) as usize];
            for log in &logs {
                if log.habit_id == h.id
                    && let Ok(d) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d")
                {
                    let day = d.day() as usize;
                    if day < days.len() {
                        days[day] = true;
                    }
                }
            }
            HabitDto {
                id: h.id,
                name: h.name,
                description: h.description,
                color: h.color,
                category: h.category,
                days,
            }
        })
        .collect();

    Ok(HabitsResponse {
        habits: habit_dtos,
        month,
        year,
        days_in_month,
    })
}

/// Create a new habit.
#[tauri::command]
pub fn create_habit(
    controller: State<'_, Arc<AppController>>,
    name: String,
    description: Option<String>,
    color: String,
    category: String,
) -> Result<String, String> {
    controller
        .create_habit(name, description, color, category)
        .map_err(|e| e.to_string())
}

/// Update an existing habit.
#[tauri::command]
pub fn update_habit(
    controller: State<'_, Arc<AppController>>,
    id: String,
    name: String,
    description: Option<String>,
    color: String,
    category: String,
) -> Result<(), String> {
    controller
        .update_habit(id, name, description, color, category, false)
        .map_err(|e| e.to_string())
}

/// Delete a habit.
#[tauri::command]
pub fn delete_habit(
    controller: State<'_, Arc<AppController>>,
    id: String,
) -> Result<(), String> {
    controller.delete_habit(id).map_err(|e| e.to_string())
}

/// Toggle habit completion for a specific date.
#[tauri::command]
pub fn toggle_habit(
    controller: State<'_, Arc<AppController>>,
    habit_id: String,
    date: String,
) -> Result<(), String> {
    controller
        .toggle_habit_completion(habit_id.clone(), date)
        .map_err(|e| e.to_string())?;

    // Check and unlock milestones for streak rewards linked to this habit
    if let Ok(rewards) = controller.get_streak_rewards_by_habit(&habit_id) {
        for reward in rewards {
            let _ = controller.check_and_unlock_milestones(reward.id);
        }
    }

    Ok(())
}

/// Fetch habit summary stats (streaks, completion rate).
#[tauri::command]
pub fn fetch_habit_summary(
    controller: State<'_, Arc<AppController>>,
    habit_id: String,
) -> Result<HabitSummary, String> {
    // Calculate streaks from habit logs
    let logs = controller.get_all_habit_logs().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().date_naive();

    // Filter logs for this habit
    let mut dates: Vec<NaiveDate> = logs
        .iter()
        .filter(|l| l.habit_id == habit_id)
        .filter_map(|l| NaiveDate::parse_from_str(&l.completed_date, "%Y-%m-%d").ok())
        .collect();
    dates.sort();
    dates.dedup();

    // Calculate current streak
    let mut current_streak = 0i32;
    let mut cursor = today;
    loop {
        if dates.binary_search(&cursor).is_ok() {
            current_streak += 1;
            cursor = match cursor.pred_opt() {
                Some(d) => d,
                None => break,
            };
        } else {
            break;
        }
    }

    // Calculate best streak
    let mut best_streak = 0i32;
    let mut streak = 0i32;
    let mut prev: Option<NaiveDate> = None;
    for d in &dates {
        if let Some(p) = prev {
            if *d == p.succ_opt().unwrap_or(p) {
                streak += 1;
            } else {
                streak = 1;
            }
        } else {
            streak = 1;
        }
        if streak > best_streak {
            best_streak = streak;
        }
        prev = Some(*d);
    }

    // Last 30 days count
    let thirty_days_ago = today - chrono::Duration::days(30);
    let last_30 = dates.iter().filter(|d| **d >= thirty_days_ago).count() as i32;

    // Completion rate (last 30 days)
    let completion_rate = last_30 as f64 / 30.0;

    Ok(HabitSummary {
        habit_id,
        current_streak,
        best_streak,
        completion_rate,
        last_30_days: last_30,
        best_day: None,
    })
}

// ==================== Heatmap ====================

/// Fetch heatmap data for a given year.
#[tauri::command]
pub fn fetch_heatmap(
    controller: State<'_, Arc<AppController>>,
    year: i32,
) -> Result<HeatmapResponse, String> {
    let start = format!("{year}-01-01");
    let end = format!("{year}-12-31");

    let habits = controller.get_habits().map_err(|e| e.to_string())?;
    let habit_count = habits.iter().filter(|h| !h.archived).count() as f64;
    let logs = controller
        .get_habit_logs(start, end)
        .map_err(|e| e.to_string())?;

    // Count completions per date
    let mut day_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for log in &logs {
        *day_counts.entry(log.completed_date.clone()).or_insert(0) += 1;
    }

    // Build days list
    let mut days: Vec<HeatmapDay> = Vec::new();
    if let Some(mut cursor) = NaiveDate::from_ymd_opt(year, 1, 1) {
        let year_end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or(cursor);
        while cursor <= year_end {
            let date_str = cursor.format("%Y-%m-%d").to_string();
            let count = day_counts.get(&date_str).copied().unwrap_or(0);
            let intensity = if habit_count > 0.0 {
                let ratio = count as f64 / habit_count;
                if ratio == 0.0 { 0 }
                else if ratio <= 0.25 { 1 }
                else if ratio <= 0.5 { 2 }
                else if ratio <= 0.75 { 3 }
                else { 4 }
            } else {
                0
            };
            days.push(HeatmapDay { date: date_str, intensity });
            cursor = match cursor.succ_opt() {
                Some(d) => d,
                None => break,
            };
        }
    }

    Ok(HeatmapResponse { year, data: days })
}

// ==================== Analytics ====================

/// Fetch habit analytics data (radar chart, weekday efficiency).
#[tauri::command]
pub fn fetch_habit_analytics(
    controller: State<'_, Arc<AppController>>,
    days: Option<i32>,
) -> Result<HabitAnalyticsResponse, String> {
    let analytics = controller
        .get_habit_analytics(days.unwrap_or(90))
        .map_err(|e| e.to_string())?;

    // Map weekday data to DTO
    let weekday_labels: Vec<String> = analytics.weekday_data.iter().map(|w| w.day_short.clone()).collect();
    let weekday_values: Vec<f64> = analytics.weekday_data.iter().map(|w| w.avg_count as f64).collect();

    // Build radar from category data (categories = habit categories, values = completion counts)
    let radar_categories: Vec<String> = analytics.category_data.iter().map(|c| c.category.clone()).collect();
    let radar_values: Vec<f64> = analytics.category_data.iter().map(|c| c.count as f64).collect();
    let radar_max = radar_values.iter().cloned().fold(0.0_f64, f64::max);

    // Generate summary text
    let best_day = analytics.weekday_data.iter().find(|w| w.is_best).map(|w| w.day_name.clone());
    let weekly_summary = match &best_day {
        Some(day) => format!("Your best day is {day}"),
        None => "No data yet".to_string(),
    };

    Ok(HabitAnalyticsResponse {
        radar: RadarChartData {
            categories: radar_categories,
            values: radar_values,
            max_value: radar_max,
        },
        weekday_efficiency: WeekdayChartData {
            labels: weekday_labels,
            values: weekday_values,
        },
        weekly_summary,
        insight: String::new(),
    })
}

// ==================== Streak Rewards ====================

/// Fetch all streak rewards.
#[tauri::command]
pub fn fetch_rewards(
    controller: State<'_, Arc<AppController>>,
) -> Result<Vec<StreakRewardDto>, String> {
    let rewards = controller.get_streak_rewards().map_err(|e| e.to_string())?;
    let habits = controller.get_habits().map_err(|e| e.to_string())?;
    let habit_names: std::collections::HashMap<String, String> = habits
        .into_iter()
        .map(|h| (h.id.clone(), h.name))
        .collect();

    let dtos: Vec<StreakRewardDto> = rewards
        .into_iter()
        .map(|r| {
            let milestones = controller
                .get_milestones(r.id.clone())
                .unwrap_or_default()
                .into_iter()
                .map(|m| MilestoneDto {
                    id: m.id,
                    target_days: m.target_days,
                    reward_text: m.reward_text,
                    unlocked: m.unlocked,
                    unlocked_at: m.unlocked_at,
                })
                .collect();

            let progress = controller.get_streak_progress(r.id.clone()).unwrap_or(0);

            StreakRewardDto {
                id: r.id,
                habit_id: r.habit_id.clone(),
                habit_name: habit_names.get(&r.habit_id).cloned().unwrap_or_default(),
                is_consecutive: r.is_consecutive,
                target_days: r.target_days,
                target_total: r.target_total,
                current_progress: progress,
                milestones,
            }
        })
        .collect();

    Ok(dtos)
}

/// Create a new streak reward.
#[tauri::command]
pub fn create_streak_reward(
    controller: State<'_, Arc<AppController>>,
    habit_id: String,
    is_consecutive: bool,
    target_days: i32,
    target_total: i32,
) -> Result<String, String> {
    controller
        .create_streak_reward(habit_id, is_consecutive, target_days, target_total)
        .map_err(|e| e.to_string())
}

/// Update a streak reward with milestones.
#[tauri::command]
pub fn update_streak_reward(
    controller: State<'_, Arc<AppController>>,
    id: String,
    habit_id: String,
    is_consecutive: bool,
    target_days: i32,
    target_total: i32,
    milestones: Vec<(i32, String)>,
) -> Result<(), String> {
    controller
        .update_streak_reward_with_milestones(
            id, habit_id, is_consecutive, target_days, target_total, milestones,
        )
        .map_err(|e| e.to_string())
}

/// Delete a streak reward.
#[tauri::command]
pub fn delete_streak_reward(
    controller: State<'_, Arc<AppController>>,
    id: String,
) -> Result<(), String> {
    controller.delete_streak_reward(id).map_err(|e| e.to_string())
}

/// Add a milestone to an existing streak reward.
#[tauri::command]
pub fn add_milestone(
    controller: State<'_, Arc<AppController>>,
    reward_id: String,
    target_days: i32,
    reward_text: String,
) -> Result<String, String> {
    controller
        .add_milestone(reward_id, target_days, reward_text)
        .map_err(|e| e.to_string())
}

// ==================== Goals ====================

/// Fetch all goals with checkpoints.
#[tauri::command]
pub fn fetch_goals(
    controller: State<'_, Arc<AppController>>,
) -> Result<Vec<GoalDto>, String> {
    let goals = controller.get_goals().map_err(|e| e.to_string())?;

    let dtos: Vec<GoalDto> = goals
        .into_iter()
        .map(|g| {
            let checkpoints = controller
                .get_checkpoints(g.id.clone())
                .unwrap_or_default()
                .into_iter()
                .map(|cp| CheckpointDto {
                    id: cp.id,
                    description: cp.description,
                    completed: cp.completed,
                    completed_at: cp.completed_at,
                })
                .collect();

            GoalDto {
                id: g.id,
                name: g.name,
                description: g.description,
                reward_text: g.reward_text,
                deadline: g.deadline,
                is_completed: g.is_completed,
                completed_at: g.completed_at,
                checkpoints,
            }
        })
        .collect();

    Ok(dtos)
}

/// Create a new goal.
#[tauri::command]
pub fn create_goal(
    controller: State<'_, Arc<AppController>>,
    name: String,
    description: String,
    reward_text: String,
    deadline: String,
) -> Result<String, String> {
    controller
        .create_goal(name, description, reward_text, deadline)
        .map_err(|e| e.to_string())
}

/// Update a goal.
#[tauri::command]
pub fn update_goal(
    controller: State<'_, Arc<AppController>>,
    id: String,
    name: String,
    description: String,
    reward_text: String,
    deadline: String,
) -> Result<(), String> {
    controller
        .update_goal(id, name, description, reward_text, deadline)
        .map_err(|e| e.to_string())
}

/// Checkpoint data for update_goal_with_checkpoints.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GoalCheckpointInput {
    pub id: String,
    pub text: String,
}

/// Update a goal with its checkpoints.
///
/// Accepts a clean Vec of checkpoints and maps to the controller's
/// positional API (max 4 checkpoints).
#[tauri::command]
pub fn update_goal_with_checkpoints(
    controller: State<'_, Arc<AppController>>,
    id: String,
    name: String,
    description: String,
    reward_text: String,
    deadline: String,
    checkpoints: Vec<GoalCheckpointInput>,
) -> Result<(), String> {
    let count = checkpoints.len().min(4) as i32;
    let get = |i: usize| -> (String, String) {
        checkpoints.get(i).map(|c| (c.id.clone(), c.text.clone()))
            .unwrap_or_default()
    };
    let (cp1_id, cp1_text) = get(0);
    let (cp2_id, cp2_text) = get(1);
    let (cp3_id, cp3_text) = get(2);
    let (cp4_id, cp4_text) = get(3);

    controller
        .update_goal_with_checkpoints(
            id, name, description, reward_text, deadline,
            count,
            cp1_id, cp1_text, cp2_id, cp2_text,
            cp3_id, cp3_text, cp4_id, cp4_text,
        )
        .map_err(|e| e.to_string())
}

/// Delete a goal.
#[tauri::command]
pub fn delete_goal(
    controller: State<'_, Arc<AppController>>,
    id: String,
) -> Result<(), String> {
    controller.delete_goal(id).map_err(|e| e.to_string())
}

/// Mark a goal as completed.
#[tauri::command]
pub fn complete_goal(
    controller: State<'_, Arc<AppController>>,
    id: String,
) -> Result<Option<String>, String> {
    controller.complete_goal(id).map_err(|e| e.to_string())
}

/// Archive a goal.
#[tauri::command]
pub fn archive_goal(
    controller: State<'_, Arc<AppController>>,
    id: String,
) -> Result<(), String> {
    controller.archive_goal(id).map_err(|e| e.to_string())
}

// ==================== Checkpoints ====================

/// Add a checkpoint to a goal.
#[tauri::command]
pub fn add_checkpoint(
    controller: State<'_, Arc<AppController>>,
    goal_id: String,
    description: String,
) -> Result<String, String> {
    controller
        .add_checkpoint(goal_id, description)
        .map_err(|e| e.to_string())
}

/// Update a checkpoint description.
#[tauri::command]
pub fn update_checkpoint(
    controller: State<'_, Arc<AppController>>,
    checkpoint_id: String,
    description: String,
) -> Result<(), String> {
    controller
        .update_checkpoint(checkpoint_id, description)
        .map_err(|e| e.to_string())
}

/// Delete a checkpoint.
#[tauri::command]
pub fn delete_checkpoint(
    controller: State<'_, Arc<AppController>>,
    checkpoint_id: String,
) -> Result<(), String> {
    controller
        .delete_checkpoint(checkpoint_id)
        .map_err(|e| e.to_string())
}

/// Toggle checkpoint completion.
#[tauri::command]
pub fn toggle_checkpoint(
    controller: State<'_, Arc<AppController>>,
    goal_id: String,
    checkpoint_id: String,
) -> Result<(), String> {
    controller
        .toggle_checkpoint(goal_id, checkpoint_id)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// ==================== Achievements ====================

/// Fetch all achievements (completed goals + unlocked milestones).
#[tauri::command]
pub fn fetch_achievements(
    controller: State<'_, Arc<AppController>>,
) -> Result<Vec<AchievementDto>, String> {
    let achievements = controller.get_achievements().map_err(|e| e.to_string())?;

    Ok(achievements
        .into_iter()
        .map(|a| AchievementDto {
            id: a.id,
            title: a.title,
            description: a.description,
            icon_path: a.icon_path,
            achievement_type: a.achievement_type,
            achieved_at: a.achieved_at,
        })
        .collect())
}
