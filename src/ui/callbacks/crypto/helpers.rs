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

//! Shared helpers for crypto callbacks

use crate::controller::{AppController, SETTING_PREFERRED_CURRENCY};
use crate::models::CryptoAsset;
use crate::services::i18n;
use crate::ui::{
    convert_usd_to_preferred, crypto_icon_for_symbol, format_fx_rate, format_preferred,
    load_wallet_icon,
};
use crate::{
    AppWindow, CryptoAdapter, CryptoAssetData, CryptoDistributionSlice, CryptoWalletData,
    WalletSimple,
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::Arc;

/// App setting keys
pub const SETTING_CRYPTO_LAST_WALLET_ID: &str = "crypto_last_wallet_id";
pub const SETTING_CRYPTO_LAST_COIN_ID: &str = "crypto_last_coin_id";
pub const SETTING_CRYPTO_LAST_UPDATED: &str = "crypto_last_updated";

fn normalize_currency_code(code: &str) -> String {
    code.trim().to_uppercase()
}

pub(super) fn format_compact_asset_amount(amount: f64) -> String {
    if !amount.is_finite() {
        return "0".to_string();
    }

    if amount.abs() > 0.0 && amount.abs() < 0.00001 {
        return format!("{:.2e}", amount);
    }

    let precision = if amount.abs() >= 1000.0 {
        2_i32
    } else if amount.abs() >= 1.0 {
        4_i32
    } else {
        6_i32
    };

    let factor = 10_f64.powi(precision);
    let truncated = (amount * factor).trunc() / factor;
    let mut formatted = format!("{:.*}", precision as usize, truncated);
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }

    if formatted == "-0" {
        "0".to_string()
    } else {
        formatted
    }
}

pub(super) fn format_compact_price_preferred(price_preferred: f64, preferred_currency: &str) -> String {
    if !price_preferred.is_finite() || price_preferred <= 0.0 {
        return "N/A".to_string();
    }

    let currency = normalize_currency_code(preferred_currency);
    if price_preferred >= 1.0 {
        return format_preferred(price_preferred, &currency);
    }

    if price_preferred < 0.0001 {
        return format!("{currency} {:.2e}", price_preferred);
    }

    let precision = if price_preferred >= 0.01 { 4 } else { 6 };
    let mut formatted = format!("{:.*}", precision, price_preferred);
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    format!("{currency} {formatted}")
}

pub fn resolve_preferred_currency(controller: &AppController) -> String {
    normalize_currency_code(
        &controller
            .get_app_setting(SETTING_PREFERRED_CURRENCY)
            .unwrap_or_else(|_| "USD".to_string()),
    )
}

pub fn badge_currency_for_preferred(preferred_currency: &str) -> String {
    let preferred = normalize_currency_code(preferred_currency);
    if preferred == "USD" {
        "CLP".to_string()
    } else {
        preferred
    }
}

pub fn usd_pair_for_target_currency(target_currency: &str) -> String {
    format!("{}_USD", normalize_currency_code(target_currency))
}

pub fn load_cached_usd_rate(controller: &AppController, target_currency: &str) -> f64 {
    let target = normalize_currency_code(target_currency);
    if target == "USD" {
        return 1.0;
    }
    let pair = usd_pair_for_target_currency(&target);
    controller
        .load_exchange_rate_allow_stale(pair)
        .ok()
        .and_then(|r| r.map(|(rate, _)| rate))
        .filter(|rate| *rate > 0.0)
        .unwrap_or(1.0)
}

pub fn load_preferred_usd_rate(controller: &AppController, preferred_currency: &str) -> f64 {
    load_cached_usd_rate(controller, preferred_currency)
}

pub fn load_crypto_badge_state(controller: &AppController) -> (String, String) {
    let preferred_currency = resolve_preferred_currency(controller);
    let target = badge_currency_for_preferred(&preferred_currency);
    let label = format!("USD/{}", target);
    let pair = usd_pair_for_target_currency(&target);
    let value = controller
        .load_exchange_rate_allow_stale(pair)
        .ok()
        .flatten()
        .and_then(|(rate, _)| {
            if rate > 0.0 {
                Some(format_fx_rate(rate, &target))
            } else {
                None
            }
        })
        .unwrap_or_else(|| "N/A".to_string());
    (label, value)
}

