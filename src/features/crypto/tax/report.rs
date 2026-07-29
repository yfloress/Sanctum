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

//! Tax report models and CSV export.

use crate::core::csv_escape;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxReport {
    pub period_id: String,
    pub period_start: String,
    pub period_end: String,
    pub jurisdiction: String,
    pub method: String,
    pub summary: TaxReportSummary,
    pub disposals: Vec<TaxDisposal>,
    pub warnings: Vec<TaxWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaxReportSummary {
    pub disposals: usize,
    pub total_proceeds: f64,
    pub total_cost: f64,
    pub total_gain: f64,
    pub short_term_gain: Option<f64>,
    pub long_term_gain: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxDisposal {
    pub tx_id: String,
    pub date: String,
    pub coin_id: String,
    pub symbol: String,
    pub amount: f64,
    pub proceeds: f64,
    pub cost_basis: f64,
    pub gain: f64,
    pub term: Option<String>,
    pub disposal_type: String,
    pub allocations: Vec<LotAllocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotAllocation {
    pub lot_id: String,
    pub lot_date: String,
    pub quantity: f64,
    pub unit_cost: f64,
    pub cost: f64,
    pub adjusted_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxWarning {
    pub code: String,
    pub message: String,
    pub tx_id: Option<String>,
}

impl TaxReport {
    pub fn to_csv(&self) -> String {
        self.to_csv_with_currency("USD", 0.0)
    }

    pub fn to_csv_with_currency(&self, currency: &str, clp_rate: f64) -> String {
        let convert = |value: f64| {
            if currency != "USD" {
                value * clp_rate
            } else {
                value
            }
        };

        let mut out = String::new();
        out.push_str("period_id,period_start,period_end,jurisdiction,method,currency,disposals,total_proceeds,total_cost,total_gain,short_term_gain,long_term_gain\n");
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{:.2},{:.2},{:.2},{},{}\n",
            csv_escape(&self.period_id),
            csv_escape(&self.period_start),
            csv_escape(&self.period_end),
            csv_escape(&self.jurisdiction),
            csv_escape(&self.method),
            csv_escape(currency),
            self.summary.disposals,
            convert(self.summary.total_proceeds),
            convert(self.summary.total_cost),
            convert(self.summary.total_gain),
            self.summary
                .short_term_gain
                .map(|v| format!("{:.2}", convert(v)))
                .unwrap_or_default(),
            self.summary
                .long_term_gain
                .map(|v| format!("{:.2}", convert(v)))
                .unwrap_or_default(),
        ));

        out.push('\n');
        out.push_str(
            "tx_id,date,coin_id,symbol,amount,proceeds,cost_basis,gain,term,disposal_type,fiat_currency,lot_breakdown\n",
        );

        for disposal in &self.disposals {
            let lot_breakdown = disposal
                .allocations
                .iter()
                .map(|lot| {
                    let adjusted = lot
                        .adjusted_cost
                        .map(|v| format!("{:.4}", convert(v)))
                        .unwrap_or_default();
                    format!(
                        "{}|{}|{:.8}|{:.8}|{:.4}|{}",
                        lot.lot_id,
                        lot.lot_date,
                        lot.quantity,
                        convert(lot.unit_cost),
                        convert(lot.cost),
                        adjusted
                    )
                })
                .collect::<Vec<String>>()
                .join(";");

            out.push_str(&format!(
                "{},{},{},{},{:.8},{:.2},{:.2},{:.2},{},{},{},{}\n",
                csv_escape(&disposal.tx_id),
                csv_escape(&disposal.date),
                csv_escape(&disposal.coin_id),
                csv_escape(&disposal.symbol),
                disposal.amount,
                convert(disposal.proceeds),
                convert(disposal.cost_basis),
                convert(disposal.gain),
                csv_escape(disposal.term.as_deref().unwrap_or("")),
                csv_escape(&disposal.disposal_type),
                csv_escape(currency),
                csv_escape(&lot_breakdown),
            ));
        }

        if !self.warnings.is_empty() {
            out.push('\n');
            out.push_str("warnings\n");
            out.push_str("code,message,tx_id\n");
            for warning in &self.warnings {
                out.push_str(&format!(
                    "{},{},{}\n",
                    csv_escape(&warning.code),
                    csv_escape(&warning.message),
                    csv_escape(warning.tx_id.as_deref().unwrap_or("")),
                ));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_includes_summary_and_warnings() {
        let report = TaxReport {
            period_id: "2024".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            jurisdiction: "usa".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary {
                disposals: 1,
                total_proceeds: 150.0,
                total_cost: 100.0,
                total_gain: 50.0,
                short_term_gain: Some(50.0),
                long_term_gain: Some(0.0),
            },
            disposals: vec![TaxDisposal {
                tx_id: "s1".to_string(),
                date: "2024-02-01".to_string(),
                coin_id: "btc".to_string(),
                symbol: "BTC".to_string(),
                amount: 1.0,
                proceeds: 150.0,
                cost_basis: 100.0,
                gain: 50.0,
                term: Some("short".to_string()),
                disposal_type: "sell".to_string(),
                allocations: vec![LotAllocation {
                    lot_id: "b1".to_string(),
                    lot_date: "2024-01-01".to_string(),
                    quantity: 1.0,
                    unit_cost: 100.0,
                    cost: 100.0,
                    adjusted_cost: None,
                }],
            }],
            warnings: vec![TaxWarning {
                code: "sample_warning".to_string(),
                message: "Missing, price".to_string(),
                tx_id: Some("s1".to_string()),
            }],
        };

        let csv = report.to_csv();
        assert!(csv.contains("period_id,period_start,period_end"));
        assert!(csv.contains("tx_id,date,coin_id"));
        assert!(csv.contains("warnings"));
        assert!(csv.contains("sample_warning"));
    }

    #[test]
    fn csv_with_currency_converts_summary_values_for_clp() {
        let report = TaxReport {
            period_id: "2024".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            jurisdiction: "chile".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary {
                disposals: 1,
                total_proceeds: 10.0,
                total_cost: 7.0,
                total_gain: 3.0,
                short_term_gain: Some(3.0),
                long_term_gain: Some(0.0),
            },
            disposals: Vec::new(),
            warnings: Vec::new(),
        };

        let csv = report.to_csv_with_currency("CLP", 1000.0);
        assert!(csv.contains(",CLP,1,10000.00,7000.00,3000.00,3000.00,0.00"));
    }
}
