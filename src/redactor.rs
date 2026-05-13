use std::collections::HashMap;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::detectors;
use crate::types::PiiMatch;

#[derive(Clone, Copy, Debug)]
pub enum RedactionMode {
    Placeholder,
    Mask,
}

pub struct Redactor {
    builtin_patterns: Vec<BuiltinPattern>,
    custom_patterns: Vec<CustomPattern>,
    placeholders: HashMap<String, String>,
    pub default_mode: RedactionMode,
}

#[derive(Clone, Copy, Debug)]
pub enum BuiltinPattern {
    Email,
    Phone,
    CreditCard,
}

struct CustomPattern {
    type_name: String,
    pattern: String,
}

impl Redactor {
    pub fn new(
        py: Python<'_>,
        custom_patterns: Option<HashMap<String, String>>,
        placeholders: Option<HashMap<String, String>>,
        default_mode: RedactionMode,
        builtin_patterns: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let mut compiled_custom_patterns = Vec::new();

        if let Some(patterns) = custom_patterns {
            for (type_name, pattern) in patterns {
                if type_name.trim().is_empty() {
                    return Err(PyValueError::new_err(
                        "custom pattern names cannot be empty",
                    ));
                }
                validate_regex(py, &type_name, &pattern)?;
                compiled_custom_patterns.push(CustomPattern { type_name, pattern });
            }
        }

        Ok(Self {
            builtin_patterns: validate_builtin_patterns(builtin_patterns)?,
            custom_patterns: compiled_custom_patterns,
            placeholders: placeholders.unwrap_or_default(),
            default_mode,
        })
    }

    pub fn detect(&self, text: &str, py: Python<'_>) -> PyResult<Vec<PiiMatch>> {
        let mut matches =
            detectors::detect_builtin(text, &self.builtin_patterns, &self.placeholders, py)?;

        for custom_pattern in &self.custom_patterns {
            matches.extend(detectors::detect_with_python_regex(
                py,
                &custom_pattern.type_name,
                &custom_pattern.pattern,
                text,
                &self.placeholders,
            )?);
        }

        Ok(detectors::sort_and_remove_overlaps(matches))
    }

    pub fn detect_py(&self, py: Python<'_>, text: &str) -> PyResult<Vec<Py<PyAny>>> {
        self.detect(text, py)?
            .iter()
            .map(|pii_match| pii_match.to_py_dict(py))
            .collect()
    }
}

impl BuiltinPattern {
    pub fn from_name(name: &str) -> PyResult<Self> {
        match name {
            "email" => Ok(Self::Email),
            "phone" => Ok(Self::Phone),
            "credit_card" => Ok(Self::CreditCard),
            _ => Err(PyValueError::new_err(format!(
                "invalid built-in pattern '{name}'; expected one of: email, phone, credit_card"
            ))),
        }
    }
}

fn validate_builtin_patterns(patterns: Option<Vec<String>>) -> PyResult<Vec<BuiltinPattern>> {
    let patterns = patterns.unwrap_or_else(|| {
        vec![
            "email".to_string(),
            "phone".to_string(),
            "credit_card".to_string(),
        ]
    });

    patterns
        .iter()
        .map(|pattern| BuiltinPattern::from_name(pattern))
        .collect()
}

pub fn validate_mode(mode: &str) -> PyResult<RedactionMode> {
    match mode {
        "placeholder" => Ok(RedactionMode::Placeholder),
        "mask" => Ok(RedactionMode::Mask),
        _ => Err(PyValueError::new_err(format!(
            "invalid redaction mode '{mode}'; expected 'placeholder' or 'mask'"
        ))),
    }
}

pub fn default_placeholder(type_name: &str) -> String {
    format!("{{{{{}}}}}", type_name.to_ascii_uppercase())
}

pub fn replacement_for(type_name: &str, placeholders: &HashMap<String, String>) -> String {
    placeholders
        .get(type_name)
        .cloned()
        .unwrap_or_else(|| default_placeholder(type_name))
}

fn validate_regex(py: Python<'_>, type_name: &str, pattern: &str) -> PyResult<()> {
    let re = py.import("re")?;
    re.call_method1("compile", (pattern,))
        .map(|_| ())
        .map_err(|err| {
            PyValueError::new_err(format!(
                "invalid custom regex for '{type_name}': {}",
                err.value(py)
            ))
        })
}
