import * as React from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

type TransactionType = "income" | "expense" | "transfer";
type CryptoType = "buy" | "sell" | "transfer_in" | "transfer_out";

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
  amount: number;
  price_per_coin?: number | null;
  fee?: number | null;
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
    amount: number;
    price_per_coin?: number | null;
    fee?: number | null;
    notes?: string | null;
  }[];
};

const translations = {
  en: {
    headerTag: "Sanctum Generator",
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
      description: "Upload a sanctum_export.json file to append new entries to the same log.",
      button: "Upload JSON",
    },
    paste: {
      title: "Paste JSON",
      description: "Use this if you only have access to text notes. The payload stays local.",
      placeholder: "Paste a sanctum_export.json payload...",
      button: "Load From Paste",
      privacy: "Privacy: data never leaves your browser. Works offline after first load.",
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
        category: "Category",
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
        empty: "No habit logs yet. Capture today’s progress.",
        remove: "Remove",
        completed: "Completed",
        skipped: "Skipped",
      },
    },
    crypto: {
      form: {
        wallet: "Wallet",
        symbol: "Symbol (BTC)",
        amount: "Amount",
        price: "Price per coin (optional)",
        fee: "Fee (optional)",
        notes: "Notes (optional)",
        add: "Add Crypto Entry",
        required: "Required: Date, Wallet, Symbol, Amount.",
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
    },
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
      transactionCategoryRequired: "Category is required unless this is a transfer.",
      transactionTransferRequired: "Transfer requires a destination account.",
      habitDateRequired: "Habit date is required.",
      habitNameRequired: "Habit name is required.",
      cryptoDateRequired: "Crypto date is required.",
      cryptoWalletRequired: "Crypto wallet is required.",
      cryptoSymbolRequired: "Crypto symbol is required.",
      cryptoAmountRequired: "Crypto amount is required.",
    },
  },
  es: {
    headerTag: "Generador Sanctum",
    headerTitle: "Construye un registro seguro",
    headerDescription:
      "Este generador respeta el esquema de importación de Sanctum. Los nombres de cuentas, hábitos, wallets y categorías deben existir en tu bóveda para que la importación funcione.",
    startOver: "Reiniciar",
    steps: {
      load: "Paso 1: Cargar",
      add: "Paso 2: Agregar",
      export: "Paso 3: Exportar",
    },
    stats: {
      transactions: "Transacciones",
      habits: "Registros de hábitos",
      crypto: "Movimientos cripto",
    },
    loaded: "Cargado",
    load: {
      title: "Cargar JSON existente",
      description: "Sube un archivo sanctum_export.json para agregar nuevas entradas al mismo registro.",
      button: "Subir JSON",
    },
    paste: {
      title: "Pegar JSON",
      description: "Úsalo si solo tienes acceso a notas de texto. El contenido se mantiene local.",
      placeholder: "Pega el contenido de sanctum_export.json...",
      button: "Cargar desde pegado",
      privacy: "Privacidad: los datos nunca salen del navegador. Funciona offline después de la primera carga.",
    },
    add: {
      title: "Agregar entradas",
      subtitle: "Construye tu registro",
      tabs: {
        finances: "Finanzas",
        crypto: "Cripto",
        habits: "Hábitos",
      },
    },
    transactions: {
      form: {
        account: "Cuenta",
        category: "Categoría",
        transferTo: "Transferir a cuenta",
        description: "Descripción",
        amount: "Monto",
        add: "Agregar transacción",
        required: "Requerido: Fecha, Cuenta, Monto, Moneda.",
        transferNote: "La categoría es obligatoria salvo que sea una transferencia.",
      },
      list: {
        title: "Transacciones recientes",
        empty: "Aún no hay transacciones. Agrega la primera.",
        remove: "Eliminar",
      },
    },
    habits: {
      form: {
        name: "Nombre del hábito",
        completed: "Completado",
        add: "Agregar registro de hábito",
        required: "Requerido: Fecha, Nombre del hábito.",
      },
      list: {
        title: "Registros de hábitos",
        empty: "Aún no hay registros. Captura el progreso de hoy.",
        remove: "Eliminar",
        completed: "Completado",
        skipped: "Omitido",
      },
    },
    crypto: {
      form: {
        wallet: "Wallet",
        symbol: "Símbolo (BTC)",
        amount: "Monto",
        price: "Precio por moneda (opcional)",
        fee: "Comisión (opcional)",
        notes: "Notas (opcional)",
        add: "Agregar movimiento cripto",
        required: "Requerido: Fecha, Wallet, Símbolo, Monto.",
      },
      list: {
        title: "Movimientos cripto",
        empty: "Aún no hay movimientos. Agrega actividad aquí.",
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
    },
    export: {
      title: "Vista previa de exportación",
      download: "Descargar JSON",
    },
    errors: {
      invalidJson: "Estructura JSON inválida.",
      unsupportedVersion: "Versión no compatible. Se esperaba la versión 1.0.",
      parseFailed: "No se pudo leer el JSON.",
      pasteRequired: "Pega un JSON antes de cargar.",
      transactionDateRequired: "La fecha de la transacción es obligatoria.",
      transactionAccountRequired: "La cuenta de la transacción es obligatoria.",
      transactionAmountRequired: "El monto de la transacción es obligatorio.",
      transactionCurrencyRequired: "La moneda de la transacción es obligatoria.",
      transactionCategoryRequired: "La categoría es obligatoria salvo que sea una transferencia.",
      transactionTransferRequired: "La transferencia requiere una cuenta destino.",
      habitDateRequired: "La fecha del hábito es obligatoria.",
      habitNameRequired: "El nombre del hábito es obligatorio.",
      cryptoDateRequired: "La fecha del movimiento cripto es obligatoria.",
      cryptoWalletRequired: "La wallet es obligatoria.",
      cryptoSymbolRequired: "El símbolo cripto es obligatorio.",
      cryptoAmountRequired: "El monto cripto es obligatorio.",
    },
  },
};

