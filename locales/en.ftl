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
action-save = Save
action-cancel = Cancel
action-delete = Delete
action-edit = Edit
action-create = Create
action-add = Add
action-close = Close
action-undo = Undo
action-confirm = Confirm
action-back = Back
action-next = Next
action-submit = Submit
action-archive = Archive
action-restore = Restore
action-clear = Clear

# Common labels
label-name = Name
label-description = Description
label-amount = Amount
label-date = Date
label-category = Category
label-type = Type
label-status = Status
label-balance = Balance
label-total = Total
label-notes = Notes
label-color = Color
label-icon = Icon
label-currency = Currency
label-search = Search
label-filter = Filter
label-loading = Loading...
label-none = None
label-all = All
label-yes = Yes
label-no = No

# Time
time-today = Today
time-yesterday = Yesterday
time-days-ago = { $count } days ago
time-week = Week
time-month = Month
time-year = Year

# Validation
validation-required = This field is required
validation-invalid-amount = Invalid amount
validation-invalid-date = Invalid date

# ==================== Login Page ====================
login-title = SANCTUM
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
dashboard-title = Dashboard
dashboard-loading = Loading dashboard...
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

# ==================== Finances ====================
# Tabs & hero
finances-total-balance = Total Balance
finances-tab-overview = Overview
finances-tab-activity = Activity
finances-tab-accounts = Accounts
finances-tab-settings = Settings
finances-loading = Loading...

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

# ==================== Crypto ====================
# -- Tabs & Hero --
crypto-tab-portfolio = Portfolio
crypto-tab-wallets = Wallets
crypto-tab-tax = Tax
crypto-tab-activity = Activity
crypto-portfolio-value = Portfolio Value
crypto-last-updated-label = Last updated: {$value}
crypto-loading = Loading...
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
crypto-tax-configure = Configure
crypto-tax-jurisdiction = Jurisdiction
crypto-tax-method = Method
crypto-tax-include-swaps = Include Swaps
crypto-tax-include-fee-crypto = Include Fee Crypto
crypto-tax-yes = Yes
crypto-tax-no = No
crypto-tax-generate-report = Generate Report
crypto-tax-processing = Processing tax data...

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
crypto-ipc-records = {$count} records {$range}
crypto-ipc-no-data = No IPC data imported
crypto-ipc-import = Import IPC CSV
crypto-ipc-desc = Download the official IPC series, convert it to CSV, and import it here. No network requests are made by default.

# -- Tax settings modal --
crypto-tax-settings-title = Tax Settings
crypto-tax-jurisdiction-us = United States
crypto-tax-jurisdiction-cl = Chile
crypto-tax-jurisdiction-ca = Canada
crypto-tax-jurisdiction-uk = United Kingdom
crypto-tax-jurisdiction-au = Australia
crypto-tax-jurisdiction-other = Other
crypto-tax-method-fifo = FIFO
crypto-tax-method-lifo = LIFO
crypto-tax-method-hifo = HIFO
crypto-tax-method-avg = Average Cost
crypto-tax-method-chile-hint = Chile (SII) only accepts FIFO and Average Cost.
crypto-tax-method-usa-hint = USA accepts FIFO and Specific ID (LIFO/HIFO); average cost is not allowed for crypto.
crypto-tax-cost-basis-method = Cost Basis Method
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
crypto-tax-regenerate = Regenerate
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
settings-timeout-never = Never

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
settings-import-source = Source:
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
settings-import-file-too-large = File too large

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

# ==================== Modals ====================
modal-add-account-title = Add Account
modal-edit-account-title = Edit Account
modal-add-transaction-title = Add Transaction
modal-edit-transaction-title = Edit Transaction
modal-transfer-title = Transfer Funds
modal-add-wallet-title = Add Wallet
modal-edit-wallet-title = Edit Wallet

