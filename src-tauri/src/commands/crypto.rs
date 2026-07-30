// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
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

//! Crypto domain Tauri commands.
//!
//! Most commands operate on the [`CryptoService`]. Amount formatting needs the
//! preferred currency (from [`SettingsService`]) and its USD exchange rate
//! (cached by [`FinanceService`]), so display-oriented commands also inject
//! those states. Exchange-rate persistence lives on [`FinanceService`].

use chrono::Local;
use sanctum::error::AppError;
use sanctum::features::crypto::{CryptoService, default_coin_catalog};
use sanctum::features::finance::FinanceService;
use sanctum::features::settings::{SETTING_PREFERRED_CURRENCY, SettingsService};
use sanctum::models::{CryptoTransaction, CryptoWallet};
use sanctum::ui::dto::crypto::{
    CoinCatalogDto, CryptoAssetPriceDto, CryptoSwapInput, CryptoTransactionDto,
    CryptoTransactionEditData, CryptoTransactionInput, CryptoTransactionListResponse,
    CryptoTransactionUpdateInput, CryptoTransferInput, CryptoTxFilterInput, DistributionItem,
    FxRateDto, IpcSummaryDto, PortfolioAssetDto, PortfolioResponse, PortfolioTrendData,
    TaxReportDto, TaxSettingsDto, TaxSummaryDto, WalletDetailResponse, WalletDto, WalletHoldingDto,
    WalletSimpleDto, WalletsResponse,
};
use std::collections::HashMap;
use tauri::State;

// ==================== Portfolio ====================

#[tauri::command]
pub fn fetch_portfolio(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
) -> Result<PortfolioResponse, AppError> {
    let assets = crypto.get_aggregated_portfolio().map_err(AppError::from)?;
    let prices = crypto.load_crypto_prices().unwrap_or_default();
    let catalog = crypto
        .get_coin_catalog()
        .unwrap_or_else(|_| default_coin_catalog());

    let price_map: HashMap<String, (f64, f64)> = prices
        .iter()
        .map(|p| {
            (
                p.id.clone(),
                (p.current_price, p.price_change_percentage_24h),
            )
        })
        .collect();
    let name_map: HashMap<String, (String, String)> = catalog
        .iter()
        .map(|c| (c.id.clone(), (c.name.clone(), c.symbol.clone())))
        .collect();

    let total_usd: f64 = assets
        .iter()
        .map(|a| {
            let price = price_map.get(&a.coin_id).map(|p| p.0).unwrap_or(0.0);
            a.total_amount * price
        })
        .sum();

    let portfolio_assets: Vec<PortfolioAssetDto> = assets
        .iter()
        .map(|a| {
            let (price, chg) = price_map.get(&a.coin_id).copied().unwrap_or((0.0, 0.0));
            let (name, sym) = name_map
                .get(&a.coin_id)
                .cloned()
                .unwrap_or_else(|| (a.coin_id.clone(), a.coin_id.clone()));
            let val = a.total_amount * price;
            let alloc = if total_usd > 0.0 {
                val / total_usd * 100.0
            } else {
                0.0
            };
            PortfolioAssetDto {
                coin_id: a.coin_id.clone(),
                symbol: sym,
                name,
                icon_path: None,
                price: fmt_pref(price, &settings, &finance),
                price_change_24h: format!("{:.2}%", chg),
                price_change_24h_negative: chg < 0.0,
                amount: trim_amount(a.total_amount),
                value: fmt_pref(val, &settings, &finance),
                allocation_pct: alloc,
            }
        })
        .collect();

    let distribution: Vec<DistributionItem> = assets
        .iter()
        .filter_map(|a| {
            let price = price_map.get(&a.coin_id).map(|p| p.0).unwrap_or(0.0);
            let val = a.total_amount * price;
            if val <= 0.0 {
                return None;
            }
            let sym = name_map
                .get(&a.coin_id)
                .map(|n| n.1.clone())
                .unwrap_or_else(|| a.coin_id.clone());
            Some(DistributionItem {
                coin_id: a.coin_id.clone(),
                symbol: sym,
                value: val,
                percentage: if total_usd > 0.0 {
                    val / total_usd * 100.0
                } else {
                    0.0
                },
            })
        })
        .collect();

    // Calculate unrealized PnL and ROI from aggregated assets
    let total_cost_basis: f64 = assets.iter().map(|a| a.total_cost_basis).sum();
    let unrealized_pnl = total_usd - total_cost_basis;
    let roi = if total_cost_basis > 0.0 {
        (unrealized_pnl / total_cost_basis) * 100.0
    } else {
        0.0
    };

    // Calculate realized gains YTD via tax summary
    let current_year = chrono::Local::now().format("%Y").to_string();
    let (realized_ytd_val, realized_ytd_neg) = crypto
        .generate_tax_summary(current_year)
        .map(|s| {
            (
                s.report.summary.total_gain,
                s.report.summary.total_gain < 0.0,
            )
        })
        .unwrap_or((0.0, false));

    // Save snapshot for portfolio trend chart
    let _ = crypto.save_crypto_portfolio_snapshot(total_usd, total_cost_basis);

    Ok(PortfolioResponse {
        total_value: fmt_pref(total_usd, &settings, &finance),
        unrealized_pnl: fmt_pref(unrealized_pnl.abs(), &settings, &finance),
        unrealized_pnl_negative: unrealized_pnl < 0.0,
        realized_ytd: fmt_pref(realized_ytd_val.abs(), &settings, &finance),
        realized_ytd_negative: realized_ytd_neg,
        roi: format!("{:.2}%", if roi == 0.0 { 0.0 } else { roi }),
        roi_negative: roi < 0.0,
        assets: portfolio_assets,
        distribution,
        fx_rate: build_fx_rate_badge(&settings, &finance),
        last_updated: None,
    })
}