const defaultTx = {
  date: "",
  account: "",
  transaction_type: "expense" as TransactionType,
  amount: "",
  currency: "USD",
  category: "",
  description: "",
  transfer_to_account: "",
};

const defaultHabit = {
  habit: "",
  date: "",
  completed: true,
};

const defaultCrypto = {
  date: "",
  wallet: "",
  symbol: "",
  transaction_type: "buy" as CryptoType,
  amount: "",
  price_per_coin: "",
  fee: "",
  notes: "",
};

function makeId() {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `id-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function normalizeTransactionType(value: unknown): TransactionType {
  const normalized = String(value ?? "").toLowerCase();
  if (normalized === "income" || normalized === "expense" || normalized === "transfer") {
    return normalized as TransactionType;
  }
  return "expense";
}

function normalizeCryptoType(value: unknown): CryptoType {
  const normalized = String(value ?? "").toLowerCase();
  if (
    normalized === "buy" ||
    normalized === "sell" ||
    normalized === "transfer_in" ||
    normalized === "transfer_out"
  ) {
    return normalized as CryptoType;
  }
  return "buy";
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

export default function Generator() {
  const [lang, setLang] = React.useState<"en" | "es">(getInitialLang);
  const copy = React.useMemo(() => translations[lang] ?? translations.en, [lang]);

  const [transactions, setTransactions] = React.useState<Transaction[]>([]);
  const [habits, setHabits] = React.useState<HabitLog[]>([]);
  const [cryptoTx, setCryptoTx] = React.useState<CryptoTransaction[]>([]);
  const [activeTab, setActiveTab] = React.useState<"transactions" | "habits" | "crypto">(
    "transactions"
  );
  const [error, setError] = React.useState<string | null>(null);
  const [loadedFile, setLoadedFile] = React.useState<string | null>(null);
  const [rawInput, setRawInput] = React.useState("");

  const [txForm, setTxForm] = React.useState(defaultTx);
  const [txErrors, setTxErrors] = React.useState<{
    date?: boolean;
    account?: boolean;
    amount?: boolean;
    currency?: boolean;
    category?: boolean;
    transfer_to_account?: boolean;
  }>({});
  const [txAttempted, setTxAttempted] = React.useState(false);
  const [habitForm, setHabitForm] = React.useState(defaultHabit);
  const [habitErrors, setHabitErrors] = React.useState<{
    date?: boolean;
    habit?: boolean;
  }>({});
  const [habitAttempted, setHabitAttempted] = React.useState(false);
  const [cryptoForm, setCryptoForm] = React.useState(defaultCrypto);
  const [cryptoErrors, setCryptoErrors] = React.useState<{
    date?: boolean;
    wallet?: boolean;
    symbol?: boolean;
    amount?: boolean;
  }>({});
  const [cryptoAttempted, setCryptoAttempted] = React.useState(false);

  const fileInputRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    const handleLangChange = (event: Event) => {
      const detail = (event as CustomEvent<{ lang?: string }>).detail;
      const nextLang = detail?.lang ?? document.documentElement.lang;
      const next = nextLang === "es" ? "es" : "en";
      setLang(next);
    };

    window.addEventListener("sanctum-lang-change", handleLangChange);
    return () => window.removeEventListener("sanctum-lang-change", handleLangChange);
  }, []);

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
        ...tx,
        type: transaction_type,
        price_per_coin: tx.price_per_coin ?? null,
        fee: tx.fee ?? null,
        notes: tx.notes ?? null,
      })),
    };
  }, [transactions, habits, cryptoTx]);

  const exportJson = React.useMemo(() => JSON.stringify(exportPayload, null, 2), [exportPayload]);

  function handleLoadFile(file: File) {
    const reader = new FileReader();
    reader.onload = () => {
      try {
        const text = String(reader.result ?? "");
        const parsed = JSON.parse(text);
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
              transaction_type: normalizeTransactionType(tx.transaction_type ?? tx.type),
              amount: Number(tx.amount ?? 0),
              currency: String(tx.currency ?? ""),
              category: String(tx.category ?? ""),
              description: String(tx.description ?? ""),
              transfer_to_account: tx.transfer_to_account ?? null,
            }))
          : [];
        const nextHabits = Array.isArray(parsed.habit_logs)
          ? parsed.habit_logs.map((log: HabitLog) => ({
              id: makeId(),
              ...log,
            }))
          : [];
        const nextCrypto = Array.isArray(parsed.crypto_transactions)
          ? parsed.crypto_transactions.map((tx: any) => ({
              id: makeId(),
              date: String(tx.date ?? ""),
              wallet: String(tx.wallet ?? ""),
              symbol: String(tx.symbol ?? ""),
              transaction_type: normalizeCryptoType(tx.transaction_type ?? tx.type),
              amount: Number(tx.amount ?? 0),
              price_per_coin: tx.price_per_coin ?? null,
              fee: tx.fee ?? null,
              notes: tx.notes ?? null,
            }))
          : [];

        setTransactions(nextTransactions);
        setHabits(nextHabits);
        setCryptoTx(nextCrypto);
        setLoadedFile(file.name);
        setRawInput("");
        setError(null);
        setTxErrors({});
        setHabitErrors({});
        setCryptoErrors({});
        setTxAttempted(false);
        setHabitAttempted(false);
        setCryptoAttempted(false);
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
      if (!isSupportedVersion(parsed.version)) {
        throw new Error(copy.errors.unsupportedVersion);
      }
      setTransactions(
        Array.isArray(parsed.transactions)
          ? parsed.transactions.map((tx: any) => ({
              id: makeId(),
              date: String(tx.date ?? ""),
              account: String(tx.account ?? ""),
              transaction_type: normalizeTransactionType(tx.transaction_type ?? tx.type),
              amount: Number(tx.amount ?? 0),
              currency: String(tx.currency ?? ""),
              category: String(tx.category ?? ""),
              description: String(tx.description ?? ""),
              transfer_to_account: tx.transfer_to_account ?? null,
            }))
          : []
      );
      setHabits(
        Array.isArray(parsed.habit_logs)
          ? parsed.habit_logs.map((log: HabitLog) => ({ id: makeId(), ...log }))
          : []
      );
      setCryptoTx(
        Array.isArray(parsed.crypto_transactions)
          ? parsed.crypto_transactions.map((tx: any) => ({
              id: makeId(),
              date: String(tx.date ?? ""),
              wallet: String(tx.wallet ?? ""),
              symbol: String(tx.symbol ?? ""),
              transaction_type: normalizeCryptoType(tx.transaction_type ?? tx.type),
              amount: Number(tx.amount ?? 0),
              price_per_coin: tx.price_per_coin ?? null,
              fee: tx.fee ?? null,
              notes: tx.notes ?? null,
            }))
          : []
      );
      setLoadedFile("pasted JSON");
      setError(null);
      setTxErrors({});
      setHabitErrors({});
      setCryptoErrors({});
      setTxAttempted(false);
      setHabitAttempted(false);
      setCryptoAttempted(false);
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
    setTxErrors({});
    setHabitErrors({});
    setCryptoErrors({});
    setTxAttempted(false);
    setHabitAttempted(false);
    setCryptoAttempted(false);
  }

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
    setTxForm(defaultTx);
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
    setHabitForm(defaultHabit);
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
    if (!cryptoForm.symbol) {
      nextErrors.symbol = true;
      message ??= copy.errors.cryptoSymbolRequired;
    }
    if (!cryptoForm.amount) {
      nextErrors.amount = true;
      message ??= copy.errors.cryptoAmountRequired;
    }
    if (Object.keys(nextErrors).length > 0) {
      setCryptoErrors(nextErrors);
      setError(message);
      return;
    }
    setCryptoTx((prev) => [
      {
        id: makeId(),
        date: cryptoForm.date,
        wallet: cryptoForm.wallet,
        symbol: cryptoForm.symbol.toUpperCase(),
        transaction_type: cryptoForm.transaction_type,
        amount: Number(cryptoForm.amount),
        price_per_coin: cryptoForm.price_per_coin ? Number(cryptoForm.price_per_coin) : null,
        fee: cryptoForm.fee ? Number(cryptoForm.fee) : null,
        notes: cryptoForm.notes || null,
      },
      ...prev,
    ]);
    setCryptoForm(defaultCrypto);
    setCryptoErrors({});
    setCryptoAttempted(false);
  }

  return (
    <div className="grid gap-10">
      <div className="panel-gradient-strong rounded-3xl border border-border p-6 shadow-glow">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-muted-foreground">
              {copy.headerTag}
            </p>
            <h3 className="font-display mt-3 text-2xl font-semibold text-foreground">
              {copy.headerTitle}
            </h3>
            <p className="mt-3 text-sm text-muted-foreground max-w-lg">
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
                <p className="text-xs text-muted-foreground">{copy.stats.transactions}</p>
                <p className="text-2xl font-semibold text-foreground">{transactions.length}</p>
              </div>
              <div className="panel-gradient rounded-xl border border-border p-4">
                <p className="text-xs text-muted-foreground">{copy.stats.habits}</p>
                <p className="text-2xl font-semibold text-foreground">{habits.length}</p>
              </div>
              <div className="panel-gradient rounded-xl border border-border p-4">
                <p className="text-xs text-muted-foreground">{copy.stats.crypto}</p>
                <p className="text-2xl font-semibold text-foreground">{cryptoTx.length}</p>
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
              <p className="mt-2 text-sm text-muted-foreground">{copy.load.description}</p>
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
              <p className="mt-2 text-sm text-muted-foreground">{copy.paste.description}</p>
              <Textarea
                className="mt-4 min-h-[180px] font-mono text-xs"
                value={rawInput}
                onChange={(event) => setRawInput(event.target.value)}
                placeholder={copy.paste.placeholder}
              />
              <div className="mt-4 flex justify-end">
                <Button variant="outline" onClick={handleLoadPaste}>
                  {copy.paste.button}
                </Button>
              </div>
              <p className="mt-4 text-xs text-muted-foreground">{copy.paste.privacy}</p>
            </div>
          </div>
        </div>
      </div>

      <div className="panel-gradient-strong rounded-3xl border border-border p-6">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-muted-foreground">
              {copy.add.title}
            </p>
            <h4 className="mt-1 text-xl font-semibold text-foreground">{copy.add.subtitle}</h4>
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
                    : "btn-surface text-muted-foreground hover:text-foreground"
                )}
                onClick={() => setActiveTab(tab.id as typeof activeTab)}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>

        {activeTab === "transactions" && (
          <div className="mt-6 grid gap-6 lg:grid-cols-[1fr_1.2fr]">
            <div className="space-y-4">
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
                    "border-destructive/60 focus-visible:ring-destructive/40"
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
                      setTxErrors((prev) => ({ ...prev, account: false }));
                    }
                  }}
                  aria-invalid={Boolean(txAttempted && txErrors.account)}
                  className={cn(
                    txAttempted &&
                      txErrors.account &&
                      "border-destructive/60 focus-visible:ring-destructive/40"
                  )}
                />
                <select
                  className={cn(
                    "h-10 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground",
                    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                    txAttempted &&
                      txErrors.currency &&
                      "border-destructive/60 focus-visible:ring-destructive/40"
                  )}
                  value={txForm.currency}
                  onChange={(event) => {
                    const value = event.target.value;
                    setTxForm({ ...txForm, currency: value });
                    if (value) {
                      setTxErrors((prev) => ({ ...prev, currency: false }));
                    }
                  }}
                  aria-invalid={Boolean(txAttempted && txErrors.currency)}
                >
                  <option value="USD">USD</option>
                  <option value="CLP">CLP</option>
                </select>
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <select
                  className="h-10 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  value={txForm.transaction_type}
                  onChange={(event) => {
                    const nextType = event.target.value as TransactionType;
                    setTxForm({
                      ...txForm,
                      transaction_type: nextType,
                    });
                    setTxErrors((prev) => ({
                      ...prev,
                      category: nextType === "transfer" ? false : prev.category,
                      transfer_to_account:
                        nextType === "transfer" ? prev.transfer_to_account : false,
                    }));
                  }}
                >
                  <option value="expense">{copy.types.expense}</option>
                  <option value="income">{copy.types.income}</option>
                  <option value="transfer">{copy.types.transfer}</option>
                </select>
                <Input
                  type="number"
                  step="0.01"
                  placeholder={copy.transactions.form.amount}
                  value={txForm.amount}
                  onChange={(event) => {
                    const value = event.target.value;
                    setTxForm({ ...txForm, amount: value });
                    if (value) {
                      setTxErrors((prev) => ({ ...prev, amount: false }));
                    }
                  }}
                  aria-invalid={Boolean(txAttempted && txErrors.amount)}
                  className={cn(
                    txAttempted &&
                      txErrors.amount &&
                      "border-destructive/60 focus-visible:ring-destructive/40"
                  )}
                />
              </div>
              <Input
                placeholder={copy.transactions.form.category}
                value={txForm.category}
                onChange={(event) => {
                  const value = event.target.value;
                  setTxForm({ ...txForm, category: value });
                  if (value) {
                    setTxErrors((prev) => ({ ...prev, category: false }));
                  }
                }}
                disabled={txForm.transaction_type === "transfer"}
                aria-invalid={Boolean(txAttempted && txErrors.category)}
                className={cn(
                  txAttempted &&
                    txErrors.category &&
                    "border-destructive/60 focus-visible:ring-destructive/40"
                )}
              />
              {txForm.transaction_type === "transfer" && (
                <Input
                  placeholder={copy.transactions.form.transferTo}
                  value={txForm.transfer_to_account}
                  onChange={(event) => {
                    const value = event.target.value;
                    setTxForm({ ...txForm, transfer_to_account: value });
                    if (value) {
                      setTxErrors((prev) => ({ ...prev, transfer_to_account: false }));
                    }
                  }}
                  aria-invalid={Boolean(txAttempted && txErrors.transfer_to_account)}
                  className={cn(
                    txAttempted &&
                      txErrors.transfer_to_account &&
                      "border-destructive/60 focus-visible:ring-destructive/40"
                  )}
                />
              )}
              <Input
                placeholder={copy.transactions.form.description}
                value={txForm.description}
                onChange={(event) => setTxForm({ ...txForm, description: event.target.value })}
              />
              <div className="text-xs text-muted-foreground">
                <p>{copy.transactions.form.required}</p>
                <p>{copy.transactions.form.transferNote}</p>
              </div>
              <Button onClick={addTransaction}>{copy.transactions.form.add}</Button>
            </div>

            <div className="panel-gradient rounded-2xl border border-border p-4">
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
                  const sign = tx.transaction_type === "transfer" ? "" : isExpense ? "-" : "+";
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
                            setTransactions((prev) => prev.filter((item) => item.id !== tx.id))
                          }
                        >
                          {copy.transactions.list.remove}
                        </button>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        {tx.date} · {copy.types[tx.transaction_type]}
                        {tx.transaction_type === "transfer" && tx.transfer_to_account
                          ? ` → ${tx.transfer_to_account}`
                          : ""}
                      </p>
                      {tx.description && (
                        <p className="text-xs text-muted-foreground">{tx.description}</p>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}

        {activeTab === "habits" && (
          <div className="mt-6 grid gap-6 lg:grid-cols-[1fr_1.2fr]">
            <div className="space-y-4">
              <Input
                type="date"
                value={habitForm.date}
                onChange={(event) => {
                  const value = event.target.value;
                  setHabitForm({ ...habitForm, date: value });
                  if (value) {
                    setHabitErrors((prev) => ({ ...prev, date: false }));
                  }
                }}
                aria-invalid={Boolean(habitAttempted && habitErrors.date)}
                className={cn(
                  habitAttempted &&
                    habitErrors.date &&
                    "border-destructive/60 focus-visible:ring-destructive/40"
                )}
              />
              <Input
                placeholder={copy.habits.form.name}
                value={habitForm.habit}
                onChange={(event) => {
                  const value = event.target.value;
                  setHabitForm({ ...habitForm, habit: value });
                  if (value) {
                    setHabitErrors((prev) => ({ ...prev, habit: false }));
                  }
                }}
                aria-invalid={Boolean(habitAttempted && habitErrors.habit)}
                className={cn(
                  habitAttempted &&
                    habitErrors.habit &&
                    "border-destructive/60 focus-visible:ring-destructive/40"
                )}
              />
              <label className="flex items-center gap-2 text-sm text-muted-foreground">
                <input
                  type="checkbox"
                  checked={habitForm.completed}
                  onChange={(event) =>
                    setHabitForm({ ...habitForm, completed: event.target.checked })
                  }
                />
                {copy.habits.form.completed}
              </label>
              <p className="text-xs text-muted-foreground">{copy.habits.form.required}</p>
              <Button onClick={addHabit}>{copy.habits.form.add}</Button>
            </div>

            <div className="panel-gradient rounded-2xl border border-border p-4">
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
                    className="rounded-lg border border-border bg-card px-3 py-2"
                  >
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-semibold text-foreground">{log.habit}</p>
                      <button
                        className="text-xs text-muted-foreground transition-all duration-200 ease-out hover:-translate-y-0.5 hover:text-foreground"
                        onClick={() => setHabits((prev) => prev.filter((item) => item.id !== log.id))}
                      >
                        {copy.habits.list.remove}
                      </button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {log.date} · {log.completed ? copy.habits.list.completed : copy.habits.list.skipped}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === "crypto" && (
          <div className="mt-6 grid gap-6 lg:grid-cols-[1fr_1.2fr]">
            <div className="space-y-4">
              <Input
                type="date"
                value={cryptoForm.date}
                onChange={(event) => {
                  const value = event.target.value;
                  setCryptoForm({ ...cryptoForm, date: value });
                  if (value) {
                    setCryptoErrors((prev) => ({ ...prev, date: false }));
                  }
                }}
                aria-invalid={Boolean(cryptoAttempted && cryptoErrors.date)}
                className={cn(
                  cryptoAttempted &&
                    cryptoErrors.date &&
                    "border-destructive/60 focus-visible:ring-destructive/40"
                )}
              />
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  placeholder={copy.crypto.form.wallet}
                  value={cryptoForm.wallet}
                  onChange={(event) => {
                    const value = event.target.value;
                    setCryptoForm({ ...cryptoForm, wallet: value });
                    if (value) {
                      setCryptoErrors((prev) => ({ ...prev, wallet: false }));
                    }
                  }}
                  aria-invalid={Boolean(cryptoAttempted && cryptoErrors.wallet)}
                  className={cn(
                    cryptoAttempted &&
                      cryptoErrors.wallet &&
                      "border-destructive/60 focus-visible:ring-destructive/40"
                  )}
                />
                <Input
                  placeholder={copy.crypto.form.symbol}
                  value={cryptoForm.symbol}
                  onChange={(event) => {
                    const value = event.target.value;
                    setCryptoForm({ ...cryptoForm, symbol: value });
                    if (value) {
                      setCryptoErrors((prev) => ({ ...prev, symbol: false }));
                    }
                  }}
                  aria-invalid={Boolean(cryptoAttempted && cryptoErrors.symbol)}
                  className={cn(
                    cryptoAttempted &&
                      cryptoErrors.symbol &&
                      "border-destructive/60 focus-visible:ring-destructive/40"
                  )}
                />
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <select
                  className="h-10 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground"
                  value={cryptoForm.transaction_type}
                  onChange={(event) =>
                    setCryptoForm({
                      ...cryptoForm,
                      transaction_type: event.target.value as CryptoType,
                    })
                  }
                >
                  <option value="buy">{copy.types.buy}</option>
                  <option value="sell">{copy.types.sell}</option>
                  <option value="transfer_in">{copy.types.transfer_in}</option>
                  <option value="transfer_out">{copy.types.transfer_out}</option>
                </select>
                <Input
                  type="number"
                  step="0.00000001"
                  placeholder={copy.crypto.form.amount}
                  value={cryptoForm.amount}
                  onChange={(event) => {
                    const value = event.target.value;
                    setCryptoForm({ ...cryptoForm, amount: value });
                    if (value) {
                      setCryptoErrors((prev) => ({ ...prev, amount: false }));
                    }
                  }}
                  aria-invalid={Boolean(cryptoAttempted && cryptoErrors.amount)}
                  className={cn(
                    cryptoAttempted &&
                      cryptoErrors.amount &&
                      "border-destructive/60 focus-visible:ring-destructive/40"
                  )}
                />
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  type="number"
                  step="0.01"
                  placeholder={copy.crypto.form.price}
                  value={cryptoForm.price_per_coin}
                  onChange={(event) =>
                    setCryptoForm({ ...cryptoForm, price_per_coin: event.target.value })
                  }
                />
                <Input
                  type="number"
                  step="0.01"
                  placeholder={copy.crypto.form.fee}
                  value={cryptoForm.fee}
                  onChange={(event) => setCryptoForm({ ...cryptoForm, fee: event.target.value })}
                />
              </div>
              <Input
                placeholder={copy.crypto.form.notes}
                value={cryptoForm.notes}
                onChange={(event) => setCryptoForm({ ...cryptoForm, notes: event.target.value })}
              />
              <p className="text-xs text-muted-foreground">{copy.crypto.form.required}</p>
              <Button onClick={addCrypto}>{copy.crypto.form.add}</Button>
            </div>

            <div className="panel-gradient rounded-2xl border border-border p-4">
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
                    className="rounded-lg border border-border bg-card px-3 py-2"
                  >
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-semibold text-foreground">
                        {tx.wallet} · {tx.symbol} {tx.amount}
                      </p>
                      <button
                        className="text-xs text-muted-foreground transition-all duration-200 ease-out hover:-translate-y-0.5 hover:text-foreground"
                        onClick={() =>
                          setCryptoTx((prev) => prev.filter((item) => item.id !== tx.id))
                        }
                      >
                        {copy.crypto.list.remove}
                      </button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {tx.date} · {copy.types[tx.transaction_type]}
                    </p>
                    {tx.notes && (
                      <p className="text-xs text-muted-foreground">{tx.notes}</p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>

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
          className="mt-4 min-h-[240px] font-mono text-xs"
          value={exportJson}
          readOnly
        />
      </div>
    </div>
  );
}
