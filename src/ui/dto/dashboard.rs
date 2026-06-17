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

//! Dashboard domain DTOs.
//!
//! Covers: net worth summary, recent transactions, analytics.

use serde::{Deserialize, Serialize};

/// Net worth balance breakdown for the dashboard hero section.
#[derive(Debug, Clone, Serialize)]
pub struct BalanceOverview {
    pub total: String,
    pub total_negative: bool,
    pub fiat_total: String,
    pub fiat_negative: bool,
    pub crypto_total: String,
    pub crypto_negative: bool,
    pub currency: String,
}

/// A single recent transaction for the dashboard feed.
#[derive(Debug, Clone, Serialize)]
pub struct RecentTransaction {
    pub id: String,
    pub date: String,
    pub description: String,
    pub category: String,
    pub amount: String,
    pub is_expense: bool,
    pub is_transfer: bool,
    pub account_name: String,
}

/// Expense category breakdown for the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct ExpenseBreakdownItem {
    pub category: String,
    pub amount: String,
    pub percentage: f64,
    pub color: String,
}

/// Monthly income vs expense entry for cash flow chart.
#[derive(Debug, Clone, Serialize)]
pub struct MonthlyCashFlowItem {
    pub month: String,
    pub income: f64,
    pub expenses: f64,
}

/// Analytics data for a given time range.
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsData {
    pub net_worth: String,
    pub net_worth_min: String,
    pub net_worth_max: String,
    /// Total income for the selected range (formatted, preferred currency).
    pub total_income: String,
    /// Total expenses for the selected range (formatted, preferred currency).
    pub total_expenses: String,
    /// Net for the selected range (income − expenses, formatted).
    pub total_net: String,
    pub total_net_negative: bool,
    pub expense_breakdown: Vec<ExpenseBreakdownItem>,
    /// Chart data as series for ECharts (replaces plotters PNG).
    pub chart: NetWorthChartData,
    /// Last 6 months income/expense in preferred currency (for cash flow chart).
    pub monthly_cash_flow: Vec<MonthlyCashFlowItem>,
}

/// Input for analytics time range selection.
#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsRangeInput {
    pub range: String, // "1M", "6M", "1Y", "ALL"
}

/// Net worth chart series data for ECharts.
#[derive(Debug, Clone, Serialize)]
pub struct NetWorthChartData {
    /// ISO date strings for x-axis.
    pub dates: Vec<String>,
    /// Net worth values for y-axis.
    pub values: Vec<f64>,
}
