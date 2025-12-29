//! Portfolio and price-related crypto callbacks

use super::helpers::{reload_portfolio, SETTING_CRYPTO_LAST_UPDATED};
use crate::controller::AppController;
use crate::models::CryptoTransaction;
use crate::ui::{
    crypto_icon_for_symbol, format_clp_rate, format_crypto_tx_display, format_fee_display,
    format_money, format_usd,
};
use crate::{AssetTransaction, AssetWalletBreakdown, CryptoAdapter, CryptoAssetData, AppWindow};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::sync::Arc;

/// Sets up portfolio-related callbacks
pub fn setup_portfolio_callbacks<N>(
    ui: &AppWindow,
    ui_weak: &Weak<AppWindow>,
    controller: &Arc<AppController>,
    notify: N,
) where
    N: Fn(String, bool) + Clone + Send + 'static,
{
    // on_fetch_portfolio
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        ui.global::<CryptoAdapter>().on_fetch_portfolio(move || {
            reload_portfolio(&ui_weak, &controller);
        });
    }

    // on_refresh_prices
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();
        let notify = notify.clone();

        ui.global::<CryptoAdapter>().on_refresh_prices(move || {
            let controller_async = controller.clone();
            let ui_weak_async = ui_weak.clone();
            if let Some(ui) = ui_weak.upgrade() {
                ui.global::<CryptoAdapter>().set_is_refreshing(true);
            }
            let notify_start = notify.clone();
            notify_start("Fetching prices...".into(), false);

            let notify_for_async = notify.clone();

            tokio::spawn(async move {
                let coins = controller_async
                    .get_monitored_coin_ids()
                    .unwrap_or_default();

                let limit_reached = coins.len() > 50;
                let limit_excluded = if limit_reached {
                    let extra_count = coins.len().saturating_sub(50);
                    let preview: Vec<String> = coins.iter().skip(50).take(3).cloned().collect();
                    if preview.is_empty() {
                        String::new()
                    } else if extra_count > preview.len() {
                        format!(
                            "{} +{} more",
                            preview.join(", "),
                            extra_count - preview.len()
                        )
                    } else {
                        preview.join(", ")
                    }
                } else {
                    String::new()
                };
                let has_coins = !coins.is_empty();

                let mut prices_updated = false;
                if !coins.is_empty() {
                    match controller_async.get_crypto_prices(coins).await {
                        Ok(prices) => {
                            let _ = controller_async.save_crypto_prices(prices);
                            prices_updated = true;
                        }
                        Err(e) => {
                            let notify_fail = notify_for_async.clone();
                            let _ = ui_weak_async.upgrade_in_event_loop(move |_| {
                                notify_fail(format!("Price update failed: {}", e), true);
                            });
                        }
                    }
                }

                let (clp_display, clp_updated) = match controller_async.get_clp_usd_rate().await {
                    Ok(rate) => {
                        let _ = controller_async.save_exchange_rate("CLP_USD".to_string(), rate);
                        (format_clp_rate(rate), true)
                    }
                    Err(_) => {
                        if let Ok(Some((rate, _))) =
                            controller_async.load_exchange_rate_allow_stale("CLP_USD".to_string())
                        {
                            (format_clp_rate(rate), true)
                        } else {
                            ("N/A".to_string(), false)
                        }
                    }
                };

                let notify_success = notify_for_async.clone();
                let now = chrono::Local::now();
                let timestamp_to_save = if prices_updated {
                    Some(now.to_rfc3339())
                } else {
                    None
                };
                let last_updated_label = if prices_updated {
                    Some(format!("Today at {}", now.format("%H:%M")))
                } else {
                    None
                };

                if let Some(ts) = timestamp_to_save.as_ref() {
                    let _ = controller_async.set_app_setting(SETTING_CRYPTO_LAST_UPDATED, ts);
                }

                let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                    ui.global::<CryptoAdapter>().set_is_refreshing(false);
                    ui.global::<CryptoAdapter>().invoke_fetch_portfolio();
                    ui.global::<CryptoAdapter>()
                        .set_clp_rate(SharedString::from(clp_display));
                    if let Some(label) = last_updated_label {
                        ui.global::<CryptoAdapter>().set_last_updated(label.into());
                    }
                    ui.global::<CryptoAdapter>()
                        .set_limit_reached(limit_reached);
                    ui.global::<CryptoAdapter>()
                        .set_limit_excluded(limit_excluded.into());
                    if prices_updated {
                        notify_success("Prices updated".into(), false);
                    } else if !has_coins && clp_updated {
                        notify_success("Rates updated".into(), false);
                    }
                });
            });
        });
    }

    // on_get_last_price
    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>()
            .on_get_last_price(move |coin_id| {
                let prices = controller.load_crypto_prices().unwrap_or_default();
                if let Some(data) = prices.iter().find(|p| p.id == coin_id.as_str()) {
                    format!("{:.4}", data.current_price).into()
                } else {
                    "".into()
                }
            });
    }

    // on_get_swap_quote
    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>().on_get_swap_quote(
            move |from_coin_id, to_coin_id, amount_str| {
                let amount_clean = amount_str
                    .replace(",", "")
                    .replace("$", "")
                    .trim()
                    .to_string();
                let amount: f64 = match amount_clean.parse() {
                    Ok(value) if value > 0.0 => value,
                    _ => return SharedString::from(""),
                };

                let prices = controller.load_crypto_prices().unwrap_or_default();
                let from_price = prices
                    .iter()
                    .find(|p| p.id == from_coin_id.as_str())
                    .map(|p| p.current_price)
                    .unwrap_or(0.0);
                let to_price = prices
                    .iter()
                    .find(|p| p.id == to_coin_id.as_str())
                    .map(|p| p.current_price)
                    .unwrap_or(0.0);

                if from_price <= 0.0 || to_price <= 0.0 {
                    return SharedString::from("");
                }

                let to_amount = amount * (from_price / to_price);
                let mut formatted = format!("{:.8}", to_amount);
                while formatted.contains('.') && formatted.ends_with('0') {
                    formatted.pop();
                }
                if formatted.ends_with('.') {
                    formatted.pop();
                }

                SharedString::from(formatted)
            },
        );
    }

    // on_get_available_balance
    {
        let controller = controller.clone();
        ui.global::<CryptoAdapter>()
            .on_get_available_balance(move |wallet_id, coin_id, date| {
                match controller.get_available_balance(
                    wallet_id.to_string(),
                    coin_id.to_string(),
                    date.to_string(),
                ) {
                    Ok(balance) => {
                        let mut formatted = format!("{:.8}", balance);
                        while formatted.contains('.') && formatted.ends_with('0') {
                            formatted.pop();
                        }
                        if formatted.ends_with('.') {
                            formatted.pop();
                        }
                        SharedString::from(formatted)
                    }
                    Err(e) => {
                        log::error!("Error getting available balance: {:?}", e);
                        SharedString::from("0")
                    }
                }
            });
    }

    // on_fetch_asset_details
    {
        let controller = controller.clone();
        let ui_weak = ui_weak.clone();

        ui.global::<CryptoAdapter>()
            .on_fetch_asset_details(move |coin_id| {
                let coin_id_str = coin_id.to_string();

                if let Ok(assets) = controller.get_aggregated_portfolio()
                    && let Some(asset) = assets.iter().find(|a| a.coin_id == coin_id_str)
                {
                    let prices = controller.load_crypto_prices().unwrap_or_default();
                    let price_data = prices.iter().find(|p| p.id == coin_id_str);
                    let current_price = price_data.map(|p| p.current_price).unwrap_or(0.0);
                    let price_change = price_data
                        .map(|p| p.price_change_percentage_24h)
                        .unwrap_or(0.0);
                    let asset_name = price_data
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| asset.symbol.clone());

                    let mut updated_asset = asset.clone();
                    if current_price > 0.0 {
                        updated_asset.update_with_price(current_price);
                    }

                    let missing_price = price_data.is_none();
                    let change_str = if missing_price {
                        "N/A".to_string()
                    } else if price_change >= 0.0 {
                        format!("+ {:.2}%", price_change)
                    } else {
                        format!("{:.2}%", price_change)
                    };

                    let price_fmt = if missing_price {
                        "N/A".to_string()
                    } else if updated_asset.current_price < 1.0 {
                        format!("$ {:.4}", updated_asset.current_price)
                    } else {
                        format_usd(updated_asset.current_price)
                    };

                    let value_fmt = if missing_price {
                        "N/A".to_string()
                    } else {
                        format_usd(updated_asset.current_value)
                    };

                    let selected = CryptoAssetData {
                        id: SharedString::from(&updated_asset.coin_id),
                        symbol: SharedString::from(&updated_asset.symbol),
                        icon: crypto_icon_for_symbol(&updated_asset.symbol),
                        name: SharedString::from(asset_name),
                        price: SharedString::from(price_fmt),
                        amount: SharedString::from(format!(
                            "{:.4} {}",
                            updated_asset.total_amount, updated_asset.symbol
                        )),
                        value: SharedString::from(value_fmt),
                        change_24h: SharedString::from(change_str),
                        is_positive: price_change >= 0.0,
                        allocation: 0.0,
                    };

                    let wallets = controller.get_wallets().unwrap_or_default();
                    let mut wallet_breakdown: Vec<AssetWalletBreakdown> = Vec::new();

                    for w in wallets {
                        let holdings = controller
                            .get_wallet_holdings(w.id.clone())
                            .unwrap_or_default();
                        if let Some(h) = holdings.iter().find(|h| h.coin_id == coin_id_str)
                            && h.total_amount > 0.0
                        {
                            let val = h.total_amount * current_price;
                            wallet_breakdown.push(AssetWalletBreakdown {
                                wallet_name: SharedString::from(w.name),
                                amount: SharedString::from(format!("{:.4}", h.total_amount)),
                                value: SharedString::from(format_money(
                                    (val * 100.0) as i64,
                                    "USD",
                                )),
                            });
                        }
                    }

                    let history = controller
                        .get_crypto_transactions_by_coin(coin_id_str)
                        .unwrap_or_default();
                    let symbol_map: HashMap<String, String> = controller
                        .get_coin_catalog_or_default()
                        .into_iter()
                        .map(|coin| (coin.id, coin.symbol))
                        .collect();
                    let history_map: HashMap<String, CryptoTransaction> = history
                        .iter()
                        .cloned()
                        .map(|tx| (tx.id.clone(), tx))
                        .collect();
                    let history_mapped: Vec<AssetTransaction> = history
                        .iter()
                        .map(|tx| {
                            let related =
                                tx.related_tx_id.as_ref().and_then(|id| history_map.get(id));
                            let (label, amount_display, price_display, is_swap) =
                                format_crypto_tx_display(tx, related);
                            let fee_fmt = format_fee_display(tx, &symbol_map);
                            let notes = tx.notes.clone().unwrap_or_default();

                            AssetTransaction {
                                id: SharedString::from(&tx.id),
                                date: SharedString::from(&tx.date),
                                r#type: SharedString::from(label),
                                amount: SharedString::from(amount_display),
                                price: SharedString::from(price_display),
                                fee: SharedString::from(fee_fmt),
                                notes: SharedString::from(notes),
                                is_swap,
                            }
                        })
                        .collect();

                    if let Some(ui) = ui_weak.upgrade() {
                        let adapter = ui.global::<CryptoAdapter>();
                        adapter.set_selected_asset(selected);
                        adapter.set_asset_wallets(ModelRc::new(VecModel::from(wallet_breakdown)));
                        adapter.set_asset_history(ModelRc::new(VecModel::from(history_mapped)));
                    }
                }
            });
    }
}
