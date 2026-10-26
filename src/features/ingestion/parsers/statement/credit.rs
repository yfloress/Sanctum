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

//! National (CLP) credit card statements.
//!
//! Only what was billed this period is imported: purchases, the installment
//! due this month of purchases in installments, and fees. The previous
//! balance and the payments to the card are left out, because the payment
//! already shows up in the checking account it came from and importing both
//! would count the spending twice. Future installments are left out too.

use super::clp::{
    is_digits, parse_amount, parse_full_date, parse_installment, parse_short_year_date,
};
use super::document::{Document, fold};
use super::period::{PeriodRule, find_period};
use super::{
    ParsedStatement, RowIssue, RowProblem, StatementError, StatementFormat, StatementRow,
    StatementWarning,
};

/// Words of a line with a leading `$` split off: "$12.345" → "$", "12.345".
fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for word in text.split_whitespace() {
        match word.strip_prefix('$') {
            Some(rest) if !rest.is_empty() => {
                out.push("$".to_string());
                out.push(rest.to_string());
            }
            _ => out.push(word.to_string()),
        }
    }
    out
}

fn require_markers(
    doc: &Document,
    markers: &[&str],
    format: StatementFormat,
) -> Result<(), StatementError> {
    let text = doc.folded_text();
    if markers.iter().all(|m| text.contains(m)) {
        Ok(())
    } else {
        Err(StatementError::WrongFormat(format))
    }
}

fn billing_period(
    doc: &Document,
) -> Result<(chrono::NaiveDate, chrono::NaiveDate), StatementError> {
    find_period(
        &doc.folded_tokens(),
        &[PeriodRule::AfterHeader(&["PERIODO", "FACTURADO"])],
    )?
    .ok_or(StatementError::MissingPeriod)
}

fn mismatch(statement: &mut ParsedStatement, section: &'static str, declared: i64, parsed: i64) {
    if declared != parsed {
        statement.warnings.push(StatementWarning::SectionMismatch {
            section,
            declared,
            parsed,
        });
    }
}

// ─── Banco de Chile ──────────────────────────────────────────────────────────

pub const BANCO_CHILE_CREDIT_MARKERS: &[&str] = &[
    "ESTADO DE CUENTA NACIONAL DE TARJETA DE CREDITO",
    "PERIODO FACTURADO",
    "TOTAL TRANSACCIONES EN UNA CUOTA",
    "TOTAL TRANSACCIONES EN CUOTAS",
    "III. INFORMACION DE PAGO",
];

/// Subtotal lines of the "II. Detalle" section. Each subtotal is printed
/// after the rows it adds up, so the rows following subtotal B belong to
/// section C, and so on.
const BANCO_CHILE_SECTIONS: &[(char, &str)] = &[
    ('A', "MONTO FACTURADO A PAGAR"),
    ('B', "TOTAL PAGOS A LA CUENTA"),
    ('C', "TOTAL PAT A LA CUENTA"),
    ('D', "TOTAL TRANSACCIONES EN UNA CUOTA"),
    ('E', "TOTAL TRANSACCIONES EN CUOTAS"),
    (
        'F',
        "TOTAL PRODUCTOS O SERVICIOS VOLUNTARIAMENTE CONTRATADOS",
    ),
    ('G', "TOTAL CARGOS, COMISIONES, IMPUESTOS Y ABONOS"),
];
/// D one-installment purchases, E installments, F subscribed services,
/// G fees and taxes. A is the previous balance, B and C payments.
const BANCO_CHILE_BILLED: &[char] = &['D', 'E', 'F', 'G'];

/// `... G $ 12.345` at the end of a subtotal line.
fn section_subtotal(words: &[String], letter: char) -> Option<i64> {
    let [.., label, dollar, amount] = words else {
        return None;
    };
    (label.len() == 1 && label.starts_with(letter) && dollar == "$")
        .then(|| parse_amount(amount).map(|a| a.signed()))
        .flatten()
}

