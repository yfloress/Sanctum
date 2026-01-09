//! Rewards-related controller methods
//!
//! CRUD operations for streak rewards, goals, checkpoints, and achievements.

use super::{AppController, ControllerError, validate_uuid};
use crate::models::{Achievement, Checkpoint, Goal, Milestone, StreakReward};
use chrono::NaiveDate;

impl AppController {
    // ==================== Streak Rewards ====================

    pub fn create_streak_reward(
        &self,
        habit_id: String,
        is_consecutive: bool,
        target_days: i32,
        target_total: i32,
    ) -> Result<String, ControllerError> {
        if validate_uuid(&habit_id).is_err() {
            return Err(ControllerError::Validation("Invalid habit UUID".into()));
        }

        let (days_opt, total_opt) = if is_consecutive {
            (None, None)
        } else {
            if target_days <= 0 || target_total <= 0 {
                return Err(ControllerError::Validation(
                    "Target days and total must be positive".into(),
                ));
            }
            if target_days > target_total {
                return Err(ControllerError::Validation(
                    "Target days cannot exceed total days".into(),
                ));
            }
            (Some(target_days), Some(target_total))
        };

        self.rewards_service
            .create_streak_reward(habit_id, is_consecutive, days_opt, total_opt)
            .map_err(ControllerError::Database)
    }

    pub fn get_streak_rewards(&self) -> Result<Vec<StreakReward>, ControllerError> {
        self.rewards_service
            .get_streak_rewards()
            .map_err(ControllerError::Database)
    }

    pub fn get_streak_rewards_by_habit(
        &self,
        habit_id: &str,
    ) -> Result<Vec<StreakReward>, ControllerError> {
        if validate_uuid(habit_id).is_err() {
            return Err(ControllerError::Validation("Invalid habit UUID".into()));
        }
        self.rewards_service
            .get_streak_rewards_by_habit(habit_id)
            .map_err(ControllerError::Database)
    }

