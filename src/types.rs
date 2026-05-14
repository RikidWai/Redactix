use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaskStrategy {
    Placeholder,
    Fixed,
    LengthPreserving,
}

#[derive(Clone, Debug)]
pub struct RedactionSettings {
    pub mask_strategy: MaskStrategy,
    pub placeholder_format: String,
    pub mask_char: char,
    pub fixed_mask: String,
}

#[derive(Clone, Debug)]
pub struct DetectorConfig {
    pub name: String,
    pub entity_type: String,
    pub placeholder: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub order: usize,
}

#[derive(Clone, Debug)]
pub struct PiiMatch {
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub entity_type: String,
    pub detector_name: String,
    pub replacement: String,
    pub priority: i32,
    pub order: usize,
}

impl PiiMatch {
    pub fn new(
        start: usize,
        end: usize,
        value: impl Into<String>,
        entity_type: impl Into<String>,
        detector_name: impl Into<String>,
        replacement: impl Into<String>,
        priority: i32,
        order: usize,
    ) -> Self {
        Self {
            start,
            end,
            value: value.into(),
            entity_type: entity_type.into(),
            detector_name: detector_name.into(),
            replacement: replacement.into(),
            priority,
            order,
        }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn to_py_dict(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("start", self.start)?;
        dict.set_item("end", self.end)?;
        dict.set_item("value", &self.value)?;
        dict.set_item("entity_type", &self.entity_type)?;
        dict.set_item("detector_name", &self.detector_name)?;
        dict.set_item("replacement", &self.replacement)?;
        Ok(dict.into_any().unbind())
    }
}

impl RedactionSettings {
    pub fn replacement_for(&self, detector: &DetectorConfig, value: &str) -> String {
        match self.mask_strategy {
            MaskStrategy::Placeholder => detector.placeholder.clone().unwrap_or_else(|| {
                self.placeholder_format
                    .replace("{entity_type}", &detector.entity_type)
                    .replace("{detector_name}", &detector.name)
            }),
            MaskStrategy::Fixed => self.fixed_mask.clone(),
            MaskStrategy::LengthPreserving => {
                self.mask_char.to_string().repeat(value.chars().count())
            }
        }
    }
}
