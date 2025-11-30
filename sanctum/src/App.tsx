import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from "recharts";
import "./App.css";

// Colors for pie chart (cyberpunk/neon theme)
const CHART_COLORS = [
  "#8b5cf6", // violet
  "#10b981", // emerald
  "#f59e0b", // amber
  "#ef4444", // red
  "#06b6d4", // cyan
  "#ec4899", // pink
  "#6366f1", // indigo
  "#84cc16", // lime
  "#f97316", // orange
];

interface Transaction {
  id: string;
  amount: number;
  category: string;
  description: string;
  date: string;
  type: string;
}

interface BalanceSummary {
  total_balance: number;
  total_income: number;
  total_expense: number;
}

const EXPENSE_CATEGORIES = [
  "Food",
  "Transport",
  "Housing",
  "Utilities",
  "Health",
  "Entertainment",
  "Education",
  "Technology",
  "Other",
] as const;

const INCOME_CATEGORIES = [
  "Salary",
  "Freelance",
  "Investments",
  "Gifts",
  "Other",
] as const;

function App() {
  const [isInitialized, setIsInitialized] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [dbPathInput, setDbPathInput] = useState("");
  const [successMessage, setSuccessMessage] = useState("");
  const [loadingAction, setLoadingAction] = useState<"open" | "create" | null>(
    null,
  );

  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [category, setCategory] = useState<string>(EXPENSE_CATEGORIES[0]);
  const [isExpense, setIsExpense] = useState(true);
  const [date, setDate] = useState(
    () => new Date().toISOString().split("T")[0],
  );
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [activeTab, setActiveTab] = useState<
    "dashboard" | "transactions" | "analytics"
  >("dashboard");
  const [balance, setBalance] = useState<BalanceSummary>({
    total_balance: 0,
    total_income: 0,
    total_expense: 0,
  });
  const [transactionToDelete, setTransactionToDelete] = useState<string | null>(
    null,
  );

  // Analytics data: expenses grouped by category for pie chart
  const expensesByCategory = useMemo(() => {
    const expenses = transactions.filter((tx) => tx.type === "expense");
    const grouped = expenses.reduce(
      (acc, tx) => {
        acc[tx.category] = (acc[tx.category] || 0) + tx.amount;
        return acc;
      },
      {} as Record<string, number>,
    );

    return Object.entries(grouped)
      .map(([name, value]) => ({ name, value: value / 100 }))
      .sort((a, b) => b.value - a.value);
  }, [transactions]);

  // Analytics data: balance evolution over time for area chart
  const balanceEvolution = useMemo(() => {
    if (transactions.length === 0) return [];

    // Sort by date ascending
    const sorted = [...transactions].sort(
      (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
    );

    // Group by date and calculate daily balance
    const dailyData: Record<string, { income: number; expense: number }> = {};

    sorted.forEach((tx) => {
      const dateKey = new Date(tx.date).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
      });

      if (!dailyData[dateKey]) {
        dailyData[dateKey] = { income: 0, expense: 0 };
      }

      if (tx.type === "income") {
        dailyData[dateKey].income += tx.amount;
      } else {
        dailyData[dateKey].expense += tx.amount;
      }
    });

    // Calculate cumulative balance
    let cumulative = 0;
    return Object.entries(dailyData).map(([date, data]) => {
      cumulative += (data.income - data.expense) / 100;
      return {
        date,
        balance: cumulative,
        income: data.income / 100,
        expense: data.expense / 100,
      };
    });
  }, [transactions]);

  const errorTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const successTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const setTemporaryError = useCallback((message: string, duration = 5000) => {
    if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
    setError(message);
    errorTimeoutRef.current = setTimeout(() => setError(""), duration);
  }, []);

  const setTemporarySuccess = useCallback(
    (message: string, duration = 3000) => {
      if (successTimeoutRef.current) clearTimeout(successTimeoutRef.current);
      setSuccessMessage(message);
      successTimeoutRef.current = setTimeout(
        () => setSuccessMessage(""),
        duration,
      );
    },
    [],
  );

  useEffect(() => {
    return () => {
      if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
      if (successTimeoutRef.current) clearTimeout(successTimeoutRef.current);
    };
  }, []);

  const categories = useMemo(
    () => (isExpense ? EXPENSE_CATEGORIES : INCOME_CATEGORIES),
    [isExpense],
  );

  const loadDbPath = useCallback(async () => {
    try {
      const path = await invoke<string>("get_db_path");
      setDbPathInput(path);
    } catch (err) {
      console.error("Error getting path:", err);
    }
  }, []);

  const loadTransactions = useCallback(async () => {
    try {
      const txs = await invoke<Transaction[]>("get_transactions");
      setTransactions(txs);
    } catch (err) {
      console.error("Error loading transactions:", err);
    }
  }, []);

  const loadBalance = useCallback(async () => {
    try {
      const bal = await invoke<BalanceSummary>("get_balance");
      setBalance(bal);
    } catch (err) {
      console.error("Error loading balance:", err);
    }
  }, []);

  const checkDatabaseStatus = useCallback(async () => {
    try {
      setIsLoading(true);
      setError("");
      const initialized = await invoke<boolean>("is_db_initialized");
      setIsInitialized(initialized);
      await loadDbPath();
      if (initialized) {
        await loadTransactions();
        await loadBalance();
      }
    } catch (err) {
      setError(`Error checking status: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, [loadDbPath, loadTransactions]);

  useEffect(() => {
    checkDatabaseStatus();
  }, [checkDatabaseStatus]);

  const clearMessages = useCallback(() => {
    setError("");
    setSuccessMessage("");
  }, []);

  const handleVaultAction = useCallback(
    async (action: "open" | "create") => {
      clearMessages();

      const trimmedPassword = password.trim();
      if (!trimmedPassword) {
        setTemporaryError("Password cannot be empty");
        return;
      }
      if (trimmedPassword.length < 8) {
        setTemporaryError("Password must be at least 8 characters");
        return;
      }

      const targetPath = dbPathInput.trim() || null;

      try {
        setIsLoading(true);
        setLoadingAction(action);
        const command = action === "create" ? "create_db" : "open_db";
        await invoke<string>(command, {
          password: trimmedPassword,
          path: targetPath,
        });
        setIsInitialized(true);
        setPassword("");
        await loadDbPath();
        await loadTransactions();
        await loadBalance();
      } catch (err) {
        setTemporaryError(`Error: ${err}`);
      } finally {
        setIsLoading(false);
        setLoadingAction(null);
      }
    },
    [
      password,
      dbPathInput,
      clearMessages,
      loadDbPath,
      loadTransactions,
      loadBalance,
      setTemporaryError,
      setTemporarySuccess,
    ],
  );

  const handleCloseVault = useCallback(async () => {
    try {
      setIsLoading(true);
      clearMessages();
      const result = await invoke<string>("close_db");
      setTemporarySuccess(result);
      setIsInitialized(false);
      setTransactions([]);
      await loadDbPath();
    } catch (err) {
      setTemporaryError(`Error: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, [clearMessages, loadDbPath, setTemporaryError, setTemporarySuccess]);

  const handleExpenseToggle = useCallback((checked: boolean) => {
    setIsExpense(checked);
    setCategory(checked ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0]);
  }, []);

  const handleDeleteTransaction = useCallback((id: string) => {
    setTransactionToDelete(id);
  }, []);

  const confirmDelete = useCallback(async () => {
    if (!transactionToDelete) return;

    try {
      setIsLoading(true);
      clearMessages();
      await invoke("delete_transaction", { id: transactionToDelete });
      setTemporarySuccess("Transaction deleted successfully");
      await loadTransactions();
      await loadBalance();
    } catch (err) {
      setTemporaryError(`Error deleting transaction: ${err}`);
    } finally {
      setIsLoading(false);
      setTransactionToDelete(null);
    }
  }, [
    transactionToDelete,
    clearMessages,
    loadTransactions,
    loadBalance,
    setTemporaryError,
    setTemporarySuccess,
  ]);

  const cancelDelete = useCallback(() => {
    setTransactionToDelete(null);
  }, []);

  const handleAddTransaction = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      clearMessages();

      const parsedAmount = parseFloat(amount);
      if (!amount || parsedAmount <= 0) {
        setTemporaryError("Amount must be greater than zero");
        return;
      }
      if (!category.trim()) {
        setTemporaryError("Category cannot be empty");
        return;
      }

      try {
        setIsLoading(true);
        const amountInCents = Math.round(parsedAmount * 100);

        const now = new Date();
        const selectedDate = new Date(date);
        selectedDate.setMinutes(
          selectedDate.getMinutes() + selectedDate.getTimezoneOffset(),
        );
        selectedDate.setHours(
          now.getHours(),
          now.getMinutes(),
          now.getSeconds(),
        );

        await invoke<string>("add_transaction", {
          amount: amountInCents,
          category: category.trim(),
          description: description.trim(),
          date: selectedDate.toISOString(),
          isExpense,
        });

        setTemporarySuccess(
          `${isExpense ? "Expense" : "Income"} added successfully`,
        );

        setAmount("");
        setDescription("");
        setCategory(isExpense ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0]);
        setDate(new Date().toISOString().split("T")[0]);

        await loadTransactions();
        await loadBalance();
      } catch (err) {
        setTemporaryError(`Error creating transaction: ${err}`);
      } finally {
        setIsLoading(false);
      }
    },
    [
      amount,
      category,
      description,
      date,
      isExpense,
      clearMessages,
      loadTransactions,
      loadBalance,
      setTemporaryError,
      setTemporarySuccess,
    ],
  );

  const formatAmount = useCallback(
    (cents: number) => (cents / 100).toFixed(2),
    [],
  );

  const formatDate = useCallback((isoDate: string) => {
    return new Date(isoDate).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }, []);

  if (isLoading && !isInitialized) {
    return (
      <div className="vault-container">
        <div className="vault-card">
          <div className="loader" />
          <p>Checking vault status...</p>
        </div>
      </div>
    );
  }

  if (!isInitialized) {
    return (
      <div className="vault-container">
        {error && <div className="message error login-message">{error}</div>}
        <div className="vault-card login-card">
          <div className="login-layout">
            <div className="login-branding">
              <div className="vault-icon locked">🔒</div>
              <h1>Sanctum</h1>
              <p className="vault-subtitle">Secure Financial Vault</p>
              <p className="vault-tagline">
                Your data is protected with AES-256 encryption
              </p>
            </div>

            <div className="login-form-section">
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  handleVaultAction("open");
                }}
                className="vault-form"
              >
                <div className="form-group password-group">
                  <label htmlFor="password">Master Password</label>
                  <div className="password-input-wrapper">
                    <input
                      id="password"
                      type={showPassword ? "text" : "password"}
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder="Enter your password"
                      disabled={isLoading}
                      autoFocus
                    />
                    <button
                      type="button"
                      className="password-toggle"
                      onClick={() => setShowPassword(!showPassword)}
                      disabled={isLoading}
                      aria-label={
                        showPassword ? "Hide password" : "Show password"
                      }
                    >
                      {showPassword ? "👁️" : "🙈"}
                    </button>
                  </div>
                  <span className="input-hint">Minimum 8 characters</span>
                </div>

                <div className="button-row">
                  <button
                    type="submit"
                    className="btn-primary"
                    disabled={isLoading}
                  >
                    {isLoading && loadingAction === "open" ? "..." : "Unlock"}
                  </button>
                  <button
                    type="button"
                    className="btn-secondary"
                    onClick={() => handleVaultAction("create")}
                    disabled={isLoading}
                  >
                    {isLoading && loadingAction === "create" ? "..." : "Create"}
                  </button>
                </div>

                <details className="path-details">
                  <summary>Advanced options</summary>
                  <div className="form-group path-group">
                    <label htmlFor="db-path">Vault Path</label>
                    <input
                      id="db-path"
                      type="text"
                      value={dbPathInput}
                      onChange={(e) => setDbPathInput(e.target.value)}
                      placeholder="Custom path (empty = default)"
                      disabled={isLoading}
                    />
                    <span className="input-hint">
                      Last used path is remembered automatically
                    </span>
                  </div>
                </details>
              </form>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app-layout">
      <aside className="sidebar">
        <div className="sidebar-logo">
          <span className="logo-icon">🔓</span>
          <span className="logo-text">Sanctum</span>
        </div>

        <nav className="sidebar-nav">
          <button
            className={`nav-item ${activeTab === "dashboard" ? "active" : ""}`}
            onClick={() => setActiveTab("dashboard")}
          >
            <span className="nav-icon">📊</span>
            <span className="nav-label">Dashboard</span>
          </button>
          <button
            className={`nav-item ${activeTab === "transactions" ? "active" : ""}`}
            onClick={() => setActiveTab("transactions")}
          >
            <span className="nav-icon">💸</span>
            <span className="nav-label">Transactions</span>
          </button>
          <button
            className={`nav-item ${activeTab === "analytics" ? "active" : ""}`}
            onClick={() => setActiveTab("analytics")}
          >
            <span className="nav-icon">📈</span>
            <span className="nav-label">Analytics</span>
          </button>
        </nav>

        <div className="sidebar-footer">
          <button
            onClick={handleCloseVault}
            className="nav-item lock-btn"
            disabled={isLoading}
          >
            <span className="nav-icon">🔒</span>
            <span className="nav-label">
              {isLoading ? "Locking..." : "Lock Vault"}
            </span>
          </button>
        </div>
      </aside>

      <main className="content-area">
        {error && <div className="message error">{error}</div>}
        {successMessage && (
          <div className="message success">{successMessage}</div>
        )}

        {activeTab === "dashboard" && (
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
                        <div className="transaction-category">
                          {tx.category}
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
                          className={`transaction-amount ${tx.type === "income" ? "income" : "expense"}`}
                        >
                          {tx.type === "income" ? "+" : "-"}$
                          {formatAmount(tx.amount)}
                        </div>
                        <button
                          className="btn-delete"
                          onClick={() => handleDeleteTransaction(tx.id)}
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
        )}

        {activeTab === "transactions" && (
          <div className="transactions-page">
            <h1 className="page-title">Transactions</h1>

            <div className="transactions-layout">
              <div className="transaction-form-section">
                <h2 className="section-title">New Transaction</h2>
                <form
                  onSubmit={handleAddTransaction}
                  className="transaction-form"
                >
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
                        onChange={(e) => handleExpenseToggle(e.target.checked)}
                        disabled={isLoading}
                      />
                      <span className="switch-text">
                        {isExpense ? "Expense" : "Income"}
                      </span>
                    </label>
                  </div>

                  <button
                    type="submit"
                    className="btn-primary"
                    disabled={isLoading}
                  >
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
                          <div className="transaction-category">
                            {tx.category}
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
                            className={`transaction-amount ${tx.type === "income" ? "income" : "expense"}`}
                          >
                            {tx.type === "income" ? "+" : "-"}$
                            {formatAmount(tx.amount)}
                          </div>
                          <button
                            className="btn-delete"
                            onClick={() => handleDeleteTransaction(tx.id)}
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
        )}

        {activeTab === "analytics" && (
          <div className="analytics-page">
            <h1 className="page-title">Analytics</h1>

            {transactions.length === 0 ? (
              <p className="empty-state">
                No transaction data available for analysis
              </p>
            ) : (
              <div className="analytics-grid">
                {/* Expenses by Category - Pie Chart */}
                <div className="chart-card">
                  <h2 className="section-title">Expenses by Category</h2>
                  {expensesByCategory.length === 0 ? (
                    <p className="empty-state">No expenses recorded</p>
                  ) : (
                    <div className="chart-container">
                      <ResponsiveContainer width="100%" height={300}>
                        <PieChart>
                          <Pie
                            data={expensesByCategory}
                            cx="50%"
                            cy="50%"
                            innerRadius={60}
                            outerRadius={100}
                            paddingAngle={3}
                            dataKey="value"
                            stroke="none"
                          >
                            {expensesByCategory.map((_, index) => (
                              <Cell
                                key={`cell-${index}`}
                                fill={CHART_COLORS[index % CHART_COLORS.length]}
                              />
                            ))}
                          </Pie>
                          <Tooltip
                            contentStyle={{
                              backgroundColor: "#111827",
                              border: "1px solid #8b5cf6",
                              borderRadius: "8px",
                              color: "#e8ecf6",
                            }}
                            formatter={(value: number) => [
                              `$${value.toFixed(2)}`,
                              "Amount",
                            ]}
                          />
                        </PieChart>
                      </ResponsiveContainer>
                      <div className="chart-legend">
                        {expensesByCategory.map((entry, index) => (
                          <div key={entry.name} className="legend-item">
                            <span
                              className="legend-color"
                              style={{
                                backgroundColor:
                                  CHART_COLORS[index % CHART_COLORS.length],
                              }}
                            />
                            <span className="legend-label">{entry.name}</span>
                            <span className="legend-value">
                              ${entry.value.toFixed(2)}
                            </span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>

                {/* Balance Evolution - Area Chart */}
                <div className="chart-card">
                  <h2 className="section-title">Balance Evolution</h2>
                  <div className="chart-container">
                    <ResponsiveContainer width="100%" height={300}>
                      <AreaChart data={balanceEvolution}>
                        <defs>
                          <linearGradient
                            id="balanceGradient"
                            x1="0"
                            y1="0"
                            x2="0"
                            y2="1"
                          >
                            <stop
                              offset="5%"
                              stopColor="#7f8aff"
                              stopOpacity={0.4}
                            />
                            <stop
                              offset="95%"
                              stopColor="#7f8aff"
                              stopOpacity={0}
                            />
                          </linearGradient>
                        </defs>
                        <CartesianGrid
                          stroke="#374151"
                          strokeDasharray="3 3"
                          vertical={false}
                        />
                        <XAxis
                          dataKey="date"
                          stroke="#8c93a8"
                          fontSize={12}
                          tickLine={false}
                          axisLine={{ stroke: "#374151" }}
                        />
                        <YAxis
                          stroke="#8c93a8"
                          fontSize={12}
                          tickLine={false}
                          axisLine={{ stroke: "#374151" }}
                          tickFormatter={(value) => `$${value}`}
                        />
                        <Tooltip
                          contentStyle={{
                            backgroundColor: "#111827",
                            border: "1px solid #7f8aff",
                            borderRadius: "8px",
                            color: "#e8ecf6",
                          }}
                          formatter={(value: number) => [
                            `$${value.toFixed(2)}`,
                            "Balance",
                          ]}
                          labelStyle={{ color: "#c1c7d7" }}
                        />
                        <Area
                          type="monotone"
                          dataKey="balance"
                          stroke="#7f8aff"
                          strokeWidth={2}
                          fill="url(#balanceGradient)"
                        />
                      </AreaChart>
                    </ResponsiveContainer>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </main>

      {/* Delete Confirmation Modal */}
      {transactionToDelete !== null && (
        <div className="modal-overlay" onClick={cancelDelete}>
          <div
            className="modal-card delete-modal"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-header">
              <span className="modal-icon">⚠️</span>
              <h2>Confirm Deletion</h2>
            </div>
            <div className="modal-body">
              <p>Are you sure you want to delete this transaction?</p>
              <p className="modal-warning">This action cannot be undone.</p>
            </div>
            <div className="modal-actions">
              <button
                type="button"
                className="btn-secondary"
                onClick={cancelDelete}
                disabled={isLoading}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                onClick={confirmDelete}
                disabled={isLoading}
              >
                {isLoading ? "Deleting..." : "Delete"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
