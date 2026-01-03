//! Habits-related controller methods
//!
//! Habit CRUD, logs, and analytics.

use super::{validate_uuid, normalize_habit_category, AppController, ControllerError};
use super::{HabitAnalytics, WeekdayEfficiency, MonthlyTrendPoint};
use crate::models::{Habit, HabitLog};
use chrono::{Datelike, NaiveDate};
use regex::Regex;

impl AppController {
    // ==================== Habit Management ====================

    pub fn create_habit(
        &self,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
    ) -> std::result::Result<String, ControllerError> {
        if name.trim().is_empty() {
            return Err(ControllerError::Validation(
                "Habit name cannot be empty".to_string(),
            ));
        }

        // Validate color format (basic hex)
        let color_regex = Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap();
        if !color_regex.is_match(&color) {
            return Err(ControllerError::Validation(
                "Invalid color format. Use #RRGGBB".to_string(),
            ));
        }

        let category = normalize_habit_category(&category).ok_or_else(|| {
            ControllerError::Validation("Invalid habit category".to_string())
        })?;

        self.habit_service
            .create_habit(name, description, color, category)
            .map_err(ControllerError::Database)
    }

    pub fn get_habits(&self) -> std::result::Result<Vec<Habit>, ControllerError> {
        self.habit_service
            .get_habits()
            .map_err(ControllerError::Database)
    }

