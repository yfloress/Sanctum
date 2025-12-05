/**
 * Crypto Store - Zustand State Management
 *
 * SECURITY: This store lives in RAM only. NO persistence middleware.
 * The real persistence is handled by Rust (SQLCipher encrypted database).
 *
 * KILL SWITCH: The reset() action clears all data from memory when
 * the user locks the vault.
 */

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
  AggregatedAsset,
  CryptoAsset,
  CryptoHolding,
  CryptoSubTab,
  CryptoTransaction,
  CryptoWallet,
} from "../types/index.ts";
import {
  DEFAULT_TRACKED_COINS,
  MAX_TRACKED_COINS,
  POPULAR_CRYPTOS,
} from "../types/index.ts";
import {
  enrichAssetsWithPrices,
  getLocalDateString,
  isValidCoinId,
} from "../utils/index.ts";

// ==================== Types ====================

interface WalletFormData {
  name: string;
  category: string;
  icon: string;
}

interface TransactionFormData {
  walletId: string;
  coinId: string;
  symbol: string;
  type: string;
  amount: string;
  price: string;
  fee: string;
  date: string;
  notes: string;
}

interface TransferFormData {
  fromWalletId: string;
  toWalletId: string;
  coinId: string;
  symbol: string;
  amount: string;
  fee: string;
  date: string;
}

interface SwapFormData {
  walletId: string;
  fromCoinId: string;
  fromSymbol: string;
  fromAmount: string;
  toCoinId: string;
  toSymbol: string;
  toAmount: string;
  fee: string;
  date: string;
}

interface HoldingFormData {
  coinId: string;
  symbol: string;
  amount: string;
  price: string;
  date: string;
}

interface CryptoState {
  // UI State
  isLoading: boolean;
  error: string | null;
  successMessage: string | null;
  subTab: CryptoSubTab;

  // Prices & Watchlist
  prices: CryptoAsset[];
  watchlist: string[];
  searchQuery: string;

  // Wallets & Portfolio
  wallets: CryptoWallet[];
  selectedWallet: CryptoWallet | null;
  walletTransactions: CryptoTransaction[];
  walletHoldings: AggregatedAsset[];
  portfolio: AggregatedAsset[];

  // Legacy Holdings
  holdings: CryptoHolding[];

  // Modal States
  showAddCrypto: boolean;
  showAddWallet: boolean;
  showAddTransaction: boolean;
  showTransferModal: boolean;
  showSwapModal: boolean;
  showAddHolding: boolean;

  // Delete Confirmations
  walletToDelete: string | null;
  transactionToDelete: string | null;
  holdingToDelete: string | null;

  // Form States
  walletForm: WalletFormData;
  transactionForm: TransactionFormData;
  transferForm: TransferFormData;
  swapForm: SwapFormData;
  holdingForm: HoldingFormData;
}

interface CryptoActions {
  // Data Loading
  loadAll: () => Promise<void>;
  fetchPrices: () => Promise<void>;
  loadWallets: () => Promise<void>;
  loadPortfolio: () => Promise<void>;
  loadWalletDetails: (walletId: string) => Promise<void>;
  loadHoldings: () => Promise<void>;

  // UI State
  setSubTab: (tab: CryptoSubTab) => void;
  setSearchQuery: (query: string) => void;

  // Watchlist
  addToWatchlist: (coinId: string) => void;
  removeFromWatchlist: (coinId: string) => void;

  // Wallet Operations
  addWallet: () => Promise<boolean>;
  deleteWallet: () => Promise<boolean>;
  selectWallet: (wallet: CryptoWallet | null) => Promise<void>;
  setWalletToDelete: (id: string | null) => void;

  // Transaction Operations
  addTransaction: () => Promise<boolean>;
  addTransfer: () => Promise<boolean>;
  addSwap: () => Promise<boolean>;
  deleteTransaction: () => Promise<boolean>;
  setTransactionToDelete: (id: string | null) => void;

