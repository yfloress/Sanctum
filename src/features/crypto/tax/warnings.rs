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

//! Tax warning code helpers shared across tax service and UI layer.

/// Missing-price warning codes that can be resolved with USD price sync.
pub const RESOLVABLE_MISSING_PRICE_WARNING_CODES: &[&str] = &[
    "missing_price",
    "fee_missing_price",
    "swap_missing_price",
    "income_missing_price",
];

/// Missing-price warning codes that represent non-USD quote normalization
/// issues and require FX/normalization inputs instead of USD-only sync.
pub const NON_USD_QUOTE_MISSING_PRICE_WARNING_CODES: &[&str] = &[
    "missing_price_non_usd_quote",
    "swap_missing_price_non_usd_quote",
    "income_missing_price_non_usd_quote",
];

/// IPC data gap warning code for Chile tax valuation adjustments.
pub const IPC_MISSING_WARNING_CODE: &str = "ipc_missing";

pub fn is_resolvable_missing_price_warning(code: &str) -> bool {
    RESOLVABLE_MISSING_PRICE_WARNING_CODES.contains(&code)
}

pub fn is_non_usd_quote_missing_price_warning(code: &str) -> bool {
    NON_USD_QUOTE_MISSING_PRICE_WARNING_CODES.contains(&code)
}

pub fn is_ipc_missing_warning(code: &str) -> bool {
    code == IPC_MISSING_WARNING_CODE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_classification_handles_all_supported_codes() {
        assert!(is_resolvable_missing_price_warning("missing_price"));
        assert!(is_resolvable_missing_price_warning("fee_missing_price"));
        assert!(is_non_usd_quote_missing_price_warning(
            "missing_price_non_usd_quote"
        ));
        assert!(is_non_usd_quote_missing_price_warning(
            "swap_missing_price_non_usd_quote"
        ));
        assert!(is_ipc_missing_warning("ipc_missing"));

        assert!(!is_resolvable_missing_price_warning(
            "missing_price_non_usd_quote"
        ));
        assert!(!is_non_usd_quote_missing_price_warning("missing_price"));
        assert!(!is_ipc_missing_warning("fee_missing_price"));
    }
}
