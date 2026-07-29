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

//! MEXC CSV parsers.
//!
//! Supported formats:
//! - Spot trade history
//! - Trade history
//! - Deposit history
//! - Withdrawal history
//! - Statement exports (earn/spot/futures)
//! - Fiat OTC / P2P order exports
//! - Funding other / transfer history
//! - Futures capital / orders / positions / trades

mod deposits;
mod fiat_common;
mod fiat_otc;
mod fiat_p2p;
mod funding;
mod futures_capital;
mod futures_common;
mod futures_copy;
mod futures_reports;
mod spot;
mod statements;
mod trades;
mod withdrawals;

pub use deposits::MexcDepositParser;
pub use fiat_otc::MexcFiatOtcParser;
pub use fiat_p2p::MexcFiatP2pParser;
pub use funding::{MexcFundingOtherParser, MexcFundingTransferParser};
pub use futures_capital::MexcFuturesCapitalFlowParser;
pub use futures_copy::MexcFuturesCopyTradeOrderParser;
pub use futures_reports::{
    MexcFuturesOrderHistoryParser, MexcFuturesPositionHistoryParser, MexcFuturesTradeHistoryParser,
};
pub use spot::MexcSpotParser;
pub use statements::MexcStatementParser;
pub use trades::MexcTradeParser;
pub use withdrawals::MexcWithdrawalParser;

#[cfg(test)]
mod spot_tests;
