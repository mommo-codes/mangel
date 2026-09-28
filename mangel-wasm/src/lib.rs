//! WebAssembly bindings for mangel.
//!
//! The reason this target exists: Name Scrubbing normalises while someone
//! types, in the browser, and the backend normalises OCR output. Compiling the
//! same source for both is what stops the browser growing its own copy of the
//! rules — and, because the vocabulary is compiled in as well, its own copy of
//! the word lists.
//!
//! Everything here is a thin shell. No rule is decided in this file. A rule
//! that declines throws an `Error` whose name is `"Declined"` and whose
//! message is the core's sentence.

use js_sys::{Array, Object, Reflect};
use mangel::rules::{self, Category, CategoryVat};
use mangel::{Market, Vocabulary};
use wasm_bindgen::prelude::*;

fn declined(reason: rules::Declined) -> JsValue {
    let error = js_sys::Error::new(reason.reason());
    error.set_name("Declined");
    error.into()
}

fn market(code: &str) -> Result<Market, JsValue> {
    Market::from_code(code).map_err(|e| JsError::from(e).into())
}

fn set(object: &Object, key: &str, value: JsValue) {
    // Setting a plain property on a fresh object cannot fail.
    let _ = Reflect::set(object, &JsValue::from_str(key), &value);
}

fn category_object(category: &Category) -> Object {
    let out = Object::new();
    set(&out, "name", JsValue::from_str(category.name));
    set(&out, "group", JsValue::from_str(category.group));
    let vat = match category.vat {
        CategoryVat::Rate(rate) => JsValue::from(rate),
        CategoryVat::Manual => JsValue::NULL,
    };
    set(&out, "vat", vat);
    out
}

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

/// A size read into `{ amount, unit }`.
#[wasm_bindgen]
pub fn size(text: &str, market: &str) -> Result<Object, JsValue> {
    let read = rules::size(text, self::market(market)?).map_err(declined)?;
    let out = Object::new();
    set(&out, "amount", JsValue::from_str(&read.amount));
    set(&out, "unit", JsValue::from_str(read.unit));
    Ok(out)
}

/// A cleaned name, checked.
#[wasm_bindgen(js_name = cleanedName)]
pub fn cleaned_name(text: &str) -> Result<String, JsValue> {
    rules::cleaned_name(text).map_err(declined)
}

/// A category found from what was typed: `{ name, group, vat }`.
#[wasm_bindgen]
pub fn category(text: &str, market: &str) -> Result<Object, JsValue> {
    let found = rules::category(text, self::market(market)?).map_err(declined)?;
    Ok(category_object(&found))
}

/// Every category of the market, sorted by name.
#[wasm_bindgen]
pub fn categories(market: &str) -> Result<Array, JsValue> {
    let all = rules::categories(self::market(market)?);
    Ok(all
        .iter()
        .map(|c| JsValue::from(category_object(c)))
        .collect())
}

/// The VAT for a product in `category`, given what was typed in its field.
#[wasm_bindgen]
pub fn vat(category: &str, text: &str, market: &str) -> Result<u8, JsValue> {
    let market = self::market(market)?;
    let found = rules::category(category, market).map_err(declined)?;
    rules::vat(&found, text, market).map_err(declined)
}

/// A deposit, or `undefined` for an empty field.
#[wasm_bindgen]
pub fn deposit(text: &str, market: &str) -> Result<Option<u8>, JsValue> {
    rules::deposit(text, self::market(market)?).map_err(declined)
}
