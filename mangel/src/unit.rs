//! Units of a size, without a language.
//!
//! A unit is what a size measures in, not how anyone spells it. How a label
//! spells it is a language's vocabulary (`gram`, `styck`); how the register
//! writes it is a market's (`L`, `st`). Both map to this.
//!
//! This file is also compiled into `build.rs`, which checks every unit named
//! in the vocabulary against [`Unit::ALL`], so it depends on nothing but
//! `std`.

/// A unit of a size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Unit {
    /// g
    Gram,
    /// kg
    Kilogram,
    /// ml
    Millilitre,
    /// cl
    Centilitre,
    /// dl
    Decilitre,
    /// l
    Litre,
    /// A count of pieces.
    Piece,
}

impl Unit {
    /// Every unit.
    pub const ALL: &'static [Unit] = &[
        Unit::Gram,
        Unit::Kilogram,
        Unit::Millilitre,
        Unit::Centilitre,
        Unit::Decilitre,
        Unit::Litre,
        Unit::Piece,
    ];

    /// The unit's code: its SI symbol in lowercase, or `piece`. The
    /// vocabulary names units by these.
    pub const fn code(self) -> &'static str {
        match self {
            Unit::Gram => "g",
            Unit::Kilogram => "kg",
            Unit::Millilitre => "ml",
            Unit::Centilitre => "cl",
            Unit::Decilitre => "dl",
            Unit::Litre => "l",
            Unit::Piece => "piece",
        }
    }

    /// The unit with this code.
    pub fn from_code(code: &str) -> Option<Unit> {
        Unit::ALL.iter().copied().find(|unit| unit.code() == code)
    }

    /// The unit written as a symbol, in any language and any case: `g`,
    /// `KG`, `Ml`, `L`. A piece has no symbol.
    pub fn from_symbol(written: &str) -> Option<Unit> {
        let lowered = written.to_lowercase();
        Unit::ALL
            .iter()
            .copied()
            .filter(|unit| *unit != Unit::Piece)
            .find(|unit| unit.code() == lowered)
    }
}
