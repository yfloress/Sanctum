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
