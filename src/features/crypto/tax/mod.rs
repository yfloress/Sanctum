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

//! Crypto tax module (offline-first).

pub mod engine;
pub mod ipc;
pub mod report;
pub mod summary;
pub mod types;

pub use ipc::{
    IpcEntry, IpcImportSummary, IpcParsed, IpcSummary, build_import_summary, map_to_entries,
    parse_ipc_csv, summarize_ipc,
};

pub use engine::build_tax_report;
pub use report::{LotAllocation, TaxDisposal, TaxReport, TaxReportSummary, TaxWarning};

pub use summary::{TaxReadinessItem, TaxSummaryPayload};
pub use types::{
    TaxJurisdiction, TaxMethod, TaxPeriodSettings, TaxSettingsStore, TaxTxType,
    is_loss_only_subtype, normalize_tax_subtype,
};

use crate::models::CryptoTransaction;

/// Returns the fiscal category stored in `transaction_type`.
///
/// `type` holds the fiscal category directly
/// (`trade` / `income` / `expense` / `transfer`), so this is a simple parse.
pub fn resolve_tax_type(tx: &CryptoTransaction) -> TaxTxType {
    TaxTxType::parse(&tx.transaction_type).unwrap_or(TaxTxType::Trade)
}

/// Returns the validated subtype (e.g. "airdrop", "deposit", "swap").
pub fn resolve_tax_subtype(tx: &CryptoTransaction) -> Option<String> {
    let tax_type = resolve_tax_type(tx);
    let raw = tx.subtype.as_deref()?;
    normalize_tax_subtype(tax_type.as_str(), raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(fiscal_type: &str, subtype: Option<&str>) -> CryptoTransaction {
        let mut tx = CryptoTransaction::new(
            "tx-1".to_string(),
            "wallet-1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            fiscal_type.to_string(),
            1.0,
            Some(100.0),
            None,
            "2026-01-10".to_string(),
            None,
        );
        tx.subtype = subtype.map(str::to_string);
        tx
    }

    #[test]
    fn resolve_tax_type_reads_fiscal_type_field() {
        assert_eq!(resolve_tax_type(&tx("trade", Some("buy"))), TaxTxType::Trade);
        assert_eq!(
            resolve_tax_type(&tx("income", Some("airdrop"))),
            TaxTxType::Income
        );
        assert_eq!(
            resolve_tax_type(&tx("expense", Some("donation"))),
            TaxTxType::Expense
        );
        assert_eq!(
            resolve_tax_type(&tx("transfer", Some("deposit"))),
            TaxTxType::Transfer
        );
    }

    #[test]
    fn resolve_tax_subtype_requires_valid_subtype_for_selected_type() {
        assert_eq!(
            resolve_tax_subtype(&tx("income", Some("airdrop"))).as_deref(),
            Some("airdrop")
        );
        assert_eq!(
            resolve_tax_subtype(&tx("transfer", Some("withdrawal"))).as_deref(),
            Some("withdrawal")
        );
        assert_eq!(resolve_tax_subtype(&tx("income", Some("sell"))), None);
        assert_eq!(resolve_tax_subtype(&tx("trade", None)), None);
    }
}
