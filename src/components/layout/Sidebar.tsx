import type { TabType } from "../../types";

interface SidebarProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  onLockVault: () => void;
  isLoading: boolean;
  onCryptoTabClick?: () => void;
}

export function Sidebar({
  activeTab,
  setActiveTab,
  onLockVault,
  isLoading,
  onCryptoTabClick,
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <span className="logo-icon">🔓</span>
        <span className="logo-text">Sanctum</span>
      </div>

      <nav className="sidebar-nav">
        <button
          className={`nav-item ${activeTab === "dashboard" ? "active" : ""}`}
          onClick={() => setActiveTab("dashboard")}
        >
          <span className="nav-icon">📊</span>
          <span className="nav-label">Dashboard</span>
        </button>
        <button
          className={`nav-item ${activeTab === "transactions" ? "active" : ""}`}
          onClick={() => setActiveTab("transactions")}
        >
          <span className="nav-icon">💸</span>
          <span className="nav-label">Transactions</span>
        </button>
        <button
          className={`nav-item ${activeTab === "analytics" ? "active" : ""}`}
          onClick={() => setActiveTab("analytics")}
        >
          <span className="nav-icon">📈</span>
          <span className="nav-label">Analytics</span>
        </button>
        <button
          className={`nav-item ${activeTab === "crypto" ? "active" : ""}`}
          onClick={() => {
            setActiveTab("crypto");
            onCryptoTabClick?.();
          }}
        >
          <span className="nav-icon">₿</span>
          <span className="nav-label">Crypto</span>
        </button>
        <button
          className={`nav-item ${activeTab === "habits" ? "active" : ""}`}
          onClick={() => setActiveTab("habits")}
        >
          <span className="nav-icon">🎯</span>
          <span className="nav-label">Habits</span>
        </button>
      </nav>

      <div className="sidebar-footer">
        <button
          onClick={onLockVault}
          className="nav-item lock-btn"
          disabled={isLoading}
        >
          <span className="nav-icon">🔒</span>
          <span className="nav-label">
            {isLoading ? "Locking..." : "Lock Vault"}
          </span>
        </button>
      </div>
    </aside>
  );
}
