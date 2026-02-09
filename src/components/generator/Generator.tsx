import * as React from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// ==================== Types ====================

type TransactionType = "income" | "expense" | "transfer";
type CryptoType = "trade" | "income" | "expense" | "transfer";

type Transaction = {
  id: string;
  date: string;
  account: string;
  transaction_type: TransactionType;
  amount: number;
  currency: string;
  category: string;
  description: string;
  transfer_to_account?: string | null;
};

type HabitLog = {
  id: string;
  habit: string;
  date: string;
  completed: boolean;
};

type CryptoTransaction = {
  id: string;
  date: string;
  wallet: string;
  symbol: string;
  transaction_type: CryptoType;
  subtype?: string | null;
  amount: number;
  price_per_coin?: number | null;
  fee?: number | null;
  swap_to_symbol?: string | null;
  swap_to_amount?: number | null;
  fee_coin_symbol?: string | null;
  fee_amount?: number | null;
  override_proceeds?: number | null;
  override_cost_basis?: number | null;
  notes?: string | null;
};

type ExportData = {
  version: string;
  exported_at: string;
  transactions: {
    date: string;
    account: string;
    type: TransactionType;
    amount: number;
    currency: string;
    category: string;
    description: string;
    transfer_to_account?: string | null;
  }[];
  habit_logs: Omit<HabitLog, "id">[];
  crypto_transactions: {
    date: string;
    wallet: string;
    symbol: string;
    type: CryptoType;
    subtype?: string | null;
    amount: number;
    price_per_coin?: number | null;
    fee?: number | null;
    swap_to_symbol?: string | null;
    swap_to_amount?: number | null;
    fee_coin_symbol?: string | null;
    fee_amount?: number | null;
    override_proceeds?: number | null;
    override_cost_basis?: number | null;
    notes?: string | null;
  }[];
};

// ==================== Catalogs ====================

const EXPENSE_CATEGORIES = [
  "FOOD",
  "TRANSPORT",
  "UTILITIES",
  "ENTERTAINMENT",
  "HEALTH",
  "SHOPPING",
  "EDUCATION",
  "OTHER",
];

const INCOME_CATEGORIES = [
  "SALARY",
  "FREELANCE",
  "INVESTMENT",
  "GIFT",
  "OTHER",
];

const CURRENCIES = ["USD", "CLP"];

// Mirrors default_coin_catalog() from src/features/crypto/api.rs (feat/crypto-tax)
// Order and IDs must stay in sync with the Rust catalog.
const CRYPTO_COINS = [
  { id: "bitcoin", symbol: "BTC", name: "Bitcoin" },
  { id: "litecoin", symbol: "LTC", name: "Litecoin" },
  { id: "monero", symbol: "XMR", name: "Monero" },
  { id: "ethereum", symbol: "ETH", name: "Ethereum" },
  { id: "tether", symbol: "USDT", name: "Tether" },
  { id: "binancecoin", symbol: "BNB", name: "BNB" },
  { id: "solana", symbol: "SOL", name: "Solana" },
  { id: "ripple", symbol: "XRP", name: "XRP" },
  { id: "usd-coin", symbol: "USDC", name: "USDC" },
  { id: "cardano", symbol: "ADA", name: "Cardano" },
  { id: "dogecoin", symbol: "DOGE", name: "Dogecoin" },
  { id: "tron", symbol: "TRX", name: "TRON" },
  { id: "polygon-ecosystem-token", symbol: "POL", name: "Polygon" },
  { id: "chainlink", symbol: "LINK", name: "Chainlink" },
  { id: "polkadot", symbol: "DOT", name: "Polkadot" },
  { id: "shiba-inu", symbol: "SHIB", name: "Shiba Inu" },
  { id: "avalanche-2", symbol: "AVAX", name: "Avalanche" },
  { id: "stellar", symbol: "XLM", name: "Stellar" },
  { id: "bitcoin-cash", symbol: "BCH", name: "Bitcoin Cash" },
  { id: "uniswap", symbol: "UNI", name: "Uniswap" },
  { id: "cosmos", symbol: "ATOM", name: "Cosmos Hub" },
  { id: "ethereum-classic", symbol: "ETC", name: "Ethereum Classic" },
  { id: "hedera-hashgraph", symbol: "HBAR", name: "Hedera" },
  { id: "aave", symbol: "AAVE", name: "Aave" },
  { id: "vechain", symbol: "VET", name: "VeChain" },
  { id: "near", symbol: "NEAR", name: "NEAR Protocol" },
  { id: "algorand", symbol: "ALGO", name: "Algorand" },
  { id: "quant-network", symbol: "QNT", name: "Quant" },
  { id: "arbitrum", symbol: "ARB", name: "Arbitrum" },
  { id: "sui", symbol: "SUI", name: "Sui" },
  { id: "aptos", symbol: "APT", name: "Aptos" },
  { id: "crypto-com-chain", symbol: "CRO", name: "Cronos" },
  { id: "zcash", symbol: "ZEC", name: "Zcash" },
  { id: "dai", symbol: "DAI", name: "Dai" },
  { id: "the-open-network", symbol: "TON", name: "Toncoin" },
  { id: "internet-computer", symbol: "ICP", name: "Internet Computer" },
  { id: "kaspa", symbol: "KAS", name: "Kaspa" },
  { id: "mantle", symbol: "MNT", name: "Mantle" },
  { id: "bittensor", symbol: "TAO", name: "Bittensor" },
  { id: "worldcoin-wld", symbol: "WLD", name: "Worldcoin" },
];

// Mirrors TAX_SUBTYPES_* from src/features/crypto/tax/types.rs (feat/crypto-tax)
// Each scenario maps directly to fiscal (type, subtype).
type CryptoScenario = {
  key: string;
  group: "trade" | "transfer" | "income" | "expense";
  fiscalType: CryptoType;
  subtype: string | null;
  en: string;
  es: string;
};

const SCENARIO_GROUPS: Record<string, { en: string; es: string }> = {
  trade: { en: "Trade", es: "Operaciones" },
  transfer: { en: "Transfer", es: "Transferencia" },
  income: { en: "Income", es: "Ingreso" },
  expense: { en: "Expense", es: "Gasto" },
};

const CRYPTO_SCENARIOS: CryptoScenario[] = [
  // ── Trade ──
  {
    key: "trade:buy",
    group: "trade",
    fiscalType: "trade",
    subtype: "buy",
    en: "Buy",
    es: "Compra",
  },
  {
    key: "trade:sell",
    group: "trade",
    fiscalType: "trade",
    subtype: "sell",
    en: "Sell",
    es: "Venta",
  },
  {
    key: "trade:swap",
    group: "trade",
    fiscalType: "trade",
    subtype: "swap",
    en: "Swap",
    es: "Swap",
  },
  {
    key: "trade:other",
    group: "trade",
    fiscalType: "trade",
    subtype: "other",
    en: "Other Trade",
    es: "Otra operacion",
  },
  // ── Transfer ──
  {
    key: "transfer:deposit",
    group: "transfer",
    fiscalType: "transfer",
    subtype: "deposit",
    en: "Deposit",
    es: "Deposito",
  },
  {
    key: "transfer:withdrawal",
    group: "transfer",
    fiscalType: "transfer",
    subtype: "withdrawal",
    en: "Withdrawal",
    es: "Retiro",
  },
  // ── Income ──
  {
    key: "income:airdrop",
    group: "income",
    fiscalType: "income",
    subtype: "airdrop",
    en: "Airdrop",
    es: "Airdrop",
  },
  {
    key: "income:staking",
    group: "income",
    fiscalType: "income",
    subtype: "staking",
    en: "Staking Reward",
    es: "Recompensa staking",
  },
  {
    key: "income:mining",
    group: "income",
    fiscalType: "income",
    subtype: "mining",
    en: "Mining",
    es: "Mineria",
  },
  {
    key: "income:interest",
    group: "income",
    fiscalType: "income",
    subtype: "interest",
    en: "Interest",
    es: "Interes",
  },
  {
    key: "income:reward",
    group: "income",
    fiscalType: "income",
    subtype: "reward",
    en: "Reward",
    es: "Recompensa",
  },
  {
    key: "income:gift",
    group: "income",
    fiscalType: "income",
    subtype: "gift",
    en: "Gift Received",
    es: "Regalo recibido",
  },
  {
    key: "income:fork",
    group: "income",
    fiscalType: "income",
    subtype: "fork",
    en: "Fork",
    es: "Fork",
  },
  {
    key: "income:payment",
    group: "income",
    fiscalType: "income",
    subtype: "payment",
    en: "Payment Received",
    es: "Pago recibido",
  },
  {
    key: "income:rebate",
    group: "income",
    fiscalType: "income",
    subtype: "rebate",
    en: "Rebate",
    es: "Reembolso",
  },
  {
    key: "income:other",
    group: "income",
    fiscalType: "income",
    subtype: "other",
    en: "Other Income",
    es: "Otro ingreso",
  },
  // ── Expense ──
  {
    key: "expense:payment",
    group: "expense",
    fiscalType: "expense",
    subtype: "payment",
    en: "Payment",
    es: "Pago",
  },
  {
    key: "expense:gift",
    group: "expense",
    fiscalType: "expense",
    subtype: "gift",
    en: "Gift Sent",
    es: "Regalo enviado",
  },
  {
    key: "expense:fee",
    group: "expense",
    fiscalType: "expense",
    subtype: "fee",
    en: "Fee",
    es: "Comision",
  },
  {
    key: "expense:lost",
    group: "expense",
    fiscalType: "expense",
    subtype: "lost",
    en: "Lost",
    es: "Perdida",
  },
  {
    key: "expense:stolen",
    group: "expense",
    fiscalType: "expense",
    subtype: "stolen",
    en: "Stolen",
    es: "Robo",
  },
  {
    key: "expense:donation",
    group: "expense",
    fiscalType: "expense",
    subtype: "donation",
    en: "Donation",
    es: "Donacion",
  },
  {
    key: "expense:sell",
    group: "expense",
    fiscalType: "expense",
    subtype: "sell",
    en: "Liquidation",
    es: "Liquidacion",
  },
  {
    key: "expense:other",
    group: "expense",
    fiscalType: "expense",
    subtype: "other",
    en: "Other Expense",
    es: "Otro gasto",
  },
];

