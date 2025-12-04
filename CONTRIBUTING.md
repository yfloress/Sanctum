# CONTRIBUTING.md

> Technical guide for Sanctum contributors.
>
> **[Leer en Espanol](CONTRIBUTING_ES.md)**

This document establishes engineering rules to keep the code clean, predictable,
and extensible. It's not bureaucracy, it's architecture.

---

## 1. Quick Start

Before contributing, make sure you have the development environment configured.
See detailed instructions in **[INSTALL.md](INSTALL.md)**.

### Command Cheatsheet

| Action                | Command                         |
| :-------------------- | :------------------------------ |
| **Start Environment** | `nix develop` (or manual setup) |
| **Run App (Dev)**     | `cargo tauri dev`               |
| **Linting (Rust)**    | `cargo clippy`                  |
| **Linting (TS)**      | `deno task check`               |
| **Format Code**       | `cargo fmt && deno fmt`         |
| **Build Release**     | `cargo tauri build`             |

---

## 2. Project Philosophy (Non-Functional Requirements)

Any code change must respect these three pillars:

1. **Local-First:** The app must be 100% functional without internet. Data lives
   on the user's device.
2. **Privacy by Design:** No telemetry, no analytics, no "ping" to external
   servers (except CoinGecko on explicit user demand).
3. **Zero Hidden Dependencies:** Do not use libraries that require proprietary
   servers (e.g., Firebase).

---

## 3. Project Architecture

Sanctum follows a layered architecture with strict separation of concerns:

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (React)                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │  Components │◄───│   Stores    │◄───│  Tauri IPC Bridge   │  │
│  │  (View)     │    │  (Zustand)  │    │  invoke()           │  │
│  └─────────────┘    └─────────────┘    └──────────┬──────────┘  │
└──────────────────────────────────────────────────│──────────────┘
                                                   │
┌──────────────────────────────────────────────────│──────────────┐
│                        BACKEND (Rust)            │              │
│  ┌─────────────┐    ┌─────────────┐    ┌────────▼──────────┐   │
│  │   db.rs     │◄───│  commands.rs│◄───│  #[tauri::command] │   │
│  │  (Model)    │    │ (Controller)│    │  (Entry Points)    │   │
│  └──────┬──────┘    └─────────────┘    └───────────────────┘   │
│         │                                                       │
│  ┌──────▼──────┐                                                │
│  │   SQLite    │                                                │
│  │  (SQLCipher)│                                                │
│  └─────────────┘                                                │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **SQLite (SQLCipher)**: Encrypted storage. Single source of truth.
2. **db.rs**: Data access layer. CRUD, migrations, queries.
3. **commands.rs**: Input validation, sanitization, orchestration.
4. **Tauri IPC**: Bridge between Rust and JavaScript via `invoke()`.
5. **Zustand Stores**: Frontend global state. Contains all business logic.
6. **React Components**: Pure rendering. No logic, only presentation.

### MVC Mapping

| Layer      | Location                    | Responsibility                         |
| :--------- | :-------------------------- | :------------------------------------- |
| Model      | `src-tauri/src/db.rs`       | Persistence, SQL queries, migrations   |
| Controller | `src-tauri/src/commands.rs` | Validation, sanitization, coordination |
| ViewModel  | `src/stores/*.ts`           | State, business logic, IPC calls       |
| View       | `src/features/**/*.tsx`     | Rendering, UI events                   |

---

## 4. Code Standards

### Frontend (TypeScript/React)

**Cardinal Rule:** Business logic goes in Stores, NOT in components.

```typescript
// WRONG: Logic in component
function TransactionsView() {
  const [transactions, setTransactions] = useState([]);

  useEffect(() => {
    invoke("get_transactions").then(setTransactions); // NO
  }, []);

  const handleDelete = async (id: string) => {
    await invoke("delete_transaction", { id }); // NO
    setTransactions((prev) => prev.filter((t) => t.id !== id));
  };
}

// CORRECT: Component consumes the Store
function TransactionsView() {
  const transactions = useTransactions(); // Read only
  const { deleteTransaction } = useFinancialStore(); // Actions only

  return (
    <ul>
      {transactions.map((t) => (
        <li
          key={t.id}
          onClick={() => deleteTransaction(t.id)}
        >
          {t.description}
        </li>
      ))}
    </ul>
  );
}
```

**Other rules:**

- Use specific selectors to avoid unnecessary re-renders
- Do not use `persist` middleware in Zustand (sensitive data)
- `invoke()` calls only inside Stores
- Components in `features/` are complete views, in `components/` are reusable
- For large views, extract sub-components into `features/<name>/components/`
- Extract modals into `features/<name>/modals/` for better organization

### Backend (Rust)

**Cardinal Rule:** The database is the single source of truth. Never cache
derived state.

