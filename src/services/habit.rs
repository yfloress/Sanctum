use crate::db::{Database, DbError};
use crate::models::{Habit, HabitLog};
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

pub struct HabitService {
    db: Arc<Mutex<Option<Database>>>,
}

impl HabitService {
    pub fn new(db: Arc<Mutex<Option<Database>>>) -> Self {
        Self { db }
    }

    fn get_db<F, T>(&self, f: F) -> Result<T, DbError>
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

    pub fn create_habit(
        &self,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Local::now().to_rfc3339();

        let habit = Habit {
            id: id.clone(),
            name,
            description,
            color,
            category,
            created_at: now,
            archived: false,
        };

        self.get_db(|db| {
            db.create_habit(&habit)?;
            Ok(())
        })?;

        Ok(id)
    }

    pub fn get_habits(&self) -> Result<Vec<Habit>, DbError> {
        self.get_db(|db| db.get_habits())
    }

    pub fn update_habit(
        &self,
        id: String,
        name: String,
        description: Option<String>,
        color: String,
        category: String,
        is_archived: bool,
    ) -> Result<(), DbError> {
        self.get_db(|db| {
            match db.get_habit(&id)? {
                Some(mut habit) => {
                    habit.name = name.clone();
                    habit.description = description.clone();
                    habit.color = color.clone();
                    habit.category = category.clone();
                    // habit.archived = is_archived; // DB update doesn't support this

                    db.update_habit(&habit)?;

                    if is_archived {
                        db.archive_habit(&id)?;
                    }
                    Ok(())
                }
                None => Err(DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows)),
            }
        })
    }

    pub fn archive_habit(&self, id: String) -> Result<(), DbError> {
        self.get_db(|db| db.archive_habit(&id))
    }

    pub fn delete_habit(&self, id: String) -> Result<(), DbError> {
        self.get_db(|db| db.delete_habit(&id))
    }

    pub fn toggle_habit_completion(&self, habit_id: String, date: String) -> Result<bool, DbError> {
        let (active, _) = self.get_db(|db| db.toggle_habit_log(&habit_id, &date))?;
        Ok(active)
    }

    pub fn get_habit_logs(
        &self,
        start_date: String,
        end_date: String,
    ) -> Result<Vec<HabitLog>, DbError> {
        self.get_db(|db| db.get_habit_logs(&start_date, &end_date))
    }
}
