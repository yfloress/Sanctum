/**
 * Transactions View Component
 *
 * Displays transaction form and history list.
 * Consumes state directly from Zustand stores - no props needed.
 */

import type { FormEvent } from "react";
import {
  useTransactions,
  useTransactionForm,
  useFinancialStore,
  useFinancialLoading,
} from "../../stores";
import { formatAmount, formatDate } from "../../utils";

export function TransactionsView() {
  // Consume state directly from store (optimized selectors)
  const transactions = useTransactions();
  const form = useTransactionForm();
  const isLoading = useFinancialLoading();

  // Get actions from store
  const setFormField = useFinancialStore((state) => state.setFormField);
  const toggleExpenseType = useFinancialStore(
    (state) => state.toggleExpenseType,
  );
  const addTransaction = useFinancialStore((state) => state.addTransaction);
  const setTransactionToDelete = useFinancialStore(
    (state) => state.setTransactionToDelete,
  );
  const getCategories = useFinancialStore((state) => state.getCategories);

  // Get categories based on current expense type
  const categories = getCategories();

  // Handle form submission
  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    await addTransaction(form);
  };

  return (
    <div className="transactions-page">
      <h1 className="page-title">Transactions</h1>

      <div className="transactions-layout">
        {/* Transaction Form */}
        <div className="transaction-form-section">
          <h2 className="section-title">New Transaction</h2>
          <form onSubmit={handleSubmit} className="transaction-form">
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

            <button type="submit" className="btn-primary" disabled={isLoading}>
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
                    <div className="transaction-category">{tx.category}</div>
                    <div className="transaction-description">
                      {tx.description}
                    </div>
                    <div className="transaction-date">
                      {formatDate(tx.date)}
                    </div>
                  </div>
                  <div className="transaction-actions">
                    <div
                      className={`transaction-amount ${tx.type === "income" ? "income" : "expense"}`}
                    >
                      {tx.type === "income" ? "+" : "-"}$
                      {formatAmount(tx.amount)}
                    </div>
                    <button
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
