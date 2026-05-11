pub mod credit_card;
pub mod email;
pub mod phone;

use std::collections::HashMap;

use pyo3::prelude::*;

use crate::redactor::replacement_for;
use crate::types::PiiMatch;

pub fn detect_builtin(
    text: &str,
    placeholders: &HashMap<String, String>,
    py: Python<'_>,
) -> PyResult<Vec<PiiMatch>> {
    let mut matches = Vec::new();
    matches.extend(email::detect(text, placeholders, py)?);
    matches.extend(phone::detect(text, placeholders, py)?);
    matches.extend(credit_card::detect(text, placeholders, py)?);
    Ok(matches)
}

pub fn detect_with_python_regex(
    py: Python<'_>,
    type_name: &str,
    pattern: &str,
    text: &str,
    placeholders: &HashMap<String, String>,
) -> PyResult<Vec<PiiMatch>> {
    let re = py.import("re")?;
    let compiled = re.call_method1("compile", (pattern,))?;
    let iterator = compiled.call_method1("finditer", (text,))?;
    let mut matches = Vec::new();

    for match_obj in iterator.try_iter()? {
        let match_obj = match_obj?;
        let start: usize = match_obj.call_method0("start")?.extract()?;
        let end: usize = match_obj.call_method0("end")?.extract()?;
        let matched_text: String = match_obj.call_method0("group")?.extract()?;

        if start == end {
            continue;
        }

        matches.push(PiiMatch::new(
            type_name,
            start,
            end,
            matched_text,
            replacement_for(type_name, placeholders),
        ));
    }

    Ok(matches)
}

pub fn sort_and_remove_overlaps(mut matches: Vec<PiiMatch>) -> Vec<PiiMatch> {
    matches.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.end.cmp(&left.end))
            .then_with(|| left.type_name.cmp(&right.type_name))
    });

    let mut filtered = Vec::new();
    let mut last_end = 0usize;

    for pii_match in matches {
        if pii_match.start >= last_end {
            last_end = pii_match.end;
            filtered.push(pii_match);
        }
    }

    filtered
}
