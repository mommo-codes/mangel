//! Python bindings for mangel.
//!
//! Thin by design: every answer comes from the core crate, and nothing here
//! decides anything about product data. If a rule would have to be written in
//! this file, it belongs in `mangel/` instead, where both runtimes get it.
//!
//! A rule that declines raises `mangel.Declined` — a `ValueError` whose
//! message is the core's sentence, unchanged.

use mangel::rules::{self, Category, CategoryVat};
use mangel::{Market, Vocabulary};
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
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
    Declined::new_err(reason.to_string())
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

#[pymodule]
fn _mangel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("Declined", m.py().get_type::<Declined>())?;
    m.add_function(wrap_pyfunction!(abbreviations, m)?)?;
    m.add_function(wrap_pyfunction!(size, m)?)?;
    m.add_function(wrap_pyfunction!(cleaned_name, m)?)?;
    m.add_function(wrap_pyfunction!(category, m)?)?;
    m.add_function(wrap_pyfunction!(categories, m)?)?;
    m.add_function(wrap_pyfunction!(vat, m)?)?;
    m.add_function(wrap_pyfunction!(deposit, m)?)
}