# Confirmation dialogs
confirm-delete-title = Confirm Delete
confirm-delete-message = This action cannot be undone.
confirm-delete-button = Delete
confirm-delete-account = Are you sure you want to delete this account?
confirm-delete-account-tx-count = This will also remove {$count} transaction(s).
confirm-delete-category = Are you sure you want to delete this category?
confirm-delete-transaction = Are you sure you want to delete this transaction?
confirm-delete-wallet = Are you sure you want to delete this wallet?
confirm-reset-settings = Reset all settings to their default values? This will not affect your data.

# ==================== Notifications ====================
notify-success = Success
notify-error = Error
notify-saved = Changes saved
notify-deleted = Item deleted
notify-created = Item created
notify-updated = Item updated

# ==================== Empty States ====================
empty-no-data = No data available
empty-add-first = Add your first { $item } to get started
empty-no-results = No results found
empty-try-different = Try a different search or filter

# ==================== Errors ====================
error-generic = Something went wrong
error-connection = Connection error
error-invalid-input = Invalid input
error-not-found = Not found
error-unauthorized = Unauthorized access

# ==================== Misc UI Text ====================
bank-icons-title = BANK ICONS
no-expenses-recorded = No expenses recorded
fee-label = Fee
empty-no-transactions-account = No transactions for this account
crypto-total-holdings = TOTAL HOLDINGS
crypto-no-wallet-data = No wallet data available
crypto-no-transactions-found = No transactions found
crypto-portfolio-distribution = PORTFOLIO DISTRIBUTION
confirm-delete-generic = This will permanently delete

# ==================== Dashboard Extended ====================
dashboard-total-net-worth = TOTAL NET WORTH
dashboard-exchange-rate-warning = Exchange rate unavailable for some currencies. Balances shown with fallback 1:1 rate.
dashboard-usd-clp = USD/CLP

# ==================== Finances Extended ====================
finances-configure = CONFIGURE
finances-transaction-categories = Transaction Categories
finances-manage-categories = Manage income and expense categories
finances-delete-transaction = Delete Transaction
finances-delete-confirm = Are you sure you want to delete

