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
nav-habits = Habits
nav-settings = Settings
nav-lock = Lock
nav-collapse = Collapse

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

# ==================== Finances ====================
finances-title = FINANCES
finances-accounts = ACCOUNTS
finances-transactions = Transactions
finances-add-account = Add Account
finances-add-transaction = Add Transaction
finances-no-accounts = No accounts yet
finances-no-transactions = No transactions found
finances-transfer = Transfer
finances-income = Income
finances-expense = Expense
finances-transfer-funds = Transfer Funds

# Account types
account-type-bank = Bank
account-type-cash = Cash
account-type-savings = Savings
account-type-credit = Credit Card
account-type-other = Other

# Transaction filters
filter-all-accounts = All Accounts
filter-all-types = All Types
filter-all-categories = All Categories
filter-date-range = Date Range
filter-this-month = This Month
filter-last-month = Last Month
filter-this-year = This Year
filter-custom = Custom

# ==================== Crypto ====================
crypto-title = Crypto
crypto-portfolio = Portfolio
crypto-wallets = WALLETS
crypto-tax-tab = TAX
crypto-assets = ASSETS
crypto-add-wallet = Add Wallet
crypto-add-transaction = Add Transaction
crypto-no-wallets = No wallets yet
crypto-no-assets = No assets found
crypto-total-value = Total Value
crypto-price = Price
crypto-holdings = Holdings
crypto-change-24h = 24h Change
crypto-market-cap = Market Cap
crypto-volume = Volume

# Wallet types
wallet-type-exchange = Exchange
wallet-type-hardware = Hardware
wallet-type-software = Software
wallet-type-multi = Multi-Signature

# Transaction types
crypto-tx-buy = Buy
crypto-tx-sell = Sell
crypto-tx-transfer-in = Transfer In
crypto-tx-transfer-out = Transfer Out
crypto-tx-swap = Swap

# Transaction messages
crypto-tx-added = Asset added successfully
crypto-tx-transfer-added = Transfer added successfully
crypto-tx-swap-added = Swap added successfully
crypto-tx-deleted = Transaction deleted
crypto-tx-wallet-required = Please create a wallet first
crypto-tx-two-wallets-required = Create two wallets to move assets
crypto-tx-amount-required = Amount is required
crypto-tx-price-required = Price is required
crypto-tx-coins-required = Add coins in settings first
crypto-tx-different-wallets = Pick two different wallets
crypto-tx-to-amount-required = To amount is required
crypto-tx-swap-different-assets = Swap assets must be different

# ==================== Habits ====================
habits-title = HABITS
habits-my-habits = My Habits
habits-add-habit = Add Habit
habits-no-habits = No habits yet
habits-streak = Streak
habits-best-streak = Best
habits-current-streak = Current
habits-completion-rate = Completion Rate
habits-days = { $count ->
    [one] { $count } day
   *[other] { $count } days
}

# Habit categories
habit-category-mind = Mind
habit-category-body = Body
habit-category-spirit = Spirit

# Habit frequency
habit-frequency-daily = Daily
habit-frequency-weekly = Weekly

# Analytics
habits-analytics = Analytics
habits-life-balance = Life Balance (Last 30 Days)
habits-weekday-efficiency = Weekday Efficiency
habits-empty-chart = The chart is empty, but your potential is full.
habits-empty-chart-subtitle = Your legend will appear here. Start writing it today.
habits-complete-to-see = Complete habits to see your weekly pattern.
habits-discover-days = Discover which days you're most consistent.

# Habits Tab sections
habits-add-button = + HABIT
habits-yearly-overview = YEARLY OVERVIEW
habits-my-habits-section = MY HABITS
habits-no-tracked-month = No habits tracked this month
habits-create-to-build = Create a habit to start building consistency
habits-analytics-section = ANALYTICS
habits-weekly-report = WEEKLY REPORT
habits-insights = INSIGHTS
habits-summary = HABIT SUMMARY
habits-select-placeholder = Select a habit...

# ==================== Rewards ====================
rewards-title = Rewards
rewards-goals = Goals
rewards-streak-rewards = Streak Rewards
rewards-history = History
rewards-add-goal = Add Goal
rewards-add-reward = Add Streak Reward
rewards-no-goals = No goals yet
rewards-no-rewards = No streak rewards yet
rewards-no-history = No history yet
rewards-progress = Progress
rewards-milestones = Milestones
rewards-unlocked = Unlocked
rewards-locked = Locked
rewards-claim = Claim
rewards-completed = Completed

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
settings-export-btn = Export
settings-export-success = Backup saved successfully

