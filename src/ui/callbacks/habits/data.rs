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

//! Habit data loading functions

use crate::controller::AppController;
use crate::ui::{calculate_best_streak, calculate_current_streak, color_from_hex};
use crate::{AppWindow, HabitAdapter, HabitData, HabitDay, HeatmapDay, HeatmapWeek};
use chrono::{Datelike, NaiveDate};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::Arc;

use super::helpers::{habit_color_index, normalize_habit_category_value};

struct HabitSummary {
    current_streak: i32,
    best_streak: i32,
    completion_rate: i32,
    completion_note: String,
    last_30: i32,
    best_day: String,
}

fn build_habit_summary(
    habit_dates: &[NaiveDate],
    monthly_completed: i32,
    days_total: u32,
    today: NaiveDate,
    is_current_month: bool,
) -> HabitSummary {
    let current_streak = calculate_current_streak(habit_dates, today);

    let window_start = today
        .checked_sub_signed(chrono::Duration::days(364))
        .unwrap_or(today);
    let recent_dates: Vec<NaiveDate> = habit_dates
        .iter()
        .cloned()
        .filter(|date| *date >= window_start)
        .collect();
    let best_streak = calculate_best_streak(&recent_dates);

    let completion_rate = if days_total > 0 {
        ((monthly_completed as f32 / days_total as f32) * 100.0).round() as i32
    } else {
        0
    };

    let completion_note = if is_current_month {
        format!(
            "Month-to-date: {} of {} days",
            monthly_completed, days_total
        )
    } else {
        format!(
            "Selected month: {} of {} days",
            monthly_completed, days_total
        )
    };

    let last_30_start = today
        .checked_sub_signed(chrono::Duration::days(29))
        .unwrap_or(today);
    let mut last_30 = 0;
    let mut weekday_counts = [0i32; 7];

    for date in habit_dates.iter().filter(|date| **date >= last_30_start) {
        last_30 += 1;
        let idx = date.weekday().num_days_from_monday() as usize;
        weekday_counts[idx] += 1;
    }

    let weekday_names = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    let (best_idx, best_count) = weekday_counts
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .unwrap_or((0, &0));

    let best_day = if *best_count > 0 {
        format!(
            "Best day (30d): {} ({})",
            weekday_names[best_idx], best_count
        )
    } else {
        "Best day (30d): No data yet".to_string()
    };

    HabitSummary {
        current_streak,
        best_streak,
        completion_rate,
        completion_note,
        last_30,
        best_day,
    }
}

fn clear_habit_summary(adapter: &HabitAdapter) {
    adapter.set_selected_habit_id(SharedString::default());
    adapter.set_summary_habit_name(SharedString::default());
    adapter.set_summary_habit_color(color_from_hex("#000000"));
    adapter.set_summary_current_streak(0);
    adapter.set_summary_best_streak(0);
    adapter.set_summary_completion_rate(0);
    adapter.set_summary_completion_note(SharedString::default());
    adapter.set_summary_last_30(0);
    adapter.set_summary_best_day(SharedString::default());
}

fn apply_habit_summary(
    adapter: &HabitAdapter,
    habit: &crate::models::Habit,
    summary: HabitSummary,
) {
    adapter.set_summary_habit_name(SharedString::from(habit.name.clone()));
    adapter.set_summary_habit_color(color_from_hex(&habit.color));
    adapter.set_summary_current_streak(summary.current_streak);
    adapter.set_summary_best_streak(summary.best_streak);
    adapter.set_summary_completion_rate(summary.completion_rate);
    adapter.set_summary_completion_note(SharedString::from(summary.completion_note));
    adapter.set_summary_last_30(summary.last_30);
    adapter.set_summary_best_day(SharedString::from(summary.best_day));
}

