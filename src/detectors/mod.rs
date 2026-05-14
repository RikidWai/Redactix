pub mod credit_card;
pub mod email;
pub mod phone;

use regex::Regex;

use crate::redactor::BuiltinDetector;
use crate::types::{DetectorConfig, PiiMatch, RedactionSettings};

pub fn detect_builtin_detector(
    text: &str,
    detector: BuiltinDetector,
    config: &DetectorConfig,
    settings: &RedactionSettings,
) -> Vec<PiiMatch> {
    match detector {
        BuiltinDetector::Email => email::detect(text, config, settings),
        BuiltinDetector::Phone => phone::detect(text, config, settings),
        BuiltinDetector::CreditCard => credit_card::detect(text, config, settings),
    }
}

pub fn detect_with_rust_regex(
    config: &DetectorConfig,
    regex: &Regex,
    text: &str,
    settings: &RedactionSettings,
) -> Vec<PiiMatch> {
    regex
        .find_iter(text)
        .filter(|regex_match| !regex_match.as_str().is_empty())
        .map(|regex_match| {
            pii_match_from_byte_span(
                config,
                text,
                regex_match.start(),
                regex_match.end(),
                settings,
            )
        })
        .collect()
}

pub fn pii_match_from_byte_span(
    config: &DetectorConfig,
    text: &str,
    start: usize,
    end: usize,
    settings: &RedactionSettings,
) -> PiiMatch {
    let (start_char, end_char) = char_span_from_byte_span(text, start, end);
    let value = text[start..end].to_string();
    let replacement = settings.replacement_for(config, &value);

    PiiMatch::new(
        start_char,
        end_char,
        value,
        config.entity_type.clone(),
        config.name.clone(),
        replacement,
        config.priority,
        config.order,
    )
}

pub fn previous_char(text: &str, byte_index: usize) -> Option<char> {
    text[..byte_index].chars().next_back()
}

pub fn next_char(text: &str, byte_index: usize) -> Option<char> {
    text[byte_index..].chars().next()
}

pub fn resolve_overlaps(mut matches: Vec<PiiMatch>) -> Vec<PiiMatch> {
    matches.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| right.len().cmp(&left.len()))
            .then_with(|| left.start.cmp(&right.start))
            .then_with(|| left.end.cmp(&right.end))
            .then_with(|| left.order.cmp(&right.order))
            .then_with(|| left.detector_name.cmp(&right.detector_name))
    });

    let mut filtered = Vec::new();

    for pii_match in matches {
        if filtered
            .iter()
            .all(|selected| !ranges_overlap(selected, &pii_match))
        {
            filtered.push(pii_match);
        }
    }

    filtered.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| left.end.cmp(&right.end))
    });

    filtered
}

fn ranges_overlap(left: &PiiMatch, right: &PiiMatch) -> bool {
    left.start < right.end && right.start < left.end
}

fn char_span_from_byte_span(text: &str, start: usize, end: usize) -> (usize, usize) {
    if text.is_ascii() {
        return (start, end);
    }

    (text[..start].chars().count(), text[..end].chars().count())
}
