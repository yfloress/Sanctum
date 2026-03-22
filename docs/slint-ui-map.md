# Sanctum Slint UI Map (Migration Reference)

Comprehensive inventory of the current Slint-based UI for migration to Tauri + Svelte.

---

## Application Shell (`ui/app.slint`)

### Layout Structure
- **When logged out:** Full-screen LoginPage
- **When logged in:** HorizontalLayout with collapsible Sidebar + main content area
- Content area conditionally renders one page at a time based on `AppState.active-page`

### Routing
Pages are selected by string key: `"dashboard"`, `"finances"`, `"habits"`, `"crypto"`, `"settings"`.

### Sidebar (`ui/components/sidebar.slint`)
- Collapsible (icon-only vs icon+label modes)
- Navigation items: Dashboard, Finances, Habits, Crypto, Settings
- Active page indicated with left-border accent and background highlight
- Lock/logout action at the bottom
- Collapse state persisted via `SettingsAdapter.set-sidebar-collapsed()`

### Theme System
- Dark mode by default; light mode available after login
- Login page always forces dark mode
- Theme sync: `SettingsAdapter.dark-mode` controls `Theme.is-dark` and `Palette.color-scheme`

### Global Overlay: Notification Toast
- Positioned bottom-right
- Shows success or error messages
- Controlled via `NotificationAdapter` (message, is-error, active)

---

## Page: Login (`ui/pages/login.slint`)

### Purpose
Authentication gate. Handles both vault creation (first run) and vault unlock (returning user).

### Sections
- **Background:** Gradient background with optional custom wallpaper (user-configurable)
- **Logo and title:** Sanctum logo SVG + "SANCTUM" title with subtitle
- **Password input:** Custom component with show/hide toggle
- **Weak password warning:** Inline warning banner for new vault creation (requires double-confirm)
- **Action button:** "Create Vault" or "Unlock" depending on vault existence
- **Restore link:** "Restore from backup" text link (only when vault exists)

### Data Displayed
- Vault existence status (checked on init)
- Password strength feedback (for new vault only)

### User Actions
1. Enter master password
2. Create new vault (with password strength check + double-confirm for weak passwords)
3. Unlock existing vault
4. Toggle password visibility
5. Restore vault from backup (opens RestoreVaultModal)

### Modals Opened
- RestoreVaultModal (via VaultAdapter.restore-vault)

---

## Page: Dashboard (`ui/pages/dashboard.slint`)

### Purpose
"Net Worth Command Center" -- overview of total financial position.

### Sections

1. **Exchange Rate Warning** -- conditional banner when FX rate is missing
2. **Hero Section** -- large total net worth display with fiat and crypto sub-totals as stat cards
3. **Controls Row** -- time range selector (1M, 6M, 1Y, ALL) + "Add Transaction" button
4. **Net Worth Chart** -- rendered chart image with min/max labels
5. **Spending Breakdown** -- scrollable list of expense categories with percentages and color bars
6. **Recent Activity** -- list of recent transactions with "View All" link to Finances page
7. **Loading State** -- animated icon + "Loading" text
8. **Error State** -- error message + retry button

### Data Displayed
- Total net worth (combined fiat + crypto)
- Fiat balance, crypto value (separate stat cards)
- Net worth chart over selected time range
- Expense category breakdown (name, amount, percentage, color)
- Recent transactions (date, description, category, amount, account)

### User Actions
1. Select analytics time range (1M / 6M / 1Y / ALL)
2. Add new transaction (opens AddTransactionModal)
3. View all transactions (navigates to Finances page)
4. Retry on error

### Modals Opened
- AddTransactionModal

---

## Page: Finances (`ui/pages/finances.slint`)

### Purpose
Unified view for managing fiat accounts and transactions.

### Tabs
1. **Activity** -- transaction list with filters
2. **Accounts** -- account management
3. **Settings** -- finance-specific settings (categories)

### Hero Header
- Total balance (all accounts combined)
- "Total Balance" subtitle
- Tab selector (pill-style)

