# Sanctum Architecture Migration Plan

## Current State Analysis

### Problem: Monolithic Files
| File | Lines | Issue |
|------|-------|-------|
| `main.rs` | 4224 | UI + logic + callbacks all mixed |
| `db.rs` | 2633 | All DB operations for all domains |
| `controller.rs` | 1706 | God object orchestrating everything |
| `models.rs` | 573 | All models in one file |
| `services/crypto.rs` | 1959 | Too large, mixed concerns |

### Domains Identified
1. **Finance** - Accounts, transactions, categories
2. **Crypto** - Wallets, transactions, API, portfolio
3. **Habits** - Habits, logs, streaks

---

## Target Architecture

```
src/
├── main.rs                    # ~100 lines: init + UI bootstrap only
├── lib.rs                     # Re-exports public API
├── app.rs                     # AppController (slim orchestrator)
│
├── core/                      # Shared infrastructure
│   ├── mod.rs
│   ├── database.rs            # Connection management only
│   ├── error.rs               # Common error types
│   └── security.rs            # Auth, encryption, session
│
├── features/                  # One module per domain
│   ├── finance/
│   │   ├── mod.rs
│   │   ├── models.rs          # Account, Transaction, Category
│   │   ├── repository.rs      # DB queries for finance
│   │   └── service.rs         # Business logic
│   │
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── models.rs          # Wallet, CryptoTransaction, Asset
│   │   ├── repository.rs      # DB queries for crypto
│   │   ├── service.rs         # Business logic
│   │   └── api.rs             # CoinGecko client
│   │
│   └── habits/
│       ├── mod.rs
│       ├── models.rs          # Habit, HabitLog
│       ├── repository.rs      # DB queries for habits
│       └── service.rs         # Business logic
│
├── ui/                        # Slint UI layer
│   ├── mod.rs
│   ├── callbacks.rs           # All Slint callback setup
│   ├── finance.rs             # Finance-specific UI logic
│   ├── crypto.rs              # Crypto-specific UI logic
│   └── habits.rs              # Habits-specific UI logic
│
└── services/                  # Shared services (charts, analytics)
    ├── mod.rs
    └── charts.rs
```

---

## Design Patterns Used

### 1. **Feature-Sliced Design**
Each domain is self-contained with its own models, repository, and service.

### 2. **Repository Pattern**
DB access abstracted behind repository interfaces per domain.

### 3. **Service Layer**
Business logic separated from data access and UI.

### 4. **Dependency Injection**
Services receive their dependencies (DB connection) via constructor.

### 5. **Single Responsibility**
Each file < 400 lines, each module has one clear purpose.

---

## Migration Phases

### Phase 1: Core Infrastructure
- [x] Create `core/` module structure
- [x] Extract common errors to `core/error.rs`
- [x] Extract security/auth logic to `core/security.rs`
- [ ] Gradually move `Database` implementation to `core/database.rs`

### Phase 2: Finance Feature
- [x] Create `features/finance/` structure
- [x] Create finance models in `features/finance/models.rs`
- [x] Create `features/finance/repository.rs` (delegates to db.rs)
- [x] Create `features/finance/service.rs` with business logic
- [ ] Gradually move DB operations from `db.rs` to repository

### Phase 3: Crypto Feature
- [x] Create `features/crypto/` structure
- [x] Create crypto models in `features/crypto/models.rs`
- [x] Create `features/crypto/repository.rs` (delegates to db.rs)
- [x] Extract API client from `services/crypto.rs` → `features/crypto/api.rs`
- [x] Create `features/crypto/service.rs` (move CryptoService from services/)
- [ ] Gradually move DB operations from `db.rs` to repository

### Phase 4: Habits Feature
- [x] Create `features/habits/` structure
- [x] Create habit models in `features/habits/models.rs`
- [x] Create `features/habits/repository.rs` (delegates to db.rs)
- [x] Create `features/habits/service.rs` with business logic
- [ ] Gradually move DB operations from `db.rs` to repository

### Phase 5: UI Layer Separation
- [x] Create `ui/` module structure
- [x] Extract helpers to `ui/helpers.rs` (formatting, parsing, icons)
- [x] Extract data loading to `ui/data.rs` (intermediate types)
- [ ] Extract Slint callbacks from `main.rs` to `ui/callbacks.rs`
- [ ] Split callbacks by domain: `ui/finance.rs`, `ui/crypto.rs`, `ui/habits.rs`
- [ ] Slim down `main.rs` to initialization only

