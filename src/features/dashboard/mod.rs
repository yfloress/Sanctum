//! Dashboard feature module
//!
//! Charts and aggregation logic for the dashboard view.
//! Combines data from finance and crypto to provide unified insights.

pub mod charts;

pub use charts::{DashboardCharts, DashboardData, ExpenseSlice};
