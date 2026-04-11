# Pending Fixes — Finances Account Icons

## Fixed in this session

- [x] Account types corrected: bank, savings, credit, cash, other (removed checking/investment)
- [x] `getDefaultIconPath` handles `credit_card` (backend-normalized value)
- [x] Default type changed to `bank`
- [x] `$derived` replaces `{@const}` for `pickedIcon` to ensure reactivity
- [x] `refreshAccounts()` restored in edit account flow
- [x] **Create modal preview**: replaced `$derived pickedIcon` with `$effect` writing to `$state`
  vars `pickedIconSrc` / `pickedIconGeneric` — guarantees reactivity when icon is selected
- [x] **Account cards (list)**: removed `is_bank` restriction in `src/ui/data.rs` —
  `fetch_accounts` now returns `icon_path` for all account types (savings, cash, credit, other),
  not just bank accounts
