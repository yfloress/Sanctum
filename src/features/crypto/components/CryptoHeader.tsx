/**
 * Crypto Header Component
 *
 * Displays the main header for the crypto page with title and refresh button.
 * Connects directly to Zustand store for state management.
 */

import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function CryptoHeader() {
  // ==================== Store State ====================
  const isLoading = useCryptoStore((state) => state.isLoading);

  // ==================== Store Actions ====================
  const fetchPrices = useCryptoStore((state) => state.fetchPrices);

  return (
    <div className="crypto-header">
      <h1 className="page-title">Cryptocurrency</h1>
      <div className="crypto-actions">
        <button
          type="button"
          className="btn-icon"
          onClick={fetchPrices}
          disabled={isLoading}
          title="Refresh prices"
        >
          {isLoading ? "⏳" : "↻"}
        </button>
      </div>
    </div>
  );
}