# ==================== Crypto Extended ====================
crypto-portfolio-title = CRYPTO PORTFOLIO
crypto-last-updated = Last updated
crypto-last-updated-info = Last updated: {$value}
crypto-last-updated-never = Never
crypto-last-updated-today-at = Today at {$time}
crypto-coin-limit = Coin limit reached (50). Some assets may not update.
crypto-skipped = Skipped
crypto-your-holdings = YOUR HOLDINGS
crypto-no-assets-yet = No assets tracked yet
crypto-create-wallet-first = Create a wallet first, then add your crypto holdings
crypto-start-adding = Start by adding a wallet and your first asset
crypto-import-csv = IMPORT CSV
crypto-unrealized = UNREALIZED
crypto-tax-title = TAX & REPORTS
crypto-tax-subtab-settings = SETTINGS
crypto-tax-subtab-summary = SUMMARY
crypto-tax-period-label = TAX PERIOD (YEAR)
crypto-tax-jurisdiction-label = JURISDICTION
crypto-tax-method-label = COST BASIS METHOD
crypto-tax-include-swaps-desc = Treat swaps as disposals for tax reports.
crypto-tax-include-fee-crypto-desc = Treat fee coin as a taxable disposal.
crypto-tax-save-settings = Save Tax Settings
crypto-tax-report-title = REPORT GENERATION
crypto-tax-report-desc = Generate a tax report for the selected period. Exports are local CSV files.
crypto-tax-report-generate = Generate Report
crypto-tax-report-export = Export CSV
crypto-tax-report-summary-label = REPORT SUMMARY
crypto-tax-report-summary-empty = No report generated yet
crypto-tax-report-summary-us = Disposals: {$disposals} | Proceeds: {$proceeds} | Cost: {$cost} | Gain: {$gain} | Short: {$short} | Long: {$long}
crypto-tax-report-warnings-label = WARNINGS
crypto-tax-report-warnings-empty = No warnings
crypto-tax-report-warnings-count = Warnings: {$count} (see CSV)
crypto-tax-report-generated = Report generated
crypto-tax-report-exported = Report exported
crypto-tax-summary-title = TAX SUMMARY
crypto-tax-summary-empty = Generate a report to see your summary
crypto-tax-summary-capital = CAPITAL GAINS
crypto-tax-summary-income = TAXABLE INCOME
crypto-tax-summary-balance = END OF YEAR BALANCE
crypto-tax-summary-proceeds = Proceeds
crypto-tax-summary-cost = Cost basis
crypto-tax-summary-gain = Gain / Loss
crypto-tax-summary-income-total = Total income
crypto-tax-summary-balance-total = Total value
crypto-tax-summary-reports = TAX REPORTS
crypto-tax-summary-export-history = Export Transaction History
crypto-tax-summary-export-capital = Export Capital Gains
crypto-tax-summary-simulation = SIMULATION
crypto-tax-summary-transactions = Transactions computed
crypto-tax-summary-volume = Volume processed
crypto-tax-summary-short-term = SHORT TERM
crypto-tax-summary-long-term = LONG TERM
crypto-tax-summary-disposals = Disposals
crypto-tax-readiness-banner = issues to review before filing
crypto-tax-empty-title = No report generated
crypto-tax-empty-desc = Configure your settings and generate a report to see your tax summary
crypto-tax-exports-title = EXPORT REPORTS
crypto-tax-exports-capital-desc = Detailed disposals with lot allocations
crypto-tax-exports-history-desc = All transactions with tax classifications
crypto-tax-report-details = REPORT DETAILS
crypto-tax-settings-advanced = ADVANCED OPTIONS
crypto-tax-wallet-exclusions = WALLET EXCLUSIONS
crypto-tax-wallet-exclusions-desc = Exclude wallets from tax calculations. Transactions in excluded wallets will not appear in reports.
crypto-tax-wallet-none = No wallets found
crypto-tax-wallet-excluded-label = excluded
crypto-tax-filing-title = FILING GUIDE
crypto-tax-save-generate = Save & Generate
crypto-tax-readiness-settings-suffix = transactions in period
crypto-tax-readiness-settings-warn-detail = No transactions found in the computed commercial period. In Chile, the Tax Year uses transactions from the previous commercial year (example: Tax Year 2026 uses 2025 transactions). Review period selection, wallet exclusions, and imported files.
crypto-tax-readiness-settings-excluded-warn-suffix = transactions are excluded by wallet filters. Review Wallet Exclusions in Settings.
crypto-tax-readiness-history-warn-suffix = disposals have insufficient lots. Reimport missing history or fix transaction classification.
crypto-tax-readiness-prices-invalid-suffix = Invalid dates or types found
crypto-tax-readiness-prices-warn-suffix = items are missing tax prices. Use Resolve missing prices and regenerate.
crypto-tax-readiness-prices-fx-warn-suffix = items use non-USD quote pricing. Add historical FX references for those dates.
crypto-tax-readiness-transfers-warn-suffix = transfers are unpaired. Link deposit/withdrawal counterparts.
crypto-tax-readiness-balances-warn-suffix = assets in end-of-year balance are missing current prices. Sync prices and regenerate.
crypto-tax-readiness-sii-gain-detail = Gain -> F22 Line 10, Casilla 1032. Warning: Casilla codes may change each year.
crypto-tax-readiness-sii-loss-detail = Loss -> F22 Line 17, Casilla 169 (capped). Warning: Casilla codes may change each year.
crypto-tax-readiness-sii-neutral-detail = No net gain or loss. Warning: F22 casilla codes may change each year.
crypto-tax-readiness-usa-filing-detail = Report on Form 8949 + Schedule D.
crypto-tax-readiness-other-filing-detail = Warning: Review your country's specific crypto tax legislation -- rules vary significantly between jurisdictions. This report uses standard international rules (fees in cost basis, FMV for income, short/long term at 365 days). Consult a local tax advisor before filing.
crypto-tax-readiness-title = READINESS CHECK
crypto-tax-readiness-desc = Review issues before filing
crypto-tax-readiness-settings = Review settings
crypto-tax-readiness-settings-count = {$count} transactions in period
crypto-tax-readiness-history = Check history coverage
crypto-tax-readiness-history-warn = {$count} disposals with insufficient lots
crypto-tax-readiness-balances = Check balances
crypto-tax-readiness-balances-warn = Some disposals exceed available lot quantity
crypto-tax-readiness-prices = Resolve missing prices
crypto-tax-readiness-prices-fx = Normalize FX-priced trades
crypto-tax-readiness-prices-invalid = Invalid dates or transaction types found
crypto-tax-readiness-prices-warn = {$count} items missing price data
crypto-tax-readiness-prices-fx-warn = {$count} items use non-USD quote pricing
crypto-tax-price-sync-running = Price sync is already running
crypto-tax-price-sync-no-missing = No missing prices to sync
crypto-tax-price-sync-finished = Synced {$count} missing prices
crypto-tax-price-sync-unresolved = {$count} items could not be resolved automatically
crypto-tax-readiness-transfers = Review transfers
crypto-tax-readiness-transfers-warn = {$count} unpaired transfers
crypto-tax-readiness-filing = Filing guidance
crypto-tax-readiness-sii-f22 = SII Form 22
crypto-tax-readiness-sii-gain = Gain -> F22 Line 10, Casilla 1032. Warning: Casilla codes may change each year.
crypto-tax-readiness-sii-loss = Loss -> F22 Line 17, Casilla 169 (capped at casillas 105+155+152+1032+1891+1104). Warning: Casilla codes may change each year.
crypto-tax-readiness-sii-neutral = No net gain or loss. Warning: F22 casilla codes may change each year.
crypto-tax-readiness-usa-filing = USA: Report on Form 8949 + Schedule D.
crypto-tax-readiness-other-filing = Warning: Your country's tax legislation may differ. Consult a local tax advisor before filing.
crypto-tax-sii-casilla-warning = F22 casilla codes may change each tax year. Always verify against the current SII supplementary instructions.
crypto-tax-ipc-title = IPC (Chile)
crypto-tax-ipc-desc = Download the official IPC series, convert it to CSV, and import it here. No network requests are made by default.
crypto-tax-ipc-source-label = OFFICIAL SOURCE (MANUAL DOWNLOAD)
crypto-tax-ipc-source-url = https://www.ine.gob.cl/docs/default-source/%C3%ADndice-de-precios-al-consumidor/cuadros-estadisticos/series-empalmadas-y-antecedentes-historicos/series-empalmadas-diciembre-2009-a-la-fecha/serie-hist%C3%B3rica-empalmada-ipc-diciembre-2009-a-la-fecha-xls.xlsx
crypto-tax-ipc-import = Import IPC (CSV)
crypto-tax-ipc-copy-url = Copy URL
crypto-tax-ipc-summary-label = IPC STATUS
crypto-tax-ipc-empty = No IPC data loaded
crypto-tax-ipc-summary = Loaded: {$first} -> {$last} ({$count} months)
crypto-tax-ipc-import-success = IPC imported: {$count} months ({$first} -> {$last})
crypto-tax-settings-saved = Tax settings saved
crypto-tax-period-required = Tax period is required
crypto-tax-advanced = TAX CLASSIFICATION (OPTIONAL)
modal-transaction-type = TAX CATEGORY
modal-transaction-type-placeholder = trade / income / expense / transfer
modal-transaction-subtype = TAX SUBTYPE
modal-transaction-subtype-placeholder = airdrop / staking / fee / other
modal-tax-override-proceeds = OVERRIDE PROCEEDS
modal-tax-override-cost = OVERRIDE COST BASIS
crypto-assets-across-wallets = { $assets } assets across { $wallets } wallets
crypto-wallet = WALLET

