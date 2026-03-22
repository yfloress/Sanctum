# Slint UI Callback Inventory (IPC Boundary Map)

Complete inventory of all Slint UI callbacks registered in the Rust backend.
Each callback defines one IPC boundary that will become a Tauri command.

---

## 1. Vault Domain

**Adapter:** `VaultAdapter`
**File:** `src/ui/callbacks/vault.rs`

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 1 | `on_export_vault` | Export encrypted vault backup to file | None (opens native file save dialog) | Writes `.db` file to chosen path; shows notification | `controller.export_vault(path)` |
| 2 | `on_restore_vault` | Open file picker to select a vault backup | None (opens native file picker) | Sets `restore_backup_path` on VaultAdapter; shows restore confirmation modal | None (UI state only) |
| 3 | `on_confirm_restore` | Execute the vault restore from selected backup | None (reads `restore_backup_path` from adapter) | Replaces current vault DB; closes modal; redirects to login screen | `controller.restore_vault(path)` |
| 4 | `on_rollback_restore` | Roll back a previously restored vault | None | Reverts to previous vault; redirects to login screen | `controller.rollback_restore()` |

---

## 2. Settings Domain

**Adapter:** `SettingsAdapter`
**File:** `src/ui/callbacks/settings.rs`

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 1 | `on_load_settings` | Load all persisted settings into UI | None | Sets dark_mode, auto_fetch, proxy, session_timeout, currency, language, sidebar, wallpaper on adapters; starts auto-fetch timer if enabled; applies language | `controller.get_app_setting(*)`, `controller.get_login_wallpaper_path()`, `controller.load_exchange_rate_allow_stale()` |
| 2 | `on_set_dark_mode` | Toggle dark mode | `enabled: bool` | Persists setting | `controller.set_app_setting("dark_mode", val)` |
| 3 | `on_set_auto_fetch` | Toggle auto-fetch of crypto prices | `enabled: bool` | Starts/stops 60s timer; triggers silent price refresh if enabling | `controller.set_app_setting("auto_fetch", val)` |
| 4 | `on_set_proxy_enabled` | Toggle crypto API proxy | `enabled: bool` | Validates proxy URL first; persists setting | `controller.validate_crypto_proxy_url()`, `controller.set_crypto_proxy_enabled()` |
| 5 | `on_set_proxy_url` | Set crypto proxy URL | `url: SharedString` | Persists proxy URL | `controller.set_crypto_proxy_url(url)` |
| 6 | `on_set_session_timeout` | Set vault auto-lock timeout | `timeout_secs: i32` | Persists timeout in seconds | `controller.set_session_timeout(timeout)` |
| 7 | `on_set_preferred_currency` | Change display currency | `currency: SharedString` | Persists setting; refreshes dashboard balance, portfolio, accounts, and FX rate | `controller.set_app_setting("preferred_currency", val)` |
| 8 | `on_set_preferred_language` | Change UI language | `language: SharedString` | Persists setting; reloads all i18n translations | `controller.set_app_setting("preferred_language", val)` |
| 9 | `on_select_login_wallpaper` | Pick custom login wallpaper image | None (opens file picker) | Loads image; persists path in config.toml; updates AppState | `controller.set_login_wallpaper_path(Some(path))` |
| 10 | `on_reset_login_wallpaper` | Reset wallpaper to default | None | Clears wallpaper path; resets UI flags | `controller.set_login_wallpaper_path(None)` |
| 11 | `on_set_sidebar_collapsed` | Persist sidebar collapse state | `collapsed: bool` | Persists setting | `controller.set_app_setting("sidebar_collapsed", val)` |
| 12 | `on_reset_settings` | Reset all settings to defaults | None | Resets all settings to defaults; reloads settings and translations; clears wallpaper | `controller.set_app_setting(*)`, `controller.set_login_wallpaper_path(None)` |

---

## 3. Dashboard Domain

