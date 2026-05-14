use std::sync::OnceLock;

use regex::Regex;

use crate::detectors::{next_char, pii_match_from_byte_span, previous_char};
use crate::types::{DetectorConfig, PiiMatch, RedactionSettings};
use crate::validators::luhn;

const CREDIT_CARD_PATTERN: &str = r"(?:\d[ -]?){12,18}\d";
static CREDIT_CARD_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn detect(text: &str, config: &DetectorConfig, settings: &RedactionSettings) -> Vec<PiiMatch> {
    credit_card_regex()
        .find_iter(text)
        .filter_map(|candidate| {
            if !is_digit_boundary(text, candidate.start(), candidate.end()) {
                return None;
            }

            let digits: String = candidate
                .as_str()
                .chars()
                .filter(|character| character.is_ascii_digit())
                .collect();

            if luhn::is_valid(&digits) {
                Some(pii_match_from_byte_span(
                    config,
                    text,
                    candidate.start(),
                    candidate.end(),
                    settings,
                ))
            } else {
                None
            }
        })
        .collect()
}

fn credit_card_regex() -> &'static Regex {
    CREDIT_CARD_REGEX
        .get_or_init(|| Regex::new(CREDIT_CARD_PATTERN).expect("valid credit card regex"))
}

fn is_digit_boundary(text: &str, start: usize, end: usize) -> bool {
    previous_char(text, start).is_none_or(|character| !character.is_ascii_digit())
        && next_char(text, end).is_none_or(|character| !character.is_ascii_digit())
}
