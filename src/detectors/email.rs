use std::sync::OnceLock;

use regex::Regex;

use crate::detectors::detect_with_rust_regex;
use crate::types::{DetectorConfig, PiiMatch, RedactionSettings};

const EMAIL_PATTERN: &str = r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b";
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn detect(text: &str, config: &DetectorConfig, settings: &RedactionSettings) -> Vec<PiiMatch> {
    detect_with_rust_regex(config, email_regex(), text, settings)
}

fn email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| Regex::new(EMAIL_PATTERN).expect("valid email regex"))
}