**Adapters:** `DashboardAdapter`, `AnalyticsAdapter`
**File:** `src/ui/callbacks/dashboard.rs`

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 1 | `on_fetch_balance` | Compute and display net worth (fiat + crypto) | None | Sets `BalanceData` with total, fiat, and crypto balances in preferred currency; handles multi-currency normalization via FX rates | `controller.get_accounts()`, `controller.get_account_balances()`, `controller.get_aggregated_portfolio()`, `controller.load_crypto_prices()` |
| 2 | `on_fetch_recent` | Load recent transactions for dashboard | None | Populates recent transactions list via `reload_recent` closure | Delegated to `reload_recent()` |
| 3 | `on_fetch_analytics` | Generate analytics data with chart for a time range | `range: SharedString` (e.g. "7D", "30D", "ALL") | Sets `AnalyticsData` with chart image, net worth, min/max, expense breakdown by category | `controller.get_dashboard_data()`, `controller.render_net_worth_chart()`, `controller.get_crypto_portfolio_snapshots()` |

---

## 4. Finance Domain

**Adapters:** `AccountAdapter`, `TransactionAdapter`, `CategoryAdapter`
**File:** `src/ui/callbacks/finance.rs`

### AccountAdapter

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 1 | `on_fetch_accounts` | Load all accounts list | None | Populates accounts list via `reload_accounts` closure | Delegated to `reload_accounts()` |
| 2 | `on_fetch_account_details` | Load single account detail with transactions | `account_id: SharedString` | Sets selected account name/type/currency/balance/icon and filtered transaction history | `controller.get_accounts()`, `controller.get_account_balances()`, `controller.get_transactions()`, `controller.get_transaction_categories()` |
| 3 | `on_create_account` | Create a new financial account | `name, account_type, currency, initial_balance: SharedString` | Returns error string or empty on success; reloads accounts; opens icon editor for bank type | `controller.create_account(name, type, currency, amount, color, icon)` |
| 4 | `on_update_account` | Update an existing account | `id, name, account_type, currency, initial_balance: SharedString` | Returns error string or empty; reloads accounts and recent; refreshes dashboard | `controller.update_account(...)` |
| 5 | `on_transfer_funds` | Transfer money between accounts | `from_id, to_id, amount, description, date: SharedString` | Returns error string or empty; reloads accounts, transactions, recent; refreshes dashboard | `controller.transfer_funds(from, to, amount, desc, date)` |
| 6 | `on_update_transfer` | Edit an existing transfer | `id, from_id, to_id, amount, description, date: SharedString` | Returns error string or empty; reloads all relevant views | `controller.update_transfer(...)` |
| 7 | `on_delete_account` | Archive (soft-delete) an account | `id: SharedString` | Returns error string or empty; reloads accounts | `controller.archive_account(id)` |
| 8 | `on_update_account_icon` | Set or clear account bank icon | `id, icon: SharedString` | Returns error string or empty; reloads accounts and detail view | `controller.update_account_icon(id, icon_path)` |
| 9 | `on_update_account_name` | Rename an account | `id, new_name: SharedString` | Returns error string or empty; reloads accounts and detail name | `controller.update_account_name(id, name)` |

### TransactionAdapter

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 10 | `on_fetch_transactions` | Load all transactions | None | Populates transactions and recent via closures | Delegated to `reload_transactions()`, `reload_recent()` |
| 11 | `on_add_transaction` | Add a new transaction | `account_id, amount, category, description, date: SharedString, is_expense: bool` | Returns error string or empty; reloads transactions/accounts/recent; refreshes dashboard/analytics | `controller.add_transaction(account_id, amount, category, desc, date, is_expense)` |
| 12 | `on_update_transaction` | Update an existing transaction | `id, account_id, amount, category, description, date: SharedString, is_expense: bool` | Returns error string or empty; reloads all relevant views | `controller.update_transaction(...)` |
| 13 | `on_delete_transaction` | Delete a transaction | `id: SharedString` | Returns error string or empty; reloads all relevant views | `controller.delete_transaction(id)` |

