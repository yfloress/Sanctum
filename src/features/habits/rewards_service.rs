//! Rewards service
//!
//! Business logic for streak rewards, goals, and achievements.

use super::rewards_repository::RewardsRepository;
use crate::db::{Database, DbError};
use crate::models::{Achievement, Checkpoint, Goal, Milestone, StreakReward};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct RewardsService {
    db: Arc<Mutex<Option<Database>>>,
}

impl RewardsService {
    pub fn new(db: Arc<Mutex<Option<Database>>>) -> Self {
        Self { db }
    }

    fn with_db<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Database) -> Result<T, DbError>,
    {
        let guard = self.db.lock().map_err(|_| DbError::InvalidPassword)?;
        if let Some(db) = guard.as_ref() {
            f(db)
        } else {
            Err(DbError::InvalidPassword)
        }
    }

    // ==================== Streak Rewards ====================

    pub fn create_streak_reward(
        &self,
        habit_id: String,
        is_consecutive: bool,
        target_days: Option<i32>,
        target_total: Option<i32>,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let reward = StreakReward::new(
            id.clone(),
            habit_id,
            is_consecutive,
            target_days,
            target_total,
        );

        self.with_db(|db| {
            RewardsRepository::create_streak_reward(db, &reward)?;
            Ok(())
        })?;

        Ok(id)
    }

    pub fn get_streak_rewards(&self) -> Result<Vec<StreakReward>, DbError> {
        self.with_db(RewardsRepository::get_streak_rewards)
    }

    pub fn get_streak_reward(&self, id: &str) -> Result<Option<StreakReward>, DbError> {
        self.with_db(|db| RewardsRepository::get_streak_reward(db, id))
    }

    pub fn delete_streak_reward(&self, id: String) -> Result<(), DbError> {
        self.with_db(|db| RewardsRepository::delete_streak_reward(db, &id))
    }

    pub fn get_streak_progress(&self, reward: &StreakReward) -> Result<i32, DbError> {
        self.with_db(|db| {
            RewardsRepository::get_streak_progress(
                db,
                &reward.habit_id,
                reward.is_consecutive,
                reward.target_total,
            )
        })
    }

    // ==================== Milestones ====================

    pub fn add_milestone(
        &self,
        reward_id: String,
        target_days: i32,
        reward_text: String,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let milestone = Milestone::new(id.clone(), reward_id, target_days, reward_text);

        self.with_db(|db| {
            RewardsRepository::create_milestone(db, &milestone)?;
            Ok(())
        })?;

        Ok(id)
    }

    pub fn get_milestones(&self, reward_id: &str) -> Result<Vec<Milestone>, DbError> {
        self.with_db(|db| RewardsRepository::get_milestones(db, reward_id))
    }

    /// Check and unlock milestones based on current progress
    pub fn check_and_unlock_milestones(&self, reward_id: &str) -> Result<Vec<String>, DbError> {
        self.with_db(|db| {
            let reward = match RewardsRepository::get_streak_reward(db, reward_id)? {
                Some(r) => r,
                None => return Ok(vec![]),
            };

            let progress = RewardsRepository::get_streak_progress(
                db,
                &reward.habit_id,
                reward.is_consecutive,
                reward.target_total,
            )?;

            let milestones = RewardsRepository::get_milestones(db, reward_id)?;
            let mut unlocked_ids = vec![];

            for mut milestone in milestones {
                if !milestone.unlocked && progress >= milestone.target_days {
                    milestone.unlock();
                    RewardsRepository::update_milestone(db, &milestone)?;
                    unlocked_ids.push(milestone.id);
                }
            }

            Ok(unlocked_ids)
        })
    }

    // ==================== Goals ====================

    pub fn create_goal(
        &self,
        name: String,
        description: Option<String>,
        reward_text: String,
        deadline: Option<String>,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let goal = Goal::new(id.clone(), name, description, reward_text, deadline);

        self.with_db(|db| {
            RewardsRepository::create_goal(db, &goal)?;
            Ok(())
        })?;

        Ok(id)
    }

    pub fn get_goals(&self) -> Result<Vec<Goal>, DbError> {
        self.with_db(RewardsRepository::get_goals)
    }

    pub fn get_goal(&self, id: &str) -> Result<Option<Goal>, DbError> {
        self.with_db(|db| RewardsRepository::get_goal(db, id))
    }

    pub fn delete_goal(&self, id: String) -> Result<(), DbError> {
        self.with_db(|db| RewardsRepository::delete_goal(db, &id))
    }

    pub fn complete_goal(&self, id: String) -> Result<Option<String>, DbError> {
        self.with_db(|db| {
            let mut goal = match RewardsRepository::get_goal(db, &id)? {
                Some(g) => g,
                None => return Ok(None),
            };

            if goal.is_completed {
                return Ok(None);
            }

            goal.complete();
            RewardsRepository::update_goal(db, &goal)?;

            // Create achievement
            let ach_id = self.create_achievement_internal(
                db,
                goal.name.clone(),
                format!("Completed: {}", goal.reward_text),
                "trophy.svg".to_string(),
                "goal".to_string(),
                id,
            )?;

            Ok(Some(ach_id))
        })
    }

    // ==================== Checkpoints ====================

    pub fn add_checkpoint(&self, goal_id: String, description: String) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();

        self.with_db(|db| {
            let order = RewardsRepository::get_next_checkpoint_order(db, &goal_id)?;
            let checkpoint = Checkpoint::new(id.clone(), goal_id, description, order);
            RewardsRepository::create_checkpoint(db, &checkpoint)?;
            Ok(())
        })?;

        Ok(id)
    }

    pub fn get_checkpoints(&self, goal_id: &str) -> Result<Vec<Checkpoint>, DbError> {
        self.with_db(|db| RewardsRepository::get_checkpoints(db, goal_id))
    }

    pub fn toggle_checkpoint(
        &self,
        goal_id: String,
        checkpoint_id: String,
    ) -> Result<bool, DbError> {
        self.with_db(|db| {
            let mut checkpoint = match RewardsRepository::get_checkpoint(db, &checkpoint_id)? {
                Some(c) => c,
                None => return Ok(false),
            };

            let is_completed = checkpoint.toggle();
            RewardsRepository::update_checkpoint(db, &checkpoint)?;

            // Check if all checkpoints complete
            let (total, completed) = RewardsRepository::get_checkpoint_counts(db, &goal_id)?;

            if total > 0
                && total == completed
                && let Some(mut goal) = RewardsRepository::get_goal(db, &goal_id)?
                && !goal.is_completed
            {
                goal.complete();
                RewardsRepository::update_goal(db, &goal)?;

                let _ = self.create_achievement_internal(
                    db,
                    goal.name.clone(),
                    format!("Completed: {}", goal.reward_text),
                    "trophy.svg".to_string(),
                    "goal".to_string(),
                    goal_id,
                );
            }

            Ok(is_completed)
        })
    }

    pub fn get_checkpoint_progress(&self, goal_id: &str) -> Result<(i32, i32), DbError> {
        self.with_db(|db| RewardsRepository::get_checkpoint_counts(db, goal_id))
    }

    // ==================== Achievements ====================

    fn create_achievement_internal(
        &self,
        db: &Database,
        title: String,
        description: String,
        icon_path: String,
        achievement_type: String,
        source_id: String,
    ) -> Result<String, DbError> {
        if RewardsRepository::achievement_exists(db, &source_id, &achievement_type)? {
            return Ok(String::new());
        }

        let id = Uuid::new_v4().to_string();
        let achievement = Achievement::new(
            id.clone(),
            title,
            description,
            icon_path,
            achievement_type,
            source_id,
        );

        RewardsRepository::create_achievement(db, &achievement)?;
        Ok(id)
    }

    pub fn get_achievements(&self) -> Result<Vec<Achievement>, DbError> {
        self.with_db(RewardsRepository::get_achievements)
    }
}
