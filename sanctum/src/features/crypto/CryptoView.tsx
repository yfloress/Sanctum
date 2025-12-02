import type { UseCryptoReturn } from "../../hooks/useCrypto";
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

interface CryptoViewProps {
  crypto: UseCryptoReturn;
}

export function CryptoView({ crypto }: CryptoViewProps) {
  const {
    // General
    cryptoLoading,
    cryptoError,
    cryptoSubTab,
    setCryptoSubTab,

    // Prices & Watchlist
    cryptoAssets,
    trackedCoins,
    showAddCrypto,
    cryptoSearchQuery,
    setShowAddCrypto,
    setCryptoSearchQuery,
    filteredSuggestions,
    loadCryptoPrices,
    addTrackedCoin,
    removeTrackedCoin,

    // Wallets
    wallets,
    selectedWallet,
    showAddWallet,
    walletToDelete,
    setShowAddWallet,
    setWalletToDelete,
    setSelectedWallet,
    selectWallet,
    handleAddWallet,
    confirmDeleteWallet,

    // Wallet Form
    walletName,
    walletCategory,
    walletIcon,
    setWalletName,
    setWalletCategory,
    setWalletIcon,

    // Portfolio
    enrichedPortfolio,
    enrichedWalletHoldings,
    portfolioTotals,
    walletTransactions,

    // Transaction Modal
    showAddTransaction,
    setShowAddTransaction,
    txWalletId,
    txCoinId,
    txSymbol: _txSymbol,
    txType,
    txAmount,
    txPrice,
    txFee,
    txDate,
    txNotes,
    setTxWalletId,
    setTxType,
    setTxAmount,
    setTxPrice,
    setTxFee,
    setTxDate,
    setTxNotes,
    cryptoTxToDelete,
    setCryptoTxToDelete,
    handleAddCryptoTransaction,
    confirmDeleteCryptoTx,
    resetTransactionForm,
    selectCoinForTransaction,

    // Transfer Modal
    showTransferModal,
    setShowTransferModal,
    transferFromWallet,
    transferToWallet,
    transferCoinId,
    transferAmount,
    transferFee,
    transferDate,
    setTransferFromWallet,
    setTransferToWallet,
    setTransferCoinId,
    setTransferSymbol,
    setTransferAmount,
    setTransferFee,
    setTransferDate,
    handleAddTransfer,
    resetTransferForm,

    // Swap Modal
    showSwapModal,
    setShowSwapModal,
    swapWalletId,
    swapFromCoinId,
    swapFromAmount,
    swapToCoinId,
    swapToAmount,
    swapFee,
    swapDate,
    setSwapWalletId,
    setSwapFromCoinId,
    setSwapFromSymbol,
    setSwapFromAmount,
    setSwapToCoinId,
    setSwapToSymbol,
    setSwapToAmount,
    setSwapFee,
    setSwapDate,
    handleAddSwap,
    resetSwapForm,

    // Legacy Holdings
    showAddHolding,
    setShowAddHolding,
    holdingCoinId,
    holdingAmount,
    holdingPrice,
    holdingDate,
    holdingToDelete,
    setHoldingCoinId,
    setHoldingAmount,
    setHoldingPrice,
    setHoldingDate,
    setHoldingToDelete,
    addHolding,
    confirmDeleteHolding,
    selectCoinForHolding,
  } = crypto;

  const getTransactionTypeLabel = (type: string) => {
    const found = TRANSACTION_TYPES.find((t) => t.value === type);
    return found ? `${found.icon} ${found.label}` : type;
  };

  const getWalletCategoryLabel = (category: string) => {
    const found = WALLET_CATEGORIES.find((c) => c.value === category);
    return found ? found.label : category;
  };

  return (
    <div className="crypto-page">
      <div className="crypto-header">
        <h1 className="page-title">Cryptocurrency</h1>
        <div className="crypto-actions">
          <button
            className="btn-icon"
            onClick={loadCryptoPrices}
            disabled={cryptoLoading}
            title="Refresh prices"
          >
            {cryptoLoading ? "⏳" : "↻"}
          </button>
        </div>
      </div>

      {cryptoError && (
        <div className="message error crypto-error">{cryptoError}</div>
      )}

      {/* Sub-tabs for Overview and Wallets */}
      <div className="crypto-subtabs">
        <button
          className={`crypto-subtab ${cryptoSubTab === "overview" ? "active" : ""}`}
          onClick={() => {
            setCryptoSubTab("overview");
            setSelectedWallet(null);
          }}
        >
          📊 Overview
        </button>
        <button
          className={`crypto-subtab ${cryptoSubTab === "wallets" ? "active" : ""}`}
          onClick={() => setCryptoSubTab("wallets")}
        >
          👛 Wallets
        </button>
      </div>

      {/* ==================== Overview Sub-Tab ==================== */}
      {cryptoSubTab === "overview" && (
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
                  onClick={() => setCryptoSubTab("wallets")}
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
                  {trackedCoins.length}/{MAX_TRACKED_COINS}
                </span>
                <button
                  className="btn-icon"
                  onClick={() => setShowAddCrypto(true)}
                  disabled={trackedCoins.length >= MAX_TRACKED_COINS}
                  title="Track new coin"
                >
                  +
                </button>
              </div>
            </div>

            {cryptoLoading && cryptoAssets.length === 0 ? (
              <div className="crypto-loading">
                <div className="loader" />
                <p>Loading prices...</p>
              </div>
            ) : cryptoAssets.length === 0 ? (
              <div className="crypto-empty">
                <span className="crypto-empty-icon">📊</span>
                <p>Click refresh to load prices</p>
                <button className="btn-secondary" onClick={loadCryptoPrices}>
                  ↻ Load Prices
                </button>
              </div>
            ) : (
              <div className="crypto-grid">
                {cryptoAssets
                  .filter((asset) => trackedCoins.includes(asset.id))
                  .map((asset: (typeof cryptoAssets)[number]) => (
                    <div key={asset.id} className="crypto-card">
                      <button
                        className="crypto-remove"
                        onClick={() => removeTrackedCoin(asset.id)}
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
      {cryptoSubTab === "wallets" && !selectedWallet && (
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
      {cryptoSubTab === "wallets" && selectedWallet && (
        <>
          <div className="wallet-detail-header">
            <button
              className="btn-back"
              onClick={() => setSelectedWallet(null)}
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
                className="btn-primary"
                onClick={() => {
                  setTxWalletId(selectedWallet.id);
                  setShowAddTransaction(true);
                }}
              >
                + Add Transaction
              </button>
              <button
                className="btn-secondary"
                onClick={() => {
                  setTransferFromWallet(selectedWallet.id);
                  setShowTransferModal(true);
                }}
              >
                ↔ Transfer
              </button>
              <button
                className="btn-secondary"
                onClick={() => {
                  setSwapWalletId(selectedWallet.id);
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
                        onClick={() => setCryptoTxToDelete(tx.id)}
                        disabled={cryptoLoading}
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
                    value={walletName}
                    onChange={(e) => setWalletName(e.target.value)}
                    placeholder="e.g. Binance, Ledger, Metamask..."
                    required
                  />
                </div>
                <div className="form-group">
                  <label htmlFor="wallet-category">Category</label>
                  <select
                    id="wallet-category"
                    value={walletCategory}
                    onChange={(e) => setWalletCategory(e.target.value)}
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
                        className={`icon-option ${walletIcon === icon ? "selected" : ""}`}
                        onClick={() => setWalletIcon(icon)}
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
                  disabled={cryptoLoading}
                >
                  {cryptoLoading ? "Creating..." : "Create Wallet"}
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
            <form onSubmit={handleAddCryptoTransaction}>
              <div className="modal-body">
                <div className="form-group">
                  <label htmlFor="tx-wallet">Wallet</label>
                  <select
                    id="tx-wallet"
                    value={txWalletId}
                    onChange={(e) => setTxWalletId(e.target.value)}
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
                    value={txType}
                    onChange={(e) => setTxType(e.target.value)}
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
                        className={`crypto-suggestion ${txCoinId === coin.id ? "selected" : ""}`}
                        onClick={() => selectCoinForTransaction(coin)}
                      >
                        <span className="suggestion-symbol">{coin.symbol}</span>
                        <span className="suggestion-name">{coin.name}</span>
                      </button>
                    ))}
                  </div>
                </div>
                {txCoinId && (
                  <>
                    <div className="form-row">
                      <div className="form-group">
                        <label htmlFor="tx-amount">Amount</label>
                        <input
                          id="tx-amount"
                          type="number"
                          step="any"
                          value={txAmount}
                          onChange={(e) => setTxAmount(e.target.value)}
                          placeholder="0.00"
                          required
                        />
                      </div>
                      {(txType === "buy" || txType === "sell") && (
                        <div className="form-group">
                          <label htmlFor="tx-price">Price per coin ($)</label>
                          <input
                            id="tx-price"
                            type="number"
                            step="any"
                            value={txPrice}
                            onChange={(e) => setTxPrice(e.target.value)}
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
                          value={txFee}
                          onChange={(e) => setTxFee(e.target.value)}
                          placeholder="0.00"
                        />
                      </div>
                      <div className="form-group">
                        <label htmlFor="tx-date">Date</label>
                        <input
                          id="tx-date"
                          type="date"
                          value={txDate}
                          onChange={(e) => setTxDate(e.target.value)}
                        />
                      </div>
                    </div>
                    <div className="form-group">
                      <label htmlFor="tx-notes">Notes</label>
                      <input
                        id="tx-notes"
                        type="text"
                        value={txNotes}
                        onChange={(e) => setTxNotes(e.target.value)}
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
                  disabled={!txCoinId || cryptoLoading}
                >
                  {cryptoLoading ? "Adding..." : "Add Transaction"}
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
                      value={transferFromWallet}
                      onChange={(e) => setTransferFromWallet(e.target.value)}
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
                      value={transferToWallet}
                      onChange={(e) => setTransferToWallet(e.target.value)}
                      required
                    >
                      <option value="">Select...</option>
                      {wallets
                        .filter(
                          (w: CryptoWallet) => w.id !== transferFromWallet,
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
                        className={`crypto-suggestion ${transferCoinId === coin.id ? "selected" : ""}`}
                        onClick={() => {
                          setTransferCoinId(coin.id);
                          setTransferSymbol(coin.symbol);
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
                      value={transferAmount}
                      onChange={(e) => setTransferAmount(e.target.value)}
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
                      value={transferFee}
                      onChange={(e) => setTransferFee(e.target.value)}
                      placeholder="0.00"
                    />
                  </div>
                </div>
                <div className="form-group">
                  <label htmlFor="transfer-date">Date</label>
                  <input
                    id="transfer-date"
                    type="date"
                    value={transferDate}
                    onChange={(e) => setTransferDate(e.target.value)}
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
                  disabled={!transferCoinId || cryptoLoading}
                >
                  {cryptoLoading ? "Transferring..." : "Record Transfer"}
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
                    value={swapWalletId}
                    onChange={(e) => setSwapWalletId(e.target.value)}
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
                        className={`crypto-suggestion ${swapFromCoinId === coin.id ? "selected" : ""}`}
                        onClick={() => {
                          setSwapFromCoinId(coin.id);
                          setSwapFromSymbol(coin.symbol);
                        }}
                      >
                        <span className="suggestion-symbol">{coin.symbol}</span>
                      </button>
                    ))}
                  </div>
                  <input
                    type="number"
                    step="any"
                    value={swapFromAmount}
                    onChange={(e) => setSwapFromAmount(e.target.value)}
                    placeholder="Amount to swap"
                    required
                  />
                </div>
                <div className="swap-arrow">⬇️</div>
                <div className="swap-section">
                  <h4>To (Receive)</h4>
                  <div className="crypto-suggestions compact">
                    {POPULAR_CRYPTOS.filter((c) => c.id !== swapFromCoinId).map(
                      (coin) => (
                        <button
                          key={coin.id}
                          type="button"
                          className={`crypto-suggestion ${swapToCoinId === coin.id ? "selected" : ""}`}
                          onClick={() => {
                            setSwapToCoinId(coin.id);
                            setSwapToSymbol(coin.symbol);
                          }}
                        >
                          <span className="suggestion-symbol">
                            {coin.symbol}
                          </span>
                        </button>
                      ),
                    )}
                  </div>
                  <input
                    type="number"
                    step="any"
                    value={swapToAmount}
                    onChange={(e) => setSwapToAmount(e.target.value)}
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
                      value={swapFee}
                      onChange={(e) => setSwapFee(e.target.value)}
                      placeholder="0.00"
                    />
                  </div>
                  <div className="form-group">
                    <label htmlFor="swap-date">Date</label>
                    <input
                      id="swap-date"
                      type="date"
                      value={swapDate}
                      onChange={(e) => setSwapDate(e.target.value)}
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
                  disabled={!swapFromCoinId || !swapToCoinId || cryptoLoading}
                >
                  {cryptoLoading ? "Recording..." : "Record Swap"}
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
                  value={cryptoSearchQuery}
                  onChange={(e) => setCryptoSearchQuery(e.target.value)}
                  placeholder="e.g. bitcoin, eth, solana..."
                  autoFocus
                />
              </div>
              <div className="crypto-suggestions">
                {filteredSuggestions.length > 0 ? (
                  filteredSuggestions.map(
                    (coin: (typeof filteredSuggestions)[number]) => (
                      <button
                        key={coin.id}
                        className="crypto-suggestion"
                        onClick={() => addTrackedCoin(coin.id)}
                      >
                        <span className="suggestion-symbol">{coin.symbol}</span>
                        <span className="suggestion-name">{coin.name}</span>
                      </button>
                    ),
                  )
                ) : cryptoSearchQuery.trim() ? (
                  <button
                    className="crypto-suggestion custom"
                    onClick={() => addTrackedCoin(cryptoSearchQuery)}
                  >
                    <span className="suggestion-symbol">+</span>
                    <span className="suggestion-name">
                      Add "{cryptoSearchQuery}" as custom coin
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
                  setCryptoSearchQuery("");
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
                disabled={cryptoLoading}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                onClick={confirmDeleteWallet}
                disabled={cryptoLoading}
              >
                {cryptoLoading ? "Deleting..." : "Delete Wallet"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Crypto Transaction Confirmation Modal */}
      {cryptoTxToDelete !== null && (
        <div
          className="modal-overlay"
          onClick={() => setCryptoTxToDelete(null)}
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
                onClick={() => setCryptoTxToDelete(null)}
                disabled={cryptoLoading}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                onClick={confirmDeleteCryptoTx}
                disabled={cryptoLoading}
              >
                {cryptoLoading ? "Deleting..." : "Delete"}
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
            <form onSubmit={addHolding}>
              <div className="modal-body">
                <div className="form-group">
                  <label>Select Coin</label>
                  <div className="crypto-suggestions compact">
                    {POPULAR_CRYPTOS.map((coin) => (
                      <button
                        key={coin.id}
                        type="button"
                        className={`crypto-suggestion ${holdingCoinId === coin.id ? "selected" : ""}`}
                        onClick={() => selectCoinForHolding(coin)}
                      >
                        <span className="suggestion-symbol">{coin.symbol}</span>
                        <span className="suggestion-name">{coin.name}</span>
                      </button>
                    ))}
                  </div>
                </div>
                {holdingCoinId && (
                  <>
                    <div className="form-row">
                      <div className="form-group">
                        <label htmlFor="holding-amount">Amount</label>
                        <input
                          id="holding-amount"
                          type="number"
                          step="any"
                          value={holdingAmount}
                          onChange={(e) => setHoldingAmount(e.target.value)}
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
                          value={holdingPrice}
                          onChange={(e) => setHoldingPrice(e.target.value)}
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
                        value={holdingDate}
                        onChange={(e) => setHoldingDate(e.target.value)}
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
                    setHoldingCoinId("");
                    setHoldingAmount("");
                    setHoldingPrice("");
                  }}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="btn-primary"
                  disabled={!holdingCoinId || cryptoLoading}
                >
                  {cryptoLoading ? "Adding..." : "Add Holding"}
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
                disabled={cryptoLoading}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                onClick={confirmDeleteHolding}
                disabled={cryptoLoading}
              >
                {cryptoLoading ? "Removing..." : "Remove"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
