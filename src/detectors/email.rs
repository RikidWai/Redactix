use std::collections::HashMap;

use pyo3::prelude::*;

use crate::detectors::detect_with_python_regex;
use crate::types::PiiMatch;

const EMAIL_PATTERN: &str = r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b";

pub fn detect(
    text: &str,
    placeholders: &HashMap<String, String>,
    py: Python<'_>,
) -> PyResult<Vec<PiiMatch>> {
    detect_with_python_regex(py, "email", EMAIL_PATTERN, text, placeholders)
}
