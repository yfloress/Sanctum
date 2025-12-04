/**
 * SubTabs Component
 *
 * Navigation tabs for switching between Overview and Wallets views.
 * Connects directly to Zustand store for state management.
 */

import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function SubTabs() {
  // ==================== Store State ====================
  const subTab = useCryptoStore((state) => state.subTab);

  // ==================== Store Actions ====================
  const setSubTab = useCryptoStore((state) => state.setSubTab);
  const selectWallet = useCryptoStore((state) => state.selectWallet);

  // ==================== Handlers ====================
  const handleOverviewClick = () => {
    setSubTab("overview");
    selectWallet(null);
  };

  const handleWalletsClick = () => {
    setSubTab("wallets");
  };

  return (
    <div className="crypto-subtabs">
      <button
        type="button"
        className={`crypto-subtab ${subTab === "overview" ? "active" : ""}`}
        onClick={handleOverviewClick}
      >
        📊 Overview
      </button>
      <button
        type="button"
        className={`crypto-subtab ${subTab === "wallets" ? "active" : ""}`}
        onClick={handleWalletsClick}
      >
        👛 Wallets
      </button>
    </div>
  );
}
