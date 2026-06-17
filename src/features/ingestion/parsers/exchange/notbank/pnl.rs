// Sanctum — a privacy-first personal finance and crypto vault.
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

/// NotBank PnL export is a summary report (unrealized/realized sections),
/// not a transactional ledger. We accept/detect it to keep multi-file import
/// flow smooth, but intentionally emit no importable transactions.
pub struct NotBankPnlParser;

impl ExchangeParser for NotBankPnlParser {
    fn parse(
        &self,
        _content: &str,
        _wallet_name: &str,
    ) -> Result<ParseResult<ImportCryptoTransaction>, RowError> {
        Ok(ParseResult::default())
    }

    fn source(&self) -> ExchangeSource {
        ExchangeSource::NotBankPnlReport
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pnl_parser_returns_empty_result() {
        let csv = "Unrealized Gain/Loss\n\
\"AccountId\",\"AccountName\",\"Product\",\"FullName\",\"TimeStamp\",\"ProductQuantity\",\"PurchasePrice\",\"TotalFeeAsProduct\",\"TotalQuantityMinusFee\",\"TotalPurchaseValue\",\"ProductEndPrice\",\"TotalSaleValue\",\"P/L\",\"%Return\"\n\
\"1\",\"Primary\",\"BTC\",\"Bitcoin\",\"2/23/2026 02:31:37 AM\",\"0.1\",\"90000\",\"0\",\"0.1\",\"9000\",\"85000\",\"8500\",\"-500\",\"-5%\"\n";

        let parser = NotBankPnlParser;
        let result = parser.parse(csv, "NotBank").unwrap();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.items.len(), 0);
    }
}
