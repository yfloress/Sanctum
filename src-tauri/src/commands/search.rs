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

//! The one command behind the global search box.
//!
//! Each domain is asked for its rows, they are handed to the ranker as one
//! pool, and the ordering comes out of a single comparison. Ranking per domain
//! and stitching the lists together afterwards would mean the top result
//! depended on which domain it happened to come from.

use std::collections::{HashMap, HashSet};

use sanctum::error::AppError;
use sanctum::features::crypto::CryptoService;
use sanctum::features::finance::FinanceService;
use sanctum::services::search::{Candidate, DEFAULT_SEARCH_LIMIT, HitKind, MAX_SEARCH_LIMIT, rank};
use sanctum::ui::currency::format_money;
use sanctum::ui::dto::search::{SearchHitDto, SearchInput};
use sanctum::ui::helpers::format_category_label;
use tauri::State;

/// Search everything the user owns, best match first.
///
/// Failures in one domain are swallowed rather than propagated: a crypto side
/// that will not load is no reason to refuse to find a transaction.
#[tauri::command]
pub fn global_search(
    finance: State<'_, FinanceService>,
    crypto: State<'_, CryptoService>,
    input: SearchInput,
) -> Result<Vec<SearchHitDto>, AppError> {
    let limit = input
        .limit
        .unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT);

    // Bail before touching the database: an empty query matches nothing, and
    // the palette asks on every keystroke.
    if input.query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let accounts = finance.get_accounts().unwrap_or_default();
    let account_lookup: HashMap<String, (String, String)> = accounts
        .iter()
        .map(|a| (a.id.clone(), (a.currency.clone(), a.name.clone())))
        .collect();

    let mut candidates: Vec<Candidate> = Vec::new();
    push_accounts(&mut candidates, &accounts);
    push_categories(&mut candidates, &finance);
    push_transactions(&mut candidates, &finance, &account_lookup);
    push_wallets(&mut candidates, &crypto);
    push_coins(&mut candidates, &crypto);

    Ok(rank(&input.query, candidates, limit)
        .into_iter()
        .map(|hit| SearchHitDto {
            kind: hit.candidate.kind,
            id: hit.candidate.id,
            title: hit.candidate.title,
            subtitle: hit.candidate.subtitle,
            account_id: hit.candidate.account_id,
        })
        .collect())
}

fn push_accounts(out: &mut Vec<Candidate>, accounts: &[sanctum::models::Account]) {
    for account in accounts {
        if account.is_archived {
            continue;
        }
        out.push(Candidate {
            kind: HitKind::Account,
            id: account.id.clone(),
            title: account.name.clone(),
            subtitle: account.currency.clone(),
            keywords: account.account_type.clone(),
            recency: 0,
            amount_cents: None,
            account_id: None,
        });
    }
}

fn push_categories(out: &mut Vec<Candidate>, finance: &FinanceService) {
    for kind in ["expense", "income"] {
        let Ok(categories) = finance.get_transaction_categories(kind.to_string()) else {
            continue;
        };
        for category in categories {
            out.push(Candidate {
                kind: HitKind::Category,
                // The activity filter matches on the name, not on the row id.
                id: category.name.clone(),
                title: format_category_label(&category.name),
                subtitle: kind.to_string(),
                keywords: String::new(),
                recency: 0,
                amount_cents: None,
                account_id: None,
            });
        }
    }
}

fn push_transactions(
    out: &mut Vec<Candidate>,
    finance: &FinanceService,
    account_lookup: &HashMap<String, (String, String)>,
) {
    let Ok(transactions) = finance.get_transactions() else {
        return;
    };
    for tx in transactions {
        let (currency, account_name) = account_lookup
            .get(&tx.account_id)
            .cloned()
            .unwrap_or_else(|| ("USD".to_string(), String::new()));

        let category_label = format_category_label(&tx.category);

        // A row with no description is known by its category; showing an empty
        // title and the category underneath would say the same thing twice.
        let title = if tx.description.trim().is_empty() {
            category_label.clone()
        } else {
            tx.description.clone()
        };

        let subtitle = format!(
            "{} · {} · {}",
            tx.date,
            account_name,
            format_money(tx.amount.abs(), &currency)
        );

        out.push(Candidate {
            kind: HitKind::Transaction,
            id: tx.id.clone(),
            title,
            subtitle,
            // The label as well as the code: a seeded category is stored as
            // `FOOD` but printed as "Comida", and the printed word is the one
            // the user will type.
            keywords: format!("{} {} {}", tx.category, category_label, account_name),
            recency: date_rank(&tx.date),
            amount_cents: Some(tx.amount),
            account_id: Some(tx.account_id.clone()),
        });
    }
}

fn push_wallets(out: &mut Vec<Candidate>, crypto: &CryptoService) {
    let Ok(wallets) = crypto.get_wallets() else {
        return;
    };
    for wallet in wallets {
        out.push(Candidate {
            kind: HitKind::Wallet,
            id: wallet.id.clone(),
            title: wallet.name.clone(),
            subtitle: wallet.category.clone(),
            keywords: String::new(),
            recency: 0,
            amount_cents: None,
            account_id: None,
        });
    }
}

/// Adds the coins the user actually holds, not the whole catalog.
///
/// The catalog runs to thousands of names, and offering every one of them would
/// bury the user's own rows under coins they have never touched. Held rather
/// than merely watched, because a hit opens the asset panel and there is
/// nothing to show there for a coin with no position.
fn push_coins(out: &mut Vec<Candidate>, crypto: &CryptoService) {
    let Ok(portfolio) = crypto.get_aggregated_portfolio() else {
        return;
    };
    if portfolio.is_empty() {
        return;
    }
    let held: HashSet<String> = portfolio.into_iter().map(|asset| asset.coin_id).collect();
    let Ok(catalog) = crypto.get_coin_catalog() else {
        return;
    };
    for coin in catalog {
        if !held.contains(&coin.id) {
            continue;
        }
        out.push(Candidate {
            kind: HitKind::Coin,
            id: coin.id.clone(),
            title: coin.name.clone(),
            subtitle: coin.symbol.to_uppercase(),
            keywords: coin.symbol.clone(),
            recency: 0,
            amount_cents: None,
            account_id: None,
        });
    }
}

/// Turns an ISO `YYYY-MM-DD` date into a sortable number.
///
/// Anything unparseable ranks oldest rather than newest, so a malformed date
/// cannot push itself to the top of the list.
fn date_rank(date: &str) -> i64 {
    date.chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}
