/**
 * Add Wallet Modal Component
 *
 * Modal for creating new crypto wallets.
 * Connects directly to Zustand store for state management.
 */

import type { FormEvent } from "react";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";
import { WALLET_CATEGORIES, WALLET_ICONS } from "../../../types/index.ts";

export function AddWalletModal() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const showAddWallet = useCryptoStore((state) => state.showAddWallet);
  const walletForm = useCryptoStore((state) => state.walletForm);

  // ==================== Store Actions ====================
  const setShowAddWallet = useCryptoStore((state) => state.setShowAddWallet);
  const setWalletFormField = useCryptoStore(
    (state) => state.setWalletFormField,
  );
  const addWallet = useCryptoStore((state) => state.addWallet);

  // ==================== Handlers ====================
  const handleAddWallet = async (e: FormEvent) => {
    e.preventDefault();
    await addWallet();
  };

  const handleClose = () => {
    setShowAddWallet(false);
  };

  // Don't render if modal is not visible
  if (!showAddWallet) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div
        className="modal-card crypto-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <span className="modal-icon">👛</span>
          <h2>Create Wallet</h2>
        </div>
        <form onSubmit={handleAddWallet}>
          <div className="modal-body">
            <div className="form-group">
              <label htmlFor="wallet-name">Wallet Name</label>
              <input
                id="wallet-name"
                type="text"
                value={walletForm.name}
                onChange={(e) => setWalletFormField("name", e.target.value)}
                placeholder="e.g. Binance, Ledger, Metamask..."
                required
              />
            </div>
            <div className="form-group">
              <label htmlFor="wallet-category">Category</label>
              <select
                id="wallet-category"
                value={walletForm.category}
                onChange={(e) =>
                  setWalletFormField("category", e.target.value)
                }
              >
                {WALLET_CATEGORIES.map((cat) => (
                  <option key={cat.value} value={cat.value}>
                    {cat.icon} {cat.label}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-group">
              <label>Icon</label>
              <div className="icon-picker">
                {WALLET_ICONS.map((icon) => (
                  <button
                    key={icon}
                    type="button"
                    className={`icon-option ${
                      walletForm.icon === icon ? "selected" : ""
                    }`}
                    onClick={() => setWalletFormField("icon", icon)}
                  >
                    {icon}
                  </button>
                ))}
              </div>
            </div>
          </div>
          <div className="modal-actions">
            <button
              type="button"
              className="btn-secondary"
              onClick={handleClose}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="btn-primary"
              disabled={isLoading}
            >
              {isLoading ? "Creating..." : "Create Wallet"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
