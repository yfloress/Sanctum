/**
 * Financial Store - Zustand State Management
 *
 * SECURITY: This store lives in RAM only. NO persistence middleware.
 * The real persistence is handled by Rust (SQLCipher encrypted database).
 *
 * COHERENCE PRINCIPLE:
 * - Each transaction MUST belong to an account
 * - Balance = Sum of (Account Initial Balances) + Income - Expenses
 * - Transfers are handled by accountStore
 *
 * KILL SWITCH: The reset() action clears all data from memory when
 * the user locks the vault.
 */

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { BalanceSummary, Transaction } from "../types/index.ts";
import { handleSessionError } from "./sessionManager.ts";
import { EXPENSE_CATEGORIES, INCOME_CATEGORIES } from "../types/index.ts";
import { getLocalDateString } from "../utils/index.ts";

// ==================== Types ====================

interface TransactionFormData {
  accountId: string; // Required: which account this transaction belongs to
  amount: string;
  description: string;
  category: string;
  date: string;
  isExpense: boolean;
}

interface FinancialState {
  // Data State
  transactions: Transaction[];
  balance: BalanceSummary;

  // UI State
  isLoading: boolean;
  error: string | null;
  successMessage: string | null;

  // Form State
  form: TransactionFormData;
  transactionToDelete: string | null;

  // Computed (cached)
  _expensesByCategoryCache: { name: string; value: number }[] | null;
  _balanceEvolutionCache:
    | { date: string; balance: number; income: number; expense: number }[]
    | null;
}

interface FinancialActions {
  // Data Loading
  loadData: () => Promise<void>;
  loadTransactions: () => Promise<void>;
  loadBalance: () => Promise<void>;

  // CRUD Operations
  addTransaction: (data: TransactionFormData) => Promise<boolean>;
  deleteTransaction: (id: string) => Promise<boolean>;

  // Delete Confirmation Flow
  setTransactionToDelete: (id: string | null) => void;
  confirmDelete: () => Promise<boolean>;
  cancelDelete: () => void;

  // Form Management
  setFormField: <K extends keyof TransactionFormData>(
    field: K,
    value: TransactionFormData[K],
  ) => void;
  resetForm: () => void;
  setDefaultAccount: (accountId: string) => void;
  toggleExpenseType: (isExpense: boolean) => void;

  // Messages
  setError: (error: string | null) => void;
  setSuccess: (message: string | null) => void;
  clearMessages: () => void;

  // Computed Getters
  getExpensesByCategory: () => { name: string; value: number }[];
  getBalanceEvolution: () => {
    date: string;
    balance: number;
    income: number;
    expense: number;
  }[];
  getCategories: () => readonly string[];

  // Security: RAM Clear
  reset: () => void;
}

export type FinancialStore = FinancialState & FinancialActions;

// ==================== Initial State ====================

const initialFormState: TransactionFormData = {
  accountId: "", // Will be set when accounts are loaded
  amount: "",
  description: "",
  category: EXPENSE_CATEGORIES[0],
  date: getLocalDateString(),
  isExpense: true,
};

const initialState: FinancialState = {
  transactions: [],
  balance: {
    total_balance: 0,
    total_income: 0,
    total_expense: 0,
  },
  isLoading: false,
  error: null,
  successMessage: null,
  form: { ...initialFormState },
  transactionToDelete: null,
  _expensesByCategoryCache: null,
  _balanceEvolutionCache: null,
};

// ==================== Store ====================

