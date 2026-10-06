//! The rules: one messy field in, what it means out — or a refusal.
//!
//! Every rule answers `Ok` with the value in the golden standard's form, or
//! [`Declined`] with a sentence a person can act on and a [`Code`] a program
//! can act on. **Nothing is guessed**: a field that could mean two things is
//! declined with both, never picked. The sentences name the field's content,
//! not the field, so a caller can put them under any label.

mod category;
mod code;
mod deposit;
mod name;
mod size;

use std::fmt;

pub use category::{categories, category, vat, Category, CategoryVat};
pub use code::Code;
pub use deposit::deposit;
pub use name::cleaned_name;
pub(crate) use size::reading;
pub use size::{size, Size};

/// A field a rule would not read, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declined {
    code: Code,
    reason: String,
}

impl Declined {
    pub(crate) fn new(code: Code, reason: impl Into<String>) -> Declined {
        Declined {
            code,
            reason: reason.into(),
        }
    }

    /// Why, as a code that never changes: a program can branch on it.
    pub fn code(&self) -> Code {
        self.code
    }

    /// Why, as a sentence a person can act on. The wording may improve
    /// between versions; the [`code`](Declined::code) does not.
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for Declined {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.reason)
    }
}

impl std::error::Error for Declined {}

/// A list for a sentence: `a, b and c`.
pub(crate) fn listed(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}
