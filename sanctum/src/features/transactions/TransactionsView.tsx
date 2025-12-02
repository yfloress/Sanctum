import type { Transaction } from "../../types";
import { formatAmount, formatDate } from "../../utils";

interface TransactionsViewProps {
  // Form state
  amount: string;
  setAmount: (value: string) => void;
  description: string;
  setDescription: (value: string) => void;
  category: string;
  setCategory: (value: string) => void;
  date: string;
  setDate: (value: string) => void;
  isExpense: boolean;
  categories: readonly string[];
  onExpenseToggle: (checked: boolean) => void;
  onAddTransaction: (e: React.FormEvent) => void;

  // List state
  transactions: Transaction[];
  onDeleteTransaction: (id: string) => void;
  isLoading: boolean;
}

export function TransactionsView({
  amount,
  setAmount,
  description,
  setDescription,
  category,
  setCategory,
  date,
  setDate,
  isExpense,
  categories,
  onExpenseToggle,
  onAddTransaction,
  transactions,
  onDeleteTransaction,
  isLoading,
}: TransactionsViewProps) {
  return (
    <div className="transactions-page">
      <h1 className="page-title">Transactions</h1>

      <div className="transactions-layout">
        <div className="transaction-form-section">
          <h2 className="section-title">New Transaction</h2>
          <form onSubmit={onAddTransaction} className="transaction-form">
            <div className="form-row">
              <div className="form-group">
                <label htmlFor="amount">Amount ($)</label>
                <input
                  id="amount"
                  type="number"
                  step="0.01"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0.00"
                  disabled={isLoading}
                />
              </div>
              <div className="form-group">
                <label htmlFor="date">Date</label>
                <input
                  id="date"
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                  disabled={isLoading}
                />
              </div>
            </div>

            <div className="form-group">
              <label htmlFor="category">Category</label>
              <select
                id="category"
                value={category}
                onChange={(e) => setCategory(e.target.value)}
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
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Describe the transaction"
                disabled={isLoading}
              />
            </div>

            <div className="form-group">
              <label className="switch-label">
                <input
                  type="checkbox"
                  checked={isExpense}
                  onChange={(e) => onExpenseToggle(e.target.checked)}
                  disabled={isLoading}
                />
                <span className="switch-text">
                  {isExpense ? "Expense" : "Income"}
                </span>
              </label>
            </div>

            <button type="submit" className="btn-primary" disabled={isLoading}>
              {isLoading ? "Saving..." : "Save Transaction"}
            </button>
          </form>
        </div>

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
                    <div className="transaction-date">{formatDate(tx.date)}</div>
                  </div>
                  <div className="transaction-actions">
                    <div
                      className={`transaction-amount ${tx.type === "income" ? "income" : "expense"}`}
                    >
                      {tx.type === "income" ? "+" : "-"}${formatAmount(tx.amount)}
                    </div>
                    <button
                      className="btn-delete"
                      onClick={() => onDeleteTransaction(tx.id)}
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