  // Legacy Holdings
  addHolding: () => Promise<boolean>;
  deleteHolding: () => Promise<boolean>;
  setHoldingToDelete: (id: string | null) => void;

  // Modal Controls
  setShowAddCrypto: (show: boolean) => void;
  setShowAddWallet: (show: boolean) => void;
  setShowAddTransaction: (show: boolean) => void;
  setShowTransferModal: (show: boolean) => void;
  setShowSwapModal: (show: boolean) => void;
  setShowAddHolding: (show: boolean) => void;

  // Form Management
  setWalletFormField: <K extends keyof WalletFormData>(
    field: K,
    value: WalletFormData[K],
  ) => void;
  setTransactionFormField: <K extends keyof TransactionFormData>(
    field: K,
    value: TransactionFormData[K],
  ) => void;
  setTransferFormField: <K extends keyof TransferFormData>(
    field: K,
    value: TransferFormData[K],
  ) => void;
  setSwapFormField: <K extends keyof SwapFormData>(
    field: K,
    value: SwapFormData[K],
  ) => void;
  setHoldingFormField: <K extends keyof HoldingFormData>(
    field: K,
    value: HoldingFormData[K],
  ) => void;
  resetWalletForm: () => void;
  resetTransactionForm: () => void;
  resetTransferForm: () => void;
  resetSwapForm: () => void;
  resetHoldingForm: () => void;

  // Coin Selection Helpers
  selectCoinForTransaction: (coin: { id: string; symbol: string }) => void;
  selectCoinForHolding: (coin: { id: string; symbol: string }) => void;

  // Computed Getters
  getFilteredSuggestions: () => Array<{
    id: string;
    symbol: string;
    name: string;
  }>;
  getEnrichedPortfolio: () => AggregatedAsset[];
  getEnrichedWalletHoldings: () => AggregatedAsset[];
  getPortfolioTotals: () => {
    totalValue: number;
    totalCost: number;
    totalPnl: number;
    totalPnlPercentage: number;
  };

  // Messages
  setError: (error: string | null) => void;
  setSuccess: (message: string | null) => void;
  clearMessages: () => void;

  // Security: RAM Clear
  reset: () => void;
}

export type CryptoStore = CryptoState & CryptoActions;

// ==================== Initial State ====================

const initialWalletForm: WalletFormData = {
  name: "",
  category: "exchange",
  icon: "🏦",
};

const initialTransactionForm: TransactionFormData = {
  walletId: "",
  coinId: "",
  symbol: "",
  type: "buy",
  amount: "",
  price: "",
  fee: "",
  date: getLocalDateString(),
  notes: "",
};

const initialTransferForm: TransferFormData = {
  fromWalletId: "",
  toWalletId: "",
  coinId: "",
  symbol: "",
  amount: "",
  fee: "",
  date: getLocalDateString(),
};

const initialSwapForm: SwapFormData = {
  walletId: "",
  fromCoinId: "",
  fromSymbol: "",
  fromAmount: "",
  toCoinId: "",
  toSymbol: "",
  toAmount: "",
  fee: "",
  date: getLocalDateString(),
};

const initialHoldingForm: HoldingFormData = {
  coinId: "",
  symbol: "",
  amount: "",
  price: "",
  date: getLocalDateString(),
};

const initialState: CryptoState = {
  isLoading: false,
  error: null,
  successMessage: null,
  subTab: "overview",

  prices: [],
  watchlist: [...DEFAULT_TRACKED_COINS],
  searchQuery: "",

  wallets: [],
  selectedWallet: null,
  walletTransactions: [],
  walletHoldings: [],
  portfolio: [],

  holdings: [],

  showAddCrypto: false,
  showAddWallet: false,
  showAddTransaction: false,
  showTransferModal: false,
  showSwapModal: false,
  showAddHolding: false,

  walletToDelete: null,
  transactionToDelete: null,
  holdingToDelete: null,

  walletForm: { ...initialWalletForm },
  transactionForm: { ...initialTransactionForm },
  transferForm: { ...initialTransferForm },
  swapForm: { ...initialSwapForm },
  holdingForm: { ...initialHoldingForm },
};

