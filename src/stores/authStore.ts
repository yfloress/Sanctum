/**
 * Auth Store - Zustand State Management
 *
 * SECURITY:
 * - Password is NEVER stored in state (passed as argument to actions)
 * - On logout, triggers RAM kill switch on all other stores
 * - All sensitive data lives only in memory (no persistence)
 */

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useFinancialStore } from "./financialStore.ts";
import { useCryptoStore } from "./cryptoStore.ts";
import { useHabitStore } from "./habitStore.ts";
import { useAccountStore } from "./accountStore.ts";
import {
  isSessionExpiredError,
  resetSessionExpiredFlag,
} from "./sessionManager.ts";

// ==================== Types ====================

interface AuthState {
  // Vault State
  isInitialized: boolean;
  isLoading: boolean;
  loadingAction: "open" | "create" | "check" | null;

  // Vault Existence (for login screen logic)
  vaultExists: boolean | null; // null = not checked yet

  // UI State
  error: string | null;
  successMessage: string | null;

  // Config
  dbPath: string;
}

interface AuthActions {
  // Core Actions
  checkStatus: () => Promise<void>;
  checkVaultExists: () => Promise<void>;
  login: (
    action: "open" | "create",
    password: string,
    confirmPassword?: string,
    customPath?: string,
  ) => Promise<boolean>;
  logout: () => Promise<void>;

  // Path Management
  setDbPath: (path: string) => void;

  // Messages
  setError: (error: string | null) => void;
  setSuccess: (message: string | null) => void;
  clearMessages: () => void;

  // Security: RAM Clear (called internally)
  _clearAllStores: () => void;
}

export type AuthStore = AuthState & AuthActions;

// ==================== Initial State ====================

const initialState: AuthState = {
  isInitialized: false,
  isLoading: true, // Start loading (checking status)
  loadingAction: "check",
  vaultExists: null,
  error: null,
  successMessage: null,
  dbPath: "",
};

// ==================== Store ====================

export const useAuthStore = create<AuthStore>((set, get) => ({
  ...initialState,

  // ==================== Core Actions ====================

  checkVaultExists: async () => {
    try {
      const exists = await invoke<boolean>("check_vault_exists");
      set({ vaultExists: exists });
    } catch (err) {
      // If check fails, assume no vault exists (safer for new users)
      set({ vaultExists: false });
    }
  },

  checkStatus: async () => {
    set({ isLoading: true, loadingAction: "check", error: null });

    try {
      // Check vault existence first
      await get().checkVaultExists();

      const [isInitialized, dbPath] = await Promise.all([
        invoke<boolean>("is_db_initialized"),
        invoke<string>("get_db_path"),
      ]);

      set({ isInitialized, dbPath });

      // If already initialized (app reloaded with open vault), load data
      if (isInitialized) {
        try {
          await Promise.all([
            useFinancialStore.getState().loadData(),
            useCryptoStore.getState().loadAll(),
          ]);
        } catch (err) {
          console.error("Error loading data after status check:", err);
        }
      }
    } catch (err) {
      set({ error: `Error checking vault status: ${err}` });
    } finally {
      set({ isLoading: false, loadingAction: null });
    }
  },

  login: async (
    action: "open" | "create",
    password: string,
    confirmPassword?: string,
    customPath?: string,
  ) => {
    const trimmedPassword = password.trim();

    // Validation
    if (!trimmedPassword) {
      set({ error: "Password cannot be empty" });
      return false;
    }

    if (action === "create") {
      if (trimmedPassword.length < 8) {
        set({ error: "Password must be at least 8 characters" });
        return false;
      }

      // Confirm password validation for create mode
      if (
        confirmPassword !== undefined &&
        trimmedPassword !== confirmPassword.trim()
      ) {
        set({ error: "Passwords do not match" });
        return false;
      }
    }

    set({ isLoading: true, loadingAction: action, error: null });

    try {
      const command = action === "create" ? "create_db" : "open_db";
      const targetPath = customPath?.trim() || null;

      await invoke<string>(command, {
        password: trimmedPassword,
        path: targetPath,
      });

      // Update path and vault existence after successful login
      const dbPath = await invoke<string>("get_db_path");
      set({ isInitialized: true, dbPath, vaultExists: true });

      // Load all data after successful login
      try {
        // Load accounts first (needed for transactions)
        await useAccountStore.getState().loadAll();

        await Promise.all([
          useFinancialStore.getState().loadData(),
          useCryptoStore.getState().loadAll(),
        ]);
      } catch (err) {
        console.error("Error loading initial data:", err);
        // Don't fail login if data load fails
      }

      set({
        successMessage:
          action === "create"
            ? "Vault created successfully"
            : "Vault unlocked successfully",
      });

      // Auto-clear success message
      setTimeout(() => {
        set({ successMessage: null });
      }, 3000);

      return true;
    } catch (err) {
      set({ error: `Error: ${err}` });
      return false;
    } finally {
      set({ isLoading: false, loadingAction: null });
    }
  },

  logout: async () => {
    set({ isLoading: true, error: null });

    try {
      const result = await invoke<string>("close_db");

      // SECURITY: Clear all sensitive data from RAM
      get()._clearAllStores();
      useAccountStore.getState().reset();

      set({
        isInitialized: false,
        successMessage: result,
      });

      // Reset session expired flag in session manager
      resetSessionExpiredFlag();

      // Auto-clear success message
      setTimeout(() => {
        set({ successMessage: null });
      }, 3000);

      // Reload path
      try {
        const dbPath = await invoke<string>("get_db_path");
        set({ dbPath });
      } catch {
        // Ignore path reload errors
      }
    } catch (err) {
      const errorStr = String(err);

      // Handle session expiry
      if (isSessionExpiredError(errorStr)) {
        // SECURITY: Clear stores even on error
        get()._clearAllStores();
        resetSessionExpiredFlag();
        set({
          isInitialized: false,
          error: "Session expired due to inactivity",
        });
      } else {
        set({ error: `Error: ${err}` });
      }
    } finally {
      set({ isLoading: false });
    }
  },

  // ==================== Path Management ====================

  setDbPath: (path: string) => set({ dbPath: path }),

  // ==================== Messages ====================

  setError: (error: string | null) => set({ error }),

  setSuccess: (message: string | null) => set({ successMessage: message }),

  clearMessages: () => set({ error: null, successMessage: null }),

  // ==================== Security: RAM Clear ====================

  _clearAllStores: () => {
    // Reset all stores - RAM kill switch
    useFinancialStore.getState().reset();
    useCryptoStore.getState().reset();
    useHabitStore.getState().reset();
    useAccountStore.getState().reset();
  },
}));

// ==================== Selector Hooks ====================

export const useIsInitialized = () =>
  useAuthStore((state) => state.isInitialized);
export const useAuthLoading = () => useAuthStore((state) => state.isLoading);
export const useAuthError = () => useAuthStore((state) => state.error);
export const useAuthSuccess = () =>
  useAuthStore((state) => state.successMessage);
export const useDbPath = () => useAuthStore((state) => state.dbPath);
export const useLoadingAction = () =>
  useAuthStore((state) => state.loadingAction);
export const useVaultExists = () => useAuthStore((state) => state.vaultExists);
