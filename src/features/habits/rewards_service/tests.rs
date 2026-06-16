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

use super::*;
use crate::db::Database;
use crate::models::{Habit, HabitLog};
use secrecy::SecretString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

struct TestHarness {
    service: RewardsService,
    db: Arc<Mutex<Option<Database>>>,
    test_dir: PathBuf,
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.test_dir);
    }
}

fn new_harness() -> TestHarness {
    let base_dir = std::env::temp_dir().join(format!("sanctum-rewards-test-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&base_dir).expect("create test dir");
    let db_path = base_dir.join("vault.db");
    let password = SecretString::from("test-password-123".to_string());
    let db = Database::init(db_path, &password).expect("init test database");
    let db_arc = Arc::new(Mutex::new(Some(db)));
    let service = RewardsService::new(db_arc.clone());
    TestHarness {
        service,
        db: db_arc,
        test_dir: base_dir,
    }
}

fn create_habit_log(db: &Database, habit_id: &str, date: &str) {
    let habit_log = HabitLog::new(
        Uuid::new_v4().to_string(),
        habit_id.to_string(),
        date.to_string(),
    );
    db.create_habit_log(&habit_log).expect("create habit log");
}

fn seed_habit(db: &Database) -> String {
    let id = Uuid::new_v4().to_string();
    let habit = Habit::new(
        id.clone(),
        "Test Habit".to_string(),
        None,
        "#8b5cf6".to_string(),
        "mind".to_string(),
        "2024-01-01T00:00:00Z".to_string(),
    );
    db.create_habit(&habit).expect("create habit");
    id
}

fn seed_habit_with_logs(db: &Database, count: i32) -> String {
    let h_id = seed_habit(db);
    for i in 0..count {
        let day = 1 + i;
        create_habit_log(db, &h_id, &format!("2024-06-{:02}", day));
    }
    h_id
}

fn seed_habit_with_logs_consecutive(db: &Database, count: i32, start_date: &str) -> String {
    let h_id = seed_habit(db);
    let start =
        chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d").expect("parse start date");
    for i in 0..count {
        let date = start + chrono::Duration::days(i as i64);
        create_habit_log(db, &h_id, &date.format("%Y-%m-%d").to_string());
    }
    h_id
}

// ==================== Streak Rewards ====================

#[test]
fn test_create_streak_reward_consecutive() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    let id = h
        .service
        .create_streak_reward(habit_id.clone(), true, Some(7), None)
        .expect("create reward");
    assert!(!id.is_empty());
    let reward = h.service.get_streak_reward(&id).expect("get reward");
    assert!(reward.is_some());
    let r = reward.unwrap();
    assert!(r.is_consecutive);
    assert_eq!(r.target_days, Some(7));
    assert_eq!(r.target_total, None);
}

#[test]
fn test_create_streak_reward_accumulative() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    let id = h
        .service
        .create_streak_reward(habit_id, false, Some(30), Some(30))
        .expect("create reward");
    let reward = h.service.get_streak_reward(&id).expect("get reward");
    assert!(reward.is_some());
    let r = reward.unwrap();
    assert!(!r.is_consecutive);
    assert_eq!(r.target_days, Some(30));
    assert_eq!(r.target_total, Some(30));
}

#[test]
fn test_get_streak_rewards_empty() {
    let h = new_harness();
    let rewards = h.service.get_streak_rewards().expect("get rewards");
    assert!(rewards.is_empty());
}

#[test]
fn test_get_streak_rewards_returns_all() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    h.service
        .create_streak_reward(habit_id.clone(), true, Some(3), None)
        .expect("r1");
    h.service
        .create_streak_reward(habit_id, false, Some(5), Some(5))
        .expect("r2");
    let rewards = h.service.get_streak_rewards().expect("get rewards");
    assert_eq!(rewards.len(), 2);
}

#[test]
fn test_get_streak_rewards_by_habit_filters() {
    let h = new_harness();
    let (h1, h2) = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        (seed_habit(db), seed_habit(db))
    };
    h.service
        .create_streak_reward(h1.clone(), true, Some(3), None)
        .expect("r1");
    h.service
        .create_streak_reward(h1.clone(), true, Some(7), None)
        .expect("r2");
    h.service
        .create_streak_reward(h2, true, Some(5), None)
        .expect("r3");
    let for_h1 = h.service.get_streak_rewards_by_habit(&h1).expect("filter");
    assert_eq!(for_h1.len(), 2);
}

