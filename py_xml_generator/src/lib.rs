use std::fmt;
use pyo3::prelude::*;
use pyo3::exceptions::PyOSError;
use xmlgenerator::{generate_xml, XMLGeneratorError};

#[derive(Debug)]
struct PyXMLGeneratorError;

impl std::error::Error for PyXMLGeneratorError {}

impl fmt::Display for PyXMLGeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Oh no!")
    }
}

impl From<XMLGeneratorError> for PyXMLGeneratorError {
    fn from(err: XMLGeneratorError) -> Self {
        match err {
            XMLGeneratorError::XSDParserError(_) => PyXMLGeneratorError,
            XMLGeneratorError::DataTypesFormatError(_) => PyXMLGeneratorError,
            XMLGeneratorError::XMLBuilderError(_) => PyXMLGeneratorError,
        }
    }
}

impl std::convert::From<PyXMLGeneratorError> for PyErr {
    fn from(err: PyXMLGeneratorError) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn generate(xsd_string: String) -> PyResult<String> {
    let result = generate_xml(&xsd_string);
    match result {
        Ok(xml_string) => Ok(xml_string),
        Err(e) => {
            let error: PyXMLGeneratorError = From::from(e);
            Err(PyOSError::new_err(error.to_string()))
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyxmlgenerator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    Ok(())
}
