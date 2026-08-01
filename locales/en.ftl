# ==================== Sanctum English Translations ====================
# Base language file - all keys must be defined here first
#
# Fluent syntax: https://projectfluent.org/fluent/guide/
# - Variables: { $variableName }
# - Plurals: { $count -> [one] item *[other] items }

# ==================== Common ====================
app-name = SANCTUM
app-subtitle = Personal Financial Fortress

# Common actions
action-delete = Delete
action-edit = Edit
action-close = Close
action-undo = Undo

# Time
time-today = Today

# ==================== Login Page ====================
login-subtitle = Privacy-first personal vault
login-placeholder-unlock = Enter master password
login-placeholder-create = Create master password
login-unlock = Unlock Vault
login-create = Create Vault
login-confirm-create = Confirm Create
login-authenticating = Authenticating...
login-initializing = Initializing...
login-weak-hint = Press again to confirm with weak password
login-restore = Restore from backup
login-version = Sanctum v0.1.0

# ==================== Sidebar ====================
nav-dashboard = Dashboard
nav-finances = Finances
nav-crypto = Crypto
nav-settings = Settings
nav-lock = Lock
nav-collapse = Collapse
nav-expand = Expand
nav-group-overview = Overview
nav-group-vault = Vault
nav-group-system = System
nav-hide-balances = Hide Balances
nav-show-balances = Show Balances
nav-menu = Menu
nav-close = Close

# ==================== Dashboard ====================
dashboard-retry = Retry
dashboard-net-worth = Net Worth
dashboard-fiat = Fiat
dashboard-crypto = Crypto
dashboard-income = Income
dashboard-expenses = Expenses
dashboard-net = Net
dashboard-last = last
dashboard-net-worth-trend = Net Worth Trend
dashboard-monthly-cash-flow = Monthly Cash Flow
dashboard-last-6-months = Last 6 months
dashboard-no-data-range = No data for this range
dashboard-spending-breakdown = Spending Breakdown
dashboard-recent-activity = Recent Activity
dashboard-welcome = Welcome to Sanctum
dashboard-welcome-desc = Add accounts and transactions in the Finances page to see your overview here.

# Tabs & hero
finances-total-balance = Total Balance
finances-tab-overview = Overview
finances-tab-activity = Activity
finances-tab-accounts = Accounts
finances-tab-settings = Settings

# Overview stats
finances-income-this-month = Income this month
finances-expenses-this-month = Expenses this month
finances-net-this-month = Net this month

# Overview charts
finances-monthly-overview = Monthly Overview
finances-balance-distribution = Balance Distribution
finances-no-positive-balances = No positive balances to display
finances-expenses-by-category = Expenses by Category

# Overview accounts section
finances-accounts = Accounts
finances-transfer = Transfer
finances-new-account = New Account
finances-no-accounts = No accounts yet.
finances-recent-transactions = Recent Transactions
finances-view-all = View All
finances-no-transactions = No transactions yet.

# Activity
finances-search-placeholder = Search transactions...
finances-all-accounts = All Accounts
finances-all-categories = All Categories
finances-date-range = Date range
finances-date-all = All time
finances-date-this-month = This month
finances-date-last-30 = Last 30 days
finances-date-last-90 = Last 90 days
finances-date-this-year = This year
finances-date-custom = Custom range
finances-date-from = From
finances-date-to = To
finances-clear = Clear
finances-new-entry = New Entry
finances-no-matching = No matching transactions
finances-no-transactions-yet = No transactions yet
finances-load-more = Load More

# Accounts tab
finances-my-accounts = My Accounts
finances-no-accounts-create = No accounts yet. Create your first account.

# Settings/Categories
finances-new-category = New Category
finances-category-placeholder = Category name...
finances-expense = Expense
finances-expenses = Expenses
finances-income = Income
finances-add = Add

# Detail panel
finances-change-icon = Change Icon
finances-close = Close
finances-type = Type
finances-currency = Currency
finances-balance = Balance
finances-edit-account = Edit Account
finances-delete-account = Delete Account

# Transaction modal
finances-edit-transaction = Edit Transaction
finances-add-transaction = Add Transaction
finances-account = Account
finances-amount = Amount
finances-category = Category
finances-select = Select...
finances-description = Description
finances-date = Date
finances-cancel = Cancel
finances-update = Update
finances-add-btn = Add

