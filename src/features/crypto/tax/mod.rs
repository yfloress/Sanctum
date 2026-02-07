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
    is_loss_only_subtype, normalize_tax_subtype, normalize_tax_type,
};

use crate::models::CryptoTransaction;

pub fn resolve_tax_type(tx: &CryptoTransaction) -> TaxTxType {
    if let Some(value) = tx.tax_type.as_deref()
        && let Some(normalized) = TaxTxType::parse(value)
    {
        return normalized;
    }

    match tx.transaction_type.as_str() {
        "transfer_in" | "transfer_out" => TaxTxType::Transfer,
        "buy" | "sell" | "swap" => TaxTxType::Trade,
        _ => TaxTxType::Trade,
    }
}

pub fn resolve_tax_subtype(tx: &CryptoTransaction) -> Option<String> {
    let tax_type = resolve_tax_type(tx);
    let raw = tx.tax_subtype.as_deref()?;
    normalize_tax_subtype(tax_type.as_str(), raw)
}
