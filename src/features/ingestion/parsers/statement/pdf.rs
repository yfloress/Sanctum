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

//! PDF → [`Document`]: every glyph with its position, regrouped into words
//! and visual lines.
//!
//! The PDF library panics on some malformed files, so extraction runs
//! behind `catch_unwind`: a broken statement is an error, not a crash.
//! Bank PDFs are often protected with the holder's RUT; the password is
//! used for this call only and never stored.

use std::panic::{AssertUnwindSafe, catch_unwind};

use pdf_extract::{LoadOptions, MediaBox, OutputDev, OutputError, Transform};

use super::document::{Document, Line, Word};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PdfError {
    #[error("the file is not a readable PDF")]
    Unreadable,
    #[error("the PDF is protected with a password")]
    PasswordRequired,
    #[error("the PDF password is wrong")]
    WrongPassword,
    #[error("the PDF has no text (it may be a scanned image)")]
    NoText,
}

/// Glyphs closer than this vertically, in points, share a line.
const LINE_TOLERANCE: f32 = 3.0;

pub fn read_pdf(bytes: &[u8], password: Option<&str>) -> Result<Document, PdfError> {
    let options = match password {
        Some(password) => LoadOptions::with_password(password),
        None => LoadOptions::default(),
    };
    let pdf = match pdf_extract::Document::load_mem_with_options(bytes, options) {
        Ok(pdf) => pdf,
        Err(pdf_extract::Error::InvalidPassword) => return Err(PdfError::WrongPassword),
        Err(_) => return Err(PdfError::Unreadable),
    };
    // The loader decrypts when the password (or an empty one) works and
    // leaves the document encrypted otherwise.
    if pdf.is_encrypted() {
        return Err(if password.is_some() {
            PdfError::WrongPassword
        } else {
            PdfError::PasswordRequired
        });
    }

    let mut glyphs = Glyphs::default();
    let extracted = catch_unwind(AssertUnwindSafe(|| {
        pdf_extract::output_doc(&pdf, &mut glyphs)
    }));
    match extracted {
        Ok(Ok(())) => {}
        _ => return Err(PdfError::Unreadable),
    }
    let document = glyphs.into_document();
    if document.lines.is_empty() {
        return Err(PdfError::NoText);
    }
    Ok(document)
}

#[derive(Debug, Clone)]
struct Glyph {
    page: usize,
    x0: f32,
    x1: f32,
    /// Baseline, measured from the top of the page.
    y: f32,
    size: f32,
    text: String,
}

#[derive(Default)]
struct Glyphs {
    page: usize,
    height: f64,
    glyphs: Vec<Glyph>,
}

impl OutputDev for Glyphs {
    fn begin_page(
        &mut self,
        page_num: u32,
        media_box: &MediaBox,
        _: Option<(f64, f64, f64, f64)>,
    ) -> Result<(), OutputError> {
        self.page = page_num.saturating_sub(1) as usize;
        self.height = media_box.ury - media_box.lly;
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), OutputError> {
        Ok(())
    }

    fn output_character(
        &mut self,
        trm: &Transform,
        width: f64,
        _spacing: f64,
        font_size: f64,
        char: &str,
    ) -> Result<(), OutputError> {
        // The text matrix maps glyph space to the page. Its translation is
        // where the glyph starts; PDF y grows upwards, so flip it to sort
        // lines top to bottom. The font size is scaled by the matrix too.
        let scaled_x = font_size * (trm.m11 + trm.m21);
        let scaled_y = font_size * (trm.m12 + trm.m22);
        let size = (scaled_x * scaled_y).abs().sqrt();
        let x0 = trm.m31 as f32;
        self.glyphs.push(Glyph {
            page: self.page,
            x0,
            x1: x0 + (width * size) as f32,
            y: (self.height - trm.m32) as f32,
            size: size as f32,
            text: char.to_string(),
        });
        Ok(())
    }

    fn begin_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }

    fn end_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }

    fn end_line(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
}

impl Glyphs {
    fn into_document(mut self) -> Document {
        self.glyphs.sort_by(|a, b| {
            a.page
                .cmp(&b.page)
                .then(a.y.total_cmp(&b.y))
                .then(a.x0.total_cmp(&b.x0))
        });

        let mut lines: Vec<Vec<Glyph>> = Vec::new();
        for glyph in self.glyphs {
            match lines.last_mut() {
                Some(line)
                    if line[0].page == glyph.page
                        && (glyph.y - line[0].y).abs() <= LINE_TOLERANCE =>
                {
                    line.push(glyph)
                }
                _ => lines.push(vec![glyph]),
            }
        }

        let lines = lines
            .into_iter()
            .filter_map(|mut glyphs| {
                glyphs.sort_by(|a, b| a.x0.total_cmp(&b.x0));
                let words = words(&glyphs);
                (!words.is_empty()).then(|| Line {
                    page: glyphs[0].page,
                    top: glyphs[0].y,
                    words,
                })
            })
            .collect();
        Document { lines }
    }
}

/// Splits a line's glyphs into words at spaces and at gaps wider than a
/// fraction of the font size. A glyph drawn twice in the same place (fake
/// bold) is kept once.
fn words(glyphs: &[Glyph]) -> Vec<Word> {
    let mut words = Vec::new();
    let mut text = String::new();
    let mut span: Option<(f32, f32)> = None;
    let mut last: Option<&Glyph> = None;

    let mut flush = |text: &mut String, span: &mut Option<(f32, f32)>| {
        if let Some((x0, x1)) = span.take()
            && !text.trim().is_empty()
        {
            words.push(Word::at(text.trim(), x0, x1));
        }
        text.clear();
    };

    for glyph in glyphs {
        if let Some(previous) = last
            && previous.text == glyph.text
            && (glyph.x0 - previous.x0).abs() < 0.5
        {
            continue;
        }
        if glyph.text.trim().is_empty() {
            flush(&mut text, &mut span);
            last = Some(glyph);
            continue;
        }
        let gap = span.map(|(_, x1)| glyph.x0 - x1);
        if gap.is_some_and(|gap| gap > glyph.size.max(1.0) * 0.2) {
            flush(&mut text, &mut span);
        }
        text.push_str(&glyph.text);
        span = Some(match span {
            Some((x0, x1)) => (x0, x1.max(glyph.x1)),
            None => (glyph.x0, glyph.x1),
        });
        last = Some(glyph);
    }
    flush(&mut text, &mut span);
    words
}
