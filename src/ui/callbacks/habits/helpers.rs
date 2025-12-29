//! Habit helper functions

/// Normalize habit category value to lowercase key
pub fn normalize_habit_category_value(category: &str) -> String {
    match category.to_lowercase().as_str() {
        "mind" | "mental" => "mind".to_string(),
        "body" | "physical" => "body".to_string(),
        "spirit" | "discipline" | "spiritual" => "spirit".to_string(),
        _ => "mind".to_string(),
    }
}

/// Get color index for habit color picker
pub fn habit_color_index(color_hex: &str) -> i32 {
    match color_hex.to_lowercase().as_str() {
        "#38bdf8" => 0, // sky-400
        "#22c55e" => 1, // green-500
        "#a855f7" => 2, // purple-500
        "#f97316" => 3, // orange-500
        "#ef4444" => 4, // red-500
        "#eab308" => 5, // yellow-500
        _ => 0,
    }
}
