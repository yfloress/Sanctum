//! Finance feature module
//!
//! Handles FIAT accounts, transactions, categories, and financial analytics.

pub mod models;
pub mod repository;
pub mod service;

pub use models::*;
pub use repository::FinanceRepository;
pub use service::{AnalyticsSummary, ExpenseSlice, FinanceError, FinanceService};
