use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use regex::Regex;

use crate::detectors;
use crate::types::{DetectorConfig, MaskStrategy, PiiMatch, RedactionSettings};

const DEFAULT_PRIORITY: i32 = 100;

pub struct Redactor {
    detectors: Vec<ActiveDetector>,
    settings: RedactionSettings,
    next_order: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum BuiltinDetector {
    Email,
    Phone,
    CreditCard,
}

enum ActiveDetector {
    Builtin {
        detector: BuiltinDetector,
        config: DetectorConfig,
    },
    Custom {
        config: DetectorConfig,
        regex: Regex,
    },
}

impl Redactor {
    pub fn new(detectors: Option<Vec<String>>, settings: RedactionSettings) -> PyResult<Self> {
        let builtins = match detectors {
            Some(detectors) => validate_builtin_detectors(detectors)?,
            None => BuiltinDetector::all(),
        };

        let mut active_detectors = Vec::with_capacity(builtins.len());
        for builtin in builtins {
            active_detectors.push(ActiveDetector::Builtin {
                detector: builtin,
                config: DetectorConfig {
                    name: builtin.name().to_string(),
                    entity_type: builtin.entity_type().to_string(),
                    placeholder: None,
                    enabled: true,
                    priority: DEFAULT_PRIORITY,
                    order: active_detectors.len(),
                },
            });
        }

        let next_order = active_detectors.len();

        Ok(Self {
            detectors: active_detectors,
            settings,
            next_order,
        })
    }

    pub fn detect(&self, text: &str) -> PyResult<Vec<PiiMatch>> {
        let mut matches = Vec::new();

        for detector in &self.detectors {
            matches.extend(detector.detect(text, &self.settings)?);
        }

        Ok(detectors::resolve_overlaps(matches))
    }

    pub fn detect_py(&self, py: Python<'_>, text: &str) -> PyResult<Vec<Py<PyAny>>> {
        self.detect(text)?
            .iter()
            .map(|pii_match| pii_match.to_py_dict(py))
            .collect()
    }

    pub fn redact(&self, text: &str) -> PyResult<String> {
        let matches = self.detect(text)?;
        crate::redaction::apply_redaction(text, &matches)
    }

    pub fn register_detector(
        &mut self,
        name: String,
        pattern: String,
        placeholder: Option<String>,
        enabled: bool,
        priority: i32,
    ) -> PyResult<()> {
        let name = normalize_detector_name(&name)?;
        if BuiltinDetector::from_name(&name).is_ok() || self.contains_detector(&name) {
            return Err(PyValueError::new_err(format!(
                "detector '{name}' already exists"
            )));
        }

        let regex = Regex::new(&pattern).map_err(|err| {
            PyValueError::new_err(format!("invalid regex for detector '{name}': {err}"))
        })?;

        self.detectors.push(ActiveDetector::Custom {
            config: DetectorConfig {
                entity_type: entity_type_for_name(&name),
                name,
                placeholder,
                enabled,
                priority,
                order: self.next_order,
            },
            regex,
        });
        self.next_order += 1;
        Ok(())
    }

    fn contains_detector(&self, name: &str) -> bool {
        self.detectors
            .iter()
            .any(|detector| detector.config().name == name)
    }
}

impl ActiveDetector {
    fn config(&self) -> &DetectorConfig {
        match self {
            ActiveDetector::Builtin { config, .. } => config,
            ActiveDetector::Custom { config, .. } => config,
        }
    }

    fn detect(&self, text: &str, settings: &RedactionSettings) -> PyResult<Vec<PiiMatch>> {
        let config = self.config();
        if !config.enabled {
            return Ok(Vec::new());
        }

        match self {
            ActiveDetector::Builtin { detector, .. } => Ok(detectors::detect_builtin_detector(
                text, *detector, config, settings,
            )),
            ActiveDetector::Custom { regex, .. } => Ok(detectors::detect_with_rust_regex(
                config, regex, text, settings,
            )),
        }
    }
}

impl BuiltinDetector {
    pub fn all() -> Vec<Self> {
        vec![Self::Email, Self::Phone, Self::CreditCard]
    }

    pub fn from_name(name: &str) -> PyResult<Self> {
        match name {
            "email" => Ok(Self::Email),
            "phone" => Ok(Self::Phone),
            "credit_card" => Ok(Self::CreditCard),
            _ => Err(PyValueError::new_err(format!(
                "invalid built-in detector '{name}'; expected one of: email, phone, credit_card"
            ))),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Phone => "phone",
            Self::CreditCard => "credit_card",
        }
    }

    fn entity_type(self) -> &'static str {
        match self {
            Self::Email => "EMAIL",
            Self::Phone => "PHONE",
            Self::CreditCard => "CREDIT_CARD",
        }
    }
}

pub fn settings_from_parts(
    mask_strategy: &str,
    placeholder_format: &str,
    mask_char: &str,
    fixed_mask: &str,
) -> PyResult<RedactionSettings> {
    let mask_strategy = match mask_strategy {
        "placeholder" => MaskStrategy::Placeholder,
        "fixed" => MaskStrategy::Fixed,
        "length_preserving" => MaskStrategy::LengthPreserving,
        _ => {
            return Err(PyValueError::new_err(format!(
                "invalid mask_strategy '{mask_strategy}'; expected one of: placeholder, fixed, length_preserving"
            )));
        }
    };

    let mut mask_chars = mask_char.chars();
    let Some(mask_char) = mask_chars.next() else {
        if mask_strategy == MaskStrategy::LengthPreserving {
            return Err(PyValueError::new_err("mask_char cannot be empty"));
        }
        return Ok(RedactionSettings {
            mask_strategy,
            placeholder_format: placeholder_format.to_string(),
            mask_char: '*',
            fixed_mask: fixed_mask.to_string(),
        });
    };
    if mask_strategy == MaskStrategy::LengthPreserving && mask_chars.next().is_some() {
        return Err(PyValueError::new_err(
            "mask_char must contain exactly one character",
        ));
    }

    Ok(RedactionSettings {
        mask_strategy,
        placeholder_format: placeholder_format.to_string(),
        mask_char,
        fixed_mask: fixed_mask.to_string(),
    })
}

fn validate_builtin_detectors(detectors: Vec<String>) -> PyResult<Vec<BuiltinDetector>> {
    let mut validated = Vec::new();

    for detector in detectors {
        let normalized = normalize_detector_name(&detector)?;
        let builtin_detector = BuiltinDetector::from_name(&normalized)?;
        if validated
            .iter()
            .any(|existing: &BuiltinDetector| existing.name() == builtin_detector.name())
        {
            return Err(PyValueError::new_err(format!(
                "detector '{normalized}' already exists"
            )));
        }
        validated.push(builtin_detector);
    }

    Ok(validated)
}

fn normalize_detector_name(name: &str) -> PyResult<String> {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(PyValueError::new_err("detector names cannot be empty"));
    }

    let mut chars = normalized.chars();
    let first = chars.next().expect("validated non-empty detector name");
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(PyValueError::new_err(
            "detector names must contain only ASCII letters, numbers, and underscores, and cannot start with a number",
        ));
    }

    Ok(normalized)
}

fn entity_type_for_name(name: &str) -> String {
    name.to_ascii_uppercase()
}