/// Returns a sparse step-chart of historical portfolio value, anchored at the
/// dates we genuinely have data for: existing snapshots in range + a
/// carry-forward seed from the latest snapshot strictly before the range +
/// today's spot. Days without an anchor are omitted; the frontend renders the
/// gaps as horizontal "value held" segments via `step: 'end'`.
#[tauri::command]
pub fn fetch_portfolio_trend(
    crypto: State<'_, CryptoService>,
    days: i64,
) -> Result<PortfolioTrendData, AppError> {
    let today = Local::now().date_naive();
    let start = today - chrono::Duration::days(days.max(1) - 1);
    let start_str = start.format("%Y-%m-%d").to_string();
    let today_str = today.format("%Y-%m-%d").to_string();

    // Today's value from current spot prices (always the most fresh anchor).
    let assets = crypto.get_aggregated_portfolio().map_err(AppError::from)?;
    let prices = crypto.load_crypto_prices().unwrap_or_default();
    let price_map: HashMap<String, f64> = prices
        .iter()
        .map(|p| (p.id.clone(), p.current_price))
        .collect();
    let today_value: f64 = assets
        .iter()
        .map(|a| a.total_amount * price_map.get(&a.coin_id).copied().unwrap_or(0.0))
        .sum();

    // All snapshots ever (one row per day max — bounded by app lifetime).
    // We need everything because we want the latest snapshot strictly before
    // the range to seed a carry-forward starting value.
    let all_snaps = crypto
        .get_crypto_portfolio_snapshots(36_500)
        .unwrap_or_default();

    // Anchor map keyed by date so duplicates collapse cleanly.
    let mut anchors: HashMap<String, f64> = HashMap::new();

    // Seed: latest snapshot before the range carries forward to start_date.
    if let Some((_, v, _)) = all_snaps
        .iter()
        .rfind(|(d, _, _)| d.as_str() < start_str.as_str())
    {
        anchors.insert(start_str.clone(), *v);
    }

    // Snapshots within [start, today].
    for (d, v, _) in &all_snaps {
        if d.as_str() >= start_str.as_str() && d.as_str() <= today_str.as_str() {
            anchors.insert(d.clone(), *v);
        }
    }

    // Today: always present, always uses fresh spot (overrides any stale snapshot).
    if today_value > 0.0 || !anchors.is_empty() {
        anchors.insert(today_str.clone(), today_value);
    }

    if anchors.is_empty() {
        return Ok(PortfolioTrendData {
            dates: vec![],
            values: vec![],
        });
    }

    let mut sorted: Vec<(String, f64)> = anchors.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let (dates, values): (Vec<_>, Vec<_>) = sorted.into_iter().unzip();
    Ok(PortfolioTrendData { dates, values })
}

