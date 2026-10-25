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

//! Statement parser tests. Every name, account number and amount in the
//! fixtures is made up.

use chrono::NaiveDate;

use super::clp::parse_amount;
use super::period::{PeriodRule, find_period};
use super::*;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn amounts(statement: &ParsedStatement) -> Vec<i64> {
    statement.rows.iter().map(|r| r.amount).collect()
}

fn positioned(page: usize, top: f32, words: &[(&str, f32, f32)]) -> Line {
    Line {
        page,
        top,
        words: words
            .iter()
            .map(|(t, x0, x1)| Word::at(t, *x0, *x1))
            .collect(),
    }
}

/// Swaps plain lines for positioned ones with the same page and text.
fn with_layout(mut doc: Document, lines: Vec<Line>) -> Document {
    for line in lines {
        let slot = doc
            .lines
            .iter_mut()
            .find(|l| l.page == line.page && !l.is_positioned() && l.text() == line.text())
            .unwrap_or_else(|| panic!("no plain line {:?}", line.text()));
        *slot = line;
    }
    doc
}

// ─── Fixtures ────────────────────────────────────────────────────────────────

const BANCO_CHILE: &str = "\
BANCO DE CHILE
CUENTA CORRIENTE
TELEFONO : 0 DESDE : 30/12/2025 HASTA : 01/04/2026
FECHA DETALLE DE TRANSACCION SUCURSAL N° DOCTO MONTO CHEQUES MONTO DEPOSITOS SALDO
DIA/MES O CARGOS O ABONOS
30/12 SALDO INICIAL 50.156 D
07/01 TRASPASO A:Persona Ejemplo INTERNET 43.383 D
07/01 TRASPASO DE:DLOCAL CHILE SPA INTERNET 43.383 50.156 D
26/01 PAGO:GOOGLE PLAY YOUTU OFICINA BAN VI 20.200 29.800 D
02/03 PAGO:DE HONORARIOS 0111111111 CENTRAL 1.449.225 1.449.225 D
01/04 SALDO FINAL 1.438.355 D";

const SANTANDER_SIMPLE: &str = "\
BANCO SANTANDER CHILE
CARTOLA CUENTA CORRIENTE
PERIODO DESDE: 01/05/2026 HASTA: 31/05/2026
FECHA DESCRIPCION CARGO ABONO SALDO
01/05 COMPRA SUPERMERCADO CARGO 12.345 SALDO 87.655
02/05 TRANSFERENCIA RECIBIDA ABONO 50.000 SALDO 137.655";

const SANTANDER_PAGE_1: &str = "\
CUENTA VISTA
ESTADO CUENTA VISTA
CARTOLA DESDE HASTA PAGINA
0-000-00-12345-6 75 27/02/2026 31/03/2026 1 DE 2
Saldo Inicial Cheques o Cargos Depositos o Abonos Saldo Final
32.010 1.219.871 1.215.397 27.536
MOVIMIENTO DE SU CUENTA SALDO DIARIO
FECHA NUMERO SUC DESCRIPCION CHEQUES Y DEPOSITOS Y SALDO
CARGOS ABONOS
02/03 93 Compra PAGO ONLINE KUSHK 20.000
--- Saldo Dia --- 12.010
04/03 93 Compra PAYU *UBER TRIP 4.516
--- Saldo Dia --- 7.494
06/03 9260651 401 0111111111 Transf. 20.000
93 0222222222 Transf a PERSONA UNO 7.494
--- Saldo Dia --- 20.000
09/03 9001812 401 0333333333 Transf. COMERCIO UNO 20.000
93 Compra RIPLEY FLORIDA 20.240
--- Saldo Dia --- 19.760
12/03 93 Compra PAYU *UBER TRIP 1.615
93 0222222222 Transf a PERSONA UNO 18.145
--- Saldo Dia --- 0
16/03 2157027 401 0444444444 Transf. PERSONA DOS 3.800
93 Compra PAYU *UBER TRIP 3.764
--- Saldo Dia --- 36
17/03 1123916 401 0222222222 Transf. PERSONA TRES 1.087.864
93 0222222222 Transf a PERSONA UNO 1.087.900
--- Saldo Dia --- 0
19/03 9260776 401 0111111111 Transf. 10.000
9260776 401 0111111111 Transf. 10.000
--- Saldo Dia --- 20.000
20/03 9260788 401 0111111111 Transf. 20.000
--- Saldo Dia --- 40.000
23/03 9260813 401 0111111111 Transf. 20.000
MENSAJES";

