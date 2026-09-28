//! # mangel
//!
//! Irregular product data in, regular product data out. One implementation,
//! callable from Rust, Python and TypeScript.
//!
//! mangel takes a messy field and returns what it means: `3x45m` is a
//! multipack of three 45-metre units, `45l` is 45 litres in the golden
//! standard's spelling, `Laktosfri` is `LF`, `coca cola` is `Coca-Cola`.
//!
//! The first rules are the ones Name Scrubbing's cleaning sheet reads with
//! — see [`rules`]: a size, a cleaned name, a category and its VAT, and a
//! deposit. Each answers for one [`Market`], from the [`Vocabulary`]
//! compiled into the binary from `vocabulary/`.
//!
//! ## Deterministic and pure
//!
//! No network, no database, no credentials, no clock, no randomness, no
//! environment, and nothing read at run time. The same input gives the same
//! output, every time, offline — in a backend worker and in a browser tab.
//! That is the design constraint everything else follows from:
//!
//! - The crate has **no runtime dependencies**, and CI asserts it.
//! - `clippy.toml` refuses the types and functions that would quietly break
//!   the constraint — files, sockets, clocks, environment variables, and
//!   hash maps whose iteration order changes between runs.
//! - The vocabulary is data, but it is **compiled in**, not loaded. See
//!   [`vocabulary`].

#![forbid(unsafe_code)]

mod market;
pub mod rules;
pub mod vocabulary;

pub use market::{Market, UnknownMarket};
pub use vocabulary::Vocabulary;
