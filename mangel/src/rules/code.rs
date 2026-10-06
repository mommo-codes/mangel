//! Decline codes.
//!
//! A sentence is for a person and may be reworded; a code is for a program
//! and never changes. Each code below is listed, with what it means, in
//! `docs/decline-codes.md`, and a test pins every one to its text. A new
//! decline gets a new code. An existing code is never renamed or reused.

use std::fmt;

/// Why a field was declined, as a stable identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Code {
    /// Nothing was given.
    Empty,
    /// A size with no amount.
    NoAmount,
    /// A size with no unit.
    NoUnit,
    /// A size that is more than one size.
    NotOneSize,
    /// A unit mangel does not know.
    UnknownUnit,
    /// A count, a sign and a size: several units in one field.
    Multipack,
    /// An amount that reads as two numbers: `1.000 g`.
    TwoReadings,
    /// A size of 0.
    Zero,
    /// A name whose first letter should be a capital and is not.
    SmallFirstLetter,
    /// A category's group, which holds several categories.
    CategoryIsGroup,
    /// A word in several categories.
    CategoryAmbiguous,
    /// Not a category.
    CategoryUnknown,
    /// A category whose VAT must be typed per product, with none typed.
    VatMustBeTyped,
    /// Not a VAT rate the market has.
    VatUnknown,
    /// A VAT rate that contradicts the category.
    VatMismatch,
    /// Not a deposit the market has.
    DepositUnknown,
    /// mangel has no rule for this field in this profile.
    NoRule,
}

impl Code {
    /// Every code.
    pub const ALL: &'static [Code] = &[
        Code::Empty,
        Code::NoAmount,
        Code::NoUnit,
        Code::NotOneSize,
        Code::UnknownUnit,
        Code::Multipack,
        Code::TwoReadings,
        Code::Zero,
        Code::SmallFirstLetter,
        Code::CategoryIsGroup,
        Code::CategoryAmbiguous,
        Code::CategoryUnknown,
        Code::VatMustBeTyped,
        Code::VatUnknown,
        Code::VatMismatch,
        Code::DepositUnknown,
        Code::NoRule,
    ];

    /// The code as text, `snake_case`: what Python and TypeScript see.
    pub const fn as_str(self) -> &'static str {
        match self {
            Code::Empty => "empty",
            Code::NoAmount => "no_amount",
            Code::NoUnit => "no_unit",
            Code::NotOneSize => "not_one_size",
            Code::UnknownUnit => "unknown_unit",
            Code::Multipack => "multipack",
            Code::TwoReadings => "two_readings",
            Code::Zero => "zero",
            Code::SmallFirstLetter => "small_first_letter",
            Code::CategoryIsGroup => "category_is_group",
            Code::CategoryAmbiguous => "category_ambiguous",
            Code::CategoryUnknown => "category_unknown",
            Code::VatMustBeTyped => "vat_must_be_typed",
            Code::VatUnknown => "vat_unknown",
            Code::VatMismatch => "vat_mismatch",
            Code::DepositUnknown => "deposit_unknown",
            Code::NoRule => "no_rule",
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
