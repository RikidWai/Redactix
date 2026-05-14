pub mod credit_card;
pub mod email;
pub mod phone;

use std::collections::HashMap;

use pyo3::prelude::*;
use regex::Regex;

use crate::redactor::{BuiltinDetector, replacement_for};
use crate::types::PiiMatch;

pub fn detect_builtin_detector(
    text: &str,
    detector: BuiltinDetector,
    placeholders: &HashMap<String, String>,
) -> Vec<PiiMatch> {
    match detector {
        BuiltinDetector::Email => email::detect(text, placeholders),
        BuiltinDetector::Phone => phone::detect(text, placeholders),
        BuiltinDetector::CreditCard => credit_card::detect(text, placeholders),
    }
}

pub fn detect_with_rust_regex(
    type_name: &str,
    regex: &Regex,
    text: &str,
    placeholders: &HashMap<String, String>,
) -> Vec<PiiMatch> {
    regex
        .find_iter(text)
        .filter(|regex_match| !regex_match.as_str().is_empty())
        .map(|regex_match| {
            pii_match_from_byte_span(
                type_name,
                text,
                regex_match.start(),
                regex_match.end(),
                placeholders,
            )
        })
        .collect()
}

pub fn pii_match_from_byte_span(
    type_name: &str,
    text: &str,
    start: usize,
    end: usize,
    placeholders: &HashMap<String, String>,
) -> PiiMatch {
    let (start_char, end_char) = char_span_from_byte_span(text, start, end);
    PiiMatch::new(
        type_name,
        start_char,
        end_char,
        text[start..end].to_string(),
        replacement_for(type_name, placeholders),
    )
}

pub fn detect_with_python_regex(
    py: Python<'_>,
    type_name: &str,
    regex: &Py<PyAny>,
    text: &str,
    placeholders: &HashMap<String, String>,
) -> PyResult<Vec<PiiMatch>> {
    let iterator = regex.bind(py).call_method1("finditer", (text,))?;
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

pub fn previous_char(text: &str, byte_index: usize) -> Option<char> {
    text[..byte_index].chars().next_back()
}

pub fn next_char(text: &str, byte_index: usize) -> Option<char> {
    text[byte_index..].chars().next()
}

pub fn sort_and_remove_overlaps(mut matches: Vec<PiiMatch>) -> Vec<PiiMatch> {
    matches.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.end.cmp(&left.end))
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

fn char_span_from_byte_span(text: &str, start: usize, end: usize) -> (usize, usize) {
    if text.is_ascii() {
        return (start, end);
    }

    (text[..start].chars().count(), text[..end].chars().count())
}
