/**
 * Legacy Holding Modals Component
 *
 * Groups legacy portfolio modals (Add Holding, Delete Holding).
 * These are marked as legacy because the new wallet-based system
 * should be preferred for tracking crypto holdings.
 *
 * Connects directly to Zustand store following project architecture.
 */

import type { FormEvent } from "react";
import { POPULAR_CRYPTOS } from "../../../types/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

/**
 * Add Holding Modal (Legacy)
 *
 * Modal for adding crypto holdings directly to portfolio.
 * @deprecated Prefer using wallet-based transactions instead.
 */
export function AddHoldingModal() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const showAddHolding = useCryptoStore((state) => state.showAddHolding);
  const holdingForm = useCryptoStore((state) => state.holdingForm);

  // ==================== Store Actions ====================
  const setShowAddHolding = useCryptoStore((state) => state.setShowAddHolding);
  const setHoldingFormField = useCryptoStore(
    (state) => state.setHoldingFormField,
  );
  const resetHoldingForm = useCryptoStore((state) => state.resetHoldingForm);
  const selectCoinForHolding = useCryptoStore(
    (state) => state.selectCoinForHolding,
  );
  const addHolding = useCryptoStore((state) => state.addHolding);

  // ==================== Handlers ====================
  const handleAddHolding = async (e: FormEvent) => {
    e.preventDefault();
    await addHolding();
  };

  const handleClose = () => {
    setShowAddHolding(false);
    resetHoldingForm();
  };

  // Don't render if modal is not visible
  if (!showAddHolding) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div
        className="modal-card crypto-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <span className="modal-icon">💼</span>
          <h2>Add to Portfolio (Legacy)</h2>
        </div>
        <form onSubmit={handleAddHolding}>
          <div className="modal-body">
            <div className="form-group">
              <label>Select Coin</label>
              <div className="crypto-suggestions compact">
                {POPULAR_CRYPTOS.map((coin) => (
                  <button
                    key={coin.id}
                    type="button"
                    className={`crypto-suggestion ${
                      holdingForm.coinId === coin.id ? "selected" : ""
                    }`}
                    onClick={() => selectCoinForHolding(coin)}
                  >
                    <span className="suggestion-symbol">{coin.symbol}</span>
                    <span className="suggestion-name">{coin.name}</span>
                  </button>
                ))}
              </div>
            </div>
            {holdingForm.coinId && (
              <>
                <div className="form-row">
                  <div className="form-group">
                    <label htmlFor="holding-amount">Amount</label>
                    <input
                      id="holding-amount"
                      type="number"
                      step="any"
                      value={holdingForm.amount}
                      onChange={(e) =>
                        setHoldingFormField("amount", e.target.value)
                      }
                      placeholder="0.00"
                      required
                    />
                  </div>
                  <div className="form-group">
                    <label htmlFor="holding-price">Purchase Price ($)</label>
                    <input
                      id="holding-price"
                      type="number"
                      step="any"
                      value={holdingForm.price}
                      onChange={(e) =>
                        setHoldingFormField("price", e.target.value)
                      }
                      placeholder="0.00"
                      required
                    />
                  </div>
                </div>
                <div className="form-group">
                  <label htmlFor="holding-date">Purchase Date</label>
                  <input
                    id="holding-date"
                    type="date"
                    value={holdingForm.date}
                    onChange={(e) =>
                      setHoldingFormField("date", e.target.value)
                    }
                  />
                </div>
              </>
            )}
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
              disabled={!holdingForm.coinId || isLoading}
            >
              {isLoading ? "Adding..." : "Add Holding"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

/**
 * Delete Holding Confirmation Modal (Legacy)
 *
 * Confirmation modal for removing legacy holdings.
 * @deprecated Part of the legacy holding system.
 */
export function DeleteHoldingModal() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const holdingToDelete = useCryptoStore((state) => state.holdingToDelete);

  // ==================== Store Actions ====================
  const setHoldingToDelete = useCryptoStore(
    (state) => state.setHoldingToDelete,
  );
  const deleteHolding = useCryptoStore((state) => state.deleteHolding);

  // ==================== Handlers ====================
  const handleClose = () => {
    setHoldingToDelete(null);
  };

  const handleDelete = async () => {
    await deleteHolding();
  };

  // Don't render if no holding is selected for deletion
  if (holdingToDelete === null) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div
        className="modal-card delete-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <span className="modal-icon">⚠️</span>
          <h2>Remove Holding</h2>
        </div>
        <div className="modal-body">
          <p>Are you sure you want to remove this holding?</p>
          <p className="modal-warning">This action cannot be undone.</p>
        </div>
        <div className="modal-actions">
          <button
            type="button"
            className="btn-secondary"
            onClick={handleClose}
            disabled={isLoading}
          >
            Cancel
          </button>
          <button
            type="button"
            className="btn-danger"
            onClick={handleDelete}
            disabled={isLoading}
          >
            {isLoading ? "Removing..." : "Remove"}
          </button>
        </div>
      </div>
    </div>
  );
}
