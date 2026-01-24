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

//! Common error types for Sanctum

use thiserror::Error;

/// Custom errors for database operations
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Could not access application data directory")]
    AppDataDir,

    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Could not open vault")]
    InvalidPassword,

    #[error("Could not create data directory")]
    DirectoryCreation,

    #[error("Invalid transaction type")]
    InvalidTransactionType,

    #[error("Wallet not found")]
    WalletNotFound,

    #[error("Invalid wallet category")]
    InvalidWalletCategory,

    #[error("Wallet has existing transactions")]
    WalletNotEmpty,

    #[error("Session expired due to inactivity")]
    SessionExpired,

    #[error("Too many failed attempts")]
    RateLimited,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Account has existing transactions")]
    AccountNotEmpty,

    #[error("Invalid account type")]
    InvalidAccountType,

    #[error("Cannot transfer to the same account")]
    SameAccountTransfer,

    #[error("Transaction not found")]
    TransactionNotFound,
}
