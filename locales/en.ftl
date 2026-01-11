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
login-subtitle = Personal Financial Fortress
login-password-placeholder = password
login-password-create-placeholder = create password
login-unlock = UNLOCK VAULT
login-create = CREATE VAULT
login-unlocking = UNLOCKING...
login-creating = CREATING...
login-password-required = Password required
login-encryption-note = AES-256 ENCRYPTED
login-weak-password-confirm = Weak password: click create again to confirm.
login-show = SHOW
login-hide = HIDE

# ==================== Sidebar ====================
nav-dashboard = DASHBOARD
nav-finances = FINANCES
nav-crypto = CRYPTO
nav-habits = HABITS
nav-settings = SETTINGS
nav-lock = LOCK VAULT

# ==================== Dashboard ====================
dashboard-title = Dashboard
dashboard-welcome = Welcome back
dashboard-net-worth = Net Worth
dashboard-total-balance = Total Balance
dashboard-monthly-income = Monthly Income
dashboard-monthly-expenses = Monthly Expenses
dashboard-recent-transactions = Recent Transactions
dashboard-no-transactions = No recent transactions
dashboard-view-all = View All
dashboard-quick-actions = Quick Actions
dashboard-add-transaction = Add Transaction
dashboard-add-account = Add Account

# ==================== Finances ====================
finances-title = Finances
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
settings-general = General
settings-appearance = Appearance
settings-security = Security
settings-data = Data
settings-about = About

# General settings
settings-language = Language
settings-language-desc = Application interface language
settings-currency = Currency
settings-currency-desc = Default currency for display
settings-preferred-currency = Preferred Currency
settings-preferred-currency-desc = Base currency for displaying amounts (UI only)

# Appearance settings
settings-dark-mode = Dark Mode
settings-dark-mode-desc = Enable dark theme
settings-dark-mode-title = Dark Mode
settings-dark-mode-toggle-desc = Switch between dark and light (cream/gold) themes

# Security settings
settings-session-timeout = Session Timeout
settings-session-timeout-desc = Auto-lock after inactivity
settings-timeout-5min = 5 minutes
settings-timeout-15min = 15 minutes
settings-timeout-30min = 30 minutes
settings-timeout-1hour = 1 hour
settings-timeout-never = Never
settings-timeout-warning = Timeout changes will apply on next vault open.

# Crypto settings
settings-auto-fetch = Auto-fetch Prices
settings-auto-fetch-desc = Automatically update crypto prices
settings-auto-fetch-title = Auto-fetch Crypto Prices
settings-auto-fetch-toggle-desc = Refresh prices every minute while app is running (uses network)
settings-proxy = Proxy
settings-proxy-enabled = Enable Proxy
settings-proxy-url = Proxy URL
settings-proxy-title = Use Network Proxy
settings-proxy-toggle-desc = Route crypto price requests through a proxy (optional)
settings-proxy-placeholder = http://127.0.0.1:8080 or socks5h://127.0.0.1:9050

# Data settings
settings-reset = Reset Settings
settings-reset-desc = Reset all settings to defaults
settings-reset-confirm = Are you sure you want to reset all settings?

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
dashboard-exchange-rate-warning = Exchange rate unavailable. CLP balances shown at 1:1 rate.
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
crypto-coin-limit = Coin limit reached (50). Some assets may not update.
crypto-skipped = Skipped
crypto-your-holdings = YOUR HOLDINGS
crypto-no-assets-yet = No assets tracked yet
crypto-create-wallet-first = Create a wallet first, then add your crypto holdings
crypto-start-adding = Start by adding a wallet and your first asset
crypto-assets-across-wallets = { $assets } assets across { $wallets } wallets
crypto-wallet = WALLET
crypto-value = VALUE

crypto-add-first-wallet = Add your first wallet to start tracking your crypto
crypto-no-wallets-created = No wallets created
crypto-delete-wallet = Delete Wallet?
crypto-delete-wallet-confirm-prefix = This will permanently delete "
crypto-delete-wallet-confirm-suffix = " and all its transaction history.
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
modal-search-coins = Search coins...

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

# ==================== Asset/Wallet Details ====================
section-transaction-history = TRANSACTION HISTORY

# ==================== Transaction Entry Modal ====================
modal-new-entry = NEW ENTRY
modal-edit-entry = EDIT ENTRY
modal-save-entry = SAVE ENTRY
modal-add-note = Add a note...