// ==================== Wallets ====================

#[tauri::command]
pub fn fetch_wallets(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
) -> Result<WalletsResponse, AppError> {
    let wallets = crypto.get_wallets().map_err(AppError::from)?;
    let prices = crypto.load_crypto_prices().unwrap_or_default();
    let pm: HashMap<String, f64> = prices
        .into_iter()
        .map(|p| (p.id, p.current_price))
        .collect();

    let dtos: Vec<WalletDto> = wallets
        .iter()
        .map(|w| {
            let holdings = crypto.get_wallet_holdings(w.id.clone()).unwrap_or_default();
            let total: f64 = holdings
                .iter()
                .map(|h| h.total_amount * pm.get(&h.coin_id).copied().unwrap_or(0.0))
                .sum();
            WalletDto {
                id: w.id.clone(),
                name: w.name.clone(),
                category: w.category.clone(),
                icon_path: w.icon.clone(),
                total_value: fmt_pref(total, &settings, &finance),
                assets_count: holdings.len() as i32,
            }
        })
        .collect();
    let simple: Vec<WalletSimpleDto> = wallets
        .iter()
        .map(|w| WalletSimpleDto {
            id: w.id.clone(),
            name: w.name.clone(),
            category: w.category.clone(),
        })
        .collect();
    Ok(WalletsResponse {
        wallets: dtos,
        simple_list: simple,
    })
}

#[tauri::command]
pub fn fetch_wallet_detail(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
    wallet_id: String,
) -> Result<WalletDetailResponse, AppError> {
    let wallets = crypto.get_wallets().map_err(AppError::from)?;
    let wallet = wallets
        .iter()
        .find(|w| w.id == wallet_id)
        .ok_or_else(|| AppError::not_found("Wallet not found"))?;
    let holdings = crypto
        .get_wallet_holdings(wallet_id.clone())
        .map_err(AppError::from)?;
    let prices = crypto.load_crypto_prices().unwrap_or_default();
    let pm: HashMap<String, f64> = prices
        .into_iter()
        .map(|p| (p.id, p.current_price))
        .collect();

    let hdtos: Vec<WalletHoldingDto> = holdings
        .iter()
        .map(|h| {
            let price = pm.get(&h.coin_id).copied().unwrap_or(0.0);
            WalletHoldingDto {
                coin_id: h.coin_id.clone(),
                symbol: h.symbol.clone(),
                amount: trim_amount(h.total_amount),
                value: fmt_pref(h.total_amount * price, &settings, &finance),
                price: fmt_pref(price, &settings, &finance),
            }
        })
        .collect();

    let total: f64 = holdings
        .iter()
        .map(|h| h.total_amount * pm.get(&h.coin_id).copied().unwrap_or(0.0))
        .sum();

    let txs = crypto
        .get_wallet_transactions(wallet_id)
        .map_err(AppError::from)?;
    let tx_dtos = map_crypto_transactions(&txs, &wallets, &settings, &finance);

    Ok(WalletDetailResponse {
        id: wallet.id.clone(),
        name: wallet.name.clone(),
        category: wallet.category.clone(),
        icon_path: wallet.icon.clone(),
        total_value: fmt_pref(total, &settings, &finance),
        holdings: hdtos,
        transactions: tx_dtos,
    })
}