pub fn parse_banco_chile_credit(doc: &Document) -> Result<ParsedStatement, StatementError> {
    let format = StatementFormat::BancoChileCredit;
    require_markers(doc, BANCO_CHILE_CREDIT_MARKERS, format)?;
    let period = billing_period(doc)?;
    let mut statement = ParsedStatement::new(format, Some(period));

    let mut subtotals: Vec<(char, i64)> = Vec::new();
    let mut parsed: Vec<(char, i64)> = BANCO_CHILE_BILLED.iter().map(|&c| (c, 0)).collect();
    let mut billed_total = None;
    let mut section: Option<char> = None;

    for (index, line) in doc.lines.iter().enumerate() {
        let text = line.text();
        let folded = fold(&text);
        if folded.starts_with("III. INFORMACION") {
            break;
        }
        let words = tokens(&folded);
        if folded.starts_with("MONTO TOTAL FACTURADO A PAGAR") && folded.contains("A+B+C+D+E+F+G") {
            billed_total = words
                .last()
                .and_then(|w| parse_amount(w))
                .map(|a| a.signed());
            continue;
        }
        let subtotal = BANCO_CHILE_SECTIONS.iter().find_map(|&(letter, label)| {
            let matches =
                folded.starts_with(label) && (letter != 'A' || folded.contains("ANTERIOR"));
            matches
                .then(|| section_subtotal(&words, letter).map(|v| (letter, v)))
                .flatten()
        });
        if let Some((letter, value)) = subtotal {
            subtotals.push((letter, value));
            let next = BANCO_CHILE_SECTIONS
                .iter()
                .position(|s| s.0 == letter)
                .map(|i| i + 1);
            section = next.and_then(|i| BANCO_CHILE_SECTIONS.get(i)).map(|s| s.0);
            continue;
        }

        let raw = tokens(&text);
        let Some(start) = raw.windows(2).position(|w| {
            parse_short_year_date(&w[0]).is_some()
                && is_digits(&w[1])
                && (10..=14).contains(&w[1].len())
        }) else {
            continue;
        };
        let Some(current) = section.filter(|s| BANCO_CHILE_BILLED.contains(s)) else {
            continue;
        };
        match banco_chile_row(&raw, start, index) {
            Ok(row) => {
                if let Some(entry) = parsed.iter_mut().find(|e| e.0 == current) {
                    entry.1 -= row.amount;
                }
                statement.rows.push(row);
            }
            Err(issue) => statement.problems.push(RowProblem {
                line: index,
                text,
                issue,
            }),
        }
    }

    for (letter, total) in &parsed {
        match subtotals.iter().find(|s| s.0 == *letter) {
            Some(declared) => mismatch(&mut statement, section_name(*letter), declared.1, *total),
            None => statement.warnings.push(StatementWarning::TotalsUnreadable),
        }
    }
    let declared_sum: i64 = subtotals.iter().map(|s| s.1).sum();
    match billed_total {
        Some(total) if subtotals.len() == BANCO_CHILE_SECTIONS.len() => {
            mismatch(&mut statement, "total", total, declared_sum)
        }
        _ => statement.warnings.push(StatementWarning::TotalsUnreadable),
    }
    statement.warnings.dedup();
    Ok(statement)
}

fn section_name(letter: char) -> &'static str {
    match letter {
        'D' => "D",
        'E' => "E",
        'F' => "F",
        'G' => "G",
        _ => "?",
    }
}

/// `dd/mm/yy reference description $ operation $ payable nn/nn $ installment`.
/// Anything before the date (the city, on some rows) is ignored.
fn banco_chile_row(words: &[String], start: usize, line: usize) -> Result<StatementRow, RowIssue> {
    let date = parse_short_year_date(&words[start]).ok_or(RowIssue::InvalidDate)?;
    let n = words.len();
    if n < start + 10 {
        return Err(RowIssue::UnreadableRow);
    }
    let tail = &words[n - 7..];
    let (installment, monthly) = match tail {
        [d1, _operation, d2, _payable, quota, d3, monthly]
            if d1 == "$" && d2 == "$" && d3 == "$" =>
        {
            (
                parse_installment(quota).ok_or(RowIssue::UnreadableRow)?,
                monthly,
            )
        }
        _ => return Err(RowIssue::UnreadableRow),
    };
    let monthly = parse_amount(monthly)
        .ok_or(RowIssue::UnreadableRow)?
        .signed();
    if monthly == 0 {
        return Err(RowIssue::ZeroAmount);
    }
    let (number, count) = installment;
    if number == 0 || number > count {
        return Err(RowIssue::UnreadableRow);
    }
    let description = words[start + 2..n - 7].join(" ");
    if description.is_empty() {
        return Err(RowIssue::MissingDescription);
    }
    Ok(StatementRow {
        line,
        date,
        raw_description: description.clone(),
        description,
        amount: -monthly,
        sign_from_keyword: false,
        installment: (count > 1).then_some(installment),
    })
}

// ─── Santander ───────────────────────────────────────────────────────────────

pub const SANTANDER_CREDIT_MARKERS: &[&str] = &[
    "ESTADO DE CUENTA EN MONEDA NACIONAL DE TARJETA DE CREDITO",
    "PERIODO FACTURADO",
    "MOVIMIENTOS TARJETA",
    "3. CARGOS, COMISIONES, IMPUESTOS Y ABONOS",
    "III. INFORMACION DE PAGO",
];

const OPERATIONS_TOTAL: &str = "1. TOTAL OPERACIONES";
const CHARGES_TOTAL: &str = "3. CARGOS, COMISIONES, IMPUESTOS Y ABONOS";
const BILLED_TOTAL: &str = "MONTO TOTAL FACTURADO A PAGAR";

#[derive(Clone, Copy, PartialEq, Eq)]
enum SantanderSection {
    Operations,
    Charges,
}

/// A line that is exactly `label $ amount`.
fn labeled_amount(folded: &str, label: &str) -> Option<i64> {
    let rest = folded.strip_prefix(label)?;
    match tokens(rest).as_slice() {
        [dollar, amount] if dollar == "$" => parse_amount(amount).map(|a| a.signed()),
        _ => None,
    }
}

