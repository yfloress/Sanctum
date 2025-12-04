/**
 * AssetTable Component
 *
 * Displays the portfolio grid with aggregated crypto assets.
 * Connects directly to Zustand store for state management.
 */

import type { AggregatedAsset } from "../../../types/index.ts";
import { formatCryptoAmount, formatUSD } from "../../../utils/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function AssetTable() {
  // ==================== Store State ====================
  const getEnrichedPortfolio = useCryptoStore(
    (state) => state.getEnrichedPortfolio,
  );
  const setSubTab = useCryptoStore((state) => state.setSubTab);

  // ==================== Computed Values ====================
  const enrichedPortfolio = getEnrichedPortfolio();

  // ==================== Render ====================
  if (enrichedPortfolio.length === 0) {
    return (
      <div className="portfolio-empty">
        <span className="portfolio-empty-icon">💼</span>
        <p>No holdings yet. Add a wallet and start tracking your portfolio!</p>
        <button
          type="button"
          className="btn-secondary"
          onClick={() => setSubTab("wallets")}
        >
          Go to Wallets
        </button>
      </div>
    );
  }

  return (
    <div className="portfolio-grid">
      {enrichedPortfolio.map((asset: AggregatedAsset) => (
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
          <div className="portfolio-details">
            <span>Avg: ${formatCryptoAmount(asset.avg_buy_price, 6)}</span>
            <span
              className={asset.unrealized_pnl >= 0 ? "positive" : "negative"}
            >
              {asset.unrealized_pnl >= 0 ? "+" : ""}$
              {formatUSD(asset.unrealized_pnl)}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}
