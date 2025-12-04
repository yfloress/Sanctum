/**
 * Add Transaction Modal Component
 *
 * Modal for adding crypto transactions (buy, sell, transfer_in, transfer_out).
 * Connects directly to Zustand store for state management.
 */

import type { FormEvent } from "react";
import type { CryptoWallet } from "../../../types/index.ts";
import { POPULAR_CRYPTOS, TRANSACTION_TYPES } from "../../../types/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function AddTransactionModal() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const wallets = useCryptoStore((state) => state.wallets);
  const transactionForm = useCryptoStore((state) => state.transactionForm);

  // ==================== Store Actions ====================
  const setShowAddTransaction = useCryptoStore(
    (state) => state.setShowAddTransaction,
  );
  const setTransactionFormField = useCryptoStore(
    (state) => state.setTransactionFormField,
  );
  const resetTransactionForm = useCryptoStore(
    (state) => state.resetTransactionForm,
  );
  const selectCoinForTransaction = useCryptoStore(
    (state) => state.selectCoinForTransaction,
  );
  const addTransaction = useCryptoStore((state) => state.addTransaction);

  // ==================== Handlers ====================
  const handleAddTransaction = async (e: FormEvent) => {
    e.preventDefault();
    await addTransaction();
  };

  const handleClose = () => {
    setShowAddTransaction(false);
    resetTransactionForm();
  };

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div
        className="modal-card crypto-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <span className="modal-icon">📝</span>
          <h2>Add Transaction</h2>
        </div>
        <form onSubmit={handleAddTransaction}>
          <div className="modal-body">
            <div className="form-group">
              <label htmlFor="tx-wallet">Wallet</label>
              <select
                id="tx-wallet"
                value={transactionForm.walletId}
                onChange={(e) =>
                  setTransactionFormField("walletId", e.target.value)
                }
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
            <div className="form-group">
              <label htmlFor="tx-type">Type</label>
              <select
                id="tx-type"
                value={transactionForm.type}
                onChange={(e) =>
                  setTransactionFormField("type", e.target.value)
                }
              >
                {TRANSACTION_TYPES.filter((t) => t.value !== "swap").map(
                  (t) => (
                    <option key={t.value} value={t.value}>
                      {t.icon} {t.label}
                    </option>
                  ),
                )}
              </select>
            </div>
            <div className="form-group">
              <label>Coin</label>
              <div className="crypto-suggestions compact">
                {POPULAR_CRYPTOS.map((coin) => (
                  <button
                    key={coin.id}
                    type="button"
                    className={`crypto-suggestion ${
                      transactionForm.coinId === coin.id ? "selected" : ""
                    }`}
                    onClick={() => selectCoinForTransaction(coin)}
                  >
                    <span className="suggestion-symbol">{coin.symbol}</span>
                    <span className="suggestion-name">{coin.name}</span>
                  </button>
                ))}
              </div>
            </div>
            {transactionForm.coinId && (
              <>
                <div className="form-row">
                  <div className="form-group">
                    <label htmlFor="tx-amount">Amount</label>
                    <input
                      id="tx-amount"
                      type="number"
                      step="any"
                      value={transactionForm.amount}
                      onChange={(e) =>
                        setTransactionFormField("amount", e.target.value)
                      }
                      placeholder="0.00"
                      required
                    />
                  </div>
                  {(transactionForm.type === "buy" ||
                    transactionForm.type === "sell") && (
                    <div className="form-group">
                      <label htmlFor="tx-price">Price per coin ($)</label>
                      <input
                        id="tx-price"
                        type="number"
                        step="any"
                        value={transactionForm.price}
                        onChange={(e) =>
                          setTransactionFormField("price", e.target.value)
                        }
                        placeholder="0.00"
                      />
                    </div>
                  )}
                </div>
                <div className="form-row">
                  <div className="form-group">
                    <label htmlFor="tx-fee">Fee ($)</label>
                    <input
                      id="tx-fee"
                      type="number"
                      step="any"
                      value={transactionForm.fee}
                      onChange={(e) =>
                        setTransactionFormField("fee", e.target.value)
                      }
                      placeholder="0.00"
                    />
                  </div>
                  <div className="form-group">
                    <label htmlFor="tx-date">Date</label>
                    <input
                      id="tx-date"
                      type="date"
                      value={transactionForm.date}
                      onChange={(e) =>
                        setTransactionFormField("date", e.target.value)
                      }
                    />
                  </div>
                </div>
                <div className="form-group">
                  <label htmlFor="tx-notes">Notes</label>
                  <input
                    id="tx-notes"
                    type="text"
                    value={transactionForm.notes}
                    onChange={(e) =>
                      setTransactionFormField("notes", e.target.value)
                    }
                    placeholder="Optional notes..."
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
              disabled={!transactionForm.coinId || isLoading}
            >
              {isLoading ? "Adding..." : "Add Transaction"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
