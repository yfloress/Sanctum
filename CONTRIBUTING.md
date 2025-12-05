# CONTRIBUTING.md

> Technical guide for Sanctum contributors.
>
> **[Leer en Español](CONTRIBUTING_ES.md)**

---

## 1. Quick Start

```bash
# Enter development environment
nix develop  # or direnv allow

# Run in development
cargo tauri dev

# Linting
cargo clippy && deno task check

# Format
cargo fmt && deno fmt

# Production build
cargo tauri build
```

---

## 2. Project Philosophy

1. **Local-First:** 100% functional offline. Data lives on the device.
2. **Privacy by Design:** No telemetry or external servers (except CoinGecko on demand).
3. **Zero Hidden Dependencies:** No libraries requiring proprietary servers.

---

## 3. Architecture

### Currency System
- **Base currency:** USD (all net worth calculations)
- **Supported:** USD, CLP (Chilean Peso)
- **Exchange rate:** Fetched from CoinGecko, cached in SQLCipher for offline use

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (React)                         │
│  Components (View) ◄── Stores (Zustand) ◄── Tauri IPC Bridge    │
└──────────────────────────────────────────────────────────────────┘
                                   │
┌──────────────────────────────────│───────────────────────────────┐
│                        BACKEND (Rust)                            │
│  db.rs (Model) ◄── commands.rs (Controller) ◄── #[tauri::command]│
│       │                                                          │
│  SQLite + SQLCipher (Encrypted)                                  │
└──────────────────────────────────────────────────────────────────┘
```

### MVC Mapping

| Layer      | Location              | Responsibility                         |
| :--------- | :-------------------- | :------------------------------------- |
| Model      | `src-tauri/src/db.rs` | Persistence, SQL, migrations           |
| Controller | `commands.rs`         | Validation, sanitization, coordination |
| ViewModel  | `src/stores/*.ts`     | State, business logic, IPC             |
| View       | `src/features/**`     | Pure rendering, UI events              |

---

## 4. Directory Structure

```
src/
├── features/           # Views by functionality
│   ├── accounts/       #   FIAT accounts
│   ├── analytics/      #   Reports and charts
│   ├── auth/           #   Login, vault creation
│   ├── crypto/         #   Crypto portfolio, wallets
│   ├── dashboard/      #   Main view
│   ├── habits/         #   Habit tracker
│   └── transactions/   #   FIAT transactions
│
├── stores/             # Global state (Zustand)
│   ├── accountStore.ts #   FIAT accounts and balances
│   ├── authStore.ts    #   Authentication, vault, kill switch
│   ├── cryptoStore.ts  #   Wallets, portfolio, prices
│   ├── financialStore.ts   # FIAT transactions
│   ├── habitStore.ts   #   Habits and logs
│   └── toastStore.ts   #   UI notifications
│
├── components/         # Reusable components
├── types/              # TypeScript interfaces
└── utils/              # Pure functions

src-tauri/src/
├── lib.rs              # Entry point, command registration
├── commands.rs         # IPC commands (validation)
├── db.rs               # Data access, migrations, SQLCipher
├── models.rs           # Domain structs
├── crypto.rs           # HTTP client for CoinGecko + exchange rates
└── security_log.rs     # Security logging
```

---

## 5. Code Standards

### Frontend

**Cardinal Rule:** Business logic goes in Stores, NOT in components.

```typescript
// CORRECT
function TransactionsView() {
  const transactions = useTransactions(); // Only reads from store
  const { deleteTransaction } = useFinancialStore();
  // Component only renders
}

// ❌ INCORRECT
function TransactionsView() {
  const [transactions, setTransactions] = useState([]);
  useEffect(() => {
    invoke("get_transactions").then(setTransactions); // NO
  }, []);
}
```

**Rules:**
- Use specific selectors to avoid re-renders
- DO NOT use `persist` middleware (sensitive data)
- `invoke()` calls only inside Stores
- Every store must have a `reset()` method for the kill switch

### Backend

**Cardinal Rule:** The DB is the single source of truth. Never cache state.

```rust
// Passwords: ALWAYS SecretString
fn open_db(password: SecretString) -> Result<(), DbError> {
    let key = password.expose_secret();
}

// User input: ALWAYS validate
fn add_transaction(amount: i64, category: &str) -> Result<(), DbError> {
    validate_amount(amount)?;
    let safe = sanitize_string(category, MAX_LENGTH)?;
}
```

**Rules:**
- Prepared statements for all queries
- Validation in `commands.rs`, persistence in `db.rs`
- Generic errors to user, detailed in internal logs

---

## 6. SQLCipher Security

The database uses SQLCipher with hardened parameters:

- **Cipher:** AES-256-CBC
- **KDF:** PBKDF2-HMAC-SHA512 with 600,000 iterations
- **HMAC:** HMAC-SHA512
- **Memory Security:** Enabled

**IMPORTANT:** Cipher parameters must be applied on EVERY database open,
not just on creation. See `apply_sqlcipher_hardening()` in `db.rs`.

---

## 7. Adding a New Feature

1. **Backend - Migration:** `db.rs` → `run_migrations()`
2. **Backend - Model:** `models.rs` → struct with Serialize/Deserialize
3. **Backend - Command:** `commands.rs` → validation + db call
4. **Backend - Register:** `lib.rs` → add to `invoke_handler!`
5. **Frontend - Types:** `types/index.ts` → TypeScript interface
6. **Frontend - Store:** `stores/` → new store with `reset()`
7. **Frontend - View:** `features/` → component consuming store
8. **Integration:** Add to Sidebar, App.tsx, and kill switch in authStore

---

## 8. PR Checklist

**Backend:**
- [ ] `cargo clippy` no warnings
- [ ] Sensitive data uses `SecretString`
- [ ] Commands validated in `commands.rs`
- [ ] Commands registered in `lib.rs`

**Frontend:**
- [ ] `deno task check` passes
- [ ] No `console.log` (only `console.error` for errors)
- [ ] Store has `reset()` for kill switch
- [ ] Store exported in `stores/index.ts`

**General:**
- [ ] No hardcoded secrets
- [ ] Types synchronized between Rust and TypeScript

---

## 9. Commits

We follow Conventional Commits:

```
feat(crypto): add swap transaction support
fix(auth): handle rate limit edge case
refactor(stores): migrate from hooks to Zustand
docs: update contributing guide
```

---

## 10. Contact

Questions before coding → open an Issue with tag `question`.
