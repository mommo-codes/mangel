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

use std::collections::BTreeSet;

/// Everything wrong with the layout, one message per problem.
pub fn problems(files: &[String], markets: &[&str], tables: &[&str]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut unknown_markets = BTreeSet::new();

    for path in files {
        let parts: Vec<&str> = path.split('/').collect();

        // Hidden files — .DS_Store, editor swap files — are not ours to judge.
        if parts.iter().any(|part| part.starts_with('.')) {
            continue;
        }

        match parts.as_slice() {
            ["README.md"] => {}
            [_] => problems.push(format!(
                "vocabulary/{path} does not belong here. This directory holds README.md \
                 and one directory per market."
            )),
            [market, ..] if !markets.contains(market) => {
                unknown_markets.insert(market.to_string());
            }
            [_, file] => {
                if !tables.iter().any(|table| *file == format!("{table}.toml")) {
                    problems.push(format!(
                        "vocabulary/{path} is not a table mangel knows. The tables are: {}. \
                         If the name is a typo, rename the file.",
                        listed(tables.iter().map(|table| format!("{table}.toml"))),
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
            listed(markets.iter().map(|market| format!("{market}/"))),
        ));
    }

    for market in markets {
        for table in tables {
            let expected = format!("{market}/{table}.toml");
            if !files.contains(&expected) {
                problems.push(format!(
                    "vocabulary/{expected} is missing. Every market has every table; a file \
                     holding only a comment that says why it is empty is fine."
                ));
            }
        }
    }

    problems
}

fn listed(names: impl Iterator<Item = String>) -> String {
    names.collect::<Vec<_>>().join(", ")
}