### CategoryAdapter

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 14 | `on_load_categories` | Load expense/income categories | None | Populates category lists via closure | Delegated to `reload_categories()` |
| 15 | `on_add_category` | Create a new category | `name, category_type: SharedString` | Returns error string or empty; reloads categories | `controller.add_transaction_category(name, type)` |
| 16 | `on_update_category` | Rename a category | `id, new_name: SharedString` | Returns error string or empty; reloads categories | `controller.update_transaction_category(id, name)` |
| 17 | `on_delete_category` | Delete a category | `id: SharedString` | Returns error string or empty; reloads categories | `controller.delete_transaction_category(id)` |

---

## 5. Crypto Domain

**Adapter:** `CryptoAdapter`
**Files:** `src/ui/callbacks/crypto/*.rs`

### Portfolio & Prices (`crypto/portfolio.rs`)

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 1 | `on_fetch_portfolio` | Load aggregated portfolio with prices, charts, tickers, distribution, PnL, realized gains, trend | None | Sets portfolio list, market tickers, total value/PnL/ROI/realized, distribution chart, trend chart, FX badge, last updated label | `controller.get_aggregated_portfolio()`, `controller.load_crypto_prices()`, `controller.get_active_ticker_ids()`, `controller.render_portfolio_distribution_chart()`, `controller.render_portfolio_trend_chart()`, `controller.generate_tax_summary()`, `controller.save_crypto_portfolio_snapshot()` |
| 2 | `on_refresh_prices` | Manual price refresh with loading overlay and cooldown | None | Fetches prices via API (async/tokio); saves prices; fetches FX rate; updates all portfolio UI; 8s cooldown between manual refreshes | `controller.get_monitored_coin_ids()`, `controller.get_crypto_prices()` (async), `controller.save_crypto_prices()`, `controller.get_usd_fx_rate()` (async), `controller.save_exchange_rate()` |
| 3 | `on_refresh_prices_silent` | Auto-fetch price refresh without loading overlay | None | Same as manual refresh but no loading indicator; triggered by auto-fetch timer | Same as `on_refresh_prices` |
| 4 | `on_get_last_price` | Get cached price for a coin | `coin_id: SharedString` | Returns formatted price string or empty | `controller.load_crypto_prices()` |
| 5 | `on_request_historical_price` | Fetch historical price for a coin on a specific date | `coin_id, date: SharedString, user_initiated: bool` | Async fetch; sets `historical_price_key` and `historical_price_value` on adapter; caches results; deduplicates requests | `controller.get_crypto_historical_price_usd()` (async) |
| 6 | `on_show_last_updated_info` | Display last price update timestamp as notification | None | Shows toast with formatted timestamp | None (reads adapter state) |
| 7 | `on_get_swap_quote` | Calculate swap quote between two coins | `from_coin_id, to_coin_id, amount_str: SharedString` | Returns formatted to-amount string | `controller.load_crypto_prices()` |
| 8 | `on_get_available_balance` | Get available balance for a coin in a wallet at a date | `wallet_id, coin_id, date: SharedString` | Returns formatted balance string | `controller.get_available_balance(wallet, coin, date)` |
| 9 | `on_fetch_asset_details` | Load detailed view for a single crypto asset | `coin_id: SharedString` | Sets selected asset data, per-wallet breakdown, and transaction history | `controller.get_aggregated_portfolio()`, `controller.get_wallets()`, `controller.get_wallet_holdings()`, `controller.get_crypto_transactions_by_coin()` |

