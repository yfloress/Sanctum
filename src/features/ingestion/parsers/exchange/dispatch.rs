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

use super::{ExchangeSource, ParseResult, binance, feather, kraken, mexc, monero_gui, notbank};
use crate::features::ingestion::types::{ImportCryptoTransaction, RowError};

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
