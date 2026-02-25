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
pub mod mexc;
pub mod monero_gui;
pub mod notbank;

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
    MoneroGuiWallet,
    MexcSpotTradeHistory,
    MexcTradeHistory,
    MexcDepositHistory,
    MexcWithdrawalHistory,
    MexcStatementHistory,
    MexcFiatOtcOrders,
    MexcFiatP2pOrders,
    MexcFundingOtherHistory,
    MexcFundingTransferHistory,
    MexcFuturesCopyTradeOrderHistory,
    MexcFuturesCapitalFlow,
    MexcFuturesOrderHistory,
    MexcFuturesPositionHistory,
    MexcFuturesTradeHistory,
    NotBankTransactions,
    NotBankTradeActivity,
    NotBankPnlReport,
    // Future:
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
            ExchangeSource::MoneroGuiWallet => "Monero GUI Wallet",
            ExchangeSource::MexcSpotTradeHistory => "MEXC Spot Trade History",
            ExchangeSource::MexcTradeHistory => "MEXC Trade History",
            ExchangeSource::MexcDepositHistory => "MEXC Deposit History",
            ExchangeSource::MexcWithdrawalHistory => "MEXC Withdrawal History",
            ExchangeSource::MexcStatementHistory => "MEXC Statement History",
            ExchangeSource::MexcFiatOtcOrders => "MEXC Fiat OTC Orders",
            ExchangeSource::MexcFiatP2pOrders => "MEXC Fiat P2P Orders",
            ExchangeSource::MexcFundingOtherHistory => "MEXC Funding Other History",
            ExchangeSource::MexcFundingTransferHistory => "MEXC Funding Transfer History",
            ExchangeSource::MexcFuturesCopyTradeOrderHistory => {
                "MEXC Futures Copy Trade Order History"
            }
            ExchangeSource::MexcFuturesCapitalFlow => "MEXC Futures Capital Flow",
            ExchangeSource::MexcFuturesOrderHistory => "MEXC Futures Order History",
            ExchangeSource::MexcFuturesPositionHistory => "MEXC Futures Position History",
            ExchangeSource::MexcFuturesTradeHistory => "MEXC Futures Trade History",
            ExchangeSource::NotBankTransactions => "NotBank Transaction Report",
            ExchangeSource::NotBankTradeActivity => "NotBank Trade Activity Report",
            ExchangeSource::NotBankPnlReport => "NotBank Profit And Loss Report",
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
            ExchangeSource::MoneroGuiWallet => "monero_gui",
            ExchangeSource::MexcSpotTradeHistory => "mexc_spot",
            ExchangeSource::MexcTradeHistory => "mexc_trades",
            ExchangeSource::MexcDepositHistory => "mexc_deposit",
            ExchangeSource::MexcWithdrawalHistory => "mexc_withdrawal",
            ExchangeSource::MexcStatementHistory => "mexc_statement",
            ExchangeSource::MexcFiatOtcOrders => "mexc_fiat_otc",
            ExchangeSource::MexcFiatP2pOrders => "mexc_fiat_p2p",
            ExchangeSource::MexcFundingOtherHistory => "mexc_funding_other",
            ExchangeSource::MexcFundingTransferHistory => "mexc_funding_transfer",
            ExchangeSource::MexcFuturesCopyTradeOrderHistory => "mexc_futures_copy",
            ExchangeSource::MexcFuturesCapitalFlow => "mexc_futures_capital",
            ExchangeSource::MexcFuturesOrderHistory => "mexc_futures_orders",
            ExchangeSource::MexcFuturesPositionHistory => "mexc_futures_positions",
            ExchangeSource::MexcFuturesTradeHistory => "mexc_futures_trades",
            ExchangeSource::NotBankTransactions => "notbank_transaction",
            ExchangeSource::NotBankTradeActivity => "notbank_trade",
            ExchangeSource::NotBankPnlReport => "notbank_pnl",
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
            ExchangeSource::MoneroGuiWallet => "Monero GUI",
            ExchangeSource::MexcSpotTradeHistory
            | ExchangeSource::MexcTradeHistory
            | ExchangeSource::MexcDepositHistory
            | ExchangeSource::MexcWithdrawalHistory
            | ExchangeSource::MexcStatementHistory
            | ExchangeSource::MexcFiatOtcOrders
            | ExchangeSource::MexcFiatP2pOrders
            | ExchangeSource::MexcFundingOtherHistory
            | ExchangeSource::MexcFundingTransferHistory
            | ExchangeSource::MexcFuturesCopyTradeOrderHistory
            | ExchangeSource::MexcFuturesCapitalFlow
            | ExchangeSource::MexcFuturesOrderHistory
            | ExchangeSource::MexcFuturesPositionHistory
            | ExchangeSource::MexcFuturesTradeHistory => "MEXC",
            ExchangeSource::NotBankTransactions
            | ExchangeSource::NotBankTradeActivity
            | ExchangeSource::NotBankPnlReport => "NotBank",
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
    // NotBank (ex-CryptoMarket) Transaction report
    (
        &[
            "RegisteredEntityId",
            "PostingEntryId",
            "PostingEntryType",
            "PostingDatetime",
            "AccountId",
            "AccountName",
            "Product",
            "CR",
            "DR",
            "ReferenceTransactionType",
            "ReferenceTransactionId",
            "SystemRecordReference",
            "OMSId",
            "Balance",
        ],
        ExchangeSource::NotBankTransactions,
    ),
    // NotBank Trade Activity report
    (
        &[
            "RegisteredEntityId",
            "TransReportId",
            "TransReportRevision",
            "TransReportType",
            "OrderId",
            "ClientOrderId",
            "QuoteId",
            "ExtTradeReportId",
            "TradeId",
            "TransReportDatetime",
            "Side",
            "Quantity",
            "Instrument",
            "Price",
            "InsideBid",
            "InsideBidSize",
            "InsideOffer",
            "InsideOfferSize",
            "LeavesSize",
            "MakerTaker",
            "Trader",
            "AccountId",
            "AccountName",
            "Fee",
            "FeeProduct",
            "Notional",
            "BaseSettlementAmount",
            "CounterpartySettlementAmount",
            "OMSId",
        ],
        ExchangeSource::NotBankTradeActivity,
    ),
    // NotBank Profit And Loss report (Unrealized section header)
    (
        &[
            "AccountId",
            "AccountName",
            "Product",
            "FullName",
            "TimeStamp",
            "ProductQuantity",
            "PurchasePrice",
            "TotalFeeAsProduct",
            "TotalQuantityMinusFee",
            "TotalPurchaseValue",
            "ProductEndPrice",
            "TotalSaleValue",
            "P/L",
            "%Return",
        ],
        ExchangeSource::NotBankPnlReport,
    ),
    // NotBank Profit And Loss report (Realized section header)
    (
        &[
            "AccountId",
            "AccountName",
            "Product",
            "FullName",
            "TimeStamp",
            "ProductQuantity",
            "PurchasePrice",
            "TotalFeeAsProduct",
            "TotalQuantityMinusFee",
            "TotalPurchaseValue",
            "SalePrice",
            "TotalSaleFee",
            "TotalSaleWithFee",
            "P/L",
            "%Return",
        ],
        ExchangeSource::NotBankPnlReport,
    ),
    // MEXC Spot Trade History
    (
        &[
            "UID",
            "Pairs",
            "Time",
            "Type",
            "Direction",
            "Average Filled Price",
            "Order Price",
            "Filled Quantity",
            "Order Quantity",
            "Order Amount",
            "Status",
        ],
        ExchangeSource::MexcSpotTradeHistory,
    ),
    // MEXC Trade History (compact/fills)
    (
        &[
            "UID",
            "Pairs",
            "Time",
            "Side",
            "Filled Price",
            "Executed Amount",
            "Total",
            "Fee",
            "Role",
        ],
        ExchangeSource::MexcTradeHistory,
    ),
    // MEXC Withdrawal History
    (
        &[
            "UID",
            "Status",
            "Time",
            "Crypto",
            "Network",
            "Request Amount",
            "Withdrawal Address",
            "memo",
            "TxID",
            "Trading Fee",
            "Settlement Amount",
            "Withdrawal Descriptions",
        ],
        ExchangeSource::MexcWithdrawalHistory,
    ),
    // MEXC Statement-style exports (Earn, Spot Statement, Futures Statement)
    (
        &[
            "UID",
            "Creation Time(UTC+00:00)",
            "Crypto",
            "Transaction Type",
            "Direction",
            "Quantity",
        ],
        ExchangeSource::MexcStatementHistory,
    ),
    // MEXC Fiat OTC Orders
    (
        &[
            "UID",
            "Order ID",
            "Start Time(UTC+00:00)",
            "End Time(UTC+00:00)",
            "Trading Token",
            "Trading Direction",
            "Status",
            "Order Quantity",
            "Settlement Token",
            "Order Amount",
            "Payment Method",
        ],
        ExchangeSource::MexcFiatOtcOrders,
    ),
    // MEXC Fiat P2P Orders
    (
        &[
            "UID",
            "P2P Type",
            "User UID",
            "Opponent UID",
            "Start Time(UTC+00:00)",
            "End Time(UTC+00:00)",
            "Trading Token",
            "Trading Direction",
            "Status",
            "Order Quantity",
            "Price",
            "Fee",
            "Settlement Token",
            "Order Amount",
        ],
        ExchangeSource::MexcFiatP2pOrders,
    ),
    // MEXC Funding Other History
    (
        &[
            "UID", "Time", "Crypto", "Type", "Quantity", "Status", "Remark",
        ],
        ExchangeSource::MexcFundingOtherHistory,
    ),
    // MEXC Funding Transfer History
    (
        &[
            "UID",
            "From System",
            "To System",
            "Currency",
            "Amount",
            "Status",
            "update_time(UTC+00:00)",
            "create_time(UTC+00:00)",
            "Transfer Type",
        ],
        ExchangeSource::MexcFundingTransferHistory,
    ),
    // MEXC Futures Copy Trade Order History
    (
        &[
            "UID",
            "copy_trader_uid",
            "copy_state",
            "futures",
            "used_margin",
            "leverage",
            "open_type",
            "vol(Cont)",
            "copy_amount(USDT)",
            "deal_avg_price",
            "close_avg_price",
            "Position Profit/Loss(USDT)",
            "fee",
            "create_time(UTC+00:00)",
            "close_time(UTC+00:00)",
        ],
        ExchangeSource::MexcFuturesCopyTradeOrderHistory,
    ),
    // MEXC Futures Capital Flow
    (
        &[
            "UID",
            "Time(UTC+00:00)",
            "Futures Trading Pair",
            "Crypto",
            "Fund Type",
            "Fund Flow Type",
            "Amount",
        ],
        ExchangeSource::MexcFuturesCapitalFlow,
    ),
    // MEXC Futures Order History
    (
        &[
            "UID",
            "Time(UTC+00:00)",
            "Futures Trading Pair",
            "Direction",
            "Leverage",
            "Order Type",
            "Order Qty (Cont.)",
            "Filled Qty (Cont.)",
            "Order Qty (Crypto)",
            "Filled Qty (Crypto)",
            "Order Qty (Amount)",
            "Filled Qty (Amount)",
            "Order Price",
            "Average Filled Price",
            "Closing PNL",
            "Trading Fee",
            "Fee-payment Crypto",
            "Status",
        ],
        ExchangeSource::MexcFuturesOrderHistory,
    ),
    // MEXC Futures Position History
    (
        &[
            "UID",
            "Futures",
            "Open Time(UTC+00:00)",
            "Close Time",
            "Margin Mode",
            "Avg Entry Price",
            "Avg Close Price",
            "Direction",
            "Closing Qty (Cont.)",
            "Fee",
            "Realized PNL",
            "Status",
        ],
        ExchangeSource::MexcFuturesPositionHistory,
    ),
    // MEXC Futures Trade History
    (
        &[
            "UID",
            "Time(UTC+00:00)",
            "Futures Trading Pair",
            "Direction",
            "Order Type",
            "Filled Qty (Cont.)",
            "Filled Qty (Crypto)",
            "Filled Qty (Amount)",
            "Filled Price",
            "Trading Fee",
            "Fee-payment Crypto",
            "Role",
            "Closing PNL",
        ],
        ExchangeSource::MexcFuturesTradeHistory,
    ),
    // MEXC Deposit History
    (
        &[
            "UID",
            "Status",
            "Time",
            "Crypto",
            "Network",
            "Deposit Amount",
            "TxID",
            "Progress",
        ],
        ExchangeSource::MexcDepositHistory,
    ),
    // Monero GUI Wallet (official)
    (
        &[
            "blockHeight",
            "epoch",
            "date",
            "direction",
            "amount",
            "atomicAmount",
            "fee",
            "txid",
            "label",
            "subaddrAccount",
            "paymentId",
            "description",
        ],
        ExchangeSource::MoneroGuiWallet,
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
/// the first few non-empty lines for a matching header. Returns `None` if
/// the format is not recognized.
///
/// Leading BOM characters are stripped automatically.
pub fn detect_exchange_source(content: &str) -> Option<ExchangeSource> {
    let lines: Vec<&str> = content
        .trim_start_matches('\u{feff}') // strip UTF-8 BOM
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(5)
        .collect();

    if lines.is_empty() {
        return None;
    }

    // Some files (e.g., NotBank PnL) include a non-CSV section title before
    // the actual header. We inspect the first few non-empty lines to find the
    // first matching header signature.
    for line in lines {
        let headers: Vec<&str> = line
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
        ExchangeSource::MoneroGuiWallet => Box::new(monero_gui::MoneroGuiParser),
        ExchangeSource::MexcSpotTradeHistory => Box::new(mexc::MexcSpotParser),
        ExchangeSource::MexcTradeHistory => Box::new(mexc::MexcTradeParser),
        ExchangeSource::MexcDepositHistory => Box::new(mexc::MexcDepositParser),
        ExchangeSource::MexcWithdrawalHistory => Box::new(mexc::MexcWithdrawalParser),
        ExchangeSource::MexcStatementHistory => Box::new(mexc::MexcStatementParser),
        ExchangeSource::MexcFiatOtcOrders => Box::new(mexc::MexcFiatOtcParser),
        ExchangeSource::MexcFiatP2pOrders => Box::new(mexc::MexcFiatP2pParser),
        ExchangeSource::MexcFundingOtherHistory => Box::new(mexc::MexcFundingOtherParser),
        ExchangeSource::MexcFundingTransferHistory => Box::new(mexc::MexcFundingTransferParser),
        ExchangeSource::MexcFuturesCopyTradeOrderHistory => {
            Box::new(mexc::MexcFuturesCopyTradeOrderParser)
        }
        ExchangeSource::MexcFuturesCapitalFlow => Box::new(mexc::MexcFuturesCapitalFlowParser),
        ExchangeSource::MexcFuturesOrderHistory => Box::new(mexc::MexcFuturesOrderHistoryParser),
        ExchangeSource::MexcFuturesPositionHistory => {
            Box::new(mexc::MexcFuturesPositionHistoryParser)
        }
        ExchangeSource::MexcFuturesTradeHistory => Box::new(mexc::MexcFuturesTradeHistoryParser),
        ExchangeSource::NotBankTransactions => Box::new(notbank::NotBankTransactionParser),
        ExchangeSource::NotBankTradeActivity => Box::new(notbank::NotBankTradeParser),
        ExchangeSource::NotBankPnlReport => Box::new(notbank::NotBankPnlParser),
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
    fn detect_notbank_transactions() {
        let csv = "\"RegisteredEntityId\",\"PostingEntryId\",\"PostingEntryType\",\"PostingDatetime\",\"AccountId\",\"AccountName\",\"Product\",\"CR\",\"DR\",\"ReferenceTransactionType\",\"ReferenceTransactionId\",\"SystemRecordReference\",\"OMSId\",\"Balance\"\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::NotBankTransactions)
        );
    }

    #[test]
    fn detect_notbank_trade_activity() {
        let csv = "\"RegisteredEntityId\",\"TransReportId\",\"TransReportRevision\",\"TransReportType\",\"OrderId\",\"ClientOrderId\",\"QuoteId\",\"ExtTradeReportId\",\"TradeId\",\"TransReportDatetime\",\"Side\",\"Quantity\",\"Instrument\",\"Price\",\"InsideBid\",\"InsideBidSize\",\"InsideOffer\",\"InsideOfferSize\",\"LeavesSize\",\"MakerTaker\",\"Trader\",\"AccountId\",\"AccountName\",\"Fee\",\"FeeProduct\",\"Notional\",\"BaseSettlementAmount\",\"CounterpartySettlementAmount\",\"OMSId\"\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::NotBankTradeActivity)
        );
    }

    #[test]
    fn detect_notbank_pnl_with_section_header() {
        let csv = "Unrealized Gain/Loss\n\"AccountId\",\"AccountName\",\"Product\",\"FullName\",\"TimeStamp\",\"ProductQuantity\",\"PurchasePrice\",\"TotalFeeAsProduct\",\"TotalQuantityMinusFee\",\"TotalPurchaseValue\",\"ProductEndPrice\",\"TotalSaleValue\",\"P/L\",\"%Return\"\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::NotBankPnlReport)
        );
    }

    #[test]
    fn detect_monero_gui_wallet() {
        let csv = "blockHeight,epoch,date,direction,amount,atomicAmount,fee,txid,label,subaddrAccount,paymentId,description\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MoneroGuiWallet)
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
    fn detect_mexc_spot() {
        let csv = "UID,Pairs,Time,Type,Direction,Average Filled Price,Order Price,Filled Quantity,Order Quantity,Order Amount,Status\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcSpotTradeHistory)
        );
    }

    #[test]
    fn detect_mexc_trades() {
        let csv = "UID,Pairs,Time,Side,Filled Price,Executed Amount,Total,Fee,Role\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcTradeHistory)
        );
    }

    #[test]
    fn detect_mexc_withdrawals() {
        let csv = "UID,Status,Time,Crypto,Network,Request Amount,Withdrawal Address,memo,TxID,Trading Fee,Settlement Amount,Withdrawal Descriptions\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcWithdrawalHistory)
        );
    }

    #[test]
    fn detect_mexc_deposits() {
        let csv = "UID,Status,Time,Crypto,Network,Deposit Amount,TxID,Progress\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcDepositHistory)
        );
    }

    #[test]
    fn detect_mexc_statement() {
        let csv = "UID,Creation Time(UTC+00:00),Crypto,Transaction Type,Direction,Quantity\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcStatementHistory)
        );
    }

    #[test]
    fn detect_mexc_fiat_otc_orders() {
        let csv = "UID,Order ID,Start Time(UTC+00:00),End Time(UTC+00:00),Trading Token,Trading Direction,Status,Order Quantity,Settlement Token,Order Amount,Payment Method\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFiatOtcOrders)
        );
    }

    #[test]
    fn detect_mexc_fiat_p2p_orders() {
        let csv = "UID,P2P Type,User UID,Opponent UID,Start Time(UTC+00:00),End Time(UTC+00:00),Trading Token,Trading Direction,Status,Order Quantity,Price,Fee,Settlement Token,Order Amount\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFiatP2pOrders)
        );
    }

    #[test]
    fn detect_mexc_funding_other() {
        let csv = "UID,Time,Crypto,Type,Quantity,Status,Remark\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFundingOtherHistory)
        );
    }

    #[test]
    fn detect_mexc_funding_transfer() {
        let csv = "UID,From System,To System,Currency,Amount,Status,update_time(UTC+00:00),create_time(UTC+00:00),Transfer Type\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFundingTransferHistory)
        );
    }

    #[test]
    fn detect_mexc_futures_copy_order_history() {
        let csv = "UID,copy_trader_uid,copy_state,futures,used_margin,leverage,open_type,vol(Cont),copy_amount(USDT),deal_avg_price,close_avg_price,Position Profit/Loss(USDT),fee,create_time(UTC+00:00),close_time(UTC+00:00)\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFuturesCopyTradeOrderHistory)
        );
    }

    #[test]
    fn detect_mexc_futures_capital_flow() {
        let csv =
            "UID,Time(UTC+00:00),Futures Trading Pair,Crypto,Fund Type,Fund Flow Type,Amount\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFuturesCapitalFlow)
        );
    }

    #[test]
    fn detect_mexc_futures_order_history() {
        let csv = "UID,Time(UTC+00:00),Futures Trading Pair,Direction,Leverage,Order Type,Order Qty (Cont.),Filled Qty (Cont.),Order Qty (Crypto),Filled Qty (Crypto),Order Qty (Amount),Filled Qty (Amount),Order Price,Average Filled Price,Closing PNL,Trading Fee,Fee-payment Crypto,Status\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFuturesOrderHistory)
        );
    }

    #[test]
    fn detect_mexc_futures_position_history() {
        let csv = "UID,Futures,Open Time(UTC+00:00),Close Time,Margin Mode,Avg Entry Price,Avg Close Price,Direction,Closing Qty (Cont.),Fee,Realized PNL,Status\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFuturesPositionHistory)
        );
    }

    #[test]
    fn detect_mexc_futures_trade_history() {
        let csv = "UID,Time(UTC+00:00),Futures Trading Pair,Direction,Order Type,Filled Qty (Cont.),Filled Qty (Crypto),Filled Qty (Amount),Filled Price,Trading Fee,Fee-payment Crypto,Role,Closing PNL\n";
        assert_eq!(
            detect_exchange_source(csv),
            Some(ExchangeSource::MexcFuturesTradeHistory)
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
            ExchangeSource::MoneroGuiWallet,
            ExchangeSource::MexcSpotTradeHistory,
            ExchangeSource::MexcTradeHistory,
            ExchangeSource::MexcDepositHistory,
            ExchangeSource::MexcWithdrawalHistory,
            ExchangeSource::MexcStatementHistory,
            ExchangeSource::MexcFiatOtcOrders,
            ExchangeSource::MexcFiatP2pOrders,
            ExchangeSource::MexcFundingOtherHistory,
            ExchangeSource::MexcFundingTransferHistory,
            ExchangeSource::MexcFuturesCopyTradeOrderHistory,
            ExchangeSource::MexcFuturesCapitalFlow,
            ExchangeSource::MexcFuturesOrderHistory,
            ExchangeSource::MexcFuturesPositionHistory,
            ExchangeSource::MexcFuturesTradeHistory,
            ExchangeSource::NotBankTransactions,
            ExchangeSource::NotBankTradeActivity,
            ExchangeSource::NotBankPnlReport,
        ];
        for source in sources {
            assert!(!source.label().is_empty());
            assert!(!source.id().is_empty());
            assert!(!source.default_wallet_name().is_empty());
        }
    }
}