/// Reload habits for a given month
/// Optionally accepts a notify closure to report errors
pub fn reload_habits<N>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    current_date: NaiveDate,
    notify: Option<&N>,
) where
    N: Fn(String, bool),
{
    let year = current_date.year();
    let month = current_date.month();

    let Some(start_date) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return;
    };
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let Some(end_date) =
        NaiveDate::from_ymd_opt(next_year, next_month, 1).and_then(|d| d.pred_opt())
    else {
        return;
    };

    let days_in_month = end_date.day();

    let habits = match controller.get_habits() {
        Ok(h) => h,
        Err(e) => {
            if let Some(n) = notify {
                n(format!("Failed to load habits: {}", e), true);
            }
            return;
        }
    };

    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();

    // Fetch logs for the current month view (optimized: single query)
    let logs = controller
        .get_habit_logs(start_str, end_str)
        .unwrap_or_default();

    let mut log_map: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for log in logs {
        log_map.insert((log.habit_id, log.completed_date));
    }

    // OPTIMIZATION: Fetch ALL historical logs once for streak calculations
    let all_history_logs = controller.get_all_habit_logs().unwrap_or_default();

    // Group history logs by habit_id -> Sorted Vec of NaiveDates
    let mut history_map: HashMap<String, Vec<NaiveDate>> = HashMap::new();

    for log in all_history_logs {
        if let Ok(date) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d") {
            history_map.entry(log.habit_id).or_default().push(date);
        }
    }

    // Sort and dedup dates for each habit
    for dates in history_map.values_mut() {
        dates.sort();
        dates.dedup();
    }

    let mut monthly_completion_map: HashMap<String, i32> = HashMap::new();

    // Calculate 'today' once outside the loop to avoid repeated syscalls
    // and ensure consistent date across all habits in this refresh
    let today = chrono::Local::now().date_naive();

    let mapped_habits: Vec<HabitData> = habits
        .iter()
        .map(|h| {
            let mut days_vec: Vec<HabitDay> = Vec::new();
            let mut completions = 0;
            let habit_id = h.id.clone();

            // Build monthly view
            for d in 1..=days_in_month {
                let Some(date) = NaiveDate::from_ymd_opt(year, month, d) else {
                    continue;
                };
                let date_str = date.format("%Y-%m-%d").to_string();
                let is_future = date > today;

                let completed = log_map.contains(&(habit_id.clone(), date_str.clone()));
                if completed {
                    completions += 1;
                }

                days_vec.push(HabitDay {
                    day: d as i32,
                    completed,
                    date: SharedString::from(date_str),
                    is_future,
                });
            }

            monthly_completion_map.insert(habit_id.clone(), completions);

            // Calculate streaks using helper functions
            let habit_dates = history_map.get(&habit_id).cloned().unwrap_or_default();
            let current_streak = calculate_current_streak(&habit_dates, today);
            let best_streak = calculate_best_streak(&habit_dates);

            let color = color_from_hex(&h.color);
            let color_hex = h.color.clone();

            HabitData {
                id: SharedString::from(habit_id),
                name: SharedString::from(h.name.clone()),
                description: SharedString::from(h.description.clone().unwrap_or_default()),
                color,
                color_hex: SharedString::from(color_hex.clone()),
                color_index: habit_color_index(&color_hex),
                category: SharedString::from(normalize_habit_category_value(&h.category)),
                streak: current_streak,
                best_streak,
                completion_rate: if days_in_month > 0 {
                    ((completions as f32 / days_in_month as f32) * 100.0) as i32
                } else {
                    0
                },
                days: ModelRc::new(VecModel::from(days_vec)),
            }
        })
        .collect();

    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<HabitAdapter>();
        adapter.set_habits(ModelRc::new(VecModel::from(mapped_habits)));
        adapter.set_current_month_name(SharedString::from(
            start_date.format("%B").to_string().to_uppercase(),
        ));
        adapter.set_current_year(year);
        adapter.set_current_month_index(month as i32);

        // Auto-scroll context
        let is_current = year == today.year() && month == today.month();
        adapter.set_is_viewing_current_month(is_current);
        adapter.set_current_day_int(today.day() as i32);

        if habits.is_empty() {
            clear_habit_summary(&adapter);
            return;
        }

        let existing_selected = adapter.get_selected_habit_id().to_string();
        let selected_id = if !existing_selected.is_empty()
            && habits.iter().any(|habit| habit.id == existing_selected)
        {
            existing_selected
        } else {
            habits
                .first()
                .map(|habit| habit.id.clone())
                .unwrap_or_default()
        };

        adapter.set_selected_habit_id(SharedString::from(selected_id.clone()));

        if let Some(selected_habit) = habits.iter().find(|habit| habit.id == selected_id) {
            let habit_dates = history_map
                .get(&selected_habit.id)
                .cloned()
                .unwrap_or_default();
            let days_total = if is_current {
                today.day()
            } else {
                days_in_month
            };
            let monthly_completed = *monthly_completion_map.get(&selected_habit.id).unwrap_or(&0);
            let summary = build_habit_summary(
                &habit_dates,
                monthly_completed,
                days_total,
                today,
                is_current,
            );
            apply_habit_summary(&adapter, selected_habit, summary);
        } else {
            clear_habit_summary(&adapter);
        }
    }
}

