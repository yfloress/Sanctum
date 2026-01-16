//! UI helper functions
//!
//! Formatting, parsing, and display utilities for the UI layer.

use crate::models::CryptoTransaction;
use slint::Image;
use std::cell::RefCell;
use std::collections::HashMap;

pub const CRYPTO_ICON_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/ui/assets/crypto-icons");

pub const HABIT_COLOR_CHOICES: [&str; 16] = [
    "#8b5cf6", "#ec4899", "#ef4444", "#f97316", "#f59e0b", "#eab308", "#84cc16", "#22c55e",
    "#10b981", "#14b8a6", "#06b6d4", "#0ea5e9", "#3b82f6", "#6366f1", "#a16207", "#64748b",
];

thread_local! {
    pub static CRYPTO_ICON_CACHE: RefCell<HashMap<String, Image>> = RefCell::new(HashMap::new());
}

// ==================== Amount Formatting ====================

/// Formats amount in cents to display string with thousand separators
pub fn format_amount(amount_cents: i64) -> String {
    let abs = amount_cents.abs();
    let units = abs / 100;
    let cents = abs % 100;

    let units_str = units.to_string();
    let mut formatted_units = String::new();
    for (count, c) in units_str.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            formatted_units.insert(0, ',');
        }
        formatted_units.insert(0, c);
    }
    format!("{formatted_units}.{cents:02}")
}

/// Formats value in cents to decimal string (without thousand separators)
pub fn format_decimal_from_cents(value: i64) -> String {
    let units = value / 100;
    let cents = value.abs() % 100;
    format!("{units}.{cents:02}")
}

/// Formats amount with currency code
pub fn format_money(amount_cents: i64, currency: &str) -> String {
    let code = currency.to_uppercase();
    format!("{code} {}", format_amount(amount_cents))
}

/// Formats a float amount as USD (converts dollars to cents internally)
pub fn format_usd(amount: f64) -> String {
    format_money((amount * 100.0) as i64, "USD")
}

/// Formats CLP exchange rate with thousand separators
pub fn format_clp_rate(rate: f64) -> String {
    let rounded = rate.round() as i64;
    let mut digits = rounded.abs().to_string();
    let mut grouped = String::new();

    while digits.len() > 3 {
        let chunk = digits.split_off(digits.len() - 3);
        grouped = format!(",{chunk}{grouped}");
    }

    let formatted = format!("{digits}{grouped}");
    format!("$ {formatted}")
}

/// Formats crypto amount removing trailing zeros
pub fn format_crypto_amount(amount: f64) -> String {
    let mut formatted = format!("{:.8}", amount);
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    formatted
}

/// Parses amount input string to cents
pub fn parse_amount_input(value: &str) -> Option<i64> {
    let cleaned = value.trim().replace(',', "");
    if cleaned.is_empty() {
        return None;
    }
    let parsed: f64 = cleaned.parse().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    Some((parsed * 100.0).round() as i64)
}

// ==================== Currency Conversion ====================

/// Converts cents from one currency to another using the CLP/USD rate.
/// The rate represents how many CLP equals 1 USD.
pub fn convert_currency(amount_cents: i64, from: &str, to: &str, clp_rate: f64) -> i64 {
    if from == to || clp_rate <= 0.0 {
        return amount_cents;
    }
    match (from, to) {
        ("CLP", "USD") => (amount_cents as f64 / clp_rate).round() as i64,
        ("USD", "CLP") => (amount_cents as f64 * clp_rate).round() as i64,
        _ => amount_cents,
    }
}

/// Converts a float USD amount to the preferred currency.
pub fn convert_usd_to_preferred(amount_usd: f64, preferred: &str, clp_rate: f64) -> f64 {
    if preferred == "CLP" && clp_rate > 0.0 {
        amount_usd * clp_rate
    } else {
        amount_usd
    }
}

