//! Currency conversion and formatting utilities
//!
//! Handles multi-currency support with proper formatting per locale.
//! CLP uses Chilean format (dot for thousands, no decimals).

/// Formats a whole number with thousand separators (no decimals).
fn format_thousands_with_sep(value: i64, sep: char) -> String {
    let abs = value.abs();
    let digits = abs.to_string();
    let mut result = String::new();
    for (count, c) in digits.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            result.insert(0, sep);
        }
        result.insert(0, c);
    }
    result
}

/// Formats amount in cents with US format (comma for thousands, 2 decimals).
fn format_amount_usd(amount_cents: i64) -> String {
    let abs = amount_cents.abs();
    let units = abs / 100;
    let cents = abs % 100;
    format!("{}.{:02}", format_thousands_with_sep(units, ','), cents)
}

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

/// Formats a float amount in CLP (no decimals, Chilean format with dot for thousands).
pub fn format_clp(amount: f64) -> String {
    let rounded = amount.round() as i64;
    format!("CLP {}", format_thousands_with_sep(rounded, '.'))
}

/// Formats a float amount as USD (converts dollars to cents internally).
pub fn format_usd(amount: f64) -> String {
    format_money((amount * 100.0) as i64, "USD")
}

/// Formats a float amount in the preferred currency.
pub fn format_preferred(amount: f64, currency: &str) -> String {
    if currency == "CLP" {
        format_clp(amount)
    } else {
        format_usd(amount)
    }
}

/// Formats amount in cents with currency code.
/// CLP uses Chilean format (no decimals, dot for thousands).
pub fn format_money(amount_cents: i64, currency: &str) -> String {
    let code = currency.to_uppercase();
    if code == "CLP" {
        let units = amount_cents.abs() / 100;
        format!("{code} {}", format_thousands_with_sep(units, '.'))
    } else {
        format!("{code} {}", format_amount_usd(amount_cents))
    }
}

/// Formats cents with a leading minus sign if negative.
pub fn format_money_signed(amount_cents: i64, currency: &str) -> String {
    if amount_cents < 0 {
        format!("- {}", format_money(amount_cents.abs(), currency))
    } else {
        format_money(amount_cents, currency)
    }
}
