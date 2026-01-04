//! Finance feature module
//!
//! Handles FIAT accounts, transactions, and categories.
//!
//! Split into focused submodules:
//! - `repository` - Database operations
//! - `service` - Core service and account operations
//! - `transactions` - Transaction and transfer operations
//! - `validation` - Input validation helpers
//!
//! Note: Charts moved to `features/dashboard/` as it serves the dashboard view.

pub mod repository;
pub mod service;
pub mod transactions;
pub mod validation;

pub use repository::FinanceRepository;
pub use service::{FinanceError, FinanceService};

// Re-export chart types from dashboard
pub use super::dashboard::{DashboardData, ExpenseSlice};