### Tab: Activity
- **Header:** Section title + "New Entry" button
- **Filters:** Text search, account dropdown, category dropdown (grouped by expense/income/transfer), clear-filters button
- **Transaction list:** Scrollable list, each item clickable for editing; delete button per row; "Load More" pagination (100 at a time)
- **Empty state:** Different messages for "no transactions yet" vs "no matching results"

### Tab: Accounts
- **Header:** "My Accounts" + "Transfer" button + "New Account" button
- **Account list:** Each account shows icon, name, type, balance; clickable to open detail panel
- **Empty state:** Prompt to create first account

### Tab: Settings
- **Categories section:** Card with "Configure" button that opens ConfigureCategoriesModal

### Overlays
- **Account Detail Panel:** Slide-in panel from right showing selected account details (name, type, currency, balance, icon, transaction history); backdrop click to close
- **Delete Transaction Confirmation:** Modal dialog with cancel/delete buttons

### Data Displayed
- Total balance across all accounts
- Transaction list (date, description, category, amount, account, expense/income/transfer type)
- Account list (name, type, icon, currency, balance, archived status)
- Expense and income categories for filtering

### User Actions
1. Switch tabs (Activity / Accounts / Settings)
2. Add new transaction (opens AddTransactionModal)
3. Edit existing transaction (opens AddTransactionModal in edit mode)
4. Edit existing transfer (opens TransferFundsModal in edit mode)
5. Delete transaction (confirmation dialog)
6. Filter transactions by text, account, category
7. Load more transactions (pagination)
8. Add new account (opens AddAccountModal)
9. Transfer funds between accounts (opens TransferFundsModal)
10. View account details (slide-in panel)
11. Configure transaction categories (opens ConfigureCategoriesModal)

### Modals Opened
- AddTransactionModal (add/edit)
- TransferFundsModal (add/edit)
- AddAccountModal
- ConfigureCategoriesModal

---

## Page: Crypto (`ui/pages/crypto.slint`)

### Purpose
Crypto portfolio management with market tickers, wallets, and tax tools.

### Tabs
1. **Portfolio** (Assets) -- holdings overview
2. **Wallets** -- wallet management
3. **Tax** -- tax reporting tools

### Ticker Bar (always visible)
- **FX Rate Badge:** Shows USD/CLP exchange rate with live indicator
- **Scrollable ticker strip:** Horizontal list of market tickers (icon, symbol, price, 24h change)
- **Settings button:** Opens ConfigureTickerModal to select which coins appear

### Hero Section
- Portfolio total value (large text)
- Asset count and wallet count summary
- Coin limit warning (if too many coins)
- Tab selector (pill-style)
- Action button: "Add Transaction" (portfolio tab) or "Add Wallet" (wallets tab)
- Utility buttons: "Import CSV", "Sync" (refresh prices), "?" (last updated info)

### Tab: Portfolio
- **Stats bar:** Unrealized P&L, Realized YTD, ROI -- color-coded green/red
- **Holdings cards:** Horizontally scrollable crypto cards (icon, name, symbol, price, amount, value, 24h change); auto-layout in 1 or 2 rows based on count; clickable to open asset detail
- **Portfolio Trend Chart:** Line chart of portfolio value over time
- **Distribution Chart:** Pie/donut chart with allocation legend
- **Empty state:** "No assets yet" with prompt to create wallet first

### Tab: Wallets
- Delegated to `CryptoWalletsTab` component
- Wallet list with details, delete confirmation

### Tab: Tax
- Delegated to `CryptoTaxTab` component
- Tax period settings, jurisdiction, calculation method
- IPC data import, tax readiness checks
- Wallet exclusion toggles
- Tax report generation and export
- Tax history export

### Overlays (via `CryptoPageOverlays` component)
- **Asset Detail View:** Shows selected asset info, wallet breakdown, transaction history
- **Wallet Detail View:** Shows wallet holdings, transaction history, icon, category
- **Delete Wallet Confirmation:** Modal dialog

