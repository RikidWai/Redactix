use std::collections::HashMap;

use pyo3::prelude::*;

use crate::detectors::detect_with_python_regex;
use crate::redactor::replacement_for;
use crate::types::PiiMatch;
use crate::validators::luhn;

const CREDIT_CARD_PATTERN: &str = r"(?<!\d)(?:\d[ -]?){12,18}\d(?!\d)";

pub fn detect(
    text: &str,
    placeholders: &HashMap<String, String>,
    py: Python<'_>,
) -> PyResult<Vec<PiiMatch>> {
    let candidates =
        detect_with_python_regex(py, "credit_card", CREDIT_CARD_PATTERN, text, placeholders)?;
    let replacement = replacement_for("credit_card", placeholders);

    Ok(candidates
        .into_iter()
        .filter_map(|candidate| {
            let digits: String = candidate
                .text
                .chars()
                .filter(|character| character.is_ascii_digit())
                .collect();

            if luhn::is_valid(&digits) {
                Some(PiiMatch::new(
                    "credit_card",
                    candidate.start,
                    candidate.end,
                    candidate.text,
                    replacement.clone(),
                ))
            } else {
                None
            }
        })
        .collect())
}
