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

## Known Gaps

### Crypto Module
- [ ] **Tax tab** is a placeholder — needs full UI for tax reports,
      cost-basis methods, IPC import, and year selection.
- [ ] **Crypto SVG icons** are not rendering — verify asset icon
      paths / inline SVGs are correct.
- [ ] Portfolio sync / price-fetch button may not be wired.
- [ ] Verify add-transaction flow for crypto wallets works end-to-end.

### Habits Module
- [ ] **Streak rewards CRUD** — UI only displays rewards, no create/edit/
      delete flow exists.
- [ ] **Goals CRUD** — no UI to create or edit goals, only display and
      toggle checkpoints.
- [ ] Verify heatmap renders correctly with real data.
- [ ] Radar and Weekday charts — confirm ECharts integration is
      producing output with real analytics data.

### Finances Module
- [ ] Verify all transaction CRUD operations work (add, edit, delete).
- [ ] Verify account detail panel shows correct data.
- [ ] Transfer flow — confirm accounts list populates and transfer
      executes.
- [ ] Category management — add/delete should reflect in dropdowns.

### Settings / Data
- [ ] **Data import** (generic CSV + exchange CSV) — full flow:
      file select, preview, confirm, results.
- [ ] **Vault export** — confirm file dialog opens and backup is saved.
- [ ] Session timeout auto-lock — verify the monitor triggers lock.
- [ ] Language switch — verify translations reload across all pages.

### Dashboard
- [ ] Net worth chart — confirm ECharts renders with real time-series.
- [ ] Spending breakdown — verify category colors and percentages.
- [ ] Recent activity — confirm navigation to edit transaction works.

### General
- [ ] Audit every `invoke()` call: parameter names **must** match Rust
      snake_case exactly (e.g. `account_id`, not `accountId`).
- [ ] Audit all TypeScript types in `lib/types.ts` against actual Rust
      response structs — missing or mismatched fields will silently fail.
- [ ] Check ECharts tree-shaking — ensure all used chart types
      (line, bar, radar, pie) are registered.
- [ ] Test with a fresh vault (no data) — empty states should be
      graceful, not broken.
- [ ] Test with a populated vault — real data may expose serialization
      or rendering issues.

### Others
- And more things can be missing you need to be capable of find all. When you finish
all from above do this a research for more missing functionalities. 

---

## Workflow

1. Pick a module from the list above.
2. Open the relevant `src-tauri/src/commands/*.rs` file and the
   corresponding `ui-svelte/src/lib/api/*.ts` + page component.
3. Cross-check every command, parameter, and response type.
4. Fix, test, mark the checkbox.
5. When all checkboxes are done, delete this file and start building
   new frontend features.

---

## Reference

| Branch    | Purpose                                      |
|-----------|----------------------------------------------|
| `dev`     | Old Slint UI — feature reference              |
| `main`    | Stable baseline with slint                    |
| `feat/tauri-svelte-frontend` | Current working branch     |