# Account modal
finances-edit-account-modal = Edit Account
finances-new-account-modal = New Account
finances-name = Name
finances-account-name-placeholder = Account name
finances-account-type-bank = Bank
finances-account-type-savings = Savings
finances-account-type-credit = Credit Card
finances-account-type-cash = Cash
finances-account-type-other = Other
finances-initial-balance = Initial Balance
finances-icon = Icon
finances-change = Change
finances-create = Create

# Transfer modal
finances-edit-transfer = Edit Transfer
finances-transfer-funds = Transfer Funds
finances-from = From
finances-to = To
finances-transfer-note = Transfer note
finances-transfer-btn = Transfer

# Toast messages
finances-tx-added = Transaction added
finances-tx-updated = Transaction updated
finances-tx-deleted = Transaction deleted
finances-tx-restored = Transaction restored
finances-acc-created = Account created
finances-acc-updated = Account updated
finances-acc-deleted = Account deleted
finances-acc-restored = Account restored
finances-archived-accounts = Archived Accounts
finances-restore = Restore
finances-duplicate = Duplicate
finances-tf-completed = Transfer completed
finances-tf-updated = Transfer updated
finances-cat-added = Category added
finances-cat-deleted = Category deleted
finances-cat-restored = Category restored

# -- Tabs & Hero --
crypto-tab-portfolio = Portfolio
crypto-tab-wallets = Wallets
crypto-tab-tax = Tax
crypto-tab-activity = Activity
crypto-portfolio-value = Portfolio Value
crypto-last-updated-label = Last updated: {$value}
crypto-welcome = Welcome to Crypto
crypto-welcome-desc = Add wallets and transactions in the Wallets tab to start tracking your portfolio.

# -- Ticker bar --
crypto-no-tickers = No tickers configured
crypto-sync-prices = Sync prices
crypto-configure-ticker = Configure ticker

# -- Portfolio tab --
crypto-new-transaction = New Transaction
crypto-unrealized-pnl = Unrealized P&L
crypto-realized-ytd = Realized YTD
crypto-roi = ROI
crypto-portfolio-trend = Portfolio Trend
crypto-distribution = Distribution
crypto-recent-transactions = Recent Transactions
crypto-no-transactions = No transactions yet.
crypto-search-transactions = Search transactions...
crypto-no-matching = No matching transactions
crypto-load-more = Load More
crypto-no-assets-empty = No assets yet. Create a wallet and add transactions to get started.

# -- Wallets tab --
crypto-wallets-title = Wallets
crypto-add-wallet = Add Wallet
crypto-no-wallets = No wallets yet.
crypto-wallet-assets-one = asset
crypto-wallet-assets-other = assets
crypto-delete-wallet = Delete Wallet

# -- Wallet detail panel --
crypto-click-rename = Click to rename
crypto-holdings = Holdings
crypto-transactions = Transactions
crypto-save = Save
crypto-cancel = Cancel
crypto-saving = Saving...
crypto-edit = Edit
crypto-delete = Delete
crypto-duplicate = Duplicate
crypto-all-wallets = All Wallets
crypto-all-types = All Types
crypto-type-trade = Trade
crypto-type-income = Income
crypto-type-expense = Expense
crypto-type-transfer = Transfer
crypto-toast-tx-duplicated = Transaction duplicated
crypto-change = Change
crypto-close = Close
crypto-wallet-icon = Wallet Icon
crypto-change-icon = Change Icon

# -- Asset detail panel --
crypto-amount = Amount
crypto-value = Value
crypto-allocation = Allocation

# -- Tax tab --
crypto-tax-period-id = Tax Period ID
crypto-tax-period-placeholder = e.g., 2024
crypto-tax-load-settings = Load Settings
crypto-tax-jurisdiction = Jurisdiction
crypto-tax-method = Method
crypto-tax-generate-report = Generate Report

# -- Tax report --
crypto-tax-report-summary = Report Summary
crypto-tax-disposals = Disposals
crypto-tax-total-proceeds = Total Proceeds
crypto-tax-total-cost = Total Cost
crypto-tax-total-gain = Total Gain
crypto-tax-short-term = Short-term Gain
crypto-tax-long-term = Long-term Gain
crypto-tax-warnings = Warnings
crypto-tax-readiness = Readiness
crypto-tax-events = Events (showing first 50)
crypto-tax-col-date = Date
crypto-tax-col-coin = Coin
crypto-tax-col-amount = Amount
crypto-tax-col-proceeds = Proceeds
crypto-tax-col-cost-basis = Cost Basis
crypto-tax-col-gain = Gain
crypto-tax-col-term = Term
crypto-tax-export-events = Export Events CSV
crypto-tax-export-history = Export History CSV