### Phase 6: Controller Refactor
- [x] Split `controller.rs` into domain-specific modules
- [x] Each domain gets its own controller file
- [ ] Consider renaming to `app.rs` (thin orchestrator) in future

### Phase 7: Cleanup
- [x] Verify project compiles
- [ ] Delete empty/unused files
- [ ] Remove duplicate models (keep only in features/)
- [ ] Run `cargo clippy` and fix warnings
- [ ] Run `cargo test` to ensure nothing broke

---

## Rules for Future Development

1. **Max 400 lines per file** - Split if larger
2. **Features don't import each other** - Use events or shared services
3. **UI → Service → Repository → DB** - Never skip layers
4. **Models are domain-specific** - No global `models.rs`
5. **Test each layer independently** - Repository mocks for service tests

---

## Current Structure

```
src/
├── core/                      # NEW: Shared infrastructure
│   ├── mod.rs
│   ├── database.rs            # Re-exports from db.rs (gradual migration)
│   ├── error.rs               # Common error types
│   └── security.rs            # Security logging
│
├── features/                  # NEW: Domain modules
│   ├── mod.rs
│   ├── finance/
│   │   ├── mod.rs
│   │   ├── repository.rs      # Delegates to db.rs
│   │   └── service.rs         # Business logic (~400 lines)
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── api.rs             # CoinGecko API client (~443 lines)
│   │   ├── catalog.rs         # Coin catalog management (~203 lines)
│   │   ├── repository.rs      # Delegates to db.rs (~115 lines)
│   │   ├── service.rs         # Core service (~466 lines)
│   │   ├── transactions.rs    # Transaction operations (~626 lines)
│   │   └── validation.rs      # Input validation (~249 lines)
│   └── habits/
│       ├── mod.rs
│       ├── repository.rs      # Delegates to db.rs
│       └── service.rs         # Business logic (~100 lines)
│
├── ui/                        # NEW: UI layer (placeholder)
│   └── mod.rs
│
├── services/                  # Shared services (cross-cutting concerns)
│   └── charts.rs              # Chart generation (shared by features)
│
├── db/                        # Database operations (split by domain)
│   ├── mod.rs                 # Core: init, settings, session (~864 lines)
│   ├── crypto/                # Crypto DB ops (split)
│   │   ├── mod.rs             # Module declarations
│   │   ├── prices.rs          # Price cache (~146 lines)
│   │   ├── wallets.rs         # Wallet CRUD (~110 lines)
│   │   ├── transactions.rs    # Transaction CRUD (~268 lines)
│   │   └── portfolio.rs       # Aggregation, balance (~531 lines)
│   ├── finance.rs             # Finance DB ops (~572 lines)
│   └── habits.rs              # Habits DB ops (~184 lines)
│
├── controller/                # Split by domain
│   ├── mod.rs                 # Core: errors, types, db management (~723 lines)
│   ├── crypto.rs              # Crypto operations (~306 lines)
│   ├── finance.rs             # Finance operations (~274 lines)
│   ├── habits.rs              # Habits operations (~352 lines)
│   └── settings.rs            # App settings (~88 lines)
├── ui/                        # UI layer helpers
│   ├── mod.rs                 # Module declarations
│   ├── data.rs                # Intermediate data types for UI (~334 lines)
│   └── helpers.rs             # Formatting, parsing, streak utils (~366 lines)
│
├── models.rs                  # Single source: all domain models (~573 lines)
├── main.rs                    # UI + callbacks (~3885 lines, partially extracted)
└── lib.rs                     # Updated with new exports
```

---

## Progress Log

### Session 1 - 2024-12-28
- Created core/ infrastructure (error.rs, security.rs, database.rs)
- Created features/ structure for Finance, Crypto, Habits
- Created repository pattern for each feature (delegating to db.rs)
- Created service layer for Finance and Habits
- Updated lib.rs with new module structure
- Verified project compiles successfully

