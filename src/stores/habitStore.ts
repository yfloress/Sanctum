/**
 * Habit Store - Zustand State Management (OPTIMIZED)
 *
 * PERFORMANCE OPTIMIZATIONS:
 * - Pre-computed statistics cached in state
 * - Stats only recalculated when logs change
 * - O(1) lookup for completions via Set
 * - Memoized selectors for React components
 *
 * SECURITY: This store lives in RAM only. NO persistence middleware.
 * The real persistence is handled by Rust (SQLCipher encrypted database).
 */

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Habit, HabitLog } from "../types/index.ts";

// ==================== Types ====================

interface HabitFormData {
  name: string;
  description: string;
  color: string;
}

interface HabitStats {
  completionRates: Map<string, number>; // habitId -> percentage
  streaks: Map<string, number>; // habitId -> streak count
  completionsPerDay: Map<string, number>; // "YYYY-MM-DD" -> count
  totalCompletions: number;
}

interface HabitState {
  // Data State
  habits: Habit[];
  logs: Set<string>; // Format: "habitId:YYYY-MM-DD" for O(1) lookup
  logsRaw: HabitLog[]; // Raw logs for statistics

  // Cached Statistics (recalculated only when logs change)
  stats: HabitStats;

  // Navigation State
  currentMonth: Date;

  // UI State
  isLoading: boolean;
  error: string | null;
  successMessage: string | null;

  // Form State
  form: HabitFormData;
  habitToEdit: Habit | null;
  habitToDelete: Habit | null;
  showAddModal: boolean;
}

interface HabitActions {
  // Data Loading
  loadHabits: () => Promise<void>;
  loadLogsForMonth: (year: number, month: number) => Promise<void>;
  loadAll: () => Promise<void>;

  // Habit CRUD
  addHabit: (data: HabitFormData) => Promise<boolean>;
  updateHabit: (id: string, data: HabitFormData) => Promise<boolean>;
  archiveHabit: (id: string) => Promise<boolean>;
  deleteHabit: (id: string) => Promise<boolean>;

  // Log Operations (Optimistic UI)
  toggleLog: (habitId: string, date: string) => void; // Synchronous for instant UI
  isCompleted: (habitId: string, date: string) => boolean;

  // Navigation
  setCurrentMonth: (date: Date) => void;
  goToPreviousMonth: () => void;
  goToNextMonth: () => void;
  goToToday: () => void;

  // Form Management
  setFormField: <K extends keyof HabitFormData>(
    field: K,
    value: HabitFormData[K],
  ) => void;
  resetForm: () => void;
  setShowAddModal: (show: boolean) => void;
  setHabitToEdit: (habit: Habit | null) => void;
  setHabitToDelete: (habit: Habit | null) => void;

  // Delete Confirmation Flow
  confirmDelete: () => Promise<boolean>;
  cancelDelete: () => void;

  // Messages
  setError: (error: string | null) => void;
  setSuccess: (message: string | null) => void;
  clearMessages: () => void;

  // Cached Stats Getters (O(1) lookups)
  getCompletionRate: (habitId: string) => number;
  getTotalCompletionsForMonth: () => number;
  getCompletionsPerDay: () => Map<string, number>;
  getCurrentStreak: (habitId: string) => number;

  // Security: RAM Clear
  reset: () => void;
}

export type HabitStore = HabitState & HabitActions;

// ==================== Initial State ====================

const initialFormState: HabitFormData = {
  name: "",
  description: "",
  color: "#8b5cf6",
};

const initialStats: HabitStats = {
  completionRates: new Map(),
  streaks: new Map(),
  completionsPerDay: new Map(),
  totalCompletions: 0,
};

const initialState: HabitState = {
  habits: [],
  logs: new Set(),
  logsRaw: [],
  stats: initialStats,
  currentMonth: new Date(),
  isLoading: false,
  error: null,
  successMessage: null,
  form: { ...initialFormState },
  habitToEdit: null,
  habitToDelete: null,
  showAddModal: false,
};

// ==================== Helpers ====================

