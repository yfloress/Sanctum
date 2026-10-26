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

//! Account statement layouts of each supported bank.
//!
//! Keywords and markers are folded (uppercase, no diacritics). Income
//! keywords are checked before expense ones, so "PAGO RECIBIDO" wins over
//! "PAGO".

use super::StatementFormat;
use super::account::{AccountProfile, ColumnHeaders, ColumnOrder, Totals, amounts_after};
use super::clp::is_digits;
use super::document::fold;
use super::period::PeriodRule;

pub const BANCO_CHILE: AccountProfile = AccountProfile {
    format: StatementFormat::BancoChileAccount,
    markers: &["BANCO DE CHILE", "BANCOCHILE"],
    periods: &[PeriodRule::DesdeHasta],
    income_keywords: &["TRASPASO DE:", "ABONO", "DEPOSITO", "PAGO:DE HONORARIOS"],
    expense_keywords: &[
        "TRASPASO A:",
        "PAGO:",
        "COMPRA",
        "GIRO",
        "CARGO",
        "COMISION",
    ],
    continuation_lines: false,
    start_markers: &[],
    end_markers: &[],
    columns: Some(ColumnHeaders {
        expense: &["CHEQUES", "CARGOS"],
        income: &["DEPOSITOS", "ABONOS"],
        balance: &["SALDO"],
        description: &["DETALLE"],
    }),
    zero_filled: None,
    summary: None,
    clean_description: None,
};

/// "FECHA DETALLE CARGOS ABONOS SALDO", with `0` in the empty column.
pub const BANCO_ESTADO: AccountProfile = AccountProfile {
    format: StatementFormat::BancoEstadoAccount,
    markers: &["BANCOESTADO", "BANCO ESTADO", "CUENTARUT"],
    periods: &[PeriodRule::DesdeHasta],
    income_keywords: &[
        "ABONO",
        "DEPOSITO",
        "TRANSFERENCIA RECIBIDA",
        "REMUNERACION",
        "PAGO RECIBIDO",
    ],
    expense_keywords: &[
        "CARGO",
        "COMPRA",
        "PAGO",
        "GIRO",
        "COMISION",
        "TRANSFERENCIA ENVIADA",
        "PAC",
    ],
    continuation_lines: false,
    start_markers: &[],
    end_markers: &[],
    columns: Some(ColumnHeaders {
        expense: &["CARGOS", "CARGO"],
        income: &["ABONOS", "ABONO"],
        balance: &["SALDO"],
        description: &["DETALLE"],
    }),
    zero_filled: Some(ColumnOrder::ExpenseFirst),
    summary: None,
    clean_description: None,
};

pub const SANTANDER: AccountProfile = AccountProfile {
    format: StatementFormat::SantanderAccount,
    markers: &["BANCO SANTANDER", "SANTANDER CHILE"],
    periods: &[
        PeriodRule::DesdeHasta,
        PeriodRule::PeriodoRange,
        PeriodRule::AfterHeader(&["CARTOLA", "DESDE", "HASTA", "PAGINA"]),
    ],
    income_keywords: &[
        "ABONO",
        "DEPOSITO",
        "TRANSFERENCIA RECIBIDA",
        "TRASPASO RECIBIDO",
        "REMUNERACION",
        "PAGO RECIBIDO",
        "TRANSF.",
    ],
    expense_keywords: &[
        "CARGO",
        "COMPRA",
        "PAGO",
        "GIRO",
        "COMISION",
        "TRANSFERENCIA ENVIADA",
        "TRANSF A ",
        "PAC",
        "PAT",
    ],
    continuation_lines: true,
    start_markers: &[
        "MOVIMIENTO DE SU CUENTA",
        "CARGOS ABONOS",
        "CARGOS Y ABONOS",
        "CARGO ABONO",
        "CARGO Y ABONO",
    ],
    end_markers: &["MENSAJES", "RESUMEN DE COMISIONES"],
    columns: Some(ColumnHeaders {
        expense: &["CHEQUES", "CARGOS"],
        income: &["DEPOSITOS", "ABONOS"],
        balance: &["SALDO"],
        description: &["DESCRIPCION"],
    }),
    zero_filled: None,
    summary: Some(santander_totals),
    clean_description: Some(clean_santander),
};

/// Three layouts: signed amounts ("DINERO RECIBIDO +20.000"), income and
/// expense columns with `0` in the empty one, and the account statement
/// ("Abono Ganancias <id> CLP 7,00 0,00") whose amount is signed and whose
/// last figure is "otros conceptos".
pub const MERCADO_PAGO: AccountProfile = AccountProfile {
    format: StatementFormat::MercadoPago,
    markers: &["MERCADO PAGO", "MERCADOPAGO"],
    periods: &[
        PeriodRule::DesdeHasta,
        PeriodRule::PeriodoRange,
        PeriodRule::AfterHeader(&["DESDE", "HASTA", "FECHA", "DE", "GENERACION"]),
    ],
    income_keywords: &[
        "ABONO",
        "DINERO RECIBIDO",
        "PAGO RECIBIDO",
        "COBRO",
        "VENTA",
        "INGRESO",
        "DEVOLUCION",
        "RENDIMIENTO",
    ],
    expense_keywords: &[
        "COMPRA",
        "PAGO",
        "RETIRO",
        "COMISION",
        "TRANSFERENCIA ENVIADA",
        "DINERO ENVIADO",
        "CARGO",
    ],
    continuation_lines: false,
    start_markers: &["FECHA DE ACREDITACION"],
    end_markers: &[],
    columns: Some(ColumnHeaders {
        expense: &["EGRESO", "EGRESOS", "CARGO", "CARGOS"],
        income: &["INGRESO", "INGRESOS", "ABONO", "ABONOS"],
        balance: &["SALDO"],
        description: &["DESCRIPCION", "DETALLE", "MOVIMIENTO", "OPERACION"],
    }),
    zero_filled: Some(ColumnOrder::IncomeFirst),
    summary: None,
    clean_description: Some(clean_mercado_pago),
};