pub fn parse_santander_credit(doc: &Document) -> Result<ParsedStatement, StatementError> {
    let format = StatementFormat::SantanderCredit;
    require_markers(doc, SANTANDER_CREDIT_MARKERS, format)?;
    let period = billing_period(doc)?;
    let mut statement = ParsedStatement::new(format, Some(period));

    let mut declared = [Vec::new(), Vec::new(), Vec::new()];
    let (mut operations, mut charges) = (0i64, 0i64);
    let mut section = None;
    let mut seen_movements = false;

    for (index, line) in doc.lines.iter().enumerate() {
        let text = line.text();
        let folded = fold(&text);
        for (slot, label) in [OPERATIONS_TOTAL, CHARGES_TOTAL, BILLED_TOTAL]
            .iter()
            .enumerate()
        {
            if let Some(amount) = labeled_amount(&folded, label) {
                declared[slot].push(amount);
            }
        }

        if folded.starts_with("MOVIMIENTOS TARJETA") {
            seen_movements = true;
            section = Some(SantanderSection::Operations);
            continue;
        }
        if seen_movements && folded.starts_with("2. PERIODO ACTUAL") {
            section = Some(SantanderSection::Operations);
            continue;
        }
        if folded.starts_with("EMISOR CLIENTE")
            || folded.starts_with("2. PRODUCTOS O SERVICIOS")
            || folded.starts_with("4. INFORMACION COMPRAS EN CUOTAS")
        {
            section = None;
            continue;
        }
        if folded.starts_with(CHARGES_TOTAL) {
            section = Some(SantanderSection::Charges);
            continue;
        }
        let Some(current) = section else {
            continue;
        };
        match santander_row(&tokens(&text), index) {
            Ok(Some(row)) => {
                match current {
                    SantanderSection::Operations => operations -= row.amount,
                    SantanderSection::Charges => charges -= row.amount,
                }
                statement.rows.push(row);
            }
            Ok(None) => {}
            Err(issue) => statement.problems.push(RowProblem {
                line: index,
                text,
                issue,
            }),
        }
    }

    match declared.map(|d| (d.len() == 1).then(|| d[0])) {
        [Some(ops), Some(fees), Some(billed)] => {
            mismatch(&mut statement, "operations", ops, operations);
            mismatch(&mut statement, "charges", fees, charges);
            mismatch(&mut statement, "total", billed, ops + fees);
        }
        _ => statement.warnings.push(StatementWarning::TotalsUnreadable),
    }
    Ok(statement)
}

/// `[city] dd/mm/yyyy description [nn/nn] $ amount ...`; the billed amount
/// is the last one. Returns `None` for lines that are not movements and for
/// payments to the card.
fn santander_row(words: &[String], line: usize) -> Result<Option<StatementRow>, RowIssue> {
    let Some((date_at, date)) = words
        .iter()
        .enumerate()
        .find_map(|(i, w)| parse_full_date(w).map(|d| (i, d)))
    else {
        return Ok(None);
    };
    let money: Vec<(usize, i64)> = words
        .windows(2)
        .enumerate()
        .filter(|(_, w)| w[0] == "$")
        .filter_map(|(i, w)| parse_amount(&w[1]).map(|a| (i, a.signed())))
        .collect();
    let (Some(&(first_money, _)), Some(&(last_money, amount))) = (money.first(), money.last())
    else {
        return Ok(None);
    };
    if first_money <= date_at {
        return Ok(None);
    }
    let mut details: Vec<String> = words[date_at + 1..first_money].to_vec();
    let folded: Vec<String> = details.iter().map(|w| fold(w)).collect();
    if details.is_empty() {
        return Err(RowIssue::MissingDescription);
    }
    if amount == 0 {
        return Err(RowIssue::ZeroAmount);
    }
    let joined = folded.join(" ");
    if amount < 0
        && (joined.contains("MONTO CANCELADO")
            || joined.contains("PAGO TARJETA")
            || joined.contains("PAGO DE TARJETA"))
    {
        return Ok(None);
    }

    let installment = words[date_at + 1..last_money]
        .iter()
        .find_map(|w| parse_installment(w))
        .filter(|(n, c)| *n >= 1 && n <= c);
    if installment.is_some() {
        // "COMPRA EN CUOTAS TRES CUOTAS PREC 0,00 %" → "COMPRA EN CUOTAS".
        let cut = (0..folded.len()).find(|&i| {
            let next = folded.get(i + 1).map(String::as_str);
            (folded[i] == "TRES" && next == Some("CUOTAS"))
                || (matches!(folded[i].as_str(), "CUOTA" | "CUOTAS")
                    && matches!(next, Some("FIJA" | "FIJAS")))
                || (folded[i].len() == 2 && is_digits(&folded[i]) && next == Some("CUOTAS"))
        });
        if let Some(cut) = cut.filter(|&c| c > 0) {
            details.truncate(cut);
        }
    }
    let raw_description = words[date_at + 1..first_money].join(" ");
    Ok(Some(StatementRow {
        line,
        date,
        description: details.join(" "),
        raw_description,
        amount: -amount,
        sign_from_keyword: false,
        installment: installment.filter(|(_, c)| *c > 1),
    }))
}