crypto-add-first-wallet = Add your first wallet to start tracking your crypto
crypto-no-wallets-created = No wallets created
crypto-delete-wallet-confirm-prefix = This will permanently delete "
crypto-delete-wallet-confirm-suffix = " and all its transaction history.
crypto-delete-wallet-warning-title = Wallet Has Transactions
crypto-delete-wallet-warning-prefix = This wallet contains 
crypto-delete-wallet-warning-suffix =  transaction(s). Deleting it will permanently remove all of them.
crypto-delete-wallet-force = Delete Anyway
crypto-loading-portfolio = Loading portfolio...
crypto-syncing-prices = Syncing prices...
crypto-syncing-wait = This may take a few seconds

# ==================== Settings Extended ====================
settings-configure-experience = Configure your Sanctum experience
settings-proxy-tip = Tip: socks5h:// routes DNS through the proxy for better privacy.
settings-data-encrypted = Your data is encrypted locally
settings-military-grade = All sensitive information is protected with military-grade encryption
settings-reset-defaults = RESET TO DEFAULTS

# ==================== Common Actions Extended ====================
action-view-all = VIEW ALL →
action-retry = RETRY
action-load-more = LOAD MORE
action-configure = CONFIGURE
action-transfer = TRANSFER

# ==================== Components ====================
# Account Item
account-balance = Balance

