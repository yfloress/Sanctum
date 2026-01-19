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

export default function Generator() {
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
          throw new Error("Invalid JSON structure.");
        }
        if (!isSupportedVersion(parsed.version)) {
          throw new Error("Unsupported version. Expected version 1.0.");
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
        const message = err instanceof Error ? err.message : "Failed to parse JSON.";
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
      setError("Paste a JSON payload before loading.");
      return;
    }
    try {
      const parsed = JSON.parse(rawInput);
      if (!isSupportedVersion(parsed.version)) {
        throw new Error("Unsupported version. Expected version 1.0.");
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
      const message = err instanceof Error ? err.message : "Failed to parse JSON.";
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
      setError("Transaction requires date, account, amount, and currency.");
      return;
    }
    if (txForm.transaction_type !== "transfer" && !txForm.category) {
      setError("Category is required unless this is a transfer.");
      return;
    }
    if (txForm.transaction_type === "transfer" && !txForm.transfer_to_account) {
      setError("Transfer requires a destination account.");
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
      setError("Habit name and date are required.");
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
      setError("Crypto entry requires date, wallet, symbol, and amount.");
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
              JSON v1 Generator
            </p>
            <h3 className="font-display mt-3 text-2xl font-semibold text-foreground">
              Build a trip-safe log
            </h3>
            <p className="mt-3 text-sm text-muted-foreground max-w-lg">
              This generator matches Sanctum&apos;s import schema. Account, habit, wallet, and category
              names must already exist in your vault for the import to succeed.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button variant="secondary" onClick={handleLoadClick}>
              Load JSON
            </Button>
            <Button variant="outline" onClick={handleReset}>
              Clear
            </Button>
            <Button onClick={handleDownload}>Download JSON</Button>
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
          <Badge variant="outline">Step 1: Load</Badge>
          <Badge variant="outline">Step 2: Add Entries</Badge>
          <Badge variant="outline">Step 3: Export</Badge>
        </div>

        <div className="mt-6 grid gap-6 lg:grid-cols-[1.1fr_0.9fr]">
          <div className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-3">
              <div className="panel-gradient rounded-xl border border-border p-4">
                <p className="text-xs text-muted-foreground">Transactions</p>
                <p className="text-2xl font-semibold text-foreground">{transactions.length}</p>
              </div>
              <div className="panel-gradient rounded-xl border border-border p-4">
                <p className="text-xs text-muted-foreground">Habit Logs</p>
                <p className="text-2xl font-semibold text-foreground">{habits.length}</p>
              </div>
              <div className="panel-gradient rounded-xl border border-border p-4">
                <p className="text-xs text-muted-foreground">Crypto Entries</p>
                <p className="text-2xl font-semibold text-foreground">{cryptoTx.length}</p>
              </div>
            </div>

            {loadedFile && (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Badge variant="outline">Loaded</Badge>
                <span>{loadedFile}</span>
              </div>
            )}

            {error && (
              <div className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive-foreground">
                {error}
              </div>
            )}

            <div className="rounded-2xl border border-border bg-card/70 p-4 text-xs text-muted-foreground">
              <p className="font-semibold text-foreground">Import Tips</p>
              <ul className="mt-2 space-y-1">
                <li>Dates must be in YYYY-MM-DD format.</li>
                <li>Currency is a 3-letter code (USD, CLP).</li>
                <li>Transfers require a destination account.</li>
              </ul>
            </div>
          </div>

          <div className="panel-gradient rounded-2xl border border-border p-5">
            <h4 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
              Paste or Load JSON
            </h4>
            <p className="mt-2 text-sm text-muted-foreground">
              Use this if you only have access to text notes. The payload stays local.
            </p>
            <Textarea
              className="mt-4 min-h-[180px] font-mono text-xs"
              value={rawInput}
              onChange={(event) => setRawInput(event.target.value)}
              placeholder="Paste a sanctum_export.json payload..."
            />
            <div className="mt-4 flex justify-end">
              <Button variant="outline" onClick={handleLoadPaste}>
                Load From Paste
              </Button>
            </div>
            <p className="mt-4 text-xs text-muted-foreground">
              Privacy: data never leaves your browser. Works offline after first load.
            </p>
          </div>
        </div>
      </div>

      <div className="panel-gradient-strong rounded-3xl border border-border p-6">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-muted-foreground">
              Add Entries
            </p>
            <h4 className="mt-1 text-xl font-semibold text-foreground">Build your log</h4>
          </div>
          <div className="flex flex-wrap gap-2">
            {[
              { id: "transactions", label: "Transactions" },
              { id: "habits", label: "Habits" },
              { id: "crypto", label: "Crypto" },
            ].map((tab) => (
              <button
                key={tab.id}
                className={cn(
                  "rounded-full border px-3 py-1 text-xs font-semibold transition",
                  activeTab === tab.id
                    ? "border-primary text-primary"
                    : "border-border text-muted-foreground hover:text-foreground"
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
                  placeholder="Account"
                  value={txForm.account}
                  onChange={(event) =>
                    setTxForm({ ...txForm, account: event.target.value })
                  }
                />
                <Input
                  placeholder="Currency (USD)"
                  value={txForm.currency}
                  onChange={(event) =>
                    setTxForm({ ...txForm, currency: event.target.value })
                  }
                />
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
                  <option value="expense">Expense</option>
                  <option value="income">Income</option>
                  <option value="transfer">Transfer</option>
                </select>
                <Input
                  type="number"
                  step="0.01"
                  placeholder="Amount"
                  value={txForm.amount}
                  onChange={(event) =>
                    setTxForm({ ...txForm, amount: event.target.value })
                  }
                />
              </div>
              <Input
                placeholder="Category"
                value={txForm.category}
                onChange={(event) => setTxForm({ ...txForm, category: event.target.value })}
                disabled={txForm.transaction_type === "transfer"}
              />
              {txForm.transaction_type === "transfer" && (
                <Input
                  placeholder="Transfer to account"
                  value={txForm.transfer_to_account}
                  onChange={(event) =>
                    setTxForm({ ...txForm, transfer_to_account: event.target.value })
                  }
                />
              )}
              <Input
                placeholder="Description"
                value={txForm.description}
                onChange={(event) => setTxForm({ ...txForm, description: event.target.value })}
              />
              <Button onClick={addTransaction}>Add Transaction</Button>
            </div>

            <div className="panel-gradient rounded-2xl border border-border p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                Recent Transactions
              </p>
              <div className="mt-4 space-y-3">
                {transactions.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    No transactions yet. Add the first one.
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
                          className="text-xs text-muted-foreground hover:text-foreground"
                          onClick={() =>
                            setTransactions((prev) => prev.filter((item) => item.id !== tx.id))
                          }
                        >
                          Remove
                        </button>
                      </div>
                      <p className="text-xs text-muted-foreground">
                        {tx.date} · {tx.transaction_type}
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
                placeholder="Habit name"
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
                Completed
              </label>
              <Button onClick={addHabit}>Add Habit Log</Button>
            </div>

            <div className="panel-gradient rounded-2xl border border-border p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                Habit Logs
              </p>
              <div className="mt-4 space-y-3">
                {habits.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    No habit logs yet. Capture today’s progress.
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
                        className="text-xs text-muted-foreground hover:text-foreground"
                        onClick={() => setHabits((prev) => prev.filter((item) => item.id !== log.id))}
                      >
                        Remove
                      </button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {log.date} · {log.completed ? "Completed" : "Skipped"}
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
                  placeholder="Wallet"
                  value={cryptoForm.wallet}
                  onChange={(event) =>
                    setCryptoForm({ ...cryptoForm, wallet: event.target.value })
                  }
                />
                <Input
                  placeholder="Symbol (BTC)"
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
                  <option value="buy">Buy</option>
                  <option value="sell">Sell</option>
                  <option value="transfer_in">Transfer In</option>
                  <option value="transfer_out">Transfer Out</option>
                </select>
                <Input
                  type="number"
                  step="0.00000001"
                  placeholder="Amount"
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
                  placeholder="Price per coin (optional)"
                  value={cryptoForm.price_per_coin}
                  onChange={(event) =>
                    setCryptoForm({ ...cryptoForm, price_per_coin: event.target.value })
                  }
                />
                <Input
                  type="number"
                  step="0.01"
                  placeholder="Fee (optional)"
                  value={cryptoForm.fee}
                  onChange={(event) => setCryptoForm({ ...cryptoForm, fee: event.target.value })}
                />
              </div>
              <Input
                placeholder="Notes (optional)"
                value={cryptoForm.notes}
                onChange={(event) => setCryptoForm({ ...cryptoForm, notes: event.target.value })}
              />
              <Button onClick={addCrypto}>Add Crypto Entry</Button>
            </div>

            <div className="panel-gradient rounded-2xl border border-border p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                Crypto Entries
              </p>
              <div className="mt-4 space-y-3">
                {cryptoTx.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    No crypto entries yet. Add wallet activity here.
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
                        className="text-xs text-muted-foreground hover:text-foreground"
                        onClick={() =>
                          setCryptoTx((prev) => prev.filter((item) => item.id !== tx.id))
                        }
                      >
                        Remove
                      </button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {tx.date} · {tx.transaction_type.replace("_", " ")}
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
        <div className="flex items-center justify-between">
          <h4 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
            Export Preview
          </h4>
          <Badge variant="outline">JSON v1</Badge>
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
