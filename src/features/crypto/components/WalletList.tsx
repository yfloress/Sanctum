/**
 * WalletList Component
 *
 * Displays the grid of crypto wallets with actions.
 * Connects directly to Zustand store for state management.
 */

import type { CryptoWallet } from "../../../types/index.ts";
import { WALLET_CATEGORIES } from "../../../types/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function WalletList() {
  // ==================== Store State ====================
  const wallets = useCryptoStore((state) => state.wallets);

  // ==================== Store Actions ====================
  const selectWallet = useCryptoStore((state) => state.selectWallet);
  const setShowAddWallet = useCryptoStore((state) => state.setShowAddWallet);
  const setWalletToDelete = useCryptoStore((state) => state.setWalletToDelete);

  // ==================== Helpers ====================
  const getWalletCategoryLabel = (category: string) => {
    const found = WALLET_CATEGORIES.find((c) => c.value === category);
    return found ? found.label : category;
  };

  // ==================== Render ====================
  return (
    <>
      <div className="section-header">
        <h2 className="section-title">My Wallets</h2>
        <button
          type="button"
          className="btn-primary"
          onClick={() => setShowAddWallet(true)}
        >
          + Add Wallet
        </button>
      </div>

      {wallets.length === 0 ? (
        <div className="portfolio-empty">
          <span className="portfolio-empty-icon">👛</span>
          <p>No wallets yet. Create your first wallet to start tracking!</p>
          <button
            type="button"
            className="btn-secondary"
            onClick={() => setShowAddWallet(true)}
          >
            + Create Wallet
          </button>
        </div>
      ) : (
        <div className="wallets-grid">
          {wallets.map((wallet: CryptoWallet) => (
            <div
              key={wallet.id}
              className="wallet-card"
              onClick={() => selectWallet(wallet)}
            >
              <button
                type="button"
                className="crypto-remove"
                onClick={(e) => {
                  e.stopPropagation();
                  setWalletToDelete(wallet.id);
                }}
                title="Delete wallet"
              >
                ×
              </button>
              <div className="wallet-icon">{wallet.icon || "👛"}</div>
              <div className="wallet-name">{wallet.name}</div>
              <div className="wallet-category">
                {getWalletCategoryLabel(wallet.category)}
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