# Crypto Widgets
crypto-holdings-label = Holdings
crypto-price-label = Price

# Crypto Charts
crypto-no-priced-assets = No priced assets yet
crypto-sync-to-see = Sync prices to see the distribution
crypto-value-label = VALUE
crypto-cost-label = COST
crypto-no-trend = No trend data yet
crypto-sync-daily = Sync prices daily to build history

# Wallet Detail
wallet-no-holdings = No holdings in this wallet

# Icon Selector
icon-choose = CHOOSE ICON
icon-exchanges = Exchanges
icon-wallet-icons = Wallet Icons

# Forms
form-search-coin = Search coin...
form-date-format = YYYY-MM-DD
form-all = ALL

# ==================== Modals ====================
# Add Account
modal-delete-account = DELETE ACCOUNT
modal-save-account = SAVE ACCOUNT
modal-delete-account-confirm = This will permanently delete and all its transaction history.

# Add Transaction
modal-no-accounts = No accounts available. Create one first.

# Add Crypto Transaction
modal-new-crypto-transaction = NEW CRYPTO TRANSACTION
modal-save-transaction = SAVE TRANSACTION
modal-create-wallet-first = Create a wallet in the Wallets tab first
modal-create-another-wallet = Create another wallet to move assets

# Edit Crypto Transaction
modal-edit-crypto-transaction = EDIT CRYPTO TRANSACTION
modal-save-changes = SAVE CHANGES

# Add Wallet
modal-new-wallet = NEW WALLET
modal-wallet-type = WALLET TYPE
modal-create-wallet = CREATE WALLET


# Configure Categories
modal-category-settings = CATEGORY SETTINGS
modal-manage-categories = Manage income and expense categories for transactions.
modal-expense-categories = EXPENSE CATEGORIES
modal-income-categories = INCOME CATEGORIES
modal-no-expense-categories = No expense categories
modal-no-income-categories = No income categories
modal-add-new-category = ADD NEW CATEGORY
modal-category-name = Category name
modal-default = DEFAULT

# Configure Ticker
modal-crypto-settings = CRYPTO SETTINGS
modal-manage-price-bar = Manage price bar and coin catalog for selection.
modal-price-bar = PRICE BAR
modal-remove = REMOVE

# Add Transaction
modal-account = ACCOUNT
modal-expense = EXPENSE
modal-income = INCOME

# Configure Ticker
modal-coin-catalog = COIN CATALOG
modal-max-coins = Max 50 active coins for price updates.
modal-catalog-info = Catalog is used for price bar and transactions.
modal-coin-list = COIN LIST
modal-add-coin = ADD COIN
modal-removing-info = Removing coins only hides them here.
modal-select-all = SELECT ALL
modal-remove-selected = REMOVE SELECTED

# Sidebar branding
sidebar-logo = S
sidebar-title = SANCTUM

# Crypto Widgets
crypto-holdings-small = Holdings
crypto-price-small = Price

