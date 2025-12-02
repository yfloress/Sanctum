/**
 * Stores Index
 *
 * Central export for all Zustand stores.
 * Import stores from here for cleaner imports throughout the app.
 */

// Auth Store
export {
  useAuthStore,
  useIsInitialized,
  useAuthLoading,
  useAuthError,
  useAuthSuccess,
  useDbPath,
  useLoadingAction,
  type AuthStore,
} from "./authStore";

// Financial Store
export {
  useFinancialStore,
  useTransactions,
  useBalance,
  useFinancialLoading,
  useFinancialError,
  useFinancialSuccess,
  useTransactionForm,
  useTransactionToDelete,
  type FinancialStore,
} from "./financialStore";

// Crypto Store
export {
  useCryptoStore,
  useCryptoPrices,
  useWatchlist,
  useWallets,
  useSelectedWallet,
  usePortfolio,
  useCryptoLoading,
  useCryptoError,
  useCryptoSuccess,
  useCryptoSubTab,
  type CryptoStore,
} from "./cryptoStore";