# -- IPC import --
crypto-ipc-label = IPC Price History
crypto-ipc-no-data = No IPC data imported
crypto-ipc-import = Import IPC CSV
crypto-ipc-desc = Download the official IPC series, convert it to CSV, and import it here. No network requests are made by default.

# -- Tax settings modal --
crypto-tax-settings-title = Tax Settings
crypto-tax-jurisdiction-us = United States
crypto-tax-jurisdiction-cl = Chile
crypto-tax-jurisdiction-other = Other
crypto-tax-method-fifo = FIFO
crypto-tax-method-lifo = LIFO
crypto-tax-method-hifo = HIFO
crypto-tax-method-avg = Average Cost
crypto-tax-method-chile-hint = Chile (SII) only accepts FIFO and Average Cost.
crypto-tax-method-usa-hint = USA accepts FIFO and Specific ID (LIFO/HIFO); average cost is not allowed for crypto.
crypto-tax-include-swaps-label = Include Swaps in Disposals
crypto-tax-include-fee-label = Include Fee Crypto as Disposal
crypto-tax-exclude-wallets = Exclude Wallets
crypto-tax-loading-settings = Loading…
crypto-tax-regenerate = Regenerate
crypto-tax-onboarding-title = Tax Reporting
crypto-tax-onboarding-desc = Generate a tax report for your crypto transactions. Follow the steps below to get started.
crypto-tax-step1-title = Enter tax year
crypto-tax-step1-desc = Type the year to report (e.g. 2024) and load your settings.
crypto-tax-step2-title = Configure jurisdiction & method
crypto-tax-step2-desc = Select your tax jurisdiction, cost basis method, and optional settings. For Chile, import IPC data.
crypto-tax-step3-title = Generate & export
crypto-tax-step3-desc = Generate the report, review warnings, fix missing prices, and export CSV for your filing.
crypto-tax-chile-info-title = Chile Tax Notes
crypto-tax-chile-ipc = IPC adjustments (correccion monetaria) are applied to cost basis and gains automatically.
crypto-tax-chile-clp = All values in this report are shown in Chilean Pesos (CLP). For filing, use the Dolar Observado published by SII.
crypto-tax-chile-f22 = File under Formulario 22, Línea 10, code 1032 (mayor valor, other income). Verify current codes in the SII suplemento tributario.
crypto-tax-chile-exemption = Net annual income under 13.5 UTA (~$11.3M CLP in 2026) is exempt from IGC.
crypto-tax-chile-fees = Fees and commissions treatment may vary. Consult a Chilean tax professional for your specific situation.
crypto-tax-beta-badge = Beta
crypto-tax-disclaimer = Tax reporting is experimental. These figures are estimates, not tax advice — verify the results with a qualified tax professional before filing.
crypto-tax-exclude-wallets-desc = Wallets you exclude are left out of tax calculations (e.g. DeFi play wallets or donation-only wallets).
crypto-tax-no-wallets = No wallets to exclude.
crypto-tax-excluded-suffix = wallet(s) excluded
crypto-tax-saved = Saved
crypto-tax-report-stale = Settings changed since this report was generated. Regenerate to apply them.
crypto-tax-no-disposals = No taxable disposals in this period. Income, transfers and unsold holdings do not produce a gain until sold or swapped.
crypto-tax-export-title = Export Tax CSV
crypto-tax-taxable-income = Taxable Income
crypto-tax-end-balance = End-of-period Balance
crypto-tax-tx-in-period = Transactions in Period
crypto-tax-volume = Volume Processed
crypto-tax-fetch-price = Fetch price
crypto-tax-fetching = Fetching…
crypto-tax-toast-price-filled = Price filled successfully

# -- Add wallet modal --
crypto-new-wallet = New Wallet
crypto-wallet-name = Name
crypto-wallet-name-placeholder = Wallet name
crypto-wallet-category = Category
crypto-wallet-create = Create

# -- Ticker config modal --
crypto-ticker-tab = Ticker
crypto-coins-tab = Coins
crypto-ticker-active = Active — use arrows to reorder
crypto-ticker-no-selected = No tickers selected yet.
crypto-ticker-add-coins = Add coins
crypto-ticker-search = Search coins...
crypto-ticker-save = Save
crypto-fx-stale = Rate is out of date — sync to refresh it