/// Reload heatmap for a given year
pub fn reload_heatmap(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, year: i32) {
    let today = chrono::Local::now().date_naive();

    let Some(first_day_year) = NaiveDate::from_ymd_opt(year, 1, 1) else {
        return;
    };
    let Some(last_day_year) = NaiveDate::from_ymd_opt(year, 12, 31) else {
        return;
    };

    // Align start to Monday (for grid alignment)
    let days_from_mon = first_day_year.weekday().num_days_from_monday();
    let start_date = first_day_year - chrono::Duration::days(days_from_mon as i64);

    // Align end to Sunday (for grid alignment)
    let days_to_sun = 6 - last_day_year.weekday().num_days_from_monday();
    let end_date = last_day_year + chrono::Duration::days(days_to_sun as i64);

    // Fetch Logs (Only up to today if current year)
    let query_end = if year == today.year() {
        today
    } else if year < today.year() {
        last_day_year
    } else {
        first_day_year
    };

    let start_str = first_day_year.format("%Y-%m-%d").to_string();
    let end_str = query_end.format("%Y-%m-%d").to_string();

    let mut daily_counts: HashMap<String, i32> = HashMap::new();

    if year <= today.year()
        && let Ok(logs) = controller.get_habit_logs(start_str, end_str)
    {
        for log in logs {
            *daily_counts.entry(log.completed_date).or_insert(0) += 1;
        }
    }

    // Build Structure
    let mut weeks_vec: Vec<HeatmapWeek> = Vec::new();
    let mut current_day = start_date;

    while current_day <= end_date {
        let mut week_days: Vec<HeatmapDay> = Vec::new();

        for _ in 0..7 {
            // Check if this day is within the actual year
            let is_padding = current_day < first_day_year || current_day > last_day_year;

            if is_padding {
                // Padding day - level -1 makes it invisible
                week_days.push(HeatmapDay {
                    date: SharedString::default(),
                    count: 0,
                    level: -1,
                });
            } else {
                let date_str = current_day.format("%Y-%m-%d").to_string();
                let count = *daily_counts.get(&date_str).unwrap_or(&0);

                let is_future = if year == today.year() {
                    current_day > today
                } else {
                    year > today.year()
                };

                let level = if is_future || count == 0 {
                    0
                } else if count <= 1 {
                    1
                } else if count <= 2 {
                    2
                } else if count <= 4 {
                    3
                } else {
                    4
                };

                week_days.push(HeatmapDay {
                    date: SharedString::from(date_str),
                    count,
                    level,
                });
            }

            current_day += chrono::Duration::days(1);
        }

        weeks_vec.push(HeatmapWeek {
            days: ModelRc::new(VecModel::from(week_days)),
        });
    }

    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<HabitAdapter>();

        // Calculate week index in heatmap (not ISO week!)
        // This is the column index where today falls in the heatmap grid
        let current_week = if year == today.year() {
            // Calculate days from heatmap start to today, divide by 7
            let days_from_start = (today - start_date).num_days();
            ((days_from_start / 7) + 1) as i32
        } else if year < today.year() {
            weeks_vec.len() as i32 // Past year: scroll to end
        } else {
            1 // Future year: scroll to start
        };

        adapter.set_heatmap_data(ModelRc::new(VecModel::from(weeks_vec)));
        adapter.set_heatmap_year(year);
        adapter.set_current_week_int(current_week);
    }
}

/// Refreshes summary data for the selected habit when the user changes selection.
pub fn refresh_habit_summary<N>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    current_date: NaiveDate,
    habit_id: String,
    notify: Option<&N>,
) where
    N: Fn(String, bool),
{
    if habit_id.trim().is_empty() {
        return;
    }

    let habits = match controller.get_habits() {
        Ok(h) => h,
        Err(e) => {
            if let Some(n) = notify {
                n(format!("Failed to load habits: {}", e), true);
            }
            return;
        }
    };

    let Some(selected_habit) = habits.iter().find(|habit| habit.id == habit_id) else {
        return;
    };

    let year = current_date.year();
    let month = current_date.month();
    let Some(start_date) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return;
    };
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let Some(end_date) =
        NaiveDate::from_ymd_opt(next_year, next_month, 1).and_then(|d| d.pred_opt())
    else {
        return;
    };

    let today = chrono::Local::now().date_naive();
    let is_current = year == today.year() && month == today.month();
    let days_total = if is_current {
        today.day()
    } else {
        end_date.day()
    };

    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();
    let monthly_logs = controller
        .get_habit_logs(start_str, end_str)
        .unwrap_or_default();
    let monthly_completed = monthly_logs
        .iter()
        .filter(|log| log.habit_id == habit_id)
        .count() as i32;

    let all_logs = controller.get_all_habit_logs().unwrap_or_default();
    let mut habit_dates: Vec<NaiveDate> = all_logs
        .into_iter()
        .filter(|log| log.habit_id == habit_id)
        .filter_map(|log| NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d").ok())
        .collect();
    habit_dates.sort();
    habit_dates.dedup();

    let summary = build_habit_summary(
        &habit_dates,
        monthly_completed,
        days_total,
        today,
        is_current,
    );

    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<HabitAdapter>();
        adapter.set_selected_habit_id(SharedString::from(habit_id));
        apply_habit_summary(&adapter, selected_habit, summary);
    }
}
