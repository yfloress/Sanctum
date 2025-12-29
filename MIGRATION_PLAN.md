# Sanctum Architecture Migration Plan

## Status: COMPLETED

Migration from monolithic architecture to Feature-Sliced Design completed on 2024-12-29.

---

## Original Problem

| File | Lines | Issue |
|------|-------|-------|
| `main.rs` | 4,224 | UI + logic + callbacks mixed |
| `db.rs` | 2,633 | All DB operations |
| `controller.rs` | 1,706 | God object |
| `services/crypto.rs` | 1,959 | Too large |

---

## Final Architecture

### src/ Structure
```
src/
├── main.rs                    # UI bootstrap (~829 lines)
├── lib.rs                     # Crate exports + slint::include_modules!()
│
├── core/                      # Shared infrastructure
│   ├── database.rs            # Re-exports from db.rs
│   ├── error.rs               # DbError enum
│   └── validation.rs          # Shared validation functions
│
├── features/                  # Domain modules
│   ├── finance/
│   │   ├── service.rs         # Account operations
│   │   ├── transactions.rs    # Transaction CRUD
│   │   ├── analytics.rs       # Net worth, charts
│   │   ├── validation.rs      # Input validation
│   │   └── repository.rs      # DB delegation
│   ├── crypto/
│   │   ├── service.rs         # Core service
│   │   ├── transactions.rs    # Transaction operations
│   │   ├── api.rs             # CoinGecko client
│   │   ├── catalog.rs         # Coin catalog
│   │   ├── validation.rs      # Input validation
│   │   └── repository.rs      # DB delegation
│   └── habits/
│       ├── service.rs         # Business logic
│       └── repository.rs      # DB delegation
│
├── controller/                # Orchestration layer
│   ├── mod.rs                 # Core + error types
│   ├── finance.rs             # Finance methods
│   ├── crypto.rs              # Crypto methods
│   ├── habits.rs              # Habits methods
│   └── settings.rs            # App settings
│
├── db/                        # Database operations
│   ├── mod.rs                 # Core: init, session, migrations
│   ├── crypto/                # Crypto DB (split)
│   ├── finance.rs             # Finance DB
│   └── habits.rs              # Habits DB
│
├── ui/                        # UI layer
│   ├── callbacks/             # Slint callback setup
│   │   ├── crypto/            # Crypto callbacks (6 files)
│   │   ├── habits/            # Habits callbacks (5 files)
│   │   ├── finance.rs         # Finance callbacks
│   │   └── dashboard.rs       # Dashboard callbacks
│   ├── data.rs                # Intermediate data types
│   └── helpers.rs             # Formatting utilities
│
├── models.rs                  # Domain models
├── security_log.rs            # Security logging
└── services/charts.rs         # Chart generation
```

### ui/ Structure (Slint)
```
ui/
├── pages/                     # Page components
├── modals/                    # Modal dialogs
├── components/
│   ├── buttons/               # TabButton
│   ├── sections/              # SectionHeader, EmptyState
│   ├── filters/               # FilterInput
│   ├── forms/                 # FormField, TypeSelector, WalletSelector, CoinSelector
│   └── [feature components]   # crypto_widgets, wallet_detail, etc.
├── globals.slint              # Adapters + Palette
├── widgets.slint              # Base widgets
└── app.slint                  # Main layout
```

---

## Design Patterns

1. **Feature-Sliced Design** - Each domain self-contained
2. **Repository Pattern** - DB access abstracted per domain
3. **Service Layer** - Business logic separated from DB
4. **Layer Separation** - UI → Controller → Features → DB

---

## Key Metrics

### src/ Reduction
| File | Before | After | Change |
|------|--------|-------|--------|
| main.rs | 4,224 | 829 | -80% |
| controller.rs | 1,706 | split | 5 files |
| db.rs | 2,633 | split | 6 files |

### ui/ Reduction
| File | Before | After | Change |
|------|--------|-------|--------|
| finances.slint | 1,262 | 1,060 | -16% |
| habits.slint | 511 | 424 | -17% |
| transaction_form_components.slint | 761 | 7 | -99% (re-exports) |

---

## Rules for Future Development

1. **Max ~500 lines per file** - Split if larger
2. **UI → Controller → Features → DB** - Never skip layers
3. **Shared validation in `core/validation.rs`** - Domain wrappers in features
4. **Components in subdirectories** - buttons/, sections/, forms/, filters/
5. **No duplicate code** - Extract to shared modules

---

## Cleanup Completed

- [x] Removed `init_logger()` - unused function
- [x] Removed `MonthlyTrendData` - unused struct in globals.slint
- [x] Removed duplicate `core/security.rs` - used security_log.rs
- [x] Removed TODO comments - completed or no longer relevant
- [x] All `cargo clippy` warnings fixed
- [x] All `cargo check` passes