#[test]
fn test_get_streak_reward_not_found() {
    let h = new_harness();
    let reward = h.service.get_streak_reward("nonexistent").expect("get");
    assert!(reward.is_none());
}

#[test]
fn test_delete_streak_reward_removes_it() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    let id = h
        .service
        .create_streak_reward(habit_id, true, Some(3), None)
        .expect("create");
    h.service.delete_streak_reward(id.clone()).expect("delete");
    let reward = h.service.get_streak_reward(&id).expect("get");
    assert!(reward.is_none());
}

// ==================== Milestones ====================

#[test]
fn test_add_milestone_to_reward() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    let reward_id = h
        .service
        .create_streak_reward(habit_id, true, Some(30), None)
        .expect("create");
    let m_id = h
        .service
        .add_milestone(reward_id.clone(), 7, "First week!".to_string())
        .expect("add milestone");
    assert!(!m_id.is_empty());
    let milestones = h
        .service
        .get_milestones(&reward_id)
        .expect("get milestones");
    assert_eq!(milestones.len(), 1);
    assert_eq!(milestones[0].target_days, 7);
    assert_eq!(milestones[0].reward_text, "First week!");
    assert!(!milestones[0].unlocked);
}

#[test]
fn test_get_milestones_empty_for_new_reward() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    let reward_id = h
        .service
        .create_streak_reward(habit_id, true, Some(30), None)
        .expect("create");
    let milestones = h
        .service
        .get_milestones(&reward_id)
        .expect("get milestones");
    assert!(milestones.is_empty());
}

#[test]
fn test_check_and_unlock_milestones_returns_empty_when_no_progress() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    let reward_id = h
        .service
        .create_streak_reward(habit_id, true, Some(30), None)
        .expect("create");
    h.service
        .add_milestone(reward_id.clone(), 5, "5 days".to_string())
        .expect("add m");
    h.service
        .add_milestone(reward_id.clone(), 10, "10 days".to_string())
        .expect("add m");
    let unlocked = h
        .service
        .check_and_unlock_milestones(&reward_id)
        .expect("check");
    assert!(unlocked.is_empty(), "no progress should unlock nothing");
}

#[test]
fn test_check_and_unlock_milestones_unlocks_reached_milestones() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        // Use logs near today (2026-06-15) for consecutive streak to work
        seed_habit_with_logs_consecutive(db, 7, "2026-06-09")
    };
    let reward_id = h
        .service
        .create_streak_reward(habit_id, true, Some(30), None)
        .expect("create");
    h.service
        .add_milestone(reward_id.clone(), 5, "5 days".to_string())
        .expect("add m");
    h.service
        .add_milestone(reward_id.clone(), 10, "10 days".to_string())
        .expect("add m");
    let unlocked = h
        .service
        .check_and_unlock_milestones(&reward_id)
        .expect("check");
    assert_eq!(unlocked.len(), 1, "only the 5-day milestone should unlock");
    let milestones = h
        .service
        .get_milestones(&reward_id)
        .expect("get milestones");
    let m5 = milestones
        .iter()
        .find(|m| m.target_days == 5)
        .expect("find 5-day");
    let m10 = milestones
        .iter()
        .find(|m| m.target_days == 10)
        .expect("find 10-day");
    assert!(m5.unlocked);
    assert!(!m10.unlocked);
}

// ==================== Update Streak Reward with Milestones ====================

#[test]
fn test_update_streak_reward_with_milestones_creates_new_milestones() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    let reward_id = h
        .service
        .create_streak_reward(habit_id.clone(), true, Some(30), None)
        .expect("create");
    let milestones = vec![(7, "Week 1".to_string()), (14, "Week 2".to_string())];
    h.service
        .update_streak_reward_with_milestones(
            reward_id.clone(),
            habit_id,
            true,
            Some(30),
            None,
            milestones,
        )
        .expect("update");
    let ms = h
        .service
        .get_milestones(&reward_id)
        .expect("get milestones");
    assert_eq!(ms.len(), 2);
}

#[test]
fn test_update_streak_reward_with_milestones_replaces_existing() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit(db)
    };
    let reward_id = h
        .service
        .create_streak_reward(habit_id.clone(), true, Some(30), None)
        .expect("create");
    h.service
        .add_milestone(reward_id.clone(), 7, "Old".to_string())
        .expect("add");
    let new_ms = vec![(14, "New".to_string())];
    h.service
        .update_streak_reward_with_milestones(
            reward_id.clone(),
            habit_id,
            true,
            Some(30),
            None,
            new_ms,
        )
        .expect("update");
    let ms = h
        .service
        .get_milestones(&reward_id)
        .expect("get milestones");
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0].target_days, 14);
}

