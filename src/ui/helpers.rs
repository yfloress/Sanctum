// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
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

//! UI helper functions
//!
//! Formatting, parsing, and display utilities for the UI layer.

use crate::models::CryptoTransaction;
use crate::ui::currency::format_money;
use std::collections::HashMap;

pub const GENERIC_BANK_ICON_PATH: &str = "../assets/icons/landmark.svg";

// ==================== Amount Formatting ====================

/// Formats amount in cents to display string with thousand separators
pub fn format_amount(amount_cents: i64) -> String {
    let prefix = if amount_cents < 0 { "-" } else { "" };
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
    format!("{prefix}{formatted_units}.{cents:02}")
}

/// Formats value in cents to decimal string (without thousand separators)
pub fn format_decimal_from_cents(value: i64) -> String {
    let units = value / 100;
    let cents = value.abs() % 100;
    format!("{units}.{cents:02}")
}

fn format_grouped_integer(value: i64) -> String {
    let mut digits = value.abs().to_string();
    let mut grouped = String::new();

    while digits.len() > 3 {
        let chunk = digits.split_off(digits.len() - 3);
        grouped = format!(",{chunk}{grouped}");
    }

    format!("{digits}{grouped}")
}

/// Formats a USD/target exchange rate for display in the ticker badge.
pub fn format_fx_rate(rate: f64, target_currency: &str) -> String {
    let target = target_currency.trim().to_uppercase();
    if target == "CLP" {
        let rounded = rate.round() as i64;
        return format!("$ {}", format_grouped_integer(rounded));
    }

    if !rate.is_finite() || rate <= 0.0 {
        return "N/A".to_string();
    }

    format!("{:.4}", rate)
}