```rust
// WRONG: Storing derived state
struct AppState {
    balance: i64,  // NO - will desync
}

// CORRECT: Always calculate from DB
pub fn get_balance(conn: &Connection) -> Result<BalanceSummary, DbError> {
    // Always query the DB
}
```

**Mandatory security:**

```rust
// Passwords: ALWAYS SecretString
use secrecy::{SecretString, ExposeSecret};

fn open_db(password: SecretString) -> Result<(), DbError> {
    let key = password.expose_secret();  // Only at point of use
    // ...
}

// User input: ALWAYS validate
fn add_transaction(amount: i64, category: &str) -> Result<(), DbError> {
    validate_amount(amount)?;
    let safe_category = sanitize_string(category, MAX_CATEGORY_LENGTH)?;
    // ...
}
```

**Other rules:**

- Prepared statements for all queries (anti SQL injection)
- Validation in `commands.rs`, persistence in `db.rs`
- Generic errors to user, detailed in internal logs
- `PRAGMA foreign_keys = ON` always active

### Tauri Window Management

**Cardinal Rule:** The main window starts hidden and is shown only after React
hydrates.

This prevents the "white flash" on startup. The pattern is implemented in:

1. `tauri.conf.json`: Window configured with `"visible": false`
2. `main.tsx`: Calls `getCurrentWindow().show()` via `onReady` callback
3. `App.tsx`: Triggers `onReady` in `useLayoutEffect` after first render

```typescript
// main.tsx
import { getCurrentWindow } from "@tauri-apps/api/window";

const showWindow = () => {
  getCurrentWindow().show().catch(console.error);
};

// App.tsx
useLayoutEffect(() => {
  if (onReady) {
    onReady();
  }
}, [onReady]);
```

**Required permission** in `src-tauri/capabilities/default.json`:

```json
"permissions": ["core:default", "core:window:allow-show", "opener:default"]
```

**Do NOT:**

- Set `"visible": true` in `tauri.conf.json`
- Call `show()` before React has mounted
- Block the main thread with synchronous operations during startup

---

## 5. Design Patterns Used

### Observer Pattern (Zustand)

Stores implement the Observer pattern. Components subscribe to state slices and
React re-renders automatically when they change.

```typescript
// The store is the Subject
const useFinancialStore = create<FinancialState>((set, get) => ({
  transactions: [],
  addTransaction: async (data) => {
    await invoke("add_transaction", data);
    set({ transactions: await invoke("get_transactions") });
  },
}));

// Components are Observers
function TransactionList() {
  // Subscribes only to `transactions`
  const transactions = useFinancialStore(state => state.transactions);
  return <>{transactions.map(...)}</>;
}
```

### Command Pattern (Tauri IPC)

Each backend operation is exposed as a discrete command. The frontend doesn't
know the implementation, only the contract.

```rust
// Backend: Define the command
#[tauri::command]
pub fn add_transaction(amount: i64, category: String) -> Result<String, String> {
    // Encapsulated implementation
}

// Frontend: Invoke the command
await invoke("add_transaction", { amount: 1000, category: "Food" });
```

### Repository Pattern (db.rs)

`db.rs` acts as a repository. It abstracts SQLite behind domain functions.

```rust
// The rest of the code doesn't know we use SQL
pub fn get_transactions(conn: &Connection) -> Result<Vec<Transaction>, DbError>;
pub fn add_wallet(conn: &Connection, wallet: &CryptoWallet) -> Result<(), DbError>;
```

---

## 6. Directory Structure

```
src/
├── features/           # Views by functionality (one folder = one app section)
│   ├── auth/           #   LoginScreen.tsx
│   ├── dashboard/      #   Dashboard.tsx
│   ├── transactions/   #   TransactionsView.tsx
│   ├── crypto/         #   Main crypto feature (see expanded structure below)
│   │   ├── CryptoView.tsx      # Layout orchestrator (~100 lines)
│   │   ├── components/         # UI sub-components
│   │   │   ├── index.ts        #   Barrel export
│   │   │   ├── AssetTable.tsx  #   Portfolio grid
│   │   │   ├── CryptoHeader.tsx
│   │   │   ├── PortfolioSummary.tsx
│   │   │   ├── SubTabs.tsx
│   │   │   ├── WalletDetail.tsx
│   │   │   ├── WalletList.tsx
│   │   │   └── Watchlist.tsx
│   │   └── modals/             # Modal components
│   │       ├── index.ts        #   Barrel export
│   │       ├── AddWalletModal.tsx
│   │       ├── AddTransactionModal.tsx
│   │       ├── TransferModal.tsx
│   │       ├── SwapModal.tsx
│   │       ├── AddCryptoModal.tsx
│   │       ├── DeleteConfirmModal.tsx
│   │       └── LegacyHoldingModals.tsx
│   └── habits/         #   HabitsView.tsx
│
├── stores/             # Global state (Zustand)
│   ├── authStore.ts    #   Authentication, vault, kill switch
│   ├── financialStore.ts   # FIAT transactions
│   ├── cryptoStore.ts  #   Wallets, portfolio, prices
│   ├── habitStore.ts   #   Habits and logs
│   └── index.ts        #   Re-exports
│
├── components/         # Reusable components (no business logic)
│   ├── layout/         #   Sidebar, Header
│   ├── modals/         #   DeleteConfirmModal
│   └── ui/             #   Toast, Button, Input
│
├── types/              # TypeScript interfaces
│   └── index.ts        #   Transaction, CryptoWallet, etc.
│
└── utils/              # Pure functions (no side effects)
    └── index.ts        #   formatAmount, formatDate, etc.

src-tauri/src/
├── lib.rs              # Entry point, command registration
├── main.rs             # Tauri bootstrap
├── commands.rs         # IPC commands (validation, orchestration)
├── db.rs               # Data access, migrations, SQLCipher
├── models.rs           # Domain structs and enums
├── crypto.rs           # HTTP client for CoinGecko
└── security_log.rs     # Security event logging
```

