import { useState, useCallback, useMemo } from "react";
import type { FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  CryptoAsset,
  CryptoHolding,
  CryptoWallet,
  CryptoTransaction,
  AggregatedAsset,
  CryptoSubTab,
} from "../types";
import {
  DEFAULT_TRACKED_COINS,
  MAX_TRACKED_COINS,
  POPULAR_CRYPTOS,
} from "../types";
import {
  getLocalDateString,
  enrichAssetsWithPrices,
  isValidCoinId,
} from "../utils";

export interface UseCryptoReturn {
  // State - General
  cryptoLoading: boolean;
  cryptoError: string;
  cryptoSubTab: CryptoSubTab;
  setCryptoSubTab: (tab: CryptoSubTab) => void;
  setCryptoError: (error: string) => void;

  // State - Prices & Watchlist
  cryptoAssets: CryptoAsset[];
  trackedCoins: string[];
  showAddCrypto: boolean;
  cryptoSearchQuery: string;
  setShowAddCrypto: (show: boolean) => void;
  setCryptoSearchQuery: (query: string) => void;
  filteredSuggestions: Array<{ id: string; symbol: string; name: string }>;

  // State - Wallets
  wallets: CryptoWallet[];
  selectedWallet: CryptoWallet | null;
  walletTransactions: CryptoTransaction[];
  walletHoldings: AggregatedAsset[];
  showAddWallet: boolean;
  walletToDelete: string | null;
  setShowAddWallet: (show: boolean) => void;
  setWalletToDelete: (id: string | null) => void;
  setSelectedWallet: (wallet: CryptoWallet | null) => void;

  // State - Wallet Form
  walletName: string;
  walletCategory: string;
  walletIcon: string;
  setWalletName: (name: string) => void;
  setWalletCategory: (category: string) => void;
  setWalletIcon: (icon: string) => void;

  // State - Transaction Modal
  showAddTransaction: boolean;
  setShowAddTransaction: (show: boolean) => void;
  txWalletId: string;
  txCoinId: string;
  txSymbol: string;
  txType: string;
  txAmount: string;
  txPrice: string;
  txFee: string;
  txDate: string;
  txNotes: string;
  setTxWalletId: (id: string) => void;
  setTxCoinId: (id: string) => void;
  setTxSymbol: (symbol: string) => void;
  setTxType: (type: string) => void;
  setTxAmount: (amount: string) => void;
  setTxPrice: (price: string) => void;
  setTxFee: (fee: string) => void;
  setTxDate: (date: string) => void;
  setTxNotes: (notes: string) => void;
  cryptoTxToDelete: string | null;
  setCryptoTxToDelete: (id: string | null) => void;

  // State - Transfer Modal
  showTransferModal: boolean;
  setShowTransferModal: (show: boolean) => void;
  transferFromWallet: string;
  transferToWallet: string;
  transferCoinId: string;
  transferSymbol: string;
  transferAmount: string;
  transferFee: string;
  transferDate: string;
  setTransferFromWallet: (id: string) => void;
  setTransferToWallet: (id: string) => void;
  setTransferCoinId: (id: string) => void;
  setTransferSymbol: (symbol: string) => void;
  setTransferAmount: (amount: string) => void;
  setTransferFee: (fee: string) => void;
  setTransferDate: (date: string) => void;

  // State - Swap Modal
  showSwapModal: boolean;
  setShowSwapModal: (show: boolean) => void;
  swapWalletId: string;
  swapFromCoinId: string;
  swapFromSymbol: string;
  swapFromAmount: string;
  swapToCoinId: string;
  swapToSymbol: string;
  swapToAmount: string;
  swapFee: string;
  swapDate: string;
  setSwapWalletId: (id: string) => void;
  setSwapFromCoinId: (id: string) => void;
  setSwapFromSymbol: (symbol: string) => void;
  setSwapFromAmount: (amount: string) => void;
  setSwapToCoinId: (id: string) => void;
  setSwapToSymbol: (symbol: string) => void;
  setSwapToAmount: (amount: string) => void;
  setSwapFee: (fee: string) => void;
  setSwapDate: (date: string) => void;

