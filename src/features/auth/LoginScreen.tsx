/**
 * Login Screen Component
 *
 * Handles vault authentication (open/create) with automatic vault detection.
 * Consumes state directly from Zustand authStore - no props needed.
 *
 * SECURITY:
 * - Password is stored in local component state only, never in Zustand store
 * - Buttons are disabled based on vault existence to prevent accidental actions
 * - Confirm password required for vault creation
 */

import { useState, useEffect } from "react";
import {
  useAuthLoading,
  useAuthStore,
  useDbPath,
  useLoadingAction,
  useVaultExists,
} from "../../stores/index.ts";

export function LoginScreen() {
  // Local state for passwords (NEVER stored in Zustand for security)
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);

  // Store state (read-only selectors)
  const isLoading = useAuthLoading();
  const dbPath = useDbPath();
  const loadingAction = useLoadingAction();
  const vaultExists = useVaultExists();

  // Store actions
  const login = useAuthStore((state) => state.login);
  const setDbPath = useAuthStore((state) => state.setDbPath);
  const clearMessages = useAuthStore((state) => state.clearMessages);
  const checkVaultExists = useAuthStore((state) => state.checkVaultExists);

  // Check vault existence on mount
  useEffect(() => {
    checkVaultExists();
  }, [checkVaultExists]);

  // Handle vault action (open or create)
  const handleVaultAction = async (action: "open" | "create") => {
    clearMessages();

    const success = await login(
      action,
      password,
      action === "create" ? confirmPassword : undefined,
      dbPath,
    );

    if (success) {
      // Clear passwords from local state after successful login
      setPassword("");
      setConfirmPassword("");
    }
  };

  // Determine button states based on vault existence
  const isCheckingVault = vaultExists === null;
  const canUnlock = vaultExists === true;
  const canCreate = vaultExists === false;

  // Determine which mode we're in for UI hints
  const isCreateMode = canCreate;

  return (
    <div className="vault-container">
      <div className="vault-card login-card">
        <div className="login-layout">
          <div className="login-branding">
            <div className="vault-icon locked">🔒</div>
            <h1>Sanctum</h1>
            <p className="vault-subtitle">Secure Financial Vault</p>
            <p className="vault-tagline">
              Your data is protected with AES-256 encryption
            </p>
          </div>

          <div className="login-form-section">
            {/* Vault Status Indicator */}
            <div className="vault-status">
              {isCheckingVault ? (
                <span className="status-checking">Checking vault...</span>
              ) : vaultExists ? (
                <span className="status-exists">🔐 Vault found</span>
              ) : (
                <span className="status-new">No vault found - Create one</span>
              )}
            </div>

            <form
              onSubmit={(e) => {
                e.preventDefault();
                if (canUnlock) {
                  handleVaultAction("open");
                } else if (canCreate) {
                  handleVaultAction("create");
                }
              }}
              className="vault-form"
            >
              <div className="form-group password-group">
                <label htmlFor="password">
                  {isCreateMode ? "Master Password" : "Password"}
                </label>
                <div className="password-input-wrapper">
                  <input
                    id="password"
                    type={showPassword ? "text" : "password"}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder={
                      isCreateMode
                        ? "Create a strong password"
                        : "Enter your password"
                    }
                    disabled={isLoading || isCheckingVault}
                    autoFocus
                  />
                  <button
                    type="button"
                    className="password-toggle"
                    onClick={() => setShowPassword(!showPassword)}
                    disabled={isLoading}
                    aria-label={
                      showPassword ? "Hide password" : "Show password"
                    }
                  >
                    {showPassword ? "👁️" : "🙈"}
                  </button>
                </div>
                <span className="input-hint">
                  {isCreateMode
                    ? "Minimum 8 characters - Choose wisely, this cannot be recovered"
                    : "Enter your master password to unlock"}
                </span>
              </div>

              {/* Confirm Password - Only shown in create mode */}
              {isCreateMode && (
                <div className="form-group password-group">
                  <label htmlFor="confirm-password">Confirm Password</label>
                  <div className="password-input-wrapper">
                    <input
                      id="confirm-password"
                      type={showPassword ? "text" : "password"}
                      value={confirmPassword}
                      onChange={(e) => setConfirmPassword(e.target.value)}
                      placeholder="Confirm your password"
                      disabled={isLoading || isCheckingVault}
                    />
                  </div>
                  {confirmPassword && password !== confirmPassword && (
                    <span className="input-hint input-error">
                      Passwords do not match
                    </span>
                  )}
                </div>
              )}

              <div className="button-row">
                {/* Unlock Button */}
                <button
                  type={canUnlock ? "submit" : "button"}
                  className={`btn-primary ${!canUnlock ? "btn-disabled" : ""}`}
                  disabled={isLoading || isCheckingVault || !canUnlock}
                  onClick={
                    canUnlock ? () => handleVaultAction("open") : undefined
                  }
                  title={
                    !canUnlock
                      ? "No vault exists - Create one first"
                      : undefined
                  }
                >
                  {isLoading && loadingAction === "open" ? (
                    <span className="btn-loading">Unlocking...</span>
                  ) : (
                    "Unlock"
                  )}
                </button>

                {/* Create Button */}
                <button
                  type={canCreate ? "submit" : "button"}
                  className={`btn-secondary ${!canCreate ? "btn-disabled" : "btn-create-active"}`}
                  disabled={isLoading || isCheckingVault || !canCreate}
                  onClick={
                    canCreate ? () => handleVaultAction("create") : undefined
                  }
                  title={
                    !canCreate ? "Vault already exists - Unlock it" : undefined
                  }
                >
                  {isLoading && loadingAction === "create" ? (
                    <span className="btn-loading">Creating...</span>
                  ) : (
                    "Create Vault"
                  )}
                </button>
              </div>

              <details className="path-details">
                <summary>Advanced options</summary>
                <div className="form-group path-group">
                  <label htmlFor="db-path">Vault Path</label>
                  <input
                    id="db-path"
                    type="text"
                    value={dbPath}
                    onChange={(e) => {
                      setDbPath(e.target.value);
                      // Re-check vault existence when path changes
                      checkVaultExists();
                    }}
                    placeholder="Custom path (empty = default)"
                    disabled={isLoading}
                  />
                  <span className="input-hint">
                    Last used path is remembered automatically
                  </span>
                </div>
              </details>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}