### Where each thing goes

| You need...                    | Location                                    |
| :----------------------------- | :------------------------------------------ |
| New screen/section             | `src/features/new-feature/`                 |
| Feature-specific sub-component | `src/features/<feature>/components/`        |
| Feature-specific modal         | `src/features/<feature>/modals/`            |
| Shared global state            | `src/stores/newStore.ts`                    |
| Reusable component (app-wide)  | `src/components/category/`                  |
| Type/Interface                 | `src/types/index.ts`                        |
| Pure helper function           | `src/utils/index.ts`                        |
| New SQL table                  | `src-tauri/src/db.rs` (migration)           |
| New IPC endpoint               | `src-tauri/src/commands.rs`                 |
| Data struct                    | `src-tauri/src/models.rs`                   |

> **Note:** The project does NOT use a `hooks/` folder. All state logic is in
> Zustand stores.

### Component Extraction Pattern

When a view file exceeds ~300 lines, extract sub-components following this pattern:

```
features/your-feature/
├── YourFeatureView.tsx     # Layout orchestrator (imports children)
├── components/
│   ├── index.ts            # Barrel export: export { A } from "./A.tsx"
│   ├── Header.tsx          # Each connects to store directly
│   └── DataTable.tsx
└── modals/
    ├── index.ts            # Barrel export
    ├── AddItemModal.tsx    # Self-managing: returns null if !showModal
    └── DeleteConfirmModal.tsx
```

**Key principles:**
- Parent view is a pure layout orchestrator (<100-200 lines)
- Child components connect directly to Zustand (no prop drilling)
- Modals can be "self-managing" (check visibility internally) or conditionally rendered
- Use barrel files (`index.ts`) for clean imports

---

## 7. Workflow for New Features

To add a new feature (example: Financial Goals module):

### Step 1: Backend - Data Model

Edit `src-tauri/src/db.rs`:

```rust
// 1. Add the migration
fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    // ... existing migrations ...
    
    // New migration
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            target_amount INTEGER NOT NULL,
            current_amount INTEGER NOT NULL DEFAULT 0,
            deadline TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    
    Ok(())
}

// 2. Add CRUD functions
pub fn add_goal(conn: &Connection, goal: &Goal) -> Result<(), DbError> { ... }
pub fn get_goals(conn: &Connection) -> Result<Vec<Goal>, DbError> { ... }
pub fn update_goal(conn: &Connection, id: &str, amount: i64) -> Result<(), DbError> { ... }
pub fn delete_goal(conn: &Connection, id: &str) -> Result<(), DbError> { ... }
```

### Step 2: Backend - Domain Model

Edit `src-tauri/src/models.rs`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub target_amount: i64,
    pub current_amount: i64,
    pub deadline: Option<String>,
    pub created_at: String,
}
```

### Step 3: Backend - IPC Commands

Edit `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn add_goal(name: String, target_amount: i64, deadline: Option<String>) -> Result<String, String> {
    // Validation
    let name = sanitize_string(&name, MAX_NAME_LENGTH)?;
    validate_amount(target_amount)?;
    
    // Persistence
    let goal = Goal { id: uuid(), name, target_amount, ... };
    db::add_goal(&get_connection()?, &goal)?;
    
    Ok(goal.id)
}

// ... other commands ...
```

Register in `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::add_goal,
    commands::get_goals,
    commands::update_goal,
    commands::delete_goal,
])
```

### Step 4: Frontend - Types

Edit `src/types/index.ts`:

```typescript
export interface Goal {
  id: string;
  name: string;
  target_amount: number;
  current_amount: number;
  deadline: string | null;
  created_at: string;
}
```

### Step 5: Frontend - Store

Create `src/stores/goalStore.ts`:

```typescript
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Goal } from "../types";

