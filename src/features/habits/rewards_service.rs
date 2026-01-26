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

//! Rewards service
//!
//! Business logic for streak rewards, goals, and achievements.

use super::rewards_repository::RewardsRepository;
use crate::db::{Database, DbError};
use crate::models::{Achievement, Checkpoint, Goal, Milestone, StreakReward};
use rusqlite::Error as RusqliteError;
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
        let guard = self.db.lock().map_err(|_| DbError::MutexPoisoned)?;
        if let Some(db) = guard.as_ref() {
            f(db)
        } else {
            Err(DbError::DatabaseNotOpen)
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

    pub fn get_streak_rewards_by_habit(
        &self,
        habit_id: &str,
    ) -> Result<Vec<StreakReward>, DbError> {
        self.with_db(|db| RewardsRepository::get_streak_rewards_by_habit(db, habit_id))
    }

    pub fn get_streak_reward(&self, id: &str) -> Result<Option<StreakReward>, DbError> {
        self.with_db(|db| RewardsRepository::get_streak_reward(db, id))
    }

    pub fn delete_streak_reward(&self, id: String) -> Result<(), DbError> {
        self.with_db(|db| RewardsRepository::delete_streak_reward(db, &id))
    }

    /// Update a streak reward and its milestones atomically
    /// Preserves unlocked status for milestones with matching target_days
    pub fn update_streak_reward_with_milestones(
        &self,
        id: String,
        habit_id: String,
        is_consecutive: bool,
        target_days: Option<i32>,
        target_total: Option<i32>,
        milestones: Vec<(i32, String)>, // (target_days, reward_text)
    ) -> Result<(), DbError> {
        self.with_db(|db| {
            db.with_transaction(|db| {
                // Get existing milestones to preserve unlocked status
                let existing_milestones = RewardsRepository::get_milestones(db, &id)?;

                // Update the reward
                let reward = StreakReward::new(
                    id.clone(),
                    habit_id,
                    is_consecutive,
                    target_days,
                    target_total,
                );
                RewardsRepository::update_streak_reward(db, &reward)?;

                // Delete existing milestones
                RewardsRepository::delete_milestones_by_reward(db, &id)?;

                // Get current progress to determine which milestones should be unlocked
                let progress = RewardsRepository::get_streak_progress(
                    db,
                    &reward.habit_id,
                    reward.is_consecutive,
                    reward.target_total,
                )?;

                // Create new milestones, preserving unlocked status based on:
                // 1. Existing milestone with same target_days that was unlocked, OR
                // 2. Current progress already exceeds the milestone target
                for (days, text) in milestones {
                    let milestone_id = Uuid::new_v4().to_string();
                    let mut milestone = Milestone::new(milestone_id, id.clone(), days, text);

                    // Check if there was an existing milestone with the same target_days that was unlocked
                    let was_previously_unlocked = existing_milestones
                        .iter()
                        .any(|m| m.target_days == days && m.unlocked);

                    // Also check if current progress already exceeds this milestone
                    let should_be_unlocked = was_previously_unlocked || progress >= days;

                    if should_be_unlocked {
                        // Find the original unlocked_at if it exists
                        if let Some(existing) = existing_milestones
                            .iter()
                            .find(|m| m.target_days == days && m.unlocked)
                        {
                            milestone.unlocked = true;
                            milestone.unlocked_at = existing.unlocked_at.clone();
                        } else {
                            // Progress exceeds but wasn't previously unlocked - unlock now
                            milestone.unlock();
                        }
                    }

                    RewardsRepository::create_milestone(db, &milestone)?;
                }

                Ok(())
            })
        })
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

    pub fn archive_goal(&self, id: String) -> Result<(), DbError> {
        self.with_db(|db| RewardsRepository::archive_goal(db, &id))
    }

    pub fn update_goal(
        &self,
        id: String,
        name: String,
        description: String,
        reward_text: String,
        deadline: String,
    ) -> Result<(), DbError> {
        self.with_db(|db| {
            let mut goal = match RewardsRepository::get_goal(db, &id)? {
                Some(g) => g,
                None => return Err(DbError::GoalNotFound),
            };

            goal.name = name;
            goal.description = if description.is_empty() {
                None
            } else {
                Some(description)
            };
            goal.reward_text = reward_text;
            goal.deadline = if deadline.is_empty() {
                None
            } else {
                Some(deadline)
            };

            RewardsRepository::update_goal(db, &goal)
        })
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

    pub fn delete_checkpoint(&self, checkpoint_id: String) -> Result<(), DbError> {
        self.with_db(|db| RewardsRepository::delete_checkpoint(db, &checkpoint_id))
    }

    pub fn update_checkpoint(
        &self,
        checkpoint_id: String,
        description: String,
    ) -> Result<(), DbError> {
        self.with_db(|db| {
            let mut checkpoint = match RewardsRepository::get_checkpoint(db, &checkpoint_id)? {
                Some(checkpoint) => checkpoint,
                None => return Err(DbError::Sqlite(RusqliteError::QueryReturnedNoRows)),
            };
            checkpoint.description = description;
            RewardsRepository::update_checkpoint(db, &checkpoint)
        })
    }

    pub fn update_goal_with_checkpoints(
        &self,
        goal_id: String,
        name: String,
        description: String,
        reward_text: String,
        deadline: String,
        checkpoints: Vec<(Option<String>, String, i32)>, // (id, text, sort_order)
    ) -> Result<(), DbError> {
        self.with_db(|db| {
            db.with_transaction(|db| {
                let mut goal = match RewardsRepository::get_goal(db, &goal_id)? {
                    Some(g) => g,
                    None => return Err(DbError::GoalNotFound),
                };

                goal.name = name;
                goal.description = if description.is_empty() {
                    None
                } else {
                    Some(description)
                };
                goal.reward_text = reward_text;
                goal.deadline = if deadline.is_empty() {
                    None
                } else {
                    Some(deadline)
                };

                RewardsRepository::update_goal(db, &goal)?;

                let existing = RewardsRepository::get_checkpoints(db, &goal_id)?;
                let mut keep_ids: Vec<String> = Vec::new();

                for (cp_id, _, _) in &checkpoints {
                    if let Some(id) = cp_id.as_ref() {
                        keep_ids.push(id.clone());
                    }
                }

                for cp in existing {
                    if !keep_ids.iter().any(|id| id == &cp.id) {
                        RewardsRepository::delete_checkpoint(db, &cp.id)?;
                    }
                }

                for (cp_id, text, order) in checkpoints {
                    if let Some(checkpoint_id) = cp_id {
                        match RewardsRepository::get_checkpoint(db, &checkpoint_id)? {
                            Some(mut checkpoint) => {
                                checkpoint.description = text;
                                checkpoint.sort_order = order;
                                RewardsRepository::update_checkpoint(db, &checkpoint)?;
                            }
                            None => {
                                let new_id = Uuid::new_v4().to_string();
                                let checkpoint =
                                    Checkpoint::new(new_id, goal_id.clone(), text, order);
                                RewardsRepository::create_checkpoint(db, &checkpoint)?;
                            }
                        }
                    } else {
                        let new_id = Uuid::new_v4().to_string();
                        let checkpoint = Checkpoint::new(new_id, goal_id.clone(), text, order);
                        RewardsRepository::create_checkpoint(db, &checkpoint)?;
                    }
                }

                Ok(())
            })
        })
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
