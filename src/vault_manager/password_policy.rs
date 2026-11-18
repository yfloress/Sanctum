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

//! Master password guidance, after NIST SP 800-63B.
//!
//! The master password is the only factor protecting the vault, so the
//! guidance is the one NIST gives for a single-factor memorized secret: judge
//! length, reject what is common or predictable, and impose no composition
//! rules (forcing a digit and a symbol yields `Password1!`, not strength).
//!
//! This only advises. The user can still choose a weak password after an
//! explicit second confirmation, which is what a throwaway test vault needs.

use std::collections::HashSet;
use std::sync::OnceLock;

/// NIST SP 800-63B rev. 4 minimum for a password used on its own.
pub const RECOMMENDED_LENGTH: usize = 15;

/// The 10,000 most common passwords (SecLists, MIT), lowercased.
const COMMON_PASSWORDS: &str = include_str!("common_passwords.txt");

/// Shown in the order checked; the first match is the one reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordWeakness {
    /// On the list of most used passwords, or one of them with a few digits or
    /// symbols added around it.
    Common,
    /// A repeated character, a run like `1234` or `abcd`, or a keyboard row.
    Predictable,
    /// Contains the application's name.
    AppName,
    /// Shorter than [`RECOMMENDED_LENGTH`] characters.
    Short,
}

impl PasswordWeakness {
    /// Locale key of the message explaining this weakness.
    pub fn message_key(self) -> &'static str {
        match self {
            Self::Common => "password-weak-common",
            Self::Predictable => "password-weak-predictable",
            Self::AppName => "password-weak-app-name",
            Self::Short => "password-weak-short",
        }
    }
}

fn common_passwords() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| COMMON_PASSWORDS.lines().filter(|l| !l.is_empty()).collect())
}

/// Assesses a password the way it will be stored (trimmed). `None` means no
/// advice to give.
pub fn assess(password: &str) -> Option<PasswordWeakness> {
    let password = password.trim();
    if password.is_empty() {
        return None;
    }
    let lower = password.to_lowercase();

    if is_common(&lower) {
        return Some(PasswordWeakness::Common);
    }
    if is_predictable(&lower) {
        return Some(PasswordWeakness::Predictable);
    }
    if lower.contains("sanctum") {
        return Some(PasswordWeakness::AppName);
    }
    // Characters, not bytes: "contraseña" is ten long, not eleven.
    if password.chars().count() < RECOMMENDED_LENGTH {
        return Some(PasswordWeakness::Short);
    }
    None
}

/// The password itself is common, or is a common one wrapped in digits and
/// symbols ("password2024!", "!qwerty1").
fn is_common(lower: &str) -> bool {
    let list = common_passwords();
    if list.contains(lower) {
        return true;
    }
    let core = lower.trim_matches(|c: char| !c.is_alphabetic());
    core.chars().count() >= 4 && list.contains(core)
}

const KEYBOARD_ROWS: &[&str] = &[
    "1234567890",
    "qwertyuiop",
    "asdfghjkl",
    "zxcvbnm",
    "qwertzuiop",
    "azertyuiop",
];

/// The whole password is one character repeated, a run of consecutive
/// characters in either direction, or a stretch of a keyboard row.
fn is_predictable(lower: &str) -> bool {
    let chars: Vec<char> = lower.chars().collect();
    if chars.len() < 4 {
        return chars.windows(2).all(|w| w[0] == w[1]);
    }
    if chars.windows(2).all(|w| w[0] == w[1]) {
        return true;
    }
    let step = |w: &[char]| w[1] as i64 - w[0] as i64;
    let first = step(&chars[..2]);
    if first.abs() == 1 && chars.windows(2).all(|w| step(w) == first) {
        return true;
    }
    KEYBOARD_ROWS.iter().any(|row| {
        let reversed: String = row.chars().rev().collect();
        row.contains(lower) || reversed.contains(lower)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_passwords_are_caught_even_when_dressed_up() {
        assert_eq!(assess("password"), Some(PasswordWeakness::Common));
        assert_eq!(assess("1234"), Some(PasswordWeakness::Common));
        assert_eq!(assess("Password1!"), Some(PasswordWeakness::Common));
        assert_eq!(assess("!!dragon2024"), Some(PasswordWeakness::Common));
    }

    #[test]
    fn predictable_runs_and_rows() {
        assert_eq!(
            assess("aaaaaaaaaaaaaaaa"),
            Some(PasswordWeakness::Predictable)
        );
        assert_eq!(assess("zyxwvutsrqpo"), Some(PasswordWeakness::Predictable));
        assert_eq!(assess("nbvcxz"), Some(PasswordWeakness::Predictable));
        assert_eq!(
            assess("abcdefghijklmnop"),
            Some(PasswordWeakness::Predictable)
        );
    }

    #[test]
    fn app_name_and_length() {
        assert_eq!(
            assess("my sanctum vault key"),
            Some(PasswordWeakness::AppName)
        );
        assert_eq!(assess("Tr0ub4dor&3"), Some(PasswordWeakness::Short));
        // Counted in characters: fourteen of them is still short.
        assert_eq!(assess("ñandú en mayo!"), Some(PasswordWeakness::Short));
    }

    #[test]
    fn long_passphrases_need_no_symbols() {
        assert_eq!(assess("correct horse battery staple"), None);
        assert_eq!(assess("el gato duerme sobre la radio"), None);
        assert_eq!(assess("   "), None);
    }
}
