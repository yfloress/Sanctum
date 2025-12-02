interface LoginScreenProps {
  password: string;
  setPassword: (value: string) => void;
  showPassword: boolean;
  setShowPassword: (value: boolean) => void;
  dbPathInput: string;
  setDbPathInput: (value: string) => void;
  isLoading: boolean;
  loadingAction: "open" | "create" | null;
  error: string;
  onVaultAction: (action: "open" | "create") => void;
}

export function LoginScreen({
  password,
  setPassword,
  showPassword,
  setShowPassword,
  dbPathInput,
  setDbPathInput,
  isLoading,
  loadingAction,
  error,
  onVaultAction,
}: LoginScreenProps) {
  return (
    <div className="vault-container">
      {error && <div className="message error login-message">{error}</div>}
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
            <form
              onSubmit={(e) => {
                e.preventDefault();
                onVaultAction("open");
              }}
              className="vault-form"
            >
              <div className="form-group password-group">
                <label htmlFor="password">Master Password</label>
                <div className="password-input-wrapper">
                  <input
                    id="password"
                    type={showPassword ? "text" : "password"}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="Enter your password"
                    disabled={isLoading}
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
                <span className="input-hint">Minimum 8 characters</span>
              </div>

              <div className="button-row">
                <button
                  type="submit"
                  className="btn-primary"
                  disabled={isLoading}
                >
                  {isLoading && loadingAction === "open" ? "..." : "Unlock"}
                </button>
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={() => onVaultAction("create")}
                  disabled={isLoading}
                >
                  {isLoading && loadingAction === "create" ? "..." : "Create"}
                </button>
              </div>

              <details className="path-details">
                <summary>Advanced options</summary>
                <div className="form-group path-group">
                  <label htmlFor="db-path">Vault Path</label>
                  <input
                    id="db-path"
                    type="text"
                    value={dbPathInput}
                    onChange={(e) => setDbPathInput(e.target.value)}
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