    /// Update a streak reward with its milestones atomically
    pub fn update_streak_reward_with_milestones(
        &self,
        id: String,
        habit_id: String,
        is_consecutive: bool,
        target_days: i32,
        target_total: i32,
        milestones: Vec<(i32, String)>, // (target_days, reward_text)
    ) -> Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid reward UUID".into()));
        }
        if validate_uuid(&habit_id).is_err() {
            return Err(ControllerError::Validation("Invalid habit UUID".into()));
        }

        // Validate milestones
        for (days, text) in &milestones {
            if *days <= 0 {
                return Err(ControllerError::Validation(
                    "Milestone target days must be positive".into(),
                ));
            }
            if text.trim().is_empty() {
                return Err(ControllerError::Validation(
                    "Milestone reward text cannot be empty".into(),
                ));
            }
        }

        let (days_opt, total_opt) = if is_consecutive {
            (None, None)
        } else {
            if target_days <= 0 || target_total <= 0 {
                return Err(ControllerError::Validation(
                    "Target days and total must be positive".into(),
                ));
            }
            if target_days > target_total {
                return Err(ControllerError::Validation(
                    "Target days cannot exceed total days".into(),
                ));
            }
            (Some(target_days), Some(target_total))
        };

        self.rewards_service
            .update_streak_reward_with_milestones(
                id,
                habit_id,
                is_consecutive,
                days_opt,
                total_opt,
                milestones,
            )
            .map_err(ControllerError::Database)
    }

    pub fn get_streak_progress(&self, reward_id: String) -> Result<i32, ControllerError> {
        if validate_uuid(&reward_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }

        let reward = self
            .rewards_service
            .get_streak_reward(&reward_id)
            .map_err(ControllerError::Database)?
            .ok_or_else(|| ControllerError::Validation("Reward not found".into()))?;

        self.rewards_service
            .get_streak_progress(&reward)
            .map_err(ControllerError::Database)
    }

    pub fn delete_streak_reward(&self, id: String) -> Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .delete_streak_reward(id)
            .map_err(ControllerError::Database)
    }

    // ==================== Milestones ====================

    pub fn add_milestone(
        &self,
        reward_id: String,
        target_days: i32,
        reward_text: String,
    ) -> Result<String, ControllerError> {
        if validate_uuid(&reward_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        if target_days <= 0 {
            return Err(ControllerError::Validation(
                "Target days must be positive".into(),
            ));
        }
        if reward_text.trim().is_empty() {
            return Err(ControllerError::Validation(
                "Reward text cannot be empty".into(),
            ));
        }

        self.rewards_service
            .add_milestone(reward_id, target_days, reward_text)
            .map_err(ControllerError::Database)
    }

    pub fn get_milestones(&self, reward_id: String) -> Result<Vec<Milestone>, ControllerError> {
        if validate_uuid(&reward_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .get_milestones(&reward_id)
            .map_err(ControllerError::Database)
    }

    pub fn check_and_unlock_milestones(
        &self,
        reward_id: String,
    ) -> Result<Vec<String>, ControllerError> {
        if validate_uuid(&reward_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .check_and_unlock_milestones(&reward_id)
            .map_err(ControllerError::Database)
    }

    // ==================== Goals ====================

    pub fn create_goal(
        &self,
        name: String,
        description: String,
        reward_text: String,
        deadline: String,
    ) -> Result<String, ControllerError> {
        if name.trim().is_empty() {
            return Err(ControllerError::Validation(
                "Goal name cannot be empty".into(),
            ));
        }
        if reward_text.trim().is_empty() {
            return Err(ControllerError::Validation(
                "Reward text cannot be empty".into(),
            ));
        }

        let deadline_opt = if deadline.trim().is_empty() {
            None
        } else {
            if NaiveDate::parse_from_str(&deadline, "%Y-%m-%d").is_err() {
                return Err(ControllerError::Validation(
                    "Invalid deadline format. Use YYYY-MM-DD".into(),
                ));
            }
            Some(deadline)
        };

        let desc_opt = if description.trim().is_empty() {
            None
        } else {
            Some(description)
        };

        self.rewards_service
            .create_goal(name, desc_opt, reward_text, deadline_opt)
            .map_err(ControllerError::Database)
    }

    pub fn get_goals(&self) -> Result<Vec<Goal>, ControllerError> {
        self.rewards_service
            .get_goals()
            .map_err(ControllerError::Database)
    }

    pub fn delete_goal(&self, id: String) -> Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .delete_goal(id)
            .map_err(ControllerError::Database)
    }

    pub fn complete_goal(&self, id: String) -> Result<Option<String>, ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .complete_goal(id)
            .map_err(ControllerError::Database)
    }

    pub fn archive_goal(&self, id: String) -> Result<(), ControllerError> {
        if validate_uuid(&id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .archive_goal(id)
            .map_err(ControllerError::Database)
    }

    // ==================== Checkpoints ====================

    pub fn add_checkpoint(
        &self,
        goal_id: String,
        description: String,
    ) -> Result<String, ControllerError> {
        if validate_uuid(&goal_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        if description.trim().is_empty() {
            return Err(ControllerError::Validation(
                "Checkpoint description cannot be empty".into(),
            ));
        }

        self.rewards_service
            .add_checkpoint(goal_id, description)
            .map_err(ControllerError::Database)
    }

    pub fn get_checkpoints(&self, goal_id: String) -> Result<Vec<Checkpoint>, ControllerError> {
        if validate_uuid(&goal_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .get_checkpoints(&goal_id)
            .map_err(ControllerError::Database)
    }

    pub fn toggle_checkpoint(
        &self,
        goal_id: String,
        checkpoint_id: String,
    ) -> Result<bool, ControllerError> {
        if validate_uuid(&goal_id).is_err() || validate_uuid(&checkpoint_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .toggle_checkpoint(goal_id, checkpoint_id)
            .map_err(ControllerError::Database)
    }

    pub fn get_checkpoint_progress(&self, goal_id: String) -> Result<(i32, i32), ControllerError> {
        if validate_uuid(&goal_id).is_err() {
            return Err(ControllerError::Validation("Invalid UUID".into()));
        }
        self.rewards_service
            .get_checkpoint_progress(&goal_id)
            .map_err(ControllerError::Database)
    }

    // ==================== Achievements ====================

    pub fn get_achievements(&self) -> Result<Vec<Achievement>, ControllerError> {
        self.rewards_service
            .get_achievements()
            .map_err(ControllerError::Database)
    }
}
