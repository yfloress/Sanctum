/**
 * Crypto View Component
 *
 * Main orchestrator for the cryptocurrency feature.
 * Renders layout and delegates to extracted components.
 *
 * Following project architecture:
 * - No business logic here (lives in stores)
 * - Components connect directly to Zustand
 * - Pure layout orchestration
 */

import { useCryptoStore } from "../../stores/cryptoStore.ts";

// Layout Components
import {
  AssetTable,
  CryptoHeader,
  PortfolioSummary,
  SubTabs,
  WalletDetail,
  WalletList,
  Watchlist,
} from "./components/index.ts";

// Modal Components
import {
  AddCryptoModal,
  AddHoldingModal,
  AddTransactionModal,
  AddWalletModal,
  DeleteConfirmModal,
  SwapModal,
  TransferModal,
} from "./modals/index.ts";

export function CryptoView() {
  // ==================== Store State ====================
  const subTab = useCryptoStore((state) => state.subTab);
  const selectedWallet = useCryptoStore((state) => state.selectedWallet);

  // Modal visibility (for conditional rendering of non-self-managing modals)
  const showAddTransaction = useCryptoStore(
    (state) => state.showAddTransaction,
  );
  const showTransferModal = useCryptoStore((state) => state.showTransferModal);
  const showSwapModal = useCryptoStore((state) => state.showSwapModal);

  // ==================== Render ====================
  return (
    <div className="crypto-page">
      {/* Header with title and refresh button */}
      <CryptoHeader />

      {/* Navigation tabs */}
      <SubTabs />

      {/* ==================== Overview Tab ==================== */}
      {subTab === "overview" && (
        <>
          {/* Portfolio Summary Section */}
          <div className="portfolio-section">
            <PortfolioSummary />
            <AssetTable />
          </div>

          {/* Watchlist Section */}
          <Watchlist />
        </>
      )}

      {/* ==================== Wallets Tab - List View ==================== */}
      {subTab === "wallets" && !selectedWallet && <WalletList />}

      {/* ==================== Wallets Tab - Detail View ==================== */}
      {subTab === "wallets" && selectedWallet && <WalletDetail />}

      {/* ==================== Footer Disclaimer ==================== */}
      <div className="crypto-disclaimer">
        <p>
          💡 Data provided by CoinGecko. Prices are for informational purposes
          only.
        </p>
      </div>

      {/* ==================== Modals ==================== */}
      {/* Self-managing modals (check visibility internally) */}
      <AddWalletModal />
      <AddCryptoModal />
      <AddHoldingModal />
      <DeleteConfirmModal />

      {/* Conditionally rendered modals */}
      {showAddTransaction && <AddTransactionModal />}
      {showTransferModal && <TransferModal />}
      {showSwapModal && <SwapModal />}
    </div>
  );
}