### Data Displayed
- Market ticker prices with 24h changes
- FX rate (USD/CLP)
- Portfolio total value, unrealized P&L, realized P&L (YTD), ROI
- Per-asset: icon, name, symbol, price, amount, value, 24h change, allocation
- Per-wallet: name, category, icon, balance, asset count
- Portfolio trend chart, distribution chart
- Tax readiness items, tax summary (period, proceeds, cost, gain, short/long term, disposals, income)

### User Actions
1. Switch tabs (Portfolio / Wallets / Tax)
2. Configure ticker bar (opens ConfigureTickerModal)
3. Add crypto transaction (opens AddCryptoTransactionModal)
4. Edit crypto transaction (opens EditCryptoTransactionModal)
5. Add wallet (opens AddCryptoWalletModal)
6. Edit wallet icon (opens EditWalletIconModal)
7. Delete wallet (confirmation dialog)
8. View asset details (overlay)
9. View wallet details (overlay)
10. Sync/refresh prices
11. Import exchange CSV (triggers exchange import flow)
12. View last updated info
13. Configure tax settings (period, jurisdiction, method, swap/fee inclusion)
14. Import IPC CSV for tax
15. Toggle wallet exclusion from tax
16. Sync missing prices for tax
17. Generate tax report
18. Export tax report / tax history

### Modals Opened
- ConfigureTickerModal
- AddCryptoTransactionModal
- EditCryptoTransactionModal
- AddCryptoWalletModal
- EditWalletIconModal
- ExchangeWalletSelectModal (via import flow)
- ImportPreviewModal (via import flow)
- ImportResultsModal (via import flow)

---

## Page: Habits (`ui/pages/habits.slint`)

### Purpose
Habit tracking with streak rewards, goals, and achievement history.

### Tabs
1. **Habits** -- daily habit tracking
2. **Rewards** -- streak rewards and goals
3. **History** -- achievement log

### Hero Header
- Page title "HABITS"
- Month navigation (left/right arrows + month/year display) -- only on Habits tab
- Tab selector (pill-style)

### Tab: Habits (delegated to `HabitsTab` component)
- Habit grid/list with daily checkboxes per month
- Heatmap visualization (last 365 days, with year navigation)
- Habit summary panel (name, current streak, best streak, completion rate, last 30 days, best day)
- Radar chart (habit analytics)
- Weekday efficiency chart
- Weekly and insight text summaries
- Add/edit/delete habit actions

### Tab: Rewards (delegated to `RewardsTab` component)
- Streak reward cards (habit name, consecutive vs accumulative, target days, milestones with progress)
- Goal cards (name, description, reward, deadline, checkpoints with toggle)
- Add/edit/delete streak rewards
- Add/edit/delete goals
- Complete/archive goals

### Tab: History (delegated to `HistoryTab` component)
- Achievement timeline (title, description, icon, date, type)

### Data Displayed
- Habits with daily completion status per month
- Heatmap (365 days, intensity levels 0-4)
- Streak data (current, best, completion rate)
- Radar chart, weekday efficiency chart
- Streak rewards with milestones and progress
- Goals with checkpoints
- Achievement history

### User Actions
1. Switch tabs (Habits / Rewards / History)
2. Navigate months (prev/next)
3. Toggle habit completion for a specific day
4. Add new habit (opens AddHabitModal)
5. Edit habit (opens AddHabitModal in edit mode)
6. Delete habit (confirmation dialog)
7. Select habit to view summary
8. Navigate heatmap years
9. Add streak reward (opens AddRewardModal)
10. Edit streak reward (opens AddRewardModal in edit mode)
11. Delete streak reward
12. Add goal (opens AddGoalModal)
13. Edit goal (opens AddGoalModal in edit mode)
14. Delete goal
15. Toggle checkpoint completion
16. Complete/archive goal

### Modals Opened
- AddHabitModal (add/edit)
- AddRewardModal (add/edit)
- AddGoalModal (add/edit)

---

## Page: Settings (`ui/pages/settings.slint`)

### Purpose
Application-wide configuration.

