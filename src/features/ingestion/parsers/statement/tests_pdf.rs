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

//! PDF extraction against a statement built in memory, with text placed
//! at fixed positions like a real cartola.

use pdf_extract::content::{Content, Operation};
use pdf_extract::{Object, Stream, dictionary};

use super::*;

const PAGE_HEIGHT: f32 = 800.0;

/// One page; each entry is (x, top, text) in points.
fn build_pdf(texts: &[(f32, f32, &str)]) -> Vec<u8> {
    let mut doc = pdf_extract::Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
    });
    let mut operations = Vec::new();
    for (x, top, text) in texts {
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F1".into(), 8.into()]));
        operations.push(Operation::new(
            "Tm",
            vec![
                1.into(),
                0.into(),
                0.into(),
                1.into(),
                (*x).into(),
                (PAGE_HEIGHT - top - 8.0).into(),
            ],
        ));
        operations.push(Operation::new("Tj", vec![Object::string_literal(*text)]));
        operations.push(Operation::new("ET", vec![]));
    }
    let content = Content { operations };
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "Contents" => content_id,
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 600.into(), (PAGE_HEIGHT as i64).into()],
        }),
    );
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    bytes
}

fn santander_pdf() -> Vec<u8> {
    build_pdf(&[
        (35.0, 40.0, "BANCO SANTANDER CHILE"),
        (35.0, 60.0, "PERIODO DESDE: 01/05/2026 HASTA: 31/05/2026"),
        (35.0, 90.0, "MOVIMIENTO DE SU CUENTA"),
        (35.0, 120.0, "FECHA"),
        (188.0, 120.0, "DESCRIPCION"),
        (321.0, 120.0, "CHEQUES Y"),
        (406.0, 120.0, "DEPOSITOS Y"),
        (510.0, 120.0, "SALDO"),
        (327.0, 130.0, "CARGOS"),
        (416.0, 130.0, "ABONOS"),
        (35.0, 150.0, "02/05"),
        (69.0, 150.0, "93 Compra UBER"),
        (335.0, 150.0, "4.516"),
        (35.0, 165.0, "05/05"),
        (69.0, 165.0, "0390684 401 UBER"),
        (440.0, 165.0, "3.733"),
        (35.0, 200.0, "MENSAJES"),
    ])
}

#[test]
fn pdf_words_keep_their_position() {
    let doc = read_pdf(&santander_pdf(), None).unwrap();
    let texts: Vec<String> = doc.lines.iter().map(Line::text).collect();
    assert_eq!(texts[0], "BANCO SANTANDER CHILE");
    assert_eq!(texts[1], "PERIODO DESDE: 01/05/2026 HASTA: 31/05/2026");
    assert!(
        texts.contains(&"02/05 93 Compra UBER 4.516".to_string()),
        "{texts:?}"
    );
    assert!(doc.lines.iter().all(Line::is_positioned));

    let row = doc
        .lines
        .iter()
        .find(|l| l.text().starts_with("02/05"))
        .unwrap();
    let amount = row.words.last().unwrap();
    let (x0, x1) = amount.span.unwrap();
    assert!((x0 - 335.0).abs() < 0.5, "{x0}");
    assert!(x1 > x0 + 10.0 && x1 < x0 + 30.0, "{x1}");
}

#[test]
fn pdf_statement_takes_the_sign_from_the_column() {
    let doc = read_pdf(&santander_pdf(), None).unwrap();
    assert_eq!(detect_format(&doc), Some(StatementFormat::SantanderAccount));
    let statement = parse_statement(&doc, StatementFormat::SantanderAccount).unwrap();
    assert!(statement.problems.is_empty(), "{:?}", statement.problems);
    let rows: Vec<(&str, i64, bool)> = statement
        .rows
        .iter()
        .map(|r| (r.description.as_str(), r.amount, r.sign_from_keyword))
        .collect();
    // "UBER" names no direction; only the credit column says it is a refund.
    assert_eq!(
        rows,
        [("Compra UBER", -4_516, false), ("UBER", 3_733, false)]
    );
}

#[test]
fn pdf_errors_are_values_not_panics() {
    assert_eq!(
        read_pdf(b"not a pdf at all", None),
        Err(PdfError::Unreadable)
    );
    assert_eq!(read_pdf(&build_pdf(&[]), None), Err(PdfError::NoText));
    let truncated = &santander_pdf()[..200];
    assert!(read_pdf(truncated, None).is_err());
}