const SCENARIO_MAP: Record<string, CryptoScenario> = Object.fromEntries(
  CRYPTO_SCENARIOS.map((s) => [s.key, s]),
);

/** Resolve a scenario key from imported fiscal (type, subtype). */
function resolveScenarioKey(
  txType: CryptoType,
  subtype: string | null,
): string {
  const normalizedSubtype = (subtype ?? "").trim().toLowerCase();
  if (normalizedSubtype) {
    const key = `${txType}:${normalizedSubtype}`;
    if (SCENARIO_MAP[key]) return key;
  }
  if (txType === "trade") return "trade:buy";
  if (txType === "transfer") return "transfer:deposit";
  if (txType === "income") return "income:other";
  if (txType === "expense") return "expense:other";
  return "trade:buy";
}

// ==================== Translations ====================

const translations = {
  en: {
    headerTag: "Generator",
    headerTitle: "Build a trip-safe log",
    headerDescription:
      "This generator matches Sanctum's import schema. Account, habit, wallet, and category names must already exist in your vault for the import to succeed.",
    startOver: "Start over",
    steps: {
      load: "Step 1: Load",
      add: "Step 2: Add Entries",
      export: "Step 3: Export",
    },
    stats: {
      transactions: "Transactions",
      habits: "Habit Logs",
      crypto: "Crypto Entries",
    },
    loaded: "Loaded",
    load: {
      title: "Load Existing JSON",
      description:
        "Upload a sanctum_export.json file to append new entries to the same log.",
      button: "Upload JSON",
    },
    paste: {
      title: "Paste JSON",
      description:
        "Use this if you only have access to text notes. The payload stays local.",
      placeholder: "Paste a sanctum_export.json payload...",
      button: "Load From Paste",
      privacy:
        "Privacy: data never leaves your browser. Works offline after first load.",
    },
    add: {
      title: "Add Entries",
      subtitle: "Build your log",
      tabs: {
        finances: "Finances",
        crypto: "Crypto",
        habits: "Habits",
      },
    },
    transactions: {
      form: {
        account: "Account",
        currency: "Currency (USD, CLP or custom)",
        category: "Category",
        selectCategory: "Select category...",
        transferTo: "Transfer to account",
        description: "Description",
        amount: "Amount",
        add: "Add Transaction",
        required: "Required: Date, Account, Amount, Currency.",
        transferNote: "Category is required unless this is a transfer.",
      },
      list: {
        title: "Recent Transactions",
        empty: "No transactions yet. Add the first one.",
        remove: "Remove",
      },
    },
    habits: {
      form: {
        name: "Habit name",
        completed: "Completed",
        add: "Add Habit Log",
        required: "Required: Date, Habit name.",
      },
      list: {
        title: "Habit Logs",
        empty: "No habit logs yet. Capture today's progress.",
        remove: "Remove",
        completed: "Completed",
        skipped: "Skipped",
      },
    },
    crypto: {
      form: {
        wallet: "Wallet",
        symbol: "Symbol",
        selectSymbol: "Select coin...",
        customSymbol: "Or type custom...",
        amount: "Amount",
        swapToSymbol: "Swap to symbol",
        swapToAmount: "Swap to amount",
        price: "Price per coin (optional)",
        fee: "Fee in USD (optional)",
        feeCoinSymbol: "Fee coin symbol",
        feeAmount: "Fee amount in crypto",
        notes: "Notes (optional)",
        add: "Add Crypto Entry",
        required:
          "Required: Date, Wallet, Symbol, Amount. Swap needs target symbol + amount.",
        overrideProceeds: "Override proceeds (USD)",
        overrideCostBasis: "Override cost basis (USD)",
        advancedSection: "Advanced options",
        selectScenario: "Select scenario...",
        feeCoinSection: "Crypto fee (optional)",
      },
      list: {
        title: "Crypto Entries",
        empty: "No crypto entries yet. Add wallet activity here.",
        remove: "Remove",
      },
    },
    types: {
      expense: "Expense",
      income: "Income",
      transfer: "Transfer",
      buy: "Buy",
      sell: "Sell",
      transfer_in: "Transfer In",
      transfer_out: "Transfer Out",
      swap: "Swap",
    } as Record<string, string>,
    export: {
      title: "Export Preview",
      download: "Download JSON",
    },
    errors: {
      invalidJson: "Invalid JSON structure.",
      unsupportedVersion: "Unsupported version. Expected version 1.0.",
      parseFailed: "Failed to parse JSON.",
      pasteRequired: "Paste a JSON payload before loading.",
      transactionDateRequired: "Transaction date is required.",
      transactionAccountRequired: "Transaction account is required.",
      transactionAmountRequired: "Transaction amount is required.",
      transactionCurrencyRequired: "Transaction currency is required.",
      transactionCategoryRequired:
        "Category is required unless this is a transfer.",
      transactionTransferRequired: "Transfer requires a destination account.",
      habitDateRequired: "Habit date is required.",
      habitNameRequired: "Habit name is required.",
      cryptoDateRequired: "Crypto date is required.",
      cryptoWalletRequired: "Crypto wallet is required.",
      cryptoSymbolRequired: "Crypto symbol is required.",
      cryptoAmountRequired: "Crypto amount is required.",
      cryptoSwapSymbolRequired: "Swap target symbol is required.",
      cryptoSwapAmountRequired: "Swap target amount is required.",
      cryptoFeePairRequired:
        "Fee coin and fee amount must be provided together.",
    },
  },
  es: {
    headerTag: "Generador Sanctum",
    headerTitle: "Construye un registro seguro",
    headerDescription:
      "Este generador respeta el esquema de importacion de Sanctum. Los nombres de cuentas, habitos, wallets y categorias deben existir en tu boveda para que la importacion funcione.",
    startOver: "Reiniciar",
    steps: {
      load: "Paso 1: Cargar",
      add: "Paso 2: Agregar",
      export: "Paso 3: Exportar",
    },
    stats: {
      transactions: "Transacciones",
      habits: "Registros de habitos",
      crypto: "Movimientos cripto",
    },
    loaded: "Cargado",
    load: {
      title: "Cargar JSON existente",
      description:
        "Sube un archivo sanctum_export.json para agregar nuevas entradas al mismo registro.",
      button: "Subir JSON",
    },
    paste: {
      title: "Pegar JSON",
      description:
        "Usalo si solo tienes acceso a notas de texto. El contenido se mantiene local.",
      placeholder: "Pega el contenido de sanctum_export.json...",
      button: "Cargar desde pegado",
      privacy:
        "Privacidad: los datos nunca salen del navegador. Funciona offline despues de la primera carga.",
    },
    add: {
      title: "Agregar entradas",
      subtitle: "Construye tu registro",
      tabs: {
        finances: "Finanzas",
        crypto: "Cripto",
        habits: "Habitos",
      },
    },
    transactions: {
      form: {
        account: "Cuenta",
        currency: "Moneda (USD, CLP u otra)",
        category: "Categoria",
        selectCategory: "Seleccionar categoria...",
        transferTo: "Transferir a cuenta",
        description: "Descripcion",
        amount: "Monto",
        add: "Agregar transaccion",
        required: "Requerido: Fecha, Cuenta, Monto, Moneda.",
        transferNote:
          "La categoria es obligatoria salvo que sea una transferencia.",
      },
      list: {
        title: "Transacciones recientes",
        empty: "Aun no hay transacciones. Agrega la primera.",
        remove: "Eliminar",
      },
    },
    habits: {
      form: {
        name: "Nombre del habito",
        completed: "Completado",
        add: "Agregar registro de habito",
        required: "Requerido: Fecha, Nombre del habito.",
      },
      list: {
        title: "Registros de habitos",
        empty: "Aun no hay registros. Captura el progreso de hoy.",
        remove: "Eliminar",
        completed: "Completado",
        skipped: "Omitido",
      },
    },
    crypto: {
      form: {
        wallet: "Wallet",
        symbol: "Simbolo",
        selectSymbol: "Seleccionar moneda...",
        customSymbol: "O escribe una...",
        amount: "Monto",
        swapToSymbol: "Simbolo destino",
        swapToAmount: "Monto destino",
        price: "Precio por moneda (opcional)",
        fee: "Comision en USD (opcional)",
        feeCoinSymbol: "Simbolo comision",
        feeAmount: "Monto comision en cripto",
        notes: "Notas (opcional)",
        add: "Agregar movimiento cripto",
        required:
          "Requerido: Fecha, Wallet, Simbolo, Monto. Swap exige simbolo + monto destino.",
        overrideProceeds: "Forzar ingresos (USD)",
        overrideCostBasis: "Forzar costo base (USD)",
        advancedSection: "Opciones avanzadas",
        selectScenario: "Seleccionar escenario...",
        feeCoinSection: "Comision cripto (opcional)",
      },
      list: {
        title: "Movimientos cripto",
        empty: "Aun no hay movimientos. Agrega actividad aqui.",
        remove: "Eliminar",
      },
    },
    types: {
      expense: "Gasto",
      income: "Ingreso",
      transfer: "Transferencia",
      buy: "Compra",
      sell: "Venta",
      transfer_in: "Transferencia entrada",
      transfer_out: "Transferencia salida",
      swap: "Swap",
    } as Record<string, string>,
    export: {
      title: "Vista previa de exportacion",
      download: "Descargar JSON",
    },
    errors: {
      invalidJson: "Estructura JSON invalida.",
      unsupportedVersion: "Version no compatible. Se esperaba la version 1.0.",
      parseFailed: "No se pudo leer el JSON.",
      pasteRequired: "Pega un JSON antes de cargar.",
      transactionDateRequired: "La fecha de la transaccion es obligatoria.",
      transactionAccountRequired: "La cuenta de la transaccion es obligatoria.",
      transactionAmountRequired: "El monto de la transaccion es obligatorio.",
      transactionCurrencyRequired:
        "La moneda de la transaccion es obligatoria.",
      transactionCategoryRequired:
        "La categoria es obligatoria salvo que sea una transferencia.",
      transactionTransferRequired:
        "La transferencia requiere una cuenta destino.",
      habitDateRequired: "La fecha del habito es obligatoria.",
      habitNameRequired: "El nombre del habito es obligatorio.",
      cryptoDateRequired: "La fecha del movimiento cripto es obligatoria.",
      cryptoWalletRequired: "La wallet es obligatoria.",
      cryptoSymbolRequired: "El simbolo cripto es obligatorio.",
      cryptoAmountRequired: "El monto cripto es obligatorio.",
      cryptoSwapSymbolRequired: "El simbolo destino del swap es obligatorio.",
      cryptoSwapAmountRequired: "El monto destino del swap es obligatorio.",
      cryptoFeePairRequired: "El simbolo y monto de comision deben ir juntos.",
    },
  },
};

