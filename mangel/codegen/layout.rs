//! Checks the shape of `vocabulary/`.
//!
//! Compiled into `build.rs`, and into `tests/loader.rs`. It is handed every
//! file under `vocabulary/` as a relative, `/`-separated path, and returns
//! every file that should not be there and every table that is missing.
//!
//! A misspelled file name is the mistake this exists for. Without it, a table
//! saved as `abbreviatons.toml` would be skipped without a word, and every
//! entry in it would be missing from the build while looking present in the
//! repository.
//!
//! Two trees. `vocabulary/<market>/` holds every market table, all of them
//! required. `vocabulary/language/<code>/` holds a language's words, and a
//! language has only the tables someone has written for it: a missing one
//! means "no words in this language yet", which is true, not an error.

use std::collections::BTreeSet;

/// The directory holding one directory per language.
pub const LANGUAGE_DIR: &str = "language";

/// What the layout is checked against.
pub struct Expected<'a> {
    pub markets: &'a [&'a str],
    pub tables: &'a [&'a str],
    pub languages: &'a [&'a str],
    pub language_tables: &'a [&'a str],
}

/// Everything wrong with the layout, one message per problem.
pub fn problems(files: &[String], expected: &Expected) -> Vec<String> {
    let mut problems = Vec::new();
    let mut unknown_markets = BTreeSet::new();
    let mut unknown_languages = BTreeSet::new();

    for path in files {
        let parts: Vec<&str> = path.split('/').collect();

        // Hidden files — .DS_Store, editor swap files — are not ours to judge.
        if parts.iter().any(|part| part.starts_with('.')) {
            continue;
        }

        match parts.as_slice() {
            ["README.md"] => {}
            [_] => problems.push(format!(
                "vocabulary/{path} does not belong here. This directory holds README.md, \
                 one directory per market, and {LANGUAGE_DIR}/."
            )),
            [LANGUAGE_DIR, _] => problems.push(format!(
                "vocabulary/{path} does not belong here. vocabulary/{LANGUAGE_DIR}/ holds one \
                 directory per language."
            )),
            [LANGUAGE_DIR, language, ..] if !expected.languages.contains(language) => {
                unknown_languages.insert(language.to_string());
            }
            [LANGUAGE_DIR, _, file] => {
                if !is_table(file, expected.language_tables) {
                    problems.push(format!(
                        "vocabulary/{path} is not a language table mangel knows. The tables \
                         are: {}. If the name is a typo, rename the file.",
                        tables(expected.language_tables),
                    ));
                }
            }
            [LANGUAGE_DIR, language, ..] => problems.push(format!(
                "vocabulary/{path} is not where a language table goes. Tables go directly in \
                 vocabulary/{LANGUAGE_DIR}/{language}/."
            )),
            [market, ..] if !expected.markets.contains(market) => {
                unknown_markets.insert(market.to_string());
            }
            [_, file] => {
                if !is_table(file, expected.tables) {
                    problems.push(format!(
                        "vocabulary/{path} is not a table mangel knows. The tables are: {}. \
                         If the name is a typo, rename the file.",
                        tables(expected.tables),
                    ));
                }
            }
            [market, ..] => problems.push(format!(
                "vocabulary/{path} is inside a folder. Tables go directly in \
                 vocabulary/{market}/."
            )),
            [] => {}
        }
    }

    for market in unknown_markets {
        problems.push(format!(
            "vocabulary/{market}/ is not a market mangel has conventions for. The markets \
             are: {}. A new market is added in code first — see vocabulary/README.md.",
            listed(expected.markets.iter().map(|market| format!("{market}/"))),
        ));
    }
    for language in unknown_languages {
        problems.push(format!(
            "vocabulary/{LANGUAGE_DIR}/{language}/ is not a language mangel reads. The \
             languages are: {}. A new language is added in code first — see \
             vocabulary/README.md.",
            listed(
                expected
                    .languages
                    .iter()
                    .map(|language| format!("{language}/"))
            ),
        ));
    }

    for market in expected.markets {
        for table in expected.tables {
            let wanted = format!("{market}/{table}.toml");
            if !files.contains(&wanted) {
                problems.push(format!(
                    "vocabulary/{wanted} is missing. Every market has every table; a file \
                     holding only a comment that says why it is empty is fine."
                ));
            }
        }
    }

    problems
}

fn is_table(file: &str, tables: &[&str]) -> bool {
    tables.iter().any(|table| file == format!("{table}.toml"))
}

fn tables(tables: &[&str]) -> String {
    listed(tables.iter().map(|table| format!("{table}.toml")))
}

fn listed(names: impl Iterator<Item = String>) -> String {
    names.collect::<Vec<_>>().join(", ")
}
