/**
 * Crypto View Component
 *
 * Displays cryptocurrency portfolio, wallets, and watchlist.
 * Consumes state directly from Zustand stores - no props needed.
 */

import type { FormEvent } from "react";
import type {
  AggregatedAsset,
  CryptoWallet,
  CryptoTransaction,
} from "../../types";
import {
  POPULAR_CRYPTOS,
  WALLET_CATEGORIES,
  WALLET_ICONS,
  TRANSACTION_TYPES,
  MAX_TRACKED_COINS,
} from "../../types";
import { formatUSD, formatCryptoAmount, formatDate } from "../../utils";
import { useCryptoStore } from "../../stores/cryptoStore";

export function CryptoView() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const error = useCryptoStore((state) => state.error);
  const subTab = useCryptoStore((state) => state.subTab);
  const prices = useCryptoStore((state) => state.prices);
  const watchlist = useCryptoStore((state) => state.watchlist);
  const searchQuery = useCryptoStore((state) => state.searchQuery);
  const wallets = useCryptoStore((state) => state.wallets);
  const selectedWallet = useCryptoStore((state) => state.selectedWallet);
  const walletTransactions = useCryptoStore(
    (state) => state.walletTransactions,
  );

  // Modal states
  const showAddCrypto = useCryptoStore((state) => state.showAddCrypto);
  const showAddWallet = useCryptoStore((state) => state.showAddWallet);
  const showAddTransaction = useCryptoStore(
    (state) => state.showAddTransaction,
  );
  const showTransferModal = useCryptoStore((state) => state.showTransferModal);
  const showSwapModal = useCryptoStore((state) => state.showSwapModal);
  const showAddHolding = useCryptoStore((state) => state.showAddHolding);
  const walletToDelete = useCryptoStore((state) => state.walletToDelete);
  const transactionToDelete = useCryptoStore(
    (state) => state.transactionToDelete,
  );
  const holdingToDelete = useCryptoStore((state) => state.holdingToDelete);

  // Form states
  const walletForm = useCryptoStore((state) => state.walletForm);
  const transactionForm = useCryptoStore((state) => state.transactionForm);
  const transferForm = useCryptoStore((state) => state.transferForm);
  const swapForm = useCryptoStore((state) => state.swapForm);
  const holdingForm = useCryptoStore((state) => state.holdingForm);

  // ==================== Store Actions ====================
  const setSubTab = useCryptoStore((state) => state.setSubTab);
  const setSearchQuery = useCryptoStore((state) => state.setSearchQuery);
  const fetchPrices = useCryptoStore((state) => state.fetchPrices);
  const addToWatchlist = useCryptoStore((state) => state.addToWatchlist);
  const removeFromWatchlist = useCryptoStore(
    (state) => state.removeFromWatchlist,
  );
  const selectWallet = useCryptoStore((state) => state.selectWallet);
  const addWallet = useCryptoStore((state) => state.addWallet);
  const deleteWallet = useCryptoStore((state) => state.deleteWallet);
  const addTransaction = useCryptoStore((state) => state.addTransaction);
  const addTransfer = useCryptoStore((state) => state.addTransfer);
  const addSwap = useCryptoStore((state) => state.addSwap);
  const deleteTransaction = useCryptoStore((state) => state.deleteTransaction);
  const addHolding = useCryptoStore((state) => state.addHolding);
  const deleteHolding = useCryptoStore((state) => state.deleteHolding);

  // Modal controls
  const setShowAddCrypto = useCryptoStore((state) => state.setShowAddCrypto);
  const setShowAddWallet = useCryptoStore((state) => state.setShowAddWallet);
  const setShowAddTransaction = useCryptoStore(
    (state) => state.setShowAddTransaction,
  );
  const setShowTransferModal = useCryptoStore(
    (state) => state.setShowTransferModal,
  );
  const setShowSwapModal = useCryptoStore((state) => state.setShowSwapModal);
  const setShowAddHolding = useCryptoStore((state) => state.setShowAddHolding);
  const setWalletToDelete = useCryptoStore((state) => state.setWalletToDelete);
  const setTransactionToDelete = useCryptoStore(
    (state) => state.setTransactionToDelete,
  );
  const setHoldingToDelete = useCryptoStore(
    (state) => state.setHoldingToDelete,
  );

  // Form field setters
  const setWalletFormField = useCryptoStore(
    (state) => state.setWalletFormField,
  );
  const setTransactionFormField = useCryptoStore(
    (state) => state.setTransactionFormField,
  );
  const setTransferFormField = useCryptoStore(
    (state) => state.setTransferFormField,
  );
  const setSwapFormField = useCryptoStore((state) => state.setSwapFormField);
  const setHoldingFormField = useCryptoStore(
    (state) => state.setHoldingFormField,
  );
  const resetTransactionForm = useCryptoStore(
    (state) => state.resetTransactionForm,
  );
  const resetTransferForm = useCryptoStore((state) => state.resetTransferForm);
  const resetSwapForm = useCryptoStore((state) => state.resetSwapForm);
  const resetHoldingForm = useCryptoStore((state) => state.resetHoldingForm);
  const selectCoinForTransaction = useCryptoStore(
    (state) => state.selectCoinForTransaction,
  );
  const selectCoinForHolding = useCryptoStore(
    (state) => state.selectCoinForHolding,
  );

  // Computed getters
  const getFilteredSuggestions = useCryptoStore(
    (state) => state.getFilteredSuggestions,
  );
  const getEnrichedPortfolio = useCryptoStore(
    (state) => state.getEnrichedPortfolio,
  );
  const getEnrichedWalletHoldings = useCryptoStore(
    (state) => state.getEnrichedWalletHoldings,
  );
  const getPortfolioTotals = useCryptoStore(
    (state) => state.getPortfolioTotals,
  );

  // ==================== Computed Values ====================
  const filteredSuggestions = getFilteredSuggestions();
  const enrichedPortfolio = getEnrichedPortfolio();
  const enrichedWalletHoldings = getEnrichedWalletHoldings();
  const portfolioTotals = getPortfolioTotals();

  // ==================== Helpers ====================
  const getTransactionTypeLabel = (type: string) => {
    const found = TRANSACTION_TYPES.find((t) => t.value === type);
    return found ? `${found.icon} ${found.label}` : type;
  };

  const getWalletCategoryLabel = (category: string) => {
    const found = WALLET_CATEGORIES.find((c) => c.value === category);
    return found ? found.label : category;
  };

  // ==================== Form Handlers ====================
  const handleAddWallet = async (e: FormEvent) => {
    e.preventDefault();
    await addWallet();
  };

  const handleAddTransaction = async (e: FormEvent) => {
    e.preventDefault();
    await addTransaction();
  };

  const handleAddTransfer = async (e: FormEvent) => {
    e.preventDefault();
    await addTransfer();
  };

  const handleAddSwap = async (e: FormEvent) => {
    e.preventDefault();
    await addSwap();
  };

  const handleAddHolding = async (e: FormEvent) => {
    e.preventDefault();
    await addHolding();
  };

  return (
    <div className="crypto-page">
      <div className="crypto-header">
        <h1 className="page-title">Cryptocurrency</h1>
        <div className="crypto-actions">
          <button
            className="btn-icon"
            onClick={fetchPrices}
            disabled={isLoading}
            title="Refresh prices"
          >
            {isLoading ? "⏳" : "↻"}
          </button>
        </div>
      </div>

      {error && <div className="message error crypto-error">{error}</div>}

      {/* Sub-tabs for Overview and Wallets */}
      <div className="crypto-subtabs">
        <button
          className={`crypto-subtab ${subTab === "overview" ? "active" : ""}`}
          onClick={() => {
            setSubTab("overview");
            selectWallet(null);
          }}
        >
          📊 Overview
        </button>
        <button
          className={`crypto-subtab ${subTab === "wallets" ? "active" : ""}`}
          onClick={() => setSubTab("wallets")}
        >
          👛 Wallets
        </button>
      </div>

      {/* ==================== Overview Sub-Tab ==================== */}
      {subTab === "overview" && (
        <>
          {/* Portfolio Summary */}
          <div className="portfolio-section">
            <div className="section-header">
              <h2 className="section-title">Total Portfolio</h2>
              <div className="portfolio-total">
                <span className="portfolio-total-label">Total Value</span>
                <span className="portfolio-total-value">
                  ${formatUSD(portfolioTotals.totalValue)}
                </span>
                <span
                  className={`portfolio-total-pnl ${portfolioTotals.totalPnl >= 0 ? "positive" : "negative"}`}
                >
                  {portfolioTotals.totalPnl >= 0 ? "+" : ""}$
                  {formatUSD(portfolioTotals.totalPnl)} (
                  {portfolioTotals.totalPnlPercentage >= 0 ? "+" : ""}
                  {portfolioTotals.totalPnlPercentage.toFixed(2)}%)
                </span>
              </div>
            </div>

            {enrichedPortfolio.length === 0 ? (
              <div className="portfolio-empty">
                <span className="portfolio-empty-icon">💼</span>
                <p>
                  No holdings yet. Add a wallet and start tracking your
                  portfolio!
                </p>
                <button
                  className="btn-secondary"
                  onClick={() => setSubTab("wallets")}
                >
                  Go to Wallets
                </button>
              </div>
            ) : (
              <div className="portfolio-grid">
                {enrichedPortfolio.map((asset: AggregatedAsset) => (
                  <div key={asset.coin_id} className="portfolio-card">
                    <div className="portfolio-card-header">
                      <span className="portfolio-symbol">{asset.symbol}</span>
                      <span
                        className={`portfolio-pnl ${asset.unrealized_pnl >= 0 ? "positive" : "negative"}`}
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
                    <div className="portfolio-details">
                      <span>
                        Avg: ${formatCryptoAmount(asset.avg_buy_price, 6)}
                      </span>
                      <span
                        className={
                          asset.unrealized_pnl >= 0 ? "positive" : "negative"
                        }
                      >
                        {asset.unrealized_pnl >= 0 ? "+" : ""}$
                        {formatUSD(asset.unrealized_pnl)}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Watchlist Section */}
          <div className="watchlist-section">
            <div className="section-header">
              <h2 className="section-title">Watchlist</h2>
              <div className="crypto-actions">
                <span className="crypto-count">
                  {watchlist.length}/{MAX_TRACKED_COINS}
                </span>
                <button
                  className="btn-icon"
                  onClick={() => setShowAddCrypto(true)}
                  disabled={watchlist.length >= MAX_TRACKED_COINS}
                  title="Track new coin"
                >
                  +
                </button>
              </div>
            </div>

            {isLoading && prices.length === 0 ? (
              <div className="crypto-loading">
                <div className="loader" />
                <p>Loading prices...</p>
              </div>
            ) : prices.length === 0 ? (
              <div className="crypto-empty">
                <span className="crypto-empty-icon">📊</span>
                <p>Click refresh to load prices</p>
                <button className="btn-secondary" onClick={fetchPrices}>
                  ↻ Load Prices
                </button>
              </div>
            ) : (
              <div className="crypto-grid">
                {prices
                  .filter((asset) => watchlist.includes(asset.id))
                  .map((asset) => (
                    <div key={asset.id} className="crypto-card">
                      <button
                        className="crypto-remove"
                        onClick={() => removeFromWatchlist(asset.id)}
                        title="Remove from watchlist"
                      >
                        ×
                      </button>
                      <div className="crypto-card-header">
                        <div className="crypto-info">
                          <span className="crypto-symbol">{asset.symbol}</span>
                          <span className="crypto-name">{asset.name}</span>
                        </div>
                        <div
                          className={`crypto-change ${asset.price_change_percentage_24h >= 0 ? "positive" : "negative"}`}
                        >
                          {asset.price_change_percentage_24h >= 0 ? "▲" : "▼"}{" "}
                          {Math.abs(asset.price_change_percentage_24h).toFixed(
                            2,
                          )}
                          %
                        </div>
                      </div>
                      <div className="crypto-price">
                        $
                        {asset.current_price.toLocaleString(undefined, {
                          minimumFractionDigits: 2,
                          maximumFractionDigits:
                            asset.current_price < 1 ? 6 : 2,
                        })}
                      </div>
                      <div className="crypto-updated">
                        Updated:{" "}
                        {new Date(asset.last_updated).toLocaleTimeString()}
                      </div>
                    </div>
                  ))}
              </div>
            )}
          </div>
        </>
      )}

      {/* ==================== Wallets Sub-Tab ==================== */}
      {subTab === "wallets" && !selectedWallet && (
        <>
          <div className="section-header">
            <h2 className="section-title">My Wallets</h2>
            <button
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
      )}

      {/* ==================== Wallet Detail View ==================== */}
      {subTab === "wallets" && selectedWallet && (
        <>
          <div className="wallet-detail-header">
            <button className="btn-back" onClick={() => selectWallet(null)}>
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
                className="btn-primary"
                onClick={() => {
                  setTransactionFormField("walletId", selectedWallet.id);
                  setShowAddTransaction(true);
                }}
              >
                + Add Transaction
              </button>
              <button
                className="btn-secondary"
                onClick={() => {
                  setTransferFormField("fromWalletId", selectedWallet.id);
                  setShowTransferModal(true);
                }}
              >
                ↔ Transfer
              </button>
              <button
                className="btn-secondary"
                onClick={() => {
                  setSwapFormField("walletId", selectedWallet.id);
                  setShowSwapModal(true);
                }}
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
                        className={`portfolio-pnl ${asset.unrealized_pnl >= 0 ? "positive" : "negative"}`}
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
                      <div className="transaction-date">
                        {formatDate(tx.date)}
                      </div>
                    </div>
                    <div className="transaction-actions">
                      <div
                        className={`transaction-amount ${tx.type === "buy" || tx.type === "transfer_in" ? "income" : "expense"}`}
                      >
                        {tx.type === "buy" || tx.type === "transfer_in"
                          ? "+"
                          : "-"}
                        {formatCryptoAmount(tx.amount)} {tx.symbol}
                      </div>
                      <button
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
      )}

      <div className="crypto-disclaimer">
        <p>
          💡 Data provided by CoinGecko. Prices are for informational purposes
          only.
        </p>
      </div>

      {/* ==================== Modals ==================== */}

      {/* Add Wallet Modal */}
      {showAddWallet && (
        <div className="modal-overlay" onClick={() => setShowAddWallet(false)}>
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
                        className={`icon-option ${walletForm.icon === icon ? "selected" : ""}`}
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
                  onClick={() => setShowAddWallet(false)}
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
      )}

      {/* Add Transaction Modal */}
      {showAddTransaction && (
        <div
          className="modal-overlay"
          onClick={() => setShowAddTransaction(false)}
        >
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
                        className={`crypto-suggestion ${transactionForm.coinId === coin.id ? "selected" : ""}`}
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
                  onClick={() => {
                    setShowAddTransaction(false);
                    resetTransactionForm();
                  }}
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
      )}

      {/* Transfer Modal */}
      {showTransferModal && (
        <div
          className="modal-overlay"
          onClick={() => setShowTransferModal(false)}
        >
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
                          (w: CryptoWallet) =>
                            w.id !== transferForm.fromWalletId,
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
                        className={`crypto-suggestion ${transferForm.coinId === coin.id ? "selected" : ""}`}
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
                      onChange={(e) =>
                        setTransferFormField("fee", e.target.value)
                      }
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
                    onChange={(e) =>
                      setTransferFormField("date", e.target.value)
                    }
                  />
                </div>
              </div>
              <div className="modal-actions">
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={() => {
                    setShowTransferModal(false);
                    resetTransferForm();
                  }}
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
      )}

      {/* Swap Modal */}
      {showSwapModal && (
        <div className="modal-overlay" onClick={() => setShowSwapModal(false)}>
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
                    onChange={(e) =>
                      setSwapFormField("walletId", e.target.value)
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
                <div className="swap-section">
                  <h4>From (Sell)</h4>
                  <div className="crypto-suggestions compact">
                    {POPULAR_CRYPTOS.map((coin) => (
                      <button
                        key={coin.id}
                        type="button"
                        className={`crypto-suggestion ${swapForm.fromCoinId === coin.id ? "selected" : ""}`}
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
                        className={`crypto-suggestion ${swapForm.toCoinId === coin.id ? "selected" : ""}`}
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
                    onChange={(e) =>
                      setSwapFormField("toAmount", e.target.value)
                    }
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
                  onClick={() => {
                    setShowSwapModal(false);
                    resetSwapForm();
                  }}
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
      )}

      {/* Add to Watchlist Modal */}
      {showAddCrypto && (
        <div className="modal-overlay" onClick={() => setShowAddCrypto(false)}>
          <div
            className="modal-card crypto-modal"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-header">
              <span className="modal-icon">₿</span>
              <h2>Add to Watchlist</h2>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label htmlFor="crypto-search">Search or enter coin ID</label>
                <input
                  id="crypto-search"
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="e.g. bitcoin, eth, solana..."
                  autoFocus
                />
              </div>
              <div className="crypto-suggestions">
                {filteredSuggestions.length > 0 ? (
                  filteredSuggestions.map((coin) => (
                    <button
                      key={coin.id}
                      className="crypto-suggestion"
                      onClick={() => addToWatchlist(coin.id)}
                    >
                      <span className="suggestion-symbol">{coin.symbol}</span>
                      <span className="suggestion-name">{coin.name}</span>
                    </button>
                  ))
                ) : searchQuery.trim() ? (
                  <button
                    className="crypto-suggestion custom"
                    onClick={() => addToWatchlist(searchQuery)}
                  >
                    <span className="suggestion-symbol">+</span>
                    <span className="suggestion-name">
                      Add "{searchQuery}" as custom coin
                    </span>
                  </button>
                ) : (
                  <p className="suggestions-empty">
                    All popular coins are already tracked
                  </p>
                )}
              </div>
            </div>
            <div className="modal-actions">
              <button
                type="button"
                className="btn-secondary"
                onClick={() => {
                  setShowAddCrypto(false);
                  setSearchQuery("");
                }}
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Wallet Confirmation Modal */}
      {walletToDelete !== null && (
        <div className="modal-overlay" onClick={() => setWalletToDelete(null)}>
          <div
            className="modal-card delete-modal"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-header">
              <span className="modal-icon">⚠️</span>
              <h2>Delete Wallet</h2>
            </div>
            <div className="modal-body">
              <p>Are you sure you want to delete this wallet?</p>
              <p className="modal-warning">
                All transactions in this wallet will be deleted. This action
                cannot be undone.
              </p>
            </div>
            <div className="modal-actions">
              <button
                type="button"
                className="btn-secondary"
                onClick={() => setWalletToDelete(null)}
                disabled={isLoading}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                onClick={deleteWallet}
                disabled={isLoading}
              >
                {isLoading ? "Deleting..." : "Delete Wallet"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Crypto Transaction Confirmation Modal */}
      {transactionToDelete !== null && (
        <div
          className="modal-overlay"
          onClick={() => setTransactionToDelete(null)}
        >
          <div
            className="modal-card delete-modal"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-header">
              <span className="modal-icon">⚠️</span>
              <h2>Delete Transaction</h2>
            </div>
            <div className="modal-body">
              <p>Are you sure you want to delete this transaction?</p>
              <p className="modal-warning">This action cannot be undone.</p>
            </div>
            <div className="modal-actions">
              <button
                type="button"
                className="btn-secondary"
                onClick={() => setTransactionToDelete(null)}
                disabled={isLoading}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                onClick={deleteTransaction}
                disabled={isLoading}
              >
                {isLoading ? "Deleting..." : "Delete"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Legacy Add Holding Modal */}
      {showAddHolding && (
        <div className="modal-overlay" onClick={() => setShowAddHolding(false)}>
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
                        className={`crypto-suggestion ${holdingForm.coinId === coin.id ? "selected" : ""}`}
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
                        <label htmlFor="holding-price">
                          Purchase Price ($)
                        </label>
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
                  onClick={() => {
                    setShowAddHolding(false);
                    resetHoldingForm();
                  }}
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
      )}

      {/* Delete Legacy Holding Confirmation Modal */}
      {holdingToDelete !== null && (
        <div className="modal-overlay" onClick={() => setHoldingToDelete(null)}>
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
                onClick={() => setHoldingToDelete(null)}
                disabled={isLoading}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                onClick={deleteHolding}
                disabled={isLoading}
              >
                {isLoading ? "Removing..." : "Remove"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
