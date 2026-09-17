//! The build script's checks, run directly.
//!
//! `build.rs` cannot be unit-tested in place, so its two checkers are compiled
//! in here as well. Every message below is one someone editing a TOML file
//! will read without knowing Rust, so the tests pin the useful part of each:
//! the line, and what to do.

#[path = "../codegen/layout.rs"]
mod layout;
#[path = "../codegen/table.rs"]
mod table;

use table::Problem;

fn problems(source: &str) -> Vec<Problem> {
    table::read(source).expect_err("expected the table to be refused")
}

fn only_problem(source: &str) -> Problem {
    let mut found = problems(source);
    assert_eq!(found.len(), 1, "expected one problem, got {found:#?}");
    found.remove(0)
}

mod table_reading {
    use super::*;

    #[test]
    fn entries_come_back_sorted_by_word() {
        let source = "\"Laktosfri\" = \"LF\"\n\"Ekologisk\" = \"EKO\"\n";
        assert_eq!(
            table::read(source),
            Ok(vec![
                ("Ekologisk".to_owned(), "EKO".to_owned()),
                ("Laktosfri".to_owned(), "LF".to_owned()),
            ])
        );
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let source = "# a note\n\n\"Laktosfri\" = \"LF\"  # why\n";
        assert_eq!(table::read(source).unwrap().len(), 1);
    }

    #[test]
    fn a_table_holding_only_a_comment_is_empty_not_broken() {
        assert_eq!(
            table::read("# nothing is abbreviated here, because…\n"),
            Ok(vec![])
        );
    }

    #[test]
    fn swedish_letters_survive() {
        let source = "\"Sötningsmedel\" = \"SÖTN\"\n";
        assert_eq!(
            table::read(source).unwrap(),
            vec![("Sötningsmedel".to_owned(), "SÖTN".to_owned())]
        );
    }

    #[test]
    fn a_word_may_map_to_itself() {
        assert!(table::read("\"Sockerfri\" = \"Sockerfri\"\n").is_ok());
    }

    #[test]
    fn a_byte_order_mark_is_ignored() {
        assert!(table::read("\u{feff}\"Laktosfri\" = \"LF\"\n").is_ok());
    }
}

mod table_refusals {
    use super::*;

    #[test]
    fn a_word_listed_twice() {
        let problem = only_problem("\"Laktosfri\" = \"LF\"\n\n\"Laktosfri\" = \"LAKT\"\n");
        assert_eq!(problem.line, Some(3));
        assert!(
            problem.message.contains("\"Laktosfri\" is listed twice"),
            "{problem:?}"
        );
    }

    #[test]
    fn a_missing_quote() {
        let problem = only_problem("\"Laktosfri\" = \"LF\"\n\"Fettfri\" = FF\n");
        assert_eq!(problem.line, Some(2));
        assert!(
            problem.message.contains("both sides in double quotes"),
            "{problem:?}"
        );
    }

    #[test]
    fn an_empty_word_or_abbreviation() {
        assert!(only_problem("\"\" = \"LF\"\n").message.contains("is empty"));
        assert!(only_problem("\"Laktosfri\" = \"\"\n")
            .message
            .contains("is empty"));
    }

    #[test]
    fn padding_on_either_side_of_either_text() {
        let found = problems("\" Laktosfri\" = \"LF \"\n");
        assert_eq!(found.len(), 2, "{found:#?}");
        assert!(found[0].message.contains("starts with a space"));
        assert!(found[1].message.contains("ends with a space"));
    }

    #[test]
    fn a_non_breaking_space_is_named() {
        let problem = only_problem("\"Laktosfri\u{a0}\" = \"LF\"\n");
        assert!(
            problem.message.contains("non-breaking space (U+00A0)"),
            "{problem:?}"
        );
        assert!(
            problem.message.contains("\"Laktosfri<U+00A0>\""),
            "{problem:?}"
        );
    }

    #[test]
    fn a_control_character_is_reported_once() {
        let problem = only_problem("\"Laktosfri\\t\" = \"LF\"\n");
        assert!(
            problem.message.contains("control character (U+0009)"),
            "{problem:?}"
        );
    }

    #[test]
    fn a_decomposed_letter() {
        // "Mjölk" with the ö stored as o + U+0308.
        let problem = only_problem("\"Mjo\u{308}lk\" = \"MJ\"\n");
        assert!(problem.message.contains("(U+0308)"), "{problem:?}");
        assert!(problem.message.contains("type it again"), "{problem:?}");
    }

    #[test]
    fn a_value_that_is_not_text() {
        let problem = only_problem("\"Laktosfri\" = 1\n");
        assert!(
            problem.message.contains("`1`, which is not text"),
            "{problem:?}"
        );
    }

    #[test]
    fn a_section_header() {
        let problem = only_problem("[mejeri]\n\"Laktosfri\" = \"LF\"\n");
        assert_eq!(problem.line, Some(1));
        assert!(
            problem.message.contains("this file has no sections"),
            "{problem:?}"
        );
    }

    #[test]
    fn every_problem_is_reported_in_line_order() {
        let found = problems("\"Laktosfri \" = \"LF\"\n\"Fettfri\" = \"\"\n\"Mager\" = 2\n");
        let lines: Vec<_> = found.iter().map(|problem| problem.line).collect();
        assert_eq!(lines, vec![Some(1), Some(2), Some(3)], "{found:#?}");
    }
}

mod layout_checks {
    use super::layout;

    const MARKETS: &[&str] = &["se"];
    const TABLES: &[&str] = &["abbreviations"];

    fn check(files: &[&str]) -> Vec<String> {
        let files: Vec<String> = files.iter().map(|file| file.to_string()).collect();
        layout::problems(&files, MARKETS, TABLES)
    }

    #[test]
    fn the_expected_layout_is_clean() {
        assert!(check(&["README.md", "se/abbreviations.toml"]).is_empty());
    }

    #[test]
    fn hidden_files_are_skipped() {
        assert!(check(&[".DS_Store", "se/.DS_Store", "se/abbreviations.toml"]).is_empty());
    }

    #[test]
    fn a_misspelled_table() {
        let found = check(&["se/abbreviations.toml", "se/abbreviatons.toml"]);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert!(found[0].starts_with("vocabulary/se/abbreviatons.toml is not a table"));
    }

    #[test]
    fn a_missing_table() {
        let found = check(&["README.md"]);
        assert_eq!(
            found,
            vec![
                "vocabulary/se/abbreviations.toml is missing. Every market has every table; a \
             file holding only a comment that says why it is empty is fine."
            ]
        );
    }

    #[test]
    fn a_market_that_does_not_exist_in_code() {
        let found = check(&[
            "se/abbreviations.toml",
            "no/abbreviations.toml",
            "no/units.toml",
        ]);
        assert_eq!(
            found.len(),
            1,
            "one message per market, not per file: {found:#?}"
        );
        assert!(found[0].starts_with("vocabulary/no/ is not a market"));
    }

    #[test]
    fn a_stray_file_at_the_root() {
        let found = check(&["notes.txt", "se/abbreviations.toml"]);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert!(found[0].starts_with("vocabulary/notes.txt does not belong here"));
    }

    #[test]
    fn a_table_in_a_subfolder() {
        let found = check(&["se/abbreviations.toml", "se/old/abbreviations.toml"]);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert!(found[0].contains("is inside a folder"));
    }
}
