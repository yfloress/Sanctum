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

//! Text of a statement as visual lines of words.
//!
//! The PDF extractor fills in each word's horizontal span. Statements whose
//! columns are only distinguishable by position (one amount per row, placed
//! under either "cargos" or "abonos") need it; plain text still parses when
//! the row says what it is.

/// One word and, when the source had layout, where it sits on the page.
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    /// Left and right edge in PDF points.
    pub span: Option<(f32, f32)>,
}

impl Word {
    pub fn plain(text: &str) -> Self {
        Self {
            text: text.to_string(),
            span: None,
        }
    }

    pub fn at(text: &str, x0: f32, x1: f32) -> Self {
        Self {
            text: text.to_string(),
            span: Some((x0, x1)),
        }
    }

    pub fn center(&self) -> Option<f32> {
        self.span.map(|(x0, x1)| (x0 + x1) / 2.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub page: usize,
    /// Distance from the top of the page, used to group header rows.
    pub top: f32,
    pub words: Vec<Word>,
}

impl Line {
    pub fn plain(page: usize, top: f32, text: &str) -> Self {
        Self {
            page,
            top,
            words: text.split_whitespace().map(Word::plain).collect(),
        }
    }

    pub fn text(&self) -> String {
        let words: Vec<&str> = self.words.iter().map(|w| w.text.as_str()).collect();
        words.join(" ")
    }

    pub fn is_positioned(&self) -> bool {
        !self.words.is_empty() && self.words.iter().all(|w| w.span.is_some())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    pub lines: Vec<Line>,
}

impl Document {
    /// Builds a document without layout, one string per page.
    pub fn from_pages(pages: &[&str]) -> Self {
        let mut lines = Vec::new();
        for (page, text) in pages.iter().enumerate() {
            for (row, raw) in text.lines().enumerate() {
                if !raw.trim().is_empty() {
                    lines.push(Line::plain(page, row as f32 * 10.0, raw));
                }
            }
        }
        Self { lines }
    }

    /// Every word, folded, in reading order.
    pub fn folded_tokens(&self) -> Vec<String> {
        self.lines
            .iter()
            .flat_map(|line| line.words.iter().map(|w| fold(&w.text)))
            .collect()
    }

    pub fn folded_text(&self) -> String {
        let lines: Vec<String> = self.lines.iter().map(|l| fold(&l.text())).collect();
        lines.join("\n")
    }
}

/// Uppercases and drops Spanish diacritics, so "Descripción" matches
/// "DESCRIPCION" whatever the PDF's font encoding produced.
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_uppercase)
        .map(|c| match c {
            'Á' | 'À' | 'Ä' | 'Â' => 'A',
            'É' | 'È' | 'Ë' | 'Ê' => 'E',
            'Í' | 'Ì' | 'Ï' | 'Î' => 'I',
            'Ó' | 'Ò' | 'Ö' | 'Ô' => 'O',
            'Ú' | 'Ù' | 'Ü' | 'Û' => 'U',
            'Ñ' => 'N',
            other => other,
        })
        .collect()
}