# -- Coin catalog --
crypto-custom-coin = Add Custom Coin
crypto-custom-id = ID
crypto-custom-name = Name
crypto-custom-symbol = Symbol
crypto-custom-add = Add

# -- Transaction modal --
crypto-tx-title = New Transaction
crypto-tx-buy = Buy
crypto-tx-sell = Sell
crypto-tx-income = Income
crypto-tx-fee = Fee
crypto-tx-transfer = Transfer
crypto-tx-swap = Swap
crypto-tx-coin = Coin
crypto-tx-search-coin = Search coin...
crypto-tx-wallet = Wallet
crypto-tx-from-wallet = From Wallet
crypto-tx-to-wallet = To Wallet
crypto-tx-amount = Amount
crypto-tx-received-amount = Received Amount (optional)
crypto-tx-received-placeholder = Same as sent if empty
# Transaction action labels (in list rows)
crypto-tx-received = Received {$detail}
crypto-tx-sent = Sent {$detail}
crypto-tx-transferred = Transferred {$detail}
crypto-tx-sold = Sold {$detail}
crypto-tx-swapped = Swapped {$detail}
crypto-tx-bought = Bought {$detail}
crypto-tx-from-coin = From Coin
crypto-tx-from-amount = From Amount
crypto-tx-to-coin = To Coin
crypto-tx-to-amount = To Amount
crypto-tx-price = Price (per coin)
crypto-tx-fee-label = Fee
crypto-tx-date = Date
crypto-tx-notes = Notes (optional)
crypto-tx-notes-placeholder = Notes...
crypto-tx-add = Add
crypto-tx-edit-title = Edit Transaction
crypto-tx-subtype = Subtype
crypto-tx-fee-coin-id = Fee Coin (optional)
crypto-tx-fee-coin-amount = Fee Coin Amount (optional)
crypto-tx-override-proceeds = Override Proceeds (optional)
crypto-tx-override-cost-basis = Override Cost Basis (optional)

# -- Toast messages --
crypto-toast-ticker-saved = Ticker config saved
crypto-toast-no-coins-sync = No coins to sync. Configure ticker first.
crypto-toast-custom-added = Custom coin added
crypto-toast-custom-deleted = Custom coin deleted
crypto-toast-tx-added = Transaction added
crypto-toast-tx-updated = Transaction updated
crypto-toast-tx-deleted = Transaction deleted
crypto-toast-tx-restored = Transaction restored
crypto-toast-wallet-restored = Wallet restored
crypto-toast-custom-restored = Custom coin restored
crypto-toast-wallet-created = Wallet created
crypto-toast-wallet-deleted = Wallet deleted
crypto-toast-wallet-renamed = Wallet renamed
crypto-toast-ipc-imported = IPC data imported
crypto-toast-settings-saved = Settings saved
crypto-toast-enter-period = Please enter a period ID
crypto-toast-exported = Exported to {$path}

# ==================== Settings ====================
settings-title = Settings

# Section headers
settings-appearance = Appearance
settings-regional = Regional
settings-security = Security
settings-vault-backup = Vault Backup
settings-data-import = Data Import
settings-data-sync = Data Sync
settings-about = About
settings-reset-section = Reset

# Appearance
settings-dark-mode = Dark Mode
settings-dark-mode-desc = Toggle dark/light theme
settings-background = Background
settings-background-desc = Choose the app background style
settings-bg-aurora = Aurora
settings-bg-diamonds = Diamonds
settings-bg-dots = Dots
settings-bg-dragon = Dragon
settings-bg-stars = Stars

# Regional
settings-preferred-currency = Preferred Currency
settings-language = Language

# Security
settings-session-timeout = Session Timeout
settings-session-timeout-desc = Auto-lock after inactivity
settings-timeout-5min = 5 minutes
settings-timeout-15min = 15 minutes
settings-timeout-30min = 30 minutes
settings-timeout-1hour = 1 hour

# Vault Backup
settings-vault-note = Your vault is encrypted with SQLCipher (AES-256).
settings-export-vault = Export Vault
settings-last-backup = Last backup
settings-last-backup-never = never
settings-last-backup-days = {$count} days ago
settings-export-transactions = Export Transactions
settings-export-transactions-desc = Plain CSV of your whole ledger, unencrypted
settings-export-csv-done = {$count} transactions exported
settings-export-btn = Export
settings-export-success = Backup saved successfully

