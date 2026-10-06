//! Checks the two kinds of unit table.
//!
//! Compiled into `build.rs`, and into `tests/loader.rs`. A unit table that
//! reads as valid TOML can still be wrong in a way only a person would
//! notice: a unit mangel has no such thing as, a unit spelled twice for one
//! market, or one not spelled at all. These say which, by word.

use std::collections::BTreeSet;

/// A market's `units.toml`: each unit's golden spelling. Every unit must be
/// there exactly once, named by its code.
pub fn market_problems(entries: &[(String, String)], codes: &[&str]) -> Vec<String> {
    let mut problems = Vec::new();
    for (unit, _) in entries {
        if !codes.contains(&unit.as_str()) {
            problems.push(format!(
                "\"{unit}\" is not a unit mangel knows. Units are named by their code: {}.",
                codes.join(", "),
            ));
        }
    }
    let named: BTreeSet<&str> = entries.iter().map(|(unit, _)| unit.as_str()).collect();
    for code in codes {
        if !named.contains(code) {
            problems.push(format!(
                "\"{code}\" has no spelling. Every unit is spelled in every market: add \
                 \"{code}\" = \"…\"."
            ));
        }
    }
    problems
}

/// A language's `units.toml`: words for units. Each word is lowercase and
/// names a unit by its code. A symbol is read in every language, so it is
/// not a word of any one of them.
pub fn language_problems(
    entries: &[(String, String)],
    codes: &[&str],
    symbols: &[&str],
) -> Vec<String> {
    let mut problems = Vec::new();
    for (word, unit) in entries {
        if *word != word.to_lowercase() {
            problems.push(format!(
                "\"{word}\" has capitals. Words are matched without regard to case, so write \
                 it in lowercase: \"{}\".",
                word.to_lowercase(),
            ));
        }
        if symbols.contains(&word.to_lowercase().as_str()) {
            problems.push(format!(
                "\"{word}\" is a symbol, read in every language already. Delete the line."
            ));
        }
        if !codes.contains(&unit.as_str()) {
            problems.push(format!(
                "\"{word}\" names \"{unit}\", which is not a unit mangel knows. Units are \
                 named by their code: {}.",
                codes.join(", "),
            ));
        }
    }
    problems
}