export const useFinancialStore = create<FinancialStore>((set, get) => ({
  ...initialState,

  // ==================== Data Loading ====================

  loadData: async () => {
    const { loadTransactions, loadBalance } = get();
    set({ isLoading: true, error: null });
    try {
      await Promise.all([loadTransactions(), loadBalance()]);
    } catch (err) {
      set({ error: `Error loading financial data: ${err}` });
    } finally {
      set({ isLoading: false });
    }
  },

  loadTransactions: async () => {
    try {
      const transactions = await invoke<Transaction[]>("get_transactions");
      set({
        transactions,
        _expensesByCategoryCache: null,
        _balanceEvolutionCache: null,
      });
    } catch (err) {
      if (handleSessionError(err)) return;
      console.error("Error loading transactions:", err);
      throw err;
    }
  },

  loadBalance: async () => {
    try {
      const balance = await invoke<BalanceSummary>("get_balance");
      set({ balance });
    } catch (err) {
      if (handleSessionError(err)) return;
      console.error("Error loading balance:", err);
      throw err;
    }
  },

  // ==================== CRUD Operations ====================

  addTransaction: async (data: TransactionFormData) => {
    const parsedAmount = parseFloat(data.amount);

    // Validation
    if (!data.accountId) {
      set({ error: "Please select an account" });
      return false;
    }
    if (!data.amount || parsedAmount <= 0) {
      set({ error: "Amount must be greater than zero" });
      return false;
    }
    if (!data.category.trim()) {
      set({ error: "Category cannot be empty" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      const amountInCents = Math.round(parsedAmount * 100);

      await invoke<string>("add_transaction", {
        accountId: data.accountId,
        amount: amountInCents,
        category: data.category.trim(),
        description: data.description.trim(),
        date: data.date,
        isExpense: data.isExpense,
      });

      // Reload data
      await get().loadData();

      // Reset form and show success
      get().resetForm();
      set({
        successMessage: `${
          data.isExpense ? "Expense" : "Income"
        } added successfully`,
      });

      // Auto-clear success message
      setTimeout(() => {
        set({ successMessage: null });
      }, 3000);

      return true;
    } catch (err) {
      if (handleSessionError(err)) return false;
      set({ error: `Error creating transaction: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  deleteTransaction: async (id: string) => {
    set({ isLoading: true, error: null });

    try {
      await invoke("delete_transaction", { id });
      await get().loadData();

      set({ successMessage: "Transaction deleted successfully" });

      // Auto-clear success message
      setTimeout(() => {
        set({ successMessage: null });
      }, 3000);

      return true;
    } catch (err) {
      if (handleSessionError(err)) return false;
      set({ error: `Error deleting transaction: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  // ==================== Delete Confirmation Flow ====================

  setTransactionToDelete: (id: string | null) => {
    set({ transactionToDelete: id });
  },

  confirmDelete: async () => {
    const { transactionToDelete, deleteTransaction } = get();
    if (!transactionToDelete) return false;

    const success = await deleteTransaction(transactionToDelete);
    set({ transactionToDelete: null });
    return success;
  },

  cancelDelete: () => {
    set({ transactionToDelete: null });
  },

  // ==================== Form Management ====================

  setFormField: (field, value) => {
    set((state) => ({
      form: { ...state.form, [field]: value },
    }));
  },

  resetForm: () => {
    const { isExpense, accountId } = get().form;
    set({
      form: {
        ...initialFormState,
        accountId, // Preserve selected account
        isExpense,
        category: isExpense ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0],
        date: getLocalDateString(),
      },
    });
  },

  setDefaultAccount: (accountId: string) => {
    set((state) => ({
      form: { ...state.form, accountId },
    }));
  },

  toggleExpenseType: (isExpense: boolean) => {
    set((state) => ({
      form: {
        ...state.form,
        isExpense,
        category: isExpense ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0],
      },
    }));
  },

  // ==================== Messages ====================

  setError: (error: string | null) => set({ error }),

  setSuccess: (message: string | null) => set({ successMessage: message }),

  clearMessages: () => set({ error: null, successMessage: null }),

  // ==================== Computed Getters ====================

  getExpensesByCategory: () => {
    const state = get();

    // Return cached if available
    if (state._expensesByCategoryCache) {
      return state._expensesByCategoryCache;
    }

    const expenses = state.transactions.filter((tx) => tx.type === "expense");
    const grouped = expenses.reduce(
      (acc: Record<string, number>, tx: Transaction) => {
        acc[tx.category] = (acc[tx.category] || 0) + tx.amount;
        return acc;
      },
      {} as Record<string, number>,
    );

    const result = Object.entries(grouped)
      .map(([name, value]) => ({ name, value: (value as number) / 100 }))
      .sort((a, b) => b.value - a.value);

    // Cache the result
    set({ _expensesByCategoryCache: result });

    return result;
  },

  getBalanceEvolution: () => {
    const state = get();

    // Return cached if available
    if (state._balanceEvolutionCache) {
      return state._balanceEvolutionCache;
    }

    if (state.transactions.length === 0) {
      return [];
    }

    const sorted = [...state.transactions].sort(
      (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
    );

    const dailyData: Record<string, { income: number; expense: number }> = {};

    sorted.forEach((tx) => {
      const [, month, day] = tx.date.split("T")[0].split("-");
      const months = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
      ];
      const dateKey = `${months[parseInt(month) - 1]} ${parseInt(day)}`;

      if (!dailyData[dateKey]) {
        dailyData[dateKey] = { income: 0, expense: 0 };
      }

      if (tx.type === "income") {
        dailyData[dateKey].income += tx.amount;
      } else {
        dailyData[dateKey].expense += tx.amount;
      }
    });

    let cumulative = 0;
    const result = Object.entries(dailyData).map(([dateStr, data]) => {
      cumulative += (data.income - data.expense) / 100;
      return {
        date: dateStr,
        balance: cumulative,
        income: data.income / 100,
        expense: data.expense / 100,
      };
    });

    // Cache the result
    set({ _balanceEvolutionCache: result });

    return result;
  },

  getCategories: () => {
    const { isExpense } = get().form;
    return isExpense ? EXPENSE_CATEGORIES : INCOME_CATEGORIES;
  },

  // ==================== Security: RAM Clear ====================

  reset: () => {
    set({ ...initialState, form: { ...initialFormState } });
  },
}));

// ==================== Selector Hooks (for optimized re-renders) ====================

// Use these selectors in components to avoid unnecessary re-renders
export const useTransactions = () =>
  useFinancialStore((state) => state.transactions);
export const useBalance = () => useFinancialStore((state) => state.balance);
export const useFinancialLoading = () =>
  useFinancialStore((state) => state.isLoading);
export const useFinancialError = () =>
  useFinancialStore((state) => state.error);
export const useFinancialSuccess = () =>
  useFinancialStore((state) => state.successMessage);
export const useTransactionForm = () =>
  useFinancialStore((state) => state.form);
export const useTransactionToDelete = () =>
  useFinancialStore((state) => state.transactionToDelete);
