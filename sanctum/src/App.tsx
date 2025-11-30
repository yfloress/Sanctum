import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from "recharts";
import "./App.css";

// Colors for pie chart (cyberpunk/neon theme)
const CHART_COLORS = [
  "#8b5cf6", // violet
  "#10b981", // emerald
  "#f59e0b", // amber
  "#ef4444", // red
  "#06b6d4", // cyan
  "#ec4899", // pink
  "#6366f1", // indigo
  "#84cc16", // lime
  "#f97316", // orange
];

// ==================== Interfaces ====================

interface Transaction {
  id: string;
  amount: number;
  category: string;
  description: string;
  date: string;
  type: string;
}

interface BalanceSummary {
  total_balance: number;
  total_income: number;
  total_expense: number;
}

interface CryptoAsset {
  id: string;
  symbol: string;
  name: string;
  current_price: number;
  price_change_percentage_24h: number;
  last_updated: string;
}

// Legacy interface - kept for backwards compatibility
interface CryptoHolding {
  id: string;
  coin_id: string;
  symbol: string;
  amount: number;
  purchase_price: number;
  purchase_date: string;
}

// New Ledger System interfaces
interface CryptoWallet {
  id: string;
  name: string;
  category: string; // "exchange" | "wallet_single" | "wallet_multi"
  icon: string | null;
}

interface CryptoTransaction {
  id: string;
  wallet_id: string;
  coin_id: string;
  symbol: string;
  type: string; // "buy" | "sell" | "transfer_in" | "transfer_out" | "swap"
  amount: number;
  price_per_coin: number | null;
  fee: number | null;
  fee_coin_id: string | null;
  fee_amount: number | null;
  date: string;
  notes: string | null;
  related_tx_id: string | null;
}

interface AggregatedAsset {
  coin_id: string;
  symbol: string;
  total_amount: number;
  total_cost_basis: number;
  avg_buy_price: number;
  current_price: number;
  current_value: number;
  unrealized_pnl: number;
  unrealized_pnl_percentage: number;
}

// ==================== Constants ====================

const MAX_TRACKED_COINS = 20;

const POPULAR_CRYPTOS = [
  // Top Tier
  { id: "bitcoin", symbol: "BTC", name: "Bitcoin" },
  { id: "ethereum", symbol: "ETH", name: "Ethereum" },
  // Stablecoins (MUST HAVE)
  { id: "tether", symbol: "USDT", name: "Tether" },
  { id: "usd-coin", symbol: "USDC", name: "USD Coin" },
  { id: "dai", symbol: "DAI", name: "Dai" },
  // Privacy Coins
  { id: "monero", symbol: "XMR", name: "Monero" },
  { id: "zcash", symbol: "ZEC", name: "Zcash" },
  { id: "dash", symbol: "DASH", name: "Dash" },
  // Major Altcoins
  { id: "litecoin", symbol: "LTC", name: "Litecoin" },
  { id: "ripple", symbol: "XRP", name: "XRP" },
  { id: "binancecoin", symbol: "BNB", name: "BNB" },
  { id: "solana", symbol: "SOL", name: "Solana" },
  { id: "cardano", symbol: "ADA", name: "Cardano" },
  { id: "dogecoin", symbol: "DOGE", name: "Dogecoin" },
  { id: "polkadot", symbol: "DOT", name: "Polkadot" },
  { id: "avalanche-2", symbol: "AVAX", name: "Avalanche" },
  { id: "chainlink", symbol: "LINK", name: "Chainlink" },
  { id: "matic-network", symbol: "MATIC", name: "Polygon" },
  { id: "tron", symbol: "TRX", name: "TRON" },
  { id: "uniswap", symbol: "UNI", name: "Uniswap" },
  { id: "cosmos", symbol: "ATOM", name: "Cosmos" },
  { id: "stellar", symbol: "XLM", name: "Stellar" },
  { id: "near", symbol: "NEAR", name: "NEAR Protocol" },
  { id: "algorand", symbol: "ALGO", name: "Algorand" },
  // Additional Privacy/Security
  { id: "decred", symbol: "DCR", name: "Decred" },
  { id: "horizen", symbol: "ZEN", name: "Horizen" },
  { id: "secret", symbol: "SCRT", name: "Secret" },
  { id: "oasis-network", symbol: "ROSE", name: "Oasis Network" },
];

// Default coins to show prices (BTC, XMR, LTC)
const DEFAULT_TRACKED_COINS = ["bitcoin", "monero", "litecoin"];

const EXPENSE_CATEGORIES = [
  "Food",
  "Transport",
  "Housing",
  "Utilities",
  "Health",
  "Entertainment",
  "Education",
  "Technology",
  "Other",
] as const;

const INCOME_CATEGORIES = [
  "Salary",
  "Freelance",
  "Investments",
  "Gifts",
  "Other",
] as const;

const WALLET_CATEGORIES = [
  { value: "exchange", label: "Exchange", icon: "🏦" },
  { value: "wallet_single", label: "Single-Coin Wallet", icon: "💳" },
  { value: "wallet_multi", label: "Multi-Coin Wallet", icon: "👛" },
];

const WALLET_ICONS = [
  "🏦",
  "💳",
  "👛",
  "🔒",
  "💎",
  "🌐",
  "📱",
  "💻",
  "🔑",
  "⚡",
];

const TRANSACTION_TYPES = [
  { value: "buy", label: "Buy", icon: "📥" },
  { value: "sell", label: "Sell", icon: "📤" },
  { value: "transfer_in", label: "Transfer In", icon: "⬇️" },
  { value: "transfer_out", label: "Transfer Out", icon: "⬆️" },
  { value: "swap", label: "Swap", icon: "🔄" },
];

// ==================== Helper Functions ====================

