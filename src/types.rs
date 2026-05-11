use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Clone, Debug)]
pub struct PiiMatch {
    pub type_name: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub replacement: String,
}

impl PiiMatch {
    pub fn new(
        type_name: impl Into<String>,
        start: usize,
        end: usize,
        text: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            start,
            end,
            text: text.into(),
            replacement: replacement.into(),
        }
    }

    pub fn to_py_dict(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("type", &self.type_name)?;
        dict.set_item("start", self.start)?;
        dict.set_item("end", self.end)?;
        dict.set_item("text", &self.text)?;
        dict.set_item("replacement", &self.replacement)?;
        Ok(dict.into_any().unbind())
    }
}