  // State - Legacy Holdings
  holdings: CryptoHolding[];
  showAddHolding: boolean;
  holdingCoinId: string;
  holdingSymbol: string;
  holdingAmount: string;
  holdingPrice: string;
  holdingDate: string;
  holdingToDelete: string | null;
  setShowAddHolding: (show: boolean) => void;
  setHoldingCoinId: (id: string) => void;
  setHoldingSymbol: (symbol: string) => void;
  setHoldingAmount: (amount: string) => void;
  setHoldingPrice: (price: string) => void;
  setHoldingDate: (date: string) => void;
  setHoldingToDelete: (id: string | null) => void;

  // Computed
  aggregatedPortfolio: AggregatedAsset[];
  enrichedPortfolio: AggregatedAsset[];
  enrichedWalletHoldings: AggregatedAsset[];
  portfolioTotals: {
    totalValue: number;
    totalCost: number;
    totalPnl: number;
    totalPnlPercentage: number;
  };

  // Actions - Data Loading
  loadCryptoPrices: () => Promise<void>;
  loadWallets: () => Promise<void>;
  loadAggregatedPortfolio: () => Promise<void>;
  loadWalletDetails: (walletId: string) => Promise<void>;
  loadHoldings: () => Promise<void>;

  // Actions - Watchlist
  addTrackedCoin: (coinId: string) => void;
  removeTrackedCoin: (coinId: string) => void;

  // Actions - Wallets
  handleAddWallet: (e: FormEvent) => Promise<void>;
  confirmDeleteWallet: () => Promise<void>;
  selectWallet: (wallet: CryptoWallet) => Promise<void>;

  // Actions - Transactions
  handleAddCryptoTransaction: (e: FormEvent) => Promise<void>;
  handleAddTransfer: (e: FormEvent) => Promise<void>;
  handleAddSwap: (e: FormEvent) => Promise<void>;
  confirmDeleteCryptoTx: () => Promise<void>;
  resetTransactionForm: () => void;
  resetTransferForm: () => void;
  resetSwapForm: () => void;
  selectCoinForTransaction: (coin: { id: string; symbol: string }) => void;

  // Actions - Legacy Holdings
  addHolding: (e: FormEvent) => Promise<void>;
  confirmDeleteHolding: () => Promise<void>;
  selectCoinForHolding: (coin: { id: string; symbol: string }) => void;

  // Actions - Reset
  resetState: () => void;
}

interface UseCryptoOptions {
  onSuccess?: (message: string) => void;
  onError?: (message: string) => void;
}