interface GoalState {
  goals: Goal[];
  isLoading: boolean;
  error: string | null;
  successMessage: string | null;

  loadGoals: () => Promise<void>;
  addGoal: (name: string, targetAmount: number) => Promise<boolean>;
  // ... other actions ...

  // Messages
  setError: (error: string | null) => void;
  setSuccess: (message: string | null) => void;
  clearMessages: () => void;

  // Security: RAM Clear
  reset: () => void;
}

export const useGoalStore = create<GoalState>((set, get) => ({
  goals: [],
  isLoading: false,
  error: null,
  successMessage: null,

  loadGoals: async () => {
    set({ isLoading: true });
    try {
      const goals = await invoke<Goal[]>("get_goals");
      set({ goals, error: null });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  addGoal: async (name, targetAmount) => {
    set({ isLoading: true, error: null });
    try {
      await invoke("add_goal", { name, targetAmount });
      await get().loadGoals();
      set({ successMessage: "Goal created successfully" });
      return true;
    } catch (e) {
      set({ error: String(e) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  setError: (error) => set({ error }),
  setSuccess: (message) => set({ successMessage: message }),
  clearMessages: () => set({ error: null, successMessage: null }),

  // IMPORTANT: Clear sensitive data from RAM when vault closes
  reset: () =>
    set({
      goals: [],
      isLoading: false,
      error: null,
      successMessage: null,
    }),
}));

// Optimized selectors (avoid unnecessary re-renders)
export const useGoals = () => useGoalStore((s) => s.goals);
export const useGoalLoading = () => useGoalStore((s) => s.isLoading);
export const useGoalError = () => useGoalStore((s) => s.error);
export const useGoalSuccess = () => useGoalStore((s) => s.successMessage);
```

### Step 6: Frontend - View

Create `src/features/goals/GoalsView.tsx`:

```typescript
import { useEffect } from "react";
import { useGoalLoading, useGoals, useGoalStore } from "../../stores/goalStore";

export function GoalsView() {
  // Specific selectors (optimize re-renders)
  const goals = useGoals();
  const isLoading = useGoalLoading();

  // Store actions
  const loadGoals = useGoalStore((s) => s.loadGoals);
  const addGoal = useGoalStore((s) => s.addGoal);

  useEffect(() => {
    loadGoals();
  }, [loadGoals]);

  if (isLoading) {
    return <div className="loader">Loading goals...</div>;
  }

  return (
    <div className="goals-view">
      <h1>Financial Goals</h1>
      <div className="goals-grid">
        {goals.map((goal) => <GoalCard key={goal.id} goal={goal} />)}
      </div>
    </div>
  );
}
```

### Step 7: Integration

1. Add the tab in `src/App.tsx`:
   - Import `GoalsView` from `./features/goals/GoalsView`
   - Add the case in the conditional tab render

2. Add the item in `src/components/layout/Sidebar.tsx`:
   - Add the button with the corresponding icon and label

3. Add the store to the kill switch in `src/stores/authStore.ts`:
   ```typescript
   // In _clearAllStores:
   useGoalStore.getState().reset();
   ```

4. Export the store in `src/stores/index.ts`:
   ```typescript
   export {
     useGoalLoading,
     useGoals,
     useGoalStore,
     // ...
   } from "./goalStore";
   ```

5. Add the tab type in `src/types/index.ts`:
   ```typescript
   export type TabType = "dashboard" | "transactions" | ... | "goals";
   ```

---

## 8. Pull Request Checklist

Before submitting a PR, verify:

**Backend (Rust):**

- [ ] `cargo clippy` passes without warnings
- [ ] `cargo test` passes (if there are tests)
- [ ] Sensitive data uses `SecretString`
- [ ] New commands have input validation in `commands.rs`
- [ ] New commands are registered in `lib.rs`

**Frontend (TypeScript):**

- [ ] `deno task check` passes (TypeScript)
- [ ] No `console.log` in final code (except in development)
- [ ] New stores have `reset()` method for the kill switch
- [ ] Store exported in `stores/index.ts`
- [ ] Specific selectors to avoid re-renders

**General:**

- [ ] No hardcoded secrets
- [ ] Documentation in `ARCHITECT.md` is updated if structure changes
- [ ] Types synchronized between Rust (`models.rs`) and TypeScript
      (`types/index.ts`)

---

## 9. Commit Conventions

We follow Conventional Commits:

```
<type>(<scope>): <description>

feat(crypto): add swap transaction support
fix(auth): handle rate limit edge case
refactor(stores): migrate from hooks to Zustand
docs(contributing): add development workflow
```

Valid types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`,
`security`

---

## 10. Contact

If you have questions about the architecture before starting to code, open an
Issue with the `question` tag. It's better to ask than to rewrite.