# Data Import
settings-import-generic = Generic CSV
settings-import-generic-desc = Import transactions from a CSV file
settings-import-exchange = Exchange CSV
settings-import-exchange-desc = Import from Kraken, Binance, MEXC, and more
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
modal-add-habit-title = Add Habit
modal-edit-habit-title = Edit Habit
modal-add-goal-title = Add Goal
modal-edit-goal-title = Edit Goal
modal-add-reward-title = Add Streak Reward
modal-edit-reward-title = Edit Streak Reward

# Confirmation dialogs
confirm-delete-title = Confirm Delete
confirm-delete-message = This action cannot be undone.
confirm-delete-account = Are you sure you want to delete this account?
confirm-delete-transaction = Are you sure you want to delete this transaction?
confirm-delete-wallet = Are you sure you want to delete this wallet?
confirm-delete-habit = Are you sure you want to delete this habit?

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
dashboard-loading = Loading dashboard...
dashboard-retry = RETRY
dashboard-usd-clp = USD/CLP

# ==================== Finances Extended ====================
finances-activity = ACTIVITY
finances-account = ACCOUNT
finances-all-accounts = All accounts
finances-all-categories = All categories
finances-load-more = LOAD MORE
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
crypto-realized-ytd = REALIZED (YTD)
crypto-roi = ROI
crypto-tax-title = TAX & REPORTS
crypto-tax-subtab-settings = SETTINGS
crypto-tax-subtab-summary = SUMMARY
crypto-tax-period-label = TAX PERIOD (YEAR)
crypto-tax-period-placeholder = 2025
crypto-tax-jurisdiction-label = JURISDICTION
crypto-tax-jurisdiction-cl = Chile
crypto-tax-jurisdiction-us = USA
crypto-tax-jurisdiction-other = Other
crypto-tax-method-label = COST BASIS METHOD
crypto-tax-include-swaps = Include swaps as taxable
crypto-tax-include-swaps-desc = Treat swaps as disposals for tax reports.
crypto-tax-include-fee-crypto = Include fee paid in crypto
crypto-tax-include-fee-crypto-desc = Treat fee coin as a taxable disposal.
crypto-tax-save-settings = Save Tax Settings
crypto-tax-report-title = REPORT GENERATION
crypto-tax-report-desc = Generate a tax report for the selected period. Exports are local CSV files.
crypto-tax-report-generate = Generate Report
crypto-tax-report-export = Export CSV
crypto-tax-report-summary-label = REPORT SUMMARY
crypto-tax-report-summary-empty = No report generated yet
crypto-tax-report-summary = Disposals: {$disposals} | Proceeds: {$proceeds} | Cost: {$cost} | Gain: {$gain}
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
crypto-value = VALUE

crypto-add-first-wallet = Add your first wallet to start tracking your crypto
crypto-no-wallets-created = No wallets created
crypto-delete-wallet = Delete Wallet?
crypto-delete-wallet-confirm-prefix = This will permanently delete "
crypto-delete-wallet-confirm-suffix = " and all its transaction history.
crypto-delete-wallet-warning-title = Wallet Has Transactions
crypto-delete-wallet-warning-prefix = This wallet contains 
crypto-delete-wallet-warning-suffix =  transaction(s). Deleting it will permanently remove all of them.
crypto-delete-wallet-force = Delete Anyway
crypto-loading-portfolio = Loading portfolio...
crypto-syncing-prices = Syncing prices...
crypto-syncing-wait = This may take a few seconds

# ==================== Habits Extended ====================
habits-rewards = REWARDS
habits-history = HISTORY

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
crypto-portfolio-trend = PORTFOLIO TREND (180 DAYS)
crypto-value-label = VALUE
crypto-cost-label = COST
crypto-no-trend = No trend data yet
crypto-sync-daily = Sync prices daily to build history

# Habit Heatmap
heatmap-less = Less
heatmap-more = More

# Habits Tab
habits-selected-hint = SELECTED HABIT · Click a habit above to view stats

# History Tab
history-total-achievements = TOTAL ACHIEVEMENTS