fn set_portfolio_summary(adapter: &CryptoAdapter, assets: usize, wallets: usize) {
    let assets_str = assets.to_string();
    let wallets_str = wallets.to_string();
    let summary = i18n::t_args(
        "crypto-assets-across-wallets",
        &[("assets", assets_str.as_str()), ("wallets", wallets_str.as_str())],
    );
    adapter.set_portfolio_summary(SharedString::from(summary));
}

fn format_signed_preferred(value_usd: f64, preferred_currency: &str, usd_rate: f64) -> (String, bool) {
    let positive = value_usd >= 0.0;
    let value_preferred = convert_usd_to_preferred(value_usd.abs(), preferred_currency, usd_rate);
    let sign = if positive { "+" } else { "-" };
    (
        format!(
            "{} {}",
            sign,
            format_preferred(value_preferred, preferred_currency)
        ),
        positive,
    )
}

fn format_roi(total_value: f64, total_cost: f64) -> String {
    if total_cost <= f64::EPSILON {
        return "N/A".to_string();
    }
    let roi = ((total_value - total_cost) / total_cost) * 100.0;
    if roi >= 0.0 {
        format!("+ {:.2}%", roi)
    } else {
        format!("{:.2}%", roi)
    }
}

/// Reload wallets list
/// Optionally accepts a notify closure to report errors
pub fn reload_wallets<N>(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, notify: Option<&N>)
where
    N: Fn(String, bool),
{
    let wallets = match controller.get_wallets() {
        Ok(w) => w,
        Err(e) => {
            if let Some(n) = notify {
                n(format!("Failed to load wallets: {}", e), true);
            }
            return;
        }
    };

    // Load preferred currency and exchange rate
    let preferred_currency = resolve_preferred_currency(controller);
    let usd_rate = load_preferred_usd_rate(controller, &preferred_currency);

    let mut wallet_data: Vec<CryptoWalletData> = Vec::new();
    let mut wallet_simple: Vec<WalletSimple> = Vec::new();

    let prices = controller.load_crypto_prices().unwrap_or_default();
    let price_map: HashMap<String, f64> = prices
        .into_iter()
        .map(|p| (p.id, p.current_price))
        .collect();

    for w in wallets {
        wallet_simple.push(WalletSimple {
            id: SharedString::from(&w.id),
            name: SharedString::from(&w.name),
        });

        let holdings = controller
            .get_wallet_holdings(w.id.clone())
            .unwrap_or_default();
        let total_bal_usd: f64 = holdings
            .iter()
            .map(|h| {
                let price = price_map.get(&h.coin_id).cloned().unwrap_or(0.0);
                h.total_amount * price
            })
            .sum();

        let total_bal = convert_usd_to_preferred(total_bal_usd, &preferred_currency, usd_rate);

        wallet_data.push(CryptoWalletData {
            id: SharedString::from(w.id),
            name: SharedString::from(w.name),
            category: SharedString::from(w.category.clone()),
            icon: load_wallet_icon(w.icon.clone(), &w.category),
            balance: SharedString::from(format_preferred(total_bal, &preferred_currency)),
            asset_count: holdings.len() as i32,
        });
    }

    if let Some(ui) = ui_weak.upgrade() {
        let last_wallet_id = controller
            .get_app_setting(SETTING_CRYPTO_LAST_WALLET_ID)
            .ok()
            .filter(|val| !val.is_empty());
        let last_wallet_index = last_wallet_id
            .as_ref()
            .and_then(|id| {
                wallet_simple
                    .iter()
                    .position(|wallet| wallet.id.as_str() == id)
            })
            .unwrap_or(0) as i32;

        let wallet_count = wallet_data.len();
        let adapter = ui.global::<CryptoAdapter>();
        adapter.set_wallets(ModelRc::new(VecModel::from(wallet_data)));
        adapter.set_wallet_list(ModelRc::new(VecModel::from(wallet_simple)));
        adapter.set_default_wallet_index(last_wallet_index);
        let asset_count = adapter.get_portfolio().row_count();
        set_portfolio_summary(&adapter, asset_count, wallet_count);
    }
}

