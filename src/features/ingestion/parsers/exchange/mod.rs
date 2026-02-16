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

//! Exchange-specific CSV parsers
//!
//! Each submodule handles a specific exchange or wallet CSV format and converts
//! rows into [`ImportCryptoTransaction`] instances that feed into the existing
//! ingestion pipeline.
//!
//! ## Adding a new exchange
//!
//! 1. Create `<exchange>.rs` in this directory.
//! 2. Implement [`ExchangeParser`] for your struct.
//! 3. Register detection headers in [`detect_exchange_source`].
//! 4. Add the variant to [`ExchangeSource`].

pub mod binance;
pub mod common;
pub mod feather;
pub mod kraken;

use super::ParseResult;
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

// ─── Exchange source identification ──────────────────────────────────────────

/// Identifies which exchange or wallet a CSV file originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeSource {
    KrakenLedger,
    KrakenTrades,
    BinanceAllStatements,
    BinanceSpotTradeHistory,
    FeatherWallet,
    // Future:
    // CryptoMkt,
    // Mexc,
    // Bybit,
    // CakeWallet,
}

impl ExchangeSource {
    /// Human-readable label for display and i18n keys.
    pub fn label(&self) -> &'static str {
        match self {
            ExchangeSource::KrakenLedger => "Kraken Ledger",
            ExchangeSource::KrakenTrades => "Kraken Trades",
            ExchangeSource::BinanceAllStatements => "Binance All Statements",
            ExchangeSource::BinanceSpotTradeHistory => "Binance Spot Trade History",
            ExchangeSource::FeatherWallet => "Feather Wallet",
        }
    }

    /// Short identifier used in logs and summaries.
    pub fn id(&self) -> &'static str {
        match self {
            ExchangeSource::KrakenLedger => "kraken_ledger",
            ExchangeSource::KrakenTrades => "kraken_trades",
            ExchangeSource::BinanceAllStatements => "binance_all",
            ExchangeSource::BinanceSpotTradeHistory => "binance_spot",
            ExchangeSource::FeatherWallet => "feather",
        }
    }

    /// Default wallet name suggestion when the user doesn't specify one.
    pub fn default_wallet_name(&self) -> &'static str {
        match self {
            ExchangeSource::KrakenLedger | ExchangeSource::KrakenTrades => "Kraken",
            ExchangeSource::BinanceAllStatements | ExchangeSource::BinanceSpotTradeHistory => {
                "Binance"
            }
            ExchangeSource::FeatherWallet => "Feather",
        }
    }
}

// ─── Header-based format detection ──────────────────────────────────────────

/// Known header sets for each exchange format.
/// Order matters: more specific patterns are checked first.
const EXCHANGE_SIGNATURES: &[(&[&str], ExchangeSource)] = &[
    // Kraken Ledger v1 (10 columns)
    (
        &[
            "txid", "refid", "time", "type", "subtype", "aclass", "asset", "amount", "fee",
            "balance",
        ],
        ExchangeSource::KrakenLedger,
    ),
    // Kraken Ledger v2 (12 columns, with subclass + wallet)
    (
        &[
            "txid", "refid", "time", "type", "subtype", "aclass", "subclass", "asset", "wallet",
            "amount", "fee", "balance",
        ],
        ExchangeSource::KrakenLedger,
    ),
    // Kraken Trades v1 (13 columns)
    (
        &[
            "txid",
            "ordertxid",
            "pair",
            "time",
            "type",
            "ordertype",
            "price",
            "cost",
            "fee",
            "vol",
            "margin",
            "misc",
            "ledgers",
        ],
        ExchangeSource::KrakenTrades,
    ),
    // Kraken Trades v2 — match on first 5 distinctive columns only, since the
    // extended format inserts `aclass` and `subclass` between `pair` and `time`.
    (
        &[
            "txid",
            "ordertxid",
            "pair",
            "aclass",
            "subclass",
            "time",
            "type",
            "ordertype",
            "price",
            "cost",
            "fee",
            "vol",
            "margin",
            "misc",
            "ledgers",
        ],
        ExchangeSource::KrakenTrades,
    ),
    // Binance All Statements
    (
        &[
            "User_ID",
            "UTC_Time",
            "Account",
            "Operation",
            "Coin",
            "Change",
            "Remark",
        ],
        ExchangeSource::BinanceAllStatements,
    ),
    // Binance Spot Trade History
    (
        &[
            "Date(UTC)",
            "Pair",
            "Side",
            "Price",
            "Executed",
            "Amount",
            "Fee",
        ],
        ExchangeSource::BinanceSpotTradeHistory,
    ),
    // Feather Wallet (Monero) — real export format
    (
        &[
            "blockHeight",
            "timestamp",
            "date",
            "accountIndex",
            "direction",
            "balanceDelta",
            "amount",
            "fee",
            "txid",
            "description",
            "paymentId",
            "fiatAmount",
            "fiatCurrency",
        ],
        ExchangeSource::FeatherWallet,
    ),
    // Feather Wallet (Monero) — legacy format
    (
        &[
            "blockheight",
            "epoch",
            "date",
            "direction",
            "amount",
            "fee",
            "txid",
            "address",
            "description",
            "paymentid",
        ],
        ExchangeSource::FeatherWallet,
    ),
];

