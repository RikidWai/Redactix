use std::sync::OnceLock;

use regex::Regex;

use crate::detectors::{next_char, pii_match_from_byte_span, previous_char};
use crate::types::{DetectorConfig, PiiMatch, RedactionSettings};

const PHONE_PATTERN: &str = r"(?:\+?\d{1,3}[\s.-]+)?(?:\(?\d{3}\)?[\s.-]+)\d{3}[\s.-]+\d{4}";
static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn detect(text: &str, config: &DetectorConfig, settings: &RedactionSettings) -> Vec<PiiMatch> {
    phone_regex()
        .find_iter(text)
        .filter(|regex_match| {
            is_left_phone_boundary(text, regex_match.start())
                && is_right_phone_boundary(text, regex_match.end())
                && is_plausible_phone(regex_match.as_str())
        })
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

fn is_plausible_phone(value: &str) -> bool {
    let digits: Vec<char> = value
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();

    if digits.len() < 10 || digits.len() > 15 {
        return false;
    }

    !digits.iter().all(|digit| *digit == digits[0])
}