const SANTANDER_PAGE_2: &str = "\
CUENTA VISTA
ESTADO CUENTA VISTA
CARTOLA DESDE HASTA PAGINA
0-000-00-12345-6 75 27/02/2026 31/03/2026 2 DE 2
MOVIMIENTO DE SU CUENTA SALDO DIARIO
FECHA NUMERO SUC DESCRIPCION CHEQUES Y DEPOSITOS Y SALDO
CARGOS ABONOS
23/03 93 Compra PAYU *UBER TRIP 4.217
401 0555555555 Transf a COMERCIO DOS 20.000
--- Saldo Dia --- 35.783
24/03 93 Compra UBER 3.990
--- Saldo Dia --- 31.793
26/03 93 Compra DL*GOOGLE YOUTUBE 7.990
--- Saldo Dia --- 23.803
27/03 401 0555555555 Transf a COMERCIO DOS 20.000
--- Saldo Dia --- 3.803
30/03 0390684 401 UBER 3.733
9260887 401 0111111111 Transf. 20.000
--- Saldo Dia --- 27.536
Resumen de Comisiones
SIN COMISIONES EN EL PERIODO
Banco Santander Chile.";

/// The movements header as Santander draws it: "CHEQUES Y" over "CARGOS",
/// "DEPOSITOS Y" over "ABONOS", amounts right-aligned under them.
fn santander_header(page: usize) -> Vec<Line> {
    vec![
        positioned(
            page,
            258.0,
            &[
                ("FECHA", 35.0, 59.0),
                ("NUMERO", 69.0, 100.0),
                ("SUC", 111.0, 126.0),
                ("DESCRIPCION", 188.0, 244.0),
                ("CHEQUES", 321.0, 361.0),
                ("Y", 363.0, 368.0),
                ("DEPOSITOS", 406.0, 453.0),
                ("Y", 455.0, 461.0),
                ("SALDO", 510.0, 538.0),
            ],
        ),
        positioned(
            page,
            268.0,
            &[("CARGOS", 327.0, 362.0), ("ABONOS", 416.0, 451.0)],
        ),
    ]
}

fn santander_multiline(layout: bool) -> Document {
    let doc = Document::from_pages(&[SANTANDER_PAGE_1, SANTANDER_PAGE_2]);
    if !layout {
        return doc;
    }
    let mut lines = santander_header(1);
    lines.push(positioned(
        1,
        394.0,
        &[
            ("30/03", 35.0, 59.0),
            ("0390684", 69.0, 103.0),
            ("401", 112.0, 127.0),
            ("UBER", 136.0, 160.0),
            ("3.733", 453.0, 477.0),
        ],
    ));
    with_layout(doc, lines)
}

const COPEC_SIGNED: &str = "\
COPECPAY
HISTORIAL DE MOVIMIENTOS
Fecha Descripción Cargos Abonos Saldo
01/05/2026 10:30 COMPRA ESTACION DE SERVICIO -15.000
02/05/2026 09:15 RECARGA DESDE BANCO +25.000";

