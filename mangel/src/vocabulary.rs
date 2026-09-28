//! The vocabulary: words, compiled in.
//!
//! Everything here comes from `vocabulary/<market>/<table>.toml`, which
//! `build.rs` reads, checks and turns into static tables at compile time.
//! Nothing is loaded at run time, so there is no file to be missing and no
//! parse to fail in production. A malformed entry fails the build instead.
//!
//! The tables hold exact spellings and nothing else. How a messy field is
//! matched against them is a parsing rule, and parsing rules live in code.
//! See `docs/vocabulary.md` for where the line between the two is drawn.

use crate::Market;

/// One market's vocabulary.
#[derive(Debug)]
#[non_exhaustive]
pub struct Vocabulary {
    /// Each word as it is written in product data, with the abbreviation the
    /// golden standard uses for it. Sorted by word in byte order, and no word
    /// appears twice.
    pub abbreviations: &'static [(&'static str, &'static str)],
    /// Each size unit as it is written, lowercase, with the golden
    /// standard's spelling of it. Sorted, no unit twice.
    pub units: &'static [(&'static str, &'static str)],
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

/// Written by `build.rs` into `OUT_DIR`. To change what is in it, edit the
/// TOML under `vocabulary/`, never the generated file.
mod compiled {
    use super::Vocabulary;
    use crate::Market;

    include!(concat!(env!("OUT_DIR"), "/vocabulary.rs"));
}