/// Formats CLP exchange rate with thousand separators
pub fn format_clp_rate(rate: f64) -> String {
    format_fx_rate(rate, "CLP")
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

pub fn normalize_bank_icon_path(icon_path: Option<String>) -> Option<String> {
    let path = icon_path?.trim().to_string();
    if path.is_empty() || path == GENERIC_BANK_ICON_PATH {
        None
    } else {
        Some(path)
    }
}

// ==================== Category Helpers ====================

/// Display label for a category.
///
/// The seeded categories are stored as uppercase codes (`FOOD`, `SALARY`), so
/// they resolve to a translated name and follow the interface language. Anything
/// the user typed keeps at least one lowercase letter and is shown verbatim —
/// their wording, their capitalisation.
pub fn format_category_label(name: &str) -> String {
    let trimmed = name.trim();

    if trimmed.chars().any(|c| c.is_lowercase()) {
        return trimmed.to_string();
    }

    if !trimmed.is_empty() {
        let key = format!("category-{}", trimmed.to_lowercase().replace(' ', "-"));
        let translated = crate::services::i18n::t(&key);
        // The i18n layer echoes the key back when there is no entry for it.
        if translated != key {
            return translated;
        }
    }

    format_category_title_case(trimmed)
}

/// Title-cases an uppercase code as a last resort when it has no translation.
fn format_category_title_case(trimmed: &str) -> String {
    if trimmed.is_empty() {
        return String::new();
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
    let tx_mech = tx.mechanical_type();
    let related_is_swap = related
        .map(|counter| counter.mechanical_type() == "swap")
        .unwrap_or(false);
    let is_swap = tx_mech == "swap" || related_is_swap;

    let is_swap_source = related.map_or(tx_mech == "swap", |counter| {
        let tx_has_fee = tx.fee.is_some() || tx.fee_coin_id.is_some() || tx.fee_amount.is_some();
        let counter_has_fee =
            counter.fee.is_some() || counter.fee_coin_id.is_some() || counter.fee_amount.is_some();
        if tx_has_fee && !counter_has_fee {
            true
        } else if counter_has_fee && !tx_has_fee {
            false
        } else {
            tx.id < counter.id
        }
    });

    let label = if is_swap {
        if is_swap_source {
            "SWAP OUT".to_string()
        } else {
            "SWAP IN".to_string()
        }
    } else {
        match tx_mech {
            "buy" => "BUY".to_string(),
            "sell" => "SELL".to_string(),
            "transfer_in" => "IN".to_string(),
            "transfer_out" => "OUT".to_string(),
            _ => tx.transaction_type.to_uppercase(),
        }
    };

    let amount_display = if is_swap {
        if let Some(counter) = related {
            if is_swap_source {
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

    let price_display = if matches!(tx_mech, "buy" | "sell") {
        format_price_display(tx.price_per_coin)
    } else {
        String::new()
    };

    (label, amount_display, price_display, is_swap)
}

// ==================== Color Helpers ====================

/// Converts hex color string to (r, g, b) tuple
pub fn color_from_hex(hex: &str) -> (u8, u8, u8) {
    if let Some(stripped) = hex.strip_prefix('#')
        && stripped.len() == 6
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&stripped[0..2], 16),
            u8::from_str_radix(&stripped[2..4], 16),
            u8::from_str_radix(&stripped[4..6], 16),
        )
    {
        return (r, g, b);
    }
    (139, 92, 246)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Amount Formatting ====================

    #[test]
    fn test_format_amount_zero() {
        assert_eq!(format_amount(0), "0.00");
    }

    #[test]
    fn test_format_amount_positive() {
        assert_eq!(format_amount(100), "1.00");
    }

    #[test]
    fn test_format_amount_large() {
        assert_eq!(format_amount(1234567), "12,345.67");
    }

    #[test]
    fn test_format_amount_negative() {
        assert_eq!(format_amount(-5000), "-50.00");
    }

    #[test]
    fn test_format_amount_cents_only() {
        assert_eq!(format_amount(1), "0.01");
    }

    #[test]
    fn test_format_amount_very_large() {
        assert_eq!(format_amount(123456789), "1,234,567.89");
    }

    #[test]
    fn test_format_decimal_from_cents_zero() {
        assert_eq!(format_decimal_from_cents(0), "0.00");
    }

    #[test]
    fn test_format_decimal_from_cents_positive() {
        assert_eq!(format_decimal_from_cents(100), "1.00");
    }

    #[test]
    fn test_format_decimal_from_cents_negative() {
        assert_eq!(format_decimal_from_cents(-5000), "-50.00");
    }

    #[test]
    fn test_format_decimal_from_cents_no_separator() {
        assert_eq!(format_decimal_from_cents(1234567), "12345.67");
    }

    // ==================== FX Rate Formatting ====================

    #[test]
    fn test_format_fx_rate_clp() {
        assert_eq!(format_fx_rate(850.75, "CLP"), "$ 851");
    }

    #[test]
    fn test_format_fx_rate_clp_zero() {
        assert_eq!(format_fx_rate(0.0, "CLP"), "$ 0");
    }

    #[test]
    fn test_format_fx_rate_usd() {
        assert_eq!(format_fx_rate(1.23456, "USD"), "1.2346");
    }

    #[test]
    fn test_format_fx_rate_zero() {
        assert_eq!(format_fx_rate(0.0, "USD"), "N/A");
    }

    #[test]
    fn test_format_fx_rate_negative() {
        assert_eq!(format_fx_rate(-1.0, "USD"), "N/A");
    }

    #[test]
    fn test_format_fx_rate_nan() {
        assert_eq!(format_fx_rate(f64::NAN, "USD"), "N/A");
    }

    #[test]
    fn test_format_clp_rate_delegates() {
        let result = format_clp_rate(1500.0);
        assert_eq!(result, "$ 1,500");
    }

    // ==================== Crypto Amount ====================

    #[test]
    fn test_format_crypto_amount_whole() {
        assert_eq!(format_crypto_amount(1.5), "1.5");
    }

    #[test]
    fn test_format_crypto_amount_trailing_zeros() {
        assert_eq!(format_crypto_amount(1.50000000), "1.5");
    }

    #[test]
    fn test_format_crypto_amount_small() {
        assert_eq!(format_crypto_amount(0.00000100), "0.000001");
    }

    #[test]
    fn test_format_crypto_amount_integer() {
        assert_eq!(format_crypto_amount(100.0), "100");
    }

    #[test]
    fn test_format_crypto_amount_full_precision() {
        assert_eq!(format_crypto_amount(1.23456789), "1.23456789");
    }

    #[test]
    fn test_format_crypto_amount_zero() {
        assert_eq!(format_crypto_amount(0.0), "0");
    }

    // ==================== Parse Input ====================

    #[test]
    fn test_parse_amount_input_normal() {
        assert_eq!(parse_amount_input("100"), Some(10000));
    }

    #[test]
    fn test_parse_amount_input_with_commas() {
        assert_eq!(parse_amount_input("1,234.56"), Some(123456));
    }

    #[test]
    fn test_parse_amount_input_decimal() {
        assert_eq!(parse_amount_input("1.234"), Some(123));
    }

    #[test]
    fn test_parse_amount_input_negative() {
        assert_eq!(parse_amount_input("-50.00"), Some(-5000));
    }

    #[test]
    fn test_parse_amount_input_empty() {
        assert_eq!(parse_amount_input(""), None);
    }

    #[test]
    fn test_parse_amount_input_whitespace() {
        assert_eq!(parse_amount_input("  "), None);
    }

    #[test]
    fn test_parse_amount_input_invalid() {
        assert_eq!(parse_amount_input("abc"), None);
    }

    #[test]
    fn test_parse_amount_input_trimmed() {
        assert_eq!(parse_amount_input("  100  "), Some(10000));
    }

    // ==================== Account Helpers ====================

    #[test]
    fn test_normalize_account_type_bank() {
        assert_eq!(normalize_account_type("bank"), "bank");
    }

    #[test]
    fn test_normalize_account_type_credit_card() {
        assert_eq!(normalize_account_type("credit card"), "credit_card");
    }

    #[test]
    fn test_normalize_account_type_credit_card_underscore() {
        assert_eq!(normalize_account_type("credit_card"), "credit_card");
    }

    #[test]
    fn test_normalize_account_type_unknown_passthrough() {
        assert_eq!(normalize_account_type("investment"), "investment");
    }

    #[test]
    fn test_normalize_account_type_case_and_trim() {
        assert_eq!(normalize_account_type("  Bank  "), "bank");
    }

    #[test]
    fn test_normalize_bank_icon_path_none() {
        assert_eq!(normalize_bank_icon_path(None), None);
    }

    #[test]
    fn test_normalize_bank_icon_path_empty() {
        assert_eq!(normalize_bank_icon_path(Some("".to_string())), None);
    }

    #[test]
    fn test_normalize_bank_icon_path_generic() {
        assert_eq!(
            normalize_bank_icon_path(Some(GENERIC_BANK_ICON_PATH.to_string())),
            None
        );
    }

    #[test]
    fn test_normalize_bank_icon_path_custom() {
        assert_eq!(
            normalize_bank_icon_path(Some("custom/path.svg".to_string())),
            Some("custom/path.svg".to_string())
        );
    }

    // ==================== Category Label ====================

    #[test]
    fn test_format_category_label_all_caps() {
        assert_eq!(format_category_label("FOOD"), "Food");
    }

    #[test]
    fn test_format_category_label_multi_word() {
        assert_eq!(format_category_label("GROCERY STORE"), "Grocery Store");
    }

    #[test]
    fn test_format_category_label_already_mixed() {
        assert_eq!(format_category_label("already mixed"), "already mixed");
    }

    #[test]
    fn test_format_category_label_empty() {
        assert_eq!(format_category_label(""), "");
    }

    #[test]
    fn test_format_category_label_trimmed() {
        assert_eq!(format_category_label("  SALARY  "), "Salary");
    }

    // ==================== Fee Display ====================

    // Test fixture builder: cohesive args mirror the struct fields it constructs.
    #[allow(clippy::too_many_arguments)]
    fn make_tx(
        id: &str,
        tx_type: &str,
        subtype: Option<&str>,
        amount: f64,
        symbol: &str,
        fee: Option<f64>,
        fee_coin_id: Option<&str>,
        fee_amount: Option<f64>,
        price: Option<f64>,
    ) -> CryptoTransaction {
        CryptoTransaction {
            id: id.to_string(),
            wallet_id: "w1".to_string(),
            coin_id: symbol.to_lowercase(),
            symbol: symbol.to_string(),
            transaction_type: tx_type.to_string(),
            amount,
            price_per_coin: price,
            fee,
            fee_coin_id: fee_coin_id.map(String::from),
            fee_amount,
            subtype: subtype.map(String::from),
            override_proceeds: None,
            override_cost_basis: None,
            date: "2024-06-15".to_string(),
            notes: None,
            related_tx_id: None,
        }
    }

    #[test]
    fn test_format_fee_display_usd_only() {
        let tx = make_tx(
            "tx1",
            "trade",
            None,
            1.0,
            "BTC",
            Some(1.50),
            None,
            None,
            None,
        );
        let map = HashMap::new();
        assert_eq!(format_fee_display(&tx, &map), "USD 1.50");
    }

    #[test]
    fn test_format_fee_display_coin_only() {
        let tx = make_tx(
            "tx1",
            "trade",
            None,
            1.0,
            "BTC",
            None,
            Some("bitcoin"),
            Some(0.01),
            None,
        );
        let map = HashMap::from([("bitcoin".to_string(), "BTC".to_string())]);
        assert_eq!(format_fee_display(&tx, &map), "0.01 BTC");
    }

    #[test]
    fn test_format_fee_display_both() {
        let tx = make_tx(
            "tx1",
            "trade",
            None,
            1.0,
            "BTC",
            Some(1.50),
            Some("bitcoin"),
            Some(0.01),
            None,
        );
        let map = HashMap::from([("bitcoin".to_string(), "BTC".to_string())]);
        assert_eq!(format_fee_display(&tx, &map), "USD 1.50 + 0.01 BTC");
    }

    #[test]
    fn test_format_fee_display_none() {
        let tx = make_tx("tx1", "trade", None, 1.0, "BTC", None, None, None, None);
        let map = HashMap::new();
        assert_eq!(format_fee_display(&tx, &map), "");
    }

    #[test]
    fn test_format_fee_display_coin_no_symbol_map() {
        let tx = make_tx(
            "tx1",
            "trade",
            None,
            1.0,
            "BTC",
            None,
            Some("bitcoin"),
            Some(0.01),
            None,
        );
        let map = HashMap::new();
        assert_eq!(format_fee_display(&tx, &map), "0.01 BITCOIN");
    }

    // ==================== Price Display ====================

    #[test]
    fn test_format_price_display_sub_dollar() {
        assert_eq!(format_price_display(Some(0.5)), "$ 0.5000");
    }

    #[test]
    fn test_format_price_display_over_dollar() {
        assert_eq!(format_price_display(Some(100.0)), "USD 100.00");
    }

    #[test]
    fn test_format_price_display_none() {
        assert_eq!(format_price_display(None), "");
    }

    #[test]
    fn test_format_price_display_zero() {
        assert_eq!(format_price_display(Some(0.0)), "");
    }

    #[test]
    fn test_format_price_display_exactly_one() {
        let result = format_price_display(Some(1.0));
        assert_eq!(result, "USD 1.00");
    }

    // ==================== Crypto TX Display ====================

    #[test]
    fn test_format_crypto_tx_display_buy() {
        let tx = make_tx(
            "tx1",
            "trade",
            None,
            1.5,
            "BTC",
            Some(0.10),
            None,
            None,
            Some(50000.0),
        );
        let (label, amount, price, is_swap) = format_crypto_tx_display(&tx, None);
        assert_eq!(label, "BUY");
        assert_eq!(amount, "1.5 BTC");
        assert!(price.contains("USD"));
        assert!(!is_swap);
    }

    #[test]
    fn test_format_crypto_tx_display_sell() {
        let tx = make_tx(
            "tx1",
            "trade",
            Some("sell"),
            1.5,
            "BTC",
            Some(0.10),
            None,
            None,
            Some(50000.0),
        );
        let (label, amount, price, is_swap) = format_crypto_tx_display(&tx, None);
        assert_eq!(label, "SELL");
        assert_eq!(amount, "1.5 BTC");
        assert!(price.contains("USD"));
        assert!(!is_swap);
    }

    #[test]
    fn test_format_crypto_tx_display_income() {
        let tx = make_tx(
            "tx1",
            "income",
            None,
            0.5,
            "ETH",
            None,
            None,
            None,
            Some(3000.0),
        );
        let (label, amount, _price, _is_swap) = format_crypto_tx_display(&tx, None);
        assert_eq!(label, "BUY");
        assert_eq!(amount, "0.5 ETH");
    }

    #[test]
    fn test_format_crypto_tx_display_transfer_in() {
        let tx = make_tx("tx1", "transfer", None, 2.0, "BTC", None, None, None, None);
        let (label, amount, price, is_swap) = format_crypto_tx_display(&tx, None);
        assert_eq!(label, "IN");
        assert_eq!(amount, "2 BTC");
        assert_eq!(price, "");
        assert!(!is_swap);
    }

    #[test]
    fn test_format_crypto_tx_display_transfer_out() {
        let tx = make_tx(
            "tx1",
            "transfer",
            Some("withdrawal"),
            2.0,
            "BTC",
            None,
            None,
            None,
            None,
        );
        let (label, amount, _price, is_swap) = format_crypto_tx_display(&tx, None);
        assert_eq!(label, "OUT");
        assert_eq!(amount, "2 BTC");
        assert!(!is_swap);
    }

    #[test]
    fn test_format_crypto_tx_display_swap_source() {
        let source = make_tx(
            "tx1",
            "trade",
            Some("swap"),
            1.0,
            "BTC",
            Some(0.05),
            None,
            None,
            None,
        );
        let dest = make_tx(
            "tx2",
            "trade",
            Some("swap"),
            100.0,
            "ETH",
            None,
            None,
            None,
            None,
        );
        let (label, amount, _price, is_swap) = format_crypto_tx_display(&source, Some(&dest));
        assert_eq!(label, "SWAP OUT");
        assert_eq!(amount, "1 BTC \u{2192} 100 ETH");
        assert!(is_swap);
    }

    #[test]
    fn test_format_crypto_tx_display_swap_dest() {
        let source = make_tx(
            "tx1",
            "trade",
            Some("swap"),
            1.0,
            "BTC",
            Some(0.05),
            None,
            None,
            None,
        );
        let dest = make_tx(
            "tx2",
            "trade",
            Some("swap"),
            100.0,
            "ETH",
            None,
            None,
            None,
            None,
        );
        let (label, amount, _price, _is_swap) = format_crypto_tx_display(&dest, Some(&source));
        assert_eq!(label, "SWAP IN");
        assert_eq!(amount, "100 ETH \u{2190} 1 BTC");
    }

    #[test]
    fn test_format_crypto_tx_display_swap_id_fallback() {
        let tx_a = make_tx(
            "tx1",
            "trade",
            Some("swap"),
            1.0,
            "BTC",
            None,
            None,
            None,
            None,
        );
        let tx_b = make_tx(
            "tx2",
            "trade",
            Some("swap"),
            100.0,
            "ETH",
            None,
            None,
            None,
            None,
        );
        let (label_a, _, _, _) = format_crypto_tx_display(&tx_a, Some(&tx_b));
        let (label_b, _, _, _) = format_crypto_tx_display(&tx_b, Some(&tx_a));
        // No fee on either — tiebreak by id (tx1 < tx2)
        assert_eq!(label_a, "SWAP OUT");
        assert_eq!(label_b, "SWAP IN");
    }

    // ==================== Color ====================

    #[test]
    fn test_color_from_hex_valid() {
        assert_eq!(color_from_hex("#8b5cf6"), (139, 92, 246));
    }

    #[test]
    fn test_color_from_hex_red() {
        assert_eq!(color_from_hex("#ff0000"), (255, 0, 0));
    }

    #[test]
    fn test_color_from_hex_invalid() {
        let (r, g, b) = color_from_hex("invalid");
        assert_eq!((r, g, b), (139, 92, 246));
    }

    #[test]
    fn test_color_from_hex_short() {
        let (r, g, b) = color_from_hex("#fff");
        assert_eq!((r, g, b), (139, 92, 246));
    }

    #[test]
    fn test_color_from_hex_empty() {
        let (r, g, b) = color_from_hex("");
        assert_eq!((r, g, b), (139, 92, 246));
    }

    #[test]
    fn test_color_from_hex_no_hash() {
        let (r, g, b) = color_from_hex("8b5cf6");
        assert_eq!((r, g, b), (139, 92, 246));
    }
}
