use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;

use crate::detectors::{next_char, pii_match_from_byte_span, previous_char};
use crate::types::PiiMatch;

const PHONE_PATTERN: &str = r"(?:\+?\d{1,3}[\s.-]+)?(?:\(?\d{3}\)?[\s.-]+)\d{3}[\s.-]+\d{4}";
static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn detect(text: &str, placeholders: &HashMap<String, String>) -> Vec<PiiMatch> {
    phone_regex()
        .find_iter(text)
        .filter(|regex_match| {
            is_left_phone_boundary(text, regex_match.start())
                && is_right_phone_boundary(text, regex_match.end())
        })
        .map(|regex_match| {
            pii_match_from_byte_span(
                "phone",
                text,
                regex_match.start(),
                regex_match.end(),
                placeholders,
            )
        })
        .collect()
}

fn phone_regex() -> &'static Regex {
    PHONE_REGEX.get_or_init(|| Regex::new(PHONE_PATTERN).expect("valid phone regex"))
}

fn is_left_phone_boundary(text: &str, start: usize) -> bool {
    previous_char(text, start)
        .is_none_or(|character| !is_word_character(character) && character != '+')
}

fn is_right_phone_boundary(text: &str, end: usize) -> bool {
    next_char(text, end).is_none_or(|character| !is_word_character(character))
}

fn is_word_character(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}
