/**
 * Transfer Modal Component
 *
 * Modal for transferring crypto between wallets.
 * Connects directly to Zustand store following project architecture.
 */

import type { FormEvent } from "react";
import type { CryptoWallet } from "../../../types/index.ts";
import { POPULAR_CRYPTOS } from "../../../types/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function TransferModal() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const wallets = useCryptoStore((state) => state.wallets);
  const transferForm = useCryptoStore((state) => state.transferForm);

  // ==================== Store Actions ====================
  const setShowTransferModal = useCryptoStore(
    (state) => state.setShowTransferModal,
  );
  const setTransferFormField = useCryptoStore(
    (state) => state.setTransferFormField,
  );
  const resetTransferForm = useCryptoStore((state) => state.resetTransferForm);
  const addTransfer = useCryptoStore((state) => state.addTransfer);

  // ==================== Handlers ====================
  const handleAddTransfer = async (e: FormEvent) => {
    e.preventDefault();
    await addTransfer();
  };

  const handleClose = () => {
    setShowTransferModal(false);
    resetTransferForm();
  };

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div
        className="modal-card crypto-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <span className="modal-icon">↔️</span>
          <h2>Transfer Between Wallets</h2>
        </div>
        <form onSubmit={handleAddTransfer}>
          <div className="modal-body">
            <div className="form-row">
              <div className="form-group">
                <label htmlFor="transfer-from">From Wallet</label>
                <select
                  id="transfer-from"
                  value={transferForm.fromWalletId}
                  onChange={(e) =>
                    setTransferFormField("fromWalletId", e.target.value)
                  }
                  required
                >
                  <option value="">Select...</option>
                  {wallets.map((w: CryptoWallet) => (
                    <option key={w.id} value={w.id}>
                      {w.icon} {w.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-group">
                <label htmlFor="transfer-to">To Wallet</label>
                <select
                  id="transfer-to"
                  value={transferForm.toWalletId}
                  onChange={(e) =>
                    setTransferFormField("toWalletId", e.target.value)
                  }
                  required
                >
                  <option value="">Select...</option>
                  {wallets
                    .filter(
                      (w: CryptoWallet) => w.id !== transferForm.fromWalletId,
                    )
                    .map((w: CryptoWallet) => (
                      <option key={w.id} value={w.id}>
                        {w.icon} {w.name}
                      </option>
                    ))}
                </select>
              </div>
            </div>
            <div className="form-group">
              <label>Coin</label>
              <div className="crypto-suggestions compact">
                {POPULAR_CRYPTOS.map((coin) => (
                  <button
                    key={coin.id}
                    type="button"
                    className={`crypto-suggestion ${
                      transferForm.coinId === coin.id ? "selected" : ""
                    }`}
                    onClick={() => {
                      setTransferFormField("coinId", coin.id);
                      setTransferFormField("symbol", coin.symbol);
                    }}
                  >
                    <span className="suggestion-symbol">{coin.symbol}</span>
                  </button>
                ))}
              </div>
            </div>
            <div className="form-row">
              <div className="form-group">
                <label htmlFor="transfer-amount">Amount</label>
                <input
                  id="transfer-amount"
                  type="number"
                  step="any"
                  value={transferForm.amount}
                  onChange={(e) =>
                    setTransferFormField("amount", e.target.value)
                  }
                  placeholder="0.00"
                  required
                />
              </div>
              <div className="form-group">
                <label htmlFor="transfer-fee">Network Fee</label>
                <input
                  id="transfer-fee"
                  type="number"
                  step="any"
                  value={transferForm.fee}
                  onChange={(e) => setTransferFormField("fee", e.target.value)}
                  placeholder="0.00"
                />
              </div>
            </div>
            <div className="form-group">
              <label htmlFor="transfer-date">Date</label>
              <input
                id="transfer-date"
                type="date"
                value={transferForm.date}
                onChange={(e) => setTransferFormField("date", e.target.value)}
              />
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
              disabled={!transferForm.coinId || isLoading}
            >
              {isLoading ? "Transferring..." : "Record Transfer"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
