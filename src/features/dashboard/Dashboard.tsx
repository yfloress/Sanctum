/**
 * Dashboard Component
 *
 * Displays the main financial overview with balance cards, account balances,
 * and recent transactions.
 * Consumes state directly from Zustand stores - no props needed.
 */

import { useMemo } from "react";
import {
  useBalance,
  useFinancialLoading,
  useFinancialStore,
  useTransactions,
} from "../../stores/index.ts";
import {
  useAccounts,
  useAccountBalances,
  useAccountStore,
} from "../../stores/accountStore.ts";
import { formatAmount, formatDate } from "../../utils/index.ts";
import { ACCOUNT_TYPES } from "../../types/index.ts";

export function Dashboard() {
  // ==================== Financial Store ====================
  const transactions = useTransactions();
  const balance = useBalance();
  const isLoading = useFinancialLoading();
  const setTransactionToDelete = useFinancialStore(
    (state) => state.setTransactionToDelete,
  );

  // ==================== Account Store ====================
  const accounts = useAccounts();
  const balances = useAccountBalances();
  const getTotalNetWorthUSD = useAccountStore(
    (state) => state.getTotalNetWorthUSD,
  );

  // ==================== Computed ====================
  const netWorth = getTotalNetWorthUSD();
  const activeAccounts = accounts.filter((acc) => !acc.is_archived);

  // Memoize recent transactions to avoid recalculation on every render
  const recentTransactions = useMemo(
    () => transactions.slice(0, 5),
    [transactions],
  );

  // Get balance for a specific account
  const getBalanceForAccount = (accountId: string) => {
    const accountBalance = balances.find((b) => b.account_id === accountId);
    return accountBalance?.current_balance ?? 0;
  };

  // Get account type info
  const getAccountTypeInfo = (type: string) => {
    return ACCOUNT_TYPES.find((t) => t.value === type) || ACCOUNT_TYPES[4];
  };

  // Get account name by ID
  const getAccountName = (accountId: string) => {
    const account = accounts.find((acc) => acc.id === accountId);
    return account?.name || "Unknown";
  };

  return (
    <div className="dashboard">
      <h1 className="page-title">Dashboard</h1>

      {/* Net Worth Card */}
      <div className="net-worth-card">
        <div className="net-worth-header">
          <span className="net-worth-icon">💰</span>
          <span className="net-worth-title">Total Net Worth</span>
        </div>
        <div
          className={`net-worth-amount ${netWorth >= 0 ? "positive" : "negative"}`}
        >
          {netWorth < 0 ? "-" : ""}${formatAmount(Math.abs(netWorth), "USD")}
        </div>
        <div className="net-worth-subtitle">
          Across {activeAccounts.length} active account
          {activeAccounts.length !== 1 ? "s" : ""}
        </div>
      </div>

      {/* Balance Cards */}
      <div className="balance-cards">
        <div className="balance-card total">
          <span className="balance-label">Total Balance</span>
          <span className="balance-value">
            ${formatAmount(balance.total_balance)}
          </span>
        </div>
        <div className="balance-card income">
          <span className="balance-label">Total Income</span>
          <span className="balance-value">
            +${formatAmount(balance.total_income)}
          </span>
        </div>
        <div className="balance-card expense">
          <span className="balance-label">Total Expenses</span>
          <span className="balance-value">
            -${formatAmount(balance.total_expense)}
          </span>
        </div>
      </div>

      {/* Account Balances Quick View */}
      {activeAccounts.length > 0 && (
        <div className="accounts-quick-view">
          <h2 className="section-title">Account Balances</h2>
          <div className="account-balances-grid">
            {activeAccounts.slice(0, 4).map((account) => {
              const typeInfo = getAccountTypeInfo(account.type);
              const currentBalance = getBalanceForAccount(account.id);

              return (
                <div
                  key={account.id}
                  className="account-balance-card"
                  style={{ borderLeftColor: account.color }}
                >
                  <div className="account-balance-header">
                    <span
                      className="account-balance-icon"
                      style={{ backgroundColor: account.color }}
                    >
                      {account.icon || typeInfo.icon}
                    </span>
                    <span className="account-balance-name">{account.name}</span>
                  </div>
                  <div
                    className={`account-balance-amount ${currentBalance >= 0 ? "positive" : "negative"}`}
                  >
                    {currentBalance < 0 ? "-" : ""}$
                    {formatAmount(Math.abs(currentBalance))}
                  </div>
                </div>
              );
            })}
          </div>
          {activeAccounts.length > 4 && (
            <div className="view-all-hint">
              +{activeAccounts.length - 4} more in Accounts tab
            </div>
          )}
        </div>
      )}

      {/* Recent Transactions */}
      <div className="recent-transactions">
        <h2 className="section-title">Recent Transactions</h2>
        {recentTransactions.length === 0 ? (
          <p className="empty-state">No transactions recorded</p>
        ) : (
          <div className="transactions-list">
            {recentTransactions.map((tx) => (
              <div key={tx.id} className="transaction-item">
                <div className="transaction-info">
                  <div className="transaction-header-row">
                    <div className="transaction-category">{tx.category}</div>
                    {tx.account_id && (
                      <div className="transaction-account-tag">
                        {getAccountName(tx.account_id)}
                      </div>
                    )}
                  </div>
                  <div className="transaction-description">
                    {tx.description}
                  </div>
                  <div className="transaction-date">{formatDate(tx.date)}</div>
                </div>
                <div className="transaction-actions">
                  <div
                    className={`transaction-amount ${
                      tx.type === "income"
                        ? "income"
                        : tx.type === "transfer"
                          ? "transfer"
                          : "expense"
                    }`}
                  >
                    {tx.type === "income"
                      ? "+"
                      : tx.type === "transfer"
                        ? "↔️ "
                        : "-"}
                    ${formatAmount(tx.amount)}
                  </div>
                  <button
                    type="button"
                    className="btn-delete"
                    onClick={() => setTransactionToDelete(tx.id)}
                    disabled={isLoading}
                    aria-label="Delete transaction"
                  >
                    🗑️
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
