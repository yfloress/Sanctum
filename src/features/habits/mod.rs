//! Habits feature module
//!
//! Handles habit tracking, logs, and streaks.

pub mod repository;
pub mod service;

pub use repository::HabitsRepository;
pub use service::HabitService;
