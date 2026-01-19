# Dev branch research notes (Sanctum app)

## Theme palette (source of truth)
- Dark (default): black base with purple/pink gradients.
  - Primary: #6366f1
  - Action gradient: #5a00f0 -> #ff2ac3
- Light: cream background with gold accents.
  - Background: #f7f1e5
  - Primary: #b8860b
  - Action gradient: #c9a227 -> #e3b566
Source: `ui/theme.slint` on `dev`.

## Core features (high level)
- Security/vault: encrypted DB, session timeout, vault backup/restore with warnings.
- Finance: accounts, transactions, categories, transfers, dashboard analytics.
- Crypto: wallets, transactions, portfolio, price sync (CoinGecko), catalog with custom coins, proxy + auto-fetch.
- Habits: daily logs, heatmap, analytics, streak rewards, milestones, goals, achievements.
- Import: preview + confirm, dedup, size limits, no network calls.

## Import (exact accepted formats)
### JSON v1
```
{
  "version": "1" | "1.0",
  "transactions": [...],
  "habit_logs": [...],
  "crypto_transactions": [...]
}
```

Transaction item:
- date (YYYY-MM-DD)
- account (string)
- type (income | expense | transfer)
- amount (positive)
- currency (3-letter)
- category (required unless transfer)
- description
- transfer_to_account (required for transfer)

Habit log item:
- habit (string)
- date (YYYY-MM-DD)
- completed (bool) -> false is skipped

Crypto transaction item:
- date (YYYY-MM-DD)
- wallet (string)
- symbol (string)
- type (buy | sell | transfer_in | transfer_out)
- amount (positive)
- price_per_coin? (optional)
- fee? (optional)
- notes? (optional)

### CSV
Transactions headers:
- date,account,type,amount,currency,category,description,transfer_to_account

Habit logs headers:
- habit,date,completed

Crypto headers:
- date,wallet,symbol,type,amount,price_per_coin,fee,notes

### TXT (mixed prefixes)
- Transactions: `T;date;account;type;amount;currency;category;description;transfer_to`
- Habit logs: `H;habit;date;completed`
- Crypto: `C;date;wallet;symbol;type;amount;price;fee;notes`

## Import rules & behavior
- File size limit: 10MB.
- Entity resolution: account/habit/category/wallet/coin lookup by trimmed, case-insensitive names. No auto-creation.
- Transfers: require destination account; category optional/ignored.
- Dedup keys:
  - Finance: date + account + currency + amount + type + description (+ transfer_to_account if transfer)
  - Habits: habit + date
  - Crypto: date + wallet + coin + type + amount
- Crypto balance validation: sell/transfer_out must not exceed available balance.
- Preview mode uses dry-run; no DB writes.

## Source files (dev)
- Theme: `ui/theme.slint`
- UI globals/adapters: `ui/globals.slint`
- Ingestion: `src/features/ingestion/*`, `src/ui/callbacks/ingestion.rs`, `src/controller/ingestion.rs`
