//! Rewards repository
//!
//! Database operations for rewards feature.

use crate::db::{Database, DbError};
use crate::models::{Achievement, Checkpoint, Goal, Milestone, StreakReward};

/// Repository for rewards-related database operations
pub struct RewardsRepository;

impl RewardsRepository {
    // ==================== Streak Rewards ====================

    pub fn create_streak_reward(db: &Database, reward: &StreakReward) -> Result<(), DbError> {
        db.create_streak_reward(reward)
    }

    pub fn get_streak_rewards(db: &Database) -> Result<Vec<StreakReward>, DbError> {
        db.get_streak_rewards()
    }

    pub fn get_streak_rewards_by_habit(
        db: &Database,
        habit_id: &str,
    ) -> Result<Vec<StreakReward>, DbError> {
        db.get_streak_rewards_by_habit(habit_id)
    }

    pub fn get_streak_reward(db: &Database, id: &str) -> Result<Option<StreakReward>, DbError> {
        db.get_streak_reward(id)
    }

    pub fn delete_streak_reward(db: &Database, id: &str) -> Result<(), DbError> {
        db.delete_streak_reward(id)
    }

    // ==================== Milestones ====================

    pub fn create_milestone(db: &Database, milestone: &Milestone) -> Result<(), DbError> {
        db.create_milestone(milestone)
    }

    pub fn get_milestones(db: &Database, reward_id: &str) -> Result<Vec<Milestone>, DbError> {
        db.get_milestones(reward_id)
    }

    pub fn update_milestone(db: &Database, milestone: &Milestone) -> Result<(), DbError> {
        db.update_milestone(milestone)
    }

    // ==================== Goals ====================

    pub fn create_goal(db: &Database, goal: &Goal) -> Result<(), DbError> {
        db.create_goal(goal)
    }

    pub fn get_goals(db: &Database) -> Result<Vec<Goal>, DbError> {
        db.get_goals()
    }

    pub fn get_goal(db: &Database, id: &str) -> Result<Option<Goal>, DbError> {
        db.get_goal(id)
    }

    pub fn update_goal(db: &Database, goal: &Goal) -> Result<(), DbError> {
        db.update_goal(goal)
    }

    pub fn delete_goal(db: &Database, id: &str) -> Result<(), DbError> {
        db.delete_goal(id)
    }

    // ==================== Checkpoints ====================

    pub fn create_checkpoint(db: &Database, checkpoint: &Checkpoint) -> Result<(), DbError> {
        db.create_checkpoint(checkpoint)
    }

    pub fn get_checkpoints(db: &Database, goal_id: &str) -> Result<Vec<Checkpoint>, DbError> {
        db.get_checkpoints(goal_id)
    }

    pub fn get_checkpoint(db: &Database, id: &str) -> Result<Option<Checkpoint>, DbError> {
        db.get_checkpoint(id)
    }

    pub fn update_checkpoint(db: &Database, checkpoint: &Checkpoint) -> Result<(), DbError> {
        db.update_checkpoint(checkpoint)
    }

    pub fn get_next_checkpoint_order(db: &Database, goal_id: &str) -> Result<i32, DbError> {
        db.get_next_checkpoint_order(goal_id)
    }

    pub fn get_checkpoint_counts(db: &Database, goal_id: &str) -> Result<(i32, i32), DbError> {
        db.get_checkpoint_counts(goal_id)
    }

    // ==================== Achievements ====================

    pub fn create_achievement(db: &Database, achievement: &Achievement) -> Result<(), DbError> {
        db.create_achievement(achievement)
    }

    pub fn get_achievements(db: &Database) -> Result<Vec<Achievement>, DbError> {
        db.get_achievements()
    }

    pub fn achievement_exists(
        db: &Database,
        source_id: &str,
        achievement_type: &str,
    ) -> Result<bool, DbError> {
        db.achievement_exists(source_id, achievement_type)
    }

    // ==================== Streak Progress ====================

    pub fn get_streak_progress(
        db: &Database,
        habit_id: &str,
        is_consecutive: bool,
        target_total: Option<i32>,
    ) -> Result<i32, DbError> {
        db.get_streak_progress(habit_id, is_consecutive, target_total)
    }
}
