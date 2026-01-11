//! Shared helpers for crypto callbacks

use crate::controller::AppController;
use crate::models::CryptoAsset;
use crate::services::i18n;
use crate::ui::{crypto_icon_for_symbol, format_clp_rate, format_usd, load_wallet_icon};
use crate::{CryptoAdapter, CryptoAssetData, CryptoDistributionSlice, CryptoWalletData, AppWindow, WalletSimple};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::Arc;

/// App setting keys
pub const SETTING_CRYPTO_LAST_WALLET_ID: &str = "crypto_last_wallet_id";
pub const SETTING_CRYPTO_LAST_COIN_ID: &str = "crypto_last_coin_id";
pub const SETTING_CRYPTO_LAST_UPDATED: &str = "crypto_last_updated";

fn set_portfolio_summary(adapter: &CryptoAdapter, assets: usize, wallets: usize) {
    let assets_str = assets.to_string();
    let wallets_str = wallets.to_string();
    let summary = i18n::t_args(
        "crypto-assets-across-wallets",
        &[("assets", assets_str.as_str()), ("wallets", wallets_str.as_str())],
    );
    adapter.set_portfolio_summary(SharedString::from(summary));
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
        let total_bal: f64 = holdings
            .iter()
            .map(|h| {
                let price = price_map.get(&h.coin_id).cloned().unwrap_or(0.0);
                h.total_amount * price
            })
            .sum();

        wallet_data.push(CryptoWalletData {
            id: SharedString::from(w.id),
            name: SharedString::from(w.name),
            category: SharedString::from(w.category.clone()),
            icon: load_wallet_icon(w.icon.clone(), &w.category),
            balance: SharedString::from(format_usd(total_bal)),
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
                let (r, g, b) = controller.chart_color_for_symbol(label, idx);
                CryptoDistributionSlice {
                    label: SharedString::from(label),
                    value: SharedString::from(format_usd(*value)),
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

    let mut total_val = 0.0;
    let mut total_cost = 0.0;
    let mut priced_assets = 0;
    let mut missing_price_assets = 0;

    let mapped_assets: Vec<CryptoAssetData> = assets
        .iter()
        .map(|a| {
            let price_data = price_map.get(&a.coin_id);
            total_cost += a.total_cost_basis;
            if price_data.is_some() {
                total_val += a.current_value;
                priced_assets += 1;
            } else {
                missing_price_assets += 1;
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
                .unwrap_or_else(|| a.symbol.clone());

            let price_fmt = if price_data.is_none() {
                "N/A".to_string()
            } else if a.current_price < 1.0 {
                format!("$ {:.4}", a.current_price)
            } else {
                format_usd(a.current_price)
            };

            let value_fmt = if price_data.is_none() {
                "N/A".to_string()
            } else {
                format_usd(a.current_value)
            };

            CryptoAssetData {
                id: SharedString::from(&a.coin_id),
                symbol: SharedString::from(&a.symbol),
                icon: crypto_icon_for_symbol(&a.symbol),
                name: SharedString::from(asset_name),
                price: SharedString::from(price_fmt),
                amount: SharedString::from(format!("{:.4} {}", a.total_amount, a.symbol)),
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
            let price_fmt = if data.current_price < 1.0 {
                format!("$ {:.4}", data.current_price)
            } else {
                format_usd(data.current_price)
            };

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

    let total_value_label = if priced_assets > 0 && missing_price_assets == 0 {
        format_usd(total_val)
    } else {
        "N/A".to_string()
    };

    let (total_pnl_label, total_pnl_positive) =
        if priced_assets > 0 && missing_price_assets == 0 {
            let total_pnl_val = total_val - total_cost;
            let pnl_sign = if total_pnl_val >= 0.0 { "+" } else { "-" };
            (
                format!("{} {}", pnl_sign, format_usd(total_pnl_val.abs())),
                total_pnl_val >= 0.0,
            )
        } else {
            ("N/A".to_string(), true)
        };

    let mut trend_image = None;
    let mut trend_ready = false;
    if priced_assets > 0 && missing_price_assets == 0 {
        let _ = controller.save_crypto_portfolio_snapshot(total_val, total_cost);
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

    let clp_cached = controller
        .load_exchange_rate_allow_stale("CLP_USD".to_string())
        .ok()
        .flatten();

    let clp_display = clp_cached
        .and_then(|(r, _)| {
            if r > 0.0 {
                Some(format_clp_rate(r))
            } else {
                None
            }
        })
        .unwrap_or_else(|| "N/A".to_string());

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
                        format!("Today at {}", local.format("%H:%M"))
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
                        format!("Today at {}", local.format("%H:%M"))
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
        adapter.set_clp_rate(SharedString::from(clp_display));
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
