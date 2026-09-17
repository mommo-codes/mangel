//! Reads one vocabulary table.
//!
//! Compiled into `build.rs`, and into `tests/loader.rs` so that every check
//! here is tested. It knows nothing about files or markets: it is handed the
//! text of one table and returns the entries in it, or everything wrong with
//! it. Everything, not the first thing, so one build shows all there is to fix.
//!
//! The messages are read by people who edit the TOML and do not write Rust.
//! Each one says what is wrong and what to do about it.

use toml::de::{DeTable, DeValue};

/// A word as written in product data, and what the table maps it to.
pub type Entry = (String, String);

/// Something wrong with a table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The 1-based line it is on, when it is on one.
    pub line: Option<usize>,
    /// What is wrong, and what to do about it.
    pub message: String,
}

/// The entries in one table, sorted by word in byte order.
pub fn read(source: &str) -> Result<Vec<Entry>, Vec<Problem>> {
    // A byte-order mark is invisible, means nothing in UTF-8, and some Windows
    // editors add one on save. It sits on line 1, so line numbers are unmoved.
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);

    let document = match DeTable::parse(source) {
        Ok(document) => document.into_inner(),
        Err(error) => return Err(vec![syntax_problem(source, &error)]),
    };

    let mut entries = Vec::new();
    let mut problems = Vec::new();
    for (key, value) in document.iter() {
        let word: &str = key.get_ref();
        let line = Some(line_of(source, key.span().start));
        let reported = problems.len();

        check_text(
            word,
            &format!("The word {}", shown(word)),
            line,
            &mut problems,
        );
        match value.get_ref() {
            DeValue::String(replacement) => {
                let subject = format!("The abbreviation for {}", shown(word));
                check_text(replacement, &subject, line, &mut problems);
                if problems.len() == reported {
                    entries.push((word.to_owned(), replacement.to_string()));
                }
            }
            DeValue::Table(_) => problems.push(Problem {
                line,
                message: format!(
                    "{} is a section — a [header] line, a {{ … }} value, or a dot in a \
                     word without quotes — but this file has no sections. Every entry is \
                     one line at the top level: \"Word\" = \"ABBR\".",
                    shown(word),
                ),
            }),
            _ => problems.push(Problem {
                line,
                message: format!(
                    "The abbreviation for {} is `{}`, which is not text. Put it in double \
                     quotes.",
                    shown(word),
                    source[value.span()].trim(),
                ),
            }),
        }
    }

    if problems.is_empty() {
        entries.sort();
        Ok(entries)
    } else {
        // The parser hands entries back in key order; a person reads top down.
        problems.sort_by_key(|problem| problem.line);
        Err(problems)
    }
}

fn syntax_problem(source: &str, error: &toml::de::Error) -> Problem {
    let span = error.span();
    let line = span.as_ref().map(|span| line_of(source, span.start));
    let reason = error.message().trim_end_matches(['\n', '.']);
    let message = if reason.contains("duplicate key") {
        let written = span.map_or("", |span| source.get(span).unwrap_or("").trim());
        format!(
            "{written} is listed twice. A word can have only one abbreviation: keep one of \
             the lines and delete the other."
        )
    } else {
        format!(
            "This line is not valid TOML ({reason}). Every entry is one line with both \
             sides in double quotes: \"Word\" = \"ABBR\"."
        )
    };
    Problem { line, message }
}

/// Checks that apply to both sides of an entry.
fn check_text(text: &str, subject: &str, line: Option<usize>, problems: &mut Vec<Problem>) {
    let mut report = |message: String| problems.push(Problem { line, message });

    if text.is_empty() {
        report(format!(
            "{subject} is empty. Fill it in, or delete the line."
        ));
        return;
    }

    // Tabs and line breaks are whitespace too, but they are reported below as
    // control characters, so they are not reported twice.
    let visible_space = |c: &char| c.is_whitespace() && !c.is_control();
    if let Some(c) = text.chars().next().filter(visible_space) {
        report(format!(
            "{subject} starts with {}. Delete it: a space inside the quotes is part of \
             the spelling.",
            space_name(c),
        ));
    }
    if let Some(c) = text.chars().next_back().filter(visible_space) {
        report(format!(
            "{subject} ends with {}. Delete it: a space inside the quotes is part of the \
             spelling.",
            space_name(c),
        ));
    }

    if let Some(c) = text.chars().find(|c| c.is_control()) {
        report(format!(
            "{subject} contains an invisible control character ({}). Retype the entry.",
            code_point(c),
        ));
    }

    // Combining Diacritical Marks. An "ä" pasted from some PDFs arrives as "a"
    // followed by a separate U+0308. It renders identically and never matches
    // the single character "ä" that product data actually contains.
    if let Some(c) = text.chars().find(|c| ('\u{0300}'..='\u{036f}').contains(c)) {
        report(format!(
            "{subject} has an accent stored as a separate character ({}) after its \
             letter. It looks right but will never match the real letter. Delete the \
             letter and type it again.",
            code_point(c),
        ));
    }
}

/// `text` in double quotes, with anything invisible spelled out.
fn shown(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        let invisible = c.is_control()
            || (c.is_whitespace() && c != ' ')
            || ('\u{0300}'..='\u{036f}').contains(&c);
        if invisible {
            out.push_str(&format!("<{}>", code_point(c)));
        } else {
            out.push(c);
        }
    }
    out.push('"');
    out
}

fn space_name(c: char) -> String {
    match c {
        ' ' => "a space".to_owned(),
        '\u{00a0}' => "a non-breaking space (U+00A0)".to_owned(),
        other => format!("an unusual space character ({})", code_point(other)),
    }
}

fn code_point(c: char) -> String {
    format!("U+{:04X}", u32::from(c))
}

fn line_of(source: &str, offset: usize) -> usize {
    let end = offset.min(source.len());
    source.as_bytes()[..end]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}
