//! WebAssembly bindings for mangel.
//!
//! The reason this target exists: Name Scrubbing normalises while someone
//! types, in the browser, and the backend normalises OCR output. Compiling the
//! same source for both is what stops the browser growing its own copy of the
//! rules — and, because the vocabulary is compiled in as well, its own copy of
//! the word lists.
//!
//! Everything here is a thin shell. No rule is decided in this file. A rule
//! that declines throws an `Error` whose name is `"Declined"`, whose message
//! is the core's sentence, and whose `code` is the core's stable code.

use js_sys::{Array, Object, Reflect};
use mangel::rules::{self, Category, CategoryVat};
use mangel::{Context, Field, Language, Market, Neutral, Profile, Vocabulary};
use wasm_bindgen::prelude::*;

fn declined(reason: rules::Declined) -> JsValue {
    let error = js_sys::Error::new(reason.reason());
    error.set_name("Declined");
    set(&error, "code", JsValue::from_str(reason.code().as_str()));
    error.into()
}

fn refused(error: impl std::fmt::Display) -> JsValue {
    js_sys::Error::new(&error.to_string()).into()
}

/// A string property of `options`, if it is there.
fn option(options: &JsValue, key: &str) -> Option<String> {
    Reflect::get(options, &JsValue::from_str(key))
        .ok()
        .and_then(|value| value.as_string())
}

fn market(code: &str) -> Result<Market, JsValue> {
    Market::from_code(code).map_err(|e| JsError::from(e).into())
}

fn set(object: &JsValue, key: &str, value: JsValue) {
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

/// Read `text` as `field`. `options` holds `market` and `profile`, and
/// optionally `label` (an array of language codes), `output` and `category`.
/// Returns `{ value, neutral, changes }`.
#[wasm_bindgen]
pub fn read(field: &str, text: &str, options: JsValue) -> Result<Object, JsValue> {
    let field = Field::from_code(field).map_err(refused)?;
    let market = self::market(&option(&options, "market").unwrap_or_default())?;
    let profile =
        Profile::from_code(&option(&options, "profile").unwrap_or_default()).map_err(refused)?;
    let mut context = Context::new(market, profile);
    let label = Reflect::get(&options, &JsValue::from_str("label")).unwrap_or(JsValue::UNDEFINED);
    if Array::is_array(&label) {
        let languages = Array::from(&label)
            .iter()
            .map(|code| Language::from_code(&code.as_string().unwrap_or_default()).map_err(refused))
            .collect::<Result<Vec<_>, _>>()?;
        context = context.with_label(&languages);
    }
    if let Some(output) = option(&options, "output") {
        let language = Language::from_code(&output).map_err(refused)?;
        context = context.with_output(language).map_err(refused)?;
    }
    let category = option(&options, "category");
    let context = match &category {
        Some(category) => context.with_category(category),
        None => context,
    };
    let read = mangel::read(field, text, &context).map_err(declined)?;

    let neutral = Object::new();
    match &read.neutral {
        Neutral::Text(text) => {
            set(&neutral, "kind", JsValue::from_str("text"));
            set(&neutral, "text", JsValue::from_str(text));
        }
        Neutral::Size { amount, unit } => {
            set(&neutral, "kind", JsValue::from_str("size"));
            set(&neutral, "amount", JsValue::from_str(amount));
            set(&neutral, "unit", JsValue::from_str(unit.code()));
        }
        Neutral::Category(found) => {
            set(&neutral, "kind", JsValue::from_str("category"));
            set(&neutral, "category", category_object(found).into());
        }
        Neutral::Rate(rate) => {
            set(&neutral, "kind", JsValue::from_str("rate"));
            set(&neutral, "rate", JsValue::from(*rate));
        }
        Neutral::Deposit(amount) => {
            set(&neutral, "kind", JsValue::from_str("deposit"));
            set(
                &neutral,
                "amount",
                amount.map_or(JsValue::NULL, JsValue::from),
            );
        }
        _ => {
            return Err(refused(
                "this build of mangel does not know the value read; upgrade it",
            ))
        }
    }
    let changes: Array = read
        .changes
        .iter()
        .map(|change| JsValue::from_str(change.code()))
        .collect();
    let out = Object::new();
    set(&out, "value", JsValue::from_str(&read.value));
    set(&out, "neutral", neutral.into());
    set(&out, "changes", changes.into());
    Ok(out)
}

/// Every field `read` takes, by code.
#[wasm_bindgen]
pub fn fields() -> Array {
    Field::ALL
        .iter()
        .map(|field| JsValue::from_str(field.code()))
        .collect()
}

/// Every profile `read` takes, by code.
#[wasm_bindgen]
pub fn profiles() -> Array {
    Profile::ALL
        .iter()
        .map(|profile| JsValue::from_str(profile.code()))
        .collect()
}

/// Every language mangel reads: `{ code, name, output }`.
#[wasm_bindgen]
pub fn languages() -> Array {
    Language::ALL
        .iter()
        .map(|language| {
            let out = Object::new();
            set(&out, "code", JsValue::from_str(language.code()));
            set(&out, "name", JsValue::from_str(language.name()));
            set(&out, "output", JsValue::from_bool(language.is_output()));
            JsValue::from(out)
        })
        .collect()
}
