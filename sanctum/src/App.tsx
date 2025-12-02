/**
 * App Component
 *
 * Main application entry point.
 * Uses Zustand stores exclusively for state management.
 *
 * ARCHITECTURE:
 * - Auth state: useAuthStore (vault open/close, session)
 * - Financial data: useFinancialStore (transactions, balance)
 * - Crypto data: useCryptoStore (wallets, portfolio, prices)
 * - Habits data: useHabitStore (habits, logs, streaks)
 *
 * SECURITY:
 * - All sensitive data lives only in RAM (no localStorage)
 * - Stores are cleared when vault is closed (kill switch in authStore)
 */

import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// Stores
import {
  useAuthStore,
  useIsInitialized,
  useAuthLoading,
  useAuthError,
  useAuthSuccess,
  useFinancialStore,
  useCryptoStore,
  useHabitStore,
} from "./stores";

// Components
import { Sidebar } from "./components/layout/Sidebar";
import { DeleteConfirmModal } from "./components/modals/DeleteConfirmModal";

// Features
import { LoginScreen } from "./features/auth/LoginScreen";
import { Dashboard } from "./features/dashboard/Dashboard";
import { TransactionsView } from "./features/transactions/TransactionsView";
import { AnalyticsView } from "./features/analytics/AnalyticsView";
import { CryptoView } from "./features/crypto/CryptoView";
import { HabitsView } from "./features/habits/HabitsView";

// Types
import type { TabType } from "./types";

// Session timeout warning threshold (2 minutes before expiry)
const SESSION_WARNING_THRESHOLD = 120;

function App() {
  // ==================== Auth Store ====================
  const isInitialized = useIsInitialized();
  const authLoading = useAuthLoading();
  const authError = useAuthError();
  const authSuccess = useAuthSuccess();
  const checkStatus = useAuthStore((state) => state.checkStatus);
  const logout = useAuthStore((state) => state.logout);
  const setAuthError = useAuthStore((state) => state.setError);

  // ==================== Financial Store ====================
  const financialError = useFinancialStore((state) => state.error);
  const financialSuccess = useFinancialStore((state) => state.successMessage);
  const financialLoading = useFinancialStore((state) => state.isLoading);
  const transactionToDelete = useFinancialStore(
    (state) => state.transactionToDelete,
  );
  const confirmDeleteTransaction = useFinancialStore(
    (state) => state.confirmDelete,
  );
  const cancelDeleteTransaction = useFinancialStore(
    (state) => state.cancelDelete,
  );

  // ==================== Crypto Store ====================
  const cryptoError = useCryptoStore((state) => state.error);
  const cryptoSuccess = useCryptoStore((state) => state.successMessage);
  const cryptoLoading = useCryptoStore((state) => state.isLoading);
  const cryptoPrices = useCryptoStore((state) => state.prices);
  const fetchPrices = useCryptoStore((state) => state.fetchPrices);

  // ==================== Habit Store ====================
  const habitError = useHabitStore((state) => state.error);
  const habitSuccess = useHabitStore((state) => state.successMessage);
  const habitLoading = useHabitStore((state) => state.isLoading);

  // ==================== Local UI State ====================
  const [activeTab, setActiveTab] = useState<TabType>("dashboard");
  const [sessionRemaining, setSessionRemaining] = useState<number | null>(null);
  const [showSessionWarning, setShowSessionWarning] = useState(false);
  const sessionCheckIntervalRef = useRef<ReturnType<typeof setInterval> | null>(
    null,
  );

  // ==================== Check Auth Status on Mount ====================
  useEffect(() => {
    checkStatus();
  }, [checkStatus]);

  // ==================== Reset UI State on Logout ====================
  useEffect(() => {
    if (!isInitialized) {
      setActiveTab("dashboard");
      setSessionRemaining(null);
      setShowSessionWarning(false);
    }
  }, [isInitialized]);

  // ==================== Session Monitoring ====================
  useEffect(() => {
    if (!isInitialized) {
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
          setAuthError("Session expired due to inactivity");
          await logout();
        }
      } catch (err) {
        const errorStr = String(err);
        if (
          errorStr.includes("Session expired") ||
          errorStr.includes("inactivity")
        ) {
          setAuthError("Session expired due to inactivity");
          await logout();
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
  }, [isInitialized, logout, setAuthError]);

  // ==================== Handlers ====================
  const handleCryptoTabClick = useCallback(() => {
    if (cryptoPrices.length === 0 && !cryptoLoading) {
      fetchPrices();
    }
  }, [cryptoPrices.length, cryptoLoading, fetchPrices]);

  // ==================== Computed Values ====================
  const isLoading =
    authLoading || financialLoading || cryptoLoading || habitLoading;
  const errorMessage = authError || financialError || cryptoError || habitError;
  const successMessage =
    authSuccess || financialSuccess || cryptoSuccess || habitSuccess;

  // ==================== Render: Loading State ====================
  if (authLoading && !isInitialized) {
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
  if (!isInitialized) {
    return <LoginScreen />;
  }

  // ==================== Render: Main Application ====================
  return (
    <div className="app-layout">
      <Sidebar
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        onLockVault={logout}
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

        {/* ==================== Habits Tab ==================== */}
        {activeTab === "habits" && <HabitsView />}

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
