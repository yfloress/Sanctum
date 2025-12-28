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
- [ ] Split `services/crypto.rs`: API client → `api.rs`, service logic → `service.rs`
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
│   │   ├── models.rs          # Finance domain models
│   │   ├── repository.rs      # Delegates to db.rs
│   │   └── service.rs         # Business logic
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── models.rs          # Crypto domain models
│   │   └── repository.rs      # Delegates to db.rs
│   └── habits/
│       ├── mod.rs
│       ├── models.rs          # Habits domain models
│       ├── repository.rs      # Delegates to db.rs
│       └── service.rs         # Business logic
│
├── ui/                        # NEW: UI layer (placeholder)
│   └── mod.rs
│
├── services/                  # LEGACY: Being migrated
│   ├── crypto.rs              # Still used (large, needs splitting)
│   ├── finance.rs             # Migrated to features/finance/service.rs
│   ├── habit.rs               # Migrated to features/habits/service.rs
│   └── charts.rs              # Shared service
│
├── controller.rs              # LEGACY: Needs refactor
├── db.rs                      # LEGACY: Being split into repositories
├── models.rs                  # LEGACY: Being split into features
├── main.rs                    # LEGACY: UI needs extraction
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

### Next Steps
1. Move DB operations from db.rs to respective repositories
2. Remove duplicate models from models.rs
3. Split main.rs callbacks into ui/ modules
4. Split crypto.rs into api.rs and service.rs
5. Refactor controller.rs to be thin orchestrator
