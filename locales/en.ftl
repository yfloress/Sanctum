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
finances-accounts = Accounts
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
crypto-wallets = Wallets
crypto-assets = Assets
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
habits-title = Habits
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

# Appearance settings
settings-dark-mode = Dark Mode
settings-dark-mode-desc = Enable dark theme

# Security settings
settings-session-timeout = Session Timeout
settings-session-timeout-desc = Auto-lock after inactivity
settings-timeout-5min = 5 minutes
settings-timeout-15min = 15 minutes
settings-timeout-30min = 30 minutes
settings-timeout-1hour = 1 hour
settings-timeout-never = Never

# Crypto settings
settings-auto-fetch = Auto-fetch Prices
settings-auto-fetch-desc = Automatically update crypto prices
settings-proxy = Proxy
settings-proxy-enabled = Enable Proxy
settings-proxy-url = Proxy URL

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
crypto-wallet = WALLET
crypto-value = VALUE
crypto-no-wallets = No wallets created
crypto-add-first-wallet = Add your first wallet to start tracking your crypto
crypto-delete-wallet = Delete Wallet?
crypto-delete-wallet-confirm = This will permanently delete and all its transaction history.
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
