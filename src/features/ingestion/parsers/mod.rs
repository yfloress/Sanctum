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

//! Parsers for different import formats
//!
//! Supports JSON, CSV, plain text, and exchange-specific CSV formats.

pub mod csv;
pub mod exchange;
pub mod json;
pub mod text;

pub use self::csv::CsvParser;
pub use self::exchange::{ExchangeParser, ExchangeSource, detect_exchange_source, parser_for};
pub use self::json::{JsonParseResult, JsonParser};
pub use self::text::{TextMixedParseResult, TextParser};

use super::types::{ImportFormat, ImportHabitLog, ImportTransaction, RowError};

#[derive(Debug)]
pub struct ParseResult<T> {
    pub items: Vec<(usize, T)>,
    pub errors: Vec<RowError>,
}

impl<T> Default for ParseResult<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Common parser trait for all formats
pub trait ImportParser {
    /// Parses transactions from raw content
    /// Returns parsed items plus row-level parse errors
    fn parse_transactions(&self, content: &str)
    -> Result<ParseResult<ImportTransaction>, RowError>;

    /// Parses habit logs from raw content
    /// Returns parsed items plus row-level parse errors
    fn parse_habit_logs(&self, content: &str) -> Result<ParseResult<ImportHabitLog>, RowError>;

    /// Returns the format name for reporting
    fn format_name(&self) -> &'static str;
}

