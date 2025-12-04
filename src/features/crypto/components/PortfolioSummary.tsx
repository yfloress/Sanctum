/**
 * Portfolio Summary Component
 *
 * Displays the total portfolio value and P&L summary.
 * Connects directly to Zustand store for state management.
 */

import { formatUSD } from "../../../utils/index.ts";
import { useCryptoStore } from "../../../stores/cryptoStore.ts";

export function PortfolioSummary() {
  // ==================== Computed Getters ====================
  const getPortfolioTotals = useCryptoStore(
    (state) => state.getPortfolioTotals,
  );

  // ==================== Computed Values ====================
  const portfolioTotals = getPortfolioTotals();

  return (
    <div className="section-header">
      <h2 className="section-title">Total Portfolio</h2>
      <div className="portfolio-total">
        <span className="portfolio-total-label">Total Value</span>
        <span className="portfolio-total-value">
          ${formatUSD(portfolioTotals.totalValue)}
        </span>
        <span
          className={`portfolio-total-pnl ${
            portfolioTotals.totalPnl >= 0 ? "positive" : "negative"
          }`}
        >
          {portfolioTotals.totalPnl >= 0 ? "+" : ""}$
          {formatUSD(portfolioTotals.totalPnl)} (
          {portfolioTotals.totalPnlPercentage >= 0 ? "+" : ""}
          {portfolioTotals.totalPnlPercentage.toFixed(2)}%)
        </span>
      </div>
    </div>
  );
}