### Sections

1. **Appearance**
   - Dark mode toggle
   - Login wallpaper selector (select image / reset to default)

2. **Regional**
   - Preferred currency dropdown (USD, CLP, EUR, GBP, BRL, MXN, ARS, CAD, AUD, CHF, JPY)
   - Language selector (English / Espanol)

3. **Security**
   - Session timeout selector (5 min, 15 min, 30 min, 1 hour, Never)
   - Warning note about timeout behavior

4. **Vault Backup**
   - Export vault button (saves encrypted backup)
   - Restore vault button (opens RestoreVaultModal)
   - Encryption note (SQLCipher)

5. **Data Import**
   - Generic CSV import (opens file picker via IngestionAdapter)
   - Supported formats note, max size note

6. **Exchange CSV Import**
   - Exchange-specific CSV import button
   - Expandable help section with instructions per exchange (Kraken, Kraken Pro, Binance, MEXC, NotBank/CryptoMarket, Feather Wallet, Monero GUI Wallet)

7. **Data Sync**
   - Auto-fetch toggle (automatic price fetching)
   - Proxy toggle + proxy URL input
   - Proxy usage tip

8. **About**
   - Version info (1.0.0)
   - Encryption type (SQLCipher)
   - Database/storage type

9. **Security Note**
   - Shield icon with "Data Encrypted" badge

10. **Reset Settings**
    - Danger button to reset all settings to defaults

### User Actions
1. Toggle dark mode
2. Select/reset login wallpaper
3. Change preferred currency
4. Change language
5. Change session timeout
6. Export vault backup
7. Restore vault from backup
8. Import generic CSV data
9. Import exchange CSV data
10. Toggle auto-fetch prices
11. Toggle proxy / configure proxy URL
12. Reset all settings to defaults

### Modals Opened
- RestoreVaultModal (via vault restore)
- ImportPreviewModal (via import flow)
- ImportResultsModal (via import flow)
- ExchangeWalletSelectModal (via exchange import flow)

---

## Modals Inventory

### Finance Modals

| Modal | File | Purpose |
|-------|------|---------|
| AddTransactionModal | `modals/add_transaction.slint` | Create or edit a fiat transaction. Fields: account, amount, category (expense/income), description, date, expense toggle. Supports edit mode with pre-populated fields. |
| AddAccountModal | `modals/add_account.slint` | Create or edit a bank/cash/credit account. Fields: name, type, currency, initial balance. Supports edit mode. |
| TransferFundsModal | `modals/transfer_funds.slint` | Transfer funds between two accounts. Fields: from account, to account, amount, description, date. Supports edit mode. |
| ConfigureCategoriesModal | `modals/configure_categories.slint` | Manage transaction categories. Two sections (expense/income). Supports add, rename, and delete (non-default categories only). |
| EditAccountIconModal | `modals/edit_account_icon.slint` | Select a bank icon for a fiat account from a predefined grid (BankIconSelector component). |

### Crypto Modals

| Modal | File | Purpose |
|-------|------|---------|
| AddCryptoTransactionModal | `modals/add_crypto_transaction.slint` | Add a crypto transaction. Supports multiple types: buy, sell, transfer, swap, income, spend. Fields: wallet, coin, type, amount, price, fee (with optional fee-in-crypto), date, notes. Has collapsible advanced section and transaction summary card with tax category preview. |
| EditCryptoTransactionModal | `modals/edit_crypto_transaction.slint` | Edit an existing crypto transaction. Shows read-only fields (wallet, coin, type) plus editable fields (amount, price, fee, date, notes). Has collapsible advanced section for override proceeds/cost basis. |
| AddCryptoWalletModal | `modals/add_crypto_wallet.slint` | Create a new crypto wallet. Fields: name, category selection (exchange, hardware/multi-sig, software/single). Visual category cards with icons. |
| EditWalletIconModal | `modals/edit_wallet_icon.slint` | Change a crypto wallet's icon. Uses IconSelector component with category-specific defaults (exchange/hardware/software). |
| ConfigureTickerModal | `modals/configure_ticker.slint` | Configure which coins appear in the market ticker bar. Searchable list with toggle switches. Shows custom coin indicator. |