/// Formats a whole number with thousand separators (no decimals).
fn format_thousands(value: i64) -> String {
    let abs = value.abs();
    let digits = abs.to_string();
    let mut result = String::new();
    for (count, c) in digits.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Formats a float amount in CLP (no decimals, with thousand separators).
pub fn format_clp(amount: f64) -> String {
    let rounded = amount.round() as i64;
    format!("CLP {}", format_thousands(rounded))
}

/// Formats a float amount in the preferred currency.
pub fn format_preferred(amount: f64, currency: &str) -> String {
    if currency == "CLP" {
        format_clp(amount)
    } else {
        format_usd(amount)
    }
}

// ==================== Account Helpers ====================

/// Normalizes account type to valid values
pub fn normalize_account_type(value: &str) -> String {
    let normalized = value.trim().to_lowercase();
    match normalized.as_str() {
        "bank" => "bank".to_string(),
        "cash" => "cash".to_string(),
        "savings" => "savings".to_string(),
        "credit" | "credit card" | "credit_card" => "credit_card".to_string(),
        "other" => "other".to_string(),
        _ => normalized,
    }
}

// ==================== Habit Helpers ====================

/// Returns index of color in the HABIT_COLOR_CHOICES array
pub fn habit_color_index(color_hex: &str) -> i32 {
    let target = color_hex.trim();
    HABIT_COLOR_CHOICES
        .iter()
        .position(|hex| hex.eq_ignore_ascii_case(target))
        .map(|idx| idx as i32)
        .unwrap_or(0)
}

/// Normalizes habit category to valid values
pub fn normalize_habit_category_value(category: &str) -> String {
    match category.trim().to_lowercase().as_str() {
        "mind" => "mind".to_string(),
        "body" => "body".to_string(),
        "spirit" | "discipline" => "spirit".to_string(),
        _ => "mind".to_string(),
    }
}

/// Calculates current streak from a sorted list of dates.
/// Returns the number of consecutive days ending at today (or yesterday if today not complete).
///
/// # Timezone Behavior
/// The `today` parameter should use the user's local date (`chrono::Local::now().date_naive()`).
/// This means streak calculations are based on the user's local calendar day.
/// **Note**: Changing timezones mid-day may affect streak counts, as a day completed in one
/// timezone might appear as a different calendar date in another.
pub fn calculate_current_streak(dates: &[chrono::NaiveDate], today: chrono::NaiveDate) -> i32 {
    if dates.is_empty() {
        return 0;
    }

    let mut streak = 0;
    let mut check_date = today;

    // If today isn't completed, start checking from yesterday
    if !dates.contains(&today) {
        if let Some(prev) = today.pred_opt() {
            check_date = prev;
        } else {
            return 0;
        }
    }

    while dates.contains(&check_date) {
        streak += 1;
        if let Some(prev) = check_date.pred_opt() {
            check_date = prev;
        } else {
            break;
        }
    }

    streak
}

/// Calculates the best (longest) streak from a sorted list of dates.
///
/// # Note
/// Limited to logs from the last 2 years for performance reasons.
/// Best streak is calculated from available data within this window.
pub fn calculate_best_streak(dates: &[chrono::NaiveDate]) -> i32 {
    if dates.is_empty() {
        return 0;
    }

    let mut best_streak = 0;
    let mut current_streak = 0;
    let mut prev_date: Option<chrono::NaiveDate> = None;

    for date in dates {
        if let Some(prev) = prev_date {
            if let Some(next) = prev.succ_opt() {
                if *date == next {
                    current_streak += 1;
                } else {
                    current_streak = 1;
                }
            } else {
                current_streak = 1;
            }
        } else {
            current_streak = 1;
        }

        if current_streak > best_streak {
            best_streak = current_streak;
        }
        prev_date = Some(*date);
    }

    best_streak
}

// ==================== Category Helpers ====================

/// Formats category label with proper capitalization
pub fn format_category_label(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.chars().any(|c| c.is_lowercase()) {
        return trimmed.to_string();
    }
    let mut parts = Vec::new();
    for word in trimmed.split_whitespace() {
        let mut chars = word.chars();
        let first = chars.next().unwrap_or_default();
        let rest: String = chars.collect();
        let mut formatted = String::new();
        formatted.extend(first.to_uppercase());
        formatted.push_str(&rest.to_lowercase());
        parts.push(formatted);
    }
    parts.join(" ")
}

// ==================== Crypto Helpers ====================

/// Formats fee display with optional coin fee
pub fn format_fee_display(tx: &CryptoTransaction, symbol_map: &HashMap<String, String>) -> String {
    let mut parts = Vec::new();
    if let Some(fee) = tx.fee
        && fee > 0.0
    {
        parts.push(format_money((fee * 100.0) as i64, "USD"));
    }
    if let (Some(fee_coin_id), Some(fee_amount)) = (tx.fee_coin_id.as_ref(), tx.fee_amount)
        && fee_amount > 0.0
    {
        let fee_coin_str: &str = fee_coin_id.as_str();
        let symbol = symbol_map
            .get(fee_coin_str)
            .cloned()
            .unwrap_or_else(|| fee_coin_id.to_uppercase());
        parts.push(format!("{} {}", format_crypto_amount(fee_amount), symbol));
    }
    parts.join(" + ")
}

/// Formats price for display
pub fn format_price_display(price: Option<f64>) -> String {
    let price_val = price.unwrap_or(0.0);
    if price_val < 1.0 && price_val > 0.0 {
        format!("$ {:.4}", price_val)
    } else if price_val > 0.0 {
        format_money((price_val * 100.0) as i64, "USD")
    } else {
        String::new()
    }
}

/// Formats crypto transaction for display
/// Returns (label, amount_display, price_display, is_swap)
pub fn format_crypto_tx_display(
    tx: &CryptoTransaction,
    related: Option<&CryptoTransaction>,
) -> (String, String, String, bool) {
    let related_is_swap = related
        .map(|counter| counter.transaction_type == "swap")
        .unwrap_or(false);
    let is_swap = tx.transaction_type == "swap" || related_is_swap;

    let label = match tx.transaction_type.as_str() {
        "buy" => "BUY".to_string(),
        "sell" => "SELL".to_string(),
        "transfer_in" => {
            if related_is_swap {
                "SWAP IN".to_string()
            } else {
                "IN".to_string()
            }
        }
        "transfer_out" => "OUT".to_string(),
        "swap" => "SWAP OUT".to_string(),
        _ => tx.transaction_type.to_uppercase(),
    };

    let amount_display = if is_swap {
        if let Some(counter) = related {
            if tx.transaction_type == "swap" {
                format!(
                    "{} {} → {} {}",
                    format_crypto_amount(tx.amount),
                    tx.symbol,
                    format_crypto_amount(counter.amount),
                    counter.symbol
                )
            } else {
                format!(
                    "{} {} ← {} {}",
                    format_crypto_amount(tx.amount),
                    tx.symbol,
                    format_crypto_amount(counter.amount),
                    counter.symbol
                )
            }
        } else {
            format!("{} {}", format_crypto_amount(tx.amount), tx.symbol)
        }
    } else {
        format!("{} {}", format_crypto_amount(tx.amount), tx.symbol)
    };

    let price_display = match tx.transaction_type.as_str() {
        "buy" | "sell" => format_price_display(tx.price_per_coin),
        _ => String::new(),
    };

    (label, amount_display, price_display, is_swap)
}

// ==================== Color Helpers ====================

/// Converts hex color string to slint::Color
pub fn color_from_hex(hex: &str) -> slint::Color {
    if let Some(stripped) = hex.strip_prefix('#')
        && stripped.len() == 6
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&stripped[0..2], 16),
            u8::from_str_radix(&stripped[2..4], 16),
            u8::from_str_radix(&stripped[4..6], 16),
        )
    {
        return slint::Color::from_rgb_u8(r, g, b);
    }
    slint::Color::from_rgb_u8(139, 92, 246)
}

