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
/// Maps hex colors to indices matching ui/modals/add_habit.slint color-options array
pub fn habit_color_index(color_hex: &str) -> i32 {
    match color_hex.to_lowercase().as_str() {
        "#8b5cf6" => 0,  // violet-500
        "#ec4899" => 1,  // pink-500
        "#ef4444" => 2,  // red-500
        "#f97316" => 3,  // orange-500
        "#f59e0b" => 4,  // amber-500
        "#eab308" => 5,  // yellow-500
        "#84cc16" => 6,  // lime-500
        "#22c55e" => 7,  // green-500
        "#10b981" => 8,  // emerald-500
        "#14b8a6" => 9,  // teal-500
        "#06b6d4" => 10, // cyan-500
        "#0ea5e9" => 11, // sky-500
        "#3b82f6" => 12, // blue-500
        "#6366f1" => 13, // indigo-500
        "#a16207" => 14, // amber-700
        "#64748b" => 15, // slate-500
        _ => 0,
    }
}
