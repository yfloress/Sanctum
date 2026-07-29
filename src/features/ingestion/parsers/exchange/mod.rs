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

//! Exchange-specific CSV parsers
//!
//! Each submodule handles a specific exchange or wallet CSV format and converts
//! rows into \[ImportCryptoTransaction\] instances that feed into the existing
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
mod dispatch;
pub mod feather;
pub mod kraken;
pub mod mexc;
pub mod monero_gui;
pub mod notbank;
mod validation;

use super::ParseResult;

pub use dispatch::{ExchangeParser, parser_for};
pub use validation::detect_exchange_source;

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

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