# Data Import
settings-import-generic = Generic CSV
settings-import-generic-desc = Import transactions from a CSV file
settings-import-exchange = Exchange / Wallet CSV
settings-import-exchange-desc = Import from Kraken, Binance, MEXC, Feather, Monero…
settings-import-custom = Custom CSV (manual mapping)
settings-import-custom-desc = Import from any other exchange by mapping its columns
settings-import-custom-intro = Match each Sanctum field to a column from your CSV. Date, asset and amount are required.
settings-import-custom-preview = Column preview (first row)
settings-import-custom-select = — Select column —
settings-import-custom-none = — None —
settings-import-custom-no-wallets = Create a wallet in the Crypto section first.
settings-import-custom-date = Date
settings-import-custom-asset = Asset (coin)
settings-import-custom-amount = Amount
settings-import-custom-type = Type
settings-import-custom-fee = Fee
settings-import-custom-fee-currency = Fee currency
settings-import-custom-price = Price
settings-import-custom-notes = Notes
settings-import-select-file = Select File
settings-import-loading = Loading...
settings-import-detected = Detected:
settings-import-records = records
settings-import-target-wallet = Target Wallet
settings-import-wallet-placeholder = Wallet name
settings-import-wallet-required = Wallet name is required
settings-import-no-detection = Could not detect exchange format
settings-import-preview-btn = Preview
settings-import-to-add = to add
settings-import-to-skip = to skip
settings-import-importing = Importing...
settings-import-confirm = Confirm Import
settings-import-processed = Processed:
settings-import-inserted = Inserted:
settings-import-skipped = Skipped:
settings-import-errors = Errors
settings-import-line = Line
settings-import-done = Done

# Data Sync
settings-auto-fetch = Auto-fetch Prices
settings-auto-fetch-desc = Automatically fetch crypto prices on sync
settings-use-proxy = Use Proxy
settings-use-proxy-desc = Route API calls through a proxy
settings-proxy-url = Proxy URL
settings-proxy-placeholder = socks5://127.0.0.1:9050

# About
settings-about-version = Version
settings-about-encryption = Encryption
settings-about-storage = Storage

# Reset
settings-reset-all = Reset All Settings
settings-reset-all-desc = Restore default values for all settings
settings-reset-btn = Reset
settings-reset-success = Settings reset to defaults

# Common actions
settings-cancel = Cancel

# Confirmation dialogs
confirm-delete-title = Confirm Delete
confirm-delete-message = This action cannot be undone.
confirm-delete-button = Delete
confirm-delete-account = Are you sure you want to delete this account?
confirm-delete-account-tx-count = This will also remove {$count} transaction(s).
confirm-delete-category = Are you sure you want to delete this category?
confirm-delete-transaction = Are you sure you want to delete this transaction?
confirm-reset-settings = Reset all settings to their default values? This will not affect your data.

# ==================== Data Import ====================

import-errors = Errors

import-error-account-not-found = Account not found: { $name }
import-error-currency-mismatch-detail = Currency mismatch: import has { $import } but account { $account } uses { $expected }
import-error-category-not-found-detail = Category not found: { $name } (type: { $type })
import-error-destination-account-not-found = Destination account not found: { $name }
import-error-same-account-transfer = Cannot transfer to the same account
import-error-wallet-not-found = Wallet not found: { $name }
import-error-insufficient-crypto-balance = Insufficient { $symbol } balance in { $wallet }: have { $available }, need { $required }
import-skipped-duplicate-transaction = Duplicate transaction (same date/account/amount/type/description)
import-skipped-duplicate-crypto = Duplicate crypto transaction (same date/wallet/coin/type/amount)
import-skipped-crypto-not-found = Crypto asset not found in catalog (row ignored)

import-preview-change-income = Income
import-preview-change-expense = Expense
import-preview-change-transfer = Transfer
import-preview-change-crypto = Crypto Transaction

# ==================== Exchange CSV Import ====================
import-exchange-hint-kraken = Kraken: Documents > export Ledgers and Trades CSV files (you can upload both together).
import-exchange-hint-kraken-pro = Kraken Pro: History > Statements > export Ledgers and Trades CSV files (you can upload both together).
import-exchange-hint-binance = Export from Binance: Orders > Transaction History > Generate All Statements
import-exchange-hint-mexc = MEXC: Help Center > Account Data Export > select required reports > convert to CSV. Supports 17 report CSV types; you can upload multiple files at once.
import-exchange-hint-notbank = NotBank (CryptoMarket): Exchange Pro > Reports > Single Report > Transaction and Trade Activity (you can upload both together).
import-exchange-hint-feather = Export from Feather Wallet: History > Export CSV
import-exchange-hint-monero-gui = Export from Monero GUI: Wallet > History > Export CSV
settings-import-exchange-help = How to export from each exchange or wallet
settings-import-wallet-missing = Wallet "{$name}" doesn't exist yet. Create it?
settings-import-create-wallet-preview = Create wallet & preview

