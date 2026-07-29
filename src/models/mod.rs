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

mod crypto;
mod finance;

pub use crypto::*;
pub use finance::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ==================== Account Tests ====================

    #[test]
    fn test_account_new() {
        let account = Account::new(
            "123".to_string(),
            "My Bank".to_string(),
            "bank".to_string(),
            "USD".to_string(),
            10000,
            "#8b5cf6".to_string(),
            Some("🏦".to_string()),
            "2024-01-01T00:00:00Z".to_string(),
        );

        assert_eq!(account.id, "123");
        assert_eq!(account.name, "My Bank");
        assert_eq!(account.account_type, "bank");
        assert_eq!(account.currency, "USD");
        assert_eq!(account.initial_balance, 10000);
        assert!(!account.is_archived);
    }

    #[test]
    fn test_account_validate_valid() {
        let account = Account::new(
            "123".to_string(),
            "My Bank".to_string(),
            "bank".to_string(),
            "USD".to_string(),
            0,
            "#8b5cf6".to_string(),
            None,
            "2024-01-01T00:00:00Z".to_string(),
        );
        assert!(account.validate());
    }

    #[test]
    fn test_account_validate_empty_name() {
        let account = Account::new(
            "123".to_string(),
            "   ".to_string(),
            "bank".to_string(),
            "USD".to_string(),
            0,
            "#8b5cf6".to_string(),
            None,
            "2024-01-01T00:00:00Z".to_string(),
        );
        assert!(!account.validate());
    }

    #[test]
    fn test_account_validate_invalid_type() {
        let account = Account::new(
            "123".to_string(),
            "My Bank".to_string(),
            "invalid_type".to_string(),
            "USD".to_string(),
            0,
            "#8b5cf6".to_string(),
            None,
            "2024-01-01T00:00:00Z".to_string(),
        );
        assert!(!account.validate());
    }

    #[test]
    fn test_account_validate_invalid_color() {
        let account = Account::new(
            "123".to_string(),
            "My Bank".to_string(),
            "bank".to_string(),
            "USD".to_string(),
            0,
            "not-a-color".to_string(),
            None,
            "2024-01-01T00:00:00Z".to_string(),
        );
        assert!(!account.validate());
    }

    #[test]
    fn test_account_type_parsing() {
        assert_eq!(AccountType::from_str("bank").unwrap(), AccountType::Bank);
        assert_eq!(AccountType::from_str("cash").unwrap(), AccountType::Cash);
        assert_eq!(
            AccountType::from_str("savings").unwrap(),
            AccountType::Savings
        );
        assert_eq!(
            AccountType::from_str("credit_card").unwrap(),
            AccountType::CreditCard
        );
        assert_eq!(AccountType::from_str("other").unwrap(), AccountType::Other);
        assert!(AccountType::from_str("invalid").is_err());
    }

    #[test]
    fn test_account_type_as_str() {
        assert_eq!(AccountType::Bank.as_str(), "bank");
        assert_eq!(AccountType::Cash.as_str(), "cash");
        assert_eq!(AccountType::Savings.as_str(), "savings");
        assert_eq!(AccountType::CreditCard.as_str(), "credit_card");
        assert_eq!(AccountType::Other.as_str(), "other");
    }

    // ==================== Transaction Tests ====================

    #[test]
    fn test_transaction_new() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Groceries".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );

        assert_eq!(tx.id, "tx1");
        assert_eq!(tx.account_id, "acc1");
        assert_eq!(tx.amount, 5000);
        assert_eq!(tx.category, "Food");
        assert_eq!(tx.transaction_type, "expense");
        assert!(tx.transfer_account_id.is_none());
    }

    #[test]
    fn test_transaction_validate_valid_expense() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );
        assert!(tx.validate());
    }

    #[test]
    fn test_transaction_validate_valid_income() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Salary".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "income".to_string(),
            None,
        );
        assert!(tx.validate());
    }

    #[test]
    fn test_transaction_validate_valid_transfer() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Transfer".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "transfer".to_string(),
            Some("acc2".to_string()),
        );
        assert!(tx.validate());
    }

    #[test]
    fn test_transaction_validate_transfer_missing_destination() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Transfer".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "transfer".to_string(),
            None,
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_validate_expense_with_destination() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            Some("acc2".to_string()),
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_validate_zero_amount() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            0,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_validate_invalid_type() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "invalid".to_string(),
            None,
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_validate_empty_account() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );
        assert!(!tx.validate());
    }

    #[test]
    fn test_transaction_type_parsing() {
        assert_eq!(
            FinancialTransactionType::from_str("income").unwrap(),
            FinancialTransactionType::Income
        );
        assert_eq!(
            FinancialTransactionType::from_str("expense").unwrap(),
            FinancialTransactionType::Expense
        );
        assert_eq!(
            FinancialTransactionType::from_str("transfer").unwrap(),
            FinancialTransactionType::Transfer
        );
        assert!(FinancialTransactionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_transaction_type_as_str() {
        assert_eq!(FinancialTransactionType::Income.as_str(), "income");
        assert_eq!(FinancialTransactionType::Expense.as_str(), "expense");
        assert_eq!(FinancialTransactionType::Transfer.as_str(), "transfer");
    }

    #[test]
    fn test_transaction_get_type() {
        let tx = Transaction::new(
            "tx1".to_string(),
            "acc1".to_string(),
            5000,
            "Food".to_string(),
            "Test".to_string(),
            "2024-12-01".to_string(),
            "expense".to_string(),
            None,
        );
        assert_eq!(tx.get_type(), Some(FinancialTransactionType::Expense));
    }

    // ==================== Crypto Transaction Tests ====================

    #[test]
    fn test_crypto_transaction_type_parsing_and_flags() {
        assert_eq!(
            "buy".parse::<CryptoTransactionType>().expect("buy parse"),
            CryptoTransactionType::Buy
        );
        assert_eq!(
            "swap".parse::<CryptoTransactionType>().expect("swap parse"),
            CryptoTransactionType::Swap
        );
        assert!("unknown".parse::<CryptoTransactionType>().is_err());

        assert!(CryptoTransactionType::Buy.is_inflow());
        assert!(CryptoTransactionType::TransferIn.is_inflow());
        assert!(!CryptoTransactionType::Sell.is_inflow());

        assert!(CryptoTransactionType::Sell.is_outflow());
        assert!(CryptoTransactionType::TransferOut.is_outflow());
        assert!(!CryptoTransactionType::Swap.is_outflow());
    }

    #[test]
    fn test_crypto_mechanical_type_from_type_and_subtype() {
        let mut tx = CryptoTransaction::new(
            "c1".to_string(),
            "w1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            "trade".to_string(),
            1.0,
            Some(100.0),
            None,
            "2024-01-01".to_string(),
            None,
        );
        tx.subtype = Some("sell".to_string());
        assert_eq!(tx.mechanical_type(), "sell");
        assert_eq!(tx.get_type(), Some(CryptoTransactionType::Sell));

        tx.transaction_type = "transfer".to_string();
        tx.subtype = Some("withdrawal".to_string());
        assert_eq!(tx.mechanical_type(), "transfer_out");

        tx.transaction_type = "income".to_string();
        tx.subtype = Some("airdrop".to_string());
        assert_eq!(tx.mechanical_type(), "buy");
    }

    #[test]
    fn test_crypto_cost_basis_includes_fee() {
        let tx = CryptoTransaction::new(
            "c2".to_string(),
            "w1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            "trade".to_string(),
            2.0,
            Some(150.0),
            Some(5.0),
            "2024-01-01".to_string(),
            None,
        );
        assert!((tx.cost_basis() - 305.0).abs() < 0.0000001);
    }

    #[test]
    fn test_crypto_transaction_validate_requires_type() {
        let valid = CryptoTransaction::new(
            "c3".to_string(),
            "w1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            "expense".to_string(),
            0.5,
            None,
            None,
            "2024-01-01".to_string(),
            None,
        );
        assert!(valid.validate());

        let invalid_type = CryptoTransaction::new(
            "c4".to_string(),
            "w1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            "buy".to_string(),
            0.5,
            None,
            None,
            "2024-01-01".to_string(),
            None,
        );
        assert!(!invalid_type.validate());
    }
}
