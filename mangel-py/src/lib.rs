//! Python bindings for mangel.
//!
//! Thin by design: every answer comes from the core crate, and nothing here
//! decides anything about product data. If a rule would have to be written in
//! this file, it belongs in `mangel/` instead, where both runtimes get it.
//!
//! A rule that declines raises `mangel.Declined` — a `ValueError` whose
//! message is the core's sentence, unchanged, and whose `code` is the core's
//! stable code.

use mangel::rules::{self, Category, CategoryVat};
use mangel::{Context, Field, Language, Market, Neutral, Profile, Vocabulary};
use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

create_exception!(
    _mangel,
    Declined,
    PyValueError,
    "A field mangel would not read. The message says why."
);

fn market(code: &str) -> PyResult<Market> {
    Market::from_code(code).map_err(|e| PyValueError::new_err(e.to_string()))
}

fn declined(reason: rules::Declined) -> PyErr {
    let error = Declined::new_err(reason.to_string());
    Python::attach(|py| {
        // Setting an attribute on a fresh exception instance cannot fail.
        let _ = error.value(py).setattr("code", reason.code().as_str());
    });
    error
}

fn refused(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn category_dict<'py>(py: Python<'py>, category: &Category) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("name", category.name)?;
    out.set_item("group", category.group)?;
    match category.vat {
        CategoryVat::Rate(rate) => out.set_item("vat", rate)?,
        CategoryVat::Manual => out.set_item("vat", py.None())?,
    }
    Ok(out)
}

/// A market's abbreviations, as a new dict: each word as it is written in
/// product data, mapped to the abbreviation the golden standard uses. Keys
/// are in sorted order.
///
/// Raises `ValueError` for a market code mangel has no conventions for.
#[pyfunction]
fn abbreviations<'py>(py: Python<'py>, market: &str) -> PyResult<Bound<'py, PyDict>> {
    let market = self::market(market)?;
    let table = PyDict::new(py);
    for (word, abbreviation) in Vocabulary::of(market).abbreviations {
        table.set_item(word, abbreviation)?;
    }
    Ok(table)
}

/// A size read into `(amount, unit)`: `"56kg"` is `("56", "kg")`.
#[pyfunction]
fn size(text: &str, market: &str) -> PyResult<(String, &'static str)> {
    let read = rules::size(text, self::market(market)?).map_err(declined)?;
    Ok((read.amount, read.unit))
}

/// A cleaned name, checked: trimmed, and starting with a capital where its
/// script has capitals.
#[pyfunction]
fn cleaned_name(text: &str) -> PyResult<String> {
    rules::cleaned_name(text).map_err(declined)
}

/// A category found from what was typed: `{"name", "group", "vat"}`, where
/// `vat` is `None` for a category whose VAT is typed per product.
#[pyfunction]
fn category<'py>(py: Python<'py>, text: &str, market: &str) -> PyResult<Bound<'py, PyDict>> {
    let found = rules::category(text, self::market(market)?).map_err(declined)?;
    category_dict(py, &found)
}

/// Every category of the market, sorted by name, as `category` returns one.
#[pyfunction]
fn categories<'py>(py: Python<'py>, market: &str) -> PyResult<Vec<Bound<'py, PyDict>>> {
    rules::categories(self::market(market)?)
        .iter()
        .map(|category| category_dict(py, category))
        .collect()
}

/// The VAT for a product in `category` (found as `category` finds one),
/// given what was typed in its VAT field.
#[pyfunction]
fn vat(category: &str, text: &str, market: &str) -> PyResult<u8> {
    let market = self::market(market)?;
    let found = rules::category(category, market).map_err(declined)?;
    rules::vat(&found, text, market).map_err(declined)
}

/// A deposit: one of the market's amounts, or `None` for an empty field.
#[pyfunction]
fn deposit(text: &str, market: &str) -> PyResult<Option<u8>> {
    rules::deposit(text, self::market(market)?).map_err(declined)
}

