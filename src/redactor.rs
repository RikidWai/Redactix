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
    detectors: Vec<ActiveDetector>,
    placeholders: HashMap<String, String>,
    pub default_mode: RedactionMode,
}

#[derive(Clone, Copy, Debug)]
pub enum BuiltinDetector {
    Email,
    Phone,
    CreditCard,
}

struct CustomDetector {
    type_name: String,
    regex: Py<PyAny>,
}

enum ActiveDetector {
    Builtin(BuiltinDetector),
    Custom(CustomDetector),
}

impl Redactor {
    pub fn new(
        py: Python<'_>,
        custom_detectors: Option<HashMap<String, String>>,
        placeholders: Option<HashMap<String, String>>,
        default_mode: RedactionMode,
        builtin_detectors: Option<Vec<String>>,
        default_detectors: bool,
    ) -> PyResult<Self> {
        if default_detectors && builtin_detectors.is_some() {
            return Err(PyValueError::new_err(
                "default_detectors=True cannot be combined with detectors",
            ));
        }

        let mut detectors: Vec<ActiveDetector> = if default_detectors {
            BuiltinDetector::all()
                .into_iter()
                .map(ActiveDetector::Builtin)
                .collect()
        } else {
            validate_builtin_detectors(builtin_detectors)?
                .into_iter()
                .map(ActiveDetector::Builtin)
                .collect()
        };

        if let Some(custom_detectors) = custom_detectors {
            for (type_name, regex) in custom_detectors {
                if contains_detector(&detectors, &type_name) {
                    return Err(PyValueError::new_err(format!(
                        "detector '{type_name}' already exists"
                    )));
                }
                detectors.push(ActiveDetector::Custom(CustomDetector::new(
                    py, type_name, regex,
                )?));
            }
        }

        Ok(Self {
            detectors,
            placeholders: placeholders.unwrap_or_default(),
            default_mode,
        })
    }

    pub fn detect(&self, text: &str, py: Python<'_>) -> PyResult<Vec<PiiMatch>> {
        let mut matches = Vec::new();

        for detector in &self.detectors {
            matches.extend(detector.detect(text, py, &self.placeholders)?);
        }

        Ok(detectors::sort_and_remove_overlaps(matches))
    }

    pub fn detect_py(&self, py: Python<'_>, text: &str) -> PyResult<Vec<Py<PyAny>>> {
        self.detect(text, py)?
            .iter()
            .map(|pii_match| pii_match.to_py_dict(py))
            .collect()
    }

    pub fn add_detector(
        &mut self,
        py: Python<'_>,
        type_name: String,
        regex: String,
    ) -> PyResult<()> {
        validate_custom_detector_name(&type_name)?;
        if contains_detector(&self.detectors, &type_name) {
            return Err(PyValueError::new_err(format!(
                "detector '{type_name}' already exists"
            )));
        }
        let custom_detector = CustomDetector::new(py, type_name, regex)?;
        self.detectors.push(ActiveDetector::Custom(custom_detector));
        Ok(())
    }

    pub fn replace_detector(
        &mut self,
        py: Python<'_>,
        type_name: String,
        regex: String,
    ) -> PyResult<()> {
        validate_custom_detector_name(&type_name)?;
        let Some(index) = find_detector_index(&self.detectors, &type_name) else {
            return Err(PyValueError::new_err(format!(
                "detector '{type_name}' does not exist"
            )));
        };
        self.detectors[index] = ActiveDetector::Custom(CustomDetector::new(py, type_name, regex)?);
        Ok(())
    }

    pub fn remove_detector(&mut self, type_name: &str) -> PyResult<()> {
        validate_custom_detector_name(type_name)?;
        let Some(index) = find_detector_index(&self.detectors, type_name) else {
            return Err(PyValueError::new_err(format!(
                "detector '{type_name}' does not exist"
            )));
        };
        self.detectors.remove(index);
        Ok(())
    }
}

impl CustomDetector {
    fn new(py: Python<'_>, type_name: String, regex: String) -> PyResult<Self> {
        validate_custom_detector_name(&type_name)?;
        Ok(Self {
            regex: compile_regex(py, &type_name, &regex)?,
            type_name,
        })
    }
}

impl ActiveDetector {
    fn type_name(&self) -> &str {
        match self {
            ActiveDetector::Builtin(builtin_detector) => builtin_detector.name(),
            ActiveDetector::Custom(custom_detector) => &custom_detector.type_name,
        }
    }

    fn detect(
        &self,
        text: &str,
        py: Python<'_>,
        placeholders: &HashMap<String, String>,
    ) -> PyResult<Vec<PiiMatch>> {
        match self {
            ActiveDetector::Builtin(builtin_detector) => Ok(detectors::detect_builtin_detector(
                text,
                *builtin_detector,
                placeholders,
            )),
            ActiveDetector::Custom(custom_detector) => detectors::detect_with_python_regex(
                py,
                &custom_detector.type_name,
                &custom_detector.regex,
                text,
                placeholders,
            ),
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
}

fn validate_builtin_detectors(detectors: Option<Vec<String>>) -> PyResult<Vec<BuiltinDetector>> {
    let Some(detectors) = detectors else {
        return Ok(Vec::new());
    };

    let mut validated = Vec::new();
    for detector in detectors {
        let builtin_detector = BuiltinDetector::from_name(&detector)?;
        if validated
            .iter()
            .any(|existing: &BuiltinDetector| existing.name() == builtin_detector.name())
        {
            return Err(PyValueError::new_err(format!(
                "detector '{detector}' already exists"
            )));
        }
        validated.push(builtin_detector);
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

fn validate_custom_detector_name(type_name: &str) -> PyResult<()> {
    if type_name.trim().is_empty() {
        return Err(PyValueError::new_err(
            "custom detector names cannot be empty",
        ));
    }
    Ok(())
}

fn contains_detector(detectors: &[ActiveDetector], type_name: &str) -> bool {
    find_detector_index(detectors, type_name).is_some()
}

fn find_detector_index(detectors: &[ActiveDetector], type_name: &str) -> Option<usize> {
    detectors
        .iter()
        .position(|detector| detector.type_name() == type_name)
}

fn compile_regex(py: Python<'_>, type_name: &str, regex: &str) -> PyResult<Py<PyAny>> {
    let re = py.import("re")?;
    re.call_method1("compile", (regex,))
        .map(|compiled| compiled.unbind())
        .map_err(|err| {
            PyValueError::new_err(format!(
                "invalid custom regex for '{type_name}': {}",
                err.value(py)
            ))
        })
}