### Wallets (`crypto/wallets.rs`)

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 10 | `on_fetch_wallets` | Load all wallets with balances | None | Sets wallets list, wallet simple list, portfolio summary | `controller.get_wallets()`, `controller.get_wallet_holdings()`, `controller.load_crypto_prices()` |
| 11 | `on_fetch_wallet_details` | Load single wallet detail with holdings and history | `wallet_id: SharedString` | Sets selected wallet name/category/icon/balance, holdings list, transaction history | `controller.get_wallets()`, `controller.get_wallet_holdings()`, `controller.get_wallet_transactions()` |
| 12 | `on_create_wallet` | Create a new wallet | `name, category: SharedString` | Returns error or empty; reloads wallets; sets icon edit fields for follow-up | `controller.add_wallet(name, category, None)` |
| 13 | `on_create_wallet_with_icon` | Create wallet with a custom icon | `name, category, icon: SharedString` | Returns error or empty; reloads wallets | `controller.add_wallet(name, category, icon)` |
| 14 | `on_get_wallet_tx_count` | Get transaction count for a wallet | `id: SharedString` | Returns `i32` count | `controller.get_wallet_transaction_count(id)` |
| 15 | `on_delete_wallet` | Delete a wallet (with optional force) | `id: SharedString, force: bool` | Returns error or empty; reloads wallets | `controller.delete_wallet(id, force)` |
| 16 | `on_update_wallet_name` | Rename a wallet | `id, new_name: SharedString` | Returns error or empty; reloads wallets and detail view | `controller.update_wallet_name(id, name)` |
| 17 | `on_update_wallet_icon` | Update wallet icon | `id, icon: SharedString` | Returns error or empty; reloads wallets and detail view | `controller.update_wallet_icon(id, icon)` |

### Transactions (`crypto/transactions.rs`)

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 18 | `on_add_transaction` | Add a crypto transaction (buy/sell/etc.) | `wallet_id, coin_id, symbol, type, amount, price, fee, fee_coin_id, fee_coin_amount, date, notes, subtype, override_proceeds, override_cost_basis: SharedString` | Returns error or empty; reloads portfolio/wallets; refreshes analytics; saves last wallet/coin IDs | `controller.add_crypto_transaction(...)` |
| 19 | `on_add_transfer` | Add a crypto transfer between wallets | `from_wallet, to_wallet, coin_id, symbol, from_amount, to_amount, fee, fee_coin_id, fee_coin_amount, date, notes: SharedString` | Returns error or empty; reloads portfolio/wallets; refreshes analytics | `controller.add_crypto_transfer(...)` |
| 20 | `on_add_swap` | Add a crypto swap (coin-to-coin) | `wallet_id, from_coin, from_symbol, from_amount, to_coin, to_symbol, to_amount, fee, fee_coin_id, fee_coin_amount, date, notes: SharedString` | Returns error or empty; reloads portfolio/wallets; refreshes analytics | `controller.add_crypto_swap(...)` |
| 21 | `on_load_edit_transaction` | Load transaction data into edit form | `id: SharedString` | Returns error or empty; populates edit_tx_* fields on adapter; blocks editing paired swap transactions | `controller.get_crypto_transaction(id)`, `controller.get_wallets()` |
| 22 | `on_update_transaction` | Update a crypto transaction | `id, amount, price, fee, fee_coin_id, fee_coin_amount, date, notes, subtype, override_proceeds, override_cost_basis: SharedString` | Returns error or empty; refreshes portfolio/wallets/analytics | `controller.update_crypto_transaction(...)` |
| 23 | `on_delete_crypto_transaction` | Delete a crypto transaction | `id: SharedString` | Returns error or empty; refreshes asset details/portfolio/wallets/analytics | `controller.delete_crypto_transaction(id)` |

