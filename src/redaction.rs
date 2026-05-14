use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::types::PiiMatch;

pub fn apply_redaction(text: &str, matches: &[PiiMatch]) -> PyResult<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0usize;

    for pii_match in matches {
        if pii_match.start < cursor
            || pii_match.end > chars.len()
            || pii_match.start > pii_match.end
        {
            return Err(PyValueError::new_err("invalid match span while redacting"));
        }

        output.extend(chars[cursor..pii_match.start].iter());
        output.push_str(&pii_match.replacement);
        cursor = pii_match.end;
    }

    output.extend(chars[cursor..].iter());
    Ok(output)
}
