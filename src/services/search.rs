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

//! Ranking behind the global search box.
//!
//! Everything here is pure: callers collect rows from their own domain, hand
//! them over as [`Candidate`]s, and get back an ordered list. Keeping the
//! matching out of the domains is what lets one query span the ledger, the
//! accounts and the crypto side and still produce a single ordering.

use serde::Serialize;

/// Longest query that will be considered. Past this it is not a search.
pub const MAX_QUERY_LENGTH: usize = 128;
/// Hits returned when the caller does not ask for a specific number.
pub const DEFAULT_SEARCH_LIMIT: usize = 20;
/// Ceiling on the hits any one search may return.
pub const MAX_SEARCH_LIMIT: usize = 100;

/// What a hit points at, so the frontend knows where to send the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HitKind {
    Account,
    Category,
    Coin,
    Transaction,
    Wallet,
}

impl HitKind {
    /// Nudge applied to every hit of this kind.
    ///
    /// Transactions outnumber everything else by orders of magnitude, so on an
    /// equal textual match the handful of accounts and wallets a user actually
    /// named are the likelier target and go first.
    fn bias(self) -> i64 {
        match self {
            HitKind::Account | HitKind::Wallet => 12,
            HitKind::Category | HitKind::Coin => 8,
            HitKind::Transaction => 0,
        }
    }
}

/// A row offered to the ranker.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub kind: HitKind,
    pub id: String,
    /// What the row reads as. Weighted heaviest.
    pub title: String,
    /// Secondary context shown under the title.
    pub subtitle: String,
    /// Text worth matching but not worth showing — an account name behind a
    /// transaction, a coin's symbol behind its name.
    pub keywords: String,
    /// Breaks ties; higher wins. Dates work as `YYYYMMDD`.
    pub recency: i64,
    /// Set when the row has a money value, so a numeric query can find it.
    pub amount_cents: Option<i64>,
    /// The account a hit belongs to, when it belongs to one. Carried through
    /// untouched: the ranker never reads it, the caller needs it to navigate.
    pub account_id: Option<String>,
}

/// A candidate that matched, with the score it earned.
#[derive(Debug, Clone)]
pub struct Hit {
    pub candidate: Candidate,
    pub score: i64,
}

/// Folds a string to the form both sides of a comparison are held in.
///
/// Accents are stripped rather than respected: someone hunting for "Nuñez"
/// types it without the tilde as often as with it, and a search that answers
/// "no results" to a spelling difference is worse than useless.
pub fn normalize(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(fold_accent)
        .collect()
}

fn fold_accent(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        other => other,
    }
}

/// Strips thousands separators so "1.500" and "1500" are the same query.
///
/// Returns `None` unless what is left is entirely digits, so a term like
/// "2024-05" is treated as text rather than as a botched number.
fn digits_only(term: &str) -> Option<String> {
    let stripped: String = term
        .chars()
        .filter(|c| !matches!(c, '.' | ',' | ' ' | '_' | '$'))
        .collect();
    if stripped.is_empty() || !stripped.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(stripped)
}

/// Scores one already-normalized term against one already-normalized field.
fn score_field(haystack: &str, term: &str) -> i64 {
    if haystack.is_empty() {
        return 0;
    }
    if haystack == term {
        return 100;
    }
    if haystack.starts_with(term) {
        return 75;
    }
    // A match that starts a word beats one buried mid-word: "mer" should rank
    // "Mercado" over "Supermercado" without excluding either.
    if haystack
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| word.starts_with(term))
    {
        return 55;
    }
    if haystack.contains(term) {
        return 30;
    }
    0
}

/// Scores a numeric term against a candidate's amount, if it has one.
fn score_amount(amount_cents: Option<i64>, term: &str) -> i64 {
    let Some(cents) = amount_cents else {
        return 0;
    };
    let Some(digits) = digits_only(term) else {
        return 0;
    };
    // Compared in whole units: people search "1500", not "150000" cents.
    let units = (cents.abs() / 100).to_string();
    if units == digits {
        return 90;
    }
    if units.starts_with(&digits) {
        return 50;
    }
    0
}

