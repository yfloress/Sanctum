//! Habits domain callbacks
//!
//! Split into focused submodules:
//! - `helpers` - Category normalization and color index utilities
//! - `data` - Habit data loading (reload_habits, reload_heatmap)
//! - `analytics` - Analytics computation and caching
//! - `callbacks` - Callback registrations

mod analytics;
mod callbacks;
mod data;
mod helpers;

pub use analytics::HabitAnalyticsCache;
pub use callbacks::setup_habit_callbacks;
