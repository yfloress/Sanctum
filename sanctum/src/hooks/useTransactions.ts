import { useState, useCallback, useMemo } from "react";
import type { FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Transaction, BalanceSummary } from "../types";
import { EXPENSE_CATEGORIES, INCOME_CATEGORIES } from "../types";
import { getLocalDateString } from "../utils";

interface UseTransactionsReturn {
  // State
  transactions: Transaction[];
  balance: BalanceSummary;
  amount: string;
  description: string;
  category: string;
  date: string;
  isExpense: boolean;
  transactionToDelete: string | null;
  categories: readonly string[];

  // Setters
  setAmount: (value: string) => void;
  setDescription: (value: string) => void;
  setCategory: (value: string) => void;
  setDate: (value: string) => void;
  setIsExpense: (value: boolean) => void;

  // Computed
  expensesByCategory: { name: string; value: number }[];
  balanceEvolution: {
    date: string;
    balance: number;
    income: number;
    expense: number;
  }[];

  // Actions
  loadTransactions: () => Promise<void>;
  loadBalance: () => Promise<void>;
  handleAddTransaction: (e: FormEvent) => Promise<void>;
  handleDeleteTransaction: (id: string) => void;
  confirmDelete: () => Promise<void>;
  cancelDelete: () => void;
  handleExpenseToggle: (checked: boolean) => void;
  resetState: () => void;
}

interface UseTransactionsOptions {
  onSuccess?: (message: string) => void;
  onError?: (message: string) => void;
  setIsLoading?: (loading: boolean) => void;
  clearMessages?: () => void;
}

export function useTransactions(
  options: UseTransactionsOptions = {},
): UseTransactionsReturn {
  const { onSuccess, onError, setIsLoading, clearMessages } = options;

  // Transaction state
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [balance, setBalance] = useState<BalanceSummary>({
    total_balance: 0,
    total_income: 0,
    total_expense: 0,
  });

  // Form state
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [category, setCategory] = useState<string>(EXPENSE_CATEGORIES[0]);
  const [date, setDate] = useState(() => getLocalDateString());
  const [isExpense, setIsExpense] = useState(true);
  const [transactionToDelete, setTransactionToDelete] = useState<string | null>(
    null,
  );

  // Computed: Categories based on transaction type
  const categories = useMemo(
    () => (isExpense ? EXPENSE_CATEGORIES : INCOME_CATEGORIES),
    [isExpense],
  );

  // Computed: Expenses grouped by category for pie chart
  const expensesByCategory = useMemo(() => {
    const expenses = transactions.filter(
      (tx: Transaction) => tx.type === "expense",
    );
    const grouped = expenses.reduce(
      (acc: Record<string, number>, tx: Transaction) => {
        acc[tx.category] = (acc[tx.category] || 0) + tx.amount;
        return acc;
      },
      {} as Record<string, number>,
    );

    return Object.entries(grouped)
      .map(([name, value]) => ({ name, value: (value as number) / 100 }))
      .sort((a, b) => b.value - a.value);
  }, [transactions]);

  // Computed: Balance evolution for area chart
  const balanceEvolution = useMemo(() => {
    if (transactions.length === 0) return [];

    const sorted = [...transactions].sort(
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
    return Object.entries(dailyData).map(([dateStr, data]) => {
      cumulative += (data.income - data.expense) / 100;
      return {
        date: dateStr,
        balance: cumulative,
        income: data.income / 100,
        expense: data.expense / 100,
      };
    });
  }, [transactions]);

  // Load transactions from backend
  const loadTransactions = useCallback(async () => {
    try {
      const txs = await invoke<Transaction[]>("get_transactions");
      setTransactions(txs);
    } catch (err) {
      console.error("Error loading transactions:", err);
    }
  }, []);

  // Load balance from backend
  const loadBalance = useCallback(async () => {
    try {
      const bal = await invoke<BalanceSummary>("get_balance");
      setBalance(bal);
    } catch (err) {
      console.error("Error loading balance:", err);
    }
  }, []);

  // Handle expense toggle
  const handleExpenseToggle = useCallback((checked: boolean) => {
    setIsExpense(checked);
    setCategory(checked ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0]);
  }, []);

  // Handle add transaction
  const handleAddTransaction = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      clearMessages?.();

      const parsedAmount = parseFloat(amount);
      if (!amount || parsedAmount <= 0) {
        onError?.("Amount must be greater than zero");
        return;
      }
      if (!category.trim()) {
        onError?.("Category cannot be empty");
        return;
      }

      try {
        setIsLoading?.(true);
        const amountInCents = Math.round(parsedAmount * 100);

        await invoke<string>("add_transaction", {
          amount: amountInCents,
          category: category.trim(),
          description: description.trim(),
          date: date,
          isExpense,
        });

        onSuccess?.(`${isExpense ? "Expense" : "Income"} added successfully`);

        // Reset form
        setAmount("");
        setDescription("");
        setCategory(isExpense ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0]);
        setDate(getLocalDateString());

        await loadTransactions();
        await loadBalance();
      } catch (err) {
        onError?.(`Error creating transaction: ${err}`);
      } finally {
        setIsLoading?.(false);
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
      onError,
      onSuccess,
      setIsLoading,
    ],
  );

  // Handle delete transaction
  const handleDeleteTransaction = useCallback((id: string) => {
    setTransactionToDelete(id);
  }, []);

  // Confirm delete
  const confirmDelete = useCallback(async () => {
    if (!transactionToDelete) return;

    try {
      setIsLoading?.(true);
      clearMessages?.();
      await invoke("delete_transaction", { id: transactionToDelete });
      onSuccess?.("Transaction deleted successfully");
      await loadTransactions();
      await loadBalance();
    } catch (err) {
      onError?.(`Error deleting transaction: ${err}`);
    } finally {
      setIsLoading?.(false);
      setTransactionToDelete(null);
    }
  }, [
    transactionToDelete,
    clearMessages,
    loadTransactions,
    loadBalance,
    onError,
    onSuccess,
    setIsLoading,
  ]);

  // Cancel delete
  const cancelDelete = useCallback(() => {
    setTransactionToDelete(null);
  }, []);

  // Reset all state (used when vault closes)
  const resetState = useCallback(() => {
    setTransactions([]);
    setBalance({
      total_balance: 0,
      total_income: 0,
      total_expense: 0,
    });
    setAmount("");
    setDescription("");
    setCategory(EXPENSE_CATEGORIES[0]);
    setDate(getLocalDateString());
    setIsExpense(true);
    setTransactionToDelete(null);
  }, []);

  return {
    // State
    transactions,
    balance,
    amount,
    description,
    category,
    date,
    isExpense,
    transactionToDelete,
    categories,

    // Setters
    setAmount,
    setDescription,
    setCategory,
    setDate,
    setIsExpense,

    // Computed
    expensesByCategory,
    balanceEvolution,

    // Actions
    loadTransactions,
    loadBalance,
    handleAddTransaction,
    handleDeleteTransaction,
    confirmDelete,
    cancelDelete,
    handleExpenseToggle,
    resetState,
  };
}
