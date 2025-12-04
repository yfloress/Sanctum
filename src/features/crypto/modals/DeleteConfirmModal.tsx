/**
 * Delete Confirm Modal Component
 *
 * Generic confirmation modal for delete operations.
 * Handles wallet, transaction, and holding deletions.
 * Connects directly to Zustand store for state management.
 */

import { useCryptoStore } from "../../../stores/cryptoStore.ts";

type DeleteType = "wallet" | "transaction" | "holding";

interface DeleteConfig {
  title: string;
  message: string;
  warning: string;
  confirmLabel: string;
  confirmingLabel: string;
}

const DELETE_CONFIGS: Record<DeleteType, DeleteConfig> = {
  wallet: {
    title: "Delete Wallet",
    message: "Are you sure you want to delete this wallet?",
    warning:
      "All transactions in this wallet will be deleted. This action cannot be undone.",
    confirmLabel: "Delete Wallet",
    confirmingLabel: "Deleting...",
  },
  transaction: {
    title: "Delete Transaction",
    message: "Are you sure you want to delete this transaction?",
    warning: "This action cannot be undone.",
    confirmLabel: "Delete",
    confirmingLabel: "Deleting...",
  },
  holding: {
    title: "Remove Holding",
    message: "Are you sure you want to remove this holding?",
    warning: "This action cannot be undone.",
    confirmLabel: "Remove",
    confirmingLabel: "Removing...",
  },
};

export function DeleteConfirmModal() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const walletToDelete = useCryptoStore((state) => state.walletToDelete);
  const transactionToDelete = useCryptoStore(
    (state) => state.transactionToDelete,
  );
  const holdingToDelete = useCryptoStore((state) => state.holdingToDelete);

  // ==================== Store Actions ====================
  const setWalletToDelete = useCryptoStore((state) => state.setWalletToDelete);
  const setTransactionToDelete = useCryptoStore(
    (state) => state.setTransactionToDelete,
  );
  const setHoldingToDelete = useCryptoStore(
    (state) => state.setHoldingToDelete,
  );
  const deleteWallet = useCryptoStore((state) => state.deleteWallet);
  const deleteTransaction = useCryptoStore((state) => state.deleteTransaction);
  const deleteHolding = useCryptoStore((state) => state.deleteHolding);

  // ==================== Determine Active Delete Type ====================
  let activeType: DeleteType | null = null;
  let handleClose: () => void;
  let handleConfirm: () => void;

  if (walletToDelete !== null) {
    activeType = "wallet";
    handleClose = () => setWalletToDelete(null);
    handleConfirm = deleteWallet;
  } else if (transactionToDelete !== null) {
    activeType = "transaction";
    handleClose = () => setTransactionToDelete(null);
    handleConfirm = deleteTransaction;
  } else if (holdingToDelete !== null) {
    activeType = "holding";
    handleClose = () => setHoldingToDelete(null);
    handleConfirm = deleteHolding;
  } else {
    // No active delete modal
    return null;
  }

  const config = DELETE_CONFIGS[activeType];

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div
        className="modal-card delete-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <span className="modal-icon">⚠️</span>
          <h2>{config.title}</h2>
        </div>
        <div className="modal-body">
          <p>{config.message}</p>
          <p className="modal-warning">{config.warning}</p>
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
            onClick={handleConfirm}
            disabled={isLoading}
          >
            {isLoading ? config.confirmingLabel : config.confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
