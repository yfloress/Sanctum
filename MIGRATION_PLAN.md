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
- [ ] Extract Slint callbacks from `main.rs` to `ui/callbacks.rs`
- [ ] Split callbacks by domain: `ui/finance.rs`, `ui/crypto.rs`, `ui/habits.rs`
- [ ] Slim down `main.rs` to initialization only

### Phase 6: Controller Refactor
- [ ] Slim down `controller.rs` → `app.rs` (thin orchestrator)
- [ ] Each domain service handles its own logic
- [ ] Controller only coordinates between domains

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
│   ├── crypto.rs              # Crypto DB ops (~1026 lines)
│   ├── finance.rs             # Finance DB ops (~572 lines)
│   └── habits.rs              # Habits DB ops (~184 lines)
│
├── controller.rs              # LEGACY: Needs refactor (~1706 lines)
├── models.rs                  # Single source: all domain models (~573 lines)
├── main.rs                    # LEGACY: UI needs extraction (~4224 lines)
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

### Next Steps
1. Split main.rs callbacks into ui/ modules
2. Refactor controller.rs to be thin orchestrator
3. Consider splitting db/crypto.rs further if needed