/// Reload portfolio with prices and charts
/// Optionally accepts a notify closure to report errors
pub fn reload_portfolio<N>(ui_weak: &Weak<AppWindow>, controller: &Arc<AppController>, notify: Option<&N>)
where
    N: Fn(String, bool),
{
    let mut assets = match controller.get_aggregated_portfolio() {
        Ok(a) => a,
        Err(e) => {
            if let Some(n) = notify {
                n(format!("Failed to load portfolio: {}", e), true);
            }
            return;
        }
    };

    // Load preferred currency and exchange rate
    let preferred_currency = resolve_preferred_currency(controller);
    let usd_rate = load_preferred_usd_rate(controller, &preferred_currency);

    let prices = controller.load_crypto_prices().unwrap_or_default();
    let price_map: HashMap<String, CryptoAsset> = prices
        .clone()
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect();
    let catalog = controller.get_coin_catalog_or_default();
    let catalog_map: HashMap<String, (String, String)> = catalog
        .into_iter()
        .map(|coin| (coin.id, (coin.name, coin.symbol)))
        .collect();

    for asset in &mut assets {
        if let Some(price_data) = price_map.get(&asset.coin_id) {
            asset.update_with_price(price_data.current_price);
        }
    }

    assets.sort_by(|a, b| {
        b.current_value
            .partial_cmp(&a.current_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut chart_assets: Vec<(String, f64)> = assets
        .iter()
        .filter(|asset| price_map.contains_key(&asset.coin_id) && asset.current_value > 0.0)
        .map(|asset| (asset.symbol.clone(), asset.current_value))
        .collect();
    chart_assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let chart_assets = if chart_assets.len() > 6 {
        let mut trimmed = chart_assets[..6].to_vec();
        let other_sum: f64 = chart_assets[6..].iter().map(|(_, v)| *v).sum();
        if other_sum > 0.0 {
            trimmed.push(("OTHER".to_string(), other_sum));
        }
        trimmed
    } else {
        chart_assets
    };

    let chart_total: f64 = chart_assets.iter().map(|(_, value)| *value).sum();
    let distribution: Vec<CryptoDistributionSlice> = if chart_total > 0.0 {
        chart_assets
            .iter()
            .enumerate()
            .map(|(idx, (label, value))| {
                let percent = (*value / chart_total) * 100.0;
                let value_preferred =
                    convert_usd_to_preferred(*value, &preferred_currency, usd_rate);
                let (r, g, b) = controller.chart_color_for_symbol(label, idx);
                CryptoDistributionSlice {
                    label: SharedString::from(label),
                    value: SharedString::from(format_preferred(value_preferred, &preferred_currency)),
                    percent: SharedString::from(format!("{:.1}%", percent)),
                    color: slint::Color::from_rgb_u8(r, g, b),
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let chart_image = controller.render_portfolio_distribution_chart(&chart_assets);
    let chart_ready = chart_image.is_some();

    let mut total_val_priced = 0.0;
    let mut total_cost_priced = 0.0;
    let mut priced_assets = 0;

    let mapped_assets: Vec<CryptoAssetData> = assets
        .iter()
        .map(|a| {
            let price_data = price_map.get(&a.coin_id);
            if price_data.is_some() {
                total_val_priced += a.current_value;
                total_cost_priced += a.total_cost_basis;
                priced_assets += 1;
            }

            let change_percent = price_data
                .map(|p| p.price_change_percentage_24h)
                .unwrap_or(0.0);

            let change_str = if price_data.is_none() {
                "N/A".to_string()
            } else if change_percent >= 0.0 {
                format!("+ {:.2}%", change_percent)
            } else {
                format!("{:.2}%", change_percent)
            };

            let asset_name = price_data
                .map(|p| p.name.clone())
                .or_else(|| catalog_map.get(&a.coin_id).map(|(name, _)| name.clone()))
                .unwrap_or_else(|| a.symbol.clone());

            // Convert price and value to preferred currency
            let price_preferred =
                convert_usd_to_preferred(a.current_price, &preferred_currency, usd_rate);
            let value_preferred =
                convert_usd_to_preferred(a.current_value, &preferred_currency, usd_rate);

            let price_fmt = if price_data.is_none() {
                "N/A".to_string()
            } else {
                format_compact_price_preferred(price_preferred, &preferred_currency)
            };

            let value_fmt = if price_data.is_none() {
                "N/A".to_string()
            } else {
                format_preferred(value_preferred, &preferred_currency)
            };

            CryptoAssetData {
                id: SharedString::from(&a.coin_id),
                symbol: SharedString::from(&a.symbol),
                icon: crypto_icon_for_symbol(&a.symbol),
                name: SharedString::from(asset_name),
                price: SharedString::from(price_fmt),
                amount: SharedString::from(format!(
                    "{} {}",
                    format_compact_asset_amount(a.total_amount),
                    a.symbol
                )),
                value: SharedString::from(value_fmt),
                change_24h: SharedString::from(change_str),
                is_positive: change_percent >= 0.0,
                allocation: 0.0,
            }
        })
        .collect();

    // Tickers
    let ticker_ids = controller.get_active_ticker_ids();
    let mut tickers: Vec<CryptoAssetData> = Vec::new();

    for id in ticker_ids {
        if let Some(data) = price_map.get(&id) {
            let change_str = if data.price_change_percentage_24h >= 0.0 {
                format!("+ {:.2}%", data.price_change_percentage_24h)
            } else {
                format!("{:.2}%", data.price_change_percentage_24h)
            };
            let price_preferred =
                convert_usd_to_preferred(data.current_price, &preferred_currency, usd_rate);
            let price_fmt = format_compact_price_preferred(price_preferred, &preferred_currency);

            tickers.push(CryptoAssetData {
                id: SharedString::from(&id),
                symbol: SharedString::from(&data.symbol),
                icon: crypto_icon_for_symbol(&data.symbol),
                name: SharedString::from(&data.name),
                price: SharedString::from(price_fmt),
                amount: "".into(),
                value: "".into(),
                change_24h: SharedString::from(change_str),
                is_positive: data.price_change_percentage_24h >= 0.0,
                allocation: 0.0,
            });
        } else {
            let (name, symbol) = catalog_map
                .get(&id)
                .cloned()
                .unwrap_or_else(|| (id.clone(), id.to_uppercase()));
            let icon = crypto_icon_for_symbol(&symbol);

            tickers.push(CryptoAssetData {
                id: SharedString::from(&id),
                symbol: SharedString::from(symbol.as_str()),
                icon,
                name: SharedString::from(name),
                price: "N/A".into(),
                amount: "".into(),
                value: "".into(),
                change_24h: "N/A".into(),
                is_positive: true,
                allocation: 0.0,
            });
        }
    }

    let total_value_label = if priced_assets > 0 {
        let total_preferred =
            convert_usd_to_preferred(total_val_priced, &preferred_currency, usd_rate);
        format_preferred(total_preferred, &preferred_currency)
    } else {
        "N/A".to_string()
    };

    let (total_pnl_label, total_pnl_positive, total_roi_label) = if priced_assets > 0 {
        let total_pnl_val = total_val_priced - total_cost_priced;
        let (pnl_label, pnl_positive) =
            format_signed_preferred(total_pnl_val, &preferred_currency, usd_rate);
        let roi_label = format_roi(total_val_priced, total_cost_priced);
        (pnl_label, pnl_positive, roi_label)
    } else {
        ("N/A".to_string(), true, "N/A".to_string())
    };

    let current_period = chrono::Local::now().format("%Y").to_string();
    let (total_realized_label, total_realized_positive) = controller
        .generate_tax_summary(current_period)
        .ok()
        .map(|summary| {
            format_signed_preferred(
                summary.report.summary.total_gain,
                &preferred_currency,
                usd_rate,
            )
        })
        .unwrap_or_else(|| ("N/A".to_string(), true));

    let mut trend_image = None;
    let mut trend_ready = false;
    if priced_assets > 0 {
        let _ = controller.save_crypto_portfolio_snapshot(total_val_priced, total_cost_priced);
    }

    let snapshots = controller
        .get_crypto_portfolio_snapshots(180)
        .unwrap_or_default();
    if !snapshots.is_empty() {
        let trend_points: Vec<(String, f64, f64)> = snapshots
            .into_iter()
            .filter(|(_, value, cost)| *value > 0.0 || *cost > 0.0)
            .collect();
        trend_image = controller.render_portfolio_trend_chart(&trend_points);
        trend_ready = trend_image.is_some();
    }

    let (fx_label, fx_display) = load_crypto_badge_state(controller);

    let last_updated_label = controller
        .get_app_setting(SETTING_CRYPTO_LAST_UPDATED)
        .ok()
        .filter(|val| !val.is_empty())
        .and_then(|saved| {
            chrono::DateTime::parse_from_rfc3339(&saved)
                .ok()
                .map(|dt| {
                    let local = dt.with_timezone(&chrono::Local);
                    let now = chrono::Local::now();
                    if local.date_naive() == now.date_naive() {
                        let time = local.format("%H:%M").to_string();
                        i18n::t_args("crypto-last-updated-today-at", &[("time", time.as_str())])
                    } else {
                        local.format("%Y-%m-%d %H:%M").to_string()
                    }
                })
                .or(Some(saved))
        })
        .or_else(|| {
            prices
                .iter()
                .filter_map(|price| {
                    chrono::DateTime::parse_from_rfc3339(&price.last_updated).ok()
                })
                .max()
                .map(|dt| {
                    let local = dt.with_timezone(&chrono::Local);
                    let now = chrono::Local::now();
                    if local.date_naive() == now.date_naive() {
                        let time = local.format("%H:%M").to_string();
                        i18n::t_args("crypto-last-updated-today-at", &[("time", time.as_str())])
                    } else {
                        local.format("%Y-%m-%d %H:%M").to_string()
                    }
                })
        });

    if let Some(ui) = ui_weak.upgrade() {
        let adapter = ui.global::<CryptoAdapter>();
        let asset_count = mapped_assets.len();
        adapter.set_portfolio(ModelRc::new(VecModel::from(mapped_assets)));
        adapter.set_market_tickers(ModelRc::new(VecModel::from(tickers)));
        adapter.set_total_value(SharedString::from(total_value_label));
        adapter.set_total_pnl_positive(total_pnl_positive);
        adapter.set_total_pnl(SharedString::from(total_pnl_label));
        adapter.set_total_realized_positive(total_realized_positive);
        adapter.set_total_realized(SharedString::from(total_realized_label));
        adapter.set_total_roi(SharedString::from(total_roi_label));
        adapter.set_fx_rate_label(SharedString::from(fx_label));
        adapter.set_clp_rate(SharedString::from(fx_display));
        adapter.set_portfolio_trend_image(trend_image.unwrap_or_default());
        adapter.set_portfolio_trend_ready(trend_ready);
        adapter.set_portfolio_chart_image(chart_image.unwrap_or_default());
        adapter.set_portfolio_chart_ready(chart_ready);
        adapter.set_portfolio_distribution(ModelRc::new(VecModel::from(distribution)));
        if let Some(label) = last_updated_label {
            adapter.set_last_updated(label.into());
        }
        let wallet_count = adapter.get_wallets().row_count();
        set_portfolio_summary(&adapter, asset_count, wallet_count);
        adapter.set_is_loading(false);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        format_compact_asset_amount, format_compact_price_preferred, format_roi,
        format_signed_preferred,
    };

    #[test]
    fn format_roi_returns_na_when_cost_is_zero() {
        assert_eq!(format_roi(100.0, 0.0), "N/A");
    }

    #[test]
    fn format_roi_formats_positive_and_negative_values() {
        assert_eq!(format_roi(120.0, 100.0), "+ 20.00%");
        assert_eq!(format_roi(80.0, 100.0), "-20.00%");
    }

    #[test]
    fn format_signed_preferred_marks_positive_values() {
        let (label, positive) = format_signed_preferred(125.0, "USD", 1.0);
        assert_eq!(label, "+ USD 125.00");
        assert!(positive);
    }

    #[test]
    fn format_signed_preferred_marks_negative_values() {
        let (label, positive) = format_signed_preferred(-40.0, "USD", 1.0);
        assert_eq!(label, "- USD 40.00");
        assert!(!positive);
    }

    #[test]
    fn format_compact_price_preferred_uses_four_decimals_for_subunit_prices() {
        assert_eq!(
            format_compact_price_preferred(0.123456, "USD"),
            "USD 0.1235"
        );
    }

    #[test]
    fn format_compact_price_preferred_keeps_six_decimals_for_very_small_prices() {
        assert_eq!(
            format_compact_price_preferred(0.0054321, "USD"),
            "USD 0.005432"
        );
    }

    #[test]
    fn format_compact_price_preferred_uses_scientific_for_tiny_prices() {
        let formatted = format_compact_price_preferred(0.00000042, "USD");
        assert!(formatted.starts_with("USD "));
        assert!(formatted.contains("e-"));
    }

    #[test]
    fn format_compact_price_preferred_returns_na_for_invalid_values() {
        assert_eq!(format_compact_price_preferred(0.0, "USD"), "N/A");
        assert_eq!(format_compact_price_preferred(f64::NAN, "USD"), "N/A");
    }

    #[test]
    fn format_compact_asset_amount_uses_scientific_for_tiny_values() {
        assert_eq!(format_compact_asset_amount(0.0000064), "6.40e-6");
    }

    #[test]
    fn format_compact_asset_amount_keeps_regular_decimal_for_small_values() {
        assert_eq!(format_compact_asset_amount(0.00012), "0.00012");
    }

    #[test]
    fn format_compact_asset_amount_truncates_large_decimals() {
        assert_eq!(format_compact_asset_amount(0.12345678), "0.123456");
        assert_eq!(format_compact_asset_amount(12.98765432), "12.9876");
    }
}