export function useCrypto(options: UseCryptoOptions = {}): UseCryptoReturn {
  const { onSuccess } = options;

  // General state
  const [cryptoLoading, setCryptoLoading] = useState(false);
  const [cryptoError, setCryptoError] = useState("");
  const [cryptoSubTab, setCryptoSubTab] = useState<CryptoSubTab>("overview");

  // Prices & Watchlist state
  const [cryptoAssets, setCryptoAssets] = useState<CryptoAsset[]>([]);
  const [trackedCoins, setTrackedCoins] = useState<string[]>(
    DEFAULT_TRACKED_COINS,
  );
  const [showAddCrypto, setShowAddCrypto] = useState(false);
  const [cryptoSearchQuery, setCryptoSearchQuery] = useState("");

  // Wallets state
  const [wallets, setWallets] = useState<CryptoWallet[]>([]);
  const [selectedWallet, setSelectedWallet] = useState<CryptoWallet | null>(
    null,
  );
  const [walletTransactions, setWalletTransactions] = useState<
    CryptoTransaction[]
  >([]);
  const [walletHoldings, setWalletHoldings] = useState<AggregatedAsset[]>([]);
  const [aggregatedPortfolio, setAggregatedPortfolio] = useState<
    AggregatedAsset[]
  >([]);
  const [showAddWallet, setShowAddWallet] = useState(false);
  const [walletToDelete, setWalletToDelete] = useState<string | null>(null);

  // Wallet form state
  const [walletName, setWalletName] = useState("");
  const [walletCategory, setWalletCategory] = useState("exchange");
  const [walletIcon, setWalletIcon] = useState("🏦");

  // Transaction modal state
  const [showAddTransaction, setShowAddTransaction] = useState(false);
  const [txWalletId, setTxWalletId] = useState("");
  const [txCoinId, setTxCoinId] = useState("");
  const [txSymbol, setTxSymbol] = useState("");
  const [txType, setTxType] = useState("buy");
  const [txAmount, setTxAmount] = useState("");
  const [txPrice, setTxPrice] = useState("");
  const [txFee, setTxFee] = useState("");
  const [txDate, setTxDate] = useState(() => getLocalDateString());
  const [txNotes, setTxNotes] = useState("");
  const [cryptoTxToDelete, setCryptoTxToDelete] = useState<string | null>(null);

  // Transfer modal state
  const [showTransferModal, setShowTransferModal] = useState(false);
  const [transferFromWallet, setTransferFromWallet] = useState("");
  const [transferToWallet, setTransferToWallet] = useState("");
  const [transferCoinId, setTransferCoinId] = useState("");
  const [transferSymbol, setTransferSymbol] = useState("");
  const [transferAmount, setTransferAmount] = useState("");
  const [transferFee, setTransferFee] = useState("");
  const [transferDate, setTransferDate] = useState(() => getLocalDateString());

  // Swap modal state
  const [showSwapModal, setShowSwapModal] = useState(false);
  const [swapWalletId, setSwapWalletId] = useState("");
  const [swapFromCoinId, setSwapFromCoinId] = useState("");
  const [swapFromSymbol, setSwapFromSymbol] = useState("");
  const [swapFromAmount, setSwapFromAmount] = useState("");
  const [swapToCoinId, setSwapToCoinId] = useState("");
  const [swapToSymbol, setSwapToSymbol] = useState("");
  const [swapToAmount, setSwapToAmount] = useState("");
  const [swapFee, setSwapFee] = useState("");
  const [swapDate, setSwapDate] = useState(() => getLocalDateString());

  // Legacy holdings state
  const [holdings, setHoldings] = useState<CryptoHolding[]>([]);
  const [showAddHolding, setShowAddHolding] = useState(false);
  const [holdingCoinId, setHoldingCoinId] = useState("");
  const [holdingSymbol, setHoldingSymbol] = useState("");
  const [holdingAmount, setHoldingAmount] = useState("");
  const [holdingPrice, setHoldingPrice] = useState("");
  const [holdingDate, setHoldingDate] = useState(() => getLocalDateString());
  const [holdingToDelete, setHoldingToDelete] = useState<string | null>(null);

  // ==================== Computed Values ====================

  const filteredSuggestions = useMemo(() => {
    const query = cryptoSearchQuery.toLowerCase().trim();
    if (!query)
      return POPULAR_CRYPTOS.filter((c) => !trackedCoins.includes(c.id));
    return POPULAR_CRYPTOS.filter(
      (c) =>
        !trackedCoins.includes(c.id) &&
        (c.id.includes(query) ||
          c.symbol.toLowerCase().includes(query) ||
          c.name.toLowerCase().includes(query)),
    );
  }, [cryptoSearchQuery, trackedCoins]);

  const enrichedPortfolio = useMemo(
    (): AggregatedAsset[] =>
      enrichAssetsWithPrices(aggregatedPortfolio, cryptoAssets),
    [aggregatedPortfolio, cryptoAssets],
  );

  const enrichedWalletHoldings = useMemo(
    (): AggregatedAsset[] =>
      enrichAssetsWithPrices(walletHoldings, cryptoAssets),
    [walletHoldings, cryptoAssets],
  );

  const portfolioTotals = useMemo(() => {
    const totalValue = enrichedPortfolio.reduce<number>(
      (sum: number, item: AggregatedAsset) => sum + item.current_value,
      0,
    );
    const totalCost = enrichedPortfolio.reduce<number>(
      (sum: number, item: AggregatedAsset) => sum + item.total_cost_basis,
      0,
    );
    const totalPnl = totalValue - totalCost;
    const totalPnlPercentage = totalCost > 0 ? (totalPnl / totalCost) * 100 : 0;

    return { totalValue, totalCost, totalPnl, totalPnlPercentage };
  }, [enrichedPortfolio]);

  // ==================== Data Loading Functions ====================

  const loadCryptoPrices = useCallback(async () => {
    const holdingCoinIds = holdings.map((h: CryptoHolding) => h.coin_id);
    const portfolioCoinIds = aggregatedPortfolio.map(
      (a: AggregatedAsset) => a.coin_id,
    );
    const allCoins = [
      ...new Set([...trackedCoins, ...holdingCoinIds, ...portfolioCoinIds]),
    ];

    if (allCoins.length === 0) {
      setCryptoAssets([]);
      return;
    }
    try {
      setCryptoLoading(true);
      setCryptoError("");
      const CHUNK_SIZE = 50;
      const results: CryptoAsset[] = [];

      for (let i = 0; i < allCoins.length; i += CHUNK_SIZE) {
        const chunk = allCoins.slice(i, i + CHUNK_SIZE);
        if (i > 0) {
          // Avoid tripping API client rate limit
          await new Promise((resolve) => setTimeout(resolve, 1600));
        }
        const assets = await invoke<CryptoAsset[]>("get_crypto_prices", {
          coins: chunk,
        });
        results.push(...assets);
      }

      setCryptoAssets(results);
    } catch (err) {
      setCryptoError(String(err));
      console.error("Error loading crypto prices:", err);
    } finally {
      setCryptoLoading(false);
    }
  }, [trackedCoins, holdings, aggregatedPortfolio]);

  const loadWallets = useCallback(async () => {
    try {
      const data = await invoke<CryptoWallet[]>("get_wallets");
      setWallets(data);
    } catch (err) {
      console.error("Error loading wallets:", err);
    }
  }, []);

  const loadAggregatedPortfolio = useCallback(async () => {
    try {
      const data = await invoke<AggregatedAsset[]>("get_aggregated_portfolio");
      setAggregatedPortfolio(data);
    } catch (err) {
      console.error("Error loading aggregated portfolio:", err);
    }
  }, []);

  const loadWalletDetails = useCallback(async (walletId: string) => {
    try {
      const [txs, holdingsData] = await Promise.all([
        invoke<CryptoTransaction[]>("get_wallet_transactions", { walletId }),
        invoke<AggregatedAsset[]>("get_wallet_holdings", { walletId }),
      ]);
      setWalletTransactions(txs);
      setWalletHoldings(holdingsData);
    } catch (err) {
      console.error("Error loading wallet details:", err);
    }
  }, []);

  const loadHoldings = useCallback(async () => {
    try {
      const data = await invoke<CryptoHolding[]>("get_crypto_holdings");
      setHoldings(data);
    } catch (err) {
      console.error("Error loading holdings:", err);
    }
  }, []);

  // ==================== Watchlist Functions ====================

  const addTrackedCoin = useCallback(
    (coinId: string) => {
      const normalized = coinId.toLowerCase().trim();

      if (!normalized || trackedCoins.includes(normalized)) return;

      if (trackedCoins.length >= MAX_TRACKED_COINS) {
        setCryptoError(`Maximum ${MAX_TRACKED_COINS} coins allowed`);
        return;
      }

      if (!isValidCoinId(normalized)) {
        setCryptoError("Invalid coin ID format");
        return;
      }

      setTrackedCoins((prev: string[]) => [...prev, normalized]);
      setShowAddCrypto(false);
      setCryptoSearchQuery("");
    },
    [trackedCoins],
  );

  const removeTrackedCoin = useCallback((coinId: string) => {
    setTrackedCoins((prev: string[]) => prev.filter((id) => id !== coinId));
    setCryptoAssets((prev: CryptoAsset[]) =>
      prev.filter((asset) => asset.id !== coinId),
    );
  }, []);

  // ==================== Form Reset Functions ====================

  const resetTransactionForm = useCallback(() => {
    setTxWalletId("");
    setTxCoinId("");
    setTxSymbol("");
    setTxType("buy");
    setTxAmount("");
    setTxPrice("");
    setTxFee("");
    setTxDate(getLocalDateString());
    setTxNotes("");
  }, []);

  const resetTransferForm = useCallback(() => {
    setTransferFromWallet("");
    setTransferToWallet("");
    setTransferCoinId("");
    setTransferSymbol("");
    setTransferAmount("");
    setTransferFee("");
    setTransferDate(getLocalDateString());
  }, []);

  const resetSwapForm = useCallback(() => {
    setSwapWalletId("");
    setSwapFromCoinId("");
    setSwapFromSymbol("");
    setSwapFromAmount("");
    setSwapToCoinId("");
    setSwapToSymbol("");
    setSwapToAmount("");
    setSwapFee("");
    setSwapDate(getLocalDateString());
  }, []);

  // ==================== Wallet Management Functions ====================

  const handleAddWallet = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();

      if (!walletName.trim()) {
        setCryptoError("Wallet name cannot be empty");
        return;
      }

      try {
        setCryptoLoading(true);
        setCryptoError("");
        await invoke("add_wallet", {
          name: walletName.trim(),
          category: walletCategory,
          icon: walletIcon,
        });

        setWalletName("");
        setWalletCategory("exchange");
        setWalletIcon("🏦");
        setShowAddWallet(false);

        await loadWallets();
        onSuccess?.("Wallet created successfully");
      } catch (err) {
        setCryptoError(String(err));
      } finally {
        setCryptoLoading(false);
      }
    },
    [walletName, walletCategory, walletIcon, loadWallets, onSuccess],
  );

  const confirmDeleteWallet = useCallback(async () => {
    if (!walletToDelete) return;

    try {
      setCryptoLoading(true);
      await invoke("delete_wallet", { id: walletToDelete });
      await loadWallets();
      await loadAggregatedPortfolio();
      if (selectedWallet?.id === walletToDelete) {
        setSelectedWallet(null);
        setWalletTransactions([]);
        setWalletHoldings([]);
      }
      onSuccess?.("Wallet deleted successfully");
    } catch (err) {
      setCryptoError(String(err));
    } finally {
      setCryptoLoading(false);
      setWalletToDelete(null);
    }
  }, [
    walletToDelete,
    loadWallets,
    loadAggregatedPortfolio,
    selectedWallet,
    onSuccess,
  ]);

  const selectWallet = useCallback(
    async (wallet: CryptoWallet) => {
      setSelectedWallet(wallet);
      await loadWalletDetails(wallet.id);
    },
    [loadWalletDetails],
  );

  // ==================== Transaction Functions ====================

  const selectCoinForTransaction = useCallback(
    (coin: { id: string; symbol: string }) => {
      setTxCoinId(coin.id);
      setTxSymbol(coin.symbol);
      const asset = cryptoAssets.find((a: CryptoAsset) => a.id === coin.id);
      if (asset && (txType === "buy" || txType === "sell")) {
        setTxPrice(asset.current_price.toString());
      }
    },
    [cryptoAssets, txType],
  );

  const handleAddCryptoTransaction = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();

      const parsedAmount = parseFloat(txAmount);
      const parsedPrice = txPrice ? parseFloat(txPrice) : null;
      const parsedFee = txFee ? parseFloat(txFee) : null;

      if (!txWalletId) {
        setCryptoError("Please select a wallet");
        return;
      }
      if (!txCoinId.trim()) {
        setCryptoError("Please select a coin");
        return;
      }
      if (isNaN(parsedAmount) || parsedAmount <= 0) {
        setCryptoError("Amount must be greater than zero");
        return;
      }

      try {
        setCryptoLoading(true);
        setCryptoError("");
        await invoke("add_crypto_transaction", {
          walletId: txWalletId,
          coinId: txCoinId.trim().toLowerCase(),
          symbol: txSymbol.trim().toUpperCase(),
          transactionType: txType,
          amount: parsedAmount,
          pricePerCoin: parsedPrice,
          fee: parsedFee,
          date: txDate,
          notes: txNotes.trim() || null,
        });

        resetTransactionForm();
        setShowAddTransaction(false);

        await loadAggregatedPortfolio();
        if (selectedWallet) {
          await loadWalletDetails(selectedWallet.id);
        }
        await loadCryptoPrices();
        onSuccess?.("Transaction added successfully");
      } catch (err) {
        setCryptoError(String(err));
      } finally {
        setCryptoLoading(false);
      }
    },
    [
      txWalletId,
      txCoinId,
      txSymbol,
      txType,
      txAmount,
      txPrice,
      txFee,
      txDate,
      txNotes,
      resetTransactionForm,
      loadAggregatedPortfolio,
      selectedWallet,
      loadWalletDetails,
      loadCryptoPrices,
      onSuccess,
    ],
  );

  const handleAddTransfer = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();

      const parsedAmount = parseFloat(transferAmount);
      const parsedFee = transferFee ? parseFloat(transferFee) : null;

      if (!transferFromWallet || !transferToWallet) {
        setCryptoError("Please select both wallets");
        return;
      }
      if (transferFromWallet === transferToWallet) {
        setCryptoError("Cannot transfer to the same wallet");
        return;
      }
      if (!transferCoinId.trim()) {
        setCryptoError("Please select a coin");
        return;
      }
      if (isNaN(parsedAmount) || parsedAmount <= 0) {
        setCryptoError("Amount must be greater than zero");
        return;
      }

      try {
        setCryptoLoading(true);
        setCryptoError("");
        await invoke("add_transfer_transaction", {
          fromWalletId: transferFromWallet,
          toWalletId: transferToWallet,
          coinId: transferCoinId.trim().toLowerCase(),
          symbol: transferSymbol.trim().toUpperCase(),
          amount: parsedAmount,
          fee: parsedFee,
          date: transferDate,
          notes: null,
        });

        resetTransferForm();
        setShowTransferModal(false);

        await loadAggregatedPortfolio();
        if (selectedWallet) {
          await loadWalletDetails(selectedWallet.id);
        }
        onSuccess?.("Transfer recorded successfully");
      } catch (err) {
        setCryptoError(String(err));
      } finally {
        setCryptoLoading(false);
      }
    },
    [
      transferFromWallet,
      transferToWallet,
      transferCoinId,
      transferSymbol,
      transferAmount,
      transferFee,
      transferDate,
      resetTransferForm,
      loadAggregatedPortfolio,
      selectedWallet,
      loadWalletDetails,
      onSuccess,
    ],
  );

  const handleAddSwap = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();

      const parsedFromAmount = parseFloat(swapFromAmount);
      const parsedToAmount = parseFloat(swapToAmount);
      const parsedFee = swapFee ? parseFloat(swapFee) : null;

      if (!swapWalletId) {
        setCryptoError("Please select a wallet");
        return;
      }
      if (!swapFromCoinId.trim() || !swapToCoinId.trim()) {
        setCryptoError("Please select both coins");
        return;
      }
      if (swapFromCoinId === swapToCoinId) {
        setCryptoError("Cannot swap the same coin");
        return;
      }
      if (isNaN(parsedFromAmount) || parsedFromAmount <= 0) {
        setCryptoError("From amount must be greater than zero");
        return;
      }
      if (isNaN(parsedToAmount) || parsedToAmount <= 0) {
        setCryptoError("To amount must be greater than zero");
        return;
      }

      try {
        setCryptoLoading(true);
        setCryptoError("");
        await invoke("add_swap_transaction", {
          walletId: swapWalletId,
          fromCoinId: swapFromCoinId.trim().toLowerCase(),
          fromSymbol: swapFromSymbol.trim().toUpperCase(),
          fromAmount: parsedFromAmount,
          toCoinId: swapToCoinId.trim().toLowerCase(),
          toSymbol: swapToSymbol.trim().toUpperCase(),
          toAmount: parsedToAmount,
          fee: parsedFee,
          feeCoinId: null,
          feeAmount: null,
          date: swapDate,
          notes: null,
        });

        resetSwapForm();
        setShowSwapModal(false);

        await loadAggregatedPortfolio();
        if (selectedWallet) {
          await loadWalletDetails(selectedWallet.id);
        }
        await loadCryptoPrices();
        onSuccess?.("Swap recorded successfully");
      } catch (err) {
        setCryptoError(String(err));
      } finally {
        setCryptoLoading(false);
      }
    },
    [
      swapWalletId,
      swapFromCoinId,
      swapFromSymbol,
      swapFromAmount,
      swapToCoinId,
      swapToSymbol,
      swapToAmount,
      swapFee,
      swapDate,
      resetSwapForm,
      loadAggregatedPortfolio,
      selectedWallet,
      loadWalletDetails,
      loadCryptoPrices,
      onSuccess,
    ],
  );

  const confirmDeleteCryptoTx = useCallback(async () => {
    if (!cryptoTxToDelete) return;

    try {
      setCryptoLoading(true);
      await invoke("delete_crypto_transaction", { id: cryptoTxToDelete });
      await loadAggregatedPortfolio();
      if (selectedWallet) {
        await loadWalletDetails(selectedWallet.id);
      }
      onSuccess?.("Transaction deleted successfully");
    } catch (err) {
      setCryptoError(String(err));
    } finally {
      setCryptoLoading(false);
      setCryptoTxToDelete(null);
    }
  }, [
    cryptoTxToDelete,
    loadAggregatedPortfolio,
    selectedWallet,
    loadWalletDetails,
    onSuccess,
  ]);

  // ==================== Legacy Holdings Functions ====================

  const selectCoinForHolding = useCallback(
    (coin: { id: string; symbol: string }) => {
      setHoldingCoinId(coin.id);
      setHoldingSymbol(coin.symbol);
      const asset = cryptoAssets.find((a: CryptoAsset) => a.id === coin.id);
      if (asset) {
        setHoldingPrice(asset.current_price.toString());
      }
    },
    [cryptoAssets],
  );

  const addHolding = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      const parsedAmount = parseFloat(holdingAmount);
      const parsedPrice = parseFloat(holdingPrice);

      if (!holdingCoinId.trim()) {
        setCryptoError("Please select a coin");
        return;
      }
      if (isNaN(parsedAmount) || parsedAmount <= 0) {
        setCryptoError("Amount must be greater than zero");
        return;
      }
      if (isNaN(parsedPrice) || parsedPrice < 0) {
        setCryptoError("Invalid purchase price");
        return;
      }

      try {
        setCryptoLoading(true);
        setCryptoError("");
        await invoke("add_crypto_holding", {
          coinId: holdingCoinId.trim().toLowerCase(),
          symbol: holdingSymbol.trim().toUpperCase(),
          amount: parsedAmount,
          purchasePrice: parsedPrice,
          purchaseDate: holdingDate,
        });

        setHoldingCoinId("");
        setHoldingSymbol("");
        setHoldingAmount("");
        setHoldingPrice("");
        setHoldingDate(getLocalDateString());
        setShowAddHolding(false);

        await loadHoldings();
        await loadCryptoPrices();
      } catch (err) {
        setCryptoError(String(err));
      } finally {
        setCryptoLoading(false);
      }
    },
    [
      holdingCoinId,
      holdingSymbol,
      holdingAmount,
      holdingPrice,
      holdingDate,
      loadHoldings,
      loadCryptoPrices,
    ],
  );

  const confirmDeleteHolding = useCallback(async () => {
    if (!holdingToDelete) return;

    try {
      setCryptoLoading(true);
      await invoke("delete_crypto_holding", { id: holdingToDelete });
      await loadHoldings();
    } catch (err) {
      setCryptoError(String(err));
    } finally {
      setCryptoLoading(false);
      setHoldingToDelete(null);
    }
  }, [holdingToDelete, loadHoldings]);

  // ==================== Reset State ====================

  const resetState = useCallback(() => {
    setWallets([]);
    setAggregatedPortfolio([]);
    setHoldings([]);
    setSelectedWallet(null);
    setWalletTransactions([]);
    setWalletHoldings([]);
    setCryptoAssets([]);
    setCryptoError("");
    setCryptoSubTab("overview");
  }, []);

  return {
    // State - General
    cryptoLoading,
    cryptoError,
    cryptoSubTab,
    setCryptoSubTab,
    setCryptoError,

    // State - Prices & Watchlist
    cryptoAssets,
    trackedCoins,
    showAddCrypto,
    cryptoSearchQuery,
    setShowAddCrypto,
    setCryptoSearchQuery,
    filteredSuggestions,

    // State - Wallets
    wallets,
    selectedWallet,
    walletTransactions,
    walletHoldings,
    showAddWallet,
    walletToDelete,
    setShowAddWallet,
    setWalletToDelete,
    setSelectedWallet,

    // State - Wallet Form
    walletName,
    walletCategory,
    walletIcon,
    setWalletName,
    setWalletCategory,
    setWalletIcon,

    // State - Transaction Modal
    showAddTransaction,
    setShowAddTransaction,
    txWalletId,
    txCoinId,
    txSymbol,
    txType,
    txAmount,
    txPrice,
    txFee,
    txDate,
    txNotes,
    setTxWalletId,
    setTxCoinId,
    setTxSymbol,
    setTxType,
    setTxAmount,
    setTxPrice,
    setTxFee,
    setTxDate,
    setTxNotes,
    cryptoTxToDelete,
    setCryptoTxToDelete,

    // State - Transfer Modal
    showTransferModal,
    setShowTransferModal,
    transferFromWallet,
    transferToWallet,
    transferCoinId,
    transferSymbol,
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

    // State - Swap Modal
    showSwapModal,
    setShowSwapModal,
    swapWalletId,
    swapFromCoinId,
    swapFromSymbol,
    swapFromAmount,
    swapToCoinId,
    swapToSymbol,
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

    // State - Legacy Holdings
    holdings,
    showAddHolding,
    holdingCoinId,
    holdingSymbol,
    holdingAmount,
    holdingPrice,
    holdingDate,
    holdingToDelete,
    setShowAddHolding,
    setHoldingCoinId,
    setHoldingSymbol,
    setHoldingAmount,
    setHoldingPrice,
    setHoldingDate,
    setHoldingToDelete,

    // Computed
    aggregatedPortfolio,
    enrichedPortfolio,
    enrichedWalletHoldings,
    portfolioTotals,

    // Actions - Data Loading
    loadCryptoPrices,
    loadWallets,
    loadAggregatedPortfolio,
    loadWalletDetails,
    loadHoldings,

    // Actions - Watchlist
    addTrackedCoin,
    removeTrackedCoin,

    // Actions - Wallets
    handleAddWallet,
    confirmDeleteWallet,
    selectWallet,

    // Actions - Transactions
    handleAddCryptoTransaction,
    handleAddTransfer,
    handleAddSwap,
    confirmDeleteCryptoTx,
    resetTransactionForm,
    resetTransferForm,
    resetSwapForm,
    selectCoinForTransaction,

    // Actions - Legacy Holdings
    addHolding,
    confirmDeleteHolding,
    selectCoinForHolding,

    // Actions - Reset
    resetState,
  };
}
