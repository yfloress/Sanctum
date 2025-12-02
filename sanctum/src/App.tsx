import { useState, useCallback, useEffect, useRef } from "react";
import "./App.css";

// Hooks
import { useAuth } from "./hooks/useAuth";
import { useTransactions } from "./hooks/useTransactions";
import { useCrypto } from "./hooks/useCrypto";

// Components
import { Sidebar } from "./components/layout/Sidebar";
import { DeleteConfirmModal } from "./components/modals/DeleteConfirmModal";

// Features
import { LoginScreen } from "./features/auth/LoginScreen";
import { Dashboard } from "./features/dashboard/Dashboard";
import { TransactionsView } from "./features/transactions/TransactionsView";
import { AnalyticsView } from "./features/analytics/AnalyticsView";
import { CryptoView } from "./features/crypto/CryptoView";

// Types
import type { TabType } from "./types";

function App() {
  // Tab state
  const [activeTab, setActiveTab] = useState<TabType>("dashboard");

  // Track previous initialization state to detect changes
  const prevInitializedRef = useRef<boolean | null>(null);

  // Initialize auth hook (no callbacks - we'll handle data loading via effects)
  const auth = useAuth({});

  // Initialize transactions hook
  const transactions = useTransactions({
    onSuccess: (message) => auth.setTemporarySuccess(message),
    onError: (message) => auth.setTemporaryError(message),
    clearMessages: () => auth.clearMessages(),
  });

  // Initialize crypto hook
  const crypto = useCrypto({
    onSuccess: (message) => auth.setTemporarySuccess(message),
  });

  // Effect to load data when vault becomes initialized
  useEffect(() => {
    const wasInitialized = prevInitializedRef.current;
    const isInitialized = auth.isInitialized;

    // Update ref for next render
    prevInitializedRef.current = isInitialized;

    // Skip on first render (wasInitialized is null)
    if (wasInitialized === null) {
      // First render - if already initialized, load data
      if (isInitialized) {
        const loadInitialData = async () => {
          try {
            await transactions.loadTransactions();
            await transactions.loadBalance();
            await crypto.loadHoldings();
            await crypto.loadWallets();
            await crypto.loadAggregatedPortfolio();
          } catch (err) {
            console.error("Error loading initial data:", err);
          }
        };
        loadInitialData();
      }
      return;
    }

    // Vault just opened (was false, now true)
    if (!wasInitialized && isInitialized) {
      const loadData = async () => {
        try {
          await transactions.loadTransactions();
          await transactions.loadBalance();
          await crypto.loadHoldings();
          await crypto.loadWallets();
          await crypto.loadAggregatedPortfolio();
        } catch (err) {
          console.error("Error loading data:", err);
        }
      };
      loadData();
    }

    // Vault just closed (was true, now false)
    if (wasInitialized && !isInitialized) {
      transactions.resetState();
      crypto.resetState();
    }
    // Only depend on isInitialized to prevent loops
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth.isInitialized]);

  // Handle vault action
  const handleVaultAction = useCallback(
    async (action: "open" | "create") => {
      await auth.handleVaultAction(action);
      // Data will be loaded via the effect when isInitialized becomes true
    },
    [auth],
  );

  // Handle close vault
  const handleCloseVault = useCallback(async () => {
    await auth.handleCloseVault();
    // State will be reset via the effect when isInitialized becomes false
  }, [auth]);

  // Handle crypto tab click - load prices if needed
  const handleCryptoTabClick = useCallback(() => {
    if (crypto.cryptoAssets.length === 0 && !crypto.cryptoLoading) {
      crypto.loadCryptoPrices();
    }
  }, [
    crypto.cryptoAssets.length,
    crypto.cryptoLoading,
    crypto.loadCryptoPrices,
  ]);

  // ==================== Render: Loading State ====================
  if (auth.isLoading && !auth.isInitialized) {
    return (
      <div className="vault-container">
        <div className="vault-card">
          <div className="loader" />
          <p>Checking vault status...</p>
        </div>
      </div>
    );
  }

  // ==================== Render: Login Screen ====================
  if (!auth.isInitialized) {
    return (
      <LoginScreen
        password={auth.password}
        setPassword={auth.setPassword}
        showPassword={auth.showPassword}
        setShowPassword={auth.setShowPassword}
        dbPathInput={auth.dbPathInput}
        setDbPathInput={auth.setDbPathInput}
        isLoading={auth.isLoading}
        loadingAction={auth.loadingAction}
        error={auth.error}
        onVaultAction={handleVaultAction}
      />
    );
  }

  // ==================== Render: Main Application ====================
  return (
    <div className="app-layout">
      <Sidebar
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        onLockVault={handleCloseVault}
        isLoading={auth.isLoading}
        onCryptoTabClick={handleCryptoTabClick}
      />

      <main className="content-area">
        {/* Global Messages */}
        {auth.error && <div className="message error">{auth.error}</div>}
        {auth.successMessage && (
          <div className="message success">{auth.successMessage}</div>
        )}

        {/* ==================== Dashboard Tab ==================== */}
        {activeTab === "dashboard" && (
          <Dashboard
            balance={transactions.balance}
            transactions={transactions.transactions}
            onDeleteTransaction={transactions.handleDeleteTransaction}
            isLoading={auth.isLoading}
          />
        )}

        {/* ==================== Transactions Tab ==================== */}
        {activeTab === "transactions" && (
          <TransactionsView
            amount={transactions.amount}
            setAmount={transactions.setAmount}
            description={transactions.description}
            setDescription={transactions.setDescription}
            category={transactions.category}
            setCategory={transactions.setCategory}
            date={transactions.date}
            setDate={transactions.setDate}
            isExpense={transactions.isExpense}
            categories={transactions.categories}
            onExpenseToggle={transactions.handleExpenseToggle}
            onAddTransaction={transactions.handleAddTransaction}
            transactions={transactions.transactions}
            onDeleteTransaction={transactions.handleDeleteTransaction}
            isLoading={auth.isLoading}
          />
        )}

        {/* ==================== Analytics Tab ==================== */}
        {activeTab === "analytics" && (
          <AnalyticsView
            expensesByCategory={transactions.expensesByCategory}
            balanceEvolution={transactions.balanceEvolution}
            hasTransactions={transactions.transactions.length > 0}
          />
        )}

        {/* ==================== Crypto Tab ==================== */}
        {activeTab === "crypto" && <CryptoView crypto={crypto} />}

        {/* ==================== Delete Transaction Modal ==================== */}
        <DeleteConfirmModal
          isOpen={transactions.transactionToDelete !== null}
          onClose={transactions.cancelDelete}
          onConfirm={transactions.confirmDelete}
          isLoading={auth.isLoading}
          title="Confirm Deletion"
          message="Are you sure you want to delete this transaction?"
        />
      </main>
    </div>
  );
}

export default App;
