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
    patterns: Vec<ActivePattern>,
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

enum ActivePattern {
    Builtin(BuiltinPattern),
    Custom(CustomPattern),
}

impl Redactor {
    pub fn new(
        py: Python<'_>,
        custom_patterns: Option<HashMap<String, String>>,
        placeholders: Option<HashMap<String, String>>,
        default_mode: RedactionMode,
        builtin_patterns: Option<Vec<String>>,
        default_patterns: bool,
    ) -> PyResult<Self> {
        if default_patterns && builtin_patterns.is_some() {
            return Err(PyValueError::new_err(
                "default_patterns=True cannot be combined with patterns",
            ));
        }

        let mut patterns: Vec<ActivePattern> = if default_patterns {
            BuiltinPattern::all()
                .into_iter()
                .map(ActivePattern::Builtin)
                .collect()
        } else {
            validate_builtin_patterns(builtin_patterns)?
                .into_iter()
                .map(ActivePattern::Builtin)
                .collect()
        };

        if let Some(custom_patterns) = custom_patterns {
            for (type_name, pattern) in custom_patterns {
                if type_name.trim().is_empty() {
                    return Err(PyValueError::new_err(
                        "custom pattern names cannot be empty",
                    ));
                }
                if contains_pattern(&patterns, &type_name) {
                    return Err(PyValueError::new_err(format!(
                        "pattern '{type_name}' already exists"
                    )));
                }
                validate_regex(py, &type_name, &pattern)?;
                patterns.push(ActivePattern::Custom(CustomPattern { type_name, pattern }));
            }
        }

        Ok(Self {
            patterns,
            placeholders: placeholders.unwrap_or_default(),
            default_mode,
        })
    }

    pub fn detect(&self, text: &str, py: Python<'_>) -> PyResult<Vec<PiiMatch>> {
        let mut matches = Vec::new();

        for pattern in &self.patterns {
            match pattern {
                ActivePattern::Builtin(builtin_pattern) => matches.extend(
                    detectors::detect_builtin(text, &[*builtin_pattern], &self.placeholders, py)?,
                ),
                ActivePattern::Custom(custom_pattern) => {
                    matches.extend(detectors::detect_with_python_regex(
                        py,
                        &custom_pattern.type_name,
                        &custom_pattern.pattern,
                        text,
                        &self.placeholders,
                    )?);
                }
            }
        }

        Ok(detectors::sort_and_remove_overlaps(matches))
    }

    pub fn detect_py(&self, py: Python<'_>, text: &str) -> PyResult<Vec<Py<PyAny>>> {
        self.detect(text, py)?
            .iter()
            .map(|pii_match| pii_match.to_py_dict(py))
            .collect()
    }

    pub fn add_pattern(
        &mut self,
        py: Python<'_>,
        type_name: String,
        pattern: String,
    ) -> PyResult<()> {
        validate_custom_pattern_name(&type_name)?;
        if contains_pattern(&self.patterns, &type_name) {
            return Err(PyValueError::new_err(format!(
                "pattern '{type_name}' already exists"
            )));
        }
        validate_regex(py, &type_name, &pattern)?;
        self.patterns
            .push(ActivePattern::Custom(CustomPattern { type_name, pattern }));
        Ok(())
    }

    pub fn replace_pattern(
        &mut self,
        py: Python<'_>,
        type_name: String,
        pattern: String,
    ) -> PyResult<()> {
        validate_custom_pattern_name(&type_name)?;
        let Some(index) = find_pattern_index(&self.patterns, &type_name) else {
            return Err(PyValueError::new_err(format!(
                "pattern '{type_name}' does not exist"
            )));
        };
        validate_regex(py, &type_name, &pattern)?;
        self.patterns[index] = ActivePattern::Custom(CustomPattern { type_name, pattern });
        Ok(())
    }

    pub fn remove_pattern(&mut self, type_name: &str) -> PyResult<()> {
        validate_custom_pattern_name(type_name)?;
        let Some(index) = find_pattern_index(&self.patterns, type_name) else {
            return Err(PyValueError::new_err(format!(
                "pattern '{type_name}' does not exist"
            )));
        };
        self.patterns.remove(index);
        Ok(())
    }
}

impl BuiltinPattern {
    pub fn all() -> Vec<Self> {
        vec![Self::Email, Self::Phone, Self::CreditCard]
    }

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

    fn name(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Phone => "phone",
            Self::CreditCard => "credit_card",
        }
    }
}

fn validate_builtin_patterns(patterns: Option<Vec<String>>) -> PyResult<Vec<BuiltinPattern>> {
    let Some(patterns) = patterns else {
        return Ok(Vec::new());
    };

    let mut validated = Vec::new();
    for pattern in patterns {
        let builtin_pattern = BuiltinPattern::from_name(&pattern)?;
        if validated
            .iter()
            .any(|existing: &BuiltinPattern| existing.name() == builtin_pattern.name())
        {
            return Err(PyValueError::new_err(format!(
                "pattern '{pattern}' already exists"
            )));
        }
        validated.push(builtin_pattern);
    }
    Ok(validated)
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

fn validate_custom_pattern_name(type_name: &str) -> PyResult<()> {
    if type_name.trim().is_empty() {
        return Err(PyValueError::new_err(
            "custom pattern names cannot be empty",
        ));
    }
    Ok(())
}

fn contains_pattern(patterns: &[ActivePattern], type_name: &str) -> bool {
    find_pattern_index(patterns, type_name).is_some()
}

fn find_pattern_index(patterns: &[ActivePattern], type_name: &str) -> Option<usize> {
    patterns.iter().position(|pattern| match pattern {
        ActivePattern::Builtin(builtin_pattern) => builtin_pattern.name() == type_name,
        ActivePattern::Custom(custom_pattern) => custom_pattern.type_name == type_name,
    })
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