/** Obtiene la fecha local en formato YYYY-MM-DD sin problemas de timezone */
function getLocalDateString(date: Date = new Date()): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

// ==================== Main App Component ====================

function App() {
  // Auth state
  const [isInitialized, setIsInitialized] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [dbPathInput, setDbPathInput] = useState("");
  const [successMessage, setSuccessMessage] = useState("");
  const [loadingAction, setLoadingAction] = useState<"open" | "create" | null>(
    null,
  );

  // Financial transaction state
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [category, setCategory] = useState<string>(EXPENSE_CATEGORIES[0]);
  const [isExpense, setIsExpense] = useState(true);
  const [date, setDate] = useState(() => getLocalDateString());
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [activeTab, setActiveTab] = useState<
    "dashboard" | "transactions" | "analytics" | "crypto"
  >("dashboard");
  const [balance, setBalance] = useState<BalanceSummary>({
    total_balance: 0,
    total_income: 0,
    total_expense: 0,
  });
  const [transactionToDelete, setTransactionToDelete] = useState<string | null>(
    null,
  );

  // Crypto state - prices and watchlist
  const [cryptoAssets, setCryptoAssets] = useState<CryptoAsset[]>([]);
  const [cryptoLoading, setCryptoLoading] = useState(false);
  const [cryptoError, setCryptoError] = useState("");
  const [trackedCoins, setTrackedCoins] = useState<string[]>(
    DEFAULT_TRACKED_COINS,
  );
  const [showAddCrypto, setShowAddCrypto] = useState(false);
  const [cryptoSearchQuery, setCryptoSearchQuery] = useState("");

  // Crypto Ledger state - wallets and transactions
  const [cryptoSubTab, setCryptoSubTab] = useState<"overview" | "wallets">(
    "overview",
  );
  const [wallets, setWallets] = useState<CryptoWallet[]>([]);
  const [aggregatedPortfolio, setAggregatedPortfolio] = useState<
    AggregatedAsset[]
  >([]);
  const [selectedWallet, setSelectedWallet] = useState<CryptoWallet | null>(
    null,
  );
  const [walletTransactions, setWalletTransactions] = useState<
    CryptoTransaction[]
  >([]);
  const [walletHoldings, setWalletHoldings] = useState<AggregatedAsset[]>([]);

  // Modal states for crypto
  const [showAddWallet, setShowAddWallet] = useState(false);
  const [showAddTransaction, setShowAddTransaction] = useState(false);
  const [showTransferModal, setShowTransferModal] = useState(false);
  const [showSwapModal, setShowSwapModal] = useState(false);
  const [walletToDelete, setWalletToDelete] = useState<string | null>(null);
  const [cryptoTxToDelete, setCryptoTxToDelete] = useState<string | null>(null);

  // Wallet form state
  const [walletName, setWalletName] = useState("");
  const [walletCategory, setWalletCategory] = useState("exchange");
  const [walletIcon, setWalletIcon] = useState("🏦");

  // Transaction form state
  const [txWalletId, setTxWalletId] = useState("");
  const [txCoinId, setTxCoinId] = useState("");
  const [txSymbol, setTxSymbol] = useState("");
  const [txType, setTxType] = useState("buy");
  const [txAmount, setTxAmount] = useState("");
  const [txPrice, setTxPrice] = useState("");
  const [txFee, setTxFee] = useState("");
  const [txDate, setTxDate] = useState(() => getLocalDateString());
  const [txNotes, setTxNotes] = useState("");

  // Transfer form state
  const [transferFromWallet, setTransferFromWallet] = useState("");
  const [transferToWallet, setTransferToWallet] = useState("");
  const [transferCoinId, setTransferCoinId] = useState("");
  const [transferSymbol, setTransferSymbol] = useState("");
  const [transferAmount, setTransferAmount] = useState("");
  const [transferFee, setTransferFee] = useState("");
  const [transferDate, setTransferDate] = useState(() => getLocalDateString());

  // Swap form state
  const [swapWalletId, setSwapWalletId] = useState("");
  const [swapFromCoinId, setSwapFromCoinId] = useState("");
  const [swapFromSymbol, setSwapFromSymbol] = useState("");
  const [swapFromAmount, setSwapFromAmount] = useState("");
  const [swapToCoinId, setSwapToCoinId] = useState("");
  const [swapToSymbol, setSwapToSymbol] = useState("");
  const [swapToAmount, setSwapToAmount] = useState("");
  const [swapFee, setSwapFee] = useState("");
  const [swapDate, setSwapDate] = useState(() => getLocalDateString());

  // Legacy portfolio state (for backwards compatibility during migration)
  const [holdings, setHoldings] = useState<CryptoHolding[]>([]);
  const [showAddHolding, setShowAddHolding] = useState(false);
  const [holdingCoinId, setHoldingCoinId] = useState("");
  const [holdingSymbol, setHoldingSymbol] = useState("");
  const [holdingAmount, setHoldingAmount] = useState("");
  const [holdingPrice, setHoldingPrice] = useState("");
  const [holdingDate, setHoldingDate] = useState(() => getLocalDateString());
  const [holdingToDelete, setHoldingToDelete] = useState<string | null>(null);

  // ==================== Memoized Values ====================

  const expensesByCategory = useMemo(() => {
    const expenses = transactions.filter((tx) => tx.type === "expense");
    const grouped = expenses.reduce(
      (acc, tx) => {
        acc[tx.category] = (acc[tx.category] || 0) + tx.amount;
        return acc;
      },
      {} as Record<string, number>,
    );

    return Object.entries(grouped)
      .map(([name, value]) => ({ name, value: value / 100 }))
      .sort((a, b) => b.value - a.value);
  }, [transactions]);

  // Aggregated portfolio with prices
  const enrichedPortfolio = useMemo((): AggregatedAsset[] => {
    return aggregatedPortfolio.map((asset) => {
      const priceData = cryptoAssets.find((a) => a.id === asset.coin_id);
      const currentPrice = priceData?.current_price ?? 0;
      const currentValue = asset.total_amount * currentPrice;
      const unrealizedPnl = currentValue - asset.total_cost_basis;
      const unrealizedPnlPercentage =
        asset.total_cost_basis > 0
          ? (unrealizedPnl / asset.total_cost_basis) * 100
          : 0;

      return {
        ...asset,
        current_price: currentPrice,
        current_value: currentValue,
        unrealized_pnl: unrealizedPnl,
        unrealized_pnl_percentage: unrealizedPnlPercentage,
      };
    });
  }, [aggregatedPortfolio, cryptoAssets]);

  // Portfolio totals from aggregated data
  const portfolioTotals = useMemo(() => {
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
  }, [enrichedPortfolio]);

  // Wallet holdings with prices
  const enrichedWalletHoldings = useMemo((): AggregatedAsset[] => {
    return walletHoldings.map((asset) => {
      const priceData = cryptoAssets.find((a) => a.id === asset.coin_id);
      const currentPrice = priceData?.current_price ?? 0;
      const currentValue = asset.total_amount * currentPrice;
      const unrealizedPnl = currentValue - asset.total_cost_basis;
      const unrealizedPnlPercentage =
        asset.total_cost_basis > 0
          ? (unrealizedPnl / asset.total_cost_basis) * 100
          : 0;

      return {
        ...asset,
        current_price: currentPrice,
        current_value: currentValue,
        unrealized_pnl: unrealizedPnl,
        unrealized_pnl_percentage: unrealizedPnlPercentage,
      };
    });
  }, [walletHoldings, cryptoAssets]);

  const balanceEvolution = useMemo(() => {
    if (transactions.length === 0) return [];

    const sorted = [...transactions].sort(
      (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
    );

    const dailyData: Record<string, { income: number; expense: number }> = {};

    sorted.forEach((tx) => {
      const [, month, day] = tx.date.split("T")[0].split("-");
      const months = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
      ];
      const dateKey = `${months[parseInt(month) - 1]} ${parseInt(day)}`;

      if (!dailyData[dateKey]) {
        dailyData[dateKey] = { income: 0, expense: 0 };
      }

      if (tx.type === "income") {
        dailyData[dateKey].income += tx.amount;
      } else {
        dailyData[dateKey].expense += tx.amount;
      }
    });

    let cumulative = 0;
    return Object.entries(dailyData).map(([date, data]) => {
      cumulative += (data.income - data.expense) / 100;
      return {
        date,
        balance: cumulative,
        income: data.income / 100,
        expense: data.expense / 100,
      };
    });
  }, [transactions]);

  const categories = useMemo(
    () => (isExpense ? EXPENSE_CATEGORIES : INCOME_CATEGORIES),
    [isExpense],
  );

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

  // ==================== Refs for timeouts ====================

  const errorTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const successTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // ==================== Helper Functions ====================

  const setTemporaryError = useCallback((message: string, duration = 5000) => {
    if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
    setError(message);
    errorTimeoutRef.current = setTimeout(() => setError(""), duration);
  }, []);

  const setTemporarySuccess = useCallback(
    (message: string, duration = 3000) => {
      if (successTimeoutRef.current) clearTimeout(successTimeoutRef.current);
      setSuccessMessage(message);
      successTimeoutRef.current = setTimeout(
        () => setSuccessMessage(""),
        duration,
      );
    },
    [],
  );

  const clearMessages = useCallback(() => {
    setError("");
    setSuccessMessage("");
  }, []);

  const formatAmount = useCallback(
    (cents: number) => (cents / 100).toFixed(2),
    [],
  );

  const formatDate = useCallback((isoDate: string) => {
    const [year, month, day] = isoDate.split("T")[0].split("-");
    const months = [
      "Jan",
      "Feb",
      "Mar",
      "Apr",
      "May",
      "Jun",
      "Jul",
      "Aug",
      "Sep",
      "Oct",
      "Nov",
      "Dec",
    ];
    return `${months[parseInt(month) - 1]} ${parseInt(day)}, ${year}`;
  }, []);

  const formatCryptoAmount = useCallback((amount: number, decimals = 8) => {
    if (amount >= 1) {
      return amount.toLocaleString(undefined, { maximumFractionDigits: 4 });
    }
    return amount.toLocaleString(undefined, {
      maximumFractionDigits: decimals,
    });
  }, []);

  const formatUSD = useCallback((value: number) => {
    return value.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  }, []);

  const isValidCoinId = useCallback((coinId: string): boolean => {
    if (!coinId || coinId.length > 64) return false;
    if (!/^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/.test(coinId)) return false;
    if (coinId.includes("--")) return false;
    return true;
  }, []);

  const getTransactionTypeLabel = useCallback((type: string) => {
    const found = TRANSACTION_TYPES.find((t) => t.value === type);
    return found ? `${found.icon} ${found.label}` : type;
  }, []);

  const getWalletCategoryLabel = useCallback((category: string) => {
    const found = WALLET_CATEGORIES.find((c) => c.value === category);
    return found ? found.label : category;
  }, []);

  // ==================== Data Loading Functions ====================

  const loadDbPath = useCallback(async () => {
    try {
      const path = await invoke<string>("get_db_path");
      setDbPathInput(path);
    } catch (err) {
      console.error("Error getting path:", err);
    }
  }, []);

  const loadTransactions = useCallback(async () => {
    try {
      const txs = await invoke<Transaction[]>("get_transactions");
      setTransactions(txs);
    } catch (err) {
      console.error("Error loading transactions:", err);
    }
  }, []);

  const loadBalance = useCallback(async () => {
    try {
      const bal = await invoke<BalanceSummary>("get_balance");
      setBalance(bal);
    } catch (err) {
      console.error("Error loading balance:", err);
    }
  }, []);

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
      const [txs, holdings] = await Promise.all([
        invoke<CryptoTransaction[]>("get_wallet_transactions", { walletId }),
        invoke<AggregatedAsset[]>("get_wallet_holdings", { walletId }),
      ]);
      setWalletTransactions(txs);
      setWalletHoldings(holdings);
    } catch (err) {
      console.error("Error loading wallet details:", err);
    }
  }, []);

  const loadCryptoPrices = useCallback(async () => {
    // Combine tracked coins, holdings coins, and aggregated portfolio coins
    const holdingCoinIds = holdings.map((h) => h.coin_id);
    const portfolioCoinIds = aggregatedPortfolio.map((a) => a.coin_id);
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
      const assets = await invoke<CryptoAsset[]>("get_crypto_prices", {
        coins: allCoins,
      });
      setCryptoAssets(assets);
    } catch (err) {
      setCryptoError(String(err));
      console.error("Error loading crypto prices:", err);
    } finally {
      setCryptoLoading(false);
    }
  }, [trackedCoins, holdings, aggregatedPortfolio]);

  // Legacy holdings loader
  const loadHoldings = useCallback(async () => {
    try {
      const data = await invoke<CryptoHolding[]>("get_crypto_holdings");
      setHoldings(data);
    } catch (err) {
      console.error("Error loading holdings:", err);
    }
  }, []);

  // ==================== Crypto Watchlist Functions ====================

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

      setTrackedCoins((prev) => [...prev, normalized]);
      setShowAddCrypto(false);
      setCryptoSearchQuery("");
    },
    [trackedCoins, isValidCoinId],
  );

  const removeTrackedCoin = useCallback((coinId: string) => {
    setTrackedCoins((prev) => prev.filter((id) => id !== coinId));
    setCryptoAssets((prev) => prev.filter((asset) => asset.id !== coinId));
  }, []);

  // ==================== Wallet Management Functions ====================

  const handleAddWallet = useCallback(
    async (e: React.FormEvent) => {
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
        setTemporarySuccess("Wallet created successfully");
      } catch (err) {
        setCryptoError(String(err));
      } finally {
        setCryptoLoading(false);
      }
    },
    [walletName, walletCategory, walletIcon, loadWallets, setTemporarySuccess],
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
      setTemporarySuccess("Wallet deleted successfully");
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
    setTemporarySuccess,
  ]);

  const selectWallet = useCallback(
    async (wallet: CryptoWallet) => {
      setSelectedWallet(wallet);
      await loadWalletDetails(wallet.id);
    },
    [loadWalletDetails],
  );

  // ==================== Crypto Transaction Functions ====================

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

  const handleAddCryptoTransaction = useCallback(
    async (e: React.FormEvent) => {
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
        setTemporarySuccess("Transaction added successfully");
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
      setTemporarySuccess,
    ],
  );

  const resetTransferForm = useCallback(() => {
    setTransferFromWallet("");
    setTransferToWallet("");
    setTransferCoinId("");
    setTransferSymbol("");
    setTransferAmount("");
    setTransferFee("");
    setTransferDate(getLocalDateString());
  }, []);

  const handleAddTransfer = useCallback(
    async (e: React.FormEvent) => {
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
        setTemporarySuccess("Transfer recorded successfully");
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
      setTemporarySuccess,
    ],
  );

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

  const handleAddSwap = useCallback(
    async (e: React.FormEvent) => {
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
        setTemporarySuccess("Swap recorded successfully");
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
      setTemporarySuccess,
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
      setTemporarySuccess("Transaction deleted successfully");
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
    setTemporarySuccess,
  ]);

  // ==================== Legacy Holdings Functions ====================

  const addHolding = useCallback(
    async (e: React.FormEvent) => {
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

  const selectCoinForHolding = useCallback(
    (coin: { id: string; symbol: string }) => {
      setHoldingCoinId(coin.id);
      setHoldingSymbol(coin.symbol);
      const asset = cryptoAssets.find((a) => a.id === coin.id);
      if (asset) {
        setHoldingPrice(asset.current_price.toString());
      }
    },
    [cryptoAssets],
  );

  const selectCoinForTransaction = useCallback(
    (coin: { id: string; symbol: string }) => {
      setTxCoinId(coin.id);
      setTxSymbol(coin.symbol);
      const asset = cryptoAssets.find((a) => a.id === coin.id);
      if (asset && (txType === "buy" || txType === "sell")) {
        setTxPrice(asset.current_price.toString());
      }
    },
    [cryptoAssets, txType],
  );

  // ==================== Auth and DB Functions ====================

  const checkDatabaseStatus = useCallback(async () => {
    try {
      setIsLoading(true);
      setError("");
      const initialized = await invoke<boolean>("is_db_initialized");
      setIsInitialized(initialized);
      await loadDbPath();
      if (initialized) {
        await loadHoldings();
        await loadWallets();
        await loadAggregatedPortfolio();
        await loadTransactions();
        await loadBalance();
      }
    } catch (err) {
      setError(`Error checking status: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, [
    loadDbPath,
    loadTransactions,
    loadBalance,
    loadHoldings,
    loadWallets,
    loadAggregatedPortfolio,
  ]);

  const handleVaultAction = useCallback(
    async (action: "open" | "create") => {
      clearMessages();

      const trimmedPassword = password.trim();
      if (!trimmedPassword) {
        setTemporaryError("Password cannot be empty");
        return;
      }
      if (trimmedPassword.length < 8) {
        setTemporaryError("Password must be at least 8 characters");
        return;
      }

      const targetPath = dbPathInput.trim() || null;

      try {
        setIsLoading(true);
        setLoadingAction(action);
        const command = action === "create" ? "create_db" : "open_db";
        await invoke<string>(command, {
          password: trimmedPassword,
          path: targetPath,
        });
        setIsInitialized(true);
        setPassword("");
        await loadDbPath();
        await loadTransactions();
        await loadBalance();
        await loadHoldings();
        await loadWallets();
        await loadAggregatedPortfolio();
      } catch (err) {
        setTemporaryError(`Error: ${err}`);
      } finally {
        setIsLoading(false);
        setLoadingAction(null);
      }
    },
    [
      password,
      dbPathInput,
      clearMessages,
      loadDbPath,
      loadTransactions,
      loadBalance,
      loadHoldings,
      loadWallets,
      loadAggregatedPortfolio,
      setTemporaryError,
    ],
  );

  const handleCloseVault = useCallback(async () => {
    try {
      setIsLoading(true);
      clearMessages();
      const result = await invoke<string>("close_db");
      setTemporarySuccess(result);
      setIsInitialized(false);
      setTransactions([]);
      setWallets([]);
      setAggregatedPortfolio([]);
      setHoldings([]);
      setSelectedWallet(null);
      await loadDbPath();
    } catch (err) {
      setTemporaryError(`Error: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, [clearMessages, loadDbPath, setTemporaryError, setTemporarySuccess]);

  // ==================== Financial Transaction Functions ====================

  const handleExpenseToggle = useCallback((checked: boolean) => {
    setIsExpense(checked);
    setCategory(checked ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0]);
  }, []);

  const handleDeleteTransaction = useCallback((id: string) => {
    setTransactionToDelete(id);
  }, []);

  const confirmDelete = useCallback(async () => {
    if (!transactionToDelete) return;

    try {
      setIsLoading(true);
      clearMessages();
      await invoke("delete_transaction", { id: transactionToDelete });
      setTemporarySuccess("Transaction deleted successfully");
      await loadTransactions();
      await loadBalance();
    } catch (err) {
      setTemporaryError(`Error deleting transaction: ${err}`);
    } finally {
      setIsLoading(false);
      setTransactionToDelete(null);
    }
  }, [
    transactionToDelete,
    clearMessages,
    loadTransactions,
    loadBalance,
    setTemporaryError,
    setTemporarySuccess,
  ]);

  const cancelDelete = useCallback(() => {
    setTransactionToDelete(null);
  }, []);

  const handleAddTransaction = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      clearMessages();

      const parsedAmount = parseFloat(amount);
      if (!amount || parsedAmount <= 0) {
        setTemporaryError("Amount must be greater than zero");
        return;
      }
      if (!category.trim()) {
        setTemporaryError("Category cannot be empty");
        return;
      }

      try {
        setIsLoading(true);
        const amountInCents = Math.round(parsedAmount * 100);

        // El input type="date" ya devuelve formato YYYY-MM-DD (ISO)
        // Lo pasamos directamente al backend que lo acepta como fallback
        await invoke<string>("add_transaction", {
          amount: amountInCents,
          category: category.trim(),
          description: description.trim(),
          date: date, // YYYY-MM-DD directamente del input
          isExpense,
        });

        setTemporarySuccess(
          `${isExpense ? "Expense" : "Income"} added successfully`,
        );

        setAmount("");
        setDescription("");
        setCategory(isExpense ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0]);
        setDate(getLocalDateString());

        await loadTransactions();
        await loadBalance();
      } catch (err) {
        setTemporaryError(`Error creating transaction: ${err}`);
      } finally {
        setIsLoading(false);
      }
    },
    [
      amount,
      category,
      description,
      date,
      isExpense,
      clearMessages,
      loadTransactions,
      loadBalance,
      setTemporaryError,
      setTemporarySuccess,
    ],
  );

  // ==================== Effects ====================

  useEffect(() => {
    checkDatabaseStatus();
  }, [checkDatabaseStatus]);

  useEffect(() => {
    return () => {
      if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
      if (successTimeoutRef.current) clearTimeout(successTimeoutRef.current);
    };
  }, []);

  // ==================== Render ====================

  if (isLoading && !isInitialized) {
    return (
      <div className="vault-container">
        <div className="vault-card">
          <div className="loader" />
          <p>Checking vault status...</p>
        </div>
      </div>
    );
  }

  if (!isInitialized) {
    return (
      <div className="vault-container">
        {error && <div className="message error login-message">{error}</div>}
        <div className="vault-card login-card">
          <div className="login-layout">
            <div className="login-branding">
              <div className="vault-icon locked">🔒</div>
              <h1>Sanctum</h1>
              <p className="vault-subtitle">Secure Financial Vault</p>
              <p className="vault-tagline">
                Your data is protected with AES-256 encryption
              </p>
            </div>

            <div className="login-form-section">
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  handleVaultAction("open");
                }}
                className="vault-form"
              >
                <div className="form-group password-group">
                  <label htmlFor="password">Master Password</label>
                  <div className="password-input-wrapper">
                    <input
                      id="password"
                      type={showPassword ? "text" : "password"}
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder="Enter your password"
                      disabled={isLoading}
                      autoFocus
                    />
                    <button
                      type="button"
                      className="password-toggle"
                      onClick={() => setShowPassword(!showPassword)}
                      disabled={isLoading}
                      aria-label={
                        showPassword ? "Hide password" : "Show password"
                      }
                    >
                      {showPassword ? "👁️" : "🙈"}
                    </button>
                  </div>
                  <span className="input-hint">Minimum 8 characters</span>
                </div>

                <div className="button-row">
                  <button
                    type="submit"
                    className="btn-primary"
                    disabled={isLoading}
                  >
                    {isLoading && loadingAction === "open" ? "..." : "Unlock"}
                  </button>
                  <button
                    type="button"
                    className="btn-secondary"
                    onClick={() => handleVaultAction("create")}
                    disabled={isLoading}
                  >
                    {isLoading && loadingAction === "create" ? "..." : "Create"}
                  </button>
                </div>

                <details className="path-details">
                  <summary>Advanced options</summary>
                  <div className="form-group path-group">
                    <label htmlFor="db-path">Vault Path</label>
                    <input
                      id="db-path"
                      type="text"
                      value={dbPathInput}
                      onChange={(e) => setDbPathInput(e.target.value)}
                      placeholder="Custom path (empty = default)"
                      disabled={isLoading}
                    />
                    <span className="input-hint">
                      Last used path is remembered automatically
                    </span>
                  </div>
                </details>
              </form>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app-layout">
      <aside className="sidebar">
        <div className="sidebar-logo">
          <span className="logo-icon">🔓</span>
          <span className="logo-text">Sanctum</span>
        </div>

        <nav className="sidebar-nav">
          <button
            className={`nav-item ${activeTab === "dashboard" ? "active" : ""}`}
            onClick={() => setActiveTab("dashboard")}
          >
            <span className="nav-icon">📊</span>
            <span className="nav-label">Dashboard</span>
          </button>
          <button
            className={`nav-item ${activeTab === "transactions" ? "active" : ""}`}
            onClick={() => setActiveTab("transactions")}
          >
            <span className="nav-icon">💸</span>
            <span className="nav-label">Transactions</span>
          </button>
          <button
            className={`nav-item ${activeTab === "analytics" ? "active" : ""}`}
            onClick={() => setActiveTab("analytics")}
          >
            <span className="nav-icon">📈</span>
            <span className="nav-label">Analytics</span>
          </button>
          <button
            className={`nav-item ${activeTab === "crypto" ? "active" : ""}`}
            onClick={() => {
              setActiveTab("crypto");
              if (cryptoAssets.length === 0 && !cryptoLoading) {
                loadCryptoPrices();
              }
            }}
          >
            <span className="nav-icon">₿</span>
            <span className="nav-label">Crypto</span>
          </button>
        </nav>

        <div className="sidebar-footer">
          <button
            onClick={handleCloseVault}
            className="nav-item lock-btn"
            disabled={isLoading}
          >
            <span className="nav-icon">🔒</span>
            <span className="nav-label">
              {isLoading ? "Locking..." : "Lock Vault"}
            </span>
          </button>
        </div>
      </aside>

      <main className="content-area">
        {error && <div className="message error">{error}</div>}
        {successMessage && (
          <div className="message success">{successMessage}</div>
        )}

        {/* ==================== Dashboard Tab ==================== */}
        {activeTab === "dashboard" && (
          <div className="dashboard">
            <h1 className="page-title">Dashboard</h1>
            <div className="balance-cards">
              <div className="balance-card total">
                <span className="balance-label">Total Balance</span>
                <span className="balance-value">
                  ${formatAmount(balance.total_balance)}
                </span>
              </div>
              <div className="balance-card income">
                <span className="balance-label">Total Income</span>
                <span className="balance-value">
                  +${formatAmount(balance.total_income)}
                </span>
              </div>
              <div className="balance-card expense">
                <span className="balance-label">Total Expenses</span>
                <span className="balance-value">
                  -${formatAmount(balance.total_expense)}
                </span>
              </div>
            </div>

            <div className="recent-transactions">
              <h2 className="section-title">Recent Transactions</h2>
              {transactions.length === 0 ? (
                <p className="empty-state">No transactions recorded</p>
              ) : (
                <div className="transactions-list">
                  {transactions.slice(0, 5).map((tx) => (
                    <div key={tx.id} className="transaction-item">
                      <div className="transaction-info">
                        <div className="transaction-category">
                          {tx.category}
                        </div>
                        <div className="transaction-description">
                          {tx.description}
                        </div>
                        <div className="transaction-date">
                          {formatDate(tx.date)}
                        </div>
                      </div>
                      <div className="transaction-actions">
                        <div
                          className={`transaction-amount ${tx.type === "income" ? "income" : "expense"}`}
                        >
                          {tx.type === "income" ? "+" : "-"}$
                          {formatAmount(tx.amount)}
                        </div>
                        <button
                          className="btn-delete"
                          onClick={() => handleDeleteTransaction(tx.id)}
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
          </div>
        )}

        {/* ==================== Transactions Tab ==================== */}
        {activeTab === "transactions" && (
          <div className="transactions-page">
            <h1 className="page-title">Transactions</h1>

            <div className="transactions-layout">
              <div className="transaction-form-section">
                <h2 className="section-title">New Transaction</h2>
                <form
                  onSubmit={handleAddTransaction}
                  className="transaction-form"
                >
                  <div className="form-row">
                    <div className="form-group">
                      <label htmlFor="amount">Amount ($)</label>
                      <input
                        id="amount"
                        type="number"
                        step="0.01"
                        value={amount}
                        onChange={(e) => setAmount(e.target.value)}
                        placeholder="0.00"
                        disabled={isLoading}
                      />
                    </div>
                    <div className="form-group">
                      <label htmlFor="date">Date</label>
                      <input
                        id="date"
                        type="date"
                        value={date}
                        onChange={(e) => setDate(e.target.value)}
                        disabled={isLoading}
                      />
                    </div>
                  </div>

                  <div className="form-group">
                    <label htmlFor="category">Category</label>
                    <select
                      id="category"
                      value={category}
                      onChange={(e) => setCategory(e.target.value)}
                      disabled={isLoading}
                    >
                      {categories.map((cat) => (
                        <option key={cat} value={cat}>
                          {cat}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="form-group">
                    <label htmlFor="description">Description</label>
                    <input
                      id="description"
                      type="text"
                      value={description}
                      onChange={(e) => setDescription(e.target.value)}
                      placeholder="Describe the transaction"
                      disabled={isLoading}
                    />
                  </div>

                  <div className="form-group">
                    <label className="switch-label">
                      <input
                        type="checkbox"
                        checked={isExpense}
                        onChange={(e) => handleExpenseToggle(e.target.checked)}
                        disabled={isLoading}
                      />
                      <span className="switch-text">
                        {isExpense ? "Expense" : "Income"}
                      </span>
                    </label>
                  </div>

                  <button
                    type="submit"
                    className="btn-primary"
                    disabled={isLoading}
                  >
                    {isLoading ? "Saving..." : "Save Transaction"}
                  </button>
                </form>
              </div>

              <div className="transaction-history-section">
                <h2 className="section-title">History</h2>
                {transactions.length === 0 ? (
                  <p className="empty-state">No transactions recorded</p>
                ) : (
                  <div className="transactions-list">
                    {transactions.map((tx) => (
                      <div key={tx.id} className="transaction-item">
                        <div className="transaction-info">
                          <div className="transaction-category">
                            {tx.category}
                          </div>
                          <div className="transaction-description">
                            {tx.description}
                          </div>
                          <div className="transaction-date">
                            {formatDate(tx.date)}
                          </div>
                        </div>
                        <div className="transaction-actions">
                          <div
                            className={`transaction-amount ${tx.type === "income" ? "income" : "expense"}`}
                          >
                            {tx.type === "income" ? "+" : "-"}$
                            {formatAmount(tx.amount)}
                          </div>
                          <button
                            className="btn-delete"
                            onClick={() => handleDeleteTransaction(tx.id)}
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
            </div>
          </div>
        )}

        {/* ==================== Analytics Tab ==================== */}
        {activeTab === "analytics" && (
          <div className="analytics-page">
            <h1 className="page-title">Analytics</h1>

            {transactions.length === 0 ? (
              <p className="empty-state">
                No transaction data available for analysis
              </p>
            ) : (
              <div className="analytics-grid">
                <div className="chart-card">
                  <h2 className="section-title">Expenses by Category</h2>
                  {expensesByCategory.length === 0 ? (
                    <p className="empty-state">No expenses recorded</p>
                  ) : (
                    <div className="chart-container">
                      <ResponsiveContainer width="100%" height={300}>
                        <PieChart>
                          <Pie
                            data={expensesByCategory}
                            cx="50%"
                            cy="50%"
                            innerRadius={60}
                            outerRadius={100}
                            paddingAngle={3}
                            dataKey="value"
                            stroke="none"
                          >
                            {expensesByCategory.map((_, index) => (
                              <Cell
                                key={`cell-${index}`}
                                fill={CHART_COLORS[index % CHART_COLORS.length]}
                              />
                            ))}
                          </Pie>
                          <Tooltip
                            contentStyle={{
                              backgroundColor: "#111827",
                              border: "1px solid #8b5cf6",
                              borderRadius: "8px",
                              color: "#e8ecf6",
                            }}
                            formatter={(value: number) => [
                              `$${value.toFixed(2)}`,
                              "Amount",
                            ]}
                          />
                        </PieChart>
                      </ResponsiveContainer>
                      <div className="chart-legend">
                        {expensesByCategory.map((entry, index) => (
                          <div key={entry.name} className="legend-item">
                            <span
                              className="legend-color"
                              style={{
                                backgroundColor:
                                  CHART_COLORS[index % CHART_COLORS.length],
                              }}
                            />
                            <span className="legend-label">{entry.name}</span>
                            <span className="legend-value">
                              ${entry.value.toFixed(2)}
                            </span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>

                <div className="chart-card">
                  <h2 className="section-title">Balance Evolution</h2>
                  <div className="chart-container">
                    <ResponsiveContainer width="100%" height={300}>
                      <AreaChart data={balanceEvolution}>
                        <defs>
                          <linearGradient
                            id="balanceGradient"
                            x1="0"
                            y1="0"
                            x2="0"
                            y2="1"
                          >
                            <stop
                              offset="5%"
                              stopColor="#7f8aff"
                              stopOpacity={0.4}
                            />
                            <stop
                              offset="95%"
                              stopColor="#7f8aff"
                              stopOpacity={0}
                            />
                          </linearGradient>
                        </defs>
                        <CartesianGrid
                          stroke="#374151"
                          strokeDasharray="3 3"
                          vertical={false}
                        />
                        <XAxis
                          dataKey="date"
                          stroke="#8c93a8"
                          fontSize={12}
                          tickLine={false}
                          axisLine={{ stroke: "#374151" }}
                        />
                        <YAxis
                          stroke="#8c93a8"
                          fontSize={12}
                          tickLine={false}
                          axisLine={{ stroke: "#374151" }}
                          tickFormatter={(value) => `$${value}`}
                        />
                        <Tooltip
                          contentStyle={{
                            backgroundColor: "#111827",
                            border: "1px solid #7f8aff",
                            borderRadius: "8px",
                            color: "#e8ecf6",
                          }}
                          formatter={(value: number) => [
                            `$${value.toFixed(2)}`,
                            "Balance",
                          ]}
                          labelStyle={{ color: "#c1c7d7" }}
                        />
                        <Area
                          type="monotone"
                          dataKey="balance"
                          stroke="#7f8aff"
                          strokeWidth={2}
                          fill="url(#balanceGradient)"
                        />
                      </AreaChart>
                    </ResponsiveContainer>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {/* ==================== Crypto Tab ==================== */}
        {activeTab === "crypto" && (
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
                      {enrichedPortfolio.map((asset) => (
                        <div key={asset.coin_id} className="portfolio-card">
                          <div className="portfolio-card-header">
                            <span className="portfolio-symbol">
                              {asset.symbol}
                            </span>
                            <span
                              className={`portfolio-pnl ${asset.unrealized_pnl >= 0 ? "positive" : "negative"}`}
                            >
                              {asset.unrealized_pnl >= 0 ? "▲" : "▼"}{" "}
                              {Math.abs(
                                asset.unrealized_pnl_percentage,
                              ).toFixed(2)}
                              %
                            </span>
                          </div>
                          <div className="portfolio-amount">
                            {formatCryptoAmount(asset.total_amount)}{" "}
                            {asset.symbol}
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
                                asset.unrealized_pnl >= 0
                                  ? "positive"
                                  : "negative"
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
                      <button
                        className="btn-secondary"
                        onClick={loadCryptoPrices}
                      >
                        ↻ Load Prices
                      </button>
                    </div>
                  ) : (
                    <div className="crypto-grid">
                      {cryptoAssets
                        .filter((asset) => trackedCoins.includes(asset.id))
                        .map((asset) => (
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
                                <span className="crypto-symbol">
                                  {asset.symbol}
                                </span>
                                <span className="crypto-name">
                                  {asset.name}
                                </span>
                              </div>
                              <div
                                className={`crypto-change ${asset.price_change_percentage_24h >= 0 ? "positive" : "negative"}`}
                              >
                                {asset.price_change_percentage_24h >= 0
                                  ? "▲"
                                  : "▼"}{" "}
                                {Math.abs(
                                  asset.price_change_percentage_24h,
                                ).toFixed(2)}
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
                              {new Date(
                                asset.last_updated,
                              ).toLocaleTimeString()}
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
                    <p>
                      No wallets yet. Create your first wallet to start
                      tracking!
                    </p>
                    <button
                      className="btn-secondary"
                      onClick={() => setShowAddWallet(true)}
                    >
                      + Create Wallet
                    </button>
                  </div>
                ) : (
                  <div className="wallets-grid">
                    {wallets.map((wallet) => (
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
                    <p className="empty-state">
                      No holdings in this wallet yet
                    </p>
                  ) : (
                    <div className="portfolio-grid">
                      {enrichedWalletHoldings.map((asset) => (
                        <div key={asset.coin_id} className="portfolio-card">
                          <div className="portfolio-card-header">
                            <span className="portfolio-symbol">
                              {asset.symbol}
                            </span>
                            <span
                              className={`portfolio-pnl ${asset.unrealized_pnl >= 0 ? "positive" : "negative"}`}
                            >
                              {asset.unrealized_pnl >= 0 ? "▲" : "▼"}{" "}
                              {Math.abs(
                                asset.unrealized_pnl_percentage,
                              ).toFixed(2)}
                              %
                            </span>
                          </div>
                          <div className="portfolio-amount">
                            {formatCryptoAmount(asset.total_amount)}{" "}
                            {asset.symbol}
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
                      {walletTransactions.map((tx) => (
                        <div
                          key={tx.id}
                          className="transaction-item crypto-tx-item"
                        >
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
                💡 Data provided by CoinGecko. Prices are for informational
                purposes only.
              </p>
            </div>
          </div>
        )}

        {/* ==================== Modals ==================== */}

        {/* Add Wallet Modal */}
        {showAddWallet && (
          <div
            className="modal-overlay"
            onClick={() => setShowAddWallet(false)}
          >
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
                      {wallets.map((w) => (
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
                          <span className="suggestion-symbol">
                            {coin.symbol}
                          </span>
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
                        {wallets.map((w) => (
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
                          .filter((w) => w.id !== transferFromWallet)
                          .map((w) => (
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
                          <span className="suggestion-symbol">
                            {coin.symbol}
                          </span>
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
          <div
            className="modal-overlay"
            onClick={() => setShowSwapModal(false)}
          >
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
                      {wallets.map((w) => (
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
                          <span className="suggestion-symbol">
                            {coin.symbol}
                          </span>
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
                      {POPULAR_CRYPTOS.filter(
                        (c) => c.id !== swapFromCoinId,
                      ).map((coin) => (
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
                      ))}
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
          <div
            className="modal-overlay"
            onClick={() => setShowAddCrypto(false)}
          >
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
                    filteredSuggestions.map((coin) => (
                      <button
                        key={coin.id}
                        className="crypto-suggestion"
                        onClick={() => addTrackedCoin(coin.id)}
                      >
                        <span className="suggestion-symbol">{coin.symbol}</span>
                        <span className="suggestion-name">{coin.name}</span>
                      </button>
                    ))
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

        {/* Legacy Add Holding Modal */}
        {showAddHolding && (
          <div
            className="modal-overlay"
            onClick={() => setShowAddHolding(false)}
          >
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
                          <span className="suggestion-symbol">
                            {coin.symbol}
                          </span>
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
                      setHoldingSymbol("");
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

        {/* Delete Wallet Confirmation Modal */}
        {walletToDelete !== null && (
          <div
            className="modal-overlay"
            onClick={() => setWalletToDelete(null)}
          >
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

        {/* Delete Legacy Holding Confirmation Modal */}
        {holdingToDelete !== null && (
          <div
            className="modal-overlay"
            onClick={() => setHoldingToDelete(null)}
          >
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

        {/* Delete Financial Transaction Confirmation Modal */}
        {transactionToDelete !== null && (
          <div className="modal-overlay" onClick={cancelDelete}>
            <div
              className="modal-card delete-modal"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="modal-header">
                <span className="modal-icon">⚠️</span>
                <h2>Confirm Deletion</h2>
              </div>
              <div className="modal-body">
                <p>Are you sure you want to delete this transaction?</p>
                <p className="modal-warning">This action cannot be undone.</p>
              </div>
              <div className="modal-actions">
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={cancelDelete}
                  disabled={isLoading}
                >
                  Cancel
                </button>
                <button
                  type="button"
                  className="btn-danger"
                  onClick={confirmDelete}
                  disabled={isLoading}
                >
                  {isLoading ? "Deleting..." : "Delete"}
                </button>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
