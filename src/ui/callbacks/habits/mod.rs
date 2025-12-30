//! Habits domain callbacks
//!
//! Split into focused submodules:
//! - `helpers` - Category normalization and color index utilities
//! - `data` - Habit data loading (reload_habits, reload_heatmap)
//! - `charts` - Chart computation and caching
//! - `callbacks` - Callback registrations

mod callbacks;
mod charts;
mod data;
mod helpers;

pub use callbacks::setup_habit_callbacks;
pub use charts::HabitChartsCache;