#[test]
fn test_update_streak_reward_with_milestones_preserves_unlocked() {
    let h = new_harness();
    let habit_id = {
        let guard = h.db.lock().expect("lock");
        let db = guard.as_ref().expect("db");
        seed_habit_with_logs_consecutive(db, 7, "2026-06-09")
    };
    let reward_id = h
        .service
        .create_streak_reward(habit_id.clone(), true, Some(30), None)
        .expect("create");
    h.service
        .add_milestone(reward_id.clone(), 5, "5 days".to_string())
        .expect("add");
    h.service
        .check_and_unlock_milestones(&reward_id)
        .expect("check");
    let new_ms = vec![(5, "Updated 5".to_string()), (10, "New 10".to_string())];
    h.service
        .update_streak_reward_with_milestones(
            reward_id.clone(),
            habit_id,
            true,
            Some(30),
            None,
            new_ms,
        )
        .expect("update");
    let ms = h
        .service
        .get_milestones(&reward_id)
        .expect("get milestones");
    let m5 = ms.iter().find(|m| m.target_days == 5).expect("find 5-day");
    let m10 = ms
        .iter()
        .find(|m| m.target_days == 10)
        .expect("find 10-day");
    assert!(
        m5.unlocked,
        "previously unlocked milestone should stay unlocked"
    );
    assert!(m5.reward_text.contains("Updated"), "text should be updated");
    assert!(
        !m10.unlocked,
        "new milestone with target > progress should not unlock"
    );
}

// ==================== Goals ====================

#[test]
fn test_create_goal() {
    let h = new_harness();
    let id = h
        .service
        .create_goal(
            "Read 12 books".to_string(),
            Some("Read one per month".to_string()),
            "Bookworm badge".to_string(),
            Some("2024-12-31".to_string()),
        )
        .expect("create goal");
    assert!(!id.is_empty());
    let goal = h.service.get_goal(&id).expect("get goal");
    assert!(goal.is_some());
    let g = goal.unwrap();
    assert_eq!(g.name, "Read 12 books");
    assert_eq!(g.description.as_deref(), Some("Read one per month"));
    assert!(!g.is_completed);
    assert!(!g.archived);
}

#[test]
fn test_create_goal_minimal() {
    let h = new_harness();
    let id = h
        .service
        .create_goal("Minimal".to_string(), None, "Badge".to_string(), None)
        .expect("create goal");
    let goal = h.service.get_goal(&id).expect("get goal").expect("found");
    assert_eq!(goal.name, "Minimal");
    assert!(goal.description.is_none());
    assert!(goal.deadline.is_none());
}

#[test]
fn test_get_goals_empty() {
    let h = new_harness();
    let goals = h.service.get_goals().expect("get goals");
    assert!(goals.is_empty());
}