const COPEC_COLUMNS: &str = "\
COPECPAY
HISTORIAL DE MOVIMIENTOS
DESDE: 01/05/2026 HASTA: 31/05/2026
Fecha Descripción Cargos Abonos Saldo
03/05/2026 Recarga desde banco 0 40.000 55.000
04/05/2026 Compra combustible 18.750 0 36.250
31/05/2026 Compra tarjeta digital DOM AHUMADA 146 16.940 0 19.310
Total Cargos Total Abonos Saldo Final
35.690 40.000 19.310";

const BANCO_CHILE_CREDIT: &str = "\
ESTADO DE CUENTA NACIONAL DE TARJETA DE CRÉDITO
Nº DE TARJETA DE CRÉDITO XXXX XXXX XXXX 4321
PERÍODO FACTURADO 21/06/2026 20/07/2026
II. DETALLE
MONTO FACTURADO A PAGAR (PERÍODO ANTERIOR) A $ 150.000
01/07/26 123456789012 PAGO RECIBIDO GRACIAS $ -150.000 $ -150.000 01/01 $ -150.000
TOTAL PAGOS A LA CUENTA B $ -150.000
TOTAL PAT A LA CUENTA C $ 0
03/07/26 223456789012 SUPERMERCADO EJEMPLO $ 25.990 $ 25.990 01/01 $ 25.990
SANTIAGO 08/07/26 323456789012 FARMACIA EJEMPLO $ 8.500 $ 8.500 01/01 $ 8.500
TOTAL TRANSACCIONES EN UNA CUOTA D $ 34.490
10/06/26 423456789012 NOTEBOOK EJEMPLO $ 600.000 $ 600.000 02/06 $ 100.000
TOTAL TRANSACCIONES EN CUOTAS E $ 100.000
TOTAL PRODUCTOS O SERVICIOS VOLUNTARIAMENTE CONTRATADOS F $ 0
15/07/26 523456789012 IMPUESTO DE TIMBRES $ 1.200 $ 1.200 01/01 $ 1.200
TOTAL CARGOS, COMISIONES, IMPUESTOS Y ABONOS G $ 1.200
MONTO TOTAL FACTURADO A PAGAR (A+B+C+D+E+F+G) $ 135.690
III. INFORMACIÓN DE PAGO
PAGAR HASTA 05/08/2026";

const SANTANDER_CREDIT: &str = "\
ESTADO DE CUENTA EN MONEDA NACIONAL DE TARJETA DE CRÉDITO
Nº DE TARJETA DE CRÉDITO XXXX XXXX XXXX 1234
PERIODO FACTURADO 23/07/2026 25/08/2026
II. DETALLE
2. PERIODO ACTUAL
1. TOTAL OPERACIONES $ 130.000
MOVIMIENTOS TARJETA XXXX-1234 $ 130.000
SANTIAGO 24/07/2026 COMERCIO EJEMPLO $ 100.000
25/07/2026 COMPRA EN CUOTAS TRES CUOTAS PREC 0,00 % $ 90.000 $ 90.000 01/03 $ 30.000
31/07/2026 MONTO CANCELADO $ -50.000
EMISOR CLIENTE
COMPROBANTE DE PAGO
09/09/2026 $ 132.000
2. PERIODO ACTUAL
2. PRODUCTOS O SERVICIOS VOLUNTARIAMENTE CONTRATADOS $ 0
3. CARGOS, COMISIONES, IMPUESTOS Y ABONOS $ 2.000
25/08/2026 INTERESES $ 2.000
4. INFORMACION COMPRAS EN CUOTAS EN EL PERIODO $ 10.000
25/08/2026 OTRA COMPRA 03 CUOTAS $ 30.000 $ 10.000
III. INFORMACIÓN DE PAGO
MONTO TOTAL FACTURADO A PAGAR $ 132.000";

fn parse(text: &str, format: StatementFormat) -> ParsedStatement {
    parse_statement(&Document::from_pages(&[text]), format).unwrap()
}

// ─── Tokens and periods ──────────────────────────────────────────────────────

