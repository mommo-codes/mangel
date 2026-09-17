//! WebAssembly bindings for mangel.
//!
//! The reason this target exists: Name Scrubbing normalises while someone
//! types, in the browser, and the backend normalises OCR output. Compiling the
//! same source for both is what stops the browser growing its own copy of the
//! rules — and, because the vocabulary is compiled in as well, its own copy of
//! the word lists.
//!
//! Everything here is a thin shell. No rule is decided in this file.

use mangel::{Market, Vocabulary};
use wasm_bindgen::prelude::*;

/// A market's abbreviations, as a new `Map`: each word as it is written in
/// product data, mapped to the abbreviation the golden standard uses. Entries
/// are in sorted order.
///
/// Throws for a market code mangel has no conventions for.
#[wasm_bindgen]
pub fn abbreviations(market: &str) -> Result<js_sys::Map, JsError> {
    let market = Market::from_code(market)?;
    let table = js_sys::Map::new();
    for (word, abbreviation) in Vocabulary::of(market).abbreviations {
        table.set(&JsValue::from_str(word), &JsValue::from_str(abbreviation));
    }
    Ok(table)
}
