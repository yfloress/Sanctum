/**
 * Stores Index
 *
 * Central export for all Zustand stores.
 * Import stores from here for cleaner imports throughout the app.
 */

// Auth Store
export {
  type AuthStore,
  useAuthError,
  useAuthLoading,
  useAuthStore,
  useAuthSuccess,
  useDbPath,
  useIsInitialized,
  useLoadingAction,
  useVaultExists,
} from "./authStore.ts";

// Financial Store
export {
  type FinancialStore,
  useBalance,
  useFinancialError,
  useFinancialLoading,
  useFinancialStore,
  useFinancialSuccess,
  useTransactionForm,
  useTransactions,
  useTransactionToDelete,
} from "./financialStore.ts";

// Crypto Store
export {
  type CryptoStore,
  useCryptoError,
  useCryptoLoading,
  useCryptoPrices,
  useCryptoStore,
  useCryptoSubTab,
  useCryptoSuccess,
  usePortfolio,
  useSelectedWallet,
  useWallets,
  useWatchlist,
} from "./cryptoStore.ts";

// Habit Store
export {
  type HabitStore,
  useCurrentMonth,
  useHabitError,
  useHabitForm,
  useHabitLoading,
  useHabitLogs,
  useHabits,
  useHabitStats,
  useHabitStore,
  useHabitSuccess,
  useHabitToDelete,
  useHabitToEdit,
  useShowAddModal,
} from "./habitStore.ts";

// Account Store
export {
  ACCOUNT_COLORS,
  ACCOUNT_TYPES,
  type Account,
  type AccountBalance,
  type AccountFormData,
  type AccountStore,
  useAccountBalances,
  useAccountError,
  useAccountForm,
  useAccountLoading,
  useAccounts,
  useAccountStore,
  useAccountSuccess,
  useAccountToEdit,
} from "./accountStore.ts";

// Toast Store
export { type ToastItem, useToast } from "./toastStore.ts";