# ==================== Crypto Transaction Modal ====================
modal-from-asset = FROM ASSET
modal-to-asset = TO ASSET
modal-cryptocurrency = CRYPTOCURRENCY
modal-from-wallet = FROM WALLET
modal-to-wallet = TO WALLET
modal-from-amount = FROM AMOUNT
modal-to-amount = TO AMOUNT
modal-to-amount-optional = TO AMOUNT (optional)
modal-same-as-from = Same as FROM
modal-price-usd = PRICE (USD)
modal-optional = Optional
modal-required = Required
modal-fee-usd = FEE (USD)
modal-fee-coin-optional = FEE COIN (OPTIONAL)
modal-fee-amount = FEE AMOUNT
modal-notes = NOTES
modal-transaction-details = Transaction details...
modal-date = DATE
modal-fetch-price-date = Fetch price for selected date
modal-search-coins = Search coins...

# Section labels (crypto transaction modal)
section-asset-wallet = ASSET & WALLET
section-amount = AMOUNT
section-advanced = ADVANCED
section-details = DETAILS
section-fee-crypto = FEES IN CRYPTO
section-tax = TAX CLASSIFICATION

# Transaction summary
tx-summary-buying = Buying
tx-summary-selling = Selling
tx-summary-swapping = Swapping
tx-summary-moving = Moving
tx-summary-receiving = Receiving
tx-summary-sending = Sending
tx-summary-at = at
tx-summary-per-coin = /coin
tx-summary-to = to
tx-summary-from = from

# Category tabs (type selector)
tx-category-trade = TRADE
tx-category-transfer = TRANSFER
tx-category-income = INCOME
tx-category-expense = EXPENSE

# Scenario labels (type selector sub-options)
tx-scenario-deposit = Deposit
tx-scenario-withdrawal = Withdrawal
tx-scenario-interest = Interest
tx-scenario-gift = Gift
tx-scenario-reward = Reward
tx-scenario-other = Other
tx-scenario-payment = Payment
tx-scenario-donation = Donation
tx-scenario-fee = Fee
tx-scenario-lost = Lost
tx-scenario-stolen = Stolen
tx-scenario-buy = BUY
tx-scenario-sell = SELL
tx-scenario-swap = SWAP
tx-scenario-move = MOVE
tx-scenario-airdrop = Airdrop
tx-scenario-staking = Staking
tx-scenario-mining = Mining
tx-scenario-fork = Fork

# Type badge labels (edit modal)
tx-type-buy = BUY
tx-type-sell = SELL
tx-type-swap = SWAP
tx-type-transfer-in = TRANSFER IN
tx-type-transfer-out = TRANSFER OUT

# ==================== Configure Ticker Extended ====================
modal-add-custom-coin = ADD CUSTOM COIN
modal-coingecko-hint = Use CoinGecko ID (lowercase, hyphens). Example: litecoin
modal-symbol-hint = Symbol uses letters only. Example: LTC
modal-coingecko-id = COINGECKO ID
modal-coingecko-id-placeholder = e.g. litecoin
modal-name-placeholder = e.g. Litecoin
modal-symbol = SYMBOL
modal-symbol-placeholder = e.g. LTC
modal-save-configuration = SAVE CONFIGURATION

# ==================== Wallet Modal ====================
modal-wallet-name = WALLET NAME

# ==================== Transfer Modal ====================
modal-edit-transfer = EDIT TRANSFER
modal-from = FROM
modal-to = TO
modal-transfer-action = TRANSFER

# ==================== Icon Modals ====================
modal-select-bank-icon = SELECT BANK ICON
modal-select-icon = SELECT ICON
modal-save-icon = SAVE ICON

# ==================== Common Button Labels ====================
button-add = ADD
button-sync = ↻ SYNC
button-syncing = SYNCING...
button-add-transaction = + ADD TRANSACTION
button-add-transaction-short = + ADD TRANSACTION
button-new-entry = + NEW ENTRY
button-new-account = + NEW ACCOUNT

