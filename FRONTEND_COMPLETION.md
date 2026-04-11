# Frontend Completion Checklist

> The Tauri 2 + Svelte 5 frontend has been scaffolded and styled, but many
> integrations are incomplete, broken, or missing entirely. This document
> tracks what needs to happen **before** we build new frontend-exclusive
> features.

## Ground Rules

1. **Every Tauri command must have a working frontend call.**
   Walk through each file in `src-tauri/src/commands/` and verify that
   every `#[tauri::command]` function is properly invoked from
   `ui-svelte/src/lib/api/`, with correct parameter names and types.

2. **Reference the Slint `dev` branch** (`git log dev`, `git diff dev`)
   to understand what features existed in the old UI. Use it as a
   feature-parity checklist, not as code to port.

3. **Fix first, add later.** No new frontend features until every
   existing integration compiles, runs, and produces correct results
   end-to-end.

---

## Status

### Crypto Module ✅
- [x] **Tax tab** — full UI: period selector, settings config, wallet exclusions, generate report, export CSV, IPC import
- [x] **Crypto SVG icons** — ticker bar shows coin icons; catalog tab shows icons
- [x] **Portfolio sync / price-fetch** — sync button in ticker bar
- [x] **Add-transaction flow** — full form for buy/sell/swap/income/transfer/fee
- [x] **Ticker bar** — USD/CLP rate + crypto prices + sync + gear config with reordering

### Habits Module ✅
- [x] **Streak rewards CRUD** — create/edit/delete flow
- [x] **Goals CRUD** — create/edit/delete + archive/complete
- [x] **Heatmap** — renders with real data + year navigation
- [x] **Radar and Weekday charts** — ECharts integration working

### Finances Module ✅
- [x] **Transaction CRUD** — add, edit, delete
- [x] **Account detail panel** — shows balance, type, currency, recent txs
- [x] **Transfer flow** — create and edit transfers
- [x] **Account icon editing** — icon picker in detail panel
- [x] **Category management** — add/delete categories reflected in dropdowns

### Settings / Data ✅
- [x] **Data import** (generic CSV + exchange CSV) — full flow: file select, preview, confirm, results
- [x] **Vault export** — file dialog opens and backup is saved
- [x] **Session timeout** — configurable, auto-lock triggers
- [x] **Language switch** — translations reload across all pages

### Dashboard ✅
- [x] **Net worth chart** — ECharts renders with real time-series (NetWorthChart component)
- [x] **Spending breakdown** — category colors and percentages
- [x] **Recent activity** — displays recent transactions

---

## Remaining / Blocked on Backend

| Item | Reason |
|------|--------|
| On-chain wallet import | No Tauri commands implemented |
| Tax: sync missing prices | No `syncTaxMissingPrices` command in backend |
| Login wallpaper | Command exists in controller but not exposed as Tauri command |
| On-chain custom RPC endpoints | No backend API |
| Account unarchive | `delete_account` does soft-delete; no unarchive command found |

---

## General Checks

- [x] All `invoke()` calls: parameter names match Rust snake_case
- [x] TypeScript types in `lib/types/` audited against Rust response structs
- [x] ECharts tree-shaking — line, bar, radar, pie all registered
- [ ] Test with a fresh vault (no data) — empty states should be graceful
- [ ] Test with a populated vault — real data may expose serialization issues

---

## Reference

| Branch    | Purpose                                      |
|-----------|----------------------------------------------|
| `dev`     | Old Slint UI — feature reference              |
| `main`    | Stable baseline with slint                    |
| `feat/tauri-svelte-frontend` | Current working branch     |
