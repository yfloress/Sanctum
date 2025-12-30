//! Habit analytics computation and caching

use crate::controller::AppController;
use crate::{AppWindow, HabitAdapter};
use chrono::{Datelike, NaiveDate};
use slint::{ComponentHandle, Image, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use super::helpers::normalize_habit_category_value;

/// Snapshot of habit analytics data for caching
#[derive(Clone, Default)]
pub struct HabitAnalyticsSnapshot {
    pub radar_image: Image,
    pub radar_has_data: bool,
    pub weekday_image: Image,
    pub weekday_has_data: bool,
    pub weekly_primary: String,
    pub weekly_secondary: String,
    pub insight_primary: String,
    pub insight_secondary: String,
}

/// Cache key for habit analytics
#[derive(Clone, PartialEq, Eq)]
pub struct HabitAnalyticsKey {
    pub habits_len: usize,
    pub logs_len: usize,
    pub last_log_date: Option<String>,
    pub habit_hash: u64,
}

/// Cache for habit analytics to avoid recalculation
#[derive(Default)]
pub struct HabitChartsCache {
    pub key: Option<HabitAnalyticsKey>,
    pub snapshot: HabitAnalyticsSnapshot,
}

/// Refresh habit analytics with caching
pub fn refresh_habit_analytics<F: Fn(String, bool)>(
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    cache: &Rc<RefCell<HabitChartsCache>>,
    notify: &F,
) {
    let today = chrono::Local::now().date_naive();
    let days_window: i64 = 30;
    let start_date = today
        .checked_sub_signed(chrono::Duration::days(days_window - 1))
        .unwrap_or(today);

    let logs = match controller.get_habit_logs(
        start_date.format("%Y-%m-%d").to_string(),
        today.format("%Y-%m-%d").to_string(),
    ) {
        Ok(data) => data,
        Err(e) => {
            notify(format!("Failed to load habit analytics: {}", e), true);
            return;
        }
    };
    let habits = match controller.get_habits() {
        Ok(data) => data,
        Err(e) => {
            notify(format!("Failed to load habits: {}", e), true);
            return;
        }
    };

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for habit in &habits {
        habit.id.hash(&mut hasher);
        habit.name.hash(&mut hasher);
        habit.category.hash(&mut hasher);
    }
    let habit_hash = hasher.finish();
    let key = HabitAnalyticsKey {
        habits_len: habits.len(),
        logs_len: logs.len(),
        last_log_date: logs.last().map(|log| log.completed_date.clone()),
        habit_hash,
    };

    {
        let cache_guard = cache.borrow();
        if cache_guard.key.as_ref() == Some(&key) {
            let snapshot = cache_guard.snapshot.clone();
            drop(cache_guard);
            if let Some(ui) = ui_weak.upgrade() {
                let adapter = ui.global::<HabitAdapter>();
                adapter.set_habits_radar_chart_image(snapshot.radar_image);
                adapter.set_habits_radar_has_data(snapshot.radar_has_data);
                adapter.set_habits_weekday_chart_image(snapshot.weekday_image);
                adapter.set_habits_weekday_has_data(snapshot.weekday_has_data);
                adapter.set_habits_weekly_primary(snapshot.weekly_primary.into());
                adapter.set_habits_weekly_secondary(snapshot.weekly_secondary.into());
                adapter.set_habits_insight_primary(snapshot.insight_primary.into());
                adapter.set_habits_insight_secondary(snapshot.insight_secondary.into());
            }
            return;
        }
    }

    if habits.is_empty() {
        let snapshot = HabitAnalyticsSnapshot {
            radar_image: Image::default(),
            radar_has_data: false,
            weekday_image: Image::default(),
            weekday_has_data: false,
            weekly_primary: "Create your first habit to get started.".to_string(),
            weekly_secondary: "".to_string(),
            insight_primary: "Your insights will appear here once you have data.".to_string(),
            insight_secondary: "".to_string(),
        };
        let mut cache_guard = cache.borrow_mut();
        cache_guard.key = Some(key);
        cache_guard.snapshot = snapshot.clone();
        if let Some(ui) = ui_weak.upgrade() {
            let adapter = ui.global::<HabitAdapter>();
            adapter.set_habits_radar_chart_image(snapshot.radar_image);
            adapter.set_habits_radar_has_data(snapshot.radar_has_data);
            adapter.set_habits_weekday_chart_image(snapshot.weekday_image);
            adapter.set_habits_weekday_has_data(snapshot.weekday_has_data);
            adapter.set_habits_weekly_primary(snapshot.weekly_primary.into());
            adapter.set_habits_weekly_secondary(snapshot.weekly_secondary.into());
            adapter.set_habits_insight_primary(snapshot.insight_primary.into());
            adapter.set_habits_insight_secondary(snapshot.insight_secondary.into());
        }
        return;
    }

    let mut habit_categories: HashMap<String, String> = HashMap::new();
    let mut category_counts: HashMap<String, i32> = HashMap::new();
    for habit in &habits {
        let category = normalize_habit_category_value(&habit.category);
        habit_categories.insert(habit.id.clone(), category.clone());
        *category_counts.entry(category).or_insert(0) += 1;
    }

    let mut category_completions: HashMap<String, i32> = HashMap::new();
    let mut daily_counts: HashMap<NaiveDate, i32> = HashMap::new();
    let mut habit_week_counts: HashMap<String, i32> = HashMap::new();
    let mut total_completed = 0i32;

    let week_start = today
        .checked_sub_signed(chrono::Duration::days(6))
        .unwrap_or(today);
    let prev_week_start = week_start
        .checked_sub_signed(chrono::Duration::days(7))
        .unwrap_or(week_start);
    let prev_week_end = week_start
        .checked_sub_signed(chrono::Duration::days(1))
        .unwrap_or(week_start);

    for log in &logs {
        let Ok(date) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d") else {
            continue;
        };
        total_completed += 1;
        *daily_counts.entry(date).or_insert(0) += 1;

        if let Some(category) = habit_categories.get(&log.habit_id) {
            *category_completions.entry(category.clone()).or_insert(0) += 1;
        }

        if date >= week_start && date <= today {
            *habit_week_counts.entry(log.habit_id.clone()).or_insert(0) += 1;
        }
    }

    let total_days = (today - start_date).num_days().max(0) as f32 + 1.0;
    let categories = [
        ("mind", "MIND", "#38bdf8"),
        ("body", "BODY", "#22c55e"),
        ("spirit", "DISCIPLINE", "#a855f7"),
    ];

    let radar_data: Vec<(String, String, f32)> = categories
        .iter()
        .map(|(key, label, color)| {
            let count = *category_counts.get(*key).unwrap_or(&0) as f32;
            let max_total = count * total_days;
            let completed = *category_completions.get(*key).unwrap_or(&0) as f32;
            let ratio = if max_total > 0.0 {
                completed / max_total
            } else {
                0.0
            };
            (label.to_string(), (*color).to_string(), ratio)
        })
        .collect();

    let radar_image = if total_completed > 0 {
        controller.render_habit_radar_chart(&radar_data)
    } else {
        None
    };

    let max_week_total = habits.len() as f32 * 7.0;
    let mut current_week_total = 0f32;
    let mut prev_week_total = 0f32;

    for (date, count) in &daily_counts {
        if *date >= week_start && *date <= today {
            current_week_total += *count as f32;
        } else if *date >= prev_week_start && *date <= prev_week_end {
            prev_week_total += *count as f32;
        }
    }

    let current_rate = if max_week_total > 0.0 {
        current_week_total / max_week_total
    } else {
        0.0
    };
    let prev_rate = if max_week_total > 0.0 {
        prev_week_total / max_week_total
    } else {
        0.0
    };

    let weekly_primary = if total_completed == 0 {
        "Start today: complete your first habit.".to_string()
    } else if prev_rate > 0.0 {
        let diff = ((current_rate - prev_rate) / prev_rate) * 100.0;
        if diff >= 1.0 {
            format!(
                "Week Close: Your consistency is up {:.0}% vs last week.",
                diff
            )
        } else if diff <= -1.0 {
            format!(
                "Week Close: Your consistency is down {:.0}% vs last week.",
                diff.abs()
            )
        } else {
            "Week Close: Your consistency is stable.".to_string()
        }
    } else {
        "Week Close: First week logged. Good start.".to_string()
    };

    let weekly_secondary = if total_completed == 0 {
        "".to_string()
    } else if let Some((habit_id, count)) = habit_week_counts.iter().max_by_key(|(_, count)| *count)
    {
        let habit_name = habits
            .iter()
            .find(|habit| habit.id == *habit_id)
            .map(|habit| habit.name.clone())
            .unwrap_or_else(|| "Habit".to_string());
        format!("Star Habit: {}. {}/7 days completed.", habit_name, count)
    } else {
        "Star Habit: No data yet this week.".to_string()
    };

    let mut weekday_counts = [0f32; 7];
    let mut weekday_occurrences = [0f32; 7];
    let mut cursor = start_date;
    while cursor <= today {
        let idx = cursor.weekday().num_days_from_monday() as usize;
        weekday_occurrences[idx] += 1.0;
        if let Some(count) = daily_counts.get(&cursor) {
            weekday_counts[idx] += *count as f32;
        }
        if let Some(next) = cursor.succ_opt() {
            cursor = next;
        } else {
            break;
        }
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
    let mut worst_idx = 0usize;
    let mut worst_avg = f32::MAX;
    for idx in 0..7 {
        let avg = if weekday_occurrences[idx] > 0.0 {
            weekday_counts[idx] / weekday_occurrences[idx]
        } else {
            0.0
        };
        if avg < worst_avg {
            worst_avg = avg;
            worst_idx = idx;
        }
    }

    let today_idx = today.weekday().num_days_from_monday() as usize;
    let today_count = *daily_counts.get(&today).unwrap_or(&0) as f32;
    let today_avg = if weekday_occurrences[today_idx] > 0.0 {
        weekday_counts[today_idx] / weekday_occurrences[today_idx]
    } else {
        0.0
    };

    let insight_primary = if total_completed == 0 {
        "Complete your first habit to unlock insights.".to_string()
    } else {
        format!(
            "Watch out: Your stats tend to drop on {}s.",
            weekday_names[worst_idx]
        )
    };

    let insight_secondary = if total_completed == 0 {
        "".to_string()
    } else if today_count > today_avg + 1.0 {
        format!(
            "Today you're above your {} average.",
            weekday_names[today_idx]
        )
    } else if today_count + 1.0 < today_avg {
        format!(
            "Today you're below your {} average.",
            weekday_names[today_idx]
        )
    } else {
        "Today you're at your usual average.".to_string()
    };

    // Generate weekday efficiency bar chart
    let weekday_short_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let mut weekday_data: Vec<(String, f32, bool)> = Vec::new();
    let mut max_weekday_avg = 0.0f32;

    for idx in 0..7 {
        let avg = if weekday_occurrences[idx] > 0.0 {
            weekday_counts[idx] / weekday_occurrences[idx]
        } else {
            0.0
        };
        if avg > max_weekday_avg {
            max_weekday_avg = avg;
        }
        weekday_data.push((weekday_short_names[idx].to_string(), avg, false));
    }

    // Mark the best day(s)
    if max_weekday_avg > 0.0 {
        for (_, avg, is_best) in &mut weekday_data {
            if (*avg - max_weekday_avg).abs() < 0.001 {
                *is_best = true;
            }
        }
    }

    let weekday_chart_image = if total_completed > 0 && max_weekday_avg > 0.0 {
        controller.render_weekday_efficiency_chart(&weekday_data)
    } else {
        None
    };

    let snapshot = HabitAnalyticsSnapshot {
        radar_image: radar_image.unwrap_or_default(),
        radar_has_data: total_completed > 0,
        weekday_image: weekday_chart_image.unwrap_or_default(),
        weekday_has_data: total_completed > 0 && max_weekday_avg > 0.0,
        weekly_primary,
        weekly_secondary,
        insight_primary,
        insight_secondary,
    };

    {
        let mut cache_guard = cache.borrow_mut();
        cache_guard.key = Some(key);
        cache_guard.snapshot = snapshot.clone();
    }

    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<HabitAdapter>();
        adapter.set_habits_radar_chart_image(snapshot.radar_image);
        adapter.set_habits_radar_has_data(snapshot.radar_has_data);
        adapter.set_habits_weekday_chart_image(snapshot.weekday_image);
        adapter.set_habits_weekday_has_data(snapshot.weekday_has_data);
        adapter.set_habits_weekly_primary(snapshot.weekly_primary.into());
        adapter.set_habits_weekly_secondary(snapshot.weekly_secondary.into());
        adapter.set_habits_insight_primary(snapshot.insight_primary.into());
        adapter.set_habits_insight_secondary(snapshot.insight_secondary.into());
    }
}