/// Loads crypto icon for symbol with caching
pub fn crypto_icon_for_symbol(symbol: &str) -> Image {
    let key = symbol.trim().to_lowercase();
    if let Some(icon) = CRYPTO_ICON_CACHE.with(|cache| cache.borrow().get(&key).cloned()) {
        return icon;
    }

    let base_dir = std::path::Path::new(CRYPTO_ICON_DIR);
    let icon_path = if key.is_empty() {
        base_dir.join("generic.svg")
    } else {
        let svg_path = base_dir.join(format!("{key}.svg"));
        if svg_path.exists() {
            svg_path
        } else {
            base_dir.join("generic.svg")
        }
    };

    let icon = if icon_path.exists() {
        Image::load_from_path(&icon_path).unwrap_or_default()
    } else {
        Image::default()
    };

    CRYPTO_ICON_CACHE.with(|cache| {
        cache.borrow_mut().insert(key, icon.clone());
    });

    icon
}

/// Loads wallet icon from path, returns empty image if path is invalid
pub fn load_wallet_icon(icon_path: Option<String>, category: &str) -> Image {
    // If we have a custom icon path, try to load it
    if let Some(path) = icon_path
        && !path.is_empty()
    {
        // Try to load from ui/assets (path is relative like "../assets/icons/wallet.svg")
        let full_path = std::path::Path::new("ui").join(path.trim_start_matches("../"));
        if full_path.exists()
            && let Ok(icon) = Image::load_from_path(&full_path)
        {
            return icon;
        }
    }

    // Fall back to category defaults
    let default_path = match category {
        "exchange" => "ui/assets/icons/building-2.svg",
        "wallet_multi" => "ui/assets/icons/shield.svg",
        _ => "ui/assets/icons/wallet.svg",
    };

    Image::load_from_path(std::path::Path::new(default_path)).unwrap_or_default()
}

/// Loads account icon from a stored path (bank icons); returns empty image if invalid.
pub fn load_account_icon(icon_path: Option<String>) -> Image {
    if let Some(path) = icon_path
        && !path.is_empty()
    {
        let full_path = std::path::Path::new("ui").join(path.trim_start_matches("../"));
        if full_path.exists()
            && let Ok(icon) = Image::load_from_path(&full_path)
        {
            return icon;
        }
    }

    Image::default()
}
