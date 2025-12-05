/**
 * Transactions View Component
 *
 * Displays transaction form and history list.
 * Consumes state directly from Zustand stores - no props needed.
 */

import { useEffect, type FormEvent } from "react";
import {
  useFinancialLoading,
  useFinancialStore,
  useTransactionForm,
  useTransactions,
} from "../../stores/index.ts";
import { useAccounts, useAccountStore } from "../../stores/accountStore.ts";
import { formatCurrency, formatDate } from "../../utils/index.ts";
import type { CurrencyCode } from "../../utils/index.ts";

export function TransactionsView() {
  // ==================== Financial Store ====================
  const transactions = useTransactions();
  const form = useTransactionForm();
  const isLoading = useFinancialLoading();

  const setFormField = useFinancialStore((state) => state.setFormField);
  const toggleExpenseType = useFinancialStore(
    (state) => state.toggleExpenseType,
  );
  const addTransaction = useFinancialStore((state) => state.addTransaction);
  const setTransactionToDelete = useFinancialStore(
    (state) => state.setTransactionToDelete,
  );
  const getCategories = useFinancialStore((state) => state.getCategories);
  const setDefaultAccount = useFinancialStore(
    (state) => state.setDefaultAccount,
  );

  // ==================== Account Store ====================
  const accounts = useAccounts();
  const loadBalances = useAccountStore((state) => state.loadBalances);

  // Get only active accounts
  const activeAccounts = accounts.filter((acc) => !acc.is_archived);

  // Get categories based on current expense type
  const categories = getCategories();

  // Set default account if none selected and accounts are available
  useEffect(() => {
    if (!form.accountId && activeAccounts.length > 0) {
      setDefaultAccount(activeAccounts[0].id);
    }
  }, [form.accountId, activeAccounts, setDefaultAccount]);

  // Helper to get account name by ID
  const getAccountName = (accountId: string) => {
    const account = accounts.find((acc) => acc.id === accountId);
    return account?.name || "Unknown";
  };

  // Helper to get account color by ID
  const getAccountColor = (accountId: string) => {
    const account = accounts.find((acc) => acc.id === accountId);
    return account?.color || "#8b5cf6";
  };

  // Helper to get account currency by ID
  const getAccountCurrency = (accountId: string): CurrencyCode => {
    const account = accounts.find((acc) => acc.id === accountId);
    return (account?.currency as CurrencyCode) || "USD";
  };

  // Handle form submission
  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    const success = await addTransaction(form);
    if (success) {
      // Reload account balances after adding transaction
      await loadBalances();
    }
  };

  return (
    <div className="transactions-page">
      <h1 className="page-title">Transactions</h1>

      <div className="transactions-layout">
        {/* Transaction Form */}
        <div className="transaction-form-section">
          <h2 className="section-title">New Transaction</h2>
          <form onSubmit={handleSubmit} className="transaction-form">
            {/* Account Selector */}
            <div className="form-group">
              <label htmlFor="account">Account</label>
              {activeAccounts.length === 0 ? (
                <div className="account-warning">
                  <span>⚠️ No accounts available.</span>
                  <span className="hint">
                    Create an account first in the Accounts tab.
                  </span>
                </div>
              ) : (
                <select
                  id="account"
                  value={form.accountId}
                  onChange={(e) => setFormField("accountId", e.target.value)}
                  disabled={isLoading}
                >
                  {activeAccounts.map((acc) => (
                    <option key={acc.id} value={acc.id}>
                      {acc.icon || "💰"} {acc.name}
                    </option>
                  ))}
                </select>
              )}
            </div>

            <div className="form-row">
              <div className="form-group">
                <label htmlFor="amount">Amount ($)</label>
                <input
                  id="amount"
                  type="number"
                  step="0.01"
                  value={form.amount}
                  onChange={(e) => setFormField("amount", e.target.value)}
                  placeholder="0.00"
                  disabled={isLoading}
                />
              </div>
              <div className="form-group">
                <label htmlFor="date">Date</label>
                <input
                  id="date"
                  type="date"
                  value={form.date}
                  onChange={(e) => setFormField("date", e.target.value)}
                  disabled={isLoading}
                />
              </div>
            </div>

            <div className="form-group">
              <label htmlFor="category">Category</label>
              <select
                id="category"
                value={form.category}
                onChange={(e) => setFormField("category", e.target.value)}
                disabled={isLoading}
              >
                {categories.map((cat) => (
                  <option key={cat} value={cat}>
                    {cat}
                  </option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label htmlFor="description">Description</label>
              <input
                id="description"
                type="text"
                value={form.description}
                onChange={(e) => setFormField("description", e.target.value)}
                placeholder="Describe the transaction"
                disabled={isLoading}
              />
            </div>

            <div className="form-group">
              <label className="switch-label">
                <input
                  type="checkbox"
                  checked={form.isExpense}
                  onChange={(e) => toggleExpenseType(e.target.checked)}
                  disabled={isLoading}
                />
                <span className="switch-text">
                  {form.isExpense ? "Expense" : "Income"}
                </span>
              </label>
            </div>

            <button
              type="submit"
              className="btn-primary"
              disabled={isLoading || activeAccounts.length === 0}
            >
              {isLoading ? "Saving..." : "Save Transaction"}
            </button>
          </form>
        </div>

        {/* Transaction History */}
        <div className="transaction-history-section">
          <h2 className="section-title">History</h2>
          {transactions.length === 0 ? (
            <p className="empty-state">No transactions recorded</p>
          ) : (
            <div className="transactions-list">
              {transactions.map((tx) => (
                <div key={tx.id} className="transaction-item">
                  <div className="transaction-info">
                    <div className="transaction-header-row">
                      <div className="transaction-category">{tx.category}</div>
                      {tx.account_id && (
                        <div
                          className="transaction-account-badge"
                          style={{
                            backgroundColor: getAccountColor(tx.account_id),
                          }}
                        >
                          {getAccountName(tx.account_id)}
                        </div>
                      )}
                    </div>
                    <div className="transaction-description">
                      {tx.description}
                    </div>
                    <div className="transaction-date">
                      {formatDate(tx.date)}
                    </div>
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
                      {formatCurrency(
                        tx.amount,
                        tx.account_id
                          ? getAccountCurrency(tx.account_id)
                          : "USD",
                      )}
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
    </div>
  );
}
