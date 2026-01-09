//! Habits feature module
//!
//! Handles habit tracking, logs, streaks, and rewards.

pub mod repository;
pub mod rewards_repository;
pub mod rewards_service;
pub mod service;

pub use repository::HabitsRepository;
pub use rewards_repository::RewardsRepository;
pub use rewards_service::RewardsService;
pub use service::HabitService;