# Streak Rewards
rewards-ready-claim = Ready to claim
rewards-next = Next
rewards-all-unlocked = All milestones unlocked!

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
form-habit = HABIT

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

# Add Habit
modal-category = CATEGORY
modal-color = COLOR
modal-habit-name-placeholder = e.g. Read 10 pages
modal-habit-description-placeholder = Why this habit matters

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

# Add Habit
modal-checkpoints = CHECKPOINTS
modal-checkpoint-desc = Checkpoint description...

# Add Reward
modal-consecutive = CONSECUTIVE
modal-accumulative = ACCUMULATIVE
modal-type = TYPE
modal-milestones = MILESTONES
modal-reward-placeholder = Reward...

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

# Streak Rewards (with arguments)
rewards-ready-claim-with = Ready to claim: { $reward }
rewards-next-with = Next: { $reward }

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

# ==================== Goal Modal ====================
modal-new-goal = NEW GOAL
modal-edit-goal = EDIT GOAL
modal-goal-name = GOAL NAME
modal-goal-name-placeholder = e.g. Run a marathon
modal-description-optional = DESCRIPTION (OPTIONAL)
modal-goal-description-placeholder = Why this goal matters...
modal-reward = REWARD
modal-reward-placeholder-goal = e.g. New sneakers
modal-deadline-optional = DEADLINE (OPTIONAL)
modal-create-goal = CREATE GOAL

# ==================== Reward Modal ====================
modal-new-streak-reward = NEW STREAK REWARD
modal-edit-reward = EDIT REWARD
modal-consecutive-desc = Days must be consecutive (resets if missed)
modal-accumulative-desc = Accumulate days over time
modal-target-days = TARGET DAYS
modal-of-total-days = OF TOTAL DAYS
modal-days-label = days
modal-create-reward = CREATE REWARD

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
finances-search-placeholder = Search by description, category, date...
finances-no-matching = No matching transactions
finances-no-transactions-yet = No transactions yet
finances-try-adjusting = Try clearing or adjusting your filters
finances-add-first-entry = Add your first entry to start tracking your finances
finances-no-accounts-configured = No accounts configured
finances-create-account = Create an account to manage your funds

# ==================== Crypto Extended (Buttons) ====================
crypto-add-wallet-button = + NEW WALLET

# ==================== Habits Extended (Summary Labels) ====================
habits-current-streak-label = CURRENT STREAK
habits-best-streak-label = BEST STREAK (365D)
habits-days-label = days
habits-completion-rate-label = COMPLETION RATE
habits-completions-label = COMPLETIONS (30D)

# ==================== Rewards Extended (Sections/Buttons) ====================
rewards-streak-rewards-section = STREAK REWARDS
rewards-add-reward-button = + REWARD
rewards-no-streak-rewards = No streak rewards yet
rewards-link-habit-desc = Link a habit and set milestone rewards to stay motivated
rewards-goals-section = GOALS
rewards-add-goal-button = + GOAL
rewards-no-goals-set = No goals set
rewards-create-goal-desc = Create a goal with checkpoints to track your progress

# ==================== Rewards Progress ====================
rewards-days-to-go = days to go

# ==================== History Tab ====================
history-achievements-section = ACHIEVEMENTS
history-no-achievements = No achievements yet
history-complete-to-earn = Complete goals to earn trophies

# ==================== Data Import ====================
import-title = Import Data
import-description = Import transactions and habit logs from external files
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
import-error-habit-not-found = Habit not found: { $name }
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
import-skipped-habit-not-completed = Habit not completed (completed=false)
import-skipped-habit-already-logged = Habit already logged for this date
import-skipped-duplicate-crypto = Duplicate crypto transaction (same date/wallet/coin/type/amount)
import-skipped-crypto-not-found = Crypto asset not found in catalog (row ignored)

import-format-json = JSON (Sanctum Web Export)
import-format-csv = CSV (Excel/Sheets)
import-format-text = Plain Text

import-preview-change-transaction = Transaction
import-preview-change-income = Income
import-preview-change-expense = Expense
import-preview-change-transfer = Transfer
import-preview-change-habit = Habit Log
import-preview-change-crypto = Crypto Transaction
import-preview-changes = PREVIEW CHANGES

settings-import = DATA IMPORT
settings-import-desc = Import transactions and habits from files

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
