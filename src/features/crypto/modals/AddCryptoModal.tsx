/**
 * Add Crypto Modal Component
 *
 * Modal for adding cryptocurrencies to the watchlist.
 * Connects directly to Zustand store for state management.
 */

import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function AddCryptoModal() {
  // ==================== Store State ====================
  const showAddCrypto = useCryptoStore((state) => state.showAddCrypto);
  const searchQuery = useCryptoStore((state) => state.searchQuery);

  // ==================== Store Actions ====================
  const setShowAddCrypto = useCryptoStore((state) => state.setShowAddCrypto);
  const setSearchQuery = useCryptoStore((state) => state.setSearchQuery);
  const addToWatchlist = useCryptoStore((state) => state.addToWatchlist);

  // Computed getters
  const getFilteredSuggestions = useCryptoStore(
    (state) => state.getFilteredSuggestions,
  );

  // ==================== Computed Values ====================
  const filteredSuggestions = getFilteredSuggestions();

  // ==================== Handlers ====================
  const handleClose = () => {
    setShowAddCrypto(false);
    setSearchQuery("");
  };

  const handleAddCoin = (coinId: string) => {
    addToWatchlist(coinId);
  };

  // Don't render if modal is not visible
  if (!showAddCrypto) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={handleClose}>
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
                  type="button"
                  key={coin.id}
                  className="crypto-suggestion"
                  onClick={() => handleAddCoin(coin.id)}
                >
                  <span className="suggestion-symbol">{coin.symbol}</span>
                  <span className="suggestion-name">{coin.name}</span>
                </button>
              ))
            ) : searchQuery.trim() ? (
              <button
                type="button"
                className="crypto-suggestion custom"
                onClick={() => handleAddCoin(searchQuery)}
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
            onClick={handleClose}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