pub const COPEC_PAY: AccountProfile = AccountProfile {
    format: StatementFormat::CopecPay,
    markers: &["COPECPAY", "COPEC PAY"],
    periods: &[PeriodRule::DesdeHasta],
    income_keywords: &[
        "RECARGA",
        "CARGA",
        "ABONO",
        "DEVOLUCION",
        "REEMBOLSO",
        "TRANSFERENCIA RECIBIDA",
        "GANANCIA",
        "TRANSFERENCIA DE",
    ],
    expense_keywords: &[
        "COMPRA",
        "CONSUMO",
        "PAGO",
        "RETIRO",
        "COMISION",
        "TRANSFERENCIA ENVIADA",
        "TRANSFERENCIA A",
    ],
    continuation_lines: false,
    start_markers: &["FECHA DESCRIPCION"],
    end_markers: &["TOTAL CARGOS TOTAL ABONOS SALDO FINAL"],
    columns: Some(ColumnHeaders {
        expense: &["CARGOS", "CARGO"],
        income: &["ABONOS", "ABONO"],
        balance: &["SALDO"],
        description: &["DESCRIPCION"],
    }),
    zero_filled: Some(ColumnOrder::ExpenseFirst),
    summary: Some(copec_pay_totals),
    clean_description: None,
};

/// "Saldo Inicial | Cheques o Cargos | Depositos o Abonos | Saldo Final",
/// then the four figures. Some statements drop the word "Abonos".
fn santander_totals(tokens: &[String]) -> Option<Totals> {
    const FULL: &[&str] = &[
        "SALDO",
        "INICIAL",
        "CHEQUES",
        "O",
        "CARGOS",
        "DEPOSITOS",
        "O",
        "ABONOS",
        "SALDO",
        "FINAL",
    ];
    const SHORT: &[&str] = &[
        "SALDO",
        "INICIAL",
        "CHEQUES",
        "O",
        "CARGOS",
        "DEPOSITOS",
        "O",
        "SALDO",
        "FINAL",
    ];
    let amounts = amounts_after(tokens, FULL, 4).or_else(|| amounts_after(tokens, SHORT, 4))?;
    Some(Totals {
        expense: amounts[1],
        income: amounts[2],
    })
}

fn copec_pay_totals(tokens: &[String]) -> Option<Totals> {
    const LABEL: &[&str] = &["TOTAL", "CARGOS", "TOTAL", "ABONOS", "SALDO", "FINAL"];
    let amounts = amounts_after(tokens, LABEL, 3)?;
    Some(Totals {
        expense: amounts[0],
        income: amounts[1],
    })
}

/// Drops the operation codes Santander prints before the description:
/// "93 Compra UBER" → "Compra UBER", "9260651 401 0111111111 Transf." →
/// "Transf.", "0390684 401 UBER" → "UBER". The raw text keeps them, since
/// the account number of a transfer says who it came from.
fn clean_santander(words: &[String]) -> Vec<String> {
    let folded: Vec<String> = words.iter().map(|w| fold(w)).collect();
    let is_transfer = |i: usize| {
        folded.get(i).is_some_and(|w| w == "TRANSF.")
            || (folded.get(i).is_some_and(|w| w == "TRANSF")
                && folded.get(i + 1).is_some_and(|w| w == "A"))
    };
    let leading_codes = folded.iter().take_while(|w| is_digits(w)).count();

    let skip = if leading_codes > 0 && leading_codes <= 3 && is_transfer(leading_codes) {
        leading_codes
    } else if leading_codes == 1
        && folded[0] == "93"
        && folded.get(1).is_some_and(|w| w == "COMPRA")
    {
        1
    } else if leading_codes == 2
        && (6..=8).contains(&folded[0].len())
        && folded[1] == "401"
        && folded
            .get(2)
            .and_then(|w| w.chars().next())
            .is_some_and(|c| c.is_alphabetic() || c == '*')
    {
        2
    } else {
        0
    };
    words[skip..].to_vec()
}

/// "Abono Ganancias 1111111111111" → "Ganancias": the movement type is
/// already in the sign and the transaction id is unique to each row.
fn clean_mercado_pago(words: &[String]) -> Vec<String> {
    let mut words = words.to_vec();
    if words.last().is_some_and(|w| w.len() >= 10 && is_digits(w)) {
        words.pop();
    }
    if words.len() > 1 && matches!(fold(&words[0]).as_str(), "ABONO" | "CARGO") {
        words.remove(0);
    }
    words
}