# ==================== Page Titles and Sections ====================
section-fiat = FIAT
section-spending-breakdown = SPENDING BREAKDOWN
section-recent-activity = RECENT ACTIVITY
section-recent-transactions = RECENT TRANSACTIONS
section-my-accounts = MY ACCOUNTS
section-finance-settings = FINANCE SETTINGS
section-transactions = TRANSACTIONS
section-wallet-breakdown = WALLET BREAKDOWN

# ==================== Settings Page ====================
section-regional = REGIONAL
section-data-sync = DATA & SYNC
section-about = ABOUT
settings-version-label = Version
settings-encryption-label = Encryption
settings-database-label = Database
settings-encryption-type = AES-256-GCM
settings-storage-type = SQLite (Encrypted)

# ==================== Vault Backup ====================
vault-backup-section = VAULT BACKUP
vault-export-button = EXPORT BACKUP
vault-restore-button = RESTORE BACKUP
vault-restore-from-backup = Restore from backup...

vault-export-success = Vault backup created successfully
vault-restore-success = Vault restored successfully. Please log in.
vault-export-failed = Failed to export vault
vault-restore-failed = Failed to restore vault

vault-restore-warning-title = Restore Vault from Backup?
vault-restore-warning-desc = This will replace your current vault with the backup file. All current data will be overwritten. This action cannot be undone.
vault-restore-file-label = BACKUP FILE
vault-restore-cancel = CANCEL
vault-restore-confirm = RESTORE VAULT

vault-invalid-backup = Invalid backup file
vault-backup-too-large = Backup file is too large (max 1GB)
vault-permission-denied = Permission denied accessing file
vault-backup-encryption-note = Backups maintain full encryption. Never export to untrusted locations.

# ==================== Asset/Wallet Details ====================
section-transaction-history = TRANSACTION HISTORY

# ==================== Transaction Entry Modal ====================
modal-new-entry = NEW ENTRY
modal-edit-entry = EDIT ENTRY
modal-save-entry = SAVE ENTRY
modal-add-note = Add a note...

# ==================== Finances Extended (Search/Empty States) ====================
finances-try-adjusting = Try clearing or adjusting your filters
finances-add-first-entry = Add your first entry to start tracking your finances
finances-no-accounts-configured = No accounts configured
finances-create-account = Create an account to manage your funds

# ==================== Crypto Extended (Buttons) ====================
crypto-add-wallet-button = + NEW WALLET

# ==================== Data Import ====================
import-title = Import Data
import-select-file = SELECT FILE
import-supported-formats = Supported formats: JSON, CSV, TXT
import-max-size = Maximum file size: 10MB

import-processing = Processing file...
import-validating = Validating data...
import-inserting = Inserting records...

import-success = Import completed successfully
import-partial = Import completed with some issues
import-failed = Import failed

import-summary-title = IMPORT SUMMARY
import-total-processed = Total Processed
import-inserted = Inserted
import-skipped = Skipped
import-errors = Errors
import-preview-title = IMPORT PREVIEW
import-preview-subtitle = Review the detected changes before importing.
import-preview-file-label = FILE
import-preview-format-label = FORMAT
import-preview-type-label = TYPE
import-preview-confirm = IMPORT
import-preview-cancel = CANCEL

import-error-details = ERROR DETAILS
import-skipped-reasons = SKIPPED REASONS
import-line = Line { $line }
import-field = Field: { $field }