/// Read `text` as `field`, for `profile`, in `market`. Returns a new dict:
/// `value` (as the market writes it), `neutral` (the value without a
/// language), and `changes` (what was changed, as codes).
#[pyfunction]
#[pyo3(signature = (field, text, *, market, profile, label=None, output=None, category=None))]
#[allow(clippy::too_many_arguments)]
fn read<'py>(
    py: Python<'py>,
    field: &str,
    text: &str,
    market: &str,
    profile: &str,
    label: Option<Vec<String>>,
    output: Option<&str>,
    category: Option<&str>,
) -> PyResult<Bound<'py, PyDict>> {
    let field = Field::from_code(field).map_err(refused)?;
    let profile = Profile::from_code(profile).map_err(refused)?;
    let mut context = Context::new(self::market(market)?, profile);
    if let Some(label) = label {
        let languages = label
            .iter()
            .map(|code| Language::from_code(code).map_err(refused))
            .collect::<PyResult<Vec<_>>>()?;
        context = context.with_label(&languages);
    }
    if let Some(output) = output {
        let language = Language::from_code(output).map_err(refused)?;
        context = context.with_output(language).map_err(refused)?;
    }
    let context = match category {
        Some(category) => context.with_category(category),
        None => context,
    };
    let read = mangel::read(field, text, &context).map_err(declined)?;

    let out = PyDict::new(py);
    out.set_item("value", &read.value)?;
    let neutral = PyDict::new(py);
    match &read.neutral {
        Neutral::Text(text) => {
            neutral.set_item("kind", "text")?;
            neutral.set_item("text", text)?;
        }
        Neutral::Size { amount, unit } => {
            neutral.set_item("kind", "size")?;
            neutral.set_item("amount", amount)?;
            neutral.set_item("unit", unit.code())?;
        }
        Neutral::Category(found) => {
            neutral.set_item("kind", "category")?;
            neutral.set_item("category", category_dict(py, found)?)?;
        }
        Neutral::Rate(rate) => {
            neutral.set_item("kind", "rate")?;
            neutral.set_item("rate", rate)?;
        }
        Neutral::Deposit(amount) => {
            neutral.set_item("kind", "deposit")?;
            neutral.set_item("amount", amount)?;
        }
        _ => {
            return Err(PyRuntimeError::new_err(
                "this build of mangel-py does not know the value read; upgrade it",
            ))
        }
    }
    out.set_item("neutral", neutral)?;
    let changes: Vec<&str> = read.changes.iter().map(|change| change.code()).collect();
    out.set_item("changes", changes)?;
    Ok(out)
}

/// Every field `read` takes, by code.
#[pyfunction]
fn fields() -> Vec<&'static str> {
    Field::ALL.iter().map(|field| field.code()).collect()
}

/// Every profile `read` takes, by code.
#[pyfunction]
fn profiles() -> Vec<&'static str> {
    Profile::ALL.iter().map(|profile| profile.code()).collect()
}

/// Every language mangel reads, as dicts: `code`, `name` (in English), and
/// `output` (whether a value can be named in it).
#[pyfunction]
fn languages(py: Python<'_>) -> PyResult<Vec<Bound<'_, PyDict>>> {
    Language::ALL
        .iter()
        .map(|language| {
            let out = PyDict::new(py);
            out.set_item("code", language.code())?;
            out.set_item("name", language.name())?;
            out.set_item("output", language.is_output())?;
            Ok(out)
        })
        .collect()
}

#[pymodule]
fn _mangel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("Declined", m.py().get_type::<Declined>())?;
    m.add_function(wrap_pyfunction!(abbreviations, m)?)?;
    m.add_function(wrap_pyfunction!(size, m)?)?;
    m.add_function(wrap_pyfunction!(cleaned_name, m)?)?;
    m.add_function(wrap_pyfunction!(category, m)?)?;
    m.add_function(wrap_pyfunction!(categories, m)?)?;
    m.add_function(wrap_pyfunction!(vat, m)?)?;
    m.add_function(wrap_pyfunction!(deposit, m)?)?;
    m.add_function(wrap_pyfunction!(read, m)?)?;
    m.add_function(wrap_pyfunction!(fields, m)?)?;
    m.add_function(wrap_pyfunction!(profiles, m)?)?;
    m.add_function(wrap_pyfunction!(languages, m)?)
}