    /// Updates a habit
    pub fn update_habit(
        &self,
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        is_archived: bool,
    ) -> std::result::Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }

        if name.trim().is_empty() {
            return Err(ControllerError::Validation(
                "Habit name cannot be empty".to_string(),
            ));
        }

        // Validate color
        let color_regex = Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap();
        if !color_regex.is_match(&color) {
            return Err(ControllerError::Validation(
                "Invalid color format. Use #RRGGBB".to_string(),
            ));
        }

        let category = normalize_habit_category(&category).ok_or_else(|| {
            ControllerError::Validation("Invalid habit category".to_string())
        })?;

        self.habit_service
            .update_habit(id, name, description, color, category, is_archived)
            .map_err(ControllerError::Database)
    }

    pub fn archive_habit(&self, id: String) -> std::result::Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }
        self.habit_service
            .archive_habit(id)
            .map_err(ControllerError::Database)
    }

    /// Deletes a habit
    pub fn delete_habit(&self, id: String) -> std::result::Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }
        self.habit_service
            .delete_habit(id)
            .map_err(ControllerError::Database)
    }

    /// Toggles habit completion for a date
    pub fn toggle_habit_completion(
        &self,
        habit_id: String,
        date: String,
    ) -> std::result::Result<bool, ControllerError> {
        if validate_uuid(&habit_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".to_string()));
        }

        // Validate date format YYYY-MM-DD
        if NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
            return Err(ControllerError::Validation(
                "Invalid date format. Use YYYY-MM-DD".to_string(),
            ));
        }

        self.habit_service
            .toggle_habit_completion(habit_id, date)
            .map_err(ControllerError::Database)
    }

    pub fn get_habit_logs(
        &self,
        start_date: String,
        end_date: String,
    ) -> std::result::Result<Vec<HabitLog>, ControllerError> {
        // Validate dates
        if NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").is_err()
            || NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").is_err()
        {
            return Err(ControllerError::Validation(
                "Invalid date format".to_string(),
            ));
        }

        self.habit_service
            .get_habit_logs(start_date, end_date)
            .map_err(ControllerError::Database)
    }

    /// Gets habit logs for streak calculation (optimized: last 2 years)
    /// This avoids N+1 query problems by fetching relevant logs at once.
    /// Limited to 730 days to balance performance with practical streak tracking.
    pub fn get_all_habit_logs(&self) -> std::result::Result<Vec<HabitLog>, ControllerError> {
        let today = chrono::Local::now().date_naive();
        let start_date = today
            .checked_sub_signed(chrono::Duration::days(730))
            .unwrap_or(today);
        self.get_habit_logs(
            start_date.format("%Y-%m-%d").to_string(),
            today.format("%Y-%m-%d").to_string(),
        )
    }

    /// Gets habit analytics: weekday efficiency and monthly trend
    pub fn get_habit_analytics(&self, days: i32) -> Result<HabitAnalytics, ControllerError> {
        let today = chrono::Local::now().date_naive();
        let start_date = today
            .checked_sub_signed(chrono::Duration::days(days as i64))
            .unwrap_or(today);

        let logs = self.get_habit_logs(
            start_date.format("%Y-%m-%d").to_string(),
            today.format("%Y-%m-%d").to_string(),
        )?;

        // ==================== Weekday Efficiency ====================
        // Count completions per weekday
        let mut weekday_counts: [i32; 7] = [0; 7]; // Mon=0, Tue=1, ..., Sun=6
        let mut weekday_occurrences: [i32; 7] = [0; 7];

        // Count how many times each weekday appears in the range
        let mut cursor = start_date;
        while cursor <= today {
            let weekday_idx = cursor.weekday().num_days_from_monday() as usize;
            weekday_occurrences[weekday_idx] += 1;
            cursor = cursor.succ_opt().unwrap_or(cursor);
            if cursor == today && cursor == start_date {
                break; // Single day edge case
            }
            if cursor > today {
                break;
            }
        }

        // Count habit completions per weekday
        for log in &logs {
            if let Ok(date) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d") {
                let weekday_idx = date.weekday().num_days_from_monday() as usize;
                weekday_counts[weekday_idx] += 1;
            }
        }

        // Calculate averages
        let weekday_avgs: Vec<(usize, f32)> = (0..7)
            .map(|i| {
                let avg = if weekday_occurrences[i] > 0 {
                    weekday_counts[i] as f32 / weekday_occurrences[i] as f32
                } else {
                    0.0
                };
                (i, avg)
            })
            .collect();

        let max_avg = weekday_avgs
            .iter()
            .map(|(_, avg)| *avg)
            .fold(0.0_f32, f32::max);

        let day_names = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        let day_shorts = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

        let weekday_data: Vec<WeekdayEfficiency> = weekday_avgs
            .iter()
            .map(|(i, avg)| {
                let bar_height = if max_avg > 0.0 {
                    (*avg / max_avg) * 100.0
                } else {
                    0.0
                };
                WeekdayEfficiency {
                    day_name: day_names[*i].to_string(),
                    day_short: day_shorts[*i].to_string(),
                    avg_count: *avg,
                    is_best: (*avg - max_avg).abs() < 0.001 && max_avg > 0.0,
                    bar_height_percent: bar_height,
                }
            })
            .collect();

        // ==================== Monthly Trend ====================
        // Show last 12 months including current month (rolling 12-month window)
        let current_year = today.year();
        let current_month = today.month();

        // Calculate start month (11 months back)
        let (start_year, start_month_num) = if current_month <= 11 {
            (current_year - 1, current_month + 1)
        } else {
            (current_year, current_month - 11)
        };

        let twelve_months_ago_start = NaiveDate::from_ymd_opt(start_year, start_month_num, 1)
            .unwrap_or(start_date);

        // Group by month and calculate average habits per day
        let mut monthly_data_map: std::collections::BTreeMap<(i32, u32), (i32, i32)> =
            std::collections::BTreeMap::new(); // (year, month) -> (habit_count, days_in_range)

        // Count days per month in the last 12 months
        let mut cursor = twelve_months_ago_start;
        while cursor <= today {
            let key = (cursor.year(), cursor.month());
            monthly_data_map.entry(key).or_insert((0, 0)).1 += 1;
            cursor = cursor.succ_opt().unwrap_or(cursor);
            if cursor > today {
                break;
            }
        }

        // Count habits per month in the last 12 months
        for log in &logs {
            if let Ok(date) = NaiveDate::parse_from_str(&log.completed_date, "%Y-%m-%d")
                && date >= twelve_months_ago_start
            {
                let key = (date.year(), date.month());
                if let Some(entry) = monthly_data_map.get_mut(&key) {
                    entry.0 += 1;
                }
            }
        }

        // Convert to sorted vec with averages
        let month_names = [
            "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];

        let monthly_avgs: Vec<((i32, u32), f32, String)> = monthly_data_map
            .iter()
            .map(|((year, month), (count, days))| {
                let avg = if *days > 0 {
                    *count as f32 / *days as f32
                } else {
                    0.0
                };
                let label = format!("{} {}", month_names[*month as usize], year % 100);
                ((*year, *month), avg, label)
            })
            .collect();

        if monthly_avgs.is_empty() {
            return Ok(HabitAnalytics {
                weekday_data,
                monthly_data: vec![],
                monthly_path: "M 0 50 L 100 50".to_string(),
            });
        }

        let min_avg = monthly_avgs
            .iter()
            .map(|(_, avg, _)| *avg)
            .fold(f32::MAX, f32::min);
        let max_monthly_avg = monthly_avgs
            .iter()
            .map(|(_, avg, _)| *avg)
            .fold(0.0_f32, f32::max);

        let range = (max_monthly_avg - min_avg).max(0.1); // Avoid division by zero
        let len = monthly_avgs.len();

        let monthly_data: Vec<MonthlyTrendPoint> = monthly_avgs
            .iter()
            .enumerate()
            .map(|(i, (_, avg, label))| {
                let x = if len > 1 {
                    (i as f32 / (len - 1) as f32) * 100.0
                } else {
                    50.0
                };
                let y_ratio = (*avg - min_avg) / range;
                let y = 100.0 - (10.0 + y_ratio * 80.0); // 10% padding top/bottom

                MonthlyTrendPoint {
                    month_name: label.clone(),
                    avg_per_day: *avg,
                    x_percent: x,
                    y_percent: y,
                }
            })
            .collect();

        // Generate SVG path for monthly trend (simple lines)
        let mut path = String::new();
        for (i, point) in monthly_data.iter().enumerate() {
            if i == 0 {
                path.push_str(&format!("M {:.2} {:.2}", point.x_percent, point.y_percent));
            } else {
                path.push_str(&format!(" L {:.2} {:.2}", point.x_percent, point.y_percent));
            }
        }

        if path.is_empty() {
            path = "M 0 50 L 100 50".to_string();
        }

        Ok(HabitAnalytics {
            weekday_data,
            monthly_data,
            monthly_path: path,
        })
    }
}