# ==================== Session Lock Warning ====================
session-warning-title = Vault about to lock
session-warning-body = Locking in {$seconds}s due to inactivity.
session-warning-stay = Stay unlocked
session-warning-lock-now = Lock now
session-locked = Vault locked

# ==================== Master Password ====================
settings-change-password = Master Password
settings-change-password-desc = Re-encrypts the whole vault with a new password
settings-change-password-btn = Change
settings-password-current = Current password
settings-password-new = New password
settings-password-confirm = Confirm new password
settings-password-mismatch = The new passwords do not match
settings-password-changing = Re-encrypting…
settings-password-changed = Master password changed
settings-password-rollback-at = Rollback copy, still using the old password:
settings-password-backup-warning = A backup is saved first and is required. That backup keeps the OLD password: restoring it later needs the password you are replacing now.

# interface language. Categories the user creates are shown verbatim instead.
category-food = Food
category-transport = Transport
category-utilities = Utilities
category-entertainment = Entertainment
category-health = Health
category-shopping = Shopping
category-education = Education
category-other = Other
category-salary = Salary
category-freelance = Freelance
category-investment = Investment
category-gift = Gift
category-transfer = Transfer

# ==================== Recurring Entries ====================
finances-recurring = Recurring Entries
finances-recurring-desc = Created automatically on their date. Opening the app after a while fills in everything it owes.
finances-recurring-new = New
finances-recurring-frequency = Frequency
finances-recurring-weekly = Weekly
finances-recurring-monthly = Monthly
finances-recurring-yearly = Yearly
finances-recurring-first = First occurrence
finances-recurring-next = Next
finances-recurring-paused = Paused
finances-recurring-pause = Pause
finances-recurring-resume = Resume
finances-recurring-added = Recurring entry saved
finances-recurring-deleted = Recurring entry deleted
finances-recurring-delete-confirm = Delete this recurring entry? Transactions it already created are kept.
finances-recurring-applied = {$count} recurring transactions added

# ==================== Monthly Budgets ====================
finances-budgets = Monthly Budgets
finances-budgets-desc = A spending limit per category. Progress covers the current month and resets on the 1st.
finances-budget-new = New
finances-budget-saved = Budget saved
finances-budget-left = left
finances-budget-over = Over by
finances-no-budgets = No budgets yet.

# ==================== Activity Sorting ====================
finances-sort = Sort by
finances-sort-date-desc = Newest first
finances-sort-date-asc = Oldest first
finances-sort-amount-desc = Largest amount
finances-sort-amount-asc = Smallest amount

# ==================== Entry Form ====================
finances-date-today = Today
finances-date-yesterday = Yesterday

# ==================== Bulk Actions ====================
finances-select-row = Select transaction
finances-bulk-selected = {$count} selected
finances-bulk-move = Move to category
finances-bulk-deleted = {$count} transactions deleted
finances-bulk-moved = {$count} transactions moved
finances-bulk-move-undone = Categories restored
finances-bulk-restored = {$count} transactions restored
finances-bulk-restored-partial = Restored {$count} of {$total} transactions
confirm-delete-transactions = Delete {$count} transactions?

# ==================== Command Palette ====================
palette-title = Command palette
palette-placeholder = Type a command, or search your data...
palette-no-results = Nothing found
search-kind-account = Account
search-kind-category = Category
search-kind-coin = Coin
search-kind-transaction = Transaction
search-kind-wallet = Wallet

# ==================== Keyboard Shortcuts ====================
shortcuts-title = Keyboard Shortcuts
shortcuts-group-navigation = Navigation
shortcuts-group-actions = Actions
shortcuts-group-dialogs = Dialogs
shortcuts-toggle-sidebar = Collapse or expand the sidebar
shortcuts-new-entry = New entry on the current page
shortcuts-search = Jump to the search box
shortcuts-lock = Lock the vault now
shortcuts-confirm = Confirm the open form
shortcuts-close = Close without saving
shortcuts-help = Show this list
shortcuts-palette = Open the command palette
