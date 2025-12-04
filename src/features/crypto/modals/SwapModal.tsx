/**
 * Swap Modal Component
 *
 * Modal for recording crypto swaps within a wallet.
 * Connects directly to Zustand store following project architecture.
 */

import type { FormEvent } from "react";
import type { CryptoWallet } from "../../../types/index.ts";
import { POPULAR_CRYPTOS } from "../../../types/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function SwapModal() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const wallets = useCryptoStore((state) => state.wallets);
  const swapForm = useCryptoStore((state) => state.swapForm);

  // ==================== Store Actions ====================
  const setShowSwapModal = useCryptoStore((state) => state.setShowSwapModal);
  const setSwapFormField = useCryptoStore((state) => state.setSwapFormField);
  const resetSwapForm = useCryptoStore((state) => state.resetSwapForm);
  const addSwap = useCryptoStore((state) => state.addSwap);

  // ==================== Handlers ====================
  const handleAddSwap = async (e: FormEvent) => {
    e.preventDefault();
    await addSwap();
  };

  const handleClose = () => {
    setShowSwapModal(false);
    resetSwapForm();
  };

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div
        className="modal-card crypto-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <span className="modal-icon">🔄</span>
          <h2>Record Swap</h2>
        </div>
        <form onSubmit={handleAddSwap}>
          <div className="modal-body">
            <div className="form-group">
              <label htmlFor="swap-wallet">Wallet</label>
              <select
                id="swap-wallet"
                value={swapForm.walletId}
                onChange={(e) => setSwapFormField("walletId", e.target.value)}
                required
              >
                <option value="">Select wallet...</option>
                {wallets.map((w: CryptoWallet) => (
                  <option key={w.id} value={w.id}>
                    {w.icon} {w.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="swap-section">
              <h4>From (Sell)</h4>
              <div className="crypto-suggestions compact">
                {POPULAR_CRYPTOS.map((coin) => (
                  <button
                    key={coin.id}
                    type="button"
                    className={`crypto-suggestion ${
                      swapForm.fromCoinId === coin.id ? "selected" : ""
                    }`}
                    onClick={() => {
                      setSwapFormField("fromCoinId", coin.id);
                      setSwapFormField("fromSymbol", coin.symbol);
                    }}
                  >
                    <span className="suggestion-symbol">{coin.symbol}</span>
                  </button>
                ))}
              </div>
              <input
                type="number"
                step="any"
                value={swapForm.fromAmount}
                onChange={(e) =>
                  setSwapFormField("fromAmount", e.target.value)
                }
                placeholder="Amount to swap"
                required
              />
            </div>
            <div className="swap-arrow">⬇️</div>
            <div className="swap-section">
              <h4>To (Receive)</h4>
              <div className="crypto-suggestions compact">
                {POPULAR_CRYPTOS.filter(
                  (c) => c.id !== swapForm.fromCoinId,
                ).map((coin) => (
                  <button
                    key={coin.id}
                    type="button"
                    className={`crypto-suggestion ${
                      swapForm.toCoinId === coin.id ? "selected" : ""
                    }`}
                    onClick={() => {
                      setSwapFormField("toCoinId", coin.id);
                      setSwapFormField("toSymbol", coin.symbol);
                    }}
                  >
                    <span className="suggestion-symbol">{coin.symbol}</span>
                  </button>
                ))}
              </div>
              <input
                type="number"
                step="any"
                value={swapForm.toAmount}
                onChange={(e) => setSwapFormField("toAmount", e.target.value)}
                placeholder="Amount received"
                required
              />
            </div>
            <div className="form-row">
              <div className="form-group">
                <label htmlFor="swap-fee">Fee ($)</label>
                <input
                  id="swap-fee"
                  type="number"
                  step="any"
                  value={swapForm.fee}
                  onChange={(e) => setSwapFormField("fee", e.target.value)}
                  placeholder="0.00"
                />
              </div>
              <div className="form-group">
                <label htmlFor="swap-date">Date</label>
                <input
                  id="swap-date"
                  type="date"
                  value={swapForm.date}
                  onChange={(e) => setSwapFormField("date", e.target.value)}
                />
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
              disabled={
                !swapForm.fromCoinId || !swapForm.toCoinId || isLoading
              }
            >
              {isLoading ? "Recording..." : "Record Swap"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
