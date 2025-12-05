// ==================== Financial Types ====================

export interface Account {
  id: string;
  name: string;
  type: string; // "bank", "cash", "savings", "credit_card", "other"
  currency: string;
  initial_balance: number; // In cents
  color: string;
  icon: string | null;
  is_archived: boolean;
  created_at: string;
}

export interface AccountBalance {
  account_id: string;
  account_name: string;
  current_balance: number; // In cents
  total_income: number;
  total_expense: number;
}

export interface Transaction {
  id: string;
  account_id: string; // Required: which account this belongs to
  amount: number;
  category: string;
  description: string;
  date: string;
  type: string; // "income", "expense", or "transfer"
  transfer_account_id: string | null; // Only for transfers
}

export interface BalanceSummary {
  total_balance: number;
  total_income: number;
  total_expense: number;
}

// ==================== Crypto Types ====================

export interface CryptoAsset {
  id: string;
  symbol: string;
  name: string;
  current_price: number;
  price_change_percentage_24h: number;
  last_updated: string;
}

// Legacy interface - kept for backwards compatibility
export interface CryptoHolding {
  id: string;
  coin_id: string;
  symbol: string;
  amount: number;
  purchase_price: number;
  purchase_date: string;
}

// New Ledger System interfaces
export interface CryptoWallet {
  id: string;
  name: string;
  category: string; // "exchange" | "wallet_single" | "wallet_multi"
  icon: string | null;
}

export interface CryptoTransaction {
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

export interface AggregatedAsset {
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

// ==================== Habits Types ====================

export interface Habit {
  id: string;
  name: string;
  description: string | null;
  color: string;
  created_at: string;
  archived: boolean;
}

export interface HabitLog {
  id: string;
  habit_id: string;
  completed_date: string; // YYYY-MM-DD format
}

// For O(1) lookup in the grid
export type HabitLogSet = Set<string>; // Format: "habitId:YYYY-MM-DD"

// ==================== UI/App Types ====================

export type TabType =
  | "dashboard"
  | "accounts"
  | "transactions"
  | "analytics"
  | "crypto"
  | "habits";

export type CryptoSubTab = "overview" | "wallets";

export type VaultAction = "open" | "create";

// ==================== Constants ====================

export const MAX_TRACKED_COINS = 20;

export const POPULAR_CRYPTOS = [
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
] as const;

export const DEFAULT_TRACKED_COINS = ["bitcoin", "monero", "litecoin"];

export const EXPENSE_CATEGORIES = [
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

export const INCOME_CATEGORIES = [
  "Salary",
  "Freelance",
  "Investments",
  "Gifts",
  "Other",
] as const;

export const ACCOUNT_TYPES = [
  { value: "bank", label: "Bank Account", icon: "🏦" },
  { value: "cash", label: "Cash", icon: "💵" },
  { value: "savings", label: "Savings", icon: "🐷" },
  { value: "credit_card", label: "Credit Card", icon: "💳" },
  { value: "other", label: "Other", icon: "💰" },
] as const;

export const ACCOUNT_COLORS = [
  "#8b5cf6", // violet
  "#10b981", // emerald
  "#f59e0b", // amber
  "#ef4444", // red
  "#06b6d4", // cyan
  "#ec4899", // pink
  "#6366f1", // indigo
  "#84cc16", // lime
  "#f97316", // orange
  "#14b8a6", // teal
] as const;

export const DEFAULT_CURRENCY = "USD";

export const WALLET_CATEGORIES = [
  { value: "exchange", label: "Exchange", icon: "🏦" },
  { value: "wallet_single", label: "Single-Coin Wallet", icon: "💳" },
  { value: "wallet_multi", label: "Multi-Coin Wallet", icon: "👛" },
] as const;

export const WALLET_ICONS = [
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
] as const;

export const TRANSACTION_TYPES = [
  { value: "buy", label: "Buy", icon: "📥" },
  { value: "sell", label: "Sell", icon: "📤" },
  { value: "transfer_in", label: "Transfer In", icon: "⬇️" },
  { value: "transfer_out", label: "Transfer Out", icon: "⬆️" },
  { value: "swap", label: "Swap", icon: "🔄" },
] as const;

export const CHART_COLORS = [
  "#8b5cf6", // violet
  "#10b981", // emerald
  "#f59e0b", // amber
  "#ef4444", // red
  "#06b6d4", // cyan
  "#ec4899", // pink
  "#6366f1", // indigo
  "#84cc16", // lime
  "#f97316", // orange
] as const;

// ==================== Habits Constants ====================

export const HABIT_COLORS = [
  "#8b5cf6", // violet
  "#10b981", // emerald
  "#f59e0b", // amber
  "#ef4444", // red
  "#06b6d4", // cyan
  "#ec4899", // pink
  "#6366f1", // indigo
  "#84cc16", // lime
  "#f97316", // orange
  "#14b8a6", // teal
  "#a855f7", // purple
  "#22c55e", // green
] as const;

export const DEFAULT_HABIT_COLOR = "#8b5cf6";