### Catalog (`crypto/catalog.rs`)

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 24 | `on_load_ticker_options` | Load ticker configuration with enabled state | None | Sets `ticker_options` model on adapter | `controller.get_active_ticker_ids()`, `controller.get_coin_catalog_or_default()` |
| 25 | `on_load_coin_catalog` | Load full coin catalog with favorites | None | Sets `coin_catalog` and `default_coin_index` on adapter | `controller.get_coin_catalog_or_default()`, `controller.get_favorite_coin_ids()` |
| 26 | `on_save_ticker_options` | Save which tickers are enabled for the price bar | None (reads model from adapter) | Persists active ticker IDs; reloads portfolio | `controller.save_active_ticker_ids(ids)` |
| 27 | `on_add_custom_coin` | Add a custom coin to the catalog | `id, name, symbol: SharedString` | Returns error or empty; reloads catalog and ticker options | `controller.add_custom_coin(id, name, symbol)` |
| 28 | `on_set_favorite_coin` | Toggle coin favorite status | `id: SharedString, favorite: bool` | Returns error or empty | `controller.set_favorite_coin(id, favorite)` |
| 29 | `on_delete_custom_coin` | Delete a custom coin from catalog | `id: SharedString` | Returns error or empty; reloads catalog and ticker options | `controller.delete_custom_coin(id)` |
| 30 | `on_filter_ticker_options` | Filter ticker options by search query (client-side) | `query: SharedString` | Updates `visible` flag on each ticker option in-place | None (UI-only filtering) |
| 31 | `on_filter_coin_catalog` | Filter coin catalog by search query (client-side) | `query: SharedString` | Updates `visible` flag on each catalog coin in-place | None (UI-only filtering) |
| 32 | `on_select_all_coins` | Select all visible coins in catalog | None | Sets `selected = true` on all visible coins | None (UI-only) |
| 33 | `on_clear_coin_selection` | Clear all coin selections | None | Sets `selected = false` on all coins | None (UI-only) |
| 34 | `on_delete_selected_coins` | Bulk-delete selected custom coins | None (reads selection from model) | Returns error or empty; deletes each selected coin; reloads catalog/tickers | `controller.delete_custom_coin(id)` per selected coin |

### Tax (`crypto/tax.rs`)

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 35 | `on_load_tax_settings` | Load tax configuration and IPC summary | None | Loads tax period settings (jurisdiction, method, swaps, fees); populates wallet exclusion list; loads IPC summary | `controller.load_tax_settings()`, `controller.get_wallets()`, `controller.get_ipc_summary()` |
| 36 | `on_import_ipc_csv` | Import IPC (Chilean tax indicator) CSV file | None (opens file picker) | Parses and imports IPC data; updates IPC summary on UI | `controller.import_ipc_csv(content)` |
| 37 | `on_copy_ipc_url` | Copy IPC data source URL to clipboard | None | Copies URL from Translations to system clipboard | None (clipboard only) |
| 38 | `on_sync_tax_missing_prices` | Resolve missing prices for tax transactions via historical API | `user_initiated: bool` | Async; fetches historical prices for transactions missing price_per_coin, fee USD, or swap proceeds; updates tax report if prices resolved | `controller.generate_tax_summary()`, `controller.get_crypto_historical_price_usd()` (async), `controller.fill_missing_tax_price_fields()` |
| 39 | `on_generate_tax_report` | Generate capital gains tax report for a period | None (reads tax_period and tax_jurisdiction from adapter) | Generates tax summary; sets report data (events, totals, warnings, readiness items); auto-triggers price sync if auto-fetch enabled | `controller.generate_tax_summary(period)` |
| 40 | `on_export_tax_report` | Export capital gains report as CSV | None (reads period from adapter; opens file save dialog) | Writes CSV file to chosen path | `controller.export_tax_report_csv(period, path)` |
| 41 | `on_export_tax_history` | Export transaction history for tax period as CSV | None (reads period from adapter; opens file save dialog) | Writes CSV file to chosen path | `controller.export_tax_history_csv(period, path)` |
| 42 | `on_toggle_tax_wallet_exclusion` | Toggle wallet inclusion/exclusion from tax calculations | `wallet_id: SharedString` | Toggles wallet in `excluded_wallet_ids`; persists settings; updates UI model in-place | `controller.load_tax_settings()`, `controller.save_tax_settings()` |
| 43 | `on_save_tax_settings` | Save tax settings (jurisdiction, method, flags) | None (reads from adapter fields) | Persists tax period settings preserving wallet exclusions | `controller.save_tax_settings(settings)` |

