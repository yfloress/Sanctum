/**
 * Account Store - Zustand State Management
 *
 * Manages FIAT accounts (bank, cash, savings, etc.)
 *
 * SECURITY: This store lives in RAM only. NO persistence middleware.
 * The real persistence is handled by Rust (SQLCipher encrypted database).
 *
 * CURRENCY HANDLING:
 * - Accounts can be in USD or CLP
 * - Net worth is always calculated in USD (base currency)
 * - CLP accounts are converted using CoinGecko exchange rate
 * - Exchange rate is fetched on demand and cached
 */

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

// ==================== Types ====================

export interface Account {
  id: string;
  name: string;
  type: string; // "bank", "cash", "savings", "credit_card", "other"
  currency: string; // "USD" | "CLP"
  initial_balance: number; // In cents
  color: string;
  icon: string | null;
  is_archived: boolean;
  created_at: string;
}

export interface AccountBalance {
  account_id: string;
  account_name: string;
  current_balance: number; // In cents (in account's currency)
  total_income: number;
  total_expense: number;
}

export interface AccountFormData {
  name: string;
  type: string;
  currency: string;
  initial_balance: string; // String for form input
  color: string;
  icon: string | null;
}

interface AccountState {
  // Data State
  accounts: Account[];
  balances: AccountBalance[];

  // Exchange Rate State
  clpToUsdRate: number | null; // How many CLP per 1 USD (e.g., ~950)
  rateLastUpdated: number | null; // Timestamp
  rateLoading: boolean;

  // UI State
  isLoading: boolean;
  error: string | null;
  successMessage: string | null;

  // Form State
  form: AccountFormData;
  accountToEdit: Account | null;
}

interface AccountActions {
  // Data Loading
  loadAccounts: () => Promise<void>;
  loadBalances: () => Promise<void>;
  loadAll: () => Promise<void>;

  // Exchange Rate
  fetchExchangeRate: () => Promise<void>;

  // CRUD Operations
  createAccount: (data: AccountFormData) => Promise<boolean>;
  updateAccount: (id: string, data: AccountFormData) => Promise<boolean>;
  archiveAccount: (id: string) => Promise<boolean>;

  // Transfers
  transfer: (
    fromAccountId: string,
    toAccountId: string,
    amount: number,
    description: string,
    date: string,
  ) => Promise<boolean>;

  // Form Management
  setFormField: <K extends keyof AccountFormData>(
    field: K,
    value: AccountFormData[K],
  ) => void;
  resetForm: () => void;
  setAccountToEdit: (account: Account | null) => void;
  populateFormFromAccount: (account: Account) => void;

  // Computed Getters (all return values in USD cents)
  getTotalNetWorthUSD: () => number;
  getTotalIncomeUSD: () => number;
  getTotalExpensesUSD: () => number;
  getAccountById: (id: string) => Account | undefined;
  getAccountBalance: (id: string) => AccountBalance | undefined;
  convertToUSD: (amountCents: number, currency: string) => number;

  // Messages
  setError: (error: string | null) => void;
  setSuccess: (message: string | null) => void;
  clearMessages: () => void;

  // Security: RAM Clear
  reset: () => void;
}

export type AccountStore = AccountState & AccountActions;

// ==================== Constants ====================

export const ACCOUNT_TYPES = [
  { value: "bank", label: "Bank Account", icon: "🏦" },
  { value: "cash", label: "Cash", icon: "💵" },
  { value: "savings", label: "Savings", icon: "🐷" },
  { value: "credit_card", label: "Credit Card", icon: "💳" },
  { value: "other", label: "Other", icon: "💰" },
] as const;

export const ACCOUNT_COLORS = [
  "#8b5cf6", // violet
  "#10b981", // emerald
  "#f59e0b", // amber
  "#ef4444", // red
  "#06b6d4", // cyan
  "#ec4899", // pink
  "#6366f1", // indigo
  "#84cc16", // lime
  "#f97316", // orange
  "#14b8a6", // teal
] as const;

export const DEFAULT_CURRENCY = "USD";

// Rate cache duration: 5 minutes
const RATE_CACHE_DURATION_MS = 5 * 60 * 1000;

// ==================== Initial State ====================

const initialFormState: AccountFormData = {
  name: "",
  type: "bank",
  currency: DEFAULT_CURRENCY,
  initial_balance: "0",
  color: ACCOUNT_COLORS[0],
  icon: "🏦",
};

const initialState: AccountState = {
  accounts: [],
  balances: [],
  clpToUsdRate: null,
  rateLastUpdated: null,
  rateLoading: false,
  isLoading: false,
  error: null,
  successMessage: null,
  form: { ...initialFormState },
  accountToEdit: null,
};

// ==================== Store ====================

