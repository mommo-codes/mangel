//! Markets.
//!
//! A market is a set of conventions: what gets abbreviated, how units are
//! spelled, how a decimal is written. mangel answers for one market at a time
//! and never picks one on the caller's behalf — a default would be right for
//! Sweden and silently wrong for the next market.
//!
//! This file is also compiled into `build.rs`, which reads [`Market::ALL`] to
//! learn which vocabulary directories must exist. It therefore depends on
//! nothing but `std`, so the list of markets is written down exactly once.

use std::fmt;

/// A market mangel has conventions for.
///
/// Adding one is a code change before it is a vocabulary change. A new market
/// brings its own rules, not only its own words, so a directory under
/// `vocabulary/` is refused at build time until the market exists here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Market {
    /// Sweden.
    Se,
}

impl Market {
    /// Every market. A new variant must be added here as well as to the enum.
    pub const ALL: &'static [Market] = &[Market::Se];

    /// The market's code: its ISO 3166-1 alpha-2 country code, in lowercase.
    /// Also the name of its directory under `vocabulary/`.
    pub const fn code(self) -> &'static str {
        match self {
            Market::Se => "se",
        }
    }

    /// The market with this code.
    ///
    /// Exact match only. `"SE"` and `" se"` are refused rather than guessed
    /// at, so a caller passing the wrong thing finds out at the call site.
    pub fn from_code(code: &str) -> Result<Market, UnknownMarket> {
        Market::ALL
            .iter()
            .copied()
            .find(|market| market.code() == code)
            .ok_or_else(|| UnknownMarket(code.to_owned()))
    }
}

/// A market code mangel has no conventions for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownMarket(String);

impl UnknownMarket {
    /// The code that was asked for.
    pub fn code(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UnknownMarket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown market {:?}; mangel has conventions for:",
            self.0
        )?;
        for market in Market::ALL {
            write!(f, " {:?}", market.code())?;
        }
        Ok(())
    }
}

impl std::error::Error for UnknownMarket {}