### Import/Export Modals

| Modal | File | Purpose |
|-------|------|---------|
| ImportPreviewModal | `modals/import_preview.slint` | Preview before confirming data import. Shows dry-run results: what will be added, modified, or skipped. Shared between generic and exchange import flows. Confirm or cancel. |
| ImportResultsModal | `modals/import_results.slint` | Shows final import results after execution: total processed, inserted, skipped, errors with details. Close to dismiss. |
| ExchangeWalletSelectModal | `modals/exchange_wallet_select.slint` | Intermediate step in exchange CSV import. User picks an existing wallet or creates a new one as the target for imported transactions. Shows wallet list with radio-style selection. |
| RestoreVaultModal | `modals/restore_vault_modal.slint` | Confirmation dialog before restoring vault from backup file. Shows backup path, warns about overwrite. Confirm or cancel. |

### Habit/Reward Modals

| Modal | File | Purpose |
|-------|------|---------|
| AddHabitModal | `modals/add_habit.slint` | Create or edit a habit. Fields: name, description, color (palette selector), category. Supports edit mode. |
| AddRewardModal | `modals/add_reward.slint` | Create or edit a streak reward. Fields: habit selector, type toggle (consecutive vs accumulative), target days/total, up to 3 milestones (day target + reward text). Supports edit mode. |
| AddGoalModal | `modals/add_goal.slint` | Create or edit a goal. Fields: name, description, reward text, deadline date, up to 4 checkpoints (description text). Supports edit mode. |

---

## Shared State Summary (`ui/globals.slint`)

### Adapters (backend bridges)
- **AuthAdapter** -- vault existence check, create, unlock, lock, password strength
- **AccountAdapter** -- fiat account CRUD, balance, icon editing, transfers, detail view
- **TransactionAdapter** -- fiat transaction CRUD, filtering, pagination
- **CategoryAdapter** -- expense/income category CRUD
- **DashboardAdapter** -- balance summary (total/fiat/crypto), recent transactions
- **AnalyticsAdapter** -- chart data, net worth, expense breakdown by time range
- **HabitAdapter** -- habit CRUD, daily toggle, month/year navigation, heatmap, analytics
- **RewardsAdapter** -- streak rewards, goals, checkpoints, achievements, milestones
- **CryptoAdapter** -- portfolio, wallets, tickers, coin catalog, asset/wallet details, tax settings/reports, transactions
- **SettingsAdapter** -- all app settings (dark mode, currency, language, timeout, proxy, wallpaper, auto-fetch)
- **IngestionAdapter** -- generic CSV import + exchange CSV import flow
- **VaultAdapter** -- vault export/restore/rollback
- **NotificationAdapter** -- toast notification display

### AppState (UI state)
- Authentication: `is-logged-in`
- Routing: `active-page`
- Loading: `is-loading`
- Modal visibility flags: 18 boolean properties controlling modal display
- Edit mode flags and selected indices for transactions, accounts, transfers
- Current date, sidebar collapsed state, login wallpaper

---

## Key Patterns for Migration

1. **Modal management:** All modals are controlled by boolean flags on `AppState`. Modals render as full-screen overlays with backdrop click-to-close.
2. **Tab navigation:** Pages with tabs use a local `active-tab` string property. Tab content is conditionally rendered.
3. **Data loading:** Pages call `fetch-*` callbacks in their `init` block. Loading states show spinner overlays.
4. **Filtering:** The Finances page has real-time filtering (text + dropdowns) that triggers re-fetch on every change.
5. **Detail views:** Both Finances (accounts) and Crypto (assets/wallets) use slide-in overlay panels for detail views.
6. **Delete confirmations:** Always use an inline confirmation dialog before destructive actions.
7. **i18n:** All user-visible text comes from `Translations.*` keys.
8. **Theming:** All colors from `Theme.*` tokens; dark/light mode supported.