// ==================== Store ====================

export const useCryptoStore = create<CryptoStore>((set, get) => ({
  ...initialState,

  // ==================== Data Loading ====================

  loadAll: async () => {
    const { loadWallets, loadPortfolio, loadHoldings } = get();
    set({ isLoading: true, error: null });
    try {
      await Promise.all([loadWallets(), loadPortfolio(), loadHoldings()]);

      // Try to load cached prices first for immediate display
      try {
        const cachedPrices = await invoke<CryptoAsset[]>("load_crypto_prices");
        if (cachedPrices && cachedPrices.length > 0) {
          set({ prices: cachedPrices });
        }
      } catch {
        // Ignore cache errors
      }
    } catch (err) {
      set({ error: `Error loading crypto data: ${err}` });
    } finally {
      set({ isLoading: false });
    }
  },

  fetchPrices: async () => {
    const state = get();
    const holdingCoinIds = state.holdings.map((h) => h.coin_id);
    const portfolioCoinIds = state.portfolio.map((a) => a.coin_id);
    const allCoins = [
      ...new Set([...state.watchlist, ...holdingCoinIds, ...portfolioCoinIds]),
    ];

    if (allCoins.length === 0) {
      set({ prices: [] });
      return;
    }

    set({ isLoading: true, error: null });

    try {
      const CHUNK_SIZE = 50;
      const results: CryptoAsset[] = [];

      for (let i = 0; i < allCoins.length; i += CHUNK_SIZE) {
        const chunk = allCoins.slice(i, i + CHUNK_SIZE);
        if (i > 0) {
          // Rate limiting: wait 1.6s between chunks
          await new Promise((resolve) => setTimeout(resolve, 1600));
        }
        const assets = await invoke<CryptoAsset[]>("get_crypto_prices", {
          coins: chunk,
        });
        results.push(...assets);
      }

      set({ prices: results });

      // Save prices to cache for offline use
      if (results.length > 0) {
        try {
          await invoke("save_crypto_prices", { prices: results });
        } catch {
          // Ignore cache save errors
        }
      }
    } catch (err) {
      set({ error: String(err) });
      console.error("Error loading crypto prices:", err);

      // If API fails and we have no prices, try loading from cache
      if (state.prices.length === 0) {
        try {
          const cachedPrices =
            await invoke<CryptoAsset[]>("load_crypto_prices");
          if (cachedPrices && cachedPrices.length > 0) {
            set({ prices: cachedPrices });
          }
        } catch {
          // Ignore cache errors
        }
      }
    } finally {
      set({ isLoading: false });
    }
  },

  loadWallets: async () => {
    try {
      const wallets = await invoke<CryptoWallet[]>("get_wallets");
      set({ wallets });
    } catch (err) {
      console.error("Error loading wallets:", err);
      throw err;
    }
  },

  loadPortfolio: async () => {
    try {
      const portfolio = await invoke<AggregatedAsset[]>(
        "get_aggregated_portfolio",
      );
      set({ portfolio });
    } catch (err) {
      console.error("Error loading portfolio:", err);
      throw err;
    }
  },

  loadWalletDetails: async (walletId: string) => {
    try {
      const [walletTransactions, walletHoldings] = await Promise.all([
        invoke<CryptoTransaction[]>("get_wallet_transactions", { walletId }),
        invoke<AggregatedAsset[]>("get_wallet_holdings", { walletId }),
      ]);
      set({ walletTransactions, walletHoldings });
    } catch (err) {
      console.error("Error loading wallet details:", err);
    }
  },

  loadHoldings: async () => {
    try {
      const holdings = await invoke<CryptoHolding[]>("get_crypto_holdings");
      set({ holdings });
    } catch (err) {
      console.error("Error loading holdings:", err);
      throw err;
    }
  },

  // ==================== UI State ====================

  setSubTab: (tab: CryptoSubTab) => set({ subTab: tab }),

  setSearchQuery: (query: string) => set({ searchQuery: query }),

  // ==================== Watchlist ====================

  addToWatchlist: (coinId: string) => {
    const state = get();
    const normalized = coinId.toLowerCase().trim();

    if (!normalized || state.watchlist.includes(normalized)) return;

    if (state.watchlist.length >= MAX_TRACKED_COINS) {
      set({ error: `Maximum ${MAX_TRACKED_COINS} coins allowed` });
      return;
    }

    if (!isValidCoinId(normalized)) {
      set({ error: "Invalid coin ID format" });
      return;
    }

    set({
      watchlist: [...state.watchlist, normalized],
      showAddCrypto: false,
      searchQuery: "",
    });
  },

  removeFromWatchlist: (coinId: string) => {
    set((state) => ({
      watchlist: state.watchlist.filter((id) => id !== coinId),
      prices: state.prices.filter((asset) => asset.id !== coinId),
    }));
  },

  // ==================== Wallet Operations ====================

  addWallet: async () => {
    const state = get();
    const { name, category, icon } = state.walletForm;

    if (!name.trim()) {
      set({ error: "Wallet name cannot be empty" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      await invoke("add_wallet", {
        name: name.trim(),
        category,
        icon,
      });

      get().resetWalletForm();
      set({
        showAddWallet: false,
        successMessage: "Wallet created successfully",
      });

      await get().loadWallets();

      setTimeout(() => set({ successMessage: null }), 3000);
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  deleteWallet: async () => {
    const state = get();
    if (!state.walletToDelete) return false;

    set({ isLoading: true, error: null });

    try {
      await invoke("delete_wallet", { id: state.walletToDelete });

      // Clear selection if deleted wallet was selected
      if (state.selectedWallet?.id === state.walletToDelete) {
        set({
          selectedWallet: null,
          walletTransactions: [],
          walletHoldings: [],
        });
      }

      set({
        walletToDelete: null,
        successMessage: "Wallet deleted successfully",
      });

      await Promise.all([get().loadWallets(), get().loadPortfolio()]);

      setTimeout(() => set({ successMessage: null }), 3000);
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  selectWallet: async (wallet: CryptoWallet | null) => {
    set({ selectedWallet: wallet });
    if (wallet) {
      await get().loadWalletDetails(wallet.id);
    } else {
      set({ walletTransactions: [], walletHoldings: [] });
    }
  },

  setWalletToDelete: (id: string | null) => set({ walletToDelete: id }),

  // ==================== Transaction Operations ====================

  addTransaction: async () => {
    const state = get();
    const form = state.transactionForm;

    const parsedAmount = parseFloat(form.amount);
    const parsedPrice = parseFloat(form.price) || 0;
    const parsedFee = parseFloat(form.fee) || 0;

    // Validation
    if (!form.walletId) {
      set({ error: "Please select a wallet" });
      return false;
    }
    if (!form.coinId || !form.symbol) {
      set({ error: "Please select a coin" });
      return false;
    }
    if (!form.amount || parsedAmount <= 0) {
      set({ error: "Amount must be greater than zero" });
      return false;
    }
    if ((form.type === "buy" || form.type === "sell") && parsedPrice <= 0) {
      set({ error: "Price must be greater than zero" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      await invoke("add_crypto_transaction", {
        params: {
          walletId: form.walletId,
          coinId: form.coinId.toLowerCase(),
          symbol: form.symbol.toUpperCase(),
          transactionType: form.type,
          amount: parsedAmount,
          pricePerCoin: parsedPrice > 0 ? parsedPrice : null,
          fee: parsedFee > 0 ? parsedFee : null,
          date: form.date,
          notes: form.notes.trim() || null,
        },
      });

      get().resetTransactionForm();
      set({
        showAddTransaction: false,
        successMessage: "Transaction added successfully",
      });

      // Reload data
      await Promise.all([
        get().loadPortfolio(),
        state.selectedWallet
          ? get().loadWalletDetails(state.selectedWallet.id)
          : Promise.resolve(),
      ]);

      setTimeout(() => set({ successMessage: null }), 3000);
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  addTransfer: async () => {
    const state = get();
    const form = state.transferForm;

    const parsedAmount = parseFloat(form.amount);
    const parsedFee = parseFloat(form.fee) || 0;

    // Validation
    if (!form.fromWalletId || !form.toWalletId) {
      set({ error: "Please select both source and destination wallets" });
      return false;
    }
    if (form.fromWalletId === form.toWalletId) {
      set({ error: "Source and destination wallets must be different" });
      return false;
    }
    if (!form.coinId || !form.symbol) {
      set({ error: "Please select a coin" });
      return false;
    }
    if (!form.amount || parsedAmount <= 0) {
      set({ error: "Amount must be greater than zero" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      await invoke("add_transfer_transaction", {
        params: {
          fromWalletId: form.fromWalletId,
          toWalletId: form.toWalletId,
          coinId: form.coinId.toLowerCase(),
          symbol: form.symbol.toUpperCase(),
          amount: parsedAmount,
          fee: parsedFee > 0 ? parsedFee : null,
          date: form.date,
          notes: null,
        },
      });

      get().resetTransferForm();
      set({
        showTransferModal: false,
        successMessage: "Transfer recorded successfully",
      });

      // Reload data
      await Promise.all([
        get().loadPortfolio(),
        state.selectedWallet
          ? get().loadWalletDetails(state.selectedWallet.id)
          : Promise.resolve(),
      ]);

      setTimeout(() => set({ successMessage: null }), 3000);
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  addSwap: async () => {
    const state = get();
    const form = state.swapForm;

    const parsedFromAmount = parseFloat(form.fromAmount);
    const parsedToAmount = parseFloat(form.toAmount);
    const parsedFee = parseFloat(form.fee) || 0;

    // Validation
    if (!form.walletId) {
      set({ error: "Please select a wallet" });
      return false;
    }
    if (!form.fromCoinId || !form.fromSymbol) {
      set({ error: "Please select a coin to swap from" });
      return false;
    }
    if (!form.toCoinId || !form.toSymbol) {
      set({ error: "Please select a coin to swap to" });
      return false;
    }
    if (form.fromCoinId === form.toCoinId) {
      set({ error: "Cannot swap a coin for itself" });
      return false;
    }
    if (!form.fromAmount || parsedFromAmount <= 0) {
      set({ error: "From amount must be greater than zero" });
      return false;
    }
    if (!form.toAmount || parsedToAmount <= 0) {
      set({ error: "To amount must be greater than zero" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      await invoke("add_swap_transaction", {
        params: {
          walletId: form.walletId,
          fromCoinId: form.fromCoinId.toLowerCase(),
          fromSymbol: form.fromSymbol.toUpperCase(),
          fromAmount: parsedFromAmount,
          toCoinId: form.toCoinId.toLowerCase(),
          toSymbol: form.toSymbol.toUpperCase(),
          toAmount: parsedToAmount,
          fee: parsedFee > 0 ? parsedFee : null,
          feeCoinId: null,
          feeAmount: null,
          date: form.date,
          notes: null,
        },
      });

      get().resetSwapForm();
      set({
        showSwapModal: false,
        successMessage: "Swap recorded successfully",
      });

      // Reload data
      await Promise.all([
        get().loadPortfolio(),
        state.selectedWallet
          ? get().loadWalletDetails(state.selectedWallet.id)
          : Promise.resolve(),
      ]);

      setTimeout(() => set({ successMessage: null }), 3000);
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  deleteTransaction: async () => {
    const state = get();
    if (!state.transactionToDelete) return false;

    set({ isLoading: true, error: null });

    try {
      await invoke("delete_crypto_transaction", {
        id: state.transactionToDelete,
      });

      set({
        transactionToDelete: null,
        successMessage: "Transaction deleted successfully",
      });

      // Reload data
      await Promise.all([
        get().loadPortfolio(),
        state.selectedWallet
          ? get().loadWalletDetails(state.selectedWallet.id)
          : Promise.resolve(),
      ]);

      setTimeout(() => set({ successMessage: null }), 3000);
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  setTransactionToDelete: (id: string | null) =>
    set({ transactionToDelete: id }),

  // ==================== Legacy Holdings ====================

  addHolding: async () => {
    const state = get();
    const form = state.holdingForm;

    const parsedAmount = parseFloat(form.amount);
    const parsedPrice = parseFloat(form.price) || 0;

    // Validation
    if (!form.coinId || !form.symbol) {
      set({ error: "Please select a coin" });
      return false;
    }
    if (!form.amount || parsedAmount <= 0) {
      set({ error: "Amount must be greater than zero" });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      await invoke("add_crypto_holding", {
        coinId: form.coinId.toLowerCase(),
        symbol: form.symbol.toUpperCase(),
        amount: parsedAmount,
        purchasePrice: parsedPrice,
        purchaseDate: form.date,
      });

      get().resetHoldingForm();
      set({
        showAddHolding: false,
        successMessage: "Holding added successfully",
      });

      await get().loadHoldings();

      setTimeout(() => set({ successMessage: null }), 3000);
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  deleteHolding: async () => {
    const state = get();
    if (!state.holdingToDelete) return false;

    set({ isLoading: true, error: null });

    try {
      await invoke("delete_crypto_holding", { id: state.holdingToDelete });

      set({
        holdingToDelete: null,
        successMessage: "Holding deleted successfully",
      });

      await get().loadHoldings();

      setTimeout(() => set({ successMessage: null }), 3000);
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  setHoldingToDelete: (id: string | null) => set({ holdingToDelete: id }),

  // ==================== Modal Controls ====================

  setShowAddCrypto: (show: boolean) => set({ showAddCrypto: show }),
  setShowAddWallet: (show: boolean) => set({ showAddWallet: show }),
  setShowAddTransaction: (show: boolean) => set({ showAddTransaction: show }),
  setShowTransferModal: (show: boolean) => set({ showTransferModal: show }),
  setShowSwapModal: (show: boolean) => set({ showSwapModal: show }),
  setShowAddHolding: (show: boolean) => set({ showAddHolding: show }),

  // ==================== Form Management ====================

  setWalletFormField: (field, value) => {
    set((state) => ({
      walletForm: { ...state.walletForm, [field]: value },
    }));
  },

  setTransactionFormField: (field, value) => {
    set((state) => ({
      transactionForm: { ...state.transactionForm, [field]: value },
    }));
  },

  setTransferFormField: (field, value) => {
    set((state) => ({
      transferForm: { ...state.transferForm, [field]: value },
    }));
  },

  setSwapFormField: (field, value) => {
    set((state) => ({
      swapForm: { ...state.swapForm, [field]: value },
    }));
  },

  setHoldingFormField: (field, value) => {
    set((state) => ({
      holdingForm: { ...state.holdingForm, [field]: value },
    }));
  },

  resetWalletForm: () => set({ walletForm: { ...initialWalletForm } }),

  resetTransactionForm: () =>
    set({
      transactionForm: {
        ...initialTransactionForm,
        date: getLocalDateString(),
      },
    }),

  resetTransferForm: () =>
    set({
      transferForm: { ...initialTransferForm, date: getLocalDateString() },
    }),

  resetSwapForm: () =>
    set({
      swapForm: { ...initialSwapForm, date: getLocalDateString() },
    }),

  resetHoldingForm: () =>
    set({
      holdingForm: { ...initialHoldingForm, date: getLocalDateString() },
    }),

  // ==================== Coin Selection Helpers ====================

  selectCoinForTransaction: (coin: { id: string; symbol: string }) => {
    const state = get();
    const asset = state.prices.find((a) => a.id === coin.id);
    const txType = state.transactionForm.type;

    set((s) => ({
      transactionForm: {
        ...s.transactionForm,
        coinId: coin.id,
        symbol: coin.symbol,
        price:
          asset && (txType === "buy" || txType === "sell")
            ? asset.current_price.toString()
            : s.transactionForm.price,
      },
    }));
  },

  selectCoinForHolding: (coin: { id: string; symbol: string }) => {
    const state = get();
    const asset = state.prices.find((a) => a.id === coin.id);

    set((s) => ({
      holdingForm: {
        ...s.holdingForm,
        coinId: coin.id,
        symbol: coin.symbol,
        price: asset ? asset.current_price.toString() : s.holdingForm.price,
      },
    }));
  },

  // ==================== Computed Getters ====================

  getFilteredSuggestions: () => {
    const state = get();
    const query = state.searchQuery.toLowerCase().trim();

    if (!query) {
      return POPULAR_CRYPTOS.filter((c) => !state.watchlist.includes(c.id));
    }

    return POPULAR_CRYPTOS.filter(
      (c) =>
        !state.watchlist.includes(c.id) &&
        (c.id.includes(query) ||
          c.symbol.toLowerCase().includes(query) ||
          c.name.toLowerCase().includes(query)),
    );
  },

  getEnrichedPortfolio: () => {
    const state = get();
    return enrichAssetsWithPrices(state.portfolio, state.prices);
  },

  getEnrichedWalletHoldings: () => {
    const state = get();
    return enrichAssetsWithPrices(state.walletHoldings, state.prices);
  },

  getPortfolioTotals: () => {
    const enrichedPortfolio = get().getEnrichedPortfolio();

    const totalValue = enrichedPortfolio.reduce(
      (sum, item) => sum + item.current_value,
      0,
    );
    const totalCost = enrichedPortfolio.reduce(
      (sum, item) => sum + item.total_cost_basis,
      0,
    );
    const totalPnl = totalValue - totalCost;
    const totalPnlPercentage = totalCost > 0 ? (totalPnl / totalCost) * 100 : 0;

    return { totalValue, totalCost, totalPnl, totalPnlPercentage };
  },

  // ==================== Messages ====================

  setError: (error: string | null) => set({ error }),

  setSuccess: (message: string | null) => set({ successMessage: message }),

  clearMessages: () => set({ error: null, successMessage: null }),

  // ==================== Security: RAM Clear ====================

  reset: () => {
    set({
      ...initialState,
      walletForm: { ...initialWalletForm },
      transactionForm: { ...initialTransactionForm },
      transferForm: { ...initialTransferForm },
      swapForm: { ...initialSwapForm },
      holdingForm: { ...initialHoldingForm },
    });
  },
}));

// ==================== Selector Hooks (for optimized re-renders) ====================

export const useCryptoPrices = () => useCryptoStore((state) => state.prices);
export const useWatchlist = () => useCryptoStore((state) => state.watchlist);
export const useWallets = () => useCryptoStore((state) => state.wallets);
export const useSelectedWallet = () =>
  useCryptoStore((state) => state.selectedWallet);
export const usePortfolio = () => useCryptoStore((state) => state.portfolio);
export const useCryptoLoading = () =>
  useCryptoStore((state) => state.isLoading);
export const useCryptoError = () => useCryptoStore((state) => state.error);
export const useCryptoSuccess = () =>
  useCryptoStore((state) => state.successMessage);
export const useCryptoSubTab = () => useCryptoStore((state) => state.subTab);
