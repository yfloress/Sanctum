import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Transaction {
  id: string;
  amount: number;
  category: string;
  description: string;
  date: string;
  type: string;
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
  const [dbPath, setDbPath] = useState("");
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
      setDbPath(path);
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

  const checkDatabaseStatus = useCallback(async () => {
    try {
      setIsLoading(true);
      setError("");
      const initialized = await invoke<boolean>("is_db_initialized");
      setIsInitialized(initialized);
      await loadDbPath();
      if (initialized) {
        await loadTransactions();
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
        const result = await invoke<string>(command, {
          password: trimmedPassword,
          path: targetPath,
        });
        setIsInitialized(true);
        setPassword("");
        await loadDbPath();
        await loadTransactions();
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
    <div className="vault-container">
      <div className="vault-card open">
        <div className="vault-header">
          <div className="vault-icon unlocked">🔓</div>
          <h1>Vault Unlocked</h1>
          <p className="vault-subtitle">Your data is accessible</p>
        </div>

        <div className="vault-content-grid">
          <div className="vault-column-left">
            <div className="info-section">
              <h3>New Transaction</h3>
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
          </div>

          <div className="vault-column-right">
            <div className="info-section">
              <h3>Transaction History</h3>
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
                      <div
                        className={`transaction-amount ${tx.type === "income" ? "income" : "expense"}`}
                      >
                        {tx.type === "income" ? "+" : "-"}$
                        {formatAmount(tx.amount)}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="info-section">
              <h3>Connection Status</h3>
              <div className="status-badge active">Active</div>
            </div>

            <div className="info-section">
              <h3>Database Location</h3>
              <code className="db-path">{dbPath || "Loading..."}</code>
            </div>
          </div>
        </div>

        {error && <div className="message error">{error}</div>}
        {successMessage && (
          <div className="message success">{successMessage}</div>
        )}

        <button
          onClick={handleCloseVault}
          className="btn-close"
          disabled={isLoading}
        >
          {isLoading ? "Locking..." : "Lock Vault"}
        </button>

        <div className="vault-footer">
          <p>Remember to lock the vault when you're done</p>
        </div>
      </div>
    </div>
  );
}

export default App;
