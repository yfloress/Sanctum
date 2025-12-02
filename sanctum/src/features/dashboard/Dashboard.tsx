import type { Transaction, BalanceSummary } from "../../types";
import { formatAmount, formatDate } from "../../utils";

interface DashboardProps {
  balance: BalanceSummary;
  transactions: Transaction[];
  onDeleteTransaction: (id: string) => void;
  isLoading: boolean;
}

export function Dashboard({
  balance,
  transactions,
  onDeleteTransaction,
  isLoading,
}: DashboardProps) {
  return (
    <div className="dashboard">
      <h1 className="page-title">Dashboard</h1>
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

      <div className="recent-transactions">
        <h2 className="section-title">Recent Transactions</h2>
        {transactions.length === 0 ? (
          <p className="empty-state">No transactions recorded</p>
        ) : (
          <div className="transactions-list">
            {transactions.slice(0, 5).map((tx) => (
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
  );
}