---

## 6. Ingestion Domain

**Adapter:** `IngestionAdapter`
**File:** `src/ui/callbacks/ingestion.rs`

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 1 | `on_import_data` | Open file picker for generic data import (JSON/CSV/TXT) | None (opens file picker) | Reads file; previews import; shows preview modal with summary | `controller.preview_data(content, filename)` |
| 2 | `on_confirm_import` | Confirm and execute generic data import | None (uses pending import state) | Imports data; shows results modal; refreshes crypto views | `controller.import_data(content, filename)` |
| 3 | `on_cancel_preview` | Cancel generic import preview | None | Clears pending import and UI state | None |
| 4 | `on_reset_results` | Clear import results display | None | Clears import summary UI | None |
| 5 | `on_import_exchange_csv` | Open file picker for exchange CSV import (multi-file, auto-detect format) | None (opens multi-file picker for CSV) | Reads files; auto-detects exchange format (Kraken, Binance, MEXC, NotBank, Feather, Monero GUI); applies batch filters; shows wallet selection modal | `controller.detect_exchange_source(content)` |
| 6 | `on_continue_exchange_with_wallet` | Continue exchange import after wallet selection | None (reads wallet name from adapter) | Builds exchange preview summary; shows import preview modal | Internal preview building |
| 7 | `on_cancel_exchange_wallet_select` | Cancel exchange wallet selection | None | Clears pending exchange state and UI flags | None |
| 8 | `on_confirm_exchange_import` | Confirm and execute exchange CSV import | None (uses pending exchange state) | Imports exchange transactions; shows results; refreshes crypto views; clears exchange state | Internal import building using controller |
| 9 | `on_cancel_exchange_preview` | Cancel exchange import preview | None | Clears pending exchange and UI state | None |
| 10 | `on_exchange_wallet_name_changed` | Update wallet name on pending exchange import | `name: SharedString` | Updates wallet name in pending exchange state | None (state update only) |
| 11 | `on_add_missing_exchange_coin` | Add a missing coin detected during exchange import and retry | `symbol: SharedString` | Adds custom coin to catalog if not found; retries preview/import automatically | `controller.add_custom_coin(id, name, symbol)`, `controller.get_coin_catalog_or_default()` |

---

## 7. Habits Domain

