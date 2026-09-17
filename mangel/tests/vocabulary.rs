//! What the compiled vocabulary promises, whatever is in it.
//!
//! These hold for any content: they test the pipeline from TOML to static
//! table, not the words. A test that a particular word maps to a particular
//! abbreviation belongs next to the parsing rule that depends on it.

use mangel::{Market, UnknownMarket, Vocabulary};

#[test]
fn every_market_has_a_vocabulary() {
    for market in Market::ALL {
        // Reaching the table at all is the assertion: `of` is total.
        let _ = Vocabulary::of(*market);
    }
}

#[test]
fn abbreviations_are_sorted_by_word_with_no_word_twice() {
    for market in Market::ALL {
        let table = Vocabulary::of(*market).abbreviations;
        for pair in table.windows(2) {
            assert!(
                pair[0].0 < pair[1].0,
                "{}: {:?} is not strictly before {:?}",
                market.code(),
                pair[0].0,
                pair[1].0,
            );
        }
    }
}

#[test]
fn no_entry_is_blank_or_padded() {
    for market in Market::ALL {
        for (word, abbreviation) in Vocabulary::of(*market).abbreviations {
            for text in [word, abbreviation] {
                assert!(
                    !text.is_empty(),
                    "{}: empty text in {word:?}",
                    market.code()
                );
                assert_eq!(
                    text.trim(),
                    *text,
                    "{}: padded text in {word:?}",
                    market.code()
                );
            }
        }
    }
}

#[test]
fn the_toml_reaches_the_binary() {
    // One known entry, so the tests above cannot pass on a pipeline that
    // silently compiles every table to nothing.
    assert!(Vocabulary::of(Market::Se)
        .abbreviations
        .contains(&("Laktosfri", "LF")));
}

#[test]
fn market_codes_round_trip() {
    for market in Market::ALL {
        assert_eq!(Market::from_code(market.code()), Ok(*market));
    }
}

#[test]
fn market_codes_are_exact() {
    for code in ["SE", "Se", " se", "se ", "", "sweden", "no"] {
        let refused = Market::from_code(code);
        assert_eq!(refused.as_ref().map_err(UnknownMarket::code), Err(code));
    }
}

#[test]
fn the_refusal_names_what_would_have_worked() {
    let message = Market::from_code("SE").unwrap_err().to_string();
    assert_eq!(
        message,
        r#"unknown market "SE"; mangel has conventions for: "se""#
    );
}
