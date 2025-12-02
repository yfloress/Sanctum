import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface UseAuthReturn {
  // State
  isInitialized: boolean;
  isLoading: boolean;
  password: string;
  showPassword: boolean;
  error: string;
  successMessage: string;
  dbPathInput: string;
  loadingAction: "open" | "create" | null;

  // Setters
  setPassword: (value: string) => void;
  setShowPassword: (value: boolean) => void;
  setDbPathInput: (value: string) => void;
  setError: (value: string) => void;

  // Actions
  handleVaultAction: (action: "open" | "create") => Promise<void>;
  handleCloseVault: () => Promise<void>;
  checkDatabaseStatus: () => Promise<void>;

  // Helpers
  setTemporaryError: (message: string, duration?: number) => void;
  setTemporarySuccess: (message: string, duration?: number) => void;
  clearMessages: () => void;
}

interface UseAuthOptions {
  onVaultOpen?: () => Promise<void>;
  onVaultClose?: () => void;
}

export function useAuth(options: UseAuthOptions = {}): UseAuthReturn {
  const { onVaultOpen, onVaultClose } = options;

  // Auth state
  const [isInitialized, setIsInitialized] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [successMessage, setSuccessMessage] = useState("");
  const [dbPathInput, setDbPathInput] = useState("");
  const [loadingAction, setLoadingAction] = useState<"open" | "create" | null>(
    null,
  );

  // Refs for timeouts
  const errorTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const successTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Helper Functions
  const setTemporaryError = useCallback((message: string, duration = 5000) => {
    if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
    setError(message);
    errorTimeoutRef.current = setTimeout(() => setError(""), duration);
  }, []);

  const setTemporarySuccess = useCallback(
    (message: string, duration = 3000) => {
      if (successTimeoutRef.current) clearTimeout(successTimeoutRef.current);
      setSuccessMessage(message);
      successTimeoutRef.current = setTimeout(
        () => setSuccessMessage(""),
        duration,
      );
    },
    [],
  );

  const clearMessages = useCallback(() => {
    setError("");
    setSuccessMessage("");
  }, []);

  // Load DB Path
  const loadDbPath = useCallback(async () => {
    try {
      const path = await invoke<string>("get_db_path");
      setDbPathInput(path);
    } catch (err) {
      console.error("Error getting path:", err);
    }
  }, []);

  // Check Database Status
  const checkDatabaseStatus = useCallback(async () => {
    try {
      setIsLoading(true);
      setError("");
      const initialized = await invoke<boolean>("is_db_initialized");
      setIsInitialized(initialized);
      await loadDbPath();
      if (initialized && onVaultOpen) {
        await onVaultOpen();
      }
    } catch (err) {
      setError(`Error checking status: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, [loadDbPath, onVaultOpen]);

  // Handle Vault Action (Open/Create)
  const handleVaultAction = useCallback(
    async (action: "open" | "create") => {
      clearMessages();

      const trimmedPassword = password.trim();
      if (!trimmedPassword) {
        setTemporaryError("Password cannot be empty");
        return;
      }
      if (trimmedPassword.length < 8) {
        setTemporaryError("Password must be at least 8 characters");
        return;
      }

      const targetPath = dbPathInput.trim() || null;

      try {
        setIsLoading(true);
        setLoadingAction(action);
        const command = action === "create" ? "create_db" : "open_db";
        await invoke<string>(command, {
          password: trimmedPassword,
          path: targetPath,
        });
        setIsInitialized(true);
        setPassword("");
        await loadDbPath();
        if (onVaultOpen) {
          await onVaultOpen();
        }
      } catch (err) {
        setTemporaryError(`Error: ${err}`);
      } finally {
        setIsLoading(false);
        setLoadingAction(null);
      }
    },
    [
      password,
      dbPathInput,
      clearMessages,
      loadDbPath,
      setTemporaryError,
      onVaultOpen,
    ],
  );

  // Handle Close Vault
  const handleCloseVault = useCallback(async () => {
    try {
      setIsLoading(true);
      clearMessages();
      const result = await invoke<string>("close_db");
      setTemporarySuccess(result);
      setIsInitialized(false);
      if (onVaultClose) {
        onVaultClose();
      }
      await loadDbPath();
    } catch (err) {
      setTemporaryError(`Error: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, [
    clearMessages,
    loadDbPath,
    setTemporaryError,
    setTemporarySuccess,
    onVaultClose,
  ]);

  // Cleanup timeouts on unmount
  useEffect(() => {
    return () => {
      if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
      if (successTimeoutRef.current) clearTimeout(successTimeoutRef.current);
    };
  }, []);

  // Check database status on mount
  useEffect(() => {
    checkDatabaseStatus();
  }, [checkDatabaseStatus]);

  return {
    // State
    isInitialized,
    isLoading,
    password,
    showPassword,
    error,
    successMessage,
    dbPathInput,
    loadingAction,

    // Setters
    setPassword,
    setShowPassword,
    setDbPathInput,
    setError,

    // Actions
    handleVaultAction,
    handleCloseVault,
    checkDatabaseStatus,

    // Helpers
    setTemporaryError,
    setTemporarySuccess,
    clearMessages,
  };
}