**Adapter:** `HabitAdapter`
**Files:** `src/ui/callbacks/habits/callbacks.rs`

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 1 | `on_load_initial_data` | Initialize habits view with current date | None | Sets current date; loads habits for current month | `reload_habits()` (internal) |
| 2 | `on_fetch_habits` | Load habits for a specific month | `month: i32, year: i32` | Reloads habits for the given month/year | `reload_habits()` (internal) |
| 3 | `on_create_habit` | Create a new habit | `name, desc, color, category: SharedString` | Returns error or empty; reloads habits, heatmap, analytics | `controller.create_habit(name, description, color, category)` |
| 4 | `on_update_habit` | Update an existing habit | `id, name, desc, color, category: SharedString` | Returns error or empty; reloads habits, heatmap, analytics | `controller.update_habit(id, name, description, color, category, false)` |
| 5 | `on_delete_habit` | Delete a habit | `id: SharedString` | Returns error or empty; reloads habits, heatmap, analytics | `controller.delete_habit(id)` |
| 6 | `on_toggle_habit` | Toggle habit completion for a date | `id, date: SharedString` | Reloads habits, heatmap, analytics; checks and unlocks streak reward milestones; refreshes rewards UI | `controller.toggle_habit_completion(id, date)`, `controller.get_streak_rewards_by_habit()`, `controller.check_and_unlock_milestones()` |
| 7 | `on_prev_month` | Navigate to previous month | None | Decrements month; reloads habits | `reload_habits()` (internal) |
| 8 | `on_next_month` | Navigate to next month | None | Increments month; reloads habits | `reload_habits()` (internal) |
| 9 | `on_fetch_heatmap_data` | Load heatmap data for current year | None | Populates heatmap grid data | `reload_heatmap()` (internal) |
| 10 | `on_prev_heatmap_year` | Navigate heatmap to previous year | None | Decrements year; reloads heatmap | `reload_heatmap()` (internal) |
| 11 | `on_next_heatmap_year` | Navigate heatmap to next year | None | Increments year; reloads heatmap | `reload_heatmap()` (internal) |
| 12 | `on_fetch_habit_analytics` | Load habit analytics charts | None | Generates and caches analytics charts | `refresh_habit_analytics()` (internal) |
| 13 | `on_select_habit` | Select a habit to view its summary | `id: SharedString` | Loads habit summary (streaks, completion rate); updates selected habit index | `refresh_habit_summary()` (internal) |
| 14 | `on_find_habit_index` | Find index of a habit by ID | `habit_id: SharedString` | Returns `i32` index or -1 if not found | None (UI model search) |

### Rewards (`habits/rewards.rs`)

**Adapter:** `RewardsAdapter`

| # | Callback | Description | Inputs | Output / Side Effects | Service Call |
|---|----------|-------------|--------|----------------------|--------------|
| 15 | `on_fetch_rewards` | Load all streak rewards with milestones | None | Populates rewards list (deferred 100ms) | `controller.get_streak_rewards()`, `controller.get_milestones()` |
| 16 | `on_fetch_goals` | Load all goals with checkpoints | None | Populates goals list (deferred 100ms) | `controller.get_goals()`, `controller.get_checkpoints()` |
| 17 | `on_fetch_achievements` | Load completed goals as achievements | None | Populates achievements list (deferred 100ms) | `controller.get_achievements()` |
| 18 | `on_create_streak_reward` | Create a new streak reward | `habit_id: SharedString, is_consecutive: bool, target_days: i32, target_total: i32` | Returns reward ID or empty on error; refreshes rewards | `controller.create_streak_reward(habit_id, is_consecutive, target_days, target_total)` |
| 19 | `on_update_streak_reward` | Update streak reward with milestones | `id, habit_id: SharedString, is_consecutive: bool, target_days: i32, target_total: i32` | Returns error or empty; reads milestone data from adapter properties; refreshes rewards | `controller.update_streak_reward_with_milestones(...)` |
| 20 | `on_delete_streak_reward` | Delete a streak reward | `id: SharedString` | Returns error or empty; refreshes rewards | `controller.delete_streak_reward(id)` |
| 21 | `on_add_milestone` | Add a milestone to a streak reward | `reward_id: SharedString, target_days: i32, reward_text: SharedString` | Returns error or empty; refreshes rewards | `controller.add_milestone(reward_id, target_days, text)` |
| 22 | `on_create_goal` | Create a new goal | `name, description, reward_text, deadline: SharedString` | Returns goal ID or empty on error; refreshes goals | `controller.create_goal(name, desc, reward, deadline)` |
| 23 | `on_update_goal` | Update a goal | `id, name, description, reward_text, deadline: SharedString` | Returns error or empty; refreshes goals | `controller.update_goal(id, name, desc, reward, deadline)` |
| 24 | `on_update_goal_with_checkpoints` | Update goal and its checkpoints atomically | `id, name, description, reward_text, deadline: SharedString, checkpoint_count: i32, cp1_id, cp1_text, cp2_id, cp2_text, cp3_id, cp3_text, cp4_id, cp4_text: SharedString` | Returns error or empty; refreshes goals | `controller.update_goal_with_checkpoints(...)` |
| 25 | `on_delete_goal` | Delete a goal | `id: SharedString` | Returns error or empty; refreshes goals | `controller.delete_goal(id)` |
| 26 | `on_complete_goal` | Mark a goal as complete (creates achievement) | `id: SharedString` | Returns error or empty; refreshes goals and achievements | `controller.complete_goal(id)` |
| 27 | `on_archive_goal` | Archive a goal | `id: SharedString` | Returns error or empty; refreshes goals | `controller.archive_goal(id)` |
| 28 | `on_delete_checkpoint` | Delete a checkpoint | `checkpoint_id: SharedString` | Returns error or empty; refreshes goals | `controller.delete_checkpoint(id)` |
| 29 | `on_add_checkpoint` | Add a checkpoint to a goal | `goal_id, description: SharedString` | Returns error or empty; refreshes goals | `controller.add_checkpoint(goal_id, description)` |
| 30 | `on_update_checkpoint` | Update checkpoint description | `checkpoint_id, description: SharedString` | Returns error or empty; refreshes goals | `controller.update_checkpoint(id, description)` |
| 31 | `on_toggle_checkpoint` | Toggle checkpoint completion | `goal_id, checkpoint_id: SharedString` | Returns error or empty; refreshes goals and achievements | `controller.toggle_checkpoint(goal_id, checkpoint_id)` |

