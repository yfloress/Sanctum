//! Habit data loading functions

use crate::controller::AppController;
use crate::ui::{calculate_best_streak, calculate_current_streak, color_from_hex};
use crate::{AppWindow, HabitAdapter, HabitData, HabitDay, HeatmapDay, HeatmapWeek};
use chrono::{Datelike, NaiveDate};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::Arc;

use super::helpers::{habit_color_index, normalize_habit_category_value};

/// Reload habits for a given month
pub fn reload_habits(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    current_date: NaiveDate,
) {
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

    if let Ok(habits) = controller.get_habits() {
        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = end_date.format("%Y-%m-%d").to_string();

        // Fetch logs for the current month view (optimized: single query)
        let logs = controller
            .get_habit_logs(start_str, end_str)
            .unwrap_or_default();

        let mut log_map: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
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

        let mapped_habits: Vec<HabitData> = habits
            .into_iter()
            .map(|h| {
                let mut days_vec: Vec<HabitDay> = Vec::new();
                let mut completions = 0;
                let today = chrono::Local::now().date_naive();

                // Build monthly view
                for d in 1..=days_in_month {
                    let Some(date) = NaiveDate::from_ymd_opt(year, month, d) else {
                        continue;
                    };
                    let date_str = date.format("%Y-%m-%d").to_string();
                    let is_future = date > today;

                    let completed = log_map.contains(&(h.id.clone(), date_str.clone()));
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

                let completion_rate = if days_in_month > 0 {
                    ((completions as f32 / days_in_month as f32) * 100.0) as i32
                } else {
                    0
                };

                // Retrieve pre-processed historical dates for this habit
                let habit_dates = history_map.get(&h.id).cloned().unwrap_or_default();

                // Calculate streaks using helper functions
                let current_streak = calculate_current_streak(&habit_dates, today);
                let best_streak = calculate_best_streak(&habit_dates);

                let color = color_from_hex(&h.color);
                let color_hex = h.color.clone();

                HabitData {
                    id: SharedString::from(h.id),
                    name: SharedString::from(h.name),
                    description: SharedString::from(h.description.unwrap_or_default()),
                    color,
                    color_hex: SharedString::from(color_hex.clone()),
                    color_index: habit_color_index(&color_hex),
                    category: SharedString::from(normalize_habit_category_value(&h.category)),
                    streak: current_streak,
                    best_streak,
                    completion_rate,
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
            let now = chrono::Local::now().date_naive();
            let is_current = year == now.year() && month == now.month();
            adapter.set_is_viewing_current_month(is_current);
            adapter.set_current_day_int(now.day() as i32);
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