// ==================== Helpers ====================

const getLocalDateString = () => {
  const now = new Date();
  const offsetMs = now.getTimezoneOffset() * 60_000;
  return new Date(now.getTime() - offsetMs).toISOString().slice(0, 10);
};

const createDefaultTx = (date: string) => ({
  date,
  account: "",
  transaction_type: "expense" as TransactionType,
  amount: "",
  currency: "USD",
  category: "",
  description: "",
  transfer_to_account: "",
});

const createDefaultHabit = (date: string) => ({
  habit: "",
  date,
  completed: true,
});

const createDefaultCrypto = (date: string) => ({
  date,
  wallet: "",
  symbol: "",
  customSymbol: "",
  scenarioGroup: "trade" as "trade" | "transfer" | "income" | "expense",
  scenarioKey: "trade:buy",
  amount: "",
  swap_to_symbol: "",
  swap_to_amount: "",
  price_per_coin: "",
  fee: "",
  fee_coin_symbol: "",
  fee_amount: "",
  override_proceeds: "",
  override_cost_basis: "",
  notes: "",
});

function makeId() {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `id-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function normalizeTransactionType(value: unknown): TransactionType {
  const normalized = String(value ?? "").toLowerCase();
  if (
    normalized === "income" ||
    normalized === "expense" ||
    normalized === "transfer"
  ) {
    return normalized as TransactionType;
  }
  return "expense";
}

function normalizeCryptoType(value: unknown): CryptoType {
  const normalized = String(value ?? "").toLowerCase();
  if (
    normalized === "trade" ||
    normalized === "income" ||
    normalized === "expense" ||
    normalized === "transfer"
  ) {
    return normalized as CryptoType;
  }
  return "trade";
}

function isSupportedVersion(value: unknown) {
  return value === "1" || value === "1.0" || value === 1;
}

function getInitialLang(): "en" | "es" {
  if (typeof document === "undefined") {
    return "en";
  }
  return document.documentElement.lang === "es" ? "es" : "en";
}

/** Converts empty/whitespace strings to null for optional fields */
function emptyToNull(value: unknown): string | null {
  if (value == null) return null;
  const s = String(value).trim();
  return s === "" ? null : s;
}

function parseCryptoFromJson(tx: any): CryptoTransaction {
  const swapToSymbol = emptyToNull(tx.swap_to_symbol ?? tx.to_symbol);
  const swapToAmount = tx.swap_to_amount ?? tx.to_amount ?? null;
  const feeCoinSymbol = emptyToNull(tx.fee_coin_symbol ?? tx.fee_coin);
  const rawType = String(tx.transaction_type ?? tx.type ?? "")
    .trim()
    .toLowerCase();
  let txType = normalizeCryptoType(rawType);
  let subtype = emptyToNull(tx.subtype);

  // Backfill subtype when loading older payloads that used mechanical type.
  if (!subtype) {
    if (rawType === "buy") {
      txType = "trade";
      subtype = "buy";
    } else if (rawType === "sell") {
      txType = "trade";
      subtype = "sell";
    } else if (rawType === "swap") {
      txType = "trade";
      subtype = "swap";
    } else if (rawType === "transfer_in") {
      txType = "transfer";
      subtype = "deposit";
    } else if (rawType === "transfer_out") {
      txType = "transfer";
      subtype = "withdrawal";
    }
  }

  return {
    id: makeId(),
    date: String(tx.date ?? ""),
    wallet: String(tx.wallet ?? ""),
    symbol: String(tx.symbol ?? ""),
    transaction_type: txType,
    subtype,
    amount: Number(tx.amount ?? 0),
    price_per_coin:
      tx.price_per_coin != null ? Number(tx.price_per_coin) : null,
    fee: tx.fee != null ? Number(tx.fee) : null,
    swap_to_symbol: swapToSymbol,
    swap_to_amount: swapToAmount != null ? Number(swapToAmount) : null,
    fee_coin_symbol: feeCoinSymbol,
    fee_amount: tx.fee_amount != null ? Number(tx.fee_amount) : null,
    override_proceeds:
      tx.override_proceeds != null ? Number(tx.override_proceeds) : null,
    override_cost_basis:
      tx.override_cost_basis != null ? Number(tx.override_cost_basis) : null,
    notes: emptyToNull(tx.notes),
  };
}

// ==================== Select Component ====================

function SelectField({
  value,
  onChange,
  options,
  placeholder,
  className,
  "aria-invalid": ariaInvalid,
}: {
  value: string;
  onChange: (v: string) => void;
  options: { value: string; label: string }[];
  placeholder?: string;
  className?: string;
  "aria-invalid"?: boolean;
}) {
  return (
    <select
      className={cn(
        "h-10 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        className,
      )}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      aria-invalid={ariaInvalid}
    >
      {placeholder && (
        <option value="" disabled>
          {placeholder}
        </option>
      )}
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  );
}

// ==================== Main Component ====================

export default function Generator() {
  const [lang, setLang] = React.useState<"en" | "es">(getInitialLang);
  const copy = React.useMemo(
    () => translations[lang] ?? translations.en,
    [lang],
  );

  const [transactions, setTransactions] = React.useState<Transaction[]>([]);
  const [habits, setHabits] = React.useState<HabitLog[]>([]);
  const [cryptoTx, setCryptoTx] = React.useState<CryptoTransaction[]>([]);
  const [activeTab, setActiveTab] = React.useState<
    "transactions" | "habits" | "crypto"
  >("transactions");
  const [error, setError] = React.useState<string | null>(null);
  const [loadedFile, setLoadedFile] = React.useState<string | null>(null);
  const [rawInput, setRawInput] = React.useState("");

  const [txForm, setTxForm] = React.useState(() =>
    createDefaultTx(getLocalDateString()),
  );
  const [txErrors, setTxErrors] = React.useState<{
    date?: boolean;
    account?: boolean;
    amount?: boolean;
    currency?: boolean;
    category?: boolean;
    transfer_to_account?: boolean;
  }>({});
  const [txAttempted, setTxAttempted] = React.useState(false);

  const [habitForm, setHabitForm] = React.useState(() =>
    createDefaultHabit(getLocalDateString()),
  );
  const [habitErrors, setHabitErrors] = React.useState<{
    date?: boolean;
    habit?: boolean;
  }>({});
  const [habitAttempted, setHabitAttempted] = React.useState(false);

  const [cryptoForm, setCryptoForm] = React.useState(() =>
    createDefaultCrypto(getLocalDateString()),
  );
  const [cryptoErrors, setCryptoErrors] = React.useState<{
    date?: boolean;
    wallet?: boolean;
    symbol?: boolean;
    amount?: boolean;
    swap_to_symbol?: boolean;
    swap_to_amount?: boolean;
    fee_coin_symbol?: boolean;
    fee_amount?: boolean;
  }>({});
  const [cryptoAttempted, setCryptoAttempted] = React.useState(false);
  const [showAdvancedSection, setShowAdvancedSection] = React.useState(false);

  const fileInputRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    const handleLangChange = (event: Event) => {
      const detail = (event as CustomEvent<{ lang?: string }>).detail;
      const nextLang = detail?.lang ?? document.documentElement.lang;
      const next = nextLang === "es" ? "es" : "en";
      setLang(next);
    };

    window.addEventListener("sanctum-lang-change", handleLangChange);
    return () =>
      window.removeEventListener("sanctum-lang-change", handleLangChange);
  }, []);

  // Derived: categories based on tx type
  const categoryOptions = React.useMemo(() => {
    const cats =
      txForm.transaction_type === "income"
        ? INCOME_CATEGORIES
        : EXPENSE_CATEGORIES;
    return cats.map((c) => ({ value: c, label: c }));
  }, [txForm.transaction_type]);

  // Derived: selected scenario
  const selectedScenario =
    SCENARIO_MAP[cryptoForm.scenarioKey] ?? SCENARIO_MAP["trade:buy"]!;
  const isCryptoSwap =
    selectedScenario.fiscalType === "trade" &&
    selectedScenario.subtype === "swap";

  // Derived: subtype options for current group
  const subtypeOptions = React.useMemo(
    () =>
      CRYPTO_SCENARIOS.filter((s) => s.group === cryptoForm.scenarioGroup).map(
        (s) => ({ value: s.key, label: s[lang] }),
      ),
    [cryptoForm.scenarioGroup, lang],
  );

  // Derived: scenario label for display
  const scenarioLabel = React.useCallback(
    (key: string) => {
      const s = SCENARIO_MAP[key];
      if (!s) return key;
      const groupLabel = SCENARIO_GROUPS[s.group]?.[lang] ?? s.group;
      return `${groupLabel} / ${s[lang]}`;
    },
    [lang],
  );

  // Derived: resolved symbol (select or custom)
  const resolvedSymbol = React.useMemo(() => {
    if (cryptoForm.symbol === "__custom__") {
      return cryptoForm.customSymbol.trim().toUpperCase();
    }
    return cryptoForm.symbol;
  }, [cryptoForm.symbol, cryptoForm.customSymbol]);

  // ==================== Export ====================

  const exportPayload = React.useMemo<ExportData>(() => {
    return {
      version: "1.0",
      exported_at: new Date().toISOString(),
      transactions: transactions.map(({ id, transaction_type, ...tx }) => ({
        ...tx,
        type: transaction_type,
        transfer_to_account: tx.transfer_to_account || null,
      })),
      habit_logs: habits.map(({ id, ...log }) => log),
      crypto_transactions: cryptoTx.map(({ id, transaction_type, ...tx }) => ({
        date: tx.date,
        wallet: tx.wallet,
        symbol: tx.symbol,
        type: transaction_type,
        subtype: tx.subtype || null,
        amount: tx.amount,
        price_per_coin: tx.price_per_coin ?? null,
        fee: tx.fee ?? null,
        swap_to_symbol: tx.swap_to_symbol || null,
        swap_to_amount: tx.swap_to_amount ?? null,
        fee_coin_symbol: tx.fee_coin_symbol || null,
        fee_amount: tx.fee_amount ?? null,
        override_proceeds: tx.override_proceeds ?? null,
        override_cost_basis: tx.override_cost_basis ?? null,
        notes: tx.notes || null,
      })),
    };
  }, [transactions, habits, cryptoTx]);

  const exportJson = React.useMemo(
    () => JSON.stringify(exportPayload, null, 2),
    [exportPayload],
  );

  // ==================== Load / Reset ====================

  function resetFormState() {
    setTxErrors({});
    setHabitErrors({});
    setCryptoErrors({});
    setTxAttempted(false);
    setHabitAttempted(false);
    setCryptoAttempted(false);
    setShowAdvancedSection(false);
    const today = getLocalDateString();
    setTxForm(createDefaultTx(today));
    setHabitForm(createDefaultHabit(today));
    setCryptoForm(createDefaultCrypto(today));
  }

  function loadParsedData(parsed: any, sourceName: string) {
    if (!parsed || typeof parsed !== "object") {
      throw new Error(copy.errors.invalidJson);
    }
    if (!isSupportedVersion(parsed.version)) {
      throw new Error(copy.errors.unsupportedVersion);
    }

    const nextTransactions = Array.isArray(parsed.transactions)
      ? parsed.transactions.map((tx: any) => ({
          id: makeId(),
          date: String(tx.date ?? ""),
          account: String(tx.account ?? ""),
          transaction_type: normalizeTransactionType(
            tx.transaction_type ?? tx.type,
          ),
          amount: Number(tx.amount ?? 0),
          currency: String(tx.currency ?? ""),
          category: String(tx.category ?? ""),
          description: String(tx.description ?? ""),
          transfer_to_account: tx.transfer_to_account ?? null,
        }))
      : [];
    const nextHabits = Array.isArray(parsed.habit_logs)
      ? parsed.habit_logs.map((log: HabitLog) => ({
          ...log,
          id: makeId(),
        }))
      : [];
    const nextCrypto = Array.isArray(parsed.crypto_transactions)
      ? parsed.crypto_transactions.map(parseCryptoFromJson)
      : [];

    setTransactions(nextTransactions);
    setHabits(nextHabits);
    setCryptoTx(nextCrypto);
    setLoadedFile(sourceName);
    setRawInput("");
    setError(null);
    resetFormState();
  }

  function handleLoadFile(file: File) {
    const reader = new FileReader();
    reader.onload = () => {
      try {
        const text = String(reader.result ?? "");
        const parsed = JSON.parse(text);
        loadParsedData(parsed, file.name);
      } catch (err) {
        const message =
          err instanceof SyntaxError
            ? copy.errors.parseFailed
            : err instanceof Error
              ? err.message
              : copy.errors.parseFailed;
        setError(message);
      }
    };
    reader.readAsText(file);
  }

  function handleLoadClick() {
    fileInputRef.current?.click();
  }

  function handleLoadPaste() {
    if (!rawInput.trim()) {
      setError(copy.errors.pasteRequired);
      return;
    }
    try {
      const parsed = JSON.parse(rawInput);
      loadParsedData(parsed, "pasted JSON");
    } catch (err) {
      const message =
        err instanceof SyntaxError
          ? copy.errors.parseFailed
          : err instanceof Error
            ? err.message
            : copy.errors.parseFailed;
      setError(message);
    }
  }

  function handleDownload() {
    const blob = new Blob([exportJson], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "sanctum_export.json";
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
  }

  function handleReset() {
    setTransactions([]);
    setHabits([]);
    setCryptoTx([]);
    setLoadedFile(null);
    setError(null);
    resetFormState();
  }

  // ==================== Add Handlers ====================

  function addTransaction() {
    setError(null);
    setTxAttempted(true);
    const nextErrors: typeof txErrors = {};
    let message: string | null = null;
    if (!txForm.date) {
      nextErrors.date = true;
      message ??= copy.errors.transactionDateRequired;
    }
    if (!txForm.account) {
      nextErrors.account = true;
      message ??= copy.errors.transactionAccountRequired;
    }
    if (!txForm.amount) {
      nextErrors.amount = true;
      message ??= copy.errors.transactionAmountRequired;
    }
    if (!txForm.currency) {
      nextErrors.currency = true;
      message ??= copy.errors.transactionCurrencyRequired;
    }
    if (txForm.transaction_type !== "transfer" && !txForm.category) {
      nextErrors.category = true;
      message ??= copy.errors.transactionCategoryRequired;
    }
    if (txForm.transaction_type === "transfer" && !txForm.transfer_to_account) {
      nextErrors.transfer_to_account = true;
      message ??= copy.errors.transactionTransferRequired;
    }
    if (Object.keys(nextErrors).length > 0) {
      setTxErrors(nextErrors);
      setError(message);
      return;
    }

    setTransactions((prev) => [
      {
        id: makeId(),
        date: txForm.date,
        account: txForm.account,
        transaction_type: txForm.transaction_type,
        amount: Number(txForm.amount),
        currency: txForm.currency.toUpperCase(),
        category: txForm.category,
        description: txForm.description,
        transfer_to_account: txForm.transfer_to_account || null,
      },
      ...prev,
    ]);
    setTxForm(createDefaultTx(getLocalDateString()));
    setTxErrors({});
    setTxAttempted(false);
  }

  function addHabit() {
    setError(null);
    setHabitAttempted(true);
    const nextErrors: typeof habitErrors = {};
    let message: string | null = null;
    if (!habitForm.date) {
      nextErrors.date = true;
      message ??= copy.errors.habitDateRequired;
    }
    if (!habitForm.habit) {
      nextErrors.habit = true;
      message ??= copy.errors.habitNameRequired;
    }
    if (Object.keys(nextErrors).length > 0) {
      setHabitErrors(nextErrors);
      setError(message);
      return;
    }
    setHabits((prev) => [
      {
        id: makeId(),
        habit: habitForm.habit,
        date: habitForm.date,
        completed: habitForm.completed,
      },
      ...prev,
    ]);
    setHabitForm(createDefaultHabit(getLocalDateString()));
    setHabitErrors({});
    setHabitAttempted(false);
  }

  function addCrypto() {
    setError(null);
    setCryptoAttempted(true);
    const nextErrors: typeof cryptoErrors = {};
    let message: string | null = null;
    if (!cryptoForm.date) {
      nextErrors.date = true;
      message ??= copy.errors.cryptoDateRequired;
    }
    if (!cryptoForm.wallet) {
      nextErrors.wallet = true;
      message ??= copy.errors.cryptoWalletRequired;
    }
    if (!resolvedSymbol) {
      nextErrors.symbol = true;
      message ??= copy.errors.cryptoSymbolRequired;
    }
    if (!cryptoForm.amount) {
      nextErrors.amount = true;
      message ??= copy.errors.cryptoAmountRequired;
    }
    if (isCryptoSwap) {
      if (!cryptoForm.swap_to_symbol) {
        nextErrors.swap_to_symbol = true;
        message ??= copy.errors.cryptoSwapSymbolRequired;
      }
      if (!cryptoForm.swap_to_amount) {
        nextErrors.swap_to_amount = true;
        message ??= copy.errors.cryptoSwapAmountRequired;
      }
    }
    // Fee coin/amount pairing (all types)
    const hasFeeCoin = Boolean(cryptoForm.fee_coin_symbol);
    const hasFeeAmount = Boolean(cryptoForm.fee_amount);
    if ((hasFeeCoin && !hasFeeAmount) || (!hasFeeCoin && hasFeeAmount)) {
      nextErrors.fee_coin_symbol = !hasFeeCoin;
      nextErrors.fee_amount = !hasFeeAmount;
      message ??= copy.errors.cryptoFeePairRequired;
    }
    if (Object.keys(nextErrors).length > 0) {
      setCryptoErrors(nextErrors);
      setError(message);
      return;
    }

    const sc = selectedScenario;

    setCryptoTx((prev) => [
      {
        id: makeId(),
        date: cryptoForm.date,
        wallet: cryptoForm.wallet,
        symbol: resolvedSymbol,
        transaction_type: sc.fiscalType,
        subtype: sc.subtype,
        amount: Number(cryptoForm.amount),
        price_per_coin: cryptoForm.price_per_coin
          ? Number(cryptoForm.price_per_coin)
          : null,
        fee: cryptoForm.fee ? Number(cryptoForm.fee) : null,
        swap_to_symbol: cryptoForm.swap_to_symbol
          ? cryptoForm.swap_to_symbol.toUpperCase()
          : null,
        swap_to_amount: cryptoForm.swap_to_amount
          ? Number(cryptoForm.swap_to_amount)
          : null,
        fee_coin_symbol: cryptoForm.fee_coin_symbol
          ? cryptoForm.fee_coin_symbol.toUpperCase()
          : null,
        fee_amount: cryptoForm.fee_amount
          ? Number(cryptoForm.fee_amount)
          : null,
        override_proceeds: cryptoForm.override_proceeds
          ? Number(cryptoForm.override_proceeds)
          : null,
        override_cost_basis: cryptoForm.override_cost_basis
          ? Number(cryptoForm.override_cost_basis)
          : null,
        notes: cryptoForm.notes || null,
      },
      ...prev,
    ]);
    setCryptoForm(createDefaultCrypto(getLocalDateString()));
    setCryptoErrors({});
    setCryptoAttempted(false);
    setShowAdvancedSection(false);
  }

  // ==================== Coin select options ====================

  const coinOptions = React.useMemo(
    () => [
      ...CRYPTO_COINS.map((c) => ({
        value: c.symbol,
        label: `${c.symbol} - ${c.name}`,
      })),
      { value: "__custom__", label: "-- Custom --" },
    ],
    [],
  );

  const swapCoinOptions = React.useMemo(
    () =>
      CRYPTO_COINS.map((c) => ({
        value: c.symbol,
        label: `${c.symbol} - ${c.name}`,
      })),
    [],
  );

  const feeCoinOptions = React.useMemo(
    () =>
      CRYPTO_COINS.map((c) => ({
        value: c.symbol,
        label: c.symbol,
      })),
    [],
  );

  // ==================== Render ====================

  return (
    <div className="grid gap-10">
      {/* Header */}
      <div className="panel-gradient-strong rounded-3xl border border-border p-6 shadow-glow">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-muted-foreground">
              {copy.headerTag}
            </p>
            <h3 className="font-display mt-3 text-2xl font-semibold text-foreground">
              {copy.headerTitle}
            </h3>
            <p className="mt-3 max-w-lg text-sm text-muted-foreground">
              {copy.headerDescription}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={handleReset}>
              {copy.startOver}
            </Button>
          </div>
        </div>

        <input
          ref={fileInputRef}
          type="file"
          accept=".json,application/json"
          className="hidden"
          onChange={(event) => {
            const file = event.target.files?.[0];
            if (file) handleLoadFile(file);
            event.currentTarget.value = "";
          }}
        />

        <div className="mt-6 flex flex-wrap gap-3 text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
          <Badge variant="outline">{copy.steps.load}</Badge>
          <Badge variant="outline">{copy.steps.add}</Badge>
          <Badge variant="outline">{copy.steps.export}</Badge>
        </div>

        <div className="mt-6 grid gap-6 lg:grid-cols-[1.15fr_0.85fr]">
          <div className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-3">
              <div className="panel-gradient rounded-xl border border-border p-4">
                <p className="text-xs text-muted-foreground">
                  {copy.stats.transactions}
                </p>
                <p className="text-2xl font-semibold text-foreground">
                  {transactions.length}
                </p>
              </div>
              <div className="panel-gradient rounded-xl border border-border p-4">
                <p className="text-xs text-muted-foreground">
                  {copy.stats.habits}
                </p>
                <p className="text-2xl font-semibold text-foreground">
                  {habits.length}
                </p>
              </div>
              <div className="panel-gradient rounded-xl border border-border p-4">
                <p className="text-xs text-muted-foreground">
                  {copy.stats.crypto}
                </p>
                <p className="text-2xl font-semibold text-foreground">
                  {cryptoTx.length}
                </p>
              </div>
            </div>

            {error && (
              <div className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive-foreground">
                {error}
              </div>
            )}
          </div>

          <div className="space-y-4">
            <div className="panel-gradient rounded-2xl border border-border p-5">
              <h4 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                {copy.load.title}
              </h4>
              <p className="mt-2 text-sm text-muted-foreground">
                {copy.load.description}
              </p>
              <div className="mt-4 flex flex-wrap items-center gap-3">
                <Button variant="secondary" onClick={handleLoadClick}>
                  {copy.load.button}
                </Button>
                {loadedFile && (
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <Badge variant="outline">{copy.loaded}</Badge>
                    <span>{loadedFile}</span>
                  </div>
                )}
              </div>
            </div>

            <div className="panel-gradient rounded-2xl border border-border p-5">
              <h4 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                {copy.paste.title}
              </h4>
              <p className="mt-2 text-sm text-muted-foreground">
                {copy.paste.description}
              </p>
              <Textarea
                className="mt-4 min-h-45 font-mono text-xs"
                value={rawInput}
                onChange={(event) => setRawInput(event.target.value)}
                placeholder={copy.paste.placeholder}
              />
              <div className="mt-4 flex justify-end">
                <Button variant="outline" onClick={handleLoadPaste}>
                  {copy.paste.button}
                </Button>
              </div>
              <p className="mt-4 text-xs text-muted-foreground">
                {copy.paste.privacy}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="panel-gradient-strong rounded-3xl border border-border p-6">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-muted-foreground">
              {copy.add.title}
            </p>
            <h4 className="mt-1 text-xl font-semibold text-foreground">
              {copy.add.subtitle}
            </h4>
          </div>
          <div className="flex flex-wrap gap-2">
            {[
              { id: "transactions", label: copy.add.tabs.finances },
              { id: "crypto", label: copy.add.tabs.crypto },
              { id: "habits", label: copy.add.tabs.habits },
            ].map((tab) => (
              <button
                key={tab.id}
                className={cn(
                  "rounded-full px-3 py-1 text-xs font-semibold transition-all duration-200 ease-out",
                  "hover:-translate-y-0.5 hover:shadow-md hover:brightness-110",
                  activeTab === tab.id
                    ? "btn-active border-transparent text-primary-foreground"
                    : "btn-surface text-muted-foreground hover:text-foreground",
                )}
                onClick={() => setActiveTab(tab.id as typeof activeTab)}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>

        {/* ==================== Transactions Tab ==================== */}
        {activeTab === "transactions" && (
          <div className="mt-6 grid gap-6 lg:grid-cols-[1fr_1.2fr]">
            <div className="space-y-4 panel-gradient rounded-2xl border border-border p-5">
              <Input
                type="date"
                value={txForm.date}
                onChange={(event) => {
                  const value = event.target.value;
                  setTxForm({ ...txForm, date: value });
                  if (value) {
                    setTxErrors((prev) => ({ ...prev, date: false }));
                  }
                }}
                aria-invalid={Boolean(txAttempted && txErrors.date)}
                className={cn(
                  txAttempted &&
                    txErrors.date &&
                    "border-destructive/60 focus-visible:ring-destructive/40",
                )}
              />
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  placeholder={copy.transactions.form.account}
                  value={txForm.account}
                  onChange={(event) => {
                    const value = event.target.value;
                    setTxForm({ ...txForm, account: value });
                    if (value) {
                      setTxErrors((prev) => ({
                        ...prev,
                        account: false,
                      }));
                    }
                  }}
                  aria-invalid={Boolean(txAttempted && txErrors.account)}
                  className={cn(
                    txAttempted &&
                      txErrors.account &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
                <Input
                  list="currency-options"
                  placeholder={copy.transactions.form.currency}
                  value={txForm.currency}
                  onChange={(event) => {
                    const value = event.target.value.toUpperCase();
                    setTxForm({ ...txForm, currency: value });
                    if (value) {
                      setTxErrors((prev) => ({
                        ...prev,
                        currency: false,
                      }));
                    }
                  }}
                  aria-invalid={Boolean(txAttempted && txErrors.currency)}
                  className={cn(
                    txAttempted &&
                      txErrors.currency &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
              </div>
              <datalist id="currency-options">
                {CURRENCIES.map((c) => (
                  <option key={c} value={c} />
                ))}
              </datalist>
              <div className="grid gap-4 sm:grid-cols-2">
                <SelectField
                  value={txForm.transaction_type}
                  onChange={(v) => {
                    const nextType = v as TransactionType;
                    setTxForm({
                      ...txForm,
                      transaction_type: nextType,
                      category: nextType === "transfer" ? "" : txForm.category,
                    });
                    setTxErrors((prev) => ({
                      ...prev,
                      category: nextType === "transfer" ? false : prev.category,
                      transfer_to_account:
                        nextType === "transfer"
                          ? prev.transfer_to_account
                          : false,
                    }));
                  }}
                  options={[
                    {
                      value: "expense",
                      label: copy.types.expense,
                    },
                    {
                      value: "income",
                      label: copy.types.income,
                    },
                    {
                      value: "transfer",
                      label: copy.types.transfer,
                    },
                  ]}
                />
                <Input
                  type="number"
                  step="0.01"
                  placeholder={copy.transactions.form.amount}
                  value={txForm.amount}
                  onChange={(event) => {
                    const value = event.target.value;
                    setTxForm({ ...txForm, amount: value });
                    if (value) {
                      setTxErrors((prev) => ({
                        ...prev,
                        amount: false,
                      }));
                    }
                  }}
                  aria-invalid={Boolean(txAttempted && txErrors.amount)}
                  className={cn(
                    txAttempted &&
                      txErrors.amount &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
              </div>
              {txForm.transaction_type !== "transfer" ? (
                <SelectField
                  value={txForm.category}
                  onChange={(v) => {
                    setTxForm({ ...txForm, category: v });
                    if (v) {
                      setTxErrors((prev) => ({
                        ...prev,
                        category: false,
                      }));
                    }
                  }}
                  options={categoryOptions}
                  placeholder={copy.transactions.form.selectCategory}
                  aria-invalid={Boolean(txAttempted && txErrors.category)}
                  className={cn(
                    txAttempted &&
                      txErrors.category &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
              ) : (
                <Input
                  placeholder={copy.transactions.form.transferTo}
                  value={txForm.transfer_to_account}
                  onChange={(event) => {
                    const value = event.target.value;
                    setTxForm({
                      ...txForm,
                      transfer_to_account: value,
                    });
                    if (value) {
                      setTxErrors((prev) => ({
                        ...prev,
                        transfer_to_account: false,
                      }));
                    }
                  }}
                  aria-invalid={Boolean(
                    txAttempted && txErrors.transfer_to_account,
                  )}
                  className={cn(
                    txAttempted &&
                      txErrors.transfer_to_account &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
              )}
              <Input
                placeholder={copy.transactions.form.description}
                value={txForm.description}
                onChange={(event) =>
                  setTxForm({
                    ...txForm,
                    description: event.target.value,
                  })
                }
              />
              <div className="text-xs text-muted-foreground">
                <p>{copy.transactions.form.required}</p>
                <p>{copy.transactions.form.transferNote}</p>
              </div>
              <Button onClick={addTransaction} className="w-full">
                {copy.transactions.form.add}
              </Button>
            </div>

            <div className="panel-gradient-strong rounded-2xl border border-border p-5">
              <p className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                {copy.transactions.list.title}
              </p>
              <div className="mt-4 space-y-3">
                {transactions.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    {copy.transactions.list.empty}
                  </p>
                )}
                {transactions.map((tx) => {
                  const isExpense = tx.transaction_type === "expense";
                  const sign =
                    tx.transaction_type === "transfer"
                      ? ""
                      : isExpense
                        ? "-"
                        : "+";
                  return (
                    <div
                      key={tx.id}
                      className="rounded-lg border border-border bg-card px-3 py-2"
                    >
                      <div className="flex items-center justify-between">
                        <p className="text-sm font-semibold text-foreground">
                          {tx.account} · {sign} {tx.currency} {tx.amount}
                        </p>
                        <button
                          className="text-xs text-muted-foreground transition-all duration-200 ease-out hover:-translate-y-0.5 hover:text-foreground"
                          onClick={() =>
                            setTransactions((prev) =>
                              prev.filter((item) => item.id !== tx.id),
                            )
                          }
                        >
                          {copy.transactions.list.remove}
                        </button>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        {tx.date} · {copy.types[tx.transaction_type]}
                        {tx.transaction_type === "transfer" &&
                        tx.transfer_to_account
                          ? ` -> ${tx.transfer_to_account}`
                          : tx.category
                            ? ` · ${tx.category}`
                            : ""}
                      </p>
                      {tx.description && (
                        <p className="text-xs text-muted-foreground">
                          {tx.description}
                        </p>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}

        {/* ==================== Habits Tab ==================== */}
        {activeTab === "habits" && (
          <div className="mt-6 grid gap-6 lg:grid-cols-[1fr_1.2fr]">
            <div className="space-y-4 panel-gradient rounded-2xl border border-border p-5">
              <Input
                type="date"
                value={habitForm.date}
                onChange={(event) => {
                  const value = event.target.value;
                  setHabitForm({ ...habitForm, date: value });
                  if (value) {
                    setHabitErrors((prev) => ({
                      ...prev,
                      date: false,
                    }));
                  }
                }}
                aria-invalid={Boolean(habitAttempted && habitErrors.date)}
                className={cn(
                  habitAttempted &&
                    habitErrors.date &&
                    "border-destructive/60 focus-visible:ring-destructive/40",
                )}
              />
              <Input
                placeholder={copy.habits.form.name}
                value={habitForm.habit}
                onChange={(event) => {
                  const value = event.target.value;
                  setHabitForm({ ...habitForm, habit: value });
                  if (value) {
                    setHabitErrors((prev) => ({
                      ...prev,
                      habit: false,
                    }));
                  }
                }}
                aria-invalid={Boolean(habitAttempted && habitErrors.habit)}
                className={cn(
                  habitAttempted &&
                    habitErrors.habit &&
                    "border-destructive/60 focus-visible:ring-destructive/40",
                )}
              />
              <label className="flex items-center gap-2 text-sm text-muted-foreground">
                <input
                  type="checkbox"
                  checked={habitForm.completed}
                  onChange={(event) =>
                    setHabitForm({
                      ...habitForm,
                      completed: event.target.checked,
                    })
                  }
                />
                {copy.habits.form.completed}
              </label>
              <p className="text-xs text-muted-foreground">
                {copy.habits.form.required}
              </p>
              <Button onClick={addHabit} className="w-full">
                {copy.habits.form.add}
              </Button>
            </div>

            <div className="panel-gradient-strong rounded-2xl border border-border p-5">
              <p className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                {copy.habits.list.title}
              </p>
              <div className="mt-4 space-y-3">
                {habits.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    {copy.habits.list.empty}
                  </p>
                )}
                {habits.map((log) => (
                  <div
                    key={log.id}
                    className="rounded-xl border border-border bg-card/70 px-4 py-3 transition-all duration-200 ease-out hover:-translate-y-0.5 hover:shadow-md"
                  >
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-semibold text-foreground">
                        {log.habit}
                      </p>
                      <button
                        className="text-xs text-muted-foreground transition-all duration-200 ease-out hover:-translate-y-0.5 hover:text-foreground"
                        onClick={() =>
                          setHabits((prev) =>
                            prev.filter((item) => item.id !== log.id),
                          )
                        }
                      >
                        {copy.habits.list.remove}
                      </button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {log.date} ·{" "}
                      {log.completed
                        ? copy.habits.list.completed
                        : copy.habits.list.skipped}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* ==================== Crypto Tab ==================== */}
        {activeTab === "crypto" && (
          <div className="mt-6 grid gap-6 lg:grid-cols-[1fr_1.2fr]">
            <div className="space-y-4 panel-gradient rounded-2xl border border-border p-5">
              {/* Date */}
              <Input
                type="date"
                value={cryptoForm.date}
                onChange={(event) => {
                  const value = event.target.value;
                  setCryptoForm({ ...cryptoForm, date: value });
                  if (value) {
                    setCryptoErrors((prev) => ({
                      ...prev,
                      date: false,
                    }));
                  }
                }}
                aria-invalid={Boolean(cryptoAttempted && cryptoErrors.date)}
                className={cn(
                  cryptoAttempted &&
                    cryptoErrors.date &&
                    "border-destructive/60 focus-visible:ring-destructive/40",
                )}
              />

              {/* Wallet + Symbol */}
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  placeholder={copy.crypto.form.wallet}
                  value={cryptoForm.wallet}
                  onChange={(event) => {
                    const value = event.target.value;
                    setCryptoForm({ ...cryptoForm, wallet: value });
                    if (value) {
                      setCryptoErrors((prev) => ({
                        ...prev,
                        wallet: false,
                      }));
                    }
                  }}
                  aria-invalid={Boolean(cryptoAttempted && cryptoErrors.wallet)}
                  className={cn(
                    cryptoAttempted &&
                      cryptoErrors.wallet &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
                <SelectField
                  value={cryptoForm.symbol}
                  onChange={(v) => {
                    setCryptoForm({
                      ...cryptoForm,
                      symbol: v,
                      customSymbol: v === "__custom__" ? "" : "",
                    });
                    if (v && v !== "__custom__") {
                      setCryptoErrors((prev) => ({
                        ...prev,
                        symbol: false,
                      }));
                    }
                  }}
                  options={coinOptions}
                  placeholder={copy.crypto.form.selectSymbol}
                  aria-invalid={Boolean(cryptoAttempted && cryptoErrors.symbol)}
                  className={cn(
                    cryptoAttempted &&
                      cryptoErrors.symbol &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
              </div>

              {/* Custom symbol input */}
              {cryptoForm.symbol === "__custom__" && (
                <Input
                  placeholder={copy.crypto.form.customSymbol}
                  value={cryptoForm.customSymbol}
                  onChange={(event) => {
                    const value = event.target.value;
                    setCryptoForm({
                      ...cryptoForm,
                      customSymbol: value,
                    });
                    if (value.trim()) {
                      setCryptoErrors((prev) => ({
                        ...prev,
                        symbol: false,
                      }));
                    }
                  }}
                  aria-invalid={Boolean(cryptoAttempted && cryptoErrors.symbol)}
                  className={cn(
                    cryptoAttempted &&
                      cryptoErrors.symbol &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
              )}

              {/* Category + Subtype */}
              <div className="grid gap-4 sm:grid-cols-2">
                <SelectField
                  value={cryptoForm.scenarioGroup}
                  onChange={(v) => {
                    const nextGroup = v as typeof cryptoForm.scenarioGroup;
                    const firstInGroup = CRYPTO_SCENARIOS.find(
                      (s) => s.group === nextGroup,
                    );
                    const nextKey = firstInGroup?.key ?? "trade:buy";
                    const sc = SCENARIO_MAP[nextKey];
                    const isSwap =
                      sc?.fiscalType === "trade" && sc?.subtype === "swap";
                    setCryptoForm({
                      ...cryptoForm,
                      scenarioGroup: nextGroup,
                      scenarioKey: nextKey,
                      ...(isSwap
                        ? {}
                        : { swap_to_symbol: "", swap_to_amount: "" }),
                    });
                    setCryptoErrors((prev) => ({
                      ...prev,
                      swap_to_symbol: isSwap ? prev.swap_to_symbol : false,
                      swap_to_amount: isSwap ? prev.swap_to_amount : false,
                    }));
                  }}
                  options={Object.entries(SCENARIO_GROUPS).map(
                    ([key, labels]) => ({
                      value: key,
                      label: labels[lang],
                    }),
                  )}
                />
                <SelectField
                  value={cryptoForm.scenarioKey}
                  onChange={(v) => {
                    const sc = SCENARIO_MAP[v];
                    const isSwap =
                      sc?.fiscalType === "trade" && sc?.subtype === "swap";
                    setCryptoForm({
                      ...cryptoForm,
                      scenarioKey: v,
                      ...(isSwap
                        ? {}
                        : { swap_to_symbol: "", swap_to_amount: "" }),
                    });
                    setCryptoErrors((prev) => ({
                      ...prev,
                      swap_to_symbol: isSwap ? prev.swap_to_symbol : false,
                      swap_to_amount: isSwap ? prev.swap_to_amount : false,
                    }));
                  }}
                  options={subtypeOptions}
                />
              </div>

              {/* Amount */}
              <div>
                <Input
                  type="number"
                  step="0.00000001"
                  placeholder={copy.crypto.form.amount}
                  value={cryptoForm.amount}
                  onChange={(event) => {
                    const value = event.target.value;
                    setCryptoForm({ ...cryptoForm, amount: value });
                    if (value) {
                      setCryptoErrors((prev) => ({
                        ...prev,
                        amount: false,
                      }));
                    }
                  }}
                  aria-invalid={Boolean(cryptoAttempted && cryptoErrors.amount)}
                  className={cn(
                    cryptoAttempted &&
                      cryptoErrors.amount &&
                      "border-destructive/60 focus-visible:ring-destructive/40",
                  )}
                />
              </div>

              {/* Swap-specific: to_symbol + to_amount */}
              {isCryptoSwap && (
                <div className="grid gap-4 sm:grid-cols-2">
                  <SelectField
                    value={cryptoForm.swap_to_symbol}
                    onChange={(v) => {
                      setCryptoForm({
                        ...cryptoForm,
                        swap_to_symbol: v,
                      });
                      if (v) {
                        setCryptoErrors((prev) => ({
                          ...prev,
                          swap_to_symbol: false,
                        }));
                      }
                    }}
                    options={swapCoinOptions}
                    placeholder={copy.crypto.form.swapToSymbol}
                    aria-invalid={Boolean(
                      cryptoAttempted && cryptoErrors.swap_to_symbol,
                    )}
                    className={cn(
                      cryptoAttempted &&
                        cryptoErrors.swap_to_symbol &&
                        "border-destructive/60 focus-visible:ring-destructive/40",
                    )}
                  />
                  <Input
                    type="number"
                    step="0.00000001"
                    placeholder={copy.crypto.form.swapToAmount}
                    value={cryptoForm.swap_to_amount}
                    onChange={(event) => {
                      const value = event.target.value;
                      setCryptoForm({
                        ...cryptoForm,
                        swap_to_amount: value,
                      });
                      if (value) {
                        setCryptoErrors((prev) => ({
                          ...prev,
                          swap_to_amount: false,
                        }));
                      }
                    }}
                    aria-invalid={Boolean(
                      cryptoAttempted && cryptoErrors.swap_to_amount,
                    )}
                    className={cn(
                      cryptoAttempted &&
                        cryptoErrors.swap_to_amount &&
                        "border-destructive/60 focus-visible:ring-destructive/40",
                    )}
                  />
                </div>
              )}

              {/* Price + Fee USD */}
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  type="number"
                  step="0.01"
                  placeholder={copy.crypto.form.price}
                  value={cryptoForm.price_per_coin}
                  onChange={(event) =>
                    setCryptoForm({
                      ...cryptoForm,
                      price_per_coin: event.target.value,
                    })
                  }
                />
                <Input
                  type="number"
                  step="0.01"
                  placeholder={copy.crypto.form.fee}
                  value={cryptoForm.fee}
                  onChange={(event) =>
                    setCryptoForm({
                      ...cryptoForm,
                      fee: event.target.value,
                    })
                  }
                />
              </div>

              {/* Notes */}
              <Input
                placeholder={copy.crypto.form.notes}
                value={cryptoForm.notes}
                onChange={(event) =>
                  setCryptoForm({
                    ...cryptoForm,
                    notes: event.target.value,
                  })
                }
              />

              {/* Advanced section toggle */}
              <button
                type="button"
                className="flex items-center gap-2 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground"
                onClick={() => setShowAdvancedSection((prev) => !prev)}
              >
                <span
                  className="inline-block transition-transform duration-200"
                  style={{
                    transform: showAdvancedSection
                      ? "rotate(90deg)"
                      : "rotate(0deg)",
                  }}
                >
                  &#9654;
                </span>
                {copy.crypto.form.advancedSection}
              </button>

              {showAdvancedSection && (
                <div className="space-y-3 rounded-xl border border-border/50 bg-card/30 p-4">
                  {/* Fee in crypto */}
                  <div className="grid gap-4 sm:grid-cols-2">
                    <SelectField
                      value={cryptoForm.fee_coin_symbol}
                      onChange={(v) => {
                        setCryptoForm({
                          ...cryptoForm,
                          fee_coin_symbol: v,
                        });
                        if (v) {
                          setCryptoErrors((prev) => ({
                            ...prev,
                            fee_coin_symbol: false,
                          }));
                        }
                      }}
                      options={feeCoinOptions}
                      placeholder={copy.crypto.form.feeCoinSymbol}
                      aria-invalid={Boolean(
                        cryptoAttempted && cryptoErrors.fee_coin_symbol,
                      )}
                      className={cn(
                        cryptoAttempted &&
                          cryptoErrors.fee_coin_symbol &&
                          "border-destructive/60 focus-visible:ring-destructive/40",
                      )}
                    />
                    <Input
                      type="number"
                      step="0.00000001"
                      placeholder={copy.crypto.form.feeAmount}
                      value={cryptoForm.fee_amount}
                      onChange={(event) => {
                        const value = event.target.value;
                        setCryptoForm({
                          ...cryptoForm,
                          fee_amount: value,
                        });
                        if (value) {
                          setCryptoErrors((prev) => ({
                            ...prev,
                            fee_amount: false,
                          }));
                        }
                      }}
                      aria-invalid={Boolean(
                        cryptoAttempted && cryptoErrors.fee_amount,
                      )}
                      className={cn(
                        cryptoAttempted &&
                          cryptoErrors.fee_amount &&
                          "border-destructive/60 focus-visible:ring-destructive/40",
                      )}
                    />
                  </div>
                  {/* Override proceeds / cost basis */}
                  <div className="grid gap-4 sm:grid-cols-2">
                    <Input
                      type="number"
                      step="0.01"
                      placeholder={copy.crypto.form.overrideProceeds}
                      value={cryptoForm.override_proceeds}
                      onChange={(event) =>
                        setCryptoForm({
                          ...cryptoForm,
                          override_proceeds: event.target.value,
                        })
                      }
                    />
                    <Input
                      type="number"
                      step="0.01"
                      placeholder={copy.crypto.form.overrideCostBasis}
                      value={cryptoForm.override_cost_basis}
                      onChange={(event) =>
                        setCryptoForm({
                          ...cryptoForm,
                          override_cost_basis: event.target.value,
                        })
                      }
                    />
                  </div>
                </div>
              )}

              <p className="text-xs text-muted-foreground">
                {copy.crypto.form.required}
              </p>
              <Button onClick={addCrypto} className="w-full">
                {copy.crypto.form.add}
              </Button>
            </div>

            {/* Crypto list */}
            <div className="panel-gradient-strong rounded-2xl border border-border p-5">
              <p className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                {copy.crypto.list.title}
              </p>
              <div className="mt-4 space-y-3">
                {cryptoTx.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    {copy.crypto.list.empty}
                  </p>
                )}
                {cryptoTx.map((tx) => (
                  <div
                    key={tx.id}
                    className="rounded-xl border border-border bg-card/70 px-4 py-3 transition-all duration-200 ease-out hover:-translate-y-0.5 hover:shadow-md"
                  >
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-semibold text-foreground">
                        {tx.wallet} ·{" "}
                        {tx.transaction_type === "trade" &&
                        tx.subtype === "swap" &&
                        tx.swap_to_symbol &&
                        tx.swap_to_amount
                          ? `${tx.symbol} ${tx.amount} -> ${tx.swap_to_symbol} ${tx.swap_to_amount}`
                          : `${tx.symbol} ${tx.amount}`}
                      </p>
                      <button
                        className="text-xs text-muted-foreground transition-all duration-200 ease-out hover:-translate-y-0.5 hover:text-foreground"
                        onClick={() =>
                          setCryptoTx((prev) =>
                            prev.filter((item) => item.id !== tx.id),
                          )
                        }
                      >
                        {copy.crypto.list.remove}
                      </button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {tx.date} ·{" "}
                      {scenarioLabel(
                        resolveScenarioKey(
                          tx.transaction_type,
                          tx.subtype ?? null,
                        ),
                      )}
                    </p>
                    {tx.fee_coin_symbol && tx.fee_amount && (
                      <p className="text-xs text-muted-foreground">
                        Fee: {tx.fee_amount} {tx.fee_coin_symbol}
                      </p>
                    )}
                    {tx.notes && (
                      <p className="text-xs text-muted-foreground">
                        {tx.notes}
                      </p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Export */}
      <div className="panel-gradient rounded-3xl border border-border p-6">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <h4 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
              {copy.export.title}
            </h4>
            <Badge variant="outline">JSON v1</Badge>
          </div>
          <Button onClick={handleDownload}>{copy.export.download}</Button>
        </div>
        <Textarea
          className="mt-4 min-h-60 font-mono text-xs"
          value={exportJson}
          readOnly
        />
      </div>
    </div>
  );
}