/// Detects file format from content and filename.
///
/// For `.csv` files, exchange-specific formats are checked first via header
/// inspection. If a known exchange is detected the corresponding
/// `ImportFormat::ExchangeCsv(source)` variant is returned. Otherwise the
/// generic Sanctum CSV sub-formats (transactions, habits, crypto) are tried.
pub fn detect_format(content: &str, filename: &str) -> Option<ImportFormat> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Get file extension
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    // JSON detection
    if (ext == "json" || trimmed.starts_with('{'))
        && trimmed.starts_with('{')
        && trimmed.ends_with('}')
    {
        return Some(ImportFormat::Json);
    }

    // CSV detection (has comma-separated header on first line)
    if ext == "csv" {
        let first_line = trimmed.lines().next()?.to_lowercase();
        // Must have commas and not start with # (comment)
        if first_line.contains(',') && !first_line.starts_with('#') {
            // Try exchange-specific detection first (Kraken, Binance, Feather, etc.)
            if let Some(source) = detect_exchange_source(content) {
                return Some(ImportFormat::ExchangeCsv(source));
            }

            // Crypto CSV: wallet, symbol columns
            if first_line.contains("wallet") && first_line.contains("symbol") {
                return Some(ImportFormat::CsvCrypto);
            }
            // Habit CSV
            if first_line.contains("habit")
                && first_line.contains("date")
                && first_line.contains("completed")
            {
                return Some(ImportFormat::CsvHabitLogs);
            }
            // Transaction CSV
            if first_line.contains("account") && first_line.contains("amount") {
                return Some(ImportFormat::CsvTransactions);
            }
        }
    }

    // Text detection (semicolon-separated with prefixes T;, H;, C;)
    if ext == "txt" || trimmed.contains(';') {
        // Check for prefix-based mixed format (T;, H;, C;)
        let has_prefixes = trimmed.lines().any(|line| {
            let t = line.trim().to_uppercase();
            t.starts_with("T;") || t.starts_with("H;") || t.starts_with("C;")
        });

        if has_prefixes {
            return Some(ImportFormat::TextMixed);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_json_format() {
        let json = r#"{"version": "1.0", "transactions": []}"#;
        assert_eq!(detect_format(json, "data.json"), Some(ImportFormat::Json));
    }

    #[test]
    fn test_detect_csv_transactions() {
        let csv = "date,account,type,amount,currency,category,description,transfer_to_account\n";
        assert_eq!(
            detect_format(csv, "data.csv"),
            Some(ImportFormat::CsvTransactions)
        );
    }

    #[test]
    fn test_detect_csv_habits() {
        let csv = "habit,date,completed\n";
        assert_eq!(
            detect_format(csv, "habits.csv"),
            Some(ImportFormat::CsvHabitLogs)
        );
    }

    #[test]
    fn test_detect_csv_crypto() {
        let csv = "date,wallet,symbol,type,amount,price_per_coin,fee,notes\n";
        assert_eq!(
            detect_format(csv, "crypto.csv"),
            Some(ImportFormat::CsvCrypto)
        );
    }

    #[test]
    fn test_detect_text_mixed() {
        let text = "T;2024-01-15;Account;expense;100;USD;Food;Groceries;\nH;Meditate;2024-01-15;true\nC;2024-01-15;Binance;BTC;trade;0.5;buy;45000;10;";
        assert_eq!(
            detect_format(text, "data.txt"),
            Some(ImportFormat::TextMixed)
        );
    }

    #[test]
    fn test_detect_empty() {
        assert_eq!(detect_format("", "file.json"), None);
        assert_eq!(detect_format("   ", "file.csv"), None);
    }

    // ── Exchange CSV detection via detect_format ──

    #[test]
    fn test_detect_kraken_ledger_csv() {
        let csv = "\"txid\",\"refid\",\"time\",\"type\",\"subtype\",\"aclass\",\"asset\",\"amount\",\"fee\",\"balance\"\n\"ABC\",\"DEF\",\"2024-01-01\",\"deposit\",\"\",\"currency\",\"XXBT\",\"1.0\",\"0\",\"1.0\"\n";
        assert_eq!(
            detect_format(csv, "ledgers.csv"),
            Some(ImportFormat::ExchangeCsv(ExchangeSource::KrakenLedger))
        );
    }

    #[test]
    fn test_detect_kraken_trades_csv() {
        let csv = "\"txid\",\"ordertxid\",\"pair\",\"time\",\"type\",\"ordertype\",\"price\",\"cost\",\"fee\",\"vol\",\"margin\",\"misc\",\"ledgers\"\n";
        assert_eq!(
            detect_format(csv, "trades.csv"),
            Some(ImportFormat::ExchangeCsv(ExchangeSource::KrakenTrades))
        );
    }

    #[test]
    fn test_detect_binance_all_statements_csv() {
        let csv = "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n12345,2024-01-01 00:00:00,Spot,Buy,BTC,0.5,\n";
        assert_eq!(
            detect_format(csv, "statement.csv"),
            Some(ImportFormat::ExchangeCsv(
                ExchangeSource::BinanceAllStatements
            ))
        );
    }

    #[test]
    fn test_detect_binance_spot_csv() {
        let csv = "Date(UTC),Pair,Side,Price,Executed,Amount,Fee\n2024-01-01,BTCUSDT,BUY,42000,0.5BTC,21000USDT,0.001BTC\n";
        assert_eq!(
            detect_format(csv, "spot.csv"),
            Some(ImportFormat::ExchangeCsv(
                ExchangeSource::BinanceSpotTradeHistory
            ))
        );
    }

    #[test]
    fn test_detect_feather_wallet_csv() {
        let csv = "blockheight,epoch,date,direction,amount,fee,txid,address,description,paymentid\n100,1700000000,2024-01-01,in,1.5,0.0001,abc123,addr1,,\n";
        assert_eq!(
            detect_format(csv, "feather.csv"),
            Some(ImportFormat::ExchangeCsv(ExchangeSource::FeatherWallet))
        );
    }

    #[test]
    fn test_detect_monero_gui_wallet_csv() {
        let csv = "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n1111111,1610274075,2021-01-10 10:20:15,in,0.03,30000000000,0,abc,\"\",0,,\"\"\n";
        assert_eq!(
            detect_format(csv, "monero_gui.csv"),
            Some(ImportFormat::ExchangeCsv(ExchangeSource::MoneroGuiWallet))
        );
    }

    #[test]
    fn test_detect_mexc_withdrawals_csv() {
        let csv = "UID,Status,Time,Crypto,Network,Request Amount,Withdrawal Address,memo,TxID,Trading Fee,Settlement Amount,Withdrawal Descriptions\n11111111,Withdrawal Successful,2025-03-14 09:27:31,LTC,Litecoin(LTC),0.322,addr_ltc_test,--,a1b2,0.0001,0.3129,-\n";
        assert_eq!(
            detect_format(csv, "mexc-withdrawals.csv"),
            Some(ImportFormat::ExchangeCsv(
                ExchangeSource::MexcWithdrawalHistory
            ))
        );
    }

    #[test]
    fn test_detect_mexc_deposits_csv() {
        let csv = "UID,Status,Time,Crypto,Network,Deposit Amount,TxID,Progress\n10000000,Credited Successfully,2025-12-19 21:39:59,USDT,Polygon(MATIC),27,0xa1b2:0,(465/450)\n";
        assert_eq!(
            detect_format(csv, "mexc-deposits.csv"),
            Some(ImportFormat::ExchangeCsv(ExchangeSource::MexcDepositHistory))
        );
    }

    #[test]
    fn test_detect_mexc_trades_csv() {
        let csv = "UID,Pairs,Time,Side,Filled Price,Executed Amount,Total,Fee,Role\n11111111,LTC_USDT,2025-12-18 22:13:41,Buy,78.04,0.315,24.58860,0.01229430USDT,Taker\n";
        assert_eq!(
            detect_format(csv, "mexc-trades.csv"),
            Some(ImportFormat::ExchangeCsv(ExchangeSource::MexcTradeHistory))
        );
    }

    #[test]
    fn test_detect_mexc_statement_csv() {
        let csv =
            "UID,Creation Time(UTC+00:00),Crypto,Transaction Type,Direction,Quantity\nUSER_001,2025-10-01 00:00:01,USDT,Deposit,Inflow,1000.00\n";
        assert_eq!(
            detect_format(csv, "mexc-statement.csv"),
            Some(ImportFormat::ExchangeCsv(
                ExchangeSource::MexcStatementHistory
            ))
        );
    }

    #[test]
    fn test_detect_mexc_fiat_otc_csv() {
        let csv = "UID,Order ID,Start Time(UTC+00:00),End Time(UTC+00:00),Trading Token,Trading Direction,Status,Order Quantity,Settlement Token,Order Amount,Payment Method\nUSER_001,OTC-001,2025-10-05 10:00:00,2025-10-05 10:15:00,USD,Buy,Completed,1000,USDT,1000.00,Bank Transfer\n";
        assert_eq!(
            detect_format(csv, "mexc-fiat-otc.csv"),
            Some(ImportFormat::ExchangeCsv(ExchangeSource::MexcFiatOtcOrders))
        );
    }

    #[test]
    fn test_detect_mexc_funding_transfer_csv() {
        let csv = "UID,From System,To System,Currency,Amount,Status,update_time(UTC+00:00),create_time(UTC+00:00),Transfer Type\nUSER_001,Internal,External,USDT,150,Completed,2025-12-15 08:00:00,2025-12-15 07:55:00,Withdrawal\n";
        assert_eq!(
            detect_format(csv, "mexc-funding-transfer.csv"),
            Some(ImportFormat::ExchangeCsv(
                ExchangeSource::MexcFundingTransferHistory
            ))
        );
    }

    #[test]
    fn test_detect_mexc_futures_order_csv() {
        let csv = "UID,Time(UTC+00:00),Futures Trading Pair,Direction,Leverage,Order Type,Order Qty (Cont.),Filled Qty (Cont.),Order Qty (Crypto),Filled Qty (Crypto),Order Qty (Amount),Filled Qty (Amount),Order Price,Average Filled Price,Closing PNL,Trading Fee,Fee-payment Crypto,Status\nUSER_001,2025-11-20 11:00:00,BTC-USDT,Long,5,Limit,1,1,0.03,0.03,1500,1500,30000,30000,200,5,USDT,Filled\n";
        assert_eq!(
            detect_format(csv, "mexc-futures-orders.csv"),
            Some(ImportFormat::ExchangeCsv(
                ExchangeSource::MexcFuturesOrderHistory
            ))
        );
    }

    #[test]
    fn test_exchange_csv_takes_priority_over_generic() {
        // A Binance CSV has "Account" and could match generic CsvTransactions
        // if it also had "amount", but exchange detection runs first.
        let csv = "User_ID,UTC_Time,Account,Operation,Coin,Change,Remark\n";
        let fmt = detect_format(csv, "export.csv");
        assert!(matches!(fmt, Some(ImportFormat::ExchangeCsv(_))));
    }

    #[test]
    fn test_unknown_csv_falls_through_to_generic() {
        // A CSV with wallet+symbol columns but NOT matching any exchange
        // should fall through to CsvCrypto.
        let csv = "date,wallet,symbol,type,amount,fee\n";
        assert_eq!(
            detect_format(csv, "my_data.csv"),
            Some(ImportFormat::CsvCrypto)
        );
    }
}
