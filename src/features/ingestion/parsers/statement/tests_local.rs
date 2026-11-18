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
//! SANCTUM_STATEMENTS_DIR=/path/to/pdfs \
//!     cargo test statement_samples -- --ignored --nocapture
//! ```
//!
//! For encrypted PDFs, set SANCTUM_STATEMENT_PASSWORD in the environment.
//! Each file is identified by its position in the sorted directory listing.
//! Diagnostics use fixed status messages.

use super::*;

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
            Err(_) => {
                println!("PDF could not be read");
                continue;
            }
        };
        let Some(format) = detect_format(&doc) else {
            println!("statement format not recognized");
            continue;
        };
        match parse_statement(&doc, format) {
            Ok(statement) => {
                if statement.rows.is_empty() {
                    println!("no rows parsed");
                } else {
                    println!("rows parsed");
                }
                if !statement.problems.is_empty() {
                    println!("some rows require review");
                }
                for warning in &statement.warnings {
                    match warning {
                        StatementWarning::TotalsMismatch { .. } => println!("totals do not match"),
                        StatementWarning::SectionMismatch { .. } => {
                            println!("section totals do not match")
                        }
                        StatementWarning::TotalsUnreadable => println!("summary unreadable"),
                    }
                }
            }
            Err(_) => println!("statement could not be parsed"),
        }
    }
}