export const useAccountStore = create<AccountStore>((set, get) => ({
  ...initialState,

  // ==================== Data Loading ====================

  loadAccounts: async () => {
    try {
      const accounts = await invoke<Account[]>("get_accounts");
      set({ accounts });
    } catch (err) {
      console.error("Error loading accounts:", err);
      throw err;
    }
  },

  loadBalances: async () => {
    try {
      const balances = await invoke<AccountBalance[]>("get_account_balances");
      set({ balances });
    } catch (err) {
      console.error("Error loading account balances:", err);
      throw err;
    }
  },

  loadAll: async () => {
    set({ isLoading: true, error: null });
    try {
      await Promise.all([get().loadAccounts(), get().loadBalances()]);

      // Fetch exchange rate if we have any CLP accounts
      const { accounts } = get();
      const hasClpAccounts = accounts.some((acc) => acc.currency === "CLP");
      if (hasClpAccounts) {
        // First try to load cached rate for immediate display
        try {
          const cached = await invoke<[number, string] | null>(
            "load_exchange_rate",
            { pair: "CLP_USD" },
          );
          if (cached) {
            set({ clpToUsdRate: cached[0] });
          }
        } catch {
          // Ignore cache errors
        }

        // Then fetch fresh rate in background
        get().fetchExchangeRate();
      }
    } catch (err) {
      set({ error: `Error loading accounts: ${err}` });
    } finally {
      set({ isLoading: false });
    }
  },

  // ==================== Exchange Rate ====================

  fetchExchangeRate: async () => {
    const { rateLastUpdated, rateLoading, clpToUsdRate } = get();

    // Check cache validity
    if (
      rateLastUpdated &&
      Date.now() - rateLastUpdated < RATE_CACHE_DURATION_MS
    ) {
      return; // Use cached rate
    }

    // Prevent concurrent fetches
    if (rateLoading) return;

    set({ rateLoading: true });

    try {
      // Try to fetch fresh rate from API
      const rate = await invoke<number>("get_clp_usd_rate");
      set({
        clpToUsdRate: rate,
        rateLastUpdated: Date.now(),
      });

      // Save to persistent cache for offline use
      try {
        await invoke("save_exchange_rate", { pair: "CLP_USD", rate });
      } catch {
        // Ignore cache save errors
      }
    } catch (err) {
      console.error("Error fetching CLP/USD rate:", err);

      // If we don't have a rate yet, try to load from persistent cache
      if (!clpToUsdRate) {
        try {
          const cached = await invoke<[number, string] | null>(
            "load_exchange_rate",
            { pair: "CLP_USD" },
          );
          if (cached) {
            set({ clpToUsdRate: cached[0] });
          }
        } catch {
          // Ignore cache load errors
        }
      }
    } finally {
      set({ rateLoading: false });
    }
  },

  // ==================== CRUD Operations ====================

  createAccount: async (data: AccountFormData) => {
    // Validation
    if (!data.name.trim()) {
      set({ error: "Account name cannot be empty" });
      return false;
    }

    const initialBalanceCents = Math.round(
      parseFloat(data.initial_balance || "0") * 100,
    );

    set({ isLoading: true, error: null });

    try {
      await invoke<string>("create_account", {
        name: data.name.trim(),
        accountType: data.type,
        currency: data.currency,
        initialBalance: initialBalanceCents,
        color: data.color,
        icon: data.icon || null,
      });

      await get().loadAll();
      get().resetForm();

      set({ successMessage: "Account created successfully" });
      setTimeout(() => set({ successMessage: null }), 3000);

      return true;
    } catch (err) {
      set({ error: `Error creating account: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  updateAccount: async (id: string, data: AccountFormData) => {
    if (!data.name.trim()) {
      set({ error: "Account name cannot be empty" });
      return false;
    }

    const initialBalanceCents = Math.round(
      parseFloat(data.initial_balance || "0") * 100,
    );

    set({ isLoading: true, error: null });

    try {
      await invoke("update_account", {
        id,
        name: data.name.trim(),
        accountType: data.type,
        currency: data.currency,
        initialBalance: initialBalanceCents,
        color: data.color,
        icon: data.icon || null,
      });

      await get().loadAll();
      get().resetForm();
      set({
        accountToEdit: null,
        successMessage: "Account updated successfully",
      });
      setTimeout(() => set({ successMessage: null }), 3000);

      return true;
    } catch (err) {
      set({ error: `Error updating account: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  archiveAccount: async (id: string) => {
    set({ isLoading: true, error: null });

    try {
      await invoke("archive_account", { id });
      await get().loadAll();

      set({ successMessage: "Account archived successfully" });
      setTimeout(() => set({ successMessage: null }), 3000);

      return true;
    } catch (err) {
      set({ error: `${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  // ==================== Transfers ====================

  transfer: async (
    fromAccountId: string,
    toAccountId: string,
    amount: number,
    description: string,
    date: string,
  ) => {
    if (amount <= 0) {
      set({ error: "Transfer amount must be greater than zero" });
      return false;
    }

    if (fromAccountId === toAccountId) {
      set({ error: "Cannot transfer to the same account" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      const amountCents = Math.round(amount * 100);

      await invoke<string>("transfer_funds", {
        fromAccountId,
        toAccountId,
        amount: amountCents,
        description: description.trim(),
        date,
      });

      await get().loadBalances();

      set({ successMessage: "Transfer completed successfully" });
      setTimeout(() => set({ successMessage: null }), 3000);

      return true;
    } catch (err) {
      set({ error: `Error transferring funds: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  // ==================== Form Management ====================

  setFormField: (field, value) => {
    set((state) => ({
      form: { ...state.form, [field]: value },
    }));
  },

  resetForm: () => {
    set({ form: { ...initialFormState }, accountToEdit: null });
  },

  setAccountToEdit: (account: Account | null) => {
    set({ accountToEdit: account });
    if (account) {
      get().populateFormFromAccount(account);
    } else {
      get().resetForm();
    }
  },

  populateFormFromAccount: (account: Account) => {
    set({
      form: {
        name: account.name,
        type: account.type,
        currency: account.currency,
        initial_balance: (account.initial_balance / 100).toString(),
        color: account.color,
        icon: account.icon,
      },
    });
  },

  // ==================== Currency Conversion ====================

  /**
   * Converts an amount from any supported currency to USD cents
   * @param amountCents - Amount in the original currency's cents
   * @param currency - Currency code ("USD" or "CLP")
   * @returns Amount in USD cents
   */
  convertToUSD: (amountCents: number, currency: string) => {
    if (currency === "USD") {
      return amountCents;
    }

    if (currency === "CLP") {
      const { clpToUsdRate } = get();
      if (!clpToUsdRate) {
        // Fallback rate if API hasn't been called yet (~950 CLP = 1 USD)
        return Math.round(amountCents / 950);
      }
      // Convert: CLP cents -> USD cents
      // amountCents is in CLP, divide by rate to get USD
      return Math.round(amountCents / clpToUsdRate);
    }

    // Unknown currency, return as-is
    return amountCents;
  },

  // ==================== Computed Getters (All in USD) ====================

  /**
   * Calculates total net worth across all accounts in USD
   * CLP accounts are converted using current exchange rate
   */
  getTotalNetWorthUSD: () => {
    const { accounts, balances, convertToUSD } = get();

    return balances.reduce((total, bal) => {
      const account = accounts.find((acc) => acc.id === bal.account_id);
      if (!account || account.is_archived) return total;

      const balanceUSD = convertToUSD(bal.current_balance, account.currency);
      return total + balanceUSD;
    }, 0);
  },

  /**
   * Calculates total income across all accounts in USD
   */
  getTotalIncomeUSD: () => {
    const { accounts, balances, convertToUSD } = get();

    return balances.reduce((total, bal) => {
      const account = accounts.find((acc) => acc.id === bal.account_id);
      if (!account || account.is_archived) return total;

      const incomeUSD = convertToUSD(bal.total_income, account.currency);
      return total + incomeUSD;
    }, 0);
  },

  /**
   * Calculates total expenses across all accounts in USD
   */
  getTotalExpensesUSD: () => {
    const { accounts, balances, convertToUSD } = get();

    return balances.reduce((total, bal) => {
      const account = accounts.find((acc) => acc.id === bal.account_id);
      if (!account || account.is_archived) return total;

      const expenseUSD = convertToUSD(bal.total_expense, account.currency);
      return total + expenseUSD;
    }, 0);
  },

  getAccountById: (id: string) => {
    return get().accounts.find((acc) => acc.id === id);
  },

  getAccountBalance: (id: string) => {
    return get().balances.find((bal) => bal.account_id === id);
  },

  // ==================== Messages ====================

  setError: (error: string | null) => set({ error }),

  setSuccess: (message: string | null) => set({ successMessage: message }),

  clearMessages: () => set({ error: null, successMessage: null }),

  // ==================== Security: RAM Clear ====================

  reset: () => {
    set({ ...initialState, form: { ...initialFormState } });
  },
}));

// ==================== Selector Hooks ====================

export const useAccounts = () => useAccountStore((state) => state.accounts);
export const useAccountBalances = () =>
  useAccountStore((state) => state.balances);
export const useAccountLoading = () =>
  useAccountStore((state) => state.isLoading);
export const useAccountError = () => useAccountStore((state) => state.error);
export const useAccountSuccess = () =>
  useAccountStore((state) => state.successMessage);
export const useAccountForm = () => useAccountStore((state) => state.form);
export const useAccountToEdit = () =>
  useAccountStore((state) => state.accountToEdit);
export const useClpUsdRate = () =>
  useAccountStore((state) => state.clpToUsdRate);
export const useRateLoading = () =>
  useAccountStore((state) => state.rateLoading);
