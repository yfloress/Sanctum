//! Crypto database operations
//!
//! Split into focused submodules:
//! - prices: Exchange rates, crypto prices, portfolio snapshots
//! - wallets: Wallet CRUD operations
//! - transactions: Transaction CRUD operations
//! - portfolio: Balance calculations and portfolio aggregation

mod portfolio;
mod prices;
mod transactions;
mod wallets;
