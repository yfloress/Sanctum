//! Finance feature module
//!
//! Handles FIAT accounts, transactions, categories, and financial analytics.
//!
//! Split into focused submodules:
//! - `repository` - Database operations
//! - `service` - Core service and account operations
//! - `transactions` - Transaction and transfer operations
//! - `analytics` - Financial analytics and reporting
//! - `validation` - Input validation helpers

pub mod analytics;
pub mod repository;
pub mod service;
pub mod transactions;
pub mod validation;

pub use analytics::{AnalyticsSummary, ExpenseSlice};
pub use repository::FinanceRepository;
pub use service::{FinanceError, FinanceService};
