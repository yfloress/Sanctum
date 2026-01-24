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
