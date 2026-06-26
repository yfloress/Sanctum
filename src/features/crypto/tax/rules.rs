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

//! Per-jurisdiction tax policy (Strategy pattern).
//!
//! Every behaviour that varies by [`TaxJurisdiction`] — cost-basis rules, IPC
//! inflation indexing, holding-term classification, export currency and the
//! readiness checklist guidance — is expressed through [`JurisdictionRules`].
//! Adding a country means writing one new impl and one factory arm
//! ([`TaxJurisdiction::rules`]); no `match` on the jurisdiction enum is
//! scattered across the engine or the service layer.

use super::report::TaxReport;
use super::summary::TaxReadinessItem;
use super::types::TaxJurisdiction;

/// The set of tax rules that vary between jurisdictions.
///
/// Implementations are stateless zero-sized types; obtain one via
/// [`TaxJurisdiction::rules`].
pub trait JurisdictionRules {
    /// Whether acquisition fees are capitalised into the cost basis.
    ///
    /// USA capitalises fees; Chile (persona natural, per SII) excludes them.
    fn fee_in_cost_basis(&self) -> bool;

    /// Whether an income `subtype` is recognised at cost **$0** regardless of
    /// fair market value.
    ///
    /// Chile recognises `airdrop`, `staking` and `fork` at zero cost
    /// (Oficio Ord. Nº979/2022); other jurisdictions use FMV.
    fn is_zero_cost_income(&self, subtype: Option<&str>) -> bool;

    /// Whether disposal commissions reduce taxable proceeds.
    ///
    /// Business-style regimes deduct them; Chile persona natural
    /// (art. 17 N°8 m) does not.
    fn deduct_disposal_fees(&self) -> bool;

    /// Whether the cost basis and realised gains are inflation-indexed.
    ///
    /// Chile applies the IPC reajuste; other jurisdictions do not.
    fn inflation_indexed(&self) -> bool;

    /// Whether capital gains are split into short- and long-term holding
    /// buckets (USA and generic regimes).
    fn classifies_holding_term(&self) -> bool;

    /// The reporting/export currency given the user's `preferred` currency.
    ///
    /// Chile always reports in CLP; other jurisdictions honour the preference.
    fn export_currency(&self, preferred: &str) -> String;

    /// The jurisdiction-specific readiness guidance row appended to the
    /// readiness checklist.
    fn readiness_item(&self, report: &TaxReport) -> TaxReadinessItem;
}

/// Chile (persona natural) — SII rules with IPC inflation indexing.
pub struct ChileRules;

/// United States — capital gains with short/long-term holding classification.
pub struct UsaRules;

/// Generic / international regime — USA-style behaviour with neutral guidance.
pub struct OtherRules;

impl JurisdictionRules for ChileRules {
    fn fee_in_cost_basis(&self) -> bool {
        false
    }

    fn is_zero_cost_income(&self, subtype: Option<&str>) -> bool {
        matches!(subtype, Some("airdrop") | Some("staking") | Some("fork"))
    }

    fn deduct_disposal_fees(&self) -> bool {
        false
    }

    fn inflation_indexed(&self) -> bool {
        true
    }

    fn classifies_holding_term(&self) -> bool {
        false
    }

    fn export_currency(&self, _preferred: &str) -> String {
        "CLP".to_string()
    }

    fn readiness_item(&self, report: &TaxReport) -> TaxReadinessItem {
        let detail = if report.summary.total_gain > 0.0 {
            "gain"
        } else if report.summary.total_gain < 0.0 {
            "loss"
        } else {
            "neutral"
        };
        TaxReadinessItem {
            code: "sii_f22".to_string(),
            status: "info".to_string(),
            detail: detail.to_string(),
        }
    }
}

impl JurisdictionRules for UsaRules {
    fn fee_in_cost_basis(&self) -> bool {
        true
    }

    fn is_zero_cost_income(&self, _subtype: Option<&str>) -> bool {
        false
    }

    fn deduct_disposal_fees(&self) -> bool {
        true
    }

    fn inflation_indexed(&self) -> bool {
        false
    }

    fn classifies_holding_term(&self) -> bool {
        true
    }