#[tauri::command]
pub fn add_wallet(
    crypto: State<'_, CryptoService>,
    name: String,
    category: String,
    icon: Option<String>,
) -> Result<String, AppError> {
    crypto
        .add_wallet(name, category, icon)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn delete_wallet(
    crypto: State<'_, CryptoService>,
    id: String,
    force: bool,
) -> Result<(), AppError> {
    crypto.delete_wallet(id, force).map_err(AppError::from)
}

#[tauri::command]
pub fn get_wallet_transaction_count(
    crypto: State<'_, CryptoService>,
    id: String,
) -> Result<usize, AppError> {
    crypto
        .get_wallet_transaction_count(id)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn update_wallet_name(
    crypto: State<'_, CryptoService>,
    id: String,
    new_name: String,
) -> Result<(), AppError> {
    crypto
        .update_wallet_name(id, new_name)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn update_wallet_icon(
    crypto: State<'_, CryptoService>,
    id: String,
    icon: Option<String>,
) -> Result<(), AppError> {
    crypto.update_wallet_icon(id, icon).map_err(AppError::from)
}

// ==================== Transactions ====================

#[tauri::command]
pub fn add_crypto_transaction(
    crypto: State<'_, CryptoService>,
    input: CryptoTransactionInput,
) -> Result<String, AppError> {
    Ok(crypto.add_crypto_transaction(input.into_command()?)?)
}

#[tauri::command]
pub fn add_crypto_transfer(
    crypto: State<'_, CryptoService>,
    input: CryptoTransferInput,
) -> Result<String, AppError> {
    Ok(crypto.add_crypto_transfer(input.into_command()?)?)
}

#[tauri::command]
pub fn add_crypto_swap(
    crypto: State<'_, CryptoService>,
    input: CryptoSwapInput,
) -> Result<String, AppError> {
    Ok(crypto.add_crypto_swap(input.into_command()?)?)
}

#[tauri::command]
pub fn update_crypto_transaction(
    crypto: State<'_, CryptoService>,
    input: CryptoTransactionUpdateInput,
) -> Result<(), AppError> {
    Ok(crypto.update_crypto_transaction(input.into_command()?)?)
}

#[tauri::command]
pub fn delete_crypto_transaction(
    crypto: State<'_, CryptoService>,
    id: String,
) -> Result<(), AppError> {
    crypto.delete_crypto_transaction(id).map_err(AppError::from)
}

/// Copy a transaction into a new one dated today. Returns the new id.
#[tauri::command]
pub fn duplicate_crypto_transaction(
    crypto: State<'_, CryptoService>,
    id: String,
) -> Result<String, AppError> {
    crypto
        .duplicate_crypto_transaction(id)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn get_crypto_transaction(
    crypto: State<'_, CryptoService>,
    id: String,
) -> Result<CryptoTransactionEditData, AppError> {
    let tx = crypto
        .get_crypto_transaction(id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Transaction not found"))?;
    let wallets = crypto.get_wallets().map_err(AppError::from)?;
    let wn = wallets
        .iter()
        .find(|w| w.id == tx.wallet_id)
        .map(|w| w.name.clone())
        .unwrap_or_default();
    Ok(CryptoTransactionEditData {
        id: tx.id,
        wallet_name: wn,
        coin_id: tx.coin_id,
        symbol: tx.symbol,
        transaction_type: tx.transaction_type,
        subtype: tx.subtype,
        amount: tx.amount.to_string(),
        price: tx.price_per_coin.map(|p| p.to_string()).unwrap_or_default(),
        fee: tx.fee.map(|f| f.to_string()).unwrap_or_default(),
        fee_coin_id: tx.fee_coin_id,
        fee_coin_amount: tx.fee_amount.map(|f| f.to_string()),
        date: tx.date,
        notes: tx.notes,
        override_proceeds: tx.override_proceeds.map(|f| f.to_string()),
        override_cost_basis: tx.override_cost_basis.map(|f| f.to_string()),
        is_paired_swap: tx.related_tx_id.is_some(),
    })
}

#[tauri::command]
pub fn get_crypto_transactions_by_coin(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
    coin_id: String,
) -> Result<Vec<CryptoTransactionDto>, AppError> {
    let txs = crypto
        .get_crypto_transactions_by_coin(coin_id)
        .map_err(AppError::from)?;
    let wallets = crypto.get_wallets().map_err(AppError::from)?;
    Ok(map_crypto_transactions(&txs, &wallets, &settings, &finance))
}

#[tauri::command]
pub fn fetch_all_crypto_transactions(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
    offset: i64,
    limit: i64,
    filter: Option<CryptoTxFilterInput>,
) -> Result<CryptoTransactionListResponse, AppError> {
    let txs = crypto
        .get_filtered_crypto_transactions(filter.unwrap_or_default().into(), offset, limit)
        .map_err(AppError::from)?;
    let wallets = crypto.get_wallets().map_err(AppError::from)?;
    let total = txs.len();
    let effective = total.min(limit as usize);
    let has_more = total > effective;
    let mut dtos = map_crypto_transactions(&txs[..effective], &wallets, &settings, &finance);
    dtos.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(CryptoTransactionListResponse {
        transactions: dtos,
        has_more,
    })
}

// ==================== Catalog ====================

#[tauri::command]
pub fn get_coin_catalog(crypto: State<'_, CryptoService>) -> Result<Vec<CoinCatalogDto>, AppError> {
    let catalog = crypto
        .get_coin_catalog()
        .unwrap_or_else(|_| default_coin_catalog());
    let favorites = crypto.get_favorite_coin_ids();
    let custom = crypto.get_custom_coin_catalog().unwrap_or_default();
    let custom_ids: std::collections::HashSet<String> = custom.into_iter().map(|c| c.id).collect();
    Ok(catalog
        .into_iter()
        .map(|c| CoinCatalogDto {
            is_favorite: favorites.contains(&c.id),
            is_custom: custom_ids.contains(&c.id),
            id: c.id,
            name: c.name,
            symbol: c.symbol,
        })
        .collect())
}

#[tauri::command]
pub fn set_favorite_coin(
    crypto: State<'_, CryptoService>,
    id: String,
    favorite: bool,
) -> Result<(), AppError> {
    crypto
        .set_favorite_coin(id, favorite)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn add_custom_coin(
    crypto: State<'_, CryptoService>,
    id: String,
    name: String,
    symbol: String,
) -> Result<(), AppError> {
    crypto
        .add_custom_coin(id, name, symbol)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn delete_custom_coin(crypto: State<'_, CryptoService>, id: String) -> Result<(), AppError> {
    crypto.delete_custom_coin(id).map_err(AppError::from)
}

#[tauri::command]
pub fn get_active_ticker_ids(crypto: State<'_, CryptoService>) -> Vec<String> {
    crypto.get_active_ticker_ids()
}

#[tauri::command]
pub fn save_active_ticker_ids(
    crypto: State<'_, CryptoService>,
    ids: Vec<String>,
) -> Result<(), AppError> {
    crypto.save_active_ticker_ids(ids).map_err(AppError::from)
}

// ==================== Prices ====================

#[tauri::command]
pub fn save_crypto_prices(
    crypto: State<'_, CryptoService>,
    prices: Vec<CryptoAssetPriceDto>,
) -> Result<(), AppError> {
    let internal: Vec<sanctum::models::CryptoAsset> = prices
        .into_iter()
        .map(|p| sanctum::models::CryptoAsset {
            id: p.id,
            symbol: p.symbol,
            name: p.name,
            current_price: p.current_price,
            price_change_percentage_24h: p.price_change_percentage_24h,
            last_updated: p.last_updated,
        })
        .collect();
    crypto.save_crypto_prices(internal).map_err(AppError::from)
}

#[tauri::command]
pub fn load_crypto_prices(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
) -> Result<Vec<CryptoAssetPriceDto>, AppError> {
    let prices = crypto.load_crypto_prices().map_err(AppError::from)?;
    Ok(prices
        .into_iter()
        .map(|p| {
            let display = fmt_pref(p.current_price, &settings, &finance);
            CryptoAssetPriceDto {
                id: p.id,
                symbol: p.symbol,
                name: p.name,
                current_price: p.current_price,
                current_price_display: display,
                price_change_percentage_24h: p.price_change_percentage_24h,
                last_updated: p.last_updated,
            }
        })
        .collect())
}

#[tauri::command]
pub fn get_monitored_coin_ids(crypto: State<'_, CryptoService>) -> Result<Vec<String>, AppError> {
    crypto.get_monitored_coin_ids().map_err(AppError::from)
}

#[tauri::command]
pub async fn sync_crypto_data(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
) -> Result<String, AppError> {
    // 1. Fetch and save crypto prices
    let ids = crypto.get_monitored_coin_ids().map_err(AppError::from)?;
    if !ids.is_empty() {
        let prices = crypto
            .get_crypto_prices(ids)
            .await
            .map_err(AppError::from)?;
        crypto.save_crypto_prices(prices).map_err(AppError::from)?;
    }

    // 2. Fetch and save CLP rate using backend provider (Mindicador/USDT fallback)
    match crypto.get_usd_fx_rate("CLP".to_string()).await {
        Ok(rate) => {
            let _ = finance.save_exchange_rate("CLP_USD".to_string(), rate);
        }
        Err(_) => {
            // Soft failure for fx rate, not fatal
        }
    }

    Ok("Synced successfully".to_string())
}

// ==================== Tax ====================

#[tauri::command]
pub fn load_tax_settings(
    crypto: State<'_, CryptoService>,
    period_id: String,
) -> Result<TaxSettingsDto, AppError> {
    let s = crypto
        .load_tax_settings(period_id)
        .map_err(AppError::from)?;
    Ok(TaxSettingsDto {
        period_id: s.period_id,
        jurisdiction: s.jurisdiction.as_str().to_string(),
        method: s.method.as_str().to_string(),
        include_swaps: s.include_swaps,
        include_fee_crypto: s.include_fee_crypto,
        excluded_wallet_ids: s.excluded_wallet_ids,
    })
}

#[tauri::command]
pub fn save_tax_settings(
    crypto: State<'_, CryptoService>,
    settings: TaxSettingsDto,
) -> Result<(), AppError> {
    let internal = sanctum::features::crypto::TaxPeriodSettings {
        period_id: settings.period_id,
        jurisdiction: sanctum::features::crypto::TaxJurisdiction::parse_or_default(
            &settings.jurisdiction,
        ),
        method: sanctum::features::crypto::TaxMethod::parse_or_default(&settings.method),
        include_swaps: settings.include_swaps,
        include_fee_crypto: settings.include_fee_crypto,
        excluded_wallet_ids: settings.excluded_wallet_ids,
    };
    crypto.save_tax_settings(internal).map_err(AppError::from)
}

#[tauri::command]
pub fn generate_tax_report(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
    period_id: String,
) -> Result<TaxReportDto, AppError> {
    let summary = crypto
        .generate_tax_summary(period_id.clone())
        .map_err(AppError::from)?;
    Ok(map_tax_report_dto(
        period_id,
        summary.report,
        summary.readiness,
        &settings,
        &finance,
    ))
}

#[tauri::command]
pub fn generate_tax_summary(
    crypto: State<'_, CryptoService>,
    finance: State<'_, FinanceService>,
    settings: State<'_, SettingsService>,
    period_id: String,
) -> Result<TaxSummaryDto, AppError> {
    let payload = crypto
        .generate_tax_summary(period_id.clone())
        .map_err(AppError::from)?;
    let override_curr = if payload.report.jurisdiction == "chile" {
        Some("CLP")
    } else {
        None
    };
    let report = map_tax_report_dto(
        period_id,
        payload.report,
        payload.readiness,
        &settings,
        &finance,
    );
    let end_balance_value = payload
        .end_balance_value
        .map(|v| fmt_pref_override(v, override_curr, &settings, &finance));

    Ok(TaxSummaryDto {
        report,
        taxable_income_total: fmt_pref_override(
            payload.taxable_income_total,
            override_curr,
            &settings,
            &finance,
        ),
        taxable_income_count: payload.taxable_income_count,
        end_balance_value,
        end_balance_missing: payload.end_balance_missing,
        transactions_in_period: payload.transactions_in_period,
        volume_processed: fmt_pref_override(
            payload.volume_processed,
            override_curr,
            &settings,
            &finance,
        ),
    })
}

#[tauri::command]
pub async fn get_crypto_historical_price_usd(
    crypto: State<'_, CryptoService>,
    coin_id: String,
    date: String,
) -> Result<f64, AppError> {
    crypto
        .get_historical_price_usd(coin_id, date)
        .await
        .map_err(AppError::from)
}

fn map_tax_report_dto(
    period_id: String,
    r: sanctum::features::crypto::TaxReport,
    readiness: Vec<sanctum::features::crypto::TaxReadinessItem>,
    settings: &SettingsService,
    finance: &FinanceService,
) -> TaxReportDto {
    // Backend serializes jurisdiction via `TaxJurisdiction::as_str()` → "chile"/"usa"/"other".
    let override_curr = if r.jurisdiction == "chile" {
        Some("CLP")
    } else {
        None
    };

    TaxReportDto {
        period_id,
        jurisdiction: r.jurisdiction,
        method: r.method,
        disposals_count: r.summary.disposals,
        total_proceeds: fmt_pref_override(
            r.summary.total_proceeds,
            override_curr,
            settings,
            finance,
        ),
        total_cost: fmt_pref_override(r.summary.total_cost, override_curr, settings, finance),
        total_gain: fmt_pref_override(r.summary.total_gain.abs(), override_curr, settings, finance),
        total_gain_negative: r.summary.total_gain < 0.0,
        short_term_gain: r
            .summary
            .short_term_gain
            .map(|v| fmt_pref_override(v, override_curr, settings, finance)),
        long_term_gain: r
            .summary
            .long_term_gain
            .map(|v| fmt_pref_override(v, override_curr, settings, finance)),
        events: r
            .disposals
            .into_iter()
            .map(|e| sanctum::ui::dto::crypto::TaxEventDto {
                tx_id: e.tx_id,
                date: e.date,
                coin_id: e.coin_id,
                symbol: e.symbol,
                amount: e.amount.to_string(),
                proceeds: fmt_pref_override(e.proceeds, override_curr, settings, finance),
                cost_basis: fmt_pref_override(e.cost_basis, override_curr, settings, finance),
                gain: fmt_pref_override(e.gain.abs(), override_curr, settings, finance),
                gain_negative: e.gain < 0.0,
                term: e.term,
                disposal_type: e.disposal_type,
            })
            .collect(),
        warnings: r
            .warnings
            .into_iter()
            .map(|w| sanctum::ui::dto::crypto::TaxWarningDto {
                code: w.code,
                message: w.message,
                tx_id: w.tx_id,
            })
            .collect(),
        readiness: readiness
            .into_iter()
            .map(|item| sanctum::ui::dto::crypto::TaxReadinessDto {
                code: item.code,
                status: item.status,
                detail: item.detail,
            })
            .collect(),
    }
}

#[tauri::command]
pub fn export_tax_report_csv(
    crypto: State<'_, CryptoService>,
    period_id: String,
    path: String,
) -> Result<(), AppError> {
    crypto
        .export_tax_report_csv(period_id, &path)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn export_tax_history_csv(
    crypto: State<'_, CryptoService>,
    period_id: String,
    path: String,
) -> Result<(), AppError> {
    crypto
        .export_tax_history_csv(period_id, &path)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn import_ipc_csv(
    crypto: State<'_, CryptoService>,
    content: String,
) -> Result<IpcSummaryDto, AppError> {
    let s = crypto.import_ipc_csv(&content).map_err(AppError::from)?;
    let range = match (&s.first_period, &s.last_period) {
        (Some(f), Some(l)) => Some(format!("{f} - {l}")),
        _ => None,
    };
    Ok(IpcSummaryDto {
        records_count: s.inserted,
        date_range: range,
    })
}

#[tauri::command]
pub fn get_ipc_summary(
    crypto: State<'_, CryptoService>,
) -> Result<Option<IpcSummaryDto>, AppError> {
    let s = crypto.get_ipc_summary().map_err(AppError::from)?;
    Ok(s.map(|s| IpcSummaryDto {
        records_count: s.count,
        date_range: Some(format!("{} - {}", s.first_period, s.last_period)),
    }))
}

#[tauri::command]
pub fn fill_missing_tax_prices(
    crypto: State<'_, CryptoService>,
    tx_id: String,
    price_per_coin: Option<f64>,
    fee_usd: Option<f64>,
    override_proceeds: Option<f64>,
) -> Result<bool, AppError> {
    crypto
        .fill_missing_tax_price_fields(tx_id, price_per_coin, fee_usd, override_proceeds)
        .map_err(AppError::from)
}

// ==================== Exchange Rates ====================

#[tauri::command]
pub fn save_exchange_rate(
    finance: State<'_, FinanceService>,
    pair: String,
    rate: f64,
) -> Result<(), AppError> {
    finance
        .save_exchange_rate(pair, rate)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn load_exchange_rate(
    finance: State<'_, FinanceService>,
    pair: String,
) -> Result<Option<(f64, String)>, AppError> {
    finance
        .load_exchange_rate_allow_stale(pair)
        .map_err(AppError::from)
}

// ==================== Helpers ====================

fn trim_amount(v: f64) -> String {
    format!("{:.8}", v)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn fmt_pref(v: f64, settings: &SettingsService, finance: &FinanceService) -> String {
    fmt_pref_override(v, None, settings, finance)
}

fn fmt_pref_override(
    v: f64,
    override_curr: Option<&str>,
    settings: &SettingsService,
    finance: &FinanceService,
) -> String {
    let pref = override_curr.map(|s| s.to_string()).unwrap_or_else(|| {
        settings
            .get_app_setting(SETTING_PREFERRED_CURRENCY)
            .unwrap_or_else(|_| "USD".to_string())
            .trim()
            .to_uppercase()
    });

    let n = if v == 0.0 { 0.0 } else { v };
    let mut amount = n;

    if pref != "USD" {
        let pair = format!("{}_USD", pref);
        if let Ok(Some((rate, _))) = finance.load_exchange_rate_allow_stale(pair)
            && rate > 0.0
        {
            amount = n * rate;
        }
    }

    sanctum::ui::currency::format_preferred(amount, &pref)
}

fn build_fx_rate_badge(settings: &SettingsService, finance: &FinanceService) -> Option<FxRateDto> {
    let preferred = settings
        .get_app_setting(SETTING_PREFERRED_CURRENCY)
        .unwrap_or_else(|_| "USD".to_string())
        .trim()
        .to_uppercase();
    let target = if preferred == "USD" {
        "CLP".to_string()
    } else {
        preferred
    };
    let pair = format!("{}_USD", target);
    finance
        .load_exchange_rate_allow_stale(pair)
        .ok()
        .flatten()
        .filter(|(rate, _)| *rate > 0.0)
        .map(|(rate, _)| FxRateDto {
            pair: format!("USD/{}", target),
            rate: format!("{:.2}", rate),
            is_live: true,
        })
}

fn map_crypto_transactions(
    txs: &[CryptoTransaction],
    wallets: &[CryptoWallet],
    settings: &SettingsService,
    finance: &FinanceService,
) -> Vec<CryptoTransactionDto> {
    let wn: HashMap<String, String> = wallets
        .iter()
        .map(|w| (w.id.clone(), w.name.clone()))
        .collect();
    txs.iter()
        .map(|tx| CryptoTransactionDto {
            id: tx.id.clone(),
            wallet_id: tx.wallet_id.clone(),
            wallet_name: wn.get(&tx.wallet_id).cloned().unwrap_or_default(),
            coin_id: tx.coin_id.clone(),
            symbol: tx.symbol.clone(),
            transaction_type: tx.transaction_type.clone(),
            subtype: tx.subtype.clone(),
            amount: trim_amount(tx.amount),
            price: tx
                .price_per_coin
                .map(|p| fmt_pref(p, settings, finance))
                .unwrap_or_default(),
            fee: tx
                .fee
                .map(|f| fmt_pref(f, settings, finance))
                .unwrap_or_else(|| fmt_pref(0.0, settings, finance)),
            fee_coin_id: tx.fee_coin_id.clone(),
            fee_amount: tx.fee_amount.map(trim_amount),
            value: fmt_pref(
                tx.amount * tx.price_per_coin.unwrap_or(0.0),
                settings,
                finance,
            ),
            date: tx.date.clone(),
            notes: tx.notes.clone(),
            has_related_tx: tx.related_tx_id.is_some(),
        })
        .collect()
}