/**
 * Creates a log key for O(1) lookup in the Set
 */
const createLogKey = (habitId: string, date: string): string =>
  `${habitId}:${date}`;

/**
 * Gets the start and end dates for a month
 */
const getMonthRange = (
  year: number,
  month: number,
): { start: string; end: string } => {
  const start = new Date(year, month, 1);
  const end = new Date(year, month + 1, 0); // Last day of month

  const formatDate = (d: Date): string => {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${y}-${m}-${day}`;
  };

  return {
    start: formatDate(start),
    end: formatDate(end),
  };
};

/**
 * Formats a date to YYYY-MM-DD
 */
const formatDateToISO = (date: Date): string => {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
};

/**
 * Calculates all statistics at once (called when logs or habits change)
 * This is the key optimization - we compute everything once instead of per-render
 */
const calculateStats = (
  habits: Habit[],
  logs: Set<string>,
  logsRaw: HabitLog[],
  currentMonth: Date,
): HabitStats => {
  const year = currentMonth.getFullYear();
  const month = currentMonth.getMonth();
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const today = new Date();
  const isCurrentMonth = today.getFullYear() === year &&
    today.getMonth() === month;
  const maxDays = isCurrentMonth ? today.getDate() : daysInMonth;

  // Calculate completion rates for all habits
  const completionRates = new Map<string, number>();
  habits.forEach((habit) => {
    let completions = 0;
    for (let day = 1; day <= maxDays; day++) {
      const date = `${year}-${String(month + 1).padStart(2, "0")}-${
        String(day).padStart(2, "0")
      }`;
      if (logs.has(createLogKey(habit.id, date))) {
        completions++;
      }
    }
    const rate = maxDays > 0 ? Math.round((completions / maxDays) * 100) : 0;
    completionRates.set(habit.id, rate);
  });

  // Calculate streaks for all habits
  const streaks = new Map<string, number>();
  habits.forEach((habit) => {
    let streak = 0;
    const checkDate = new Date();

    for (let i = 0; i < 365; i++) {
      const dateStr = formatDateToISO(checkDate);
      if (logs.has(createLogKey(habit.id, dateStr))) {
        streak++;
      } else if (i > 0) {
        // Allow today to be incomplete
        break;
      }
      checkDate.setDate(checkDate.getDate() - 1);
    }
    streaks.set(habit.id, streak);
  });

  // Calculate completions per day
  const completionsPerDay = new Map<string, number>();
  logsRaw.forEach((log) => {
    const count = completionsPerDay.get(log.completed_date) || 0;
    completionsPerDay.set(log.completed_date, count + 1);
  });

  return {
    completionRates,
    streaks,
    completionsPerDay,
    totalCompletions: logsRaw.length,
  };
};

// ==================== Store ====================

export const useHabitStore = create<HabitStore>((set, get) => ({
  ...initialState,

  // ==================== Data Loading ====================

  loadHabits: async () => {
    try {
      const habits = await invoke<Habit[]>("get_habits");
      const { logs, logsRaw, currentMonth } = get();
      const stats = calculateStats(habits, logs, logsRaw, currentMonth);
      set({ habits, stats });
    } catch (err) {
      console.error("Error loading habits:", err);
      throw err;
    }
  },

  loadLogsForMonth: async (year: number, month: number) => {
    try {
      const { start, end } = getMonthRange(year, month);
      const logsRaw = await invoke<HabitLog[]>("get_habit_logs", {
        startDate: start,
        endDate: end,
      });

      // Build the Set for O(1) lookup
      const logs = new Set<string>();
      logsRaw.forEach((log) => {
        logs.add(createLogKey(log.habit_id, log.completed_date));
      });

      // Recalculate stats with new logs
      const { habits, currentMonth } = get();
      const stats = calculateStats(habits, logs, logsRaw, currentMonth);

      set({ logs, logsRaw, stats });
    } catch (err) {
      console.error("Error loading habit logs:", err);
      throw err;
    }
  },

  loadAll: async () => {
    const { loadHabits, loadLogsForMonth, currentMonth } = get();
    set({ isLoading: true, error: null });

    try {
      await loadHabits();
      await loadLogsForMonth(
        currentMonth.getFullYear(),
        currentMonth.getMonth(),
      );
    } catch (err) {
      set({ error: `Error loading habits data: ${err}` });
    } finally {
      set({ isLoading: false });
    }
  },

  // ==================== Habit CRUD ====================

  addHabit: async (data: HabitFormData) => {
    if (!data.name.trim()) {
      set({ error: "Habit name cannot be empty" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      await invoke<string>("create_habit", {
        name: data.name.trim(),
        description: data.description.trim() || null,
        color: data.color,
      });

      // Reload habits
      await get().loadHabits();

      // Reset form and close modal
      get().resetForm();
      set({
        showAddModal: false,
        successMessage: "Habit created successfully",
      });

      setTimeout(() => set({ successMessage: null }), 3000);

      return true;
    } catch (err) {
      set({ error: `Error creating habit: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  updateHabit: async (id: string, data: HabitFormData) => {
    if (!data.name.trim()) {
      set({ error: "Habit name cannot be empty" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      await invoke("update_habit", {
        id,
        name: data.name.trim(),
        description: data.description.trim() || null,
        color: data.color,
      });

      await get().loadHabits();

      get().resetForm();
      set({
        habitToEdit: null,
        successMessage: "Habit updated successfully",
      });

      setTimeout(() => set({ successMessage: null }), 3000);

      return true;
    } catch (err) {
      set({ error: `Error updating habit: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  archiveHabit: async (id: string) => {
    set({ isLoading: true, error: null });

    try {
      await invoke("archive_habit", { id });
      await get().loadHabits();

      set({ successMessage: "Habit archived successfully" });
      setTimeout(() => set({ successMessage: null }), 3000);

      return true;
    } catch (err) {
      set({ error: `Error archiving habit: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  deleteHabit: async (id: string) => {
    set({ isLoading: true, error: null });

    try {
      await invoke("delete_habit", { id });

      // Reload habits and logs
      const { currentMonth, loadHabits, loadLogsForMonth } = get();
      await loadHabits();
      await loadLogsForMonth(
        currentMonth.getFullYear(),
        currentMonth.getMonth(),
      );

      set({ successMessage: "Habit deleted successfully" });
      setTimeout(() => set({ successMessage: null }), 3000);

      return true;
    } catch (err) {
      set({ error: `Error deleting habit: ${err}` });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  // ==================== Log Operations (Optimistic UI) ====================

  toggleLog: (habitId: string, date: string) => {
    const { logs, logsRaw, habits, currentMonth } = get();
    const key = createLogKey(habitId, date);
    const wasCompleted = logs.has(key);

    // OPTIMISTIC UPDATE: Update UI immediately (synchronous!)
    const newLogs = new Set(logs);
    let newLogsRaw = [...logsRaw];

    if (wasCompleted) {
      newLogs.delete(key);
      newLogsRaw = newLogsRaw.filter(
        (log) => !(log.habit_id === habitId && log.completed_date === date),
      );
    } else {
      newLogs.add(key);
      newLogsRaw.push({
        id: `temp-${Date.now()}`,
        habit_id: habitId,
        completed_date: date,
      });
    }

    // Recalculate stats with new logs
    const newStats = calculateStats(habits, newLogs, newLogsRaw, currentMonth);

    // Update state immediately (this is what makes it instant)
    set({ logs: newLogs, logsRaw: newLogsRaw, stats: newStats });

    // BACKEND SYNC: Fire and forget (don't await!)
    invoke<[boolean, string | null]>("toggle_habit_completion", {
      habitId,
      date,
    }).catch((err) => {
      // ROLLBACK on error
      console.error("Error toggling habit:", err);

      // Revert to previous state
      const revertStats = calculateStats(habits, logs, logsRaw, currentMonth);
      set({
        logs,
        logsRaw,
        stats: revertStats,
        error: `Failed to save: ${err}`,
      });

      setTimeout(() => set({ error: null }), 3000);
    });
  },

  isCompleted: (habitId: string, date: string) => {
    return get().logs.has(createLogKey(habitId, date));
  },

  // ==================== Navigation ====================

  setCurrentMonth: (date: Date) => {
    set({ currentMonth: date });
    // Load logs for the new month
    get().loadLogsForMonth(date.getFullYear(), date.getMonth());
  },

  goToPreviousMonth: () => {
    const { currentMonth } = get();
    const newMonth = new Date(
      currentMonth.getFullYear(),
      currentMonth.getMonth() - 1,
      1,
    );
    get().setCurrentMonth(newMonth);
  },

  goToNextMonth: () => {
    const { currentMonth } = get();
    const newMonth = new Date(
      currentMonth.getFullYear(),
      currentMonth.getMonth() + 1,
      1,
    );
    get().setCurrentMonth(newMonth);
  },

  goToToday: () => {
    get().setCurrentMonth(new Date());
  },

  // ==================== Form Management ====================

  setFormField: (field, value) => {
    set((state) => ({
      form: { ...state.form, [field]: value },
    }));
  },

  resetForm: () => {
    set({ form: { ...initialFormState } });
  },

  setShowAddModal: (show: boolean) => {
    if (show) {
      get().resetForm();
    }
    set({ showAddModal: show });
  },

  setHabitToEdit: (habit: Habit | null) => {
    if (habit) {
      set({
        form: {
          name: habit.name,
          description: habit.description || "",
          color: habit.color,
        },
        habitToEdit: habit,
      });
    } else {
      get().resetForm();
      set({ habitToEdit: null });
    }
  },

  setHabitToDelete: (habit: Habit | null) => {
    set({ habitToDelete: habit });
  },

  // ==================== Delete Confirmation Flow ====================

  confirmDelete: async () => {
    const { habitToDelete, deleteHabit } = get();
    if (!habitToDelete) return false;

    const success = await deleteHabit(habitToDelete.id);
    set({ habitToDelete: null });
    return success;
  },

  cancelDelete: () => {
    set({ habitToDelete: null });
  },

  // ==================== Messages ====================

  setError: (error: string | null) => set({ error }),

  setSuccess: (message: string | null) => set({ successMessage: message }),

  clearMessages: () => set({ error: null, successMessage: null }),

  // ==================== Cached Stats Getters (O(1) lookups) ====================

  getCompletionRate: (habitId: string) => {
    return get().stats.completionRates.get(habitId) || 0;
  },

  getTotalCompletionsForMonth: () => {
    return get().stats.totalCompletions;
  },

  getCompletionsPerDay: () => {
    return get().stats.completionsPerDay;
  },

  getCurrentStreak: (habitId: string) => {
    return get().stats.streaks.get(habitId) || 0;
  },

  // ==================== Security: RAM Clear ====================

  reset: () => {
    set({
      ...initialState,
      form: { ...initialFormState },
      logs: new Set(),
      stats: { ...initialStats },
      currentMonth: new Date(),
    });
  },
}));

// ==================== Selector Hooks ====================

export const useHabits = () => useHabitStore((state) => state.habits);
export const useHabitLogs = () => useHabitStore((state) => state.logs);
export const useCurrentMonth = () =>
  useHabitStore((state) => state.currentMonth);
export const useHabitLoading = () => useHabitStore((state) => state.isLoading);
export const useHabitError = () => useHabitStore((state) => state.error);
export const useHabitSuccess = () =>
  useHabitStore((state) => state.successMessage);
export const useHabitForm = () => useHabitStore((state) => state.form);
export const useShowAddModal = () =>
  useHabitStore((state) => state.showAddModal);
export const useHabitToEdit = () => useHabitStore((state) => state.habitToEdit);
export const useHabitToDelete = () =>
  useHabitStore((state) => state.habitToDelete);

// Pre-computed stats selectors (already cached in state)
export const useHabitStats = () => useHabitStore((state) => state.stats);
