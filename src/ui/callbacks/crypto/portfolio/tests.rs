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

static HISTORICAL_TEST_GUARD: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn reset_historical_state_for_tests() {
    if let Ok(mut in_flight) = HISTORICAL_REQUESTS_IN_FLIGHT.lock() {
        in_flight.clear();
    }
    if let Ok(mut auto_keys) = HISTORICAL_AUTO_REQUESTED_KEYS.lock() {
        auto_keys.clear();
    }
    if let Ok(mut cache) = HISTORICAL_PRICE_CACHE.lock() {
        cache.clear();
    }
}

#[test]
fn historical_request_in_flight_deduplicates_until_finish() {
    let _guard = HISTORICAL_TEST_GUARD
        .lock()
        .expect("historical test mutex must lock");
    reset_historical_state_for_tests();

    let key = "bitcoin|2025-01-10";
    assert!(try_start_historical_request(key));
    assert!(!try_start_historical_request(key));

    finish_historical_request(key);
    assert!(try_start_historical_request(key));
}

#[test]
fn historical_auto_keys_track_requested_requests() {
    let _guard = HISTORICAL_TEST_GUARD
        .lock()
        .expect("historical test mutex must lock");
    reset_historical_state_for_tests();

    let key = "monero|2024-06-01";
    assert!(!has_auto_historical_request(key));

    mark_auto_historical_request(key);
    assert!(has_auto_historical_request(key));
}

#[test]
fn historical_price_cache_ignores_empty_values() {
    let _guard = HISTORICAL_TEST_GUARD
        .lock()
        .expect("historical test mutex must lock");
    reset_historical_state_for_tests();

    let key = "litecoin|2023-09-20";
    cache_historical_price(key, "");
    assert!(get_cached_historical_price(key).is_none());

    cache_historical_price(key, "64.1234");
    assert_eq!(get_cached_historical_price(key).as_deref(), Some("64.1234"));
}

#[test]
fn historical_price_error_suppression_matches_validation_errors() {
    assert!(should_suppress_historical_price_error(
        "Invalid date format"
    ));
    assert!(should_suppress_historical_price_error(
        "Coin ID cannot be empty"
    ));
    assert!(!should_suppress_historical_price_error("Network timeout"));
}

#[test]
fn historical_price_error_mapping_returns_user_friendly_messages() {
    let no_data = map_historical_price_error_for_ui("Historical USD price not available");
    assert_eq!(
        no_data,
        "No historical price available for that coin/date.".to_string()
    );

    let rate_limit = map_historical_price_error_for_ui("Rate limit exceeded");
    assert_eq!(
        rate_limit,
        "Historical API rate limit reached. Please wait and try again.".to_string()
    );

    let passthrough = map_historical_price_error_for_ui("API error: Kraken failed");
    assert_eq!(passthrough, "Kraken failed".to_string());
}
