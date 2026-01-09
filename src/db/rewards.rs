//! Rewards database operations
//!
//! CRUD operations for streak rewards, milestones, goals, checkpoints, and achievements.

use super::{Database, DbError};
use crate::models::{Achievement, Checkpoint, Goal, Milestone, StreakReward};
use rusqlite::params;

impl Database {
    // ==================== Streak Rewards ====================

    pub fn create_streak_reward(&self, reward: &StreakReward) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO streak_rewards (id, habit_id, is_consecutive, target_days, target_total, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &reward.id,
                &reward.habit_id,
                reward.is_consecutive as i32,
                &reward.target_days,
                &reward.target_total,
                &reward.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_streak_rewards(&self) -> Result<Vec<StreakReward>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, habit_id, is_consecutive, target_days, target_total, created_at
             FROM streak_rewards ORDER BY created_at DESC",
        )?;

        let rewards = stmt
            .query_map([], |row| {
                Ok(StreakReward {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    is_consecutive: row.get::<_, i32>(2)? != 0,
                    target_days: row.get(3)?,
                    target_total: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rewards)
    }

    pub fn get_streak_rewards_by_habit(
        &self,
        habit_id: &str,
    ) -> Result<Vec<StreakReward>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, habit_id, is_consecutive, target_days, target_total, created_at
             FROM streak_rewards WHERE habit_id = ?1 ORDER BY created_at DESC",
        )?;

        let rewards = stmt
            .query_map(params![habit_id], |row| {
                Ok(StreakReward {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    is_consecutive: row.get::<_, i32>(2)? != 0,
                    target_days: row.get(3)?,
                    target_total: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rewards)
    }

    pub fn get_streak_reward(&self, id: &str) -> Result<Option<StreakReward>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, habit_id, is_consecutive, target_days, target_total, created_at
             FROM streak_rewards WHERE id = ?1",
            params![id],
            |row| {
                Ok(StreakReward {
                    id: row.get(0)?,
                    habit_id: row.get(1)?,
                    is_consecutive: row.get::<_, i32>(2)? != 0,
                    target_days: row.get(3)?,
                    target_total: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        );

        match result {
            Ok(reward) => Ok(Some(reward)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    pub fn delete_streak_reward(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM streak_rewards WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_streak_reward(&self, reward: &StreakReward) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE streak_rewards SET habit_id = ?1, is_consecutive = ?2, target_days = ?3, target_total = ?4 WHERE id = ?5",
            params![
                &reward.habit_id,
                reward.is_consecutive as i32,
                &reward.target_days,
                &reward.target_total,
                &reward.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_milestones_by_reward(&self, reward_id: &str) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM milestones WHERE reward_id = ?1",
            params![reward_id],
        )?;
        Ok(())
    }

    // ==================== Milestones ====================

    pub fn create_milestone(&self, milestone: &Milestone) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO milestones (id, reward_id, target_days, reward_text, unlocked, unlocked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &milestone.id,
                &milestone.reward_id,
                milestone.target_days,
                &milestone.reward_text,
                milestone.unlocked as i32,
                &milestone.unlocked_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_milestones(&self, reward_id: &str) -> Result<Vec<Milestone>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, reward_id, target_days, reward_text, unlocked, unlocked_at
             FROM milestones WHERE reward_id = ?1 ORDER BY target_days ASC",
        )?;

        let milestones = stmt
            .query_map(params![reward_id], |row| {
                Ok(Milestone {
                    id: row.get(0)?,
                    reward_id: row.get(1)?,
                    target_days: row.get(2)?,
                    reward_text: row.get(3)?,
                    unlocked: row.get::<_, i32>(4)? != 0,
                    unlocked_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(milestones)
    }

    pub fn update_milestone(&self, milestone: &Milestone) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE milestones SET unlocked = ?1, unlocked_at = ?2 WHERE id = ?3",
            params![
                milestone.unlocked as i32,
                &milestone.unlocked_at,
                &milestone.id
            ],
        )?;
        Ok(())
    }

    // ==================== Goals ====================

    pub fn create_goal(&self, goal: &Goal) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO goals (id, name, description, reward_text, deadline, is_completed, completed_at, created_at, archived)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &goal.id,
                &goal.name,
                &goal.description,
                &goal.reward_text,
                &goal.deadline,
                goal.is_completed as i32,
                &goal.completed_at,
                &goal.created_at,
                goal.archived as i32,
            ],
        )?;
        Ok(())
    }

    pub fn get_goals(&self) -> Result<Vec<Goal>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, reward_text, deadline, is_completed, completed_at, created_at, archived
             FROM goals WHERE archived = 0 ORDER BY is_completed ASC, created_at DESC",
        )?;

        let goals = stmt
            .query_map([], |row| {
                Ok(Goal {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    reward_text: row.get(3)?,
                    deadline: row.get(4)?,
                    is_completed: row.get::<_, i32>(5)? != 0,
                    completed_at: row.get(6)?,
                    created_at: row.get(7)?,
                    archived: row.get::<_, i32>(8)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(goals)
    }

    pub fn get_goal(&self, id: &str) -> Result<Option<Goal>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, name, description, reward_text, deadline, is_completed, completed_at, created_at, archived
             FROM goals WHERE id = ?1",
            params![id],
            |row| {
                Ok(Goal {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    reward_text: row.get(3)?,
                    deadline: row.get(4)?,
                    is_completed: row.get::<_, i32>(5)? != 0,
                    completed_at: row.get(6)?,
                    created_at: row.get(7)?,
                    archived: row.get::<_, i32>(8)? != 0,
                })
            },
        );

        match result {
            Ok(goal) => Ok(Some(goal)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    pub fn update_goal(&self, goal: &Goal) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE goals SET name = ?1, description = ?2, reward_text = ?3, deadline = ?4,
             is_completed = ?5, completed_at = ?6, archived = ?7 WHERE id = ?8",
            params![
                &goal.name,
                &goal.description,
                &goal.reward_text,
                &goal.deadline,
                goal.is_completed as i32,
                &goal.completed_at,
                goal.archived as i32,
                &goal.id,
            ],
        )?;
        Ok(())
    }

    pub fn archive_goal(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("UPDATE goals SET archived = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_goal(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM goals WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== Checkpoints ====================

    pub fn create_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO checkpoints (id, goal_id, description, completed, completed_at, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &checkpoint.id,
                &checkpoint.goal_id,
                &checkpoint.description,
                checkpoint.completed as i32,
                &checkpoint.completed_at,
                checkpoint.sort_order,
            ],
        )?;
        Ok(())
    }

    pub fn get_checkpoints(&self, goal_id: &str) -> Result<Vec<Checkpoint>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, goal_id, description, completed, completed_at, sort_order
             FROM checkpoints WHERE goal_id = ?1 ORDER BY sort_order ASC",
        )?;

        let checkpoints = stmt
            .query_map(params![goal_id], |row| {
                Ok(Checkpoint {
                    id: row.get(0)?,
                    goal_id: row.get(1)?,
                    description: row.get(2)?,
                    completed: row.get::<_, i32>(3)? != 0,
                    completed_at: row.get(4)?,
                    sort_order: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(checkpoints)
    }

    pub fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, goal_id, description, completed, completed_at, sort_order
             FROM checkpoints WHERE id = ?1",
            params![id],
            |row| {
                Ok(Checkpoint {
                    id: row.get(0)?,
                    goal_id: row.get(1)?,
                    description: row.get(2)?,
                    completed: row.get::<_, i32>(3)? != 0,
                    completed_at: row.get(4)?,
                    sort_order: row.get(5)?,
                })
            },
        );

        match result {
            Ok(cp) => Ok(Some(cp)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    pub fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE checkpoints SET completed = ?1, completed_at = ?2 WHERE id = ?3",
            params![
                checkpoint.completed as i32,
                &checkpoint.completed_at,
                &checkpoint.id
            ],
        )?;
        Ok(())
    }

    pub fn get_next_checkpoint_order(&self, goal_id: &str) -> Result<i32, DbError> {
        let max: Option<i32> = self
            .conn
            .query_row(
                "SELECT MAX(sort_order) FROM checkpoints WHERE goal_id = ?1",
                params![goal_id],
                |row| row.get(0),
            )
            .unwrap_or(None);
        Ok(max.unwrap_or(-1) + 1)
    }

    pub fn get_checkpoint_counts(&self, goal_id: &str) -> Result<(i32, i32), DbError> {
        let (total, completed): (i32, i32) = self.conn.query_row(
            "SELECT COUNT(*), SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END)
             FROM checkpoints WHERE goal_id = ?1",
            params![goal_id],
            |row| Ok((row.get(0)?, row.get::<_, Option<i32>>(1)?.unwrap_or(0))),
        )?;
        Ok((total, completed))
    }

    // ==================== Achievements ====================

    pub fn create_achievement(&self, achievement: &Achievement) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO achievements (id, title, description, icon_path, achievement_type, source_id, achieved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &achievement.id,
                &achievement.title,
                &achievement.description,
                &achievement.icon_path,
                &achievement.achievement_type,
                &achievement.source_id,
                &achievement.achieved_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_achievements(&self) -> Result<Vec<Achievement>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, icon_path, achievement_type, source_id, achieved_at
             FROM achievements ORDER BY achieved_at DESC",
        )?;

        let achievements = stmt
            .query_map([], |row| {
                Ok(Achievement {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    icon_path: row.get(3)?,
                    achievement_type: row.get(4)?,
                    source_id: row.get(5)?,
                    achieved_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(achievements)
    }

    pub fn achievement_exists(
        &self,
        source_id: &str,
        achievement_type: &str,
    ) -> Result<bool, DbError> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM achievements WHERE source_id = ?1 AND achievement_type = ?2",
            params![source_id, achievement_type],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // ==================== Streak Progress ====================

    pub fn get_streak_progress(
        &self,
        habit_id: &str,
        is_consecutive: bool,
        target_total: Option<i32>,
    ) -> Result<i32, DbError> {
        if is_consecutive {
            self.calculate_consecutive_streak(habit_id)
        } else {
            self.calculate_accumulative_progress(habit_id, target_total.unwrap_or(30))
        }
    }

    fn calculate_consecutive_streak(&self, habit_id: &str) -> Result<i32, DbError> {
        let today = chrono::Local::now().date_naive();
        let mut streak = 0;
        let mut check_date = today;

        loop {
            let date_str = check_date.format("%Y-%m-%d").to_string();
            if self.habit_log_exists(habit_id, &date_str)? {
                streak += 1;
                check_date = match check_date.pred_opt() {
                    Some(d) => d,
                    None => break,
                };
            } else if check_date == today {
                // Today not done yet, check yesterday
                check_date = match check_date.pred_opt() {
                    Some(d) => d,
                    None => break,
                };
            } else {
                break;
            }
        }
        Ok(streak)
    }

    fn calculate_accumulative_progress(&self, habit_id: &str, days: i32) -> Result<i32, DbError> {
        let today = chrono::Local::now().date_naive();
        let start = today - chrono::Duration::days(days as i64);

        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM habit_logs
             WHERE habit_id = ?1 AND completed_date >= ?2 AND completed_date <= ?3",
            params![
                habit_id,
                start.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string()
            ],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}
