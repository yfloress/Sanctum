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

//! Checks the parser against your own statements without printing them.
//!
//! ```text
//! SANCTUM_STATEMENTS_DIR=/path/to/pdfs SANCTUM_STATEMENT_PASSWORD=12345678 \
//!     cargo test statement_samples -- --ignored --nocapture
//! ```
//!
//! For each PDF it prints the detected format, how many rows parsed, the
//! kind of each problem and warning, and the shape of unreadable rows with
//! digits and letters masked ("dd/dd Aaaa 9.999"). No amount, name or
//! description is printed, so the output is safe to share.

use super::*;

/// "02/03 93 Compra UBER 4.516" → "99/99 99 Aaaaaa AAAA 9.999".
fn shape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '0'..='9' => '9',
            c if c.is_uppercase() => 'A',
            c if c.is_alphabetic() => 'a',
            other => other,
        })
        .collect()
}

#[test]
#[ignore = "reads local files; run by hand"]
fn statement_samples() {
    let Ok(dir) = std::env::var("SANCTUM_STATEMENTS_DIR") else {
        println!("set SANCTUM_STATEMENTS_DIR to a folder of statement PDFs");
        return;
    };
    let password = std::env::var("SANCTUM_STATEMENT_PASSWORD").ok();
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .expect("readable folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf")))
        .collect();
    paths.sort();

    for (index, path) in paths.iter().enumerate() {
        println!("\n# file {}", index + 1);
        let bytes = std::fs::read(path).expect("readable file");
        let doc = match read_pdf(&bytes, None) {
            Err(PdfError::PasswordRequired) => read_pdf(&bytes, password.as_deref()),
            other => other,
        };
        let doc = match doc {
            Ok(doc) => doc,
            Err(error) => {
                println!("pdf: {error}");
                continue;
            }
        };
        let positioned = doc.lines.iter().filter(|l| l.is_positioned()).count();
        println!("lines: {} ({} positioned)", doc.lines.len(), positioned);
        let Some(format) = detect_format(&doc) else {
            println!("format: not detected; first lines:");
            for line in doc.lines.iter().take(8) {
                println!("  {}", shape(&line.text()));
            }
            continue;
        };
        println!("format: {format:?}");
        match parse_statement(&doc, format) {
            Ok(statement) => {
                let guessed = statement
                    .rows
                    .iter()
                    .filter(|r| r.sign_from_keyword)
                    .count();
                println!(
                    "rows: {} ({} signed by keyword), period found: {}",
                    statement.rows.len(),
                    guessed,
                    statement.period.is_some()
                );
                for warning in &statement.warnings {
                    let kind = match warning {
                        StatementWarning::TotalsMismatch { .. } => {
                            "totals do not match".to_string()
                        }
                        StatementWarning::SectionMismatch { section, .. } => {
                            format!("section {section} does not match")
                        }
                        StatementWarning::TotalsUnreadable => "summary unreadable".to_string(),
                    };
                    println!("warning: {kind}");
                }
                for problem in &statement.problems {
                    println!("problem {:?}: {}", problem.issue, shape(&problem.text));
                }
            }
            Err(error) => println!("parse: {error}"),
        }
    }
}
