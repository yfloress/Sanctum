// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

//! Translations Callback Module
//!
//! Loads translated strings from the i18n service into Slint UI.
//! Handles language switching and translation reloading.

use crate::services::i18n::{self, t, t_args};
use crate::{AppWindow, CryptoAdapter, Translations};
use slint::{ComponentHandle, Model, SharedString};

/// Sets up translation-related callbacks and loads initial translations
pub fn setup_translation_callbacks(ui: &AppWindow) {
    // Initial load of translations
    load_all_translations(ui);

    // Setup reload callback
    let ui_weak = ui.as_weak();
    ui.global::<Translations>().on_reload_translations(move || {
        if let Some(ui) = ui_weak.upgrade() {
            load_all_translations(&ui);
        }
    });
}

/// Loads all translations from the i18n service into the UI
pub fn load_all_translations(ui: &AppWindow) {
    let tr = ui.global::<Translations>();

    // Common
    tr.set_app_name(s(&t("app-name")));
    tr.set_app_subtitle(s(&t("app-subtitle")));

    // Common actions
    tr.set_action_save(s(&t("action-save")));
    tr.set_action_cancel(s(&t("action-cancel")));
    tr.set_action_delete(s(&t("action-delete")));
    tr.set_action_edit(s(&t("action-edit")));
    tr.set_action_create(s(&t("action-create")));
    tr.set_action_add(s(&t("action-add")));
    tr.set_action_close(s(&t("action-close")));
    tr.set_action_confirm(s(&t("action-confirm")));
    tr.set_action_back(s(&t("action-back")));
    tr.set_action_next(s(&t("action-next")));
    tr.set_action_submit(s(&t("action-submit")));
    tr.set_action_archive(s(&t("action-archive")));
    tr.set_action_restore(s(&t("action-restore")));
    tr.set_action_clear(s(&t("action-clear")));

    // Common labels
    tr.set_label_name(s(&t("label-name")));
    tr.set_label_description(s(&t("label-description")));
    tr.set_label_amount(s(&t("label-amount")));
    tr.set_label_date(s(&t("label-date")));
    tr.set_label_category(s(&t("label-category")));
    tr.set_label_type(s(&t("label-type")));
    tr.set_label_status(s(&t("label-status")));
    tr.set_label_balance(s(&t("label-balance")));
    tr.set_label_total(s(&t("label-total")));
    tr.set_label_notes(s(&t("label-notes")));
    tr.set_label_color(s(&t("label-color")));
    tr.set_label_icon(s(&t("label-icon")));
    tr.set_label_currency(s(&t("label-currency")));
    tr.set_label_search(s(&t("label-search")));
    tr.set_label_filter(s(&t("label-filter")));
    tr.set_label_loading(s(&t("label-loading")));
    tr.set_label_none(s(&t("label-none")));
    tr.set_label_all(s(&t("label-all")));
    tr.set_label_yes(s(&t("label-yes")));
    tr.set_label_no(s(&t("label-no")));

    // Time
    tr.set_time_today(s(&t("time-today")));
    tr.set_time_yesterday(s(&t("time-yesterday")));
    tr.set_time_week(s(&t("time-week")));
    tr.set_time_month(s(&t("time-month")));
    tr.set_time_year(s(&t("time-year")));

    // Validation
    tr.set_validation_required(s(&t("validation-required")));
    tr.set_validation_invalid_amount(s(&t("validation-invalid-amount")));
    tr.set_validation_invalid_date(s(&t("validation-invalid-date")));

    // Login
    tr.set_login_subtitle(s(&t("login-subtitle")));
    tr.set_login_password_placeholder(s(&t("login-password-placeholder")));
    tr.set_login_password_create_placeholder(s(&t("login-password-create-placeholder")));
    tr.set_login_unlock(s(&t("login-unlock")));
    tr.set_login_create(s(&t("login-create")));
    tr.set_login_unlocking(s(&t("login-unlocking")));
    tr.set_login_creating(s(&t("login-creating")));
    tr.set_login_password_required(s(&t("login-password-required")));
    tr.set_login_encryption_note(s(&t("login-encryption-note")));
    tr.set_login_weak_password_confirm(s(&t("login-weak-password-confirm")));
    tr.set_login_show(s(&t("login-show")));
    tr.set_login_hide(s(&t("login-hide")));

    // Sidebar
    tr.set_nav_dashboard(s(&t("nav-dashboard")));
    tr.set_nav_finances(s(&t("nav-finances")));
    tr.set_nav_crypto(s(&t("nav-crypto")));
    tr.set_nav_habits(s(&t("nav-habits")));
    tr.set_nav_settings(s(&t("nav-settings")));
    tr.set_nav_lock(s(&t("nav-lock")));

    // Dashboard
    tr.set_dashboard_title(s(&t("dashboard-title")));
    tr.set_dashboard_welcome(s(&t("dashboard-welcome")));
    tr.set_dashboard_net_worth(s(&t("dashboard-net-worth")));
    tr.set_dashboard_total_balance(s(&t("dashboard-total-balance")));
    tr.set_dashboard_monthly_income(s(&t("dashboard-monthly-income")));
    tr.set_dashboard_monthly_expenses(s(&t("dashboard-monthly-expenses")));
    tr.set_dashboard_recent_transactions(s(&t("dashboard-recent-transactions")));
    tr.set_dashboard_no_transactions(s(&t("dashboard-no-transactions")));
    tr.set_dashboard_view_all(s(&t("dashboard-view-all")));
    tr.set_dashboard_quick_actions(s(&t("dashboard-quick-actions")));
    tr.set_dashboard_add_transaction(s(&t("dashboard-add-transaction")));
    tr.set_dashboard_add_account(s(&t("dashboard-add-account")));

    // Finances
    tr.set_finances_title(s(&t("finances-title")));
    tr.set_finances_accounts(s(&t("finances-accounts")));
    tr.set_finances_transactions(s(&t("finances-transactions")));
    tr.set_finances_add_account(s(&t("finances-add-account")));
    tr.set_finances_add_transaction(s(&t("finances-add-transaction")));
    tr.set_finances_no_accounts(s(&t("finances-no-accounts")));
    tr.set_finances_no_transactions(s(&t("finances-no-transactions")));
    tr.set_finances_transfer(s(&t("finances-transfer")));
    tr.set_finances_income(s(&t("finances-income")));
    tr.set_finances_expense(s(&t("finances-expense")));
    tr.set_finances_transfer_funds(s(&t("finances-transfer-funds")));

    // Account types
    tr.set_account_type_bank(s(&t("account-type-bank")));
    tr.set_account_type_cash(s(&t("account-type-cash")));
    tr.set_account_type_savings(s(&t("account-type-savings")));
    tr.set_account_type_credit(s(&t("account-type-credit")));
    tr.set_account_type_other(s(&t("account-type-other")));

    // Filters
    tr.set_filter_all_accounts(s(&t("filter-all-accounts")));
    tr.set_filter_all_types(s(&t("filter-all-types")));
    tr.set_filter_all_categories(s(&t("filter-all-categories")));
    tr.set_filter_date_range(s(&t("filter-date-range")));
    tr.set_filter_this_month(s(&t("filter-this-month")));
    tr.set_filter_last_month(s(&t("filter-last-month")));
    tr.set_filter_this_year(s(&t("filter-this-year")));
    tr.set_filter_custom(s(&t("filter-custom")));

    // Crypto
    tr.set_crypto_title(s(&t("crypto-title")));
    tr.set_crypto_portfolio(s(&t("crypto-portfolio")));
    tr.set_crypto_wallets(s(&t("crypto-wallets")));
    tr.set_crypto_tax_tab(s(&t("crypto-tax-tab")));
    tr.set_crypto_assets(s(&t("crypto-assets")));
    tr.set_crypto_add_wallet(s(&t("crypto-add-wallet")));
    tr.set_crypto_add_transaction(s(&t("crypto-add-transaction")));
    tr.set_crypto_no_wallets(s(&t("crypto-no-wallets")));
    tr.set_crypto_no_assets(s(&t("crypto-no-assets")));
    tr.set_crypto_total_value(s(&t("crypto-total-value")));
    tr.set_crypto_price(s(&t("crypto-price")));
    tr.set_crypto_holdings(s(&t("crypto-holdings")));
    tr.set_crypto_change_24h(s(&t("crypto-change-24h")));
    tr.set_crypto_market_cap(s(&t("crypto-market-cap")));
    tr.set_crypto_volume(s(&t("crypto-volume")));
    tr.set_crypto_total_holdings(s(&t("crypto-total-holdings")));
    tr.set_crypto_no_wallet_data(s(&t("crypto-no-wallet-data")));
    tr.set_crypto_no_transactions_found(s(&t("crypto-no-transactions-found")));
    tr.set_crypto_portfolio_distribution(s(&t("crypto-portfolio-distribution")));

    // Wallet types
    tr.set_wallet_type_exchange(s(&t("wallet-type-exchange")));
    tr.set_wallet_type_hardware(s(&t("wallet-type-hardware")));
    tr.set_wallet_type_software(s(&t("wallet-type-software")));
    tr.set_wallet_type_multi(s(&t("wallet-type-multi")));

    // Crypto transactions
    tr.set_crypto_tx_buy(s(&t("crypto-tx-buy")));
    tr.set_crypto_tx_sell(s(&t("crypto-tx-sell")));
    tr.set_crypto_tx_transfer_in(s(&t("crypto-tx-transfer-in")));
    tr.set_crypto_tx_transfer_out(s(&t("crypto-tx-transfer-out")));
    tr.set_crypto_tx_swap(s(&t("crypto-tx-swap")));
    tr.set_crypto_tx_added(s(&t("crypto-tx-added")));
    tr.set_crypto_tx_transfer_added(s(&t("crypto-tx-transfer-added")));
    tr.set_crypto_tx_swap_added(s(&t("crypto-tx-swap-added")));
    tr.set_crypto_tx_deleted(s(&t("crypto-tx-deleted")));
    tr.set_crypto_tx_wallet_required(s(&t("crypto-tx-wallet-required")));
    tr.set_crypto_tx_two_wallets_required(s(&t("crypto-tx-two-wallets-required")));
    tr.set_crypto_tx_amount_required(s(&t("crypto-tx-amount-required")));
    tr.set_crypto_tx_price_required(s(&t("crypto-tx-price-required")));
    tr.set_crypto_tx_coins_required(s(&t("crypto-tx-coins-required")));
    tr.set_crypto_tx_different_wallets(s(&t("crypto-tx-different-wallets")));
    tr.set_crypto_tx_to_amount_required(s(&t("crypto-tx-to-amount-required")));
    tr.set_crypto_tx_swap_different_assets(s(&t("crypto-tx-swap-different-assets")));

    // Habits
    tr.set_habits_title(s(&t("habits-title")));
    tr.set_habits_my_habits(s(&t("habits-my-habits")));
    tr.set_habits_add_habit(s(&t("habits-add-habit")));
    tr.set_habits_no_habits(s(&t("habits-no-habits")));
    tr.set_habits_streak(s(&t("habits-streak")));
    tr.set_habits_best_streak(s(&t("habits-best-streak")));
    tr.set_habits_current_streak(s(&t("habits-current-streak")));
    tr.set_habits_completion_rate(s(&t("habits-completion-rate")));

    // Habit categories
    tr.set_habit_category_mind(s(&t("habit-category-mind")));
    tr.set_habit_category_body(s(&t("habit-category-body")));
    tr.set_habit_category_spirit(s(&t("habit-category-spirit")));

    // Habit frequency
    tr.set_habit_frequency_daily(s(&t("habit-frequency-daily")));
    tr.set_habit_frequency_weekly(s(&t("habit-frequency-weekly")));

    // Analytics
    tr.set_habits_analytics(s(&t("habits-analytics")));
    tr.set_habits_life_balance(s(&t("habits-life-balance")));
    tr.set_habits_weekday_efficiency(s(&t("habits-weekday-efficiency")));
    tr.set_habits_empty_chart(s(&t("habits-empty-chart")));
    tr.set_habits_empty_chart_subtitle(s(&t("habits-empty-chart-subtitle")));
    tr.set_habits_complete_to_see(s(&t("habits-complete-to-see")));
    tr.set_habits_discover_days(s(&t("habits-discover-days")));
    tr.set_habits_add_button(s(&t("habits-add-button")));
    tr.set_habits_yearly_overview(s(&t("habits-yearly-overview")));
    tr.set_habits_my_habits_section(s(&t("habits-my-habits-section")));
    tr.set_habits_no_tracked_month(s(&t("habits-no-tracked-month")));
    tr.set_habits_create_to_build(s(&t("habits-create-to-build")));
    tr.set_habits_analytics_section(s(&t("habits-analytics-section")));
    tr.set_habits_weekly_report(s(&t("habits-weekly-report")));
    tr.set_habits_insights(s(&t("habits-insights")));
    tr.set_habits_summary(s(&t("habits-summary")));
    tr.set_habits_select_placeholder(s(&t("habits-select-placeholder")));

    // Rewards
    tr.set_rewards_title(s(&t("rewards-title")));
    tr.set_rewards_goals(s(&t("rewards-goals")));
    tr.set_rewards_streak_rewards(s(&t("rewards-streak-rewards")));
    tr.set_rewards_history(s(&t("rewards-history")));
    tr.set_rewards_add_goal(s(&t("rewards-add-goal")));
    tr.set_rewards_add_reward(s(&t("rewards-add-reward")));
    tr.set_rewards_no_goals(s(&t("rewards-no-goals")));
    tr.set_rewards_no_rewards(s(&t("rewards-no-rewards")));
    tr.set_rewards_no_history(s(&t("rewards-no-history")));
    tr.set_rewards_progress(s(&t("rewards-progress")));
    tr.set_rewards_milestones(s(&t("rewards-milestones")));
    tr.set_rewards_unlocked(s(&t("rewards-unlocked")));
    tr.set_rewards_locked(s(&t("rewards-locked")));
    tr.set_rewards_claim(s(&t("rewards-claim")));
    tr.set_rewards_completed(s(&t("rewards-completed")));

    // Settings
    tr.set_settings_title(s(&t("settings-title")));
    tr.set_settings_general(s(&t("settings-general")));
    tr.set_settings_appearance(s(&t("settings-appearance")));
    tr.set_settings_security(s(&t("settings-security")));
    tr.set_settings_data(s(&t("settings-data")));
    tr.set_settings_about(s(&t("settings-about")));

    tr.set_settings_language(s(&t("settings-language")));
    tr.set_settings_language_desc(s(&t("settings-language-desc")));
    tr.set_settings_currency(s(&t("settings-currency")));
    tr.set_settings_currency_desc(s(&t("settings-currency-desc")));
    tr.set_settings_preferred_currency(s(&t("settings-preferred-currency")));
    tr.set_settings_preferred_currency_desc(s(&t("settings-preferred-currency-desc")));

    tr.set_settings_dark_mode(s(&t("settings-dark-mode")));
    tr.set_settings_dark_mode_desc(s(&t("settings-dark-mode-desc")));
    tr.set_settings_dark_mode_title(s(&t("settings-dark-mode-title")));
    tr.set_settings_dark_mode_toggle_desc(s(&t("settings-dark-mode-toggle-desc")));
    tr.set_settings_wallpaper_title(s(&t("settings-wallpaper-title")));
    tr.set_settings_wallpaper_desc(s(&t("settings-wallpaper-desc")));
    tr.set_settings_wallpaper_default(s(&t("settings-wallpaper-default")));
    tr.set_settings_wallpaper_select(s(&t("settings-wallpaper-select")));
    tr.set_settings_wallpaper_reset(s(&t("settings-wallpaper-reset")));
    tr.set_settings_wallpaper_note(s(&t("settings-wallpaper-note")));
    tr.set_settings_wallpaper_formats(s(&t("settings-wallpaper-formats")));

    tr.set_settings_session_timeout(s(&t("settings-session-timeout")));
    tr.set_settings_session_timeout_desc(s(&t("settings-session-timeout-desc")));
    tr.set_settings_timeout_5min(s(&t("settings-timeout-5min")));
    tr.set_settings_timeout_15min(s(&t("settings-timeout-15min")));
    tr.set_settings_timeout_30min(s(&t("settings-timeout-30min")));
    tr.set_settings_timeout_1hour(s(&t("settings-timeout-1hour")));
    tr.set_settings_timeout_never(s(&t("settings-timeout-never")));
    tr.set_settings_timeout_warning(s(&t("settings-timeout-warning")));

    tr.set_settings_auto_fetch(s(&t("settings-auto-fetch")));
    tr.set_settings_auto_fetch_desc(s(&t("settings-auto-fetch-desc")));
    tr.set_settings_auto_fetch_title(s(&t("settings-auto-fetch-title")));
    tr.set_settings_auto_fetch_toggle_desc(s(&t("settings-auto-fetch-toggle-desc")));
    tr.set_settings_proxy(s(&t("settings-proxy")));
    tr.set_settings_proxy_enabled(s(&t("settings-proxy-enabled")));
    tr.set_settings_proxy_url(s(&t("settings-proxy-url")));
    tr.set_settings_proxy_title(s(&t("settings-proxy-title")));
    tr.set_settings_proxy_toggle_desc(s(&t("settings-proxy-toggle-desc")));
    tr.set_settings_proxy_placeholder(s(&t("settings-proxy-placeholder")));

    tr.set_settings_version_label(s(&t("settings-version-label")));
    tr.set_settings_encryption_label(s(&t("settings-encryption-label")));
    tr.set_settings_database_label(s(&t("settings-database-label")));

    tr.set_settings_reset(s(&t("settings-reset")));
    tr.set_settings_reset_desc(s(&t("settings-reset-desc")));
    tr.set_settings_reset_confirm(s(&t("settings-reset-confirm")));

    tr.set_settings_import(s(&t("settings-import")));
    tr.set_settings_import_desc(s(&t("settings-import-desc")));
    tr.set_import_title(s(&t("import-title")));
    tr.set_import_description(s(&t("import-description")));
    tr.set_import_select_file(s(&t("import-select-file")));
    tr.set_import_supported_formats(s(&t("import-supported-formats")));
    tr.set_import_max_size(s(&t("import-max-size")));
    tr.set_import_processing(s(&t("import-processing")));
    tr.set_import_validating(s(&t("import-validating")));
    tr.set_import_inserting(s(&t("import-inserting")));
    tr.set_import_success(s(&t("import-success")));
    tr.set_import_partial(s(&t("import-partial")));
    tr.set_import_failed(s(&t("import-failed")));
    tr.set_import_summary_title(s(&t("import-summary-title")));
    tr.set_import_total_processed(s(&t("import-total-processed")));
    tr.set_import_inserted(s(&t("import-inserted")));
    tr.set_import_skipped(s(&t("import-skipped")));
    tr.set_import_errors(s(&t("import-errors")));
    tr.set_import_preview_title(s(&t("import-preview-title")));
    tr.set_import_preview_subtitle(s(&t("import-preview-subtitle")));
    tr.set_import_preview_file_label(s(&t("import-preview-file-label")));
    tr.set_import_preview_format_label(s(&t("import-preview-format-label")));
    tr.set_import_preview_type_label(s(&t("import-preview-type-label")));
    tr.set_import_preview_confirm(s(&t("import-preview-confirm")));
    tr.set_import_preview_cancel(s(&t("import-preview-cancel")));
    tr.set_import_error_details(s(&t("import-error-details")));
    tr.set_import_skipped_reasons(s(&t("import-skipped-reasons")));
    tr.set_import_preview_changes(s(&t("import-preview-changes")));

    // Exchange CSV import
    tr.set_settings_exchange_import(s(&t("settings-exchange-import")));
    tr.set_settings_exchange_import_desc(s(&t("settings-exchange-import-desc")));
    tr.set_import_exchange_title(s(&t("import-exchange-title")));
    tr.set_import_exchange_description(s(&t("import-exchange-description")));
    tr.set_import_exchange_select_file(s(&t("import-exchange-select-file")));
    tr.set_import_exchange_supported(s(&t("import-exchange-supported")));
    tr.set_import_exchange_wallet_label(s(&t("import-exchange-wallet-label")));
    tr.set_import_exchange_wallet_placeholder(s(&t("import-exchange-wallet-placeholder")));
    tr.set_import_exchange_detected(s(&t("import-exchange-detected")));
    tr.set_import_exchange_not_detected(s(&t("import-exchange-not-detected")));
    tr.set_import_exchange_default_wallet(s(&t("import-exchange-default-wallet")));
    tr.set_import_exchange_hint_kraken(s(&t("import-exchange-hint-kraken")));
    tr.set_import_exchange_hint_kraken_pro(s(&t("import-exchange-hint-kraken-pro")));
    tr.set_import_exchange_hint_binance(s(&t("import-exchange-hint-binance")));
    tr.set_import_exchange_hint_feather(s(&t("import-exchange-hint-feather")));
    tr.set_import_exchange_hint_monero_gui(s(&t("import-exchange-hint-monero-gui")));
    tr.set_import_exchange_hint_mexc(s(&t("import-exchange-hint-mexc")));
    tr.set_import_exchange_hint_notbank(s(&t("import-exchange-hint-notbank")));

    // Exchange wallet selection modal
    tr.set_exchange_wallet_select_title(s(&t("exchange-wallet-select-title")));
    tr.set_exchange_wallet_select_subtitle(s(&t("exchange-wallet-select-subtitle")));
    tr.set_exchange_wallet_tab_select(s(&t("exchange-wallet-tab-select")));
    tr.set_exchange_wallet_tab_create(s(&t("exchange-wallet-tab-create")));
    tr.set_exchange_wallet_select_label(s(&t("exchange-wallet-select-label")));
    tr.set_exchange_wallet_no_wallets(s(&t("exchange-wallet-no-wallets")));
    tr.set_exchange_wallet_select_required(s(&t("exchange-wallet-select-required")));
    tr.set_exchange_wallet_name_required(s(&t("exchange-wallet-name-required")));
    tr.set_exchange_wallet_continue(s(&t("exchange-wallet-continue")));
    tr.set_exchange_wallet_category_software(s(&t("exchange-wallet-category-software")));
    tr.set_exchange_wallet_category_hardware(s(&t("exchange-wallet-category-hardware")));
    tr.set_exchange_wallet_category_exchange(s(&t("exchange-wallet-category-exchange")));

    // Modals
    tr.set_modal_add_account_title(s(&t("modal-add-account-title")));
    tr.set_modal_edit_account_title(s(&t("modal-edit-account-title")));
    tr.set_modal_add_transaction_title(s(&t("modal-add-transaction-title")));
    tr.set_modal_edit_transaction_title(s(&t("modal-edit-transaction-title")));
    tr.set_modal_transfer_title(s(&t("modal-transfer-title")));
    tr.set_modal_add_wallet_title(s(&t("modal-add-wallet-title")));
    tr.set_modal_edit_wallet_title(s(&t("modal-edit-wallet-title")));
    tr.set_modal_add_habit_title(s(&t("modal-add-habit-title")));
    tr.set_modal_edit_habit_title(s(&t("modal-edit-habit-title")));
    tr.set_modal_habit_name_placeholder(s(&t("modal-habit-name-placeholder")));
    tr.set_modal_habit_description_placeholder(s(&t("modal-habit-description-placeholder")));
    tr.set_modal_add_goal_title(s(&t("modal-add-goal-title")));
    tr.set_modal_edit_goal_title(s(&t("modal-edit-goal-title")));
    tr.set_modal_add_reward_title(s(&t("modal-add-reward-title")));
    tr.set_modal_edit_reward_title(s(&t("modal-edit-reward-title")));

    // Confirmations
    tr.set_confirm_delete_title(s(&t("confirm-delete-title")));
    tr.set_confirm_delete_message(s(&t("confirm-delete-message")));
    tr.set_confirm_delete_account(s(&t("confirm-delete-account")));
    tr.set_confirm_delete_transaction(s(&t("confirm-delete-transaction")));
    tr.set_confirm_delete_wallet(s(&t("confirm-delete-wallet")));
    tr.set_confirm_delete_habit(s(&t("confirm-delete-habit")));
    tr.set_confirm_delete_generic(s(&t("confirm-delete-generic")));

    // Notifications
    tr.set_notify_success(s(&t("notify-success")));
    tr.set_notify_error(s(&t("notify-error")));
    tr.set_notify_saved(s(&t("notify-saved")));
    tr.set_notify_deleted(s(&t("notify-deleted")));
    tr.set_notify_created(s(&t("notify-created")));
    tr.set_notify_updated(s(&t("notify-updated")));

    // Empty states
    tr.set_empty_no_data(s(&t("empty-no-data")));
    tr.set_empty_no_results(s(&t("empty-no-results")));
    tr.set_empty_try_different(s(&t("empty-try-different")));
    tr.set_empty_no_transactions_account(s(&t("empty-no-transactions-account")));

    // Errors
    tr.set_error_generic(s(&t("error-generic")));
    tr.set_error_connection(s(&t("error-connection")));
    tr.set_error_invalid_input(s(&t("error-invalid-input")));
    tr.set_error_not_found(s(&t("error-not-found")));
    tr.set_error_unauthorized(s(&t("error-unauthorized")));

    // Misc
    tr.set_bank_icons_title(s(&t("bank-icons-title")));
    tr.set_no_expenses_recorded(s(&t("no-expenses-recorded")));
    tr.set_fee_label(s(&t("fee-label")));

    // Dashboard Extended
    tr.set_dashboard_total_net_worth(s(&t("dashboard-total-net-worth")));
    tr.set_dashboard_exchange_rate_warning(s(&t("dashboard-exchange-rate-warning")));
    tr.set_dashboard_loading(s(&t("dashboard-loading")));
    tr.set_dashboard_retry(s(&t("dashboard-retry")));
    tr.set_dashboard_usd_clp(s(&t("dashboard-usd-clp")));

    // Finances Extended
    tr.set_finances_activity(s(&t("finances-activity")));
    tr.set_finances_account(s(&t("finances-account")));
    tr.set_finances_all_accounts(s(&t("finances-all-accounts")));
    tr.set_finances_all_categories(s(&t("finances-all-categories")));
    tr.set_finances_load_more(s(&t("finances-load-more")));
    tr.set_finances_configure(s(&t("finances-configure")));
    tr.set_finances_transaction_categories(s(&t("finances-transaction-categories")));
    tr.set_finances_manage_categories(s(&t("finances-manage-categories")));
    tr.set_finances_delete_transaction(s(&t("finances-delete-transaction")));
    tr.set_finances_delete_confirm(s(&t("finances-delete-confirm")));

    // Crypto Extended
    tr.set_crypto_portfolio_title(s(&t("crypto-portfolio-title")));
    tr.set_crypto_last_updated(s(&t("crypto-last-updated")));
    tr.set_crypto_coin_limit(s(&t("crypto-coin-limit")));
    tr.set_crypto_skipped(s(&t("crypto-skipped")));
    tr.set_crypto_your_holdings(s(&t("crypto-your-holdings")));
    tr.set_crypto_no_assets_yet(s(&t("crypto-no-assets-yet")));
    tr.set_crypto_create_wallet_first(s(&t("crypto-create-wallet-first")));
    tr.set_crypto_start_adding(s(&t("crypto-start-adding")));
    tr.set_crypto_import_csv(s(&t("crypto-import-csv")));
    tr.set_crypto_unrealized(s(&t("crypto-unrealized")));
    tr.set_crypto_realized_ytd(s(&t("crypto-realized-ytd")));
    tr.set_crypto_roi(s(&t("crypto-roi")));
    tr.set_crypto_tax_title(s(&t("crypto-tax-title")));
    tr.set_crypto_tax_subtab_settings(s(&t("crypto-tax-subtab-settings")));
    tr.set_crypto_tax_subtab_summary(s(&t("crypto-tax-subtab-summary")));
    tr.set_crypto_tax_period_label(s(&t("crypto-tax-period-label")));
    tr.set_crypto_tax_period_placeholder(s(&t("crypto-tax-period-placeholder")));
    tr.set_crypto_tax_jurisdiction_label(s(&t("crypto-tax-jurisdiction-label")));
    tr.set_crypto_tax_jurisdiction_cl(s(&t("crypto-tax-jurisdiction-cl")));
    tr.set_crypto_tax_jurisdiction_us(s(&t("crypto-tax-jurisdiction-us")));
    tr.set_crypto_tax_jurisdiction_other(s(&t("crypto-tax-jurisdiction-other")));
    tr.set_crypto_tax_method_label(s(&t("crypto-tax-method-label")));
    tr.set_crypto_tax_include_swaps(s(&t("crypto-tax-include-swaps")));
    tr.set_crypto_tax_include_swaps_desc(s(&t("crypto-tax-include-swaps-desc")));
    tr.set_crypto_tax_include_fee_crypto(s(&t("crypto-tax-include-fee-crypto")));
    tr.set_crypto_tax_include_fee_crypto_desc(s(&t("crypto-tax-include-fee-crypto-desc")));
    tr.set_crypto_tax_save_settings(s(&t("crypto-tax-save-settings")));
    tr.set_crypto_tax_report_warnings_empty(s(&t("crypto-tax-report-warnings-empty")));
    tr.set_crypto_tax_report_warnings_count(s(&t_args(
        "crypto-tax-report-warnings-count",
        &[("count", "0")],
    )));
    tr.set_crypto_tax_report_generated(s(&t("crypto-tax-report-generated")));
    tr.set_crypto_tax_report_exported(s(&t("crypto-tax-report-exported")));
    tr.set_crypto_tax_summary_capital(s(&t("crypto-tax-summary-capital")));
    tr.set_crypto_tax_summary_income(s(&t("crypto-tax-summary-income")));
    tr.set_crypto_tax_summary_balance(s(&t("crypto-tax-summary-balance")));
    tr.set_crypto_tax_summary_proceeds(s(&t("crypto-tax-summary-proceeds")));
    tr.set_crypto_tax_summary_cost(s(&t("crypto-tax-summary-cost")));
    tr.set_crypto_tax_summary_gain(s(&t("crypto-tax-summary-gain")));
    tr.set_crypto_tax_summary_income_total(s(&t("crypto-tax-summary-income-total")));
    tr.set_crypto_tax_summary_balance_total(s(&t("crypto-tax-summary-balance-total")));
    tr.set_crypto_tax_summary_export_history(s(&t("crypto-tax-summary-export-history")));
    tr.set_crypto_tax_summary_export_capital(s(&t("crypto-tax-summary-export-capital")));
    tr.set_crypto_tax_summary_transactions(s(&t("crypto-tax-summary-transactions")));
    tr.set_crypto_tax_summary_volume(s(&t("crypto-tax-summary-volume")));
    tr.set_crypto_tax_summary_short_term(s(&t("crypto-tax-summary-short-term")));
    tr.set_crypto_tax_summary_long_term(s(&t("crypto-tax-summary-long-term")));
    tr.set_crypto_tax_summary_disposals(s(&t("crypto-tax-summary-disposals")));
    tr.set_crypto_tax_readiness_banner(s(&t("crypto-tax-readiness-banner")));
    tr.set_crypto_tax_empty_title(s(&t("crypto-tax-empty-title")));
    tr.set_crypto_tax_empty_desc(s(&t("crypto-tax-empty-desc")));
    tr.set_crypto_tax_exports_title(s(&t("crypto-tax-exports-title")));
    tr.set_crypto_tax_exports_capital_desc(s(&t("crypto-tax-exports-capital-desc")));
    tr.set_crypto_tax_exports_history_desc(s(&t("crypto-tax-exports-history-desc")));
    tr.set_crypto_tax_report_details(s(&t("crypto-tax-report-details")));
    tr.set_crypto_tax_settings_advanced(s(&t("crypto-tax-settings-advanced")));
    tr.set_crypto_tax_wallet_exclusions(s(&t("crypto-tax-wallet-exclusions")));
    tr.set_crypto_tax_wallet_exclusions_desc(s(&t("crypto-tax-wallet-exclusions-desc")));
    tr.set_crypto_tax_wallet_none(s(&t("crypto-tax-wallet-none")));
    tr.set_crypto_tax_wallet_excluded_label(s(&t("crypto-tax-wallet-excluded-label")));
    tr.set_crypto_tax_filing_title(s(&t("crypto-tax-filing-title")));
    tr.set_crypto_tax_save_generate(s(&t("crypto-tax-save-generate")));
    tr.set_crypto_tax_readiness_settings_suffix(s(&t("crypto-tax-readiness-settings-suffix")));
    tr.set_crypto_tax_readiness_settings_warn_detail(s(&t(
        "crypto-tax-readiness-settings-warn-detail",
    )));
    tr.set_crypto_tax_readiness_settings_excluded_warn_suffix(s(&t(
        "crypto-tax-readiness-settings-excluded-warn-suffix",
    )));
    tr.set_crypto_tax_readiness_history_warn_suffix(s(&t(
        "crypto-tax-readiness-history-warn-suffix",
    )));
    tr.set_crypto_tax_readiness_prices_invalid_suffix(s(&t(
        "crypto-tax-readiness-prices-invalid-suffix",
    )));
    tr.set_crypto_tax_readiness_prices_warn_suffix(s(&t(
        "crypto-tax-readiness-prices-warn-suffix",
    )));
    tr.set_crypto_tax_readiness_prices_fx_warn_suffix(s(&t(
        "crypto-tax-readiness-prices-fx-warn-suffix",
    )));
    tr.set_crypto_tax_readiness_transfers_warn_suffix(s(&t(
        "crypto-tax-readiness-transfers-warn-suffix",
    )));
    tr.set_crypto_tax_readiness_balances_warn_suffix(s(&t(
        "crypto-tax-readiness-balances-warn-suffix",
    )));
    tr.set_crypto_tax_readiness_sii_gain_detail(s(&t("crypto-tax-readiness-sii-gain-detail")));
    tr.set_crypto_tax_readiness_sii_loss_detail(s(&t("crypto-tax-readiness-sii-loss-detail")));
    tr.set_crypto_tax_readiness_sii_neutral_detail(s(&t(
        "crypto-tax-readiness-sii-neutral-detail",
    )));
    tr.set_crypto_tax_readiness_usa_filing_detail(s(&t("crypto-tax-readiness-usa-filing-detail")));
    tr.set_crypto_tax_readiness_other_filing_detail(s(&t(
        "crypto-tax-readiness-other-filing-detail",
    )));
    tr.set_crypto_tax_readiness_title(s(&t("crypto-tax-readiness-title")));
    tr.set_crypto_tax_readiness_desc(s(&t("crypto-tax-readiness-desc")));
    tr.set_crypto_tax_readiness_settings(s(&t("crypto-tax-readiness-settings")));
    tr.set_crypto_tax_readiness_history(s(&t("crypto-tax-readiness-history")));
    tr.set_crypto_tax_readiness_balances(s(&t("crypto-tax-readiness-balances")));
    tr.set_crypto_tax_readiness_prices(s(&t("crypto-tax-readiness-prices")));
    tr.set_crypto_tax_readiness_prices_fx(s(&t("crypto-tax-readiness-prices-fx")));
    tr.set_crypto_tax_readiness_transfers(s(&t("crypto-tax-readiness-transfers")));
    tr.set_crypto_tax_readiness_filing(s(&t("crypto-tax-readiness-filing")));
    tr.set_crypto_tax_ipc_title(s(&t("crypto-tax-ipc-title")));
    tr.set_crypto_tax_ipc_desc(s(&t("crypto-tax-ipc-desc")));
    tr.set_crypto_tax_ipc_source_label(s(&t("crypto-tax-ipc-source-label")));
    tr.set_crypto_tax_ipc_source_url(s(&t("crypto-tax-ipc-source-url")));
    tr.set_crypto_tax_ipc_import(s(&t("crypto-tax-ipc-import")));
    tr.set_crypto_tax_ipc_copy_url(s(&t("crypto-tax-ipc-copy-url")));
    tr.set_crypto_tax_ipc_empty(s(&t("crypto-tax-ipc-empty")));
    tr.set_crypto_tax_ipc_summary(s(&t_args(
        "crypto-tax-ipc-summary",
        &[("first", "--"), ("last", "--"), ("count", "0")],
    )));
    tr.set_crypto_tax_ipc_import_success(s(&t_args(
        "crypto-tax-ipc-import-success",
        &[("count", "0"), ("first", "--"), ("last", "--")],
    )));
    tr.set_crypto_tax_settings_saved(s(&t("crypto-tax-settings-saved")));
    tr.set_crypto_tax_period_required(s(&t("crypto-tax-period-required")));
    tr.set_crypto_tax_advanced(s(&t("crypto-tax-advanced")));
    tr.set_modal_transaction_type(s(&t("modal-transaction-type")));
    tr.set_modal_transaction_type_placeholder(s(&t("modal-transaction-type-placeholder")));
    tr.set_modal_transaction_subtype(s(&t("modal-transaction-subtype")));
    tr.set_modal_transaction_subtype_placeholder(s(&t("modal-transaction-subtype-placeholder")));
    tr.set_modal_tax_override_proceeds(s(&t("modal-tax-override-proceeds")));
    tr.set_modal_tax_override_cost(s(&t("modal-tax-override-cost")));
    tr.set_crypto_wallet(s(&t("crypto-wallet")));
    tr.set_crypto_value(s(&t("crypto-value")));
    tr.set_crypto_no_wallets(s(&t("crypto-no-wallets")));
    tr.set_crypto_add_first_wallet(s(&t("crypto-add-first-wallet")));
    tr.set_crypto_delete_wallet(s(&t("crypto-delete-wallet")));
    tr.set_crypto_delete_wallet_confirm_prefix(s(&t("crypto-delete-wallet-confirm-prefix")));
    tr.set_crypto_delete_wallet_confirm_suffix(s(&t("crypto-delete-wallet-confirm-suffix")));
    tr.set_crypto_delete_wallet_warning_title(s(&t("crypto-delete-wallet-warning-title")));
    tr.set_crypto_delete_wallet_warning_prefix(s(&t("crypto-delete-wallet-warning-prefix")));
    tr.set_crypto_delete_wallet_warning_suffix(s(&t("crypto-delete-wallet-warning-suffix")));
    tr.set_crypto_delete_wallet_force(s(&t("crypto-delete-wallet-force")));
    tr.set_crypto_loading_portfolio(s(&t("crypto-loading-portfolio")));
    tr.set_crypto_syncing_prices(s(&t("crypto-syncing-prices")));
    tr.set_crypto_syncing_wait(s(&t("crypto-syncing-wait")));

    // Habits Extended
    tr.set_habits_rewards(s(&t("habits-rewards")));
    tr.set_habits_history(s(&t("habits-history")));

    // Settings Extended
    tr.set_settings_configure_experience(s(&t("settings-configure-experience")));
    tr.set_settings_proxy_tip(s(&t("settings-proxy-tip")));
    tr.set_settings_data_encrypted(s(&t("settings-data-encrypted")));
    tr.set_settings_military_grade(s(&t("settings-military-grade")));
    tr.set_settings_reset_defaults(s(&t("settings-reset-defaults")));

    // Common Actions Extended
    tr.set_action_view_all(s(&t("action-view-all")));
    tr.set_action_retry(s(&t("action-retry")));
    tr.set_action_load_more(s(&t("action-load-more")));
    tr.set_action_configure(s(&t("action-configure")));
    tr.set_action_transfer(s(&t("action-transfer")));

    // Components - Account Item
    tr.set_account_balance(s(&t("account-balance")));

    // Crypto Widgets
    tr.set_crypto_holdings_label(s(&t("crypto-holdings-label")));
    tr.set_crypto_price_label(s(&t("crypto-price-label")));

    // Crypto Charts
    tr.set_crypto_no_priced_assets(s(&t("crypto-no-priced-assets")));
    tr.set_crypto_sync_to_see(s(&t("crypto-sync-to-see")));
    tr.set_crypto_portfolio_trend(s(&t("crypto-portfolio-trend")));
    tr.set_crypto_value_label(s(&t("crypto-value-label")));
    tr.set_crypto_cost_label(s(&t("crypto-cost-label")));
    tr.set_crypto_no_trend(s(&t("crypto-no-trend")));
    tr.set_crypto_sync_daily(s(&t("crypto-sync-daily")));

    // Habit Heatmap
    tr.set_heatmap_less(s(&t("heatmap-less")));
    tr.set_heatmap_more(s(&t("heatmap-more")));

    // Habits Tab
    tr.set_habits_selected_hint(s(&t("habits-selected-hint")));

    // History Tab
    tr.set_history_total_achievements(s(&t("history-total-achievements")));

    // Streak Rewards
    tr.set_rewards_ready_claim(s(&t("rewards-ready-claim")));
    tr.set_rewards_next(s(&t("rewards-next")));
    tr.set_rewards_all_unlocked(s(&t("rewards-all-unlocked")));

    // Wallet Detail
    tr.set_wallet_no_holdings(s(&t("wallet-no-holdings")));

    // Icon Selector
    tr.set_icon_choose(s(&t("icon-choose")));
    tr.set_icon_exchanges(s(&t("icon-exchanges")));
    tr.set_icon_wallet_icons(s(&t("icon-wallet-icons")));

    // Forms
    tr.set_form_search_coin(s(&t("form-search-coin")));
    tr.set_form_date_format(s(&t("form-date-format")));
    tr.set_form_all(s(&t("form-all")));
    tr.set_form_habit(s(&t("form-habit")));

    // Modals - Add Account
    tr.set_modal_delete_account(s(&t("modal-delete-account")));
    tr.set_modal_save_account(s(&t("modal-save-account")));
    tr.set_modal_delete_account_confirm(s(&t("modal-delete-account-confirm")));

    // Modals - Add Transaction
    tr.set_modal_no_accounts(s(&t("modal-no-accounts")));

    // Modals - Add Crypto Transaction
    tr.set_modal_new_crypto_transaction(s(&t("modal-new-crypto-transaction")));
    tr.set_modal_save_transaction(s(&t("modal-save-transaction")));
    tr.set_modal_create_wallet_first(s(&t("modal-create-wallet-first")));
    tr.set_modal_create_another_wallet(s(&t("modal-create-another-wallet")));

    // Modals - Edit Crypto Transaction
    tr.set_modal_edit_crypto_transaction(s(&t("modal-edit-crypto-transaction")));
    tr.set_modal_save_changes(s(&t("modal-save-changes")));

    // Modals - Add Wallet
    tr.set_modal_new_wallet(s(&t("modal-new-wallet")));
    tr.set_modal_wallet_type(s(&t("modal-wallet-type")));
    tr.set_modal_create_wallet(s(&t("modal-create-wallet")));

    // Modals - Add Habit
    tr.set_modal_category(s(&t("modal-category")));
    tr.set_modal_color(s(&t("modal-color")));

    // Modals - Configure Categories
    tr.set_modal_category_settings(s(&t("modal-category-settings")));
    tr.set_modal_manage_categories(s(&t("modal-manage-categories")));
    tr.set_modal_expense_categories(s(&t("modal-expense-categories")));
    tr.set_modal_income_categories(s(&t("modal-income-categories")));
    tr.set_modal_no_expense_categories(s(&t("modal-no-expense-categories")));
    tr.set_modal_no_income_categories(s(&t("modal-no-income-categories")));
    tr.set_modal_add_new_category(s(&t("modal-add-new-category")));
    tr.set_modal_category_name(s(&t("modal-category-name")));
    tr.set_modal_default(s(&t("modal-default")));

    // Modals - Configure Ticker
    tr.set_modal_crypto_settings(s(&t("modal-crypto-settings")));
    tr.set_modal_manage_price_bar(s(&t("modal-manage-price-bar")));
    tr.set_modal_price_bar(s(&t("modal-price-bar")));
    tr.set_modal_remove(s(&t("modal-remove")));

    // Modals - Add Transaction
    tr.set_modal_account(s(&t("modal-account")));
    tr.set_modal_expense(s(&t("modal-expense")));
    tr.set_modal_income(s(&t("modal-income")));

    // Modals - Add Habit
    tr.set_modal_checkpoints(s(&t("modal-checkpoints")));
    tr.set_modal_checkpoint_desc(s(&t("modal-checkpoint-desc")));

    // Modals - Add Reward
    tr.set_modal_consecutive(s(&t("modal-consecutive")));
    tr.set_modal_accumulative(s(&t("modal-accumulative")));
    tr.set_modal_type(s(&t("modal-type")));
    tr.set_modal_milestones(s(&t("modal-milestones")));
    tr.set_modal_reward_placeholder(s(&t("modal-reward-placeholder")));

    // Modals - Configure Ticker Extended
    tr.set_modal_coin_catalog(s(&t("modal-coin-catalog")));
    tr.set_modal_max_coins(s(&t("modal-max-coins")));
    tr.set_modal_catalog_info(s(&t("modal-catalog-info")));
    tr.set_modal_coin_list(s(&t("modal-coin-list")));
    tr.set_modal_add_coin(s(&t("modal-add-coin")));
    tr.set_modal_removing_info(s(&t("modal-removing-info")));
    tr.set_modal_select_all(s(&t("modal-select-all")));
    tr.set_modal_remove_selected(s(&t("modal-remove-selected")));

    // Sidebar branding
    tr.set_sidebar_logo(s(&t("sidebar-logo")));
    tr.set_sidebar_title(s(&t("sidebar-title")));

    // Crypto Widgets
    tr.set_crypto_holdings_small(s(&t("crypto-holdings-small")));
    tr.set_crypto_price_small(s(&t("crypto-price-small")));

    // Crypto Transaction Modal
    tr.set_modal_from_asset(s(&t("modal-from-asset")));
    tr.set_modal_to_asset(s(&t("modal-to-asset")));
    tr.set_modal_cryptocurrency(s(&t("modal-cryptocurrency")));
    tr.set_modal_from_wallet(s(&t("modal-from-wallet")));
    tr.set_modal_to_wallet(s(&t("modal-to-wallet")));
    tr.set_modal_from_amount(s(&t("modal-from-amount")));
    tr.set_modal_to_amount(s(&t("modal-to-amount")));
    tr.set_modal_to_amount_optional(s(&t("modal-to-amount-optional")));
    tr.set_modal_same_as_from(s(&t("modal-same-as-from")));
    tr.set_modal_price_usd(s(&t("modal-price-usd")));
    tr.set_modal_optional(s(&t("modal-optional")));
    tr.set_modal_required(s(&t("modal-required")));
    tr.set_modal_fee_usd(s(&t("modal-fee-usd")));
    tr.set_modal_fee_coin_optional(s(&t("modal-fee-coin-optional")));
    tr.set_modal_fee_amount(s(&t("modal-fee-amount")));
    tr.set_modal_notes(s(&t("modal-notes")));
    tr.set_modal_transaction_details(s(&t("modal-transaction-details")));
    tr.set_modal_date(s(&t("modal-date")));
    tr.set_modal_fetch_price_date(s(&t("modal-fetch-price-date")));
    tr.set_modal_search_coins(s(&t("modal-search-coins")));

    // Section labels (crypto transaction modal)
    tr.set_section_asset_wallet(s(&t("section-asset-wallet")));
    tr.set_section_amount(s(&t("section-amount")));
    tr.set_section_advanced(s(&t("section-advanced")));
    tr.set_section_details(s(&t("section-details")));
    tr.set_section_fee_crypto(s(&t("section-fee-crypto")));
    tr.set_section_tax(s(&t("section-tax")));

    // Transaction summary
    tr.set_tx_summary_buying(s(&t("tx-summary-buying")));
    tr.set_tx_summary_selling(s(&t("tx-summary-selling")));
    tr.set_tx_summary_swapping(s(&t("tx-summary-swapping")));
    tr.set_tx_summary_moving(s(&t("tx-summary-moving")));
    tr.set_tx_summary_receiving(s(&t("tx-summary-receiving")));
    tr.set_tx_summary_sending(s(&t("tx-summary-sending")));
    tr.set_tx_summary_at(s(&t("tx-summary-at")));
    tr.set_tx_summary_per_coin(s(&t("tx-summary-per-coin")));
    tr.set_tx_summary_to(s(&t("tx-summary-to")));
    tr.set_tx_summary_from(s(&t("tx-summary-from")));

    // Category tabs (type selector)
    tr.set_tx_category_trade(s(&t("tx-category-trade")));
    tr.set_tx_category_transfer(s(&t("tx-category-transfer")));
    tr.set_tx_category_income(s(&t("tx-category-income")));
    tr.set_tx_category_expense(s(&t("tx-category-expense")));

    // Scenario labels (type selector sub-options)
    tr.set_tx_scenario_deposit(s(&t("tx-scenario-deposit")));
    tr.set_tx_scenario_withdrawal(s(&t("tx-scenario-withdrawal")));
    tr.set_tx_scenario_interest(s(&t("tx-scenario-interest")));
    tr.set_tx_scenario_gift(s(&t("tx-scenario-gift")));
    tr.set_tx_scenario_reward(s(&t("tx-scenario-reward")));
    tr.set_tx_scenario_other(s(&t("tx-scenario-other")));
    tr.set_tx_scenario_payment(s(&t("tx-scenario-payment")));
    tr.set_tx_scenario_donation(s(&t("tx-scenario-donation")));
    tr.set_tx_scenario_fee(s(&t("tx-scenario-fee")));
    tr.set_tx_scenario_lost(s(&t("tx-scenario-lost")));
    tr.set_tx_scenario_stolen(s(&t("tx-scenario-stolen")));
    tr.set_tx_scenario_buy(s(&t("tx-scenario-buy")));
    tr.set_tx_scenario_sell(s(&t("tx-scenario-sell")));
    tr.set_tx_scenario_swap(s(&t("tx-scenario-swap")));
    tr.set_tx_scenario_move(s(&t("tx-scenario-move")));
    tr.set_tx_scenario_airdrop(s(&t("tx-scenario-airdrop")));
    tr.set_tx_scenario_staking(s(&t("tx-scenario-staking")));
    tr.set_tx_scenario_mining(s(&t("tx-scenario-mining")));
    tr.set_tx_scenario_fork(s(&t("tx-scenario-fork")));

    // Type badge labels (edit modal)
    tr.set_tx_type_buy(s(&t("tx-type-buy")));
    tr.set_tx_type_sell(s(&t("tx-type-sell")));
    tr.set_tx_type_swap(s(&t("tx-type-swap")));
    tr.set_tx_type_transfer_in(s(&t("tx-type-transfer-in")));
    tr.set_tx_type_transfer_out(s(&t("tx-type-transfer-out")));

    // Goal Modal
    tr.set_modal_new_goal(s(&t("modal-new-goal")));
    tr.set_modal_edit_goal(s(&t("modal-edit-goal")));
    tr.set_modal_goal_name(s(&t("modal-goal-name")));
    tr.set_modal_goal_name_placeholder(s(&t("modal-goal-name-placeholder")));
    tr.set_modal_description_optional(s(&t("modal-description-optional")));
    tr.set_modal_goal_description_placeholder(s(&t("modal-goal-description-placeholder")));
    tr.set_modal_reward(s(&t("modal-reward")));
    tr.set_modal_reward_placeholder_goal(s(&t("modal-reward-placeholder-goal")));
    tr.set_modal_deadline_optional(s(&t("modal-deadline-optional")));
    tr.set_modal_create_goal(s(&t("modal-create-goal")));

    // Reward Modal
    tr.set_modal_new_streak_reward(s(&t("modal-new-streak-reward")));
    tr.set_modal_edit_reward(s(&t("modal-edit-reward")));
    tr.set_modal_consecutive_desc(s(&t("modal-consecutive-desc")));
    tr.set_modal_accumulative_desc(s(&t("modal-accumulative-desc")));
    tr.set_modal_target_days(s(&t("modal-target-days")));
    tr.set_modal_of_total_days(s(&t("modal-of-total-days")));
    tr.set_modal_days_label(s(&t("modal-days-label")));
    tr.set_modal_create_reward(s(&t("modal-create-reward")));

    // Configure Ticker Extended
    tr.set_modal_add_custom_coin(s(&t("modal-add-custom-coin")));
    tr.set_modal_coingecko_hint(s(&t("modal-coingecko-hint")));
    tr.set_modal_symbol_hint(s(&t("modal-symbol-hint")));
    tr.set_modal_coingecko_id(s(&t("modal-coingecko-id")));
    tr.set_modal_coingecko_id_placeholder(s(&t("modal-coingecko-id-placeholder")));
    tr.set_modal_name_placeholder(s(&t("modal-name-placeholder")));
    tr.set_modal_symbol(s(&t("modal-symbol")));
    tr.set_modal_symbol_placeholder(s(&t("modal-symbol-placeholder")));
    tr.set_modal_save_configuration(s(&t("modal-save-configuration")));

    // Wallet Modal
    tr.set_modal_wallet_name(s(&t("modal-wallet-name")));

    // Transfer Modal
    tr.set_modal_edit_transfer(s(&t("modal-edit-transfer")));
    tr.set_modal_from(s(&t("modal-from")));
    tr.set_modal_to(s(&t("modal-to")));
    tr.set_modal_transfer_action(s(&t("modal-transfer-action")));

    // Icon Modals
    tr.set_modal_select_bank_icon(s(&t("modal-select-bank-icon")));
    tr.set_modal_select_icon(s(&t("modal-select-icon")));
    tr.set_modal_save_icon(s(&t("modal-save-icon")));

    // Common Button Labels
    tr.set_button_add(s(&t("button-add")));
    tr.set_button_sync(s(&t("button-sync")));
    tr.set_button_syncing(s(&t("button-syncing")));
    tr.set_button_add_transaction(s(&t("button-add-transaction")));
    tr.set_button_add_transaction_short(s(&t("button-add-transaction-short")));
    tr.set_button_new_entry(s(&t("button-new-entry")));
    tr.set_button_new_account(s(&t("button-new-account")));

    // Page Titles and Sections
    tr.set_section_fiat(s(&t("section-fiat")));
    tr.set_section_spending_breakdown(s(&t("section-spending-breakdown")));
    tr.set_section_recent_activity(s(&t("section-recent-activity")));
    tr.set_section_recent_transactions(s(&t("section-recent-transactions")));
    tr.set_section_my_accounts(s(&t("section-my-accounts")));
    tr.set_section_finance_settings(s(&t("section-finance-settings")));
    tr.set_section_transactions(s(&t("section-transactions")));
    tr.set_section_wallet_breakdown(s(&t("section-wallet-breakdown")));

    // Settings Page
    tr.set_section_regional(s(&t("section-regional")));
    tr.set_section_data_sync(s(&t("section-data-sync")));
    tr.set_section_about(s(&t("section-about")));
    tr.set_settings_encryption_type(s(&t("settings-encryption-type")));
    tr.set_settings_storage_type(s(&t("settings-storage-type")));

    // Vault Backup
    tr.set_vault_backup_section(s(&t("vault-backup-section")));
    tr.set_vault_export_button(s(&t("vault-export-button")));
    tr.set_vault_restore_button(s(&t("vault-restore-button")));
    tr.set_vault_restore_from_backup(s(&t("vault-restore-from-backup")));
    tr.set_vault_export_success(s(&t("vault-export-success")));
    tr.set_vault_restore_success(s(&t("vault-restore-success")));
    tr.set_vault_export_failed(s(&t("vault-export-failed")));
    tr.set_vault_restore_failed(s(&t("vault-restore-failed")));
    tr.set_vault_restore_warning_title(s(&t("vault-restore-warning-title")));
    tr.set_vault_restore_warning_desc(s(&t("vault-restore-warning-desc")));
    tr.set_vault_restore_file_label(s(&t("vault-restore-file-label")));
    tr.set_vault_restore_cancel(s(&t("vault-restore-cancel")));
    tr.set_vault_restore_confirm(s(&t("vault-restore-confirm")));
    tr.set_vault_invalid_backup(s(&t("vault-invalid-backup")));
    tr.set_vault_backup_too_large(s(&t("vault-backup-too-large")));
    tr.set_vault_permission_denied(s(&t("vault-permission-denied")));
    tr.set_vault_backup_encryption_note(s(&t("vault-backup-encryption-note")));

    // Asset/Wallet Details
    tr.set_section_transaction_history(s(&t("section-transaction-history")));

    // Transaction Entry Modal
    tr.set_modal_new_entry(s(&t("modal-new-entry")));
    tr.set_modal_edit_entry(s(&t("modal-edit-entry")));
    tr.set_modal_save_entry(s(&t("modal-save-entry")));
    tr.set_modal_add_note(s(&t("modal-add-note")));

    // Finances Extended (Search/Empty States)
    tr.set_finances_search_placeholder(s(&t("finances-search-placeholder")));
    tr.set_finances_no_matching(s(&t("finances-no-matching")));
    tr.set_finances_no_transactions_yet(s(&t("finances-no-transactions-yet")));
    tr.set_finances_try_adjusting(s(&t("finances-try-adjusting")));
    tr.set_finances_add_first_entry(s(&t("finances-add-first-entry")));
    tr.set_finances_no_accounts_configured(s(&t("finances-no-accounts-configured")));
    tr.set_finances_create_account(s(&t("finances-create-account")));

    // Crypto Extended (Buttons)
    tr.set_crypto_add_wallet_button(s(&t("crypto-add-wallet-button")));

    // Habits Extended (Summary Labels)
    tr.set_habits_current_streak_label(s(&t("habits-current-streak-label")));
    tr.set_habits_best_streak_label(s(&t("habits-best-streak-label")));
    tr.set_habits_days_label(s(&t("habits-days-label")));
    tr.set_habits_completion_rate_label(s(&t("habits-completion-rate-label")));
    tr.set_habits_completions_label(s(&t("habits-completions-label")));

    // Rewards Extended (Sections/Buttons)
    tr.set_rewards_streak_rewards_section(s(&t("rewards-streak-rewards-section")));
    tr.set_rewards_add_reward_button(s(&t("rewards-add-reward-button")));
    tr.set_rewards_no_streak_rewards(s(&t("rewards-no-streak-rewards")));
    tr.set_rewards_link_habit_desc(s(&t("rewards-link-habit-desc")));
    tr.set_rewards_goals_section(s(&t("rewards-goals-section")));
    tr.set_rewards_add_goal_button(s(&t("rewards-add-goal-button")));
    tr.set_rewards_no_goals_set(s(&t("rewards-no-goals-set")));
    tr.set_rewards_create_goal_desc(s(&t("rewards-create-goal-desc")));

    // Rewards Progress
    tr.set_rewards_days_to_go(s(&t("rewards-days-to-go")));

    // History Tab
    tr.set_history_achievements_section(s(&t("history-achievements-section")));
    tr.set_history_no_achievements(s(&t("history-no-achievements")));
    tr.set_history_complete_to_earn(s(&t("history-complete-to-earn")));

    let crypto = ui.global::<CryptoAdapter>();
    let asset_count = crypto.get_portfolio().row_count();
    if asset_count == 0 {
        crypto.set_portfolio_summary(SharedString::from(""));
    } else {
        let wallet_count = crypto.get_wallets().row_count();
        let assets_str = asset_count.to_string();
        let wallets_str = wallet_count.to_string();
        let summary = i18n::t_args(
            "crypto-assets-across-wallets",
            &[
                ("assets", assets_str.as_str()),
                ("wallets", wallets_str.as_str()),
            ],
        );
        crypto.set_portfolio_summary(SharedString::from(summary));
    }
}

/// Helper to convert String to SharedString
#[inline]
fn s(text: &str) -> SharedString {
    SharedString::from(text)
}

/// Initializes i18n with the given language
pub fn init_translations(lang: &str) {
    i18n::init(lang);
}

/// Changes language and returns true if successful
pub fn change_language(lang: &str) -> bool {
    i18n::set_language(lang)
}

/// Gets current language code
pub fn get_current_language() -> String {
    i18n::current_language()
}

/// Detects system language
pub fn detect_system_language() -> String {
    i18n::detect_system_language()
}
