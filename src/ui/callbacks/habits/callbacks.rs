//! Habit callback registrations

use crate::RewardsAdapter;
use crate::controller::AppController;
use crate::{AppWindow, HabitAdapter};
use chrono::{Datelike, NaiveDate};
use slint::{ComponentHandle, Model, SharedString, Weak};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::charts::{HabitChartsCache, refresh_habit_analytics};
use super::data::{refresh_habit_summary, reload_habits, reload_heatmap};

/// Sets up all HabitAdapter callbacks
pub fn setup_habit_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    current_habit_date: Arc<Mutex<NaiveDate>>,
    current_heatmap_year: Arc<Mutex<i32>>,
    habit_analytics_cache: Rc<RefCell<HabitChartsCache>>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + 'static,
{
    // on_load_initial_data
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>().on_load_initial_data(move || {
            let now = chrono::Local::now().date_naive();
            *date_lock.lock().unwrap() = now;
            *year_lock.lock().unwrap() = now.year();
            reload_habits(&ui_weak, &controller, now, Some(&notify));
        });
    }

    // on_fetch_habits
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>()
            .on_fetch_habits(move |month, year| {
                if let Some(date) = NaiveDate::from_ymd_opt(year, month as u32, 1) {
                    *date_lock.lock().unwrap() = date;
                    reload_habits(&ui_weak, &controller, date, Some(&notify));
                }
            });
    }

    // on_create_habit
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        let analytics_cache = habit_analytics_cache.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>().on_create_habit(
            move |name, desc, color, category| -> SharedString {
                let description = if desc.is_empty() {
                    None
                } else {
                    Some(desc.to_string())
                };
                let result = controller.create_habit(
                    name.to_string(),
                    description,
                    color.to_string(),
                    category.to_string(),
                );
                match result {
                    Ok(_) => {
                        let d = *date_lock.lock().unwrap();
                        let y = *year_lock.lock().unwrap();
                        reload_habits(&ui_weak, &controller, d, Some(&notify));
                        reload_heatmap(&ui_weak, &controller, y);
                        refresh_habit_analytics(&ui_weak, &controller, &analytics_cache, &notify);
                        notify("Habit created".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_update_habit
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        let analytics_cache = habit_analytics_cache.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>().on_update_habit(
            move |id, name, desc, color, category| -> SharedString {
                let description = if desc.is_empty() {
                    None
                } else {
                    Some(desc.to_string())
                };
                let result = controller.update_habit(
                    id.to_string(),
                    name.to_string(),
                    description,
                    color.to_string(),
                    category.to_string(),
                    false,
                );
                match result {
                    Ok(_) => {
                        let d = *date_lock.lock().unwrap();
                        let y = *year_lock.lock().unwrap();
                        reload_habits(&ui_weak, &controller, d, Some(&notify));
                        reload_heatmap(&ui_weak, &controller, y);
                        refresh_habit_analytics(&ui_weak, &controller, &analytics_cache, &notify);
                        notify("Habit updated".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            },
        );
    }

    // on_delete_habit
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        let analytics_cache = habit_analytics_cache.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>()
            .on_delete_habit(move |id| -> SharedString {
                let result = controller.delete_habit(id.to_string());
                match result {
                    Ok(_) => {
                        let d = *date_lock.lock().unwrap();
                        let y = *year_lock.lock().unwrap();
                        reload_habits(&ui_weak, &controller, d, Some(&notify));
                        reload_heatmap(&ui_weak, &controller, y);
                        refresh_habit_analytics(&ui_weak, &controller, &analytics_cache, &notify);
                        notify("Habit deleted".into(), false);
                        SharedString::from("")
                    }
                    Err(e) => SharedString::from(e.to_string()),
                }
            });
    }

    // on_toggle_habit
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let year_lock = current_heatmap_year.clone();
        let analytics_cache = habit_analytics_cache.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>()
            .on_toggle_habit(move |id, date| {
                let habit_id = id.to_string();
                match controller.toggle_habit_completion(habit_id.clone(), date.to_string()) {
                    Ok(_) => {
                        let d = *date_lock.lock().unwrap();
                        let y = *year_lock.lock().unwrap();
                        reload_habits(&ui_weak, &controller, d, Some(&notify));
                        reload_heatmap(&ui_weak, &controller, y);
                        refresh_habit_analytics(&ui_weak, &controller, &analytics_cache, &notify);

                        // Check and unlock milestones for all streak rewards linked to this habit
                        if let Ok(rewards) = controller.get_streak_rewards_by_habit(&habit_id) {
                            for reward in rewards {
                                if let Ok(unlocked) =
                                    controller.check_and_unlock_milestones(reward.id.clone())
                                    && !unlocked.is_empty()
                                {
                                    notify(
                                        format!("{} milestone(s) unlocked!", unlocked.len()),
                                        false,
                                    );
                                }
                            }
                            // Refresh rewards UI
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.global::<RewardsAdapter>().invoke_fetch_rewards();
                            }
                        }
                    }
                    Err(e) => {
                        notify(format!("Failed to toggle habit: {}", e), true);
                    }
                }
            });
    }

    // on_prev_month
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>().on_prev_month(move || {
            let mut d = date_lock.lock().unwrap();
            let month = d.month();
            let year = d.year();
            let (new_y, new_m) = if month == 1 {
                (year - 1, 12)
            } else {
                (year, month - 1)
            };
            *d = NaiveDate::from_ymd_opt(new_y, new_m, 1).unwrap();
            reload_habits(&ui_weak, &controller, *d, Some(&notify));
        });
    }

    // on_next_month
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>().on_next_month(move || {
            let mut d = date_lock.lock().unwrap();
            let month = d.month();
            let year = d.year();
            let (new_y, new_m) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };
            *d = NaiveDate::from_ymd_opt(new_y, new_m, 1).unwrap();
            reload_habits(&ui_weak, &controller, *d, Some(&notify));
        });
    }

    // on_fetch_heatmap_data
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let year_lock = current_heatmap_year.clone();
        ui.global::<HabitAdapter>().on_fetch_heatmap_data(move || {
            let y = *year_lock.lock().unwrap();
            reload_heatmap(&ui_weak, &controller, y);
        });
    }

    // on_prev_heatmap_year
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let year_lock = current_heatmap_year.clone();
        ui.global::<HabitAdapter>().on_prev_heatmap_year(move || {
            let mut y = year_lock.lock().unwrap();
            *y -= 1;
            reload_heatmap(&ui_weak, &controller, *y);
        });
    }

    // on_next_heatmap_year
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let year_lock = current_heatmap_year.clone();
        ui.global::<HabitAdapter>().on_next_heatmap_year(move || {
            let mut y = year_lock.lock().unwrap();
            *y += 1;
            reload_heatmap(&ui_weak, &controller, *y);
        });
    }

    // on_fetch_habit_analytics
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let analytics_cache = habit_analytics_cache.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>()
            .on_fetch_habit_analytics(move || {
                refresh_habit_analytics(&ui_weak, &controller, &analytics_cache, &notify);
            });
    }

    // on_select_habit
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let date_lock = current_habit_date.clone();
        let notify = notify.clone();
        ui.global::<HabitAdapter>().on_select_habit(move |id| {
            let date = *date_lock.lock().unwrap();
            refresh_habit_summary(&ui_weak, &controller, date, id.to_string(), Some(&notify));

            // Also update selected-habit-index for use in modals
            if let Some(ui) = ui_weak.upgrade() {
                let habits = ui.global::<HabitAdapter>().get_habits();
                for i in 0..habits.row_count() {
                    if let Some(habit) = habits.row_data(i)
                        && habit.id == id
                    {
                        ui.global::<HabitAdapter>().set_selected_habit_index(i as i32);
                        break;
                    }
                }
            }
        });
    }

    // on_find_habit_index - find habit index from id
    {
        let ui_weak = ui_weak.clone();
        ui.global::<HabitAdapter>()
            .on_find_habit_index(move |habit_id| {
                let ui = match ui_weak.upgrade() {
                    Some(ui) => ui,
                    None => return -1,
                };
                let habits = ui.global::<HabitAdapter>().get_habits();
                for i in 0..habits.row_count() {
                    if let Some(habit) = habits.row_data(i)
                        && habit.id == habit_id
                    {
                        return i as i32;
                    }
                }
                -1
            });
    }
}
