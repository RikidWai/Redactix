mod detectors;
mod redaction;
mod redactor;
mod types;
mod validators;

use std::collections::HashMap;

use pyo3::prelude::*;

use redaction::apply_redaction;
use redactor::{RedactionMode, Redactor as InnerRedactor, validate_mode};

#[pyfunction]
fn detect(py: Python<'_>, text: &str) -> PyResult<Vec<Py<PyAny>>> {
    let redactor = InnerRedactor::new(py, None, None, RedactionMode::Placeholder, None, true)?;
    redactor.detect_py(py, text)
}

#[pyfunction]
#[pyo3(signature = (text, mode = "placeholder"))]
fn redact(py: Python<'_>, text: &str, mode: &str) -> PyResult<String> {
    let mode = validate_mode(mode)?;
    let redactor = InnerRedactor::new(py, None, None, RedactionMode::Placeholder, None, true)?;
    let matches = redactor.detect(text, py)?;
    apply_redaction(text, &matches, mode)
}

#[pyclass(name = "Redactor")]
struct PyRedactor {
    inner: InnerRedactor,
}

#[pymethods]
impl PyRedactor {
    #[new]
    #[pyo3(signature = (custom_detectors = None, placeholders = None, mode = "placeholder", detectors = None, default_detectors = false))]
    fn new(
        py: Python<'_>,
        custom_detectors: Option<HashMap<String, String>>,
        placeholders: Option<HashMap<String, String>>,
        mode: &str,
        detectors: Option<Vec<String>>,
        default_detectors: bool,
    ) -> PyResult<Self> {
        let mode = validate_mode(mode)?;
        Ok(Self {
            inner: InnerRedactor::new(
                py,
                custom_detectors,
                placeholders,
                mode,
                detectors,
                default_detectors,
            )?,
        })
    }

    fn detect(&self, py: Python<'_>, text: &str) -> PyResult<Vec<Py<PyAny>>> {
        self.inner.detect_py(py, text)
    }

    #[pyo3(signature = (text, mode = None))]
    fn redact(&self, py: Python<'_>, text: &str, mode: Option<&str>) -> PyResult<String> {
        let mode = match mode {
            Some(mode) => validate_mode(mode)?,
            None => self.inner.default_mode,
        };
        let matches = self.inner.detect(text, py)?;
        apply_redaction(text, &matches, mode)
    }

    fn add_detector(&mut self, py: Python<'_>, name: String, regex: String) -> PyResult<()> {
        self.inner.add_detector(py, name, regex)
    }

    fn replace_detector(&mut self, py: Python<'_>, name: String, regex: String) -> PyResult<()> {
        self.inner.replace_detector(py, name, regex)
    }

    fn remove_detector(&mut self, name: &str) -> PyResult<()> {
        self.inner.remove_detector(name)
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(detect, m)?)?;
    m.add_function(wrap_pyfunction!(redact, m)?)?;
    m.add_class::<PyRedactor>()?;
    Ok(())
}
