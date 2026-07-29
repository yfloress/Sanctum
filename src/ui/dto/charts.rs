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

//! Chart data DTOs for ECharts.
//!
//! These replace plotters PNG rendering. The backend sends raw data series
//! and the frontend renders them with Apache ECharts.

use serde::Serialize;

/// Generic time series for line/area charts.
#[derive(Debug, Clone, Serialize)]
pub struct TimeSeriesData {
    pub dates: Vec<String>,
    pub values: Vec<f64>,
    pub label: Option<String>,
}

/// Bar chart data (e.g., income vs expenses by month).
#[derive(Debug, Clone, Serialize)]
pub struct BarChartData {
    pub labels: Vec<String>,
    pub series: Vec<BarChartSeries>,
}

/// A single series in a bar chart.
#[derive(Debug, Clone, Serialize)]
pub struct BarChartSeries {
    pub name: String,
    pub values: Vec<f64>,
    pub color: Option<String>,
}

/// Pie/donut chart data (e.g., category breakdown).
#[derive(Debug, Clone, Serialize)]
pub struct PieChartData {
    pub items: Vec<PieChartItem>,
}

/// A single slice in a pie chart.
#[derive(Debug, Clone, Serialize)]
pub struct PieChartItem {
    pub name: String,
    pub value: f64,
    pub color: Option<String>,
}