#[test]
fn amounts_follow_chilean_formatting() {
    let value = |t: &str| parse_amount(t).map(|a| a.signed());
    assert_eq!(value("1.449.225"), Some(1_449_225));
    assert_eq!(value("-15.000"), Some(-15_000));
    assert_eq!(value("+25.000"), Some(25_000));
    assert_eq!(value("-$5.490"), Some(-5_490));
    assert_eq!(value("$+1.000"), Some(1_000));
    assert_eq!(value("5.490,00"), Some(5_490));
    assert_eq!(value("0"), Some(0));
    // Cents mean the token is not a peso amount.
    assert_eq!(value("7,50"), None);
    // Misplaced separators are not amounts.
    assert_eq!(value("1.23"), None);
    assert_eq!(value("1234.567"), None);
    assert_eq!(value("$"), None);
    assert_eq!(value("30/03"), None);
    assert_eq!(value("A:Persona"), None);
}

#[test]
fn periods_in_every_known_layout() {
    let tokens = |t: &str| Document::from_pages(&[t]).folded_tokens();
    let rules = [
        PeriodRule::DesdeHasta,
        PeriodRule::PeriodoRange,
        PeriodRule::AfterHeader(&["CARTOLA", "DESDE", "HASTA", "PAGINA"]),
    ];
    let cases = [
        (
            "PERIODO DESDE: 01/05/2026 HASTA: 31/05/2026",
            (date(2026, 5, 1), date(2026, 5, 31)),
        ),
        (
            "DESDE 01/05/2026 AL 31/05/2026",
            (date(2026, 5, 1), date(2026, 5, 31)),
        ),
        (
            "DESDE:01/05/2026 HASTA:31/05/2026",
            (date(2026, 5, 1), date(2026, 5, 31)),
        ),
        (
            "Período: 01/05/2026 - 31/05/2026",
            (date(2026, 5, 1), date(2026, 5, 31)),
        ),
        (
            "CARTOLA DESDE HASTA PAGINA\n0-000-00-12345-6 75 27/02/2026 31/03/2026 1 DE 2",
            (date(2026, 2, 27), date(2026, 3, 31)),
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(
            find_period(&tokens(text), &rules),
            Ok(Some(expected)),
            "{text}"
        );
    }
    assert_eq!(
        find_period(&tokens("DESDE 31/05/2026 HASTA 01/05/2026"), &rules),
        Err(StatementError::InvalidPeriod)
    );
}

// ─── Account statements ──────────────────────────────────────────────────────

#[test]
fn banco_chile_rows_and_year_from_period() {
    let statement = parse(BANCO_CHILE, StatementFormat::BancoChileAccount);
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    assert_eq!(
        statement.period,
        Some((date(2025, 12, 30), date(2026, 4, 1)))
    );
    assert_eq!(amounts(&statement), [-43_383, 43_383, -20_200, 1_449_225]);
    assert_eq!(statement.rows[0].date, date(2026, 1, 7));
    assert!(statement.rows[3].description.contains("0111111111 CENTRAL"));
    // Without layout, the sign came from words like "TRASPASO A:".
    assert!(statement.rows.iter().all(|r| r.sign_from_keyword));
}

#[test]
fn santander_simple_rows_drop_amount_labels() {
    let statement = parse(SANTANDER_SIMPLE, StatementFormat::SantanderAccount);
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    assert_eq!(amounts(&statement), [-12_345, 50_000]);
    assert_eq!(statement.rows[0].description, "COMPRA SUPERMERCADO");
}

#[test]
fn santander_multiline_statement_matches_its_summary() {
    let statement = parse_statement(
        &santander_multiline(true),
        StatementFormat::SantanderAccount,
    )
    .unwrap();
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    assert!(statement.warnings.is_empty(), "{:?}", statement.warnings);
    assert_eq!(
        statement.period,
        Some((date(2026, 2, 27), date(2026, 3, 31)))
    );
    assert_eq!(statement.rows.len(), 23);

    let income: i64 = statement
        .rows
        .iter()
        .filter(|r| r.amount > 0)
        .map(|r| r.amount)
        .sum();
    let expense: i64 = statement
        .rows
        .iter()
        .filter(|r| r.amount < 0)
        .map(|r| -r.amount)
        .sum();
    assert_eq!((expense, income), (1_219_871, 1_215_397));

    let find = |text: &str| {
        statement
            .rows
            .iter()
            .find(|r| r.description.contains(text))
            .unwrap()
    };
    assert_eq!(find("KUSHK").description, "Compra PAGO ONLINE KUSHK");
    assert_eq!(find("PERSONA DOS").description, "Transf. PERSONA DOS");
    assert_eq!(
        find("PERSONA DOS").raw_description,
        "2157027 401 0444444444 Transf. PERSONA DOS"
    );
    assert_eq!(find("PERSONA UNO").description, "Transf a PERSONA UNO");

    // Only the column says this one is money in (a refund).
    let refund = statement
        .rows
        .iter()
        .find(|r| r.description == "UBER")
        .unwrap();
    assert_eq!((refund.amount, refund.date), (3_733, date(2026, 3, 30)));
    assert!(!refund.sign_from_keyword);

    // Two identical transfers on the same day are two movements.
    let same_day = statement
        .rows
        .iter()
        .filter(|r| r.date == date(2026, 3, 19) && r.amount == 10_000)
        .count();
    assert_eq!(same_day, 2);
}

#[test]
fn santander_without_layout_reports_the_row_it_cannot_sign() {
    let statement = parse_statement(
        &santander_multiline(false),
        StatementFormat::SantanderAccount,
    )
    .unwrap();
    assert_eq!(statement.rows.len(), 22);
    assert_eq!(statement.problems.len(), 1);
    assert_eq!(statement.problems[0].issue, RowIssue::UnknownSign);
    assert!(statement.problems[0].text.contains("UBER 3.733"));
    assert!(matches!(
        statement.warnings.as_slice(),
        [StatementWarning::TotalsMismatch { .. }]
    ));
}

#[test]
fn santander_summary_variants() {
    let without_abonos =
        SANTANDER_PAGE_1.replace("Depositos o Abonos Saldo Final", "Depositos o Saldo Final");
    let doc = Document::from_pages(&[&without_abonos, SANTANDER_PAGE_2]);
    let mut lines = santander_header(1);
    lines.push(positioned(
        1,
        394.0,
        &[
            ("30/03", 35.0, 59.0),
            ("0390684", 69.0, 103.0),
            ("401", 112.0, 127.0),
            ("UBER", 136.0, 160.0),
            ("3.733", 453.0, 477.0),
        ],
    ));
    let statement =
        parse_statement(&with_layout(doc, lines), StatementFormat::SantanderAccount).unwrap();
    assert!(statement.warnings.is_empty(), "{:?}", statement.warnings);

    let unreadable = SANTANDER_PAGE_1.replace(
        "Saldo Inicial Cheques o Cargos Depositos o Abonos Saldo Final",
        "Saldo Inicial Resumen no compatible Saldo Final",
    );
    let statement = parse_statement(
        &Document::from_pages(&[&unreadable, SANTANDER_PAGE_2]),
        StatementFormat::SantanderAccount,
    )
    .unwrap();
    assert!(
        statement
            .warnings
            .contains(&StatementWarning::TotalsUnreadable)
    );

    let missing_row = SANTANDER_PAGE_1.replace("93 Compra RIPLEY FLORIDA 20.240\n", "");
    let statement = parse_statement(
        &Document::from_pages(&[&missing_row, SANTANDER_PAGE_2]),
        StatementFormat::SantanderAccount,
    )
    .unwrap();
    assert!(matches!(
        statement.warnings.as_slice(),
        [StatementWarning::TotalsMismatch { declared, parsed }] if declared.expense - parsed.expense >= 20_240
    ));
}

#[test]
fn santander_layout_tells_balance_from_movement() {
    let text = "\
BANCO SANTANDER CHILE
PERIODO DESDE: 01/05/2026 HASTA: 31/05/2026
MOVIMIENTO DE SU CUENTA SALDO DIARIO
FECHA NUMERO SUC DESCRIPCION CHEQUES Y DEPOSITOS Y SALDO
CARGOS ABONOS
01/05 93 Transf. MISMO BANCO 20.000 20.000
09/05 93 Compra FLORIDA CENTER 2 21.945";
    let mut lines = santander_header(0);
    lines.push(positioned(
        0,
        286.0,
        &[
            ("01/05", 35.0, 59.0),
            ("93", 69.0, 82.0),
            ("Transf.", 136.0, 170.0),
            ("MISMO", 174.0, 205.0),
            ("BANCO", 209.0, 244.0),
            ("20.000", 330.0, 360.0),
            ("20.000", 512.0, 542.0),
        ],
    ));
    lines.push(positioned(
        0,
        296.0,
        &[
            ("09/05", 35.0, 59.0),
            ("93", 69.0, 82.0),
            ("Compra", 136.0, 170.0),
            ("FLORIDA", 174.0, 216.0),
            ("CENTER", 220.0, 256.0),
            ("2", 241.0, 247.0),
            ("21.945", 357.0, 386.0),
        ],
    ));
    let doc = with_layout(Document::from_pages(&[text]), lines);
    let statement = parse_statement(&doc, StatementFormat::SantanderAccount).unwrap();
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    // The second 20.000 is the balance; "Transf." alone would say income.
    assert_eq!(amounts(&statement), [-20_000, -21_945]);
    assert_eq!(statement.rows[0].description, "Transf. MISMO BANCO");
    assert_eq!(statement.rows[1].description, "Compra FLORIDA CENTER 2");
}

#[test]
fn copec_pay_signed_rows_infer_the_period() {
    let statement = parse(COPEC_SIGNED, StatementFormat::CopecPay);
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    assert_eq!(amounts(&statement), [-15_000, 25_000]);
    assert_eq!(statement.period, Some((date(2026, 5, 1), date(2026, 5, 2))));
    assert!(statement.rows.iter().all(|r| !r.sign_from_keyword));
}

#[test]
fn copec_pay_zero_filled_columns() {
    let statement = parse(COPEC_COLUMNS, StatementFormat::CopecPay);
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    assert!(statement.warnings.is_empty(), "{:?}", statement.warnings);
    assert_eq!(amounts(&statement), [40_000, -18_750, -16_940]);
    let descriptions: Vec<&str> = statement
        .rows
        .iter()
        .map(|r| r.description.as_str())
        .collect();
    assert_eq!(
        descriptions,
        [
            "Recarga desde banco",
            "Compra combustible",
            "Compra tarjeta digital DOM AHUMADA 146"
        ]
    );
    // The zero in the other column settles the sign, no keyword needed.
    assert!(statement.rows.iter().all(|r| !r.sign_from_keyword));

    let incomplete = COPEC_COLUMNS.replace("04/05/2026 Compra combustible 18.750 0 36.250\n", "");
    let statement = parse(&incomplete, StatementFormat::CopecPay);
    assert!(matches!(
        statement.warnings.as_slice(),
        [StatementWarning::TotalsMismatch { .. }]
    ));
}

#[test]
fn wrong_bank_is_refused() {
    let doc = Document::from_pages(&[SANTANDER_SIMPLE]);
    assert_eq!(
        parse_statement(&doc, StatementFormat::CopecPay),
        Err(StatementError::WrongFormat(StatementFormat::CopecPay))
    );
}

#[test]
fn keywords_match_whole_words() {
    let text = "\
BANCO SANTANDER CHILE
PERIODO DESDE: 01/05/2026 HASTA: 31/05/2026
03/05 RESTAURANT PACIFICO 9.900
04/05 ABONOS VARIOS 5.000";
    let statement = parse(text, StatementFormat::SantanderAccount);
    // "PAC" must not fire inside "PACIFICO"; "ABONO" accepts the plural.
    assert_eq!(statement.problems[0].issue, RowIssue::UnknownSign);
    assert_eq!(amounts(&statement), [5_000]);
}

// ─── Credit cards ────────────────────────────────────────────────────────────

#[test]
fn banco_chile_credit_imports_billed_sections_only() {
    let statement = parse(BANCO_CHILE_CREDIT, StatementFormat::BancoChileCredit);
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    assert!(statement.warnings.is_empty(), "{:?}", statement.warnings);
    // The payment in section B is not here.
    assert_eq!(amounts(&statement), [-25_990, -8_500, -100_000, -1_200]);
    assert_eq!(statement.rows[1].description, "FARMACIA EJEMPLO");
    assert_eq!(statement.rows[2].installment, Some((2, 6)));
    assert_eq!(statement.rows[0].installment, None);
    assert_eq!(
        statement.period,
        Some((date(2026, 6, 21), date(2026, 7, 20)))
    );

    let off = BANCO_CHILE_CREDIT.replace("D $ 34.490", "D $ 34.000");
    let statement = parse(&off, StatementFormat::BancoChileCredit);
    assert!(
        statement
            .warnings
            .contains(&StatementWarning::SectionMismatch {
                section: "D",
                declared: 34_000,
                parsed: 34_490,
            })
    );
}

#[test]
fn santander_credit_skips_payments_and_future_installments() {
    let statement = parse(SANTANDER_CREDIT, StatementFormat::SantanderCredit);
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    assert!(statement.warnings.is_empty(), "{:?}", statement.warnings);
    assert_eq!(amounts(&statement), [-100_000, -30_000, -2_000]);
    assert_eq!(statement.rows[1].description, "COMPRA EN CUOTAS");
    assert_eq!(statement.rows[1].installment, Some((1, 3)));

    let off = SANTANDER_CREDIT.replace("A PAGAR $ 132.000", "A PAGAR $ 131.999");
    let statement = parse(&off, StatementFormat::SantanderCredit);
    assert!(
        statement
            .warnings
            .contains(&StatementWarning::SectionMismatch {
                section: "total",
                declared: 131_999,
                parsed: 132_000,
            })
    );
}

#[test]
fn credit_card_statement_is_not_an_account_statement() {
    let doc = Document::from_pages(&[SANTANDER_CREDIT]);
    assert_eq!(detect_format(&doc), Some(StatementFormat::SantanderCredit));
    let doc = Document::from_pages(&[SANTANDER_SIMPLE]);
    assert_eq!(
        parse_statement(&doc, StatementFormat::SantanderCredit),
        Err(StatementError::WrongFormat(
            StatementFormat::SantanderCredit
        ))
    );
}

#[test]
fn detection_reads_the_heading() {
    let detect = |t: &str| detect_format(&Document::from_pages(&[t]));
    assert_eq!(
        detect(BANCO_CHILE),
        Some(StatementFormat::BancoChileAccount)
    );
    assert_eq!(
        detect(SANTANDER_SIMPLE),
        Some(StatementFormat::SantanderAccount)
    );
    assert_eq!(detect(COPEC_COLUMNS), Some(StatementFormat::CopecPay));
    assert_eq!(
        detect(BANCO_CHILE_CREDIT),
        Some(StatementFormat::BancoChileCredit)
    );
    // Two banks named in the heading: let the user choose.
    assert_eq!(
        detect(&format!(
            "{SANTANDER_SIMPLE}\n03/05 TRANSF A BANCO DE CHILE 1.000"
        )),
        None
    );
}
