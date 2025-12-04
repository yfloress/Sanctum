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

import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// Stores
import {
  useAuthError,
  useAuthLoading,
  useAuthStore,
  useAuthSuccess,
  useCryptoStore,
  useFinancialStore,
  useHabitStore,
  useIsInitialized,
} from "./stores/index.ts";

// Components
import { Sidebar } from "./components/layout/Sidebar.tsx";
import { DeleteConfirmModal } from "./components/modals/DeleteConfirmModal.tsx";
import { ToastStack } from "./components/ui/Toast.tsx";

// Toast Store
import { useToast } from "./stores/toastStore.ts";

// Features
import { LoginScreen } from "./features/auth/LoginScreen.tsx";
import { Dashboard } from "./features/dashboard/Dashboard.tsx";
import { TransactionsView } from "./features/transactions/TransactionsView.tsx";
import { AnalyticsView } from "./features/analytics/AnalyticsView.tsx";
import { CryptoView } from "./features/crypto/CryptoView.tsx";
import { HabitsView } from "./features/habits/HabitsView.tsx";

// Types
import type { TabType } from "./types/index.ts";

// Session timeout warning threshold (2 minutes before expiry)
const SESSION_WARNING_THRESHOLD = 120;

interface AppProps {
  onReady?: () => void;
}

function App({ onReady }: AppProps) {
  // ==================== Auth Store ====================
  const isInitialized = useIsInitialized();
  const authLoading = useAuthLoading();
  const authError = useAuthError();
  const authSuccess = useAuthSuccess();
  const checkStatus = useAuthStore((state) => state.checkStatus);
  const logout = useAuthStore((state) => state.logout);
  const clearAuthMessages = useAuthStore((state) => state.clearMessages);

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
  const clearFinancialMessages = useFinancialStore(
    (state) => state.clearMessages,
  );

  // ==================== Crypto Store ====================
  const cryptoError = useCryptoStore((state) => state.error);
  const cryptoSuccess = useCryptoStore((state) => state.successMessage);
  const cryptoLoading = useCryptoStore((state) => state.isLoading);
  const cryptoPrices = useCryptoStore((state) => state.prices);
  const fetchPrices = useCryptoStore((state) => state.fetchPrices);
  const clearCryptoMessages = useCryptoStore((state) => state.clearMessages);

  // ==================== Habit Store ====================
  const habitError = useHabitStore((state) => state.error);
  const habitSuccess = useHabitStore((state) => state.successMessage);
  const habitLoading = useHabitStore((state) => state.isLoading);
  const clearHabitMessages = useHabitStore((state) => state.clearMessages);

  // ==================== Toast System ====================
  const toasts = useToast((state) => state.toasts);
  const removeToast = useToast((state) => state.removeToast);
  const toast = useToast();

  // ==================== Local UI State ====================
  const [activeTab, setActiveTab] = useState<TabType>("dashboard");
  const [sessionRemaining, setSessionRemaining] = useState<number | null>(null);
  const [showSessionWarning, setShowSessionWarning] = useState(false);
  const sessionCheckIntervalRef = useRef<ReturnType<typeof setInterval> | null>(
    null,
  );

  // ==================== Convert Store Messages to Toasts ====================
  useEffect(() => {
    if (authError) {
      toast.error(authError);
      clearAuthMessages();
    }
    if (authSuccess) {
      toast.success(authSuccess);
      clearAuthMessages();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [authError, authSuccess]);

  useEffect(() => {
    if (financialError) {
      toast.error(financialError);
      clearFinancialMessages();
    }
    if (financialSuccess) {
      toast.success(financialSuccess);
      clearFinancialMessages();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [financialError, financialSuccess]);

  useEffect(() => {
    if (cryptoError) {
      toast.error(cryptoError);
      clearCryptoMessages();
    }
    if (cryptoSuccess) {
      toast.success(cryptoSuccess);
      clearCryptoMessages();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cryptoError, cryptoSuccess]);

  useEffect(() => {
    if (habitError) {
      toast.error(habitError);
      clearHabitMessages();
    }
    if (habitSuccess) {
      toast.success(habitSuccess);
      clearHabitMessages();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [habitError, habitSuccess]);

  // ==================== Check Auth Status on Mount ====================
  useEffect(() => {
    checkStatus();
  }, [checkStatus]);

  // ==================== Show Window When Ready ====================
  useLayoutEffect(() => {
    // Call onReady after first paint to show the window
    if (onReady) {
      onReady();
    }
  }, [onReady]);

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
          toast.warning("Session expired due to inactivity");
          await logout();
        }
      } catch (err) {
        const errorStr = String(err);
        if (
          errorStr.includes("Session expired") ||
          errorStr.includes("inactivity")
        ) {
          toast.warning("Session expired due to inactivity");
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
  }, [isInitialized, logout, toast]);

  // ==================== Session Warning Toast ====================
  useEffect(() => {
    if (showSessionWarning && sessionRemaining !== null) {
      toast.warning(
        `Session expires in ${Math.ceil(
          sessionRemaining / 60,
        )} minute(s). Activity will extend your session.`,
      );
    }
  }, [showSessionWarning]);

  // ==================== Handlers ====================
  const handleCryptoTabClick = useCallback(() => {
    if (cryptoPrices.length === 0 && !cryptoLoading) {
      fetchPrices();
    }
  }, [cryptoPrices.length, cryptoLoading, fetchPrices]);

  // ==================== Computed Values ====================
  const isLoading =
    authLoading || financialLoading || cryptoLoading || habitLoading;

  // ==================== Render: Loading State ====================
  if (authLoading && !isInitialized) {
    return (
      <>
        <ToastStack toasts={toasts} onRemove={removeToast} />
        <div className="vault-container">
          <div className="vault-card">
            <div className="loader" />
            <p>Checking vault status...</p>
          </div>
        </div>
      </>
    );
  }

  // ==================== Render: Login Screen ====================
  if (!isInitialized) {
    return (
      <>
        <ToastStack toasts={toasts} onRemove={removeToast} />
        <LoginScreen />
      </>
    );
  }

  // ==================== Render: Main Application ====================
  return (
    <div className="app-layout">
      <ToastStack toasts={toasts} onRemove={removeToast} />

      <Sidebar
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        onLockVault={logout}
        isLoading={isLoading}
        onCryptoTabClick={handleCryptoTabClick}
      />

      <main className="content-area">
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
