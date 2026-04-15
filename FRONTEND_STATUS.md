# Frontend Migration Status — Tauri + Svelte 5

> Single tracking document for the Slint → Tauri+Svelte migration.
> Replaces: `TODO_SVELTE.md`, `FRONTEND_COMPLETION.md`, `FIXES_PENDING.md` (deleted).

---

## Command Coverage

97/97 Tauri commands have frontend wrappers (100%).

---

## Completed Modules

- **Crypto** — Portfolio, wallets, transactions (buy/sell/swap/income/transfer/fee), ticker bar, coin catalog, tax (settings/report/export/IPC), sync, wallet exclusions
- **Finances** — Accounts CRUD, transactions CRUD, transfers, icon picker, categories, Overview/Activity/Settings tabs with ECharts
- **Habits** — Habits CRUD, toggle, heatmap (year nav), analytics (radar/weekday), streak rewards, goals with checkpoints, achievements
- **Dashboard** — Net worth chart, cash flow chart, spending breakdown, recent activity, stats row, trend badges
- **Settings** — Dark mode, currency selector, language selector, session timeout, vault export, data import (generic + exchange CSV), auto-fetch, proxy, about, reset
- **Login** — Create/unlock vault, password strength, restore from backup

---

## Pending Work

### Requires Frontend Implementation

| Item | Details |
|------|---------|
| **i18n integration** | ~1000 translation keys in `en.ftl`/`es.ftl` exist but `i18n.t()` is never called. All UI strings hardcoded English. |
| **Currency formatting** | `preferred_currency` setting persists but no page uses it to format amounts. |
| **Session timeout "Never"** | Frontend dropdown offers 0 ("Never") but backend clamps to 60s. Mismatch. |
| **Translation audit** | Locale files were written for the Slint frontend. ~80% of keys should match, but many need updating for new component text. See `TRANSLATION_STATUS.md`. |
| **Account unarchive** | `delete_account` does soft-delete; no frontend UI to view/restore archived accounts. |
| **Empty state testing** | Test all pages with a fresh vault (no data). |
| **Populated vault testing** | Test with real data for serialization issues. |

### Blocked on Backend

| Item | Reason |
|------|--------|
| On-chain wallet import | No Tauri commands implemented |
| Login wallpaper | Command exists in controller but not exposed as Tauri command |
| On-chain custom RPC endpoints | No backend API |

---

## Reference

| Branch | Purpose |
|--------|---------|
| `dev` | Old Slint UI — feature reference |
| `main` | Stable baseline with Slint |
| `feat/tauri-svelte-frontend` | Current working branch |
