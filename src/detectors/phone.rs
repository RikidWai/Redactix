use std::collections::HashMap;

use pyo3::prelude::*;

use crate::detectors::detect_with_python_regex;
use crate::types::PiiMatch;

const PHONE_PATTERN: &str =
    r"(?<![\w+])(?:\+?\d{1,3}[\s.-]+)?(?:\(?\d{3}\)?[\s.-]+)\d{3}[\s.-]+\d{4}(?!\w)";

pub fn detect(
    text: &str,
    placeholders: &HashMap<String, String>,
    py: Python<'_>,
) -> PyResult<Vec<PiiMatch>> {
    detect_with_python_regex(py, "phone", PHONE_PATTERN, text, placeholders)
}