---

## 8. Translations

**File:** `src/ui/callbacks/translations.rs`

Not inventoried in detail. This module loads approximately 400 translation keys from `locales/*.ftl` (Fluent format) into the `Translations` Slint global. It exposes two public helper functions used by other callbacks:

- `load_all_translations(ui)` -- Loads all translation strings into the UI
- `change_language(lang)` -- Switches the active i18n locale

---

## Summary

| Domain | Adapter(s) | Callback Count |
|--------|-----------|---------------|
| Vault | VaultAdapter | 4 |
| Settings | SettingsAdapter | 12 |
| Dashboard | DashboardAdapter, AnalyticsAdapter | 3 |
| Finance | AccountAdapter, TransactionAdapter, CategoryAdapter | 17 |
| Crypto | CryptoAdapter | 43 |
| Ingestion | IngestionAdapter | 11 |
| Habits | HabitAdapter, RewardsAdapter | 31 |
| **Total** | | **121** |

### Notes for Tauri Migration

1. **Async callbacks**: `on_refresh_prices`, `on_refresh_prices_silent`, `on_request_historical_price`, and `on_sync_tax_missing_prices` use `tokio::spawn` for async API calls. These will naturally map to async Tauri commands.

2. **UI-only callbacks**: `on_filter_ticker_options`, `on_filter_coin_catalog`, `on_select_all_coins`, `on_clear_coin_selection`, `on_find_habit_index`, `on_cancel_preview`, `on_reset_results`, `on_cancel_exchange_wallet_select`, `on_cancel_exchange_preview`, `on_exchange_wallet_name_changed` do not call the controller. These can be handled entirely in the Svelte frontend.

3. **File dialogs**: Many callbacks use `rfd::FileDialog` (vault export/restore, data import, exchange CSV, wallpaper, tax CSV export). In Tauri, these will use `tauri::dialog` or the `tauri-plugin-dialog`.

4. **Clipboard**: `on_copy_ipc_url` uses the Slint platform clipboard API. In Tauri, use `tauri-plugin-clipboard-manager`.

5. **Timer-based auto-fetch**: The 60-second auto-fetch timer in settings will need to be reimplemented, either as a Rust-side timer in the Tauri backend or as a JS `setInterval` in the frontend.

6. **Return pattern**: Many callbacks return `SharedString` where empty = success and non-empty = error message. This maps naturally to `Result<(), String>` in Tauri commands.

7. **Notification pattern**: The `notify(message, is_error)` closure pattern will become frontend-side toast notifications triggered after Tauri command responses.
