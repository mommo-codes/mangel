//! The vocabulary: words, compiled in.
//!
//! Everything here comes from `vocabulary/<market>/<table>.toml` and
//! `vocabulary/language/<code>/<table>.toml`, which `build.rs` reads, checks
//! and turns into static tables at compile time. A market's tables say how
//! its register writes things; a language's say how a label in it is read.
//! Nothing is loaded at run time, so there is no file to be missing and no
//! parse to fail in production. A malformed entry fails the build instead.
//!
//! The tables hold exact spellings and nothing else. How a messy field is
//! matched against them is a parsing rule, and parsing rules live in code.
//! See `docs/vocabulary.md` for where the line between the two is drawn.

use crate::{Language, Market, Unit};

/// One market's vocabulary.
#[derive(Debug)]
#[non_exhaustive]
pub struct Vocabulary {
    /// Each word as it is written in product data, with the abbreviation the
    /// golden standard uses for it. Sorted by word in byte order, and no word
    /// appears twice.
    pub abbreviations: &'static [(&'static str, &'static str)],
    /// Each unit with the golden standard's spelling of it, the one the
    /// register accepts: a litre is `L` in Sweden. Every unit, once, sorted
    /// by its code. How a label spells a unit is a language's, in
    /// [`LanguageVocabulary::units`].
    pub units: &'static [(Unit, &'static str)],
    /// Each category with the group it sits in. A category name appears
    /// once in the whole list. Sorted by category.
    pub categories: &'static [(&'static str, &'static str)],
    /// Each category's VAT rate in percent, as text, or `"manual"` for a
    /// category whose products carry more than one rate. Names exactly the
    /// categories in [`Vocabulary::categories`]. Sorted by category.
    pub category_vat: &'static [(&'static str, &'static str)],
}

impl Vocabulary {
    /// The vocabulary for `market`.
    ///
    /// ```
    /// use mangel::{Market, Vocabulary};
    ///
    /// let swedish = Vocabulary::of(Market::Se);
    /// assert!(swedish.abbreviations.contains(&("Laktosfri", "LF")));
    /// ```
    pub fn of(market: Market) -> &'static Vocabulary {
        compiled::of(market)
    }
}

impl Vocabulary {
    /// How the market spells `unit`. Every unit has a spelling in every
    /// market; `build.rs` refuses a table that leaves one out.
    pub(crate) fn spelling(&self, unit: Unit) -> &'static str {
        self.units
            .iter()
            .find(|(listed, _)| *listed == unit)
            .map_or("", |(_, spelling)| spelling)
    }
}

/// One language's vocabulary: how a label printed in it is read.
///
/// A language has only the tables someone has written for it. One without
/// a table has an empty slice here, which means no words yet, not an error.
#[derive(Debug)]
#[non_exhaustive]
pub struct LanguageVocabulary {
    /// Each word for a unit, lowercase, with the unit it means: `gram` is a
    /// gram, `styck` a piece. Symbols (`g`, `ml`) are read in every language
    /// and are not listed. Sorted by word, no word twice.
    pub units: &'static [(&'static str, Unit)],
}

impl LanguageVocabulary {
    /// The vocabulary for `language`.
    ///
    /// ```
    /// use mangel::{Language, LanguageVocabulary, Unit};
    ///
    /// let swedish = LanguageVocabulary::of(Language::Sv);
    /// assert!(swedish.units.contains(&("styck", Unit::Piece)));
    /// ```
    pub fn of(language: Language) -> &'static LanguageVocabulary {
        compiled::of_language(language)
    }

    /// The unit `word` means in this language, matched without regard to
    /// case.
    pub(crate) fn unit(&self, word: &str) -> Option<Unit> {
        let lowered = word.to_lowercase();
        self.units
            .iter()
            .find(|(listed, _)| *listed == lowered)
            .map(|(_, unit)| *unit)
    }
}

/// Written by `build.rs` into `OUT_DIR`. To change what is in it, edit the
/// TOML under `vocabulary/`, never the generated file.
mod compiled {
    use super::{LanguageVocabulary, Vocabulary};
    use crate::{Language, Market, Unit};

    include!(concat!(env!("OUT_DIR"), "/vocabulary.rs"));
}
