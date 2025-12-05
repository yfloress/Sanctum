/**
 * Session Manager - Handles session expiry detection and logout
 *
 * This module exists to break circular dependencies between stores.
 * It provides a centralized way to handle session expiry that all stores can use.
 *
 * ARCHITECTURE:
 * - Stores call `checkSessionError(err)` when they catch errors
 * - If it's a session error, this module triggers a global logout
 * - The logout callback is set by App.tsx on mount
 */

// ==================== Session Expiry Detection ====================

/**
 * Checks if an error is a session expiry error
 */
export function isSessionExpiredError(error: unknown): boolean {
  const errorStr = String(error);
  return (
    errorStr.includes("Session expired") || errorStr.includes("inactivity")
  );
}

// ==================== Global Logout Handler ====================

type LogoutHandler = () => void;

let globalLogoutHandler: LogoutHandler | null = null;
let sessionExpiredHandled = false;

/**
 * Sets the global logout handler (called by App.tsx on mount)
 * This breaks the circular dependency by using a callback pattern
 */
export function setGlobalLogoutHandler(handler: LogoutHandler): void {
  globalLogoutHandler = handler;
}

/**
 * Clears the global logout handler (called on cleanup)
 */
export function clearGlobalLogoutHandler(): void {
  globalLogoutHandler = null;
}

/**
 * Resets the session expired flag (called after successful login/logout)
 */
export function resetSessionExpiredFlag(): void {
  sessionExpiredHandled = false;
}

/**
 * Handles a session expired error by triggering global logout
 * Returns true if the error was a session expiry error (and was handled)
 * Returns false if it was a different error
 *
 * Usage in stores:
 * ```
 * try {
 *   await invoke("some_command");
 * } catch (err) {
 *   if (handleSessionError(err)) return;
 *   // Handle other errors...
 * }
 * ```
 */
export function handleSessionError(error: unknown): boolean {
  if (!isSessionExpiredError(error)) {
    return false;
  }

  // Prevent multiple handlers from running
  if (sessionExpiredHandled) {
    return true;
  }
  sessionExpiredHandled = true;

  // Trigger global logout if handler is set
  if (globalLogoutHandler) {
    globalLogoutHandler();
  } else {
    console.error(
      "[SessionManager] Session expired but no logout handler is set",
    );
  }

  return true;
}