### Session 2 - 2024-12-28
- Fixed cargo clippy warnings (type complexity, collapsible if, redundant closure, missing Default)
- Extracted CoinGecko API client to `features/crypto/api.rs` (~443 lines)
- Removed `services/crypto.rs` entirely (updated imports in controller.rs and main.rs)
- Split crypto service into focused modules:
  - `service.rs` (~466 lines) - Core: wallets, prices, portfolio
  - `transactions.rs` (~626 lines) - Transaction operations
  - `catalog.rs` (~203 lines) - Coin catalog management
  - `validation.rs` (~249 lines) - Input validation helpers
- Added CryptoPriceEntry type alias in repository for cleaner signatures

### Session 3 - 2024-12-28
- Removed duplicate service files:
  - Deleted `services/finance.rs` (logic already in features/finance/service.rs)
  - Deleted `services/habit.rs` (logic already in features/habits/service.rs)
  - Deleted empty `services/analytics.rs` and `services/system.rs`
- Updated controller.rs imports to use features modules exclusively
- Cleaned up services/mod.rs (now only charts.rs remains)
- Removed unused duplicate model files from features:
  - Deleted `features/crypto/models.rs` (unused, all imports use crate::models)
  - Deleted `features/finance/models.rs` (unused, all imports use crate::models)
  - Deleted `features/habits/models.rs` (unused, all imports use crate::models)
- Decision: Keep `models.rs` as single source of truth until db.rs is split into repositories
- Split `db.rs` (2633 lines) into domain-specific modules:
  - `db/mod.rs` (~864 lines) - Core: Database struct, init, settings, session, rate limiting
  - `db/crypto.rs` (~1026 lines) - Wallets, transactions, prices, portfolio aggregation
  - `db/finance.rs` (~572 lines) - Accounts, transactions, categories, balance
  - `db/habits.rs` (~184 lines) - Habits CRUD, habit logs

### Session 4 - 2024-12-28
- Split `controller.rs` (1706 lines) into domain-specific modules:
  - `controller/mod.rs` (~723 lines) - Core: error types, analytics types, helpers, db management
  - `controller/crypto.rs` (~306 lines) - Wallet, transaction, price, portfolio methods
  - `controller/finance.rs` (~274 lines) - Account, transaction, category methods
  - `controller/habits.rs` (~352 lines) - Habit CRUD, logs, analytics
  - `controller/settings.rs` (~88 lines) - App settings, coin catalog
- Pattern: Uses `impl AppController` blocks spread across files for cohesion
- Split `db/crypto.rs` (1026 lines) into focused submodules:
  - `db/crypto/prices.rs` (~146 lines) - Exchange rates, price cache
  - `db/crypto/wallets.rs` (~110 lines) - Wallet CRUD
  - `db/crypto/transactions.rs` (~268 lines) - Transaction CRUD
  - `db/crypto/portfolio.rs` (~531 lines) - Balance calculations, aggregation
- Extracted UI helpers from main.rs to `ui/helpers.rs` (~296 lines)
  - Formatting functions (amounts, money, crypto)
  - Parsing utilities
  - Color and icon helpers
- main.rs reduced from 4224 to 3980 lines

### Session 5 - 2024-12-28
- Created `ui/data.rs` (~334 lines) with intermediate data structures:
  - AccountDisplayData, AccountsState, load_accounts_state()
  - TransactionDisplayData, TransactionsState, load_transactions_state()
  - CategoryDisplayData, load_categories()
  - BalanceDisplayData, load_balance_data()
  - RecentTransactionData, load_recent_transactions()
- Refactored main.rs reload functions to use ui/data.rs:
  - reload_accounts() simplified from 74 to 31 lines
  - reload_categories() simplified to use load_categories()
- main.rs reduced from 3980 to 3937 lines
- Pattern: Intermediate data types in lib.rs, mapped to Slint types in main.rs
- Added streak calculation helpers to ui/helpers.rs
  - calculate_current_streak(), calculate_best_streak()
- Refactored reload_habits to use streak helpers and color_from_hex
- main.rs reduced from 3937 to 3889 lines
- Added format_usd() helper for cleaner USD formatting
- Replaced 12 occurrences of `format_money((x * 100.0) as i64, "USD")` with `format_usd(x)`
- main.rs reduced from 3889 to 3885 lines
- ui/helpers.rs now 366 lines

### Next Steps
1. Further split main.rs callbacks into domain-specific ui/ modules
2. Extract reload_* functions to respective domain modules
3. Consider callback setup functions pattern for cleaner separation
