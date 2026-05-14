mod detectors;
mod redaction;
mod redactor;
mod types;
mod validators;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use redactor::{Redactor as InnerRedactor, settings_from_parts};

#[pyfunction]
#[pyo3(signature = (text, detectors = None, mask_strategy = "placeholder", placeholder_format = "{{{entity_type}}}", mask_char = "*", fixed_mask = "***"))]
fn detect(
    py: Python<'_>,
    text: &str,
    detectors: Option<Vec<String>>,
    mask_strategy: &str,
    placeholder_format: &str,
    mask_char: &str,
    fixed_mask: &str,
) -> PyResult<Vec<Py<PyAny>>> {
    let redactor = build_redactor(
        detectors,
        mask_strategy,
        placeholder_format,
        mask_char,
        fixed_mask,
    )?;
    redactor.detect_py(py, text)
}

#[pyfunction]
#[pyo3(signature = (text, detectors = None, mask_strategy = "placeholder", placeholder_format = "{{{entity_type}}}", mask_char = "*", fixed_mask = "***"))]
fn redact(
    text: &str,
    detectors: Option<Vec<String>>,
    mask_strategy: &str,
    placeholder_format: &str,
    mask_char: &str,
    fixed_mask: &str,
) -> PyResult<String> {
    let redactor = build_redactor(
        detectors,
        mask_strategy,
        placeholder_format,
        mask_char,
        fixed_mask,
    )?;
    redactor.redact(text)
}

#[pyfunction]
#[pyo3(signature = (text, detectors = None, mask_strategy = "placeholder", placeholder_format = "{{{entity_type}}}", mask_char = "*", fixed_mask = "***"))]
fn redact_with_report(
    py: Python<'_>,
    text: &str,
    detectors: Option<Vec<String>>,
    mask_strategy: &str,
    placeholder_format: &str,
    mask_char: &str,
    fixed_mask: &str,
) -> PyResult<Py<PyAny>> {
    let redactor = build_redactor(
        detectors,
        mask_strategy,
        placeholder_format,
        mask_char,
        fixed_mask,
    )?;
    redact_with_report_py(py, &redactor, text)
}

#[pyclass(name = "Redactor")]
struct PyRedactor {
    inner: InnerRedactor,
}

#[pymethods]
impl PyRedactor {
    #[new]
    #[pyo3(signature = (detectors = None, mask_strategy = "placeholder", placeholder_format = "{{{entity_type}}}", mask_char = "*", fixed_mask = "***"))]
    fn new(
        detectors: Option<Vec<String>>,
        mask_strategy: &str,
        placeholder_format: &str,
        mask_char: &str,
        fixed_mask: &str,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: build_redactor(
                detectors,
                mask_strategy,
                placeholder_format,
                mask_char,
                fixed_mask,
            )?,
        })
    }

    fn detect(&self, py: Python<'_>, text: &str) -> PyResult<Vec<Py<PyAny>>> {
        self.inner.detect_py(py, text)
    }

    fn redact(&self, text: &str) -> PyResult<String> {
        self.inner.redact(text)
    }

    fn redact_with_report(&self, py: Python<'_>, text: &str) -> PyResult<Py<PyAny>> {
        redact_with_report_py(py, &self.inner, text)
    }

    #[pyo3(signature = (name, pattern, placeholder = None, enabled = true, priority = 100))]
    fn register_detector(
        &mut self,
        name: String,
        pattern: String,
        placeholder: Option<String>,
        enabled: bool,
        priority: i32,
    ) -> PyResult<()> {
        self.inner
            .register_detector(name, pattern, placeholder, enabled, priority)
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(detect, m)?)?;
    m.add_function(wrap_pyfunction!(redact, m)?)?;
    m.add_function(wrap_pyfunction!(redact_with_report, m)?)?;
    m.add_class::<PyRedactor>()?;
    Ok(())
}

fn build_redactor(
    detectors: Option<Vec<String>>,
    mask_strategy: &str,
    placeholder_format: &str,
    mask_char: &str,
    fixed_mask: &str,
) -> PyResult<InnerRedactor> {
    let settings = settings_from_parts(mask_strategy, placeholder_format, mask_char, fixed_mask)?;
    InnerRedactor::new(detectors, settings)
}

fn redact_with_report_py(
    py: Python<'_>,
    redactor: &InnerRedactor,
    text: &str,
) -> PyResult<Py<PyAny>> {
    let detections = redactor.detect(text)?;
    let redacted = crate::redaction::apply_redaction(text, &detections)?;
    let detection_dicts = detections
        .iter()
        .map(|pii_match| pii_match.to_py_dict(py))
        .collect::<PyResult<Vec<_>>>()?;

    let report = PyDict::new(py);
    report.set_item("text", redacted)?;
    report.set_item("detections", detection_dicts)?;
    Ok(report.into_any().unbind())
}
