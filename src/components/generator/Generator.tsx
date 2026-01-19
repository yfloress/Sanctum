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
    tips: {
      title: "Import Tips",
      date: "Dates must be in YYYY-MM-DD format.",
      currency: "Currency is USD or CLP.",
      transfer: "Transfers require a destination account.",
    },
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
      transactionRequired: "Transaction requires date, account, amount, and currency.",
      categoryRequired: "Category is required unless this is a transfer.",
      transferRequired: "Transfer requires a destination account.",
      habitRequired: "Habit name and date are required.",
      cryptoRequired: "Crypto entry requires date, wallet, symbol, and amount.",
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
    tips: {
      title: "Consejos de importación",
      date: "Las fechas deben ser YYYY-MM-DD.",
      currency: "La moneda es USD o CLP.",
      transfer: "Las transferencias requieren cuenta destino.",
    },
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
      transactionRequired: "La transacción requiere fecha, cuenta, monto y moneda.",
      categoryRequired: "La categoría es obligatoria salvo que sea una transferencia.",
      transferRequired: "La transferencia requiere una cuenta destino.",
      habitRequired: "El nombre del hábito y la fecha son obligatorios.",
      cryptoRequired: "El movimiento cripto requiere fecha, wallet, símbolo y monto.",
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
  const [habitForm, setHabitForm] = React.useState(defaultHabit);
  const [cryptoForm, setCryptoForm] = React.useState(defaultCrypto);

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
  }

  function addTransaction() {
    setError(null);
    if (!txForm.date || !txForm.account || !txForm.amount || !txForm.currency) {
      setError(copy.errors.transactionRequired);
      return;
    }
    if (txForm.transaction_type !== "transfer" && !txForm.category) {
      setError(copy.errors.categoryRequired);
      return;
    }
    if (txForm.transaction_type === "transfer" && !txForm.transfer_to_account) {
      setError(copy.errors.transferRequired);
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
  }

  function addHabit() {
    setError(null);
    if (!habitForm.habit || !habitForm.date) {
      setError(copy.errors.habitRequired);
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
  }

  function addCrypto() {
    setError(null);
    if (!cryptoForm.date || !cryptoForm.wallet || !cryptoForm.symbol || !cryptoForm.amount) {
      setError(copy.errors.cryptoRequired);
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

            <div className="rounded-2xl border border-border bg-card/70 p-4 text-xs text-muted-foreground">
              <p className="font-semibold text-foreground">{copy.tips.title}</p>
              <ul className="mt-2 space-y-1">
                <li>{copy.tips.date}</li>
                <li>{copy.tips.currency}</li>
                <li>{copy.tips.transfer}</li>
              </ul>
            </div>
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
                onChange={(event) => setTxForm({ ...txForm, date: event.target.value })}
              />
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  placeholder={copy.transactions.form.account}
                  value={txForm.account}
                  onChange={(event) =>
                    setTxForm({ ...txForm, account: event.target.value })
                  }
                />
                <select
                  className="h-10 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground"
                  value={txForm.currency}
                  onChange={(event) =>
                    setTxForm({ ...txForm, currency: event.target.value })
                  }
                >
                  <option value="USD">USD</option>
                  <option value="CLP">CLP</option>
                </select>
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <select
                  className="h-10 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground"
                  value={txForm.transaction_type}
                  onChange={(event) =>
                    setTxForm({
                      ...txForm,
                      transaction_type: event.target.value as TransactionType,
                    })
                  }
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
                  onChange={(event) =>
                    setTxForm({ ...txForm, amount: event.target.value })
                  }
                />
              </div>
              <Input
                placeholder={copy.transactions.form.category}
                value={txForm.category}
                onChange={(event) => setTxForm({ ...txForm, category: event.target.value })}
                disabled={txForm.transaction_type === "transfer"}
              />
              {txForm.transaction_type === "transfer" && (
                <Input
                  placeholder={copy.transactions.form.transferTo}
                  value={txForm.transfer_to_account}
                  onChange={(event) =>
                    setTxForm({ ...txForm, transfer_to_account: event.target.value })
                  }
                />
              )}
              <Input
                placeholder={copy.transactions.form.description}
                value={txForm.description}
                onChange={(event) => setTxForm({ ...txForm, description: event.target.value })}
              />
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
                onChange={(event) => setHabitForm({ ...habitForm, date: event.target.value })}
              />
              <Input
                placeholder={copy.habits.form.name}
                value={habitForm.habit}
                onChange={(event) => setHabitForm({ ...habitForm, habit: event.target.value })}
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
                onChange={(event) => setCryptoForm({ ...cryptoForm, date: event.target.value })}
              />
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  placeholder={copy.crypto.form.wallet}
                  value={cryptoForm.wallet}
                  onChange={(event) =>
                    setCryptoForm({ ...cryptoForm, wallet: event.target.value })
                  }
                />
                <Input
                  placeholder={copy.crypto.form.symbol}
                  value={cryptoForm.symbol}
                  onChange={(event) =>
                    setCryptoForm({ ...cryptoForm, symbol: event.target.value })
                  }
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
                  onChange={(event) =>
                    setCryptoForm({ ...cryptoForm, amount: event.target.value })
                  }
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