/// Attempts to identify the exchange source from CSV content by inspecting
/// the header row. Returns `None` if the format is not recognized.
///
/// The content must start with the CSV header line. Leading BOM characters
/// are stripped automatically.
pub fn detect_exchange_source(content: &str) -> Option<ExchangeSource> {
    let first_line = content
        .trim_start_matches('\u{feff}') // strip UTF-8 BOM
        .lines()
        .next()?
        .trim();

    if first_line.is_empty() {
        return None;
    }

    // Parse header into normalized tokens.
    // We trim whitespace and strip surrounding quotes from each column name.
    let headers: Vec<&str> = first_line
        .split(',')
        .map(|h| {
            let t = h.trim();
            t.trim_matches('"').trim()
        })
        .collect();

    for (expected, source) in EXCHANGE_SIGNATURES {
        if headers_match(&headers, expected) {
            return Some(*source);
        }
    }

    None
}

/// Returns `true` if `actual` headers contain all `expected` columns in order,
/// allowing extra trailing columns.
fn headers_match(actual: &[&str], expected: &[&str]) -> bool {
    if actual.len() < expected.len() {
        return false;
    }
    actual
        .iter()
        .zip(expected.iter())
        .all(|(a, e)| a.eq_ignore_ascii_case(e))
}

// ─── Parser trait ────────────────────────────────────────────────────────────

/// Trait implemented by each exchange-specific parser.
///
/// The `wallet_name` parameter lets the user override the target wallet;
/// parsers use it verbatim as the `wallet` field of `ImportCryptoTransaction`.
pub trait ExchangeParser {
    /// Parses the raw CSV content into intermediate crypto transactions.
    ///
    /// Returns a `ParseResult` containing successfully parsed items (with
    /// their source line numbers) and any per-row errors.
    fn parse(
        &self,
        content: &str,
        wallet_name: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError>;

    /// Returns the exchange source identifier for this parser.
    fn source(&self) -> ExchangeSource;
}

/// Convenience: create the right parser for a detected source.
pub fn parser_for(source: ExchangeSource) -> Box<dyn ExchangeParser> {
    match source {
        ExchangeSource::KrakenLedger => Box::new(kraken::KrakenLedgerParser),
        ExchangeSource::KrakenTrades => Box::new(kraken::KrakenTradesParser),
        ExchangeSource::BinanceAllStatements => Box::new(binance::BinanceAllStatementsParser),
        ExchangeSource::BinanceSpotTradeHistory => Box::new(binance::BinanceSpotParser),
        ExchangeSource::FeatherWallet => Box::new(feather::FeatherParser),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_kraken_ledger_v1() {
        let csv = "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::KrakenLedger)
        );
    }

    #[test]
    fn detect_kraken_ledger_v2() {
        let csv = "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"subclass\",\"asset\",\"wallet\",\"amount\",\"fee\",\"balance\"\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::KrakenLedger)
        );
    }

    #[test]
    fn detect_kraken_trades_v1() {
        let csv = "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::KrakenTrades)
        );
    }

    #[test]
    fn detect_binance_all_statements() {
        let csv = "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n12345,2024-01-01 00:00:00,Spot,Buy,BTC,0.5,\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::BinanceAllStatements)
        );
    }

    #[test]
    fn detect_binance_spot() {
        let csv = "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::BinanceSpotTradeHistory)
        );
    }

    #[test]
    fn detect_feather_wallet() {
        let csv = "blockHeight,timestamp,date,accountIndex,direction,balanceDelta,amount,fee,txid,description,paymentId,fiatAmount,fiatCurrency\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::FeatherWallet)
        );
    }

    #[test]
    fn detect_feather_wallet_legacy() {
        let csv =
            "blockheight,epoch,date,direction,amount,fee,txid,address,description,paymentid\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::FeatherWallet)
        );
    }

    #[test]
    fn detect_unknown_returns_none() {
        let csv = "date,account,amount,currency\n";
        assert_eq!(detect_exchange_source(csv), None);
    }

    #[test]
    fn detect_empty_returns_none() {
        assert_eq!(detect_exchange_source(""), None);
        assert_eq!(detect_exchange_source("   "), None);
    }

    #[test]
    fn detect_handles_bom() {
        let csv = "\u{feff}\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::KrakenLedger)
        );
    }

    #[test]
    fn headers_match_is_case_insensitive() {
        let actual = vec![
            "TxId", "RefId", "Time", "Type", "SubType", "AClass", "Asset", "Amount", "Fee",
            "Balance",
        ];
        let expected = &[
            "txid", "refid", "time", "type", "subtype", "aclass", "asset", "amount", "fee",
            "balance",
        ];
        assert!(headers_match(&actual, expected));
    }

    #[test]
    fn headers_match_allows_extra_trailing_columns() {
        let actual = vec![
            "txid", "refid", "time", "type", "subtype", "aclass", "asset", "amount", "fee",
            "balance", "extra1", "extra2",
        ];
        let expected = &[
            "txid", "refid", "time", "type", "subtype", "aclass", "asset", "amount", "fee",
            "balance",
        ];
        assert!(headers_match(&actual, expected));
    }

    #[test]
    fn headers_match_rejects_too_few_columns() {
        let actual = vec!["txid", "refid", "time"];
        let expected = &["txid", "refid", "time", "type", "subtype"];
        assert!(!headers_match(&actual, expected));
    }

    #[test]
    fn exchange_source_roundtrip() {
        let sources = [
            ExchangeSource::KrakenLedger,
            ExchangeSource::KrakenTrades,
            ExchangeSource::BinanceAllStatements,
            ExchangeSource::BinanceSpotTradeHistory,
            ExchangeSource::FeatherWallet,
        ];
        for source in sources {
            assert!(!source.label().is_empty());
            assert!(!source.id().is_empty());
            assert!(!source.default_wallet_name().is_empty());
        }
    }
}
