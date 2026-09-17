//! Python bindings for mangel.
//!
//! Thin by design: every answer comes from the core crate, and nothing here
//! decides anything about product data. If a rule would have to be written in
//! this file, it belongs in `mangel/` instead, where both runtimes get it.

use mangel::{Market, Vocabulary};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// A market's abbreviations, as a new dict: each word as it is written in
/// product data, mapped to the abbreviation the golden standard uses. Keys
/// are in sorted order.
///
/// Raises `ValueError` for a market code mangel has no conventions for.
#[pyfunction]
fn abbreviations<'py>(py: Python<'py>, market: &str) -> PyResult<Bound<'py, PyDict>> {
    let market = Market::from_code(market).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let table = PyDict::new(py);
    for (word, abbreviation) in Vocabulary::of(market).abbreviations {
        table.set_item(word, abbreviation)?;
    }
    Ok(table)
}

#[pymodule]
fn _mangel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(abbreviations, m)?)
}
