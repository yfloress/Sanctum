/**
 * Watchlist Component
 *
 * Displays the cryptocurrency watchlist with price cards.
 * Connects directly to Zustand store for state management.
 */

import { MAX_TRACKED_COINS } from "../../../types/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function Watchlist() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);
  const prices = useCryptoStore((state) => state.prices);
  const watchlist = useCryptoStore((state) => state.watchlist);

  // ==================== Store Actions ====================
  const fetchPrices = useCryptoStore((state) => state.fetchPrices);
  const setShowAddCrypto = useCryptoStore((state) => state.setShowAddCrypto);
  const removeFromWatchlist = useCryptoStore(
    (state) => state.removeFromWatchlist,
  );

  // ==================== Render ====================
  return (
    <div className="watchlist-section">
      <div className="section-header">
        <h2 className="section-title">Watchlist</h2>
        <div className="crypto-actions">
          <span className="crypto-count">
            {watchlist.length}/{MAX_TRACKED_COINS}
          </span>
          <button
            type="button"
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
          <button
            type="button"
            className="btn-secondary"
            onClick={fetchPrices}
          >
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
                  type="button"
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
                    className={`crypto-change ${
                      asset.price_change_percentage_24h >= 0
                        ? "positive"
                        : "negative"
                    }`}
                  >
                    {asset.price_change_percentage_24h >= 0 ? "▲" : "▼"}{" "}
                    {Math.abs(asset.price_change_percentage_24h).toFixed(2)}%
                  </div>
                </div>
                <div className="crypto-price">
                  $
                  {asset.current_price.toLocaleString(undefined, {
                    minimumFractionDigits: 2,
                    maximumFractionDigits: asset.current_price < 1 ? 6 : 2,
                  })}
                </div>
                <div className="crypto-updated">
                  Updated: {new Date(asset.last_updated).toLocaleTimeString()}
                </div>
              </div>
            ))}
        </div>
      )}
    </div>
  );
}
