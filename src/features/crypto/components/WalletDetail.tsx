/**
 * WalletDetail Component
 *
 * Displays detailed view of a selected wallet including holdings and transactions.
 * Connects directly to Zustand store for state management.
 */

import type { AggregatedAsset, CryptoTransaction } from "../../../types/index.ts";
import { TRANSACTION_TYPES, WALLET_CATEGORIES } from "../../../types/index.ts";
import { formatCryptoAmount, formatDate, formatUSD } from "../../../utils/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function WalletDetail() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const selectedWallet = useCryptoStore((state) => state.selectedWallet);
  const walletTransactions = useCryptoStore(
    (state) => state.walletTransactions,
  );

  // ==================== Store Actions ====================
  const selectWallet = useCryptoStore((state) => state.selectWallet);
  const setShowAddTransaction = useCryptoStore(
    (state) => state.setShowAddTransaction,
  );
  const setShowTransferModal = useCryptoStore(
    (state) => state.setShowTransferModal,
  );
  const setShowSwapModal = useCryptoStore((state) => state.setShowSwapModal);
  const setTransactionFormField = useCryptoStore(
    (state) => state.setTransactionFormField,
  );
  const setTransferFormField = useCryptoStore(
    (state) => state.setTransferFormField,
  );
  const setSwapFormField = useCryptoStore((state) => state.setSwapFormField);
  const setTransactionToDelete = useCryptoStore(
    (state) => state.setTransactionToDelete,
  );

  // Computed getters
  const getEnrichedWalletHoldings = useCryptoStore(
    (state) => state.getEnrichedWalletHoldings,
  );

  // ==================== Computed Values ====================
  const enrichedWalletHoldings = getEnrichedWalletHoldings();

  // ==================== Helpers ====================
  const getTransactionTypeLabel = (type: string) => {
    const found = TRANSACTION_TYPES.find((t) => t.value === type);
    return found ? `${found.icon} ${found.label}` : type;
  };

  const getWalletCategoryLabel = (category: string) => {
    const found = WALLET_CATEGORIES.find((c) => c.value === category);
    return found ? found.label : category;
  };

  // ==================== Handlers ====================
  const handleAddTransaction = () => {
    if (selectedWallet) {
      setTransactionFormField("walletId", selectedWallet.id);
      setShowAddTransaction(true);
    }
  };

  const handleTransfer = () => {
    if (selectedWallet) {
      setTransferFormField("fromWalletId", selectedWallet.id);
      setShowTransferModal(true);
    }
  };

  const handleSwap = () => {
    if (selectedWallet) {
      setSwapFormField("walletId", selectedWallet.id);
      setShowSwapModal(true);
    }
  };

  // Don't render if no wallet is selected
  if (!selectedWallet) {
    return null;
  }

  return (
    <>
      <div className="wallet-detail-header">
        <button
          type="button"
          className="btn-back"
          onClick={() => selectWallet(null)}
        >
          ← Back to Wallets
        </button>
        <div className="wallet-detail-info">
          <span className="wallet-detail-icon">
            {selectedWallet.icon || "👛"}
          </span>
          <h2>{selectedWallet.name}</h2>
          <span className="wallet-detail-category">
            {getWalletCategoryLabel(selectedWallet.category)}
          </span>
        </div>
        <div className="wallet-detail-actions">
          <button
            type="button"
            className="btn-primary"
            onClick={handleAddTransaction}
          >
            + Add Transaction
          </button>
          <button
            type="button"
            className="btn-secondary"
            onClick={handleTransfer}
          >
            ↔ Transfer
          </button>
          <button
            type="button"
            className="btn-secondary"
            onClick={handleSwap}
          >
            🔄 Swap
          </button>
        </div>
      </div>

      {/* Wallet Holdings */}
      <div className="wallet-holdings">
        <h3 className="section-title">Holdings</h3>
        {enrichedWalletHoldings.length === 0 ? (
          <p className="empty-state">No holdings in this wallet yet</p>
        ) : (
          <div className="portfolio-grid">
            {enrichedWalletHoldings.map((asset: AggregatedAsset) => (
              <div key={asset.coin_id} className="portfolio-card">
                <div className="portfolio-card-header">
                  <span className="portfolio-symbol">{asset.symbol}</span>
                  <span
                    className={`portfolio-pnl ${
                      asset.unrealized_pnl >= 0 ? "positive" : "negative"
                    }`}
                  >
                    {asset.unrealized_pnl >= 0 ? "▲" : "▼"}{" "}
                    {Math.abs(asset.unrealized_pnl_percentage).toFixed(2)}%
                  </span>
                </div>
                <div className="portfolio-amount">
                  {formatCryptoAmount(asset.total_amount)} {asset.symbol}
                </div>
                <div className="portfolio-value">
                  ${formatUSD(asset.current_value)}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Wallet Transactions */}
      <div className="wallet-transactions">
        <h3 className="section-title">Transaction History</h3>
        {walletTransactions.length === 0 ? (
          <p className="empty-state">No transactions recorded</p>
        ) : (
          <div className="transactions-list">
            {walletTransactions.map((tx: CryptoTransaction) => (
              <div key={tx.id} className="transaction-item crypto-tx-item">
                <div className="transaction-info">
                  <div className="transaction-category">
                    {getTransactionTypeLabel(tx.type)} {tx.symbol}
                  </div>
                  <div className="transaction-description">
                    {formatCryptoAmount(tx.amount)} {tx.symbol}
                    {tx.price_per_coin &&
                      ` @ $${formatCryptoAmount(tx.price_per_coin, 6)}`}
                  </div>
                  <div className="transaction-date">{formatDate(tx.date)}</div>
                </div>
                <div className="transaction-actions">
                  <div
                    className={`transaction-amount ${
                      tx.type === "buy" || tx.type === "transfer_in"
                        ? "income"
                        : "expense"
                    }`}
                  >
                    {tx.type === "buy" || tx.type === "transfer_in" ? "+" : "-"}
                    {formatCryptoAmount(tx.amount)} {tx.symbol}
                  </div>
                  <button
                    type="button"
                    className="btn-delete"
                    onClick={() => setTransactionToDelete(tx.id)}
                    disabled={isLoading}
                    aria-label="Delete transaction"
                  >
                    🗑️
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