#[test]
fn test_get_goals_returns_all() {
    let h = new_harness();
    h.service
        .create_goal("Goal A".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    h.service
        .create_goal("Goal B".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let goals = h.service.get_goals().expect("get goals");
    assert_eq!(goals.len(), 2);
}

#[test]
fn test_get_goal_not_found() {
    let h = new_harness();
    let goal = h.service.get_goal("nonexistent").expect("get");
    assert!(goal.is_none());
}

#[test]
fn test_update_goal_changes_fields() {
    let h = new_harness();
    let id = h
        .service
        .create_goal("Old".to_string(), None, "Old badge".to_string(), None)
        .expect("create");
    h.service
        .update_goal(
            id.clone(),
            "New".to_string(),
            "New desc".to_string(),
            "New badge".to_string(),
            "2024-06-30".to_string(),
        )
        .expect("update");
    let goal = h.service.get_goal(&id).expect("get").expect("found");
    assert_eq!(goal.name, "New");
    assert_eq!(goal.description.as_deref(), Some("New desc"));
    assert_eq!(goal.deadline.as_deref(), Some("2024-06-30"));
}

#[test]
fn test_archive_goal() {
    let h = new_harness();
    let id = h
        .service
        .create_goal("Archivable".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    h.service.archive_goal(id.clone()).expect("archive");
    let goal = h.service.get_goal(&id).expect("get").expect("found");
    assert!(goal.archived);
}

#[test]
fn test_delete_goal_removes_it() {
    let h = new_harness();
    let id = h
        .service
        .create_goal("Deletable".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    h.service.delete_goal(id.clone()).expect("delete");
    let goal = h.service.get_goal(&id).expect("get");
    assert!(goal.is_none());
}

#[test]
fn test_complete_goal_creates_achievement() {
    let h = new_harness();
    let id = h
        .service
        .create_goal(
            "Finish project".to_string(),
            None,
            "Ship it".to_string(),
            None,
        )
        .expect("create");
    let ach_id = h.service.complete_goal(id.clone()).expect("complete");
    assert!(ach_id.is_some(), "should return achievement id");
    let ach_id = ach_id.unwrap();
    assert!(!ach_id.is_empty());
    let goal = h.service.get_goal(&id).expect("get").expect("found");
    assert!(goal.is_completed);
    let achievements = h.service.get_achievements().expect("get achievements");
    assert!(achievements.iter().any(|a| a.id == ach_id));
}

#[test]
fn test_complete_goal_twice_returns_none() {
    let h = new_harness();
    let id = h
        .service
        .create_goal("One time".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    h.service.complete_goal(id.clone()).expect("first complete");
    let second = h.service.complete_goal(id).expect("second complete");
    assert!(second.is_none(), "completing twice should return None");
}

// ==================== Checkpoints ====================

#[test]
fn test_add_checkpoint_auto_increments_order() {
    let h = new_harness();
    let goal_id = h
        .service
        .create_goal("Goal".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let c1 = h
        .service
        .add_checkpoint(goal_id.clone(), "Step 1".to_string())
        .expect("add c1");
    let c2 = h
        .service
        .add_checkpoint(goal_id.clone(), "Step 2".to_string())
        .expect("add c2");
    let cps = h
        .service
        .get_checkpoints(&goal_id)
        .expect("get checkpoints");
    assert_eq!(cps.len(), 2);
    let c1_data = cps.iter().find(|c| c.id == c1).expect("find c1");
    let c2_data = cps.iter().find(|c| c.id == c2).expect("find c2");
    assert_eq!(c1_data.sort_order, 0);
    assert_eq!(c2_data.sort_order, 1);
}

#[test]
fn test_get_checkpoints_empty_for_new_goal() {
    let h = new_harness();
    let goal_id = h
        .service
        .create_goal("Goal".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let cps = h
        .service
        .get_checkpoints(&goal_id)
        .expect("get checkpoints");
    assert!(cps.is_empty());
}

#[test]
fn test_delete_checkpoint_removes_it() {
    let h = new_harness();
    let goal_id = h
        .service
        .create_goal("Goal".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let cp_id = h
        .service
        .add_checkpoint(goal_id.clone(), "Temp".to_string())
        .expect("add");
    h.service.delete_checkpoint(cp_id.clone()).expect("delete");
    let cps = h
        .service
        .get_checkpoints(&goal_id)
        .expect("get checkpoints");
    assert!(cps.iter().all(|c| c.id != cp_id));
}

#[test]
fn test_update_checkpoint_changes_description() {
    let h = new_harness();
    let goal_id = h
        .service
        .create_goal("Goal".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let cp_id = h
        .service
        .add_checkpoint(goal_id, "Old desc".to_string())
        .expect("add");
    h.service
        .update_checkpoint(cp_id.clone(), "New desc".to_string())
        .expect("update");
    let cps = h
        .service
        .get_checkpoints(&h.service.get_goals().expect("goals")[0].id)
        .expect("get checkpoints");
    let cp = cps.iter().find(|c| c.id == cp_id).expect("find");
    assert_eq!(cp.description, "New desc");
}

#[test]
fn test_toggle_checkpoint_toggles_completion() {
    let h = new_harness();
    let goal_id = h
        .service
        .create_goal("Goal".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let cp_id = h
        .service
        .add_checkpoint(goal_id.clone(), "Step".to_string())
        .expect("add");
    let result = h
        .service
        .toggle_checkpoint(goal_id.clone(), cp_id.clone())
        .expect("toggle on");
    assert!(result, "first toggle should complete");
    let result = h
        .service
        .toggle_checkpoint(goal_id, cp_id.clone())
        .expect("toggle off");
    assert!(!result, "second toggle should uncomplete");
}

#[test]
fn test_toggle_checkpoint_not_found_returns_false() {
    let h = new_harness();
    let result = h
        .service
        .toggle_checkpoint("nonexistent".to_string(), "bad-id".to_string())
        .expect("toggle");
    assert!(!result);
}

#[test]
fn test_toggle_checkpoint_completes_goal_when_all_done() {
    let h = new_harness();
    let goal_id = h
        .service
        .create_goal("Multi-step".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let c1 = h
        .service
        .add_checkpoint(goal_id.clone(), "Step 1".to_string())
        .expect("add c1");
    let c2 = h
        .service
        .add_checkpoint(goal_id.clone(), "Step 2".to_string())
        .expect("add c2");
    h.service
        .toggle_checkpoint(goal_id.clone(), c1)
        .expect("complete c1");
    h.service
        .toggle_checkpoint(goal_id.clone(), c2)
        .expect("complete c2");
    let goal = h.service.get_goal(&goal_id).expect("get").expect("found");
    assert!(
        goal.is_completed,
        "goal should auto-complete when all checkpoints done"
    );
    let achievements = h.service.get_achievements().expect("get achievements");
    assert!(
        achievements.iter().any(|a| a.source_id == goal_id),
        "should create achievement for completed goal"
    );
}

#[test]
fn test_get_checkpoint_progress_tracks_counts() {
    let h = new_harness();
    let goal_id = h
        .service
        .create_goal("Goal".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let c1 = h
        .service
        .add_checkpoint(goal_id.clone(), "A".to_string())
        .expect("add");
    let c2 = h
        .service
        .add_checkpoint(goal_id.clone(), "B".to_string())
        .expect("add");
    let (total, completed) = h
        .service
        .get_checkpoint_progress(&goal_id)
        .expect("progress");
    assert_eq!(total, 2);
    assert_eq!(completed, 0);
    h.service
        .toggle_checkpoint(goal_id.clone(), c1)
        .expect("complete one");
    let (_, completed) = h
        .service
        .get_checkpoint_progress(&goal_id)
        .expect("progress");
    assert_eq!(completed, 1);
    h.service
        .toggle_checkpoint(goal_id, c2)
        .expect("complete two");
    let (total, completed) = h
        .service
        .get_checkpoint_progress(&h.service.get_goals().expect("goals")[0].id)
        .expect("progress");
    assert_eq!(completed, 2, "both checkpoints done");
    assert_eq!(total, 2);
}

// ==================== Achievements ====================

#[test]
fn test_get_achievements_empty() {
    let h = new_harness();
    let achievements = h.service.get_achievements().expect("get");
    assert!(achievements.is_empty());
}

#[test]
fn test_achievements_created_via_goal_completion() {
    let h = new_harness();
    let id = h
        .service
        .create_goal("Achieve".to_string(), None, "Trophy".to_string(), None)
        .expect("create");
    h.service.complete_goal(id).expect("complete");
    let achievements = h.service.get_achievements().expect("get");
    assert_eq!(achievements.len(), 1);
    assert_eq!(achievements[0].achievement_type, "goal");
}

#[test]
fn test_achievements_dedup_on_repeat_completion() {
    let h = new_harness();
    let id = h
        .service
        .create_goal("Unique".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    h.service.complete_goal(id.clone()).expect("first");
    h.service.complete_goal(id).expect("second"); // should be deduped
    let achievements = h.service.get_achievements().expect("get");
    assert_eq!(
        achievements.len(),
        1,
        "should only create one achievement per goal"
    );
}

#[test]
fn test_update_goal_with_checkpoints_replaces_checkpoints() {
    let h = new_harness();
    let goal_id = h
        .service
        .create_goal("Goal".to_string(), None, "Badge".to_string(), None)
        .expect("create");
    let old_cp = h
        .service
        .add_checkpoint(goal_id.clone(), "Old".to_string())
        .expect("add old");
    let new_checkpoints = vec![
        (Some(old_cp.clone()), "Updated".to_string(), 0),
        (None, "New Step".to_string(), 1),
    ];
    h.service
        .update_goal_with_checkpoints(
            goal_id.clone(),
            "Updated Goal".to_string(),
            "Desc".to_string(),
            "Badge".to_string(),
            "".to_string(),
            new_checkpoints,
        )
        .expect("update");
    let cps = h
        .service
        .get_checkpoints(&goal_id)
        .expect("get checkpoints");
    assert_eq!(cps.len(), 2);
    let updated = cps.iter().find(|c| c.id == old_cp).expect("find old cp");
    assert_eq!(updated.description, "Updated");
    let goal = h.service.get_goal(&goal_id).expect("get").expect("found");
    assert_eq!(goal.name, "Updated Goal");
}