import-error-file-too-large = File too large. Maximum size is { $maxSize }MB
import-error-unsupported-format = Unsupported file format. Use JSON, CSV, or TXT
import-error-invalid-json = Invalid JSON format
import-error-no-data = No data found in file
import-error-account-not-found = Account not found: { $name }
import-error-category-not-found = Category not found: { $name }
import-error-currency-mismatch = Currency mismatch for account { $account }
import-error-duplicate = Duplicate entry skipped
import-error-currency-mismatch-detail = Currency mismatch: import has { $import } but account { $account } uses { $expected }
import-error-category-not-found-detail = Category not found: { $name } (type: { $type })
import-error-destination-account-not-found = Destination account not found: { $name }
import-error-same-account-transfer = Cannot transfer to the same account
import-error-wallet-not-found = Wallet not found: { $name }
import-error-crypto-not-found = Crypto asset not found in catalog: { $symbol }
import-error-insufficient-crypto-balance = Insufficient { $symbol } balance in { $wallet }: have { $available }, need { $required }
import-skipped-duplicate-transaction = Duplicate transaction (same date/account/amount/type/description)
import-skipped-duplicate-crypto = Duplicate crypto transaction (same date/wallet/coin/type/amount)
import-skipped-crypto-not-found = Crypto asset not found in catalog (row ignored)

import-format-json = JSON (Sanctum Web Export)
import-format-csv = CSV (Excel/Sheets)
import-format-text = Plain Text

import-preview-change-transaction = Transaction
import-preview-change-income = Income
import-preview-change-expense = Expense
import-preview-change-transfer = Transfer
import-preview-change-crypto = Crypto Transaction
import-preview-changes = PREVIEW CHANGES

settings-import = DATA IMPORT

# ==================== Exchange CSV Import ====================
import-exchange-title = Exchange Import
import-exchange-description = Import transaction history from exchanges and wallets
import-exchange-select-file = SELECT EXCHANGE CSV
import-exchange-supported = Supported: Kraken, Binance, MEXC, NotBank, Feather Wallet, Monero GUI Wallet
import-exchange-wallet-label = TARGET WALLET
import-exchange-wallet-placeholder = Wallet name for imported transactions
import-exchange-detected = Detected format:
import-exchange-not-detected = Could not detect exchange format. Supported: Kraken, Binance, MEXC, NotBank, Feather Wallet, Monero GUI Wallet.
import-exchange-default-wallet = Using default wallet:
import-exchange-importing = Importing { $exchange } transactions...
import-exchange-success = { $exchange } import completed
import-exchange-kraken-ledger = Kraken Ledger
import-exchange-kraken-trades = Kraken Trades
import-exchange-binance-all = Binance All Statements
import-exchange-binance-spot = Binance Spot Trade History
import-exchange-feather = Feather Wallet
import-exchange-monero-gui = Monero GUI Wallet
import-exchange-mexc-spot = MEXC Spot Trade History
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
import-exchange-coin-added = Coin {$symbol} added. Re-running exchange import...
import-exchange-coin-add-failed = Could not add coin {$symbol}: {$reason}
import-exchange-coin-invalid = Invalid symbol for automatic coin creation: {$symbol}
import-exchange-coin-retry-unavailable = No pending exchange import available to retry.
settings-exchange-import = EXCHANGE IMPORT
settings-exchange-import-desc = Import crypto transactions from exchange CSV exports

# ==================== Exchange Wallet Selection ====================
exchange-wallet-select-title = TARGET WALLET
exchange-wallet-select-subtitle = Select an existing wallet or create a new one for the imported transactions
exchange-wallet-tab-select = SELECT WALLET
exchange-wallet-tab-create = CREATE NEW
exchange-wallet-select-label = AVAILABLE WALLETS
exchange-wallet-no-wallets = No wallets found. Switch to the create tab to add one.
exchange-wallet-select-required = Please select a wallet to continue
exchange-wallet-name-required = Wallet name is required
exchange-wallet-continue = CONTINUE
exchange-wallet-category-software = Software Wallet
exchange-wallet-category-hardware = Hardware Wallet
exchange-wallet-category-exchange = Exchange

# ==================== Session Lock Warning ====================
session-warning-title = Vault about to lock
session-warning-body = Locking in {$seconds}s due to inactivity.
session-warning-stay = Stay unlocked
session-warning-lock-now = Lock now

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
settings-password-backup-title = Save the pre-change backup
settings-password-backup-warning = A backup is saved first and is required. That backup keeps the OLD password: restoring it later needs the password you are replacing now.

# ==================== Default Categories ====================
# Keyed off the seeded category codes, so the built-in categories follow the
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