/// Best score any of a candidate's fields gives this term.
fn score_term(candidate: &Candidate, fields: &Fields, term: &str) -> i64 {
    // Weighted so a hit on what the row is called outranks one on its context.
    let title = score_field(&fields.title, term);
    let keywords = score_field(&fields.keywords, term) * 6 / 10;
    let subtitle = score_field(&fields.subtitle, term) * 45 / 100;
    let amount = score_amount(candidate.amount_cents, term);
    title.max(keywords).max(subtitle).max(amount)
}

/// A candidate's text, folded once so a multi-term query does not refold it.
struct Fields {
    title: String,
    subtitle: String,
    keywords: String,
}

/// Ranks `candidates` against `query`, best first.
///
/// Terms are combined with AND: every word the user typed has to land
/// somewhere, so "super julio" finds the supermarket run in July rather than
/// everything that mentions either. An empty or over-long query matches
/// nothing, which is what leaves the palette showing only its commands.
pub fn rank(query: &str, candidates: Vec<Candidate>, limit: usize) -> Vec<Hit> {
    let normalized = normalize(query);
    if normalized.is_empty() || normalized.len() > MAX_QUERY_LENGTH {
        return Vec::new();
    }
    let terms: Vec<&str> = normalized.split_whitespace().collect();
    if terms.is_empty() {
        return Vec::new();
    }

    let mut hits: Vec<Hit> = candidates
        .into_iter()
        .filter_map(|candidate| {
            let fields = Fields {
                title: normalize(&candidate.title),
                subtitle: normalize(&candidate.subtitle),
                keywords: normalize(&candidate.keywords),
            };
            let mut total = 0;
            for term in &terms {
                let term_score = score_term(&candidate, &fields, term);
                if term_score == 0 {
                    return None;
                }
                total += term_score;
            }
            // Averaged over the terms so a two-word query is not automatically
            // worth more than a one-word query on a different row.
            let score = total / terms.len() as i64 + candidate.kind.bias();
            Some(Hit { candidate, score })
        })
        .collect();

    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.candidate.recency.cmp(&a.candidate.recency))
            .then_with(|| a.candidate.title.cmp(&b.candidate.title))
    });
    hits.truncate(limit.clamp(1, MAX_SEARCH_LIMIT));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(kind: HitKind, title: &str) -> Candidate {
        Candidate {
            kind,
            id: title.to_string(),
            title: title.to_string(),
            subtitle: String::new(),
            keywords: String::new(),
            recency: 0,
            amount_cents: None,
            account_id: None,
        }
    }

    #[test]
    fn normalize_folds_case_and_accents() {
        assert_eq!(normalize("  Almacén Ñuñoa "), "almacen nunoa");
    }

    #[test]
    fn empty_query_matches_nothing() {
        let hits = rank("   ", vec![candidate(HitKind::Account, "Checking")], 10);
        assert!(hits.is_empty());
    }

    #[test]
    fn over_long_query_matches_nothing() {
        let query = "a".repeat(MAX_QUERY_LENGTH + 1);
        let hits = rank(&query, vec![candidate(HitKind::Account, &query)], 10);
        assert!(hits.is_empty());
    }

    #[test]
    fn accents_are_ignored_in_both_directions() {
        let hits = rank("nunoa", vec![candidate(HitKind::Account, "Ñuñoa")], 10);
        assert_eq!(hits.len(), 1);
        let hits = rank("ñuñoa", vec![candidate(HitKind::Account, "Nunoa")], 10);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn exact_match_outranks_prefix_which_outranks_substring() {
        let hits = rank(
            "mer",
            vec![
                candidate(HitKind::Account, "Supermercado"),
                candidate(HitKind::Account, "Mercado"),
                candidate(HitKind::Account, "Mer"),
            ],
            10,
        );
        let order: Vec<&str> = hits.iter().map(|h| h.candidate.title.as_str()).collect();
        assert_eq!(order, vec!["Mer", "Mercado", "Supermercado"]);
    }

    #[test]
    fn word_start_beats_mid_word() {
        let hits = rank(
            "cado",
            vec![
                candidate(HitKind::Account, "Mercado Libre"),
                candidate(HitKind::Account, "Cado Store"),
            ],
            10,
        );
        assert_eq!(hits[0].candidate.title, "Cado Store");
    }

    #[test]
    fn every_term_must_match() {
        let mut tx = candidate(HitKind::Transaction, "Supermercado");
        tx.subtitle = "July groceries".to_string();
        let hits = rank("supermercado july", vec![tx.clone()], 10);
        assert_eq!(hits.len(), 1);
        let hits = rank("supermercado august", vec![tx], 10);
        assert!(hits.is_empty());
    }

    #[test]
    fn a_title_match_outranks_a_subtitle_match() {
        let mut on_title = candidate(HitKind::Transaction, "Netflix");
        on_title.recency = 1;
        let mut on_subtitle = candidate(HitKind::Transaction, "Card payment");
        on_subtitle.subtitle = "Netflix".to_string();
        on_subtitle.recency = 2;
        let hits = rank("netflix", vec![on_subtitle, on_title], 10);
        assert_eq!(hits[0].candidate.title, "Netflix");
    }

    #[test]
    fn amounts_are_searchable_with_or_without_separators() {
        let mut tx = candidate(HitKind::Transaction, "Rent");
        // 450,000 in whole units.
        tx.amount_cents = Some(45_000_000);
        assert_eq!(rank("450000", vec![tx.clone()], 10).len(), 1);
        assert_eq!(rank("450.000", vec![tx.clone()], 10).len(), 1);
        assert_eq!(rank("999", vec![tx], 10).len(), 0);
    }

    #[test]
    fn a_negative_amount_is_found_by_its_magnitude() {
        let mut tx = candidate(HitKind::Transaction, "Refund");
        // -12,500 in whole units.
        tx.amount_cents = Some(-1_250_000);
        assert_eq!(rank("12500", vec![tx], 10).len(), 1);
    }

    #[test]
    fn a_dated_term_is_not_read_as_a_number() {
        let mut tx = candidate(HitKind::Transaction, "Rent");
        // 2,024 in whole units — the same digits as the year in the subtitle.
        tx.amount_cents = Some(202_400);
        tx.subtitle = "2024-05-01".to_string();
        let hits = rank("2024-05", vec![tx], 10);
        assert_eq!(hits.len(), 1, "should match the date text, not the amount");
    }

    #[test]
    fn accounts_outrank_transactions_on_an_equal_match() {
        let account = candidate(HitKind::Account, "Savings");
        let tx = candidate(HitKind::Transaction, "Savings");
        let hits = rank("savings", vec![tx, account], 10);
        assert_eq!(hits[0].candidate.kind, HitKind::Account);
    }

    #[test]
    fn ties_fall_back_to_recency() {
        let mut older = candidate(HitKind::Transaction, "Coffee");
        older.id = "older".to_string();
        older.recency = 20_240_101;
        let mut newer = candidate(HitKind::Transaction, "Coffee");
        newer.id = "newer".to_string();
        newer.recency = 20_250_101;
        let hits = rank("coffee", vec![older, newer], 10);
        assert_eq!(hits[0].candidate.id, "newer");
    }

    #[test]
    fn the_limit_is_honoured_and_clamped() {
        let many: Vec<Candidate> = (0..200)
            .map(|i| candidate(HitKind::Transaction, &format!("Coffee {i}")))
            .collect();
        assert_eq!(rank("coffee", many.clone(), 5).len(), 5);
        assert_eq!(rank("coffee", many.clone(), 0).len(), 1);
        assert_eq!(rank("coffee", many, usize::MAX).len(), MAX_SEARCH_LIMIT);
    }

    #[test]
    fn keywords_match_without_being_shown() {
        let mut coin = candidate(HitKind::Coin, "Bitcoin");
        coin.keywords = "BTC".to_string();
        let hits = rank("btc", vec![coin], 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].candidate.title, "Bitcoin");
    }
}
