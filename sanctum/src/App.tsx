/**
 * App Component
 *
 * Main application entry point.
 * Uses Zustand stores for state management (no prop drilling).
 *
 * ARCHITECTURE:
 * - Auth state: managed by useAuth hook (controls vault open/close)
 * - Financial data: managed by useFinancialStore (Zustand)
 * - Crypto data: managed by useCryptoStore (Zustand)
 *
 * SECURITY:
 * - All sensitive data lives only in RAM (no localStorage)
 * - Stores are cleared when vault is closed (kill switch in useAuth)
 */

import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// Hooks
import { useAuth } from "./hooks/useAuth";

// Stores
import { useFinancialStore } from "./stores/financialStore";
import { useCryptoStore } from "./stores/cryptoStore";

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

// Session timeout warning threshold (2 minutes before expiry)
const SESSION_WARNING_THRESHOLD = 120;

function App() {
  // ==================== Local UI State ====================
  const [activeTab, setActiveTab] = useState<TabType>("dashboard");
  const [sessionRemaining, setSessionRemaining] = useState<number | null>(null);
  const [showSessionWarning, setShowSessionWarning] = useState(false);
  const sessionCheckIntervalRef = useRef<ReturnType<typeof setInterval> | null>(
    null,
  );
  const prevInitializedRef = useRef<boolean | null>(null);

  // ==================== Auth Hook ====================
  const auth = useAuth({});

  // ==================== Store Actions ====================
  // Financial store
  const loadFinancialData = useFinancialStore((state) => state.loadData);
  const financialError = useFinancialStore((state) => state.error);
  const financialSuccess = useFinancialStore((state) => state.successMessage);
  const transactionToDelete = useFinancialStore(
    (state) => state.transactionToDelete,
  );
  const confirmDeleteTransaction = useFinancialStore(
    (state) => state.confirmDelete,
  );
  const cancelDeleteTransaction = useFinancialStore(
    (state) => state.cancelDelete,
  );
  const financialLoading = useFinancialStore((state) => state.isLoading);

  // Crypto store
  const loadCryptoData = useCryptoStore((state) => state.loadAll);
  const fetchPrices = useCryptoStore((state) => state.fetchPrices);
  const cryptoPrices = useCryptoStore((state) => state.prices);
  const cryptoLoading = useCryptoStore((state) => state.isLoading);
  const cryptoError = useCryptoStore((state) => state.error);
  const cryptoSuccess = useCryptoStore((state) => state.successMessage);

  // ==================== Data Loading Effect ====================
  useEffect(() => {
    const wasInitialized = prevInitializedRef.current;
    const isInitialized = auth.isInitialized;

    prevInitializedRef.current = isInitialized;

    // First render - if already initialized, load data
    if (wasInitialized === null && isInitialized) {
      const loadInitialData = async () => {
        try {
          await Promise.all([loadFinancialData(), loadCryptoData()]);
        } catch (err) {
          console.error("Error loading initial data:", err);
        }
      };
      loadInitialData();
      return;
    }

    // Vault just opened
    if (!wasInitialized && isInitialized) {
      const loadData = async () => {
        try {
          await Promise.all([loadFinancialData(), loadCryptoData()]);
        } catch (err) {
          console.error("Error loading data:", err);
        }
      };
      loadData();
    }

    // Vault just closed - stores are already cleared by useAuth kill switch
    if (wasInitialized && !isInitialized) {
      setSessionRemaining(null);
      setShowSessionWarning(false);
      setActiveTab("dashboard");
    }
  }, [auth.isInitialized, loadFinancialData, loadCryptoData]);

  // ==================== Session Monitoring ====================
  useEffect(() => {
    if (!auth.isInitialized) {
      if (sessionCheckIntervalRef.current) {
        clearInterval(sessionCheckIntervalRef.current);
        sessionCheckIntervalRef.current = null;
      }
      return;
    }

    const checkSession = async () => {
      try {
        const remaining = await invoke<number>("get_session_remaining");
        setSessionRemaining(remaining);

        if (remaining <= SESSION_WARNING_THRESHOLD && remaining > 0) {
          setShowSessionWarning(true);
        } else {
          setShowSessionWarning(false);
        }

        if (remaining <= 0) {
          auth.setTemporaryError("Session expired due to inactivity");
          await auth.handleCloseVault();
        }
      } catch (err) {
        const errorStr = String(err);
        if (
          errorStr.includes("Session expired") ||
          errorStr.includes("inactivity")
        ) {
          auth.setTemporaryError("Session expired due to inactivity");
          await auth.handleCloseVault();
        }
      }
    };

    checkSession();
    sessionCheckIntervalRef.current = setInterval(checkSession, 30000);

    return () => {
      if (sessionCheckIntervalRef.current) {
        clearInterval(sessionCheckIntervalRef.current);
        sessionCheckIntervalRef.current = null;
      }
    };
  }, [auth.isInitialized, auth.handleCloseVault, auth.setTemporaryError]);

  // ==================== Handlers ====================
  const handleVaultAction = useCallback(
    async (action: "open" | "create") => {
      await auth.handleVaultAction(action);
    },
    [auth],
  );

  const handleCloseVault = useCallback(async () => {
    await auth.handleCloseVault();
  }, [auth]);

  const handleCryptoTabClick = useCallback(() => {
    if (cryptoPrices.length === 0 && !cryptoLoading) {
      fetchPrices();
    }
  }, [cryptoPrices.length, cryptoLoading, fetchPrices]);

  // ==================== Computed Values ====================
  const isLoading = auth.isLoading || financialLoading || cryptoLoading;
  const errorMessage = auth.error || financialError || cryptoError;
  const successMessage =
    auth.successMessage || financialSuccess || cryptoSuccess;

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
        isLoading={isLoading}
        onCryptoTabClick={handleCryptoTabClick}
      />

      <main className="content-area">
        {/* Session Warning */}
        {showSessionWarning && sessionRemaining !== null && (
          <div className="message warning">
            ⚠️ Session expires in {Math.ceil(sessionRemaining / 60)} minute(s).
            Activity will extend your session.
          </div>
        )}

        {/* Global Messages */}
        {errorMessage && <div className="message error">{errorMessage}</div>}
        {successMessage && (
          <div className="message success">{successMessage}</div>
        )}

        {/* ==================== Dashboard Tab ==================== */}
        {activeTab === "dashboard" && <Dashboard />}

        {/* ==================== Transactions Tab ==================== */}
        {activeTab === "transactions" && <TransactionsView />}

        {/* ==================== Analytics Tab ==================== */}
        {activeTab === "analytics" && <AnalyticsView />}

        {/* ==================== Crypto Tab ==================== */}
        {activeTab === "crypto" && <CryptoView />}

        {/* ==================== Delete Transaction Modal ==================== */}
        <DeleteConfirmModal
          isOpen={transactionToDelete !== null}
          onClose={cancelDeleteTransaction}
          onConfirm={confirmDeleteTransaction}
          isLoading={financialLoading}
          title="Confirm Deletion"
          message="Are you sure you want to delete this transaction?"
        />
      </main>
    </div>
  );
}

export default App;