    fn export_currency(&self, preferred: &str) -> String {
        preferred.to_string()
    }

    fn readiness_item(&self, _report: &TaxReport) -> TaxReadinessItem {
        TaxReadinessItem {
            code: "filing".to_string(),
            status: "info".to_string(),
            detail: "usa".to_string(),
        }
    }
}

impl JurisdictionRules for OtherRules {
    fn fee_in_cost_basis(&self) -> bool {
        true
    }

    fn is_zero_cost_income(&self, _subtype: Option<&str>) -> bool {
        false
    }

    fn deduct_disposal_fees(&self) -> bool {
        true
    }

    fn inflation_indexed(&self) -> bool {
        false
    }

    fn classifies_holding_term(&self) -> bool {
        true
    }

    fn export_currency(&self, preferred: &str) -> String {
        preferred.to_string()
    }

    fn readiness_item(&self, _report: &TaxReport) -> TaxReadinessItem {
        TaxReadinessItem {
            code: "filing".to_string(),
            status: "info".to_string(),
            detail: "other".to_string(),
        }
    }
}

impl TaxJurisdiction {
    /// Returns the [`JurisdictionRules`] strategy for this jurisdiction.
    ///
    /// This is the single point that maps the jurisdiction enum to behaviour;
    /// every other site dispatches polymorphically through the returned trait
    /// object.
    pub fn rules(self) -> &'static dyn JurisdictionRules {
        match self {
            TaxJurisdiction::Chile => &ChileRules,
            TaxJurisdiction::Usa => &UsaRules,
            TaxJurisdiction::Other => &OtherRules,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crypto::tax::{TaxReportSummary, TaxWarning};

    fn report_with_gain(total_gain: f64) -> TaxReport {
        TaxReport {
            period_id: "2025".to_string(),
            period_start: "2025-01-01".to_string(),
            period_end: "2025-12-31".to_string(),
            jurisdiction: "chile".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary {
                total_gain,
                ..TaxReportSummary::default()
            },
            disposals: Vec::new(),
            warnings: Vec::<TaxWarning>::new(),
        }
    }

    #[test]
    fn chile_rules_match_sii_behaviour() {
        let r = TaxJurisdiction::Chile.rules();
        assert!(!r.fee_in_cost_basis());
        assert!(!r.deduct_disposal_fees());
        assert!(r.inflation_indexed());
        assert!(!r.classifies_holding_term());
        assert!(r.is_zero_cost_income(Some("airdrop")));
        assert!(r.is_zero_cost_income(Some("staking")));
        assert!(r.is_zero_cost_income(Some("fork")));
        assert!(!r.is_zero_cost_income(Some("interest")));
        assert!(!r.is_zero_cost_income(None));
        assert_eq!(r.export_currency("USD"), "CLP");
    }

    #[test]
    fn usa_and_other_rules_match_capital_gains_behaviour() {
        for r in [TaxJurisdiction::Usa.rules(), TaxJurisdiction::Other.rules()] {
            assert!(r.fee_in_cost_basis());
            assert!(r.deduct_disposal_fees());
            assert!(!r.inflation_indexed());
            assert!(r.classifies_holding_term());
            assert!(!r.is_zero_cost_income(Some("airdrop")));
            assert_eq!(r.export_currency("EUR"), "EUR");
        }
    }

    #[test]
    fn readiness_item_is_jurisdiction_specific() {
        let chile_gain = ChileRules.readiness_item(&report_with_gain(10.0));
        assert_eq!(chile_gain.code, "sii_f22");
        assert_eq!(chile_gain.detail, "gain");
        assert_eq!(
            ChileRules.readiness_item(&report_with_gain(-5.0)).detail,
            "loss"
        );
        assert_eq!(
            ChileRules.readiness_item(&report_with_gain(0.0)).detail,
            "neutral"
        );

        let usa = UsaRules.readiness_item(&report_with_gain(0.0));
        assert_eq!(usa.code, "filing");
        assert_eq!(usa.detail, "usa");
        assert_eq!(
            OtherRules.readiness_item(&report_with_gain(0.0)).detail,
            "other"
        );
    }
}
